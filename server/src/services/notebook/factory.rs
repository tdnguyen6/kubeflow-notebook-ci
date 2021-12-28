use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::config::Config;
use crate::models::dto;
use crate::services::repo::put_repo;
use crate::utils;

#[actix_web::put("/api/notebook")]
pub async fn put(
    pool: web::Data<PgPool>,
    nb: web::Json<dto::NotebookData>,
    nb_id: web::Query<dto::NotebookId>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "INSERT INTO notebooks(name, namespace, repo_id, image, private_registry, registry, registry_credential_secret, auto_sync) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
        ON CONFLICT (namespace, name) DO UPDATE SET
            image = EXCLUDED.image,
            repo_id = EXCLUDED.repo_id,
            registry_credential_secret = EXCLUDED.registry_credential_secret,
            private_registry = EXCLUDED.private_registry,
            registry = EXCLUDED.registry,
            auto_sync = EXCLUDED.auto_sync
        RETURNING id",
        &nb_id.name,
        &nb_id.namespace,
        &put_repo(&**pool, &nb.repo_uri, if nb.private_registry { &nb_id.namespace } else { "" }, nb.private_repo, &nb.repo_credential_secret, &nb.dockerfile).await?,
        &nb.image,
        &nb.private_registry,
        &nb.registry,
        &nb.registry_credential_secret,
        &nb.auto_sync
    )
    .fetch_one(&**pool)
    .await;

    match res {
        Ok(rec) => Ok(HttpResponse::Ok().body(format!("{}", rec.id))),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[actix_web::delete("/api/notebook")]
pub async fn remove(
    pool: web::Data<PgPool>,
    nb_id: web::Query<dto::NotebookId>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT n.id FROM ci_jobs c 
        JOIN notebooks n ON c.id = n.repo_id
        WHERE n.name = $1 AND n.namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .fetch_all(&**pool)
    .await?;

    if res.len() <= 1 {
        let res = sqlx::query!(
            "DELETE FROM ci_jobs
            WHERE id = (
                SELECT repo_id FROM notebooks
                WHERE name = $1 AND namespace = $2
            )",
            &nb_id.name,
            &nb_id.namespace,
        )
        .execute(&**pool)
        .await;

        if let Ok(r) = res {
            if r.rows_affected() == 1 {
                return Ok(HttpResponse::Ok().finish());
            }
            return Ok(HttpResponse::InternalServerError().finish());
        }
    } else {
        let res = sqlx::query!(
            "DELETE FROM notebooks WHERE name = $1 AND namespace = $2",
            &nb_id.name,
            &nb_id.namespace,
        )
        .execute(&**pool)
        .await;

        if let Ok(r) = res {
            if r.rows_affected() == 1 {
                return Ok(HttpResponse::Ok().finish());
            }
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::get("/api/notebook")]
pub async fn get(
    pool: web::Data<PgPool>,
    nb_id: web::Query<dto::NotebookId>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT 
            n.id,
            n.name,
            n.namespace,
            n.image,
            n.private_registry,
            n.registry,
            n.registry_credential_secret,
            n.repo_id,
            n.auto_sync,
            n.syncing,
            c.uri,
            c.private_repo,
            c.dockerfile,
            c.repo_credential_secret
        FROM notebooks n
        JOIN ci_jobs c
            ON n.repo_id = c.id
        WHERE n.name = $1 AND n.namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(&dto::Notebook {
        id: res.id,
        nb_id: dto::NotebookId {
            name: res.name.unwrap_or_default(),
            namespace: res.namespace.unwrap_or_default(),
        },
        nb_data: dto::NotebookData {
            image: res.image.unwrap_or_default(),
            private_registry: res.private_registry.unwrap_or_default(),
            registry: res.registry.unwrap_or_default(),
            repo_id: res.repo_id.unwrap_or_default(),
            repo_uri: res.uri.unwrap_or_default(),
            private_repo: res.private_repo.unwrap_or_default(),
            registry_credential_secret: res.registry_credential_secret.unwrap_or_default(),
            repo_credential_secret: res.repo_credential_secret.unwrap_or_default(),
            auto_sync: res.auto_sync.unwrap_or_default(),
            dockerfile: res.dockerfile,
            syncing: res.syncing.unwrap_or_default(),
        },
    }))
}

#[actix_web::get("/api/notebook/restart_pod")]
pub async fn restart_pod(
    pool: web::Data<PgPool>,
    nb_id: web::Query<dto::NotebookId>,
    config: web::Data<Config>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT * FROM notebooks WHERE name=$1 AND namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .fetch_all(&**pool)
    .await?;

    if res.len() == 0 {
        return Ok(HttpResponse::NotFound().body("this notebook is not enabled"));
    }

    let res = sqlx::query!(
        "UPDATE notebooks SET syncing = true WHERE name=$1 AND namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            let rec = sqlx::query!(
                "SELECT private_registry, registry_credential_secret, image FROM notebooks WHERE name=$1 AND namespace = $2",
                &nb_id.name,
                &nb_id.namespace,
            )
            .fetch_one(&**pool)
            .await?;

            utils::restart_nb_pod(
                &nb_id,
                &rec.private_registry.unwrap_or_default(),
                &rec.registry_credential_secret.unwrap_or_default(),
                &rec.image.unwrap_or_default(),
                &config,
            )?;

            let res = sqlx::query!(
                "UPDATE notebooks SET syncing = false WHERE name=$1 AND namespace = $2",
                &nb_id.name,
                &nb_id.namespace,
            )
            .execute(&**pool)
            .await;

            if let Ok(r) = res {
                if r.rows_affected() == 1 {
                    return Ok(HttpResponse::Ok().finish());
                }
            }
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::put("/api/notebook/reset-push_log")]
pub async fn reset_push_log(
    pool: web::Data<PgPool>,
    nb_id: web::Query<dto::NotebookId>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    sqlx::query!(
        "UPDATE notebooks SET last_push_log = push_log WHERE name = $1 AND namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .execute(&**pool)
    .await?;

    let res = sqlx::query!(
        "UPDATE notebooks SET push_log = '' WHERE name = $1 AND namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::post("/api/notebook/push_log")]
pub async fn add_push_log(
    pool: web::Data<PgPool>,
    nb_id: web::Query<dto::NotebookId>,
    body: String,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let current_log = sqlx::query!(
        "SELECT push_log FROM notebooks WHERE name = $1 AND namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .fetch_one(&**pool)
    .await?
    .push_log
    .unwrap_or_default();

    let res = sqlx::query!(
        "UPDATE notebooks SET push_log = $1 WHERE name = $2 AND namespace = $3",
        &format!("{}{}\n", &current_log, &body),
        &nb_id.name,
        &nb_id.namespace,
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::get("/api/notebook/push_log")]
pub async fn get_push_log(
    pool: web::Data<PgPool>,
    nb_id: web::Query<dto::NotebookId>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT push_log FROM notebooks WHERE name = $1 AND namespace = $2",
        &nb_id.name,
        &nb_id.namespace,
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Accepted().body(res.push_log.unwrap_or_default()))
}
