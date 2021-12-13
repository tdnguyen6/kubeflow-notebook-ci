use cmd_lib::{run_cmd, spawn_with_output};

pub fn kube_notebooks(namespace: &str) -> anyhow::Result<Vec<String>> {
    let mut proc = spawn_with_output!(kubectl get notebook -n $namespace -o jsonpath="{.items[*].metadata.name}")?;
    let output = proc.wait_with_output()?;
    Ok(output.split(" ").map(|s| String::from(s)).collect())
}

pub fn has_kube_notebook(namespace: &str, name: &str) -> anyhow::Result<bool> {
    match run_cmd!(kubectl get notebook $name -n $namespace) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
