use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};

use cmd_lib::{run_cmd, spawn_with_output};
use lazy_static::lazy_static;
use retry::delay::Fixed;

mod dto;
mod utils;

lazy_static! {
    static ref TRACKING_DIR: String = dotenv::var("TRACKING_DIR").unwrap_or_default();
    static ref BUILDING_DIR: String = dotenv::var("BUILDING_DIR").unwrap_or_default();
    static ref BACKEND_HOST: String = dotenv::var("BACKEND_HOST").unwrap_or_default();
    static ref SLEEP_TIME_SEC: u64 = dotenv::var("SLEEP_TIME_SEC")
        .unwrap_or_default()
        .parse()
        .unwrap_or(10);
    static ref INITIAL_DIR: String = std::env::current_dir()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
}

fn monitor(
    repos_map_lock: Arc<RwLock<HashMap<i32, dto::Repo>>>,
    repo_id: i32,
    last_run_success: bool,
) -> anyhow::Result<bool> {
    let repos_map = repos_map_lock
        .read()
        .expect("error unlock repos map for read");
    if !repos_map.contains_key(&repo_id) {
        run_cmd!(
            rm -rf $TRACKING_DIR/$repo_id;
            rm -rf $BUILDING_DIR/$repo_id;
        )?;

        return Ok(false);
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap_or_default();
    let monitor_path = Path::new(&*TRACKING_DIR).join(format!("{}", repo_id));
    let mut info = utils::parse_git_uri(&repos_map[&repo_id].uri)?;
    let monitor_path_str = monitor_path.to_str().unwrap_or_default();
    if repos_map[&repo_id].private_repo {
        let git_basic_auth = utils::get_git_basic_auth(
            &repos_map[&repo_id].repo_credential_secret,
            &repos_map[&repo_id].secret_namespace,
        )?;
        (*info.get_mut("repo").unwrap()) = format!(
            "{}://{}:{}@{}",
            &info["protocol"], &git_basic_auth.username, &git_basic_auth.password, &info["repo"]
        );
    } else {
        (*info.get_mut("repo").unwrap()) = format!(
            "{}://{}",
            &info["protocol"], &info["repo"]
        );
    }
    if monitor_path.is_dir() {
        if !last_run_success
            || should_build(
                &client,
                &repo_id,
                &monitor_path_str,
                &info["ref"],
                &info["pathspec"],
            )?
        {
            run_cmd!(rm -rf $monitor_path_str)?;
            build(
                &client,
                &repos_map[&repo_id],
                &info["repo"],
                &info["ref"],
                &info["pathspec"],
            )?;
        }
    } else {
        let repo = &info["repo"];

        if info["ref_kind"] == "commit" {
            client
                .patch(format!(
                    "{}/api/repo/{}/should_track/false",
                    &*BACKEND_HOST, &repo_id
                ))
                .send()?;
        } else {
            let now = Instant::now();
            let command_to_run = format!(
                "git clone --depth 1 --filter blob:none --no-checkout {} {}; exit 0",
                &repo, &monitor_path_str
            );
            let output = spawn_with_output!(
                bash -c $command_to_run 2>&1
            )?
            .wait_with_output()?;

            client
                .post(format!(
                    "{}/api/repo/{}/track_log",
                    &*BACKEND_HOST, &repo_id
                ))
                .body(format!(
                    "{}\n\n<hr><h3>Summary:</h3><b>Total time:</b> {:.2?}\n<b>Finished at:</b> {}",
                    output.replace(&monitor_path_str, "$REPO_CLONE_DIR"),
                    now.elapsed(),
                    chrono::Utc::now().to_rfc2822()
                ))
                .send()
                .expect("error posting track log");
        }

        build(
            &client,
            &repos_map[&repo_id],
            &info["repo"],
            &info["ref"],
            &info["pathspec"],
        )?;
    }

    Ok(true)
}

fn build(
    client: &reqwest::blocking::Client,
    repo_obj: &dto::Repo,
    repo: &str,
    r#ref: &str,
    pathspec: &str,
) -> anyhow::Result<()> {
    let build_dir = format!("{}/{}", &*BUILDING_DIR, &repo_obj.id);
    run_cmd!(
        rm -rf $build_dir;
        mkdir -p $build_dir;
    )?;

    // start_update
    client
        .patch(format!(
            "{}/api/repo/{}/start_update",
            &*BACKEND_HOST, repo_obj.id
        ))
        .send()?;

    let now = Instant::now();

    let build_cmd = format!(
        "git clone --progress --depth 1 --filter blob:none --no-checkout --no-single-branch {} {};
        cd {};
        git checkout --progress {} -- {};
        cd {};
        build-img -d {} {}/{} {}/image.tar;
        exit 0;",
        &repo,
        &build_dir,
        &build_dir,
        &r#ref,
        &pathspec,
        &*INITIAL_DIR,
        &repo_obj.dockerfile,
        &build_dir,
        &pathspec,
        &build_dir,
    );

    // update build log
    spawn_with_output!(
        bash -c $build_cmd 2>&1
    )?
    .wait_with_pipe(&mut |pipe| {
        BufReader::new(pipe)
            .lines()
            .filter_map(|line| line.map(|l| l.replace(&build_dir, "$REPO_CLONE_DIR")).ok())
            .for_each(|line| {
                client
                    .post(format!(
                        "{}/api/repo/{}/build_log",
                        &*BACKEND_HOST, repo_obj.id
                    ))
                    .body(line)
                    .send()
                    .expect("error posting build log");
            })
    })?;

    let mut build_successful = false;

    let image_digest = match spawn_with_output!(
        crane digest --tarball $build_dir/image.tar
    )?
    .wait_with_output()
    {
        Ok(output) => {
            build_successful = true;

            // update digest
            client
                .post(format!(
                    "{}/api/repo/{}/image_digest",
                    &*BACKEND_HOST, repo_obj.id
                ))
                .body(output.clone())
                .send()?;

            output
        }
        Err(_) => String::from(
            "Error getting digest from image tarball. Maybe the build was not successful.",
        ),
    };

    client
        .post(format!(
            "{}/api/repo/{}/build_log",
            &*BACKEND_HOST, repo_obj.id
        ))
        .body(format!(
            "\n<hr><h3>Summary:</h3><b>Digest:</b> {}\n<b>Total time:</b> {:.2?}\n<b>Finished at:</b> {}",
            &image_digest,
            now.elapsed(),
            chrono::Utc::now().to_rfc2822()
        ))
        .send()
        .expect("error posting build log");

    client
        .patch(format!(
            "{}/api/repo/{}/end_update",
            &*BACKEND_HOST, &repo_obj.id
        ))
        .send()
        .unwrap();

    for nb in &repo_obj.notebooks {
        client
            .put(format!(
                "{}/api/notebook/reset-push_log?name={}&namespace={}",
                &*BACKEND_HOST, &nb.nb_id.name, &nb.nb_id.namespace
            ))
            .send()?;
        let now = Instant::now();
        if build_successful {
            let image_name = &nb.nb_data.image;

            if nb.nb_data.private_registry {
                let cr_basic_auth = utils::get_cr_basic_auth(
                    &nb.nb_data.registry_credential_secret,
                    &repo_obj.secret_namespace,
                    &nb.nb_data.registry,
                )?;
                let registry = &nb.nb_data.registry;

                let command_to_run = format!(
                    "
                crane auth login --username {} --password {} {};
                crane push {}/image.tar {};
                exit 0;
            ",
                    &cr_basic_auth.username,
                    &cr_basic_auth.password,
                    &registry,
                    &build_dir,
                    &image_name
                );

                spawn_with_output!(bash -c $command_to_run 2>&1)?.wait_with_pipe(&mut |pipe| {
                    BufReader::new(pipe)
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| {
                            client
                                .post(format!(
                                    "{}/api/notebook/push_log?name={}&namespace={}",
                                    &*BACKEND_HOST, &nb.nb_id.name, &nb.nb_id.namespace
                                ))
                                .body(line)
                                .send()
                                .expect("error posting push log");
                        })
                })?;
            } else {
                let command_to_run = format!(
                    "
                crane push {}/image.tar {};
                exit 0;
            ",
                    &build_dir, &image_name
                );
                spawn_with_output!(
                    bash -c $command_to_run 2>&1
                )?
                .wait_with_pipe(&mut |pipe| {
                    BufReader::new(pipe)
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| {
                            client
                                .post(format!(
                                    "{}/api/notebook/push_log?name={}&namespace={}",
                                    &*BACKEND_HOST, &nb.nb_id.name, &nb.nb_id.namespace
                                ))
                                .body(line)
                                .send()
                                .expect("error posting push log");
                        })
                })?;
            }

            client
                .post(format!(
                    "{}/api/notebook/push_log?name={}&namespace={}",
                    &*BACKEND_HOST, &nb.nb_id.name, &nb.nb_id.namespace
                ))
                .body(format!(
                    "\n<hr><h3>Summary:</h3><b>Total time:</b> {:.2?}\n<b>Finished at:</b> {}",
                    now.elapsed(),
                    chrono::Utc::now().to_rfc2822()
                ))
                .send()
                .expect("error posting push log");

            if nb.nb_data.auto_sync {
                client
                    .get(format!(
                        "{}/api/notebook/restart_pod?name={}&namespace={}",
                        &*BACKEND_HOST, nb.nb_id.name, nb.nb_id.namespace
                    ))
                    .send()?;
            }
        } else {
            client
                .post(format!(
                    "{}/api/notebook/push_log?name={}&namespace={}",
                    &*BACKEND_HOST, &nb.nb_id.name, &nb.nb_id.namespace
                ))
                .body(format!(
                    "\n<hr><h3>Summary:</h3><b>Incident:</b> Error happened during build. Skip pushing\n<b>Total time:</b> {:.2?}\n<b>Finished at:</b> {}",
                    now.elapsed(),
                    chrono::Utc::now().to_rfc2822()
                ))
                .send()
                .expect("error posting push log");
        }
    }
    Ok(())
}

