use cmd_lib::{run_cmd, spawn_with_output};
use fancy_regex::Regex;

use crate::models::dto;

// pub fn kube_notebooks(namespace: &str) -> anyhow::Result<Vec<String>> {
//     let mut proc = spawn_with_output!(kubectl get notebook -n $namespace -o jsonpath="{.items[*].metadata.name}")?;
//     let output = proc.wait_with_output()?;
//     Ok(output.split(" ").map(|s| String::from(s)).collect())
// }

// pub fn has_kube_notebook(namespace: &str, name: &str) -> anyhow::Result<bool> {
//     match run_cmd!(kubectl get notebook $name -n $namespace) {
//         Ok(_) => Ok(true),
//         Err(_) => Ok(false),
//     }
// }

pub fn kf_notebooks(namespace: Option<&str>) -> anyhow::Result<Vec<String>> {
    let mut proc = match namespace {
        Some(ns) => {
            spawn_with_output!(kubectl get notebook.kubeflow.org -n $ns -o jsonpath="{.items[*].metadata.name}")
        }
        None => {
            spawn_with_output!(kubectl get notebook.kubeflow.org -A -o jsonpath="{.items[*].metadata.name}")
        }
    }?;

    Ok(proc
        .wait_with_output()?
        .split(" ")
        .filter(|s| *s != "")
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}

pub fn kf_secrets(namespace: Option<&str>) -> anyhow::Result<Vec<String>> {
    let mut proc = match namespace {
        Some(ns) => {
            spawn_with_output!(kubectl get secrets -n $ns -o jsonpath="{.items[*].metadata.name}")
        }
        None => {
            spawn_with_output!(kubectl get secrets -A -o jsonpath="{.items[*].metadata.name}")
        }
    }?;

    Ok(proc
        .wait_with_output()?
        .split(" ")
        .filter(|s| *s != "")
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}

pub fn all_kf_users() -> anyhow::Result<Vec<String>> {
    let mut proc = spawn_with_output!(kubectl get profile.kubeflow.org -A -o jsonpath="{.items[*].spec.owner.name}")?;

    Ok(proc
        .wait_with_output()?
        .split(" ")
        .filter(|s| *s != "")
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}

pub fn all_kf_users_namespaces() -> anyhow::Result<Vec<String>> {
    all_kf_users().and_then(|vs| vs.iter().map(|s| kf_user_namespace(s)).collect())
}

pub fn kf_user_namespace(user: &str) -> anyhow::Result<String> {
    let mut proc = spawn_with_output!(kubectl get profile.kubeflow.org -A -o jsonpath="{.items}")?;

    let res: serde_json::Value = serde_json::from_str(proc.wait_with_output()?.as_str())?;
    let fres = res
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["spec"]["owner"]["name"] == user)
        .map(|v| v["metadata"]["name"].as_str().unwrap().to_string())
        .collect::<Vec<String>>();

    match fres.first() {
        Some(s) => Ok(s.to_owned()),
        None => Ok(String::default()),
    }
}

pub fn kf_notebook_pod_image_digest(nb_id: &dto::NotebookId) -> anyhow::Result<String> {
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    let mut proc = spawn_with_output!(kubectl get po -n $namespace  -l notebook-name=$name -o yaml | grep imageID)?;

    let output = proc.wait_with_output()?;

    let re = Regex::new(r"^\s*imageID.*(?!istio/proxy)@(?<digest>sha256:[a-f0-9]+)")?;

    Ok(re
        .captures(&output)?
        .expect("not able to capture")
        .name("digest")
        .map_or("", |m| m.as_str())
        .to_owned())
}

pub fn restart_nb_pod(nb_id: &dto::NotebookId) -> anyhow::Result<()> {
    let current_policy = get_nb_image_pull_policy(nb_id)?;
    patch_nb_image_pull_policy(nb_id, &current_policy, "Always")?;
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    run_cmd!(
        kubectl rollout restart statefulset $name -n $namespace
        kubectl rollout status --watch --timeout=10s statefulset $name -n $namespace
    )?;
    patch_nb_image_pull_policy(nb_id, "Always", &current_policy)?;
    Ok(())
}

fn patch_nb_image_pull_policy(nb_id: &dto::NotebookId, old_policy: &str, new_policy: &str) -> anyhow::Result<()> {
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    let patch_content = spawn_with_output!(
        kubectl get notebook $name -n $namespace -o jsonpath="{.spec.template.spec.containers[0]}" | sed -e "s/\"imagePullPolicy\":\"$old_policy\"/\"imagePullPolicy\":\"$new_policy\"/" | kubectl patch notebook $name -n $namespace -p - --type=merge
    )?.wait_with_output()?;
    run_cmd!(kubectl patch notebook $name -n $namespace -p "{\"spec\":{\"template\":{\"spec\":{\"containers\":[$patch_content]}}}}" --type=merge)?;
    Ok(())
}

fn get_nb_image_pull_policy(nb_id: &dto::NotebookId) -> anyhow::Result<String> {
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    Ok(spawn_with_output!(
        kubectl get notebook $name -n $namespace -o jsonpath="{.spec.template.spec.containers[0].imagePullPolicy}"
    )?
    .wait_with_output()?)
}
