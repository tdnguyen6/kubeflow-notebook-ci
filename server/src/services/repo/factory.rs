use std::collections::HashMap;

use crate::{models::dto, utils};

use super::update_backup;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;

#[actix_web::get("/api/repo/map")]
pub async fn get_map(
    pool: web::Data<PgPool>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let all_rec = sqlx::query!(
        "SELECT 
            c.id AS repo_id,
            c.uri,
            c.secret_namespace,
            c.build_log,
            c.track_log,
            c.digest,
            c.private_repo,
            c.repo_credential_secret,
            c.updating,
            n.name,
            n.namespace
        FROM ci_jobs c
        JOIN notebooks n ON c.id = n.repo_id
        WHERE c.should_track = true
        "
    )
    .fetch_all(&**pool)
    .await?;

    let mut repo_map = HashMap::<i32, dto::Repo>::new();

    for rec in all_rec {
        let new_nb = dto::NotebookId {
            name: rec.name.unwrap().to_string(),
            namespace: rec.namespace.unwrap(),
        };
        let repo_id = rec.repo_id.unwrap();
        if repo_map.contains_key(&repo_id) {
            let onb = &mut (*repo_map.get_mut(&repo_id).unwrap()).notebooks;
            if let Some(nb) = onb {
                nb.push(new_nb);
            } else {
                (*repo_map.get_mut(&repo_id).unwrap()).notebooks = Some(vec![new_nb])
            }
        } else {
            repo_map.insert(
                repo_id,
                dto::Repo {
                    id: repo_id,
                    uri: rec.uri.unwrap(),
                    updating: rec.updating.unwrap(),
                    private_repo: rec.private_repo.unwrap(),
                    secret_namespace: rec.secret_namespace.unwrap(),
                    repo_credential_secret: rec.repo_credential_secret.unwrap(),
                    build_log: rec.build_log,
                    track_log: rec.track_log,
                    digest: rec.digest,
                    notebooks: Some(vec![new_nb]),
                },
            );
        }
    }

    Ok(HttpResponse::Ok().json(repo_map))
}

#[actix_web::get("/api/repo/{id}")]
pub async fn get_repo(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let rec = sqlx::query!(
        "SELECT 
            id,
            uri,
            secret_namespace,
            build_log,
            track_log,
            digest,
            private_repo,
            repo_credential_secret,
            updating
        FROM ci_jobs WHERE id = $1",
        &id.clone()
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(&dto::Repo {
        id: rec.id,
        uri: rec.uri.unwrap_or_default(),
        secret_namespace: rec.secret_namespace.unwrap_or_default(),
        updating: rec.updating.unwrap_or_default(),
        private_repo: rec.private_repo.unwrap_or_default(),
        repo_credential_secret: rec.repo_credential_secret.unwrap_or_default(),
        build_log: rec.build_log.unwrap_or_default(),
        track_log: rec.track_log.unwrap_or_default(),
        digest: rec.digest.unwrap_or_default(),
        notebooks: Vec::default(),
    }))
}

#[actix_web::patch("/api/repo/{id}/start_update")]
pub async fn start_update(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    if let Err(_) = update_backup::backup(id.clone(), &pool).await {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let res = sqlx::query!(
        "UPDATE ci_jobs SET updating = true WHERE id=$1",
        &id.into_inner()
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::patch("/api/repo/{id}/end_update")]
pub async fn end_update(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "UPDATE ci_jobs SET updating = false WHERE id=$1",
        &id.into_inner()
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::patch("/api/repo/{id}/revert_update")]
pub async fn revert_update(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    Ok(match update_backup::recover(id.into_inner(), &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    })
}

#[actix_web::get("/api/repo/{id}/updating")]
pub async fn updating(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!("SELECT updating FROM ci_jobs WHERE id=$1", &id.into_inner())
        .fetch_one(&**pool)
        .await?;

    #[derive(Serialize)]
    struct Out {
        updating: bool,
    }

    Ok(HttpResponse::Accepted().json(Out {
        updating: res.updating.unwrap(),
    }))
}

#[actix_web::post("/api/repo/{id}/build_log")]
pub async fn add_build_log(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    body: String,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "UPDATE ci_jobs SET build_log = build_log || $1 WHERE id=$2",
        &format!("{}\n", &body),
        &id.into_inner()
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::get("/api/repo/{id}/build_log")]
pub async fn get_build_log(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT build_log FROM ci_jobs WHERE id=$1",
        &id.into_inner()
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Accepted().body(res.build_log.unwrap()))
}

#[actix_web::post("/api/repo/{id}/track_log")]
pub async fn add_track_log(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    body: String,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    update_backup::backup_track_log(id.clone(), &**pool).await?;

    let res = sqlx::query!(
        "UPDATE ci_jobs SET track_log = track_log || $1 WHERE id=$2",
        &format!("{}\n", &body),
        &id.into_inner()
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::get("/api/repo/{id}/track_log")]
pub async fn get_track_log(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT track_log FROM ci_jobs WHERE id=$1",
        &id.into_inner()
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Accepted().body(res.track_log.unwrap()))
}

#[actix_web::post("/api/repo/{id}/image_digest/{digest}")]
pub async fn set_image_digest(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    digest: web::Path<String>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "UPDATE ci_jobs SET digest = $1 WHERE id=$2",
        &digest.into_inner(),
        &id.into_inner()
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::InternalServerError().finish())
}

#[actix_web::get("/api/repo/{id}/image_digest")]
pub async fn get_image_digest(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!("SELECT digest FROM ci_jobs WHERE id=$1", &id.into_inner())
        .fetch_one(&**pool)
        .await?;

    Ok(HttpResponse::Accepted().body(res.digest.unwrap()))
}

pub async fn put_repo(
    pool: &PgPool,
    uri: &str,
    secret_namespace: &str,
    private_repo: bool,
    repo_credential_secret: &str,
) -> anyhow::Result<i32> {
    sqlx::query!(
        "INSERT INTO ci_jobs(uri, secret_namespace, private_repo, repo_credential_secret) 
        VALUES ($1, $2, $3, $4) 
        ON CONFLICT (uri, secret_namespace) DO UPDATE SET
            uri = EXCLUDED.uri,
            secret_namespace = EXCLUDED.secret_namespace,
            private_repo = EXCLUDED.private_repo,
            repo_credential_secret = EXCLUDED.repo_credential_secret
        RETURNING id",
        uri,
        secret_namespace,
        private_repo,
        repo_credential_secret
    )
    .fetch_one(pool)
    .await
    .map(|r| r.id)
    .map_err(|e| anyhow::Error::from(e))
}

#[actix_web::patch("/api/repo/{id}/should_track/{value}")]
pub async fn should_track(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    value: web::Path<bool>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "UPDATE ci_jobs SET should_track = $2 WHERE id=$1",
        &id.into_inner(),
        &value.into_inner(),
    )
    .execute(&**pool)
    .await;

    if let Ok(r) = res {
        if r.rows_affected() == 1 {
            return Ok(HttpResponse::Ok().finish());
        }
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::InternalServerError().finish())
}
