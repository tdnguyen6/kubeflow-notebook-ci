use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notebook {
    pub id: i32,
    #[serde(flatten)]
    pub nb_id: NotebookId,
    #[serde(flatten)]
    pub nb_data: NotebookData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotebookId {
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotebookData {
    pub image: String,
    pub registry_credential_secret: String,
    pub private_registry: bool,
    pub registry: String,
    pub repo_id: i32,
    pub repo_uri: String,
    pub private_repo: bool,
    pub repo_credential_secret: String,
    pub auto_sync: bool,
    pub syncing: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Repo {
    pub id: i32,
    pub uri: String,
    pub secret_namespace: String,
    pub private_repo: bool,
    pub repo_credential_secret: String,
    pub updating: bool,
    pub build_log: String,
    pub track_log: String,
    pub digest: String,
    pub dockerfile: String,
    pub notebooks: Vec<Notebook>,
}

pub mod frontend {

    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct FrontendData {
        pub notebooks: Vec<FrontendNotebookData>,
        pub secrets: Vec<String>,
        pub namespace: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct FrontendNotebookData {
        pub name: String,
        pub building: bool,
        pub enabled: bool,
        pub out_of_sync: bool,
    }
}
