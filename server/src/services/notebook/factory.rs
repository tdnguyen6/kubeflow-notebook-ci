use std::collections::HashMap;

use actix_web::{web, HttpResponse};
use serde::{Deserialize};
use sqlx::{PgPool};

use crate::config::Config;
use crate::models::dto;
use crate::services::repo::put_repo;
use crate::utils;

#[derive(Debug, Deserialize)]
pub struct Notebook {
    name: String,
    namespace: String,
    image: Option<String>,
    // git://[repository url][#reference][#commit-id]
    repo: String,
}

#[actix_web::put("/api/notebook")]
pub async fn put(
    config: web::Data<Config>,
    pool: web::Data<PgPool>,
    nb: web::Json<Notebook>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let image_name: &String;
    let tmp_name;

    if let Some(imagex) = &nb.image {
        image_name = imagex;
    } else {
        tmp_name = format!(
            "{}:{}",
            &config.default_image_name,
            uuid::Uuid::new_v4().to_simple()
        );
        image_name = &tmp_name;
    }

    let res = sqlx::query!(
        "INSERT INTO notebooks(name, namespace, image, repo_id) 
            VALUES ($1, $2, $3, $4) 
        ON CONFLICT (namespace, name) DO UPDATE SET
            image = EXCLUDED.image,
            repo_id = EXCLUDED.repo_id
        RETURNING id",
        &nb.name,
        &nb.namespace,
        &image_name,
        &put_repo(&**pool, &nb.repo).await?
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
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT n.id FROM ci_jobs c 
        JOIN notebooks n ON c.id = n.repo_id
        WHERE n.id = $1",
        &id.clone()
    )
    .fetch_all(&**pool)
    .await?;

    if res.len() <= 1 {
        let res = sqlx::query!(
            "DELETE FROM ci_jobs
            WHERE id = (
                SELECT repo_id FROM notebooks
                WHERE id = $1
            )",
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
    } else {
        let res = sqlx::query!("DELETE FROM notebooks WHERE id=$1", &id.into_inner())
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

#[actix_web::get("/api/notebook/{id}")]
pub async fn get(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let res = sqlx::query!(
        "SELECT 
            id,
            name,
            image,
            repo_id
        FROM notebooks WHERE id = $1",
        &id.clone()
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(&dto::Notebook {
        id: res.id,
        name: res.name.unwrap(),
        image: res.image,
        repo_id: res.repo_id,
    }))
}

#[actix_web::get("/api/notebook/list/{namespace}")]
pub async fn get_all(
    pool: web::Data<PgPool>,
    namespace: web::Path<String>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    let mut nb: HashMap<String, dto::Notebook> = HashMap::from_iter(
        utils::kube_notebooks(&namespace)?
            .iter()
            .map(|s| (s.to_string(), dto::Notebook::from_name(s)))
    );

    let res = sqlx::query!(
        "SELECT 
            id,
            name,
            image,
            repo_id
        FROM notebooks WHERE namespace = $1",
        &namespace.clone()
    )
    .fetch_all(&**pool)
    .await?;

    for rec in res {
        let nb_name = rec.name.unwrap();
        if nb.contains_key(&nb_name) {
            (*nb.get_mut(&nb_name).unwrap()).id = rec.id;
            (*nb.get_mut(&nb_name).unwrap()).image = rec.image;
            (*nb.get_mut(&nb_name).unwrap()).repo_id = rec.repo_id;
        }
    }

    Ok(HttpResponse::Ok().json(nb.values().collect::<Vec<&dto::Notebook>>()))
}
