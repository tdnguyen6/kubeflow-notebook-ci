use actix_web::{post, web, Responder};

use crate::{config::Config, utils};

#[post("/api/reconcile")]
async fn reconcile(
    pool: web::Data<sqlx::PgPool>,
    _config: web::Data<Config>,
) -> actix_web::Result<impl Responder, Box<dyn std::error::Error>> {
    let namespaces = utils::all_kf_users_namespaces()?;
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
