use cmd_lib::run_cmd;

pub fn build(
    repo: &str,
    monitor_path_str: &str,
    r#ref: &str,
    pathspec: &str,
    // nb_list: &Vec<dto::NotebookId>,
) -> anyhow::Result<()> {
    // start_update
    run_cmd!(
        git clone --depth 1 --filter blob:none --filter tree:0 --no-checkout --no-single-branch $repo $monitor_path_str
        cd $monitor_path_str
        git checkout $r#ref -- $pathspec
    )?;
    // update build log
    // update digest
    // remove pod
    // // update all notebooks that are auto-synced
    // for nb in nb_list {
    //     // restart it
    // }
    // end_update
    Ok(())
}

// fn build_public_cr() {}
// fn build_private_docker_hub () {}
// fn build_private_jfrog () {}
// fn build_private_ecr () {}
// fn build_private_gcr () {}
// fn build_private_acr () {}

