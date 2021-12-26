use std::collections::{HashMap, HashSet};

use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::dto;
use crate::utils;

pub mod clean;
pub mod notebook;
pub mod repo;

#[derive(Debug, Deserialize)]
pub struct EmailOnlyParam {
    email: String,
}

#[actix_web::get("/api/frontend-data")]
pub async fn frontend_data(
    pool: web::Data<PgPool>,
    param: web::Query<EmailOnlyParam>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let ns = utils::kf_user_namespace(&param.email)?;
    let res = sqlx::query!(
        "SELECT n.name, c.updating, c.digest
            FROM notebooks n
            JOIN ci_jobs c
            ON n.repo_id = c.id
            WHERE n.namespace = $1",
        &ns,
    )
    .fetch_all(&**pool)
    .await?;
    let mut enabled_notebooks: HashMap<String, (bool, String)> = HashMap::default();
    for rec in res {
        enabled_notebooks.insert(
            rec.name.unwrap().clone(),
            (
                rec.updating.unwrap_or_default(),
                rec.digest.unwrap_or_default(),
            ),
        );
    }
    let data = dto::frontend::FrontendData {
        notebooks: utils::kf_notebooks(Some(&ns))?
            .iter()
            .map(|nb| dto::frontend::FrontendNotebookData {
                name: nb.to_owned(),
                enabled: enabled_notebooks.contains_key(nb),
                building: if enabled_notebooks.contains_key(nb) {
                    enabled_notebooks[nb].0
                } else {
                    false
                },
                out_of_sync: if enabled_notebooks.contains_key(nb) {
                    utils::kf_notebook_pod_image_digest(&dto::NotebookId {
                        name: nb.to_owned(),
                        namespace: ns.to_owned(),
                    })
                    .unwrap_or_default()
                        != enabled_notebooks[nb].1
                } else {
                    false
                },
            })
            .collect(),
        secrets: utils::kf_secrets(Some(&ns))?,
        namespace: ns,
    };

    Ok(HttpResponse::Ok().json(&data))
}
