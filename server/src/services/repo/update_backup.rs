use sqlx::PgPool;

async fn backup_build_log(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT build_log FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET last_build_log = $1 WHERE id=$2",
        res.build_log.unwrap_or_default(),
        &id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET build_log = '' WHERE id=$1",
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn backup_track_log(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT track_log FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET last_track_log = $1 WHERE id=$2",
        res.track_log.unwrap_or_default(),
        &id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET track_log = '' WHERE id=$1",
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn backup_digest(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT digest FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET last_digest = $1 WHERE id=$2",
        res.digest.unwrap_or_default(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn recover_build_log(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT last_build_log FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET build_log = $1 WHERE id=$2",
        res.last_build_log.unwrap_or_default(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// async fn recover_track_log(id: i32, pool: &PgPool) -> anyhow::Result<()> {
//     let res = sqlx::query!("SELECT last_track_log FROM ci_jobs WHERE id=$1", &id)
//         .fetch_one(pool)
//         .await?;

//     sqlx::query!(
//         "UPDATE ci_jobs SET track_log = $1 WHERE id=$2",
//         res.last_track_log.unwrap_or_default(),
//         &id
//     )
//     .execute(pool)
//     .await?;

//     Ok(())
// }

async fn recover_digest(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT last_digest FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET digest = $1 WHERE id=$2",
        res.last_digest.unwrap_or_default(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn backup(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    backup_build_log(id, pool).await?;
    // backup_track_log(id, pool).await?;
    backup_digest(id, pool).await?;
    Ok(())
}

pub async fn recover(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    recover_build_log(id, pool).await?;
    // recover_track_log(id, pool).await?;
    recover_digest(id, pool).await?;
    Ok(())
}
