use std::collections::HashMap;

use crate::models::dto;

use super::update_backup;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;

#[actix_web::get("/api/repo/{id}/map")]
pub async fn get_map(
    pool: web::Data<PgPool>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let all_rec = sqlx::query!(
        "SELECT 
            c.id AS repo_id,
            c.git,
            c.log,
            c.digest,
            c.update_time,
            c.updating,
            n.id,
            n.name,
            n.image
        FROM ci_jobs c
        JOIN notebooks n ON c.id = n.repo_id
        "
    )
    .fetch_all(&**pool)
    .await?;

    let mut repo_map = HashMap::<i32, dto::Repo>::new();

    for rec in all_rec {
        let new_nb = dto::Notebook {
            id: rec.id.unwrap(),
            name: rec.name.unwrap().to_string(),
            image: rec.image,
            repo_id: rec.repo_id,
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
                    git: rec.git.unwrap(),
                    updating: rec.updating.unwrap(),
                    log: rec.log,
                    digest: rec.digest,
                    update_time: rec.update_time.map(|t| t.to_rfc3339()),
                    notebooks: Some(vec![new_nb]),
                },
            );
        }
    }

    Ok("good")
}

#[actix_web::get("/api/repo/{id}")]
pub async fn get_repo(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let rec = sqlx::query!(
        "SELECT 
            id,
            git,
            log,
            digest,
            update_time,
            updating
        FROM ci_jobs WHERE id = $1",
        &id.clone()
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(&dto::Repo {
        id: rec.id,
        git: rec.git.unwrap(),
        updating: rec.updating.unwrap(),
        log: rec.log,
        digest: rec.digest,
        update_time: rec.update_time.map(|t| t.to_rfc3339()),
        notebooks: None,
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

#[actix_web::post("/api/repo/{id}/log")]
pub async fn add_log(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res1 = sqlx::query!("SELECT log FROM ci_jobs WHERE id=$1", &id.clone())
        .fetch_one(&**pool)
        .await?;

    let res = sqlx::query!(
        "UPDATE ci_jobs SET log = $1 WHERE id=$2",
        &res1.log.unwrap(),
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

#[actix_web::get("/api/repo/{id}/log")]
pub async fn get_log(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!("SELECT log FROM ci_jobs WHERE id=$1", &id.into_inner())
        .fetch_one(&**pool)
        .await?;

    Ok(HttpResponse::Accepted().body(res.log.unwrap()))
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

pub async fn put_repo(pool: &PgPool, git: &String) -> anyhow::Result<i32> {
    sqlx::query!(
        "INSERT INTO ci_jobs(git) VALUES ($1) 
        ON CONFLICT(git) 
        DO NOTHING
        RETURNING id",
        git
    )
    .fetch_one(pool)
    .await
    .map(|r| r.id)
    .map_err(|e| anyhow::Error::from(e))
}
