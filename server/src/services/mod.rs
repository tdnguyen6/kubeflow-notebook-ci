use std::collections::HashMap;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;
use sqlx::PgPool;

use crate::config::Config;
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
    config: web::Data<Config>,
    param: web::Query<EmailOnlyParam>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let ns = utils::kf_user_namespace(&param.email, &config)?;
    let res = sqlx::query!(
        "SELECT n.repo_id, n.name, n.syncing, c.updating, c.digest, c.dockerfile
            FROM notebooks n
            JOIN ci_jobs c
            ON n.repo_id = c.id
            WHERE n.namespace = $1",
        &ns,
    )
    .fetch_all(&**pool)
    .await?;
    let mut enabled_notebooks: HashMap<String, (bool, bool, String, i32, String)> =
        HashMap::default();
    for rec in res {
        enabled_notebooks.insert(
            rec.name.unwrap().clone(),
            (
                rec.updating.unwrap_or_default(),
                rec.syncing.unwrap_or_default(),
                rec.digest.unwrap_or_default(),
                rec.repo_id.unwrap_or_default(),
                rec.dockerfile,
            ),
        );
    }
    let data = dto::frontend::FrontendData {
        notebooks: utils::kf_notebooks(Some(&ns), &config)?
            .iter()
            .map(|nb| dto::frontend::FrontendNotebookData {
                name: nb.to_owned(),
                enabled: enabled_notebooks.contains_key(nb),
                building: if enabled_notebooks.contains_key(nb) {
                    enabled_notebooks[nb].0
                } else {
                    false
                },
                syncing: if enabled_notebooks.contains_key(nb) {
                    enabled_notebooks[nb].1
                } else {
                    false
                },
                out_of_sync: if enabled_notebooks.contains_key(nb) {
                    utils::kf_notebook_pod_image_digest(&dto::NotebookId {
                        name: nb.to_owned(),
                        namespace: ns.to_owned(),
                    })
                    .unwrap_or_default()
                        != enabled_notebooks[nb].2
                } else {
                    false
                },
                repo_id: if enabled_notebooks.contains_key(nb) {
                    enabled_notebooks[nb].3
                } else {
                    -1
                },
                dockerfile: if enabled_notebooks.contains_key(nb) {
                    enabled_notebooks[nb].4.clone()
                } else {
                    String::from("Dockerfile")
                },
            })
            .collect(),
        secrets: utils::kf_secrets(Some(&ns))?,
        namespace: ns,
    };

    Ok(HttpResponse::Ok().json(&data))
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct FrontendIndexTmpl<'a> {
    userid: &'a str,
}

#[get("/")]
pub async fn frontend(
    req: HttpRequest,
    config: web::Data<Config>,
) -> actix_web::Result<impl Responder, Box<dyn std::error::Error>> {
    let hdrs = req.headers();
    match hdrs.get(&config.kubeflow.userid_header) {
        Some(h) => Ok(HttpResponse::Ok().body(
            FrontendIndexTmpl {
                userid: h.to_str().unwrap(),
            }
            .render()
            .unwrap(),
        )),
        None => Ok(HttpResponse::Forbidden().finish()),
    }
}

#[actix_web::get("/api/stop-all-updates")]
pub async fn stop_all_updates_endpoint(
    pool: web::Data<PgPool>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    stop_all_updates(&**pool).await?;
    Ok("stopped successfully")
}

// #[actix_web::get("/api/stop-all-syncing")]
// pub async fn stop_all_syncing_endpoint(
//     pool: web::Data<PgPool>,
// ) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
// }

pub async fn stop_all_updates(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query!("UPDATE ci_jobs SET updating = false")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn stop_all_syncing(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query!("UPDATE notebooks SET syncing = false")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn stop_all_updates_and_syncing(pool: &PgPool) -> anyhow::Result<()> {
    stop_all_updates(pool).await?;
    stop_all_syncing(pool).await?;
    Ok(())
}
