use std::collections::HashMap;

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
    println!("hello");
    println!("{:#?}", nb);
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
        &put_repo(&**pool, &nb.repo_uri, if nb.private_registry { &nb_id.namespace } else { "" }, nb.private_repo, &nb.repo_credential_secret).await?,
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
            c.uri,
            c.private_repo,
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
            name: res.name.unwrap(),
            namespace: res.namespace.unwrap(),
        },
        nb_data: dto::NotebookData {
            image: res.image.unwrap(),
            private_registry: res.private_registry.unwrap(),
            registry: res.registry.unwrap(),
            repo_id: res.repo_id,
            repo_uri: res.uri.unwrap(),
            private_repo: res.private_repo.unwrap(),
            registry_credential_secret: res.registry_credential_secret.unwrap(),
            repo_credential_secret: res.repo_credential_secret.unwrap(),
            auto_sync: res.auto_sync.unwrap(),
        },
    }))
}

#[actix_web::get("/api/notebook/restart_pod")]
pub async fn restart_pod(
    nb_id: web::Query<dto::NotebookId>,
) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
    utils::restart_nb_pod(&nb_id)?;

    Ok("")
}

// #[actix_web::get("/api/notebook/list/{namespace}")]
// pub async fn get_all(
//     pool: web::Data<PgPool>,
//     namespace: web::Path<String>,
// ) -> actix_web::Result<impl actix_web::Responder, Box<dyn std::error::Error>> {
//     let mut nb: HashMap<String, dto::Notebook> = HashMap::from_iter(
//         utils::kube_notebooks(&namespace)?
//             .iter()
//             .map(|s| (s.to_string(), dto::Notebook::from_name(s))),
//     );

//     let res = sqlx::query!(
//         "SELECT
//             id,
//             name,
//             image,
//             repo_id
//         FROM notebooks WHERE namespace = $1",
//         &namespace.clone()
//     )
//     .fetch_all(&**pool)
//     .await?;

//     for rec in res {
//         let nb_name = rec.name.unwrap();
//         if nb.contains_key(&nb_name) {
//             (*nb.get_mut(&nb_name).unwrap()).id = rec.id;
//             (*nb.get_mut(&nb_name).unwrap()).image = rec.image;
//             (*nb.get_mut(&nb_name).unwrap()).repo_id = rec.repo_id;
//         }
//     }

//     Ok(HttpResponse::Ok().json(nb.values().collect::<Vec<&dto::Notebook>>()))
// }
