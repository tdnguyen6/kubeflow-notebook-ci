use std::collections::HashMap;

use cmd_lib::{run_cmd, spawn_with_output};
use fancy_regex::Regex;

pub fn parse_git_uri(uri: &str) -> anyhow::Result<HashMap<String, String>> {
    let re = Regex::new(
        r"^(?<protocol>http|https|ssh):\/\/(?<repo>(?:[A-Za-z0-9]+[.\/\-_])*(?:[A-Za-z0-9]+))(?:(?:\/\/)?(?<pathspec>(?:[A-Za-z0-9]+[\/\-_])*(?:[A-Za-z0-9]+)))?(?:@(?<ref>(?:branches\/[A-ZFa-z0-9-_\/]+)|(?:tags\/[A-ZFa-z0-9-_\/]+)|(?:[A-Fa-f0-9]+)))?$",
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
            (*hm.get_mut("ref_kind").unwrap()) = String::from("commit");
            if hm["ref"].starts_with("branches/") {
                (*hm.get_mut("ref_kind").unwrap()) = String::from("branch");
                (*hm.get_mut("ref").unwrap()) =
                    (*hm.get_mut("ref").unwrap()).replace("branches/", "origin/");
            } else if hm["ref"].starts_with("tags/") {
                (*hm.get_mut("ref_kind").unwrap()) = String::from("tag");
            }
        }
    }
    Ok(HashMap::default())
}

pub fn get_kf_secret_key(secret: &str, namespace: &str, key: &str) -> anyhow::Result<String> {
    let jsonpath = format!("{{.data.{}}}", key);
    Ok(
        spawn_with_output!(mk get secret $secret -n $namespace -o jsonpath=$jsonpath | base64 -d)?
            .wait_with_output()?
            .as_str()
            .to_owned(),
    )
}
