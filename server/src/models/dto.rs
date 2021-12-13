use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Notebook {
    pub id: i32,
    pub name: String,
    pub image: Option<String>,
    pub repo_id: Option<i32>,
}

impl Notebook {
    pub fn from_name(name: &str) -> Self {
        return Notebook {
            id: -1,
            name: name.to_string(),
            image: None,
            repo_id: None,
        };
    }
}

#[derive(Debug, Serialize)]
pub struct Repo {
    pub id: i32,
    pub git: String,
    pub updating: bool,
    pub log: Option<String>,
    pub digest: Option<String>,
    pub update_time: Option<String>,
    pub notebooks: Option<Vec<Notebook>>,
}