fn should_build(
    client: &reqwest::blocking::Client,
    repo_id: &i32,
    monitor_path_dir: &str,
    r#ref: &str,
    pathspec: &str,
) -> anyhow::Result<bool> {
    let now = Instant::now();

    let command_to_run = format!(
        "cd {};
        git fetch --tags --verbose --progress;
        git diff --quiet HEAD {} -- {} || echo '{} has changed';
        git remote prune origin 1>/dev/null 2>&1;
        git repack  1>/dev/null 2>&1;
        git prune-packed  1>/dev/null 2>&1;
        git reflog expire --expire=now --all  1>/dev/null 2>&1;
        git gc --aggressive --force --prune=now  1>/dev/null 2>&1;
        exit 0",
        monitor_path_dir, r#ref, pathspec, pathspec
    );

    let output = spawn_with_output!(bash -c $command_to_run 2>&1)?.wait_with_output()?;

    let res = output.contains(&format!("{} has changed", &pathspec));

    client
        .post(format!(
            "{}/api/repo/{}/track_log",
            &*BACKEND_HOST, &repo_id
        ))
        .body(format!(
            "{}\n\n<hr><h3>Summary:</h3><b>Total time:</b> {:.2?}\n<b>Finished at:</b> {}",
            output,
            now.elapsed(),
            chrono::Utc::now().to_rfc2822()
        ))
        .send()
        .expect("error posting track log");

    Ok(res)
}

