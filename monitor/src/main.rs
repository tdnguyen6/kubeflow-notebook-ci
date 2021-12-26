use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::Path,
    sync::{Arc, RwLock},
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
    static ref GIT_SSH_SECRET_KEY: String = dotenv::var("GIT_SSH_SECRET_KEY").unwrap_or_default();
    static ref GIT_USERNAME_SECRET_KEY: String =
        dotenv::var("GIT_USERNAME_SECRET_KEY").unwrap_or_default();
    static ref GIT_PASSWORD_SECRET_KEY: String =
        dotenv::var("GIT_PASSWORD_SECRET_KEY").unwrap_or_default();
    static ref CR_USERNAME_SECRET_KEY: String =
        dotenv::var("CR_USERNAME_SECRET_KEY").unwrap_or_default();
    static ref CR_PASSWORD_SECRET_KEY: String =
        dotenv::var("CR_PASSWORD_SECRET_KEY").unwrap_or_default();
    static ref SLEEP_TIME_SEC: u64 = dotenv::var("SLEEP_TIME_SEC")
        .unwrap_or_default()
        .parse()
        .unwrap_or(10);
}

fn monitor(
    repos_map_lock: Arc<RwLock<HashMap<i32, dto::Repo>>>,
    repo_id: i32,
) -> anyhow::Result<()> {
    loop {
        {
            let repos_map = repos_map_lock
                .read()
                .expect("error unlock repos map for read");
            if !repos_map.contains_key(&repo_id) {
                run_cmd!(rm -rf $TRACKING_DIR/$repo_id)?;
                break;
            }

            let client = reqwest::blocking::Client::new();
            let monitor_path = Path::new(&*TRACKING_DIR).join(format!("{}", repo_id));
            let mut info = utils::parse_git_uri(&repos_map[&repo_id].uri)?;
            let monitor_path_str = monitor_path.to_str().unwrap_or_default();
            if repos_map[&repo_id].private_repo {
                if &info["protocol"] == "ssh" {
                    let ssh_key = utils::get_kf_secret_key(
                        &repos_map[&repo_id].repo_credential_secret,
                        &repos_map[&repo_id].secret_namespace,
                        &*GIT_SSH_SECRET_KEY,
                    )?;
                    run_cmd!(echo $ssh_key > $TRACKING_DIR/$repo_id/id_rsa)?;
                    (*info.get_mut("repo").unwrap()) = format!("git@{}", &info["repo"]);
                } else {
                    let username = utils::get_kf_secret_key(
                        &repos_map[&repo_id].repo_credential_secret,
                        &repos_map[&repo_id].secret_namespace,
                        &*GIT_USERNAME_SECRET_KEY,
                    )?;
                    let password = utils::get_kf_secret_key(
                        &repos_map[&repo_id].repo_credential_secret,
                        &repos_map[&repo_id].secret_namespace,
                        &*GIT_PASSWORD_SECRET_KEY,
                    )?;
                    (*info.get_mut("repo").unwrap()) =
                        format!("{}:{}@{}", &username, &password, &info["repo"]);
                }
            }

            if monitor_path.is_dir() {
                if should_build(&info["ref"], &info["pathspec"])? {
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

                if info["ref_type"] == "commit" {
                    client
                        .patch(format!(
                            "{}/api/repo/{}/should_track/false",
                            &*BACKEND_HOST, &repo_id
                        ))
                        .send()?;
                } else {
                    let now = Instant::now();
                    run_cmd!(git clone --depth 1 --filter blob:none --filter tree:0 --no-checkout $repo $monitor_path_str)?;
                    client
                        .post(format!(
                            "{}/api/repo/{}/track_log",
                            &*BACKEND_HOST, &repo_id
                        ))
                        .body(format!(
                            "Summary:\nTotal time: {:.2?}\nFinished at: {}",
                            now.elapsed(),
                            chrono::Utc::now().timestamp()
                        ))
                        .send()
                        .expect("error posting build log");
                }

                build(
                    &client,
                    &repos_map[&repo_id],
                    &info["repo"],
                    &info["ref"],
                    &info["pathspec"],
                )?;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(*SLEEP_TIME_SEC));
    }

    Ok(())
}

fn build(
    client: &reqwest::blocking::Client,
    repo_obj: &dto::Repo,
    repo: &str,
    r#ref: &str,
    pathspec: &str,
    // nb_list: &Vec<dto::NotebookId>,
) -> anyhow::Result<()> {
    let build_dir = format!("{}/{}", &*BUILDING_DIR, &repo_obj.id);
    run_cmd!(
        rm -rf $build_dir
        mkdir -p $build_dir
    )?;

    // start_update
    client
        .patch(format!(
            "{}/api/repo/{}/start_update",
            &*BACKEND_HOST, repo_obj.id
        ))
        .send()?;

    let now = Instant::now();
    // update build log
    spawn_with_output!(
        git clone --depth 1 --filter blob:none --no-checkout --no-single-branch $repo $build_dir
        cd $build_dir
        git checkout $r#ref -- $pathspec
        build-img $build_dir/$pathspec
    )?
    .wait_with_pipe(&mut |pipe| {
        BufReader::new(pipe)
            .lines()
            .filter_map(|line| line.ok())
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

    for nb in &repo_obj.notebooks {
        let image_name = &nb.nb_data.image;
        if nb.nb_data.private_registry {
            let username = utils::get_kf_secret_key(
                &repo_obj.repo_credential_secret,
                &repo_obj.secret_namespace,
                &*CR_USERNAME_SECRET_KEY,
            )?;
            let password = utils::get_kf_secret_key(
                &repo_obj.repo_credential_secret,
                &repo_obj.secret_namespace,
                &*CR_PASSWORD_SECRET_KEY,
            )?;
            let registry = &nb.nb_data.registry;

            run_cmd!(
                crane auth login --username $username --password $password $registry
                crane push $build_dir/image.tar $image_name
            )?;
        } else {
            run_cmd!(
                crane push $build_dir/image.tar $image_name
            )?;
        }

        if nb.nb_data.auto_sync {
            client
                .get(format!(
                    "{}/api/notebook/restart_pod?name={}&namespace={}",
                    &*BACKEND_HOST, nb.nb_id.name, nb.nb_id.namespace
                ))
                .send()?;
        }
    }

    client
        .post(format!(
            "{}/api/repo/{}/build_log",
            &*BACKEND_HOST, repo_obj.id
        ))
        .body(format!(
            "Summary:\nTotal time: {:.2?}\nFinished at: {}",
            now.elapsed(),
            chrono::Utc::now().timestamp()
        ))
        .send()
        .expect("error posting build log");

    let image_digest = spawn_with_output!(
        crane digest --tarball $build_dir/image.tar
    )?
    .wait_with_output()?;

    // update digest
    client
        .post(format!(
            "{}/api/repo/{}/image_digest/{}",
            &*BACKEND_HOST, repo_obj.id, &image_digest
        ))
        .send()?;

    // end_update
    client
        .patch(format!(
            "{}/api/repo/{}/end_update",
            &*BACKEND_HOST, repo_obj.id
        ))
        .send()?;
    Ok(())
}

fn should_build(r#ref: &str, pathspec: &str) -> anyhow::Result<bool> {
    run_cmd!(
        git fetch --tags
        git diff --quiet HEAD origin/$r#ref -- $pathspec
    )?;
    run_cmd!(
        git remote prune origin
        git repack
        git prune-packed
        git reflog expire --expire=now --all
        git gc --aggressive --force --prune=now
    )?;

    Ok(true)
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

    for k in keys {
        if !(*repos_map).contains_key(&k) {
            let repos_map_lock_clone = repos_map_lock.clone();
            std::thread::spawn(move || monitor(repos_map_lock_clone, k.clone()).unwrap());
            // println!("{:#?}", handle.thread().id().clone());
        }
    }

    *repos_map = new_repo_map.clone();

    Ok(())
}

fn get_repo_map() -> HashMap<i32, dto::Repo> {
    retry::retry(Fixed::from_millis(1000), || {
        reqwest::blocking::get(format!("{}/api/repo/map", &*BACKEND_HOST))
            .expect("error when requesting for repo map")
            .json::<HashMap<i32, dto::Repo>>()
    })
    .expect("error when deserialize repos map")
}

fn main() -> anyhow::Result<()> {
    let tracking_dir = &*TRACKING_DIR;
    run_cmd!(rm -rf $tracking_dir)?;
    dotenv::dotenv().ok();
    let repos_map_lock = Arc::new(RwLock::new(get_repo_map()));
    loop {
        manager(repos_map_lock.clone())?;
        std::thread::sleep(std::time::Duration::from_secs(*SLEEP_TIME_SEC));
        // reconcile()
        todo!()
    }
}
