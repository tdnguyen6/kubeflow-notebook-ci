use std::collections::HashMap;

use cmd_lib::spawn_with_output;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

pub fn parse_git_uri(uri: &str) -> anyhow::Result<HashMap<String, String>> {
    let re = Regex::new(
        r"^(?<protocol>http|https):\/\/(?<repo>(?:[A-Za-z0-9]+[.:\/\-_])*(?:[A-Za-z0-9]+))(?:(?:\/\/)?(?<pathspec>(?:[A-Za-z0-9]+[\/\-_])*(?:[A-Za-z0-9]+)))?(?:@(?<ref>(?:branches\/[A-ZFa-z0-9-_\/]+)|(?:tags\/[A-ZFa-z0-9-_\/]+)|(?:[A-Fa-f0-9]+)))?$",
    )?;
    if let Ok(ocapture) = re.captures(uri) {
        if let Some(capture) = ocapture {
            let mut hm =
                HashMap::<String, String>::from_iter(re.capture_names().filter_map(|oname| {
                    oname.map(|name| {
                        (
                            name.to_owned(),
                            capture
                                .name(name)
                                .map_or(String::default(), |m| m.as_str().to_owned()),
                        )
                    })
                }));
            hm.insert(String::from("ref_kind"), String::from("commit"));
            if hm["ref"].starts_with("branches/") {
                (*hm.get_mut("ref_kind").unwrap()) = String::from("branch");
                (*hm.get_mut("ref").unwrap()) =
                    (*hm.get_mut("ref").unwrap()).replace("branches/", "origin/");
            } else if hm["ref"].starts_with("tags/") {
                (*hm.get_mut("ref_kind").unwrap()) = String::from("tag");
            } else if hm["ref"].is_empty() {
                (*hm.get_mut("ref").unwrap()) = String::from("HEAD");
                (*hm.get_mut("ref_kind").unwrap()) = String::from("branch");
            }
            return Ok(hm);
        }
    }
    Ok(HashMap::default())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

pub fn get_git_basic_auth(secret: &str, namespace: &str) -> anyhow::Result<BasicAuth> {
    Ok(BasicAuth {
        username: get_kf_secret_key(secret, namespace, "username")?,
        password: get_kf_secret_key(secret, namespace, "password")?,
    })
}
pub fn get_cr_basic_auth(
    secret: &str,
    namespace: &str,
    registry: &str,
) -> anyhow::Result<BasicAuth> {
    let jq_query = format!(".auths[\"{}\"]", registry);

    let jq_username_query = format!("{}.username", jq_query);
    let jq_password_query = format!("{}.password", jq_query);

    Ok(BasicAuth {
        username: spawn_with_output!(kubectl get secret $secret -n $namespace -o jsonpath="{.data}" | jq -r ".[\".dockerconfigjson\"]" | base64 -d | jq -r $jq_username_query)?
            .wait_with_output()?
            .as_str()
            .to_owned(),
        password: spawn_with_output!(kubectl get secret $secret -n $namespace -o jsonpath="{.data}" | jq -r ".[\".dockerconfigjson\"]" | base64 -d | jq -r $jq_password_query)?
            .wait_with_output()?
            .as_str()
            .to_owned(),
    })
}

fn get_kf_secret_key(secret: &str, namespace: &str, key: &str) -> anyhow::Result<String> {
    let jsonpath = format!("{{.data.{}}}", key);
    Ok(
        spawn_with_output!(kubectl get secret $secret -n $namespace -o jsonpath=$jsonpath | base64 -d)?
            .wait_with_output()?
            .as_str()
            .to_owned(),
    )
}