fn manager(repos_map_lock: Arc<RwLock<HashMap<i32, dto::Repo>>>) -> anyhow::Result<()> {
    // 1. update repo maps
    let mut repos_map = repos_map_lock
        .write()
        .expect("error unlock repos map for write");
    let new_repo_map = get_repo_map();
    let keys = new_repo_map
        .keys()
        .into_iter()
        .map(|k| k.clone())
        .collect::<Vec<i32>>();

    clean_all_old_dir(&HashSet::from_iter(keys.iter().map(|i| format!("{}", i))))?;

    for k in keys {
        if !(*repos_map).contains_key(&k) {
            let repos_map_lock_clone = repos_map_lock.clone();
            std::thread::spawn(move || {
                let mut last_run_success = true;
                loop {
                    match monitor(repos_map_lock_clone.clone(), k.clone(), last_run_success) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                            last_run_success = true;
                        }
                        Err(e) => {
                            println!(
                                "Error happened when running thread for repo with id: {}. Will retry. Error details: \n{}",
                                k, e
                            );
                            last_run_success = false;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_secs(*SLEEP_TIME_SEC));
                }
            });
        }
    }

    *repos_map = new_repo_map.clone();

    Ok(())
}

fn get_repo_map() -> HashMap<i32, dto::Repo> {
    retry::retry(Fixed::from_millis(1000), || {
        reqwest::blocking::Client::builder()
            .timeout(None)
            .build()
            .unwrap_or_default()
            .get(format!("{}/api/repo/map", &*BACKEND_HOST))
            .send()
            .expect("error when requesting for repo map")
            .json::<HashMap<i32, dto::Repo>>()
    })
    .expect("error when deserialize repos map")
}

fn clean_all_old_dir(dir_set: &HashSet<String>) -> anyhow::Result<()> {
    clean_old_dir(dir_set, TRACKING_DIR.clone())?;
    clean_old_dir(dir_set, BUILDING_DIR.clone())?;
    Ok(())
}

fn clean_old_dir(dir_set: &HashSet<String>, dir: String) -> anyhow::Result<()> {
    if Path::new(&dir).exists() {
        let paths = std::fs::read_dir(&dir).unwrap();

        for path in paths {
            let dir_entry = path?;
            let is_dir = &dir_entry.metadata().map(|p| p.is_dir())?;
            let file_name_res = dir_entry.file_name();
            let base_name = &file_name_res.to_str().unwrap_or_default();
            let file_path_res = dir_entry.path();
            let dir_name = file_path_res.to_str().unwrap_or_default();
            if *is_dir && !dir_set.contains(*base_name) {
                // println!("Name: {}", path.unwrap().path().display())
                std::fs::remove_dir_all(dir_name)?;
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
    signal_hook::flag::register(signal_hook::consts::SIGQUIT, Arc::clone(&term))?;

    dotenv::dotenv().ok();
    let repos_map_lock = Arc::new(RwLock::new(HashMap::default()));
    println!("Starting notebook-ci monitoring...");

    while !term.load(Ordering::Relaxed) {
        manager(repos_map_lock.clone())?;
        std::thread::sleep(std::time::Duration::from_secs(*SLEEP_TIME_SEC));
        reqwest::blocking::Client::builder()
            .timeout(None)
            .build()
            .unwrap_or_default()
            .get(format!("{}/api/reconcile", &*BACKEND_HOST))
            .send()?;
    }

    reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap_or_default()
        .get(format!("{}/api/stop-all-updates", &*BACKEND_HOST))
        .send()?;
    Ok(())
}
