use actix_web::{get, web, Responder};

use crate::{config::Config, utils};

#[get("/api/reconcile")]
async fn reconcile(
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<Config>,
) -> actix_web::Result<impl Responder, Box<dyn std::error::Error>> {
    let namespaces = utils::all_kf_users_namespaces(&config)?;
    let res = sqlx::query!(
        "DELETE FROM notebooks WHERE namespace != ALL($1)",
        &namespaces[..]
    )
    .execute(&**pool)
    .await?;

    let res2 = sqlx::query!(
        "DELETE FROM ci_jobs WHERE id NOT IN (SELECT DISTINCT repo_id FROM notebooks)"
    )
    .execute(&**pool)
    .await?;

    Ok(format!("{}", res.rows_affected() + res2.rows_affected()))
}
