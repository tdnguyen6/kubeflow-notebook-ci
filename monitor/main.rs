use cmd_lib::{run_cmd, spawn_with_output};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock}, io::BufRead,
};

fn get_notebooks_map() -> anyhow::Result<HashMap<String, String>> {
    let file = std::fs::File::open("./repos.json")?;
    let reader = std::io::BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn manager(
    notebooks_map: &HashMap<String, String>,
    repos_map_lock: Arc<RwLock<HashMap<String, ThreadContent>>>,
) -> anyhow::Result<()> {
    // 1. update repo maps
    let mut repos_map = repos_map_lock
        .write()
        .expect("error unlock repos map for write");

    for ele in notebooks_map {
        // println!("{:#?}", ele);
        if repos_map.contains_key(ele.1) {
            (*repos_map.get_mut(ele.1).unwrap())
                .notebooks
                .insert(ele.0.to_string());
        } else {
            // let repos_map_lock_clone = repos_map_lock.clone();
            let repo_id = ele.1.to_string();
            let repos_map_lock_clone = repos_map_lock.clone();
            repos_map.insert(
                repo_id.clone(),
                ThreadContent {
                    handle: std::thread::spawn(|| monitor(repo_id, repos_map_lock_clone)),
                    notebooks: HashSet::from([ele.0.to_owned()]),
                    dir: uuid::Uuid::new_v4().to_simple().to_string(),
                },
            );
        }
    }

    let current_keys: Vec<String> = repos_map.keys().map(|k| k.to_owned()).collect();
    let repos = HashSet::<&String>::from_iter(notebooks_map.values().into_iter());
    for k in current_keys {
        if !repos.contains(&k) {
            println!("{}", k);
            repos_map.remove(&k).unwrap();
        }
    }
    Ok(())
}

fn monitor(
    repo_id: String,
    repos_map_lock: Arc<RwLock<HashMap<String, ThreadContent>>>,
) -> anyhow::Result<()> {
    loop {
        {
            let repos_map = repos_map_lock.read().unwrap();
            if repos_map.contains_key(&repo_id) {
                let tc = &repos_map[&repo_id];
            } else {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

#[derive(Debug)]
struct ThreadContent {
    handle: std::thread::JoinHandle<anyhow::Result<()>>,
    notebooks: HashSet<String>,
    dir: String,
}

fn main() -> anyhow::Result<()> {
    let mut notebooks_map = HashMap::default();
    let repos_map_lock = Arc::new(RwLock::new(HashMap::default()));
    loop {
        notebooks_map = get_notebooks_map().unwrap();
        manager(&notebooks_map, Arc::clone(&repos_map_lock))?;
        println!(
            "------------------------------------------------------------------------------------"
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // let mut proc = spawn_with_output! {
    //     test-build/build.sh test-build 2>&1
    // }?;
    // std::thread::sleep(std::time::Duration::from_secs(1));

    // proc.wait_with_pipe(&mut |pipe| {
    //     std::io::BufReader::new(pipe)
    //         .lines()
    //         .filter_map(|line| line.ok())
    //         .for_each(|line| run_cmd!(echo "$line" >> a.txt).unwrap());
    // })?;

    Ok(())
}
