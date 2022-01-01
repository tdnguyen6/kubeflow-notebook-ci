use cmd_lib::{run_cmd, spawn_with_output};
use fancy_regex::Regex;

use crate::{config::Config, models::dto};

pub fn kf_notebooks(namespace: Option<&str>, config: &Config) -> anyhow::Result<Vec<String>> {
    let resource = &config.kubeflow.notebook_resource;
    let mut proc = match namespace {
        Some(ns) => {
            spawn_with_output!(kubectl get $resource -n $ns -o jsonpath="{.items[*].metadata.name}")
        }
        None => {
            spawn_with_output!(kubectl get $resource -A -o jsonpath="{.items[*].metadata.name}")
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

pub fn all_kf_users(config: &Config) -> anyhow::Result<Vec<String>> {
    let resource = &config.kubeflow.profile_resource;

    let mut proc =
        spawn_with_output!(kubectl get $resource -A -o jsonpath="{.items[*].spec.owner.name}")?;

    Ok(proc
        .wait_with_output()?
        .split(" ")
        .filter(|s| *s != "")
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}

pub fn all_kf_users_namespaces(config: &Config) -> anyhow::Result<Vec<String>> {
    all_kf_users(config).and_then(|vs| vs.iter().map(|s| kf_user_namespace(s, config)).collect())
}

pub fn kf_user_namespace(user: &str, config: &Config) -> anyhow::Result<String> {
    let resource = &config.kubeflow.profile_resource;

    let mut proc = spawn_with_output!(kubectl get $resource -A -o jsonpath="{.items}")?;

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
    let mut proc = spawn_with_output!(kubectl get po -n $namespace  -l notebook-name=$name -o yaml | awk "/imageID/ && !/istio\\/proxy/")?;

    let output = proc.wait_with_output()?;

    let re = Regex::new(r"^\s*imageID.*@(?<digest>sha256:[a-f0-9]+)$")?;
    if let Ok(ocapture) = re.captures(&output) {
        return Ok(ocapture
            .and_then(|capture| capture.name("digest"))
            .map_or("", |m| m.as_str())
            .to_owned());
    }
    Ok(String::default())
}

pub fn restart_nb_pod(
    nb_id: &dto::NotebookId,
    is_private: &bool,
    registry_credential_secret: &str,
    image: &str,
    config: &Config,
) -> anyhow::Result<()> {
    // let current_policy = get_nb_image_pull_policy(&nb_id, config)?;
    patch_nb_image_pull_secret(&nb_id, &is_private, &registry_credential_secret, config)?;
    patch_nb_image(&nb_id, &image, config)?;
    patch_nb_image_pull_policy(&nb_id, "Always", config)?;
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    run_cmd!(
        kubectl delete statefulset $name -n $namespace;
    )?;
    // kubectl rollout status --watch statefulset $name -n $namespace;
    // patch_nb_image_pull_policy(&nb_id, &current_policy, config)?;
    Ok(())
}

fn patch_nb_image_pull_secret(
    nb_id: &dto::NotebookId,
    is_private: &bool,
    secret: &str,
    config: &Config,
) -> anyhow::Result<()> {
    let resource = &config.kubeflow.notebook_resource;
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    let jq_delete_key = format!("if .imagePullSecrets? then del(.imagePullSecrets[] | select(.name == \"{}\")) else . + {{\"imagePullSecrets\": []}} end", &secret);
    let mut jq_format_string = String::from(".");
    if *is_private {
        jq_format_string = format!(
            ".imagePullSecrets[.imagePullSecrets | length] |= . + {{\"name\": \"{}\"}}",
            &secret
        );
    }
    let patch_content = spawn_with_output!(
        kubectl get $resource $name -n $namespace -o jsonpath="{.spec.template.spec}" | jq -rc $jq_delete_key | jq -rc $jq_format_string | jq .imagePullSecrets
    )?.wait_with_output()?;
    run_cmd!(kubectl patch $resource $name -n $namespace -p "{\"spec\":{\"template\":{\"spec\":{\"imagePullSecrets\":$patch_content}}}}" --type=merge)?;
    Ok(())
}

fn patch_nb_image_pull_policy(
    nb_id: &dto::NotebookId,
    new_policy: &str,
    config: &Config,
) -> anyhow::Result<()> {
    let resource = &config.kubeflow.notebook_resource;
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    let jq_format_string = format!(".imagePullPolicy = \"{}\"", &new_policy);
    let patch_content = spawn_with_output!(
        kubectl get $resource $name -n $namespace -o jsonpath="{.spec.template.spec.containers[0]}" | jq -rc $jq_format_string
    )?.wait_with_output()?;
    run_cmd!(kubectl patch $resource $name -n $namespace -p "{\"spec\":{\"template\":{\"spec\":{\"containers\":[$patch_content]}}}}" --type=merge)?;
    Ok(())
}

// fn get_nb_image_pull_policy(nb_id: &dto::NotebookId, config: &Config) -> anyhow::Result<String> {
//     let resource = &config.kubeflow.notebook_resource;

//     let (name, namespace) = (&nb_id.name, &nb_id.namespace);
//     Ok(spawn_with_output!(
//         kubectl get $resource $name -n $namespace -o jsonpath="{.spec.template.spec.containers[0].imagePullPolicy}"
//     )?
//     .wait_with_output()?)
// }

fn patch_nb_image(nb_id: &dto::NotebookId, new_image: &str, config: &Config) -> anyhow::Result<()> {
    let resource = &config.kubeflow.notebook_resource;
    let (name, namespace) = (&nb_id.name, &nb_id.namespace);
    let jq_format_string = format!(".image = \"{}\"", &new_image);
    let patch_content = spawn_with_output!(
        kubectl get $resource $name -n $namespace -o jsonpath="{.spec.template.spec.containers[0]}" | jq -rc $jq_format_string
    )?.wait_with_output()?;
    run_cmd!(kubectl patch $resource $name -n $namespace -p "{\"spec\":{\"template\":{\"spec\":{\"containers\":[$patch_content]}}}}" --type=merge)?;
    Ok(())
}
