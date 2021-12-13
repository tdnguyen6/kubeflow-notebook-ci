 use sqlx::PgPool;

async fn backup_log(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT log FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET last_log = $1 WHERE id=$2",
        res.log.unwrap(),
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
        res.digest.unwrap(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn backup_update_time(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT update_time FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET last_update_time = $1 WHERE id=$2",
        res.update_time.unwrap(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn recover_log(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT last_log FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET log = $1 WHERE id=$2",
        res.last_log.unwrap(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn recover_digest(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT last_digest FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET digest = $1 WHERE id=$2",
        res.last_digest.unwrap(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn recover_update_time(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    let res = sqlx::query!("SELECT last_update_time FROM ci_jobs WHERE id=$1", &id)
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "UPDATE ci_jobs SET update_time = $1 WHERE id=$2",
        res.last_update_time.unwrap(),
        &id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn backup(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    backup_log(id, pool).await?;
    backup_digest(id, pool).await?;
    backup_update_time(id, pool).await?;
    Ok(())
}

pub async fn recover(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    recover_log(id, pool).await?;
    recover_digest(id, pool).await?;
    recover_update_time(id, pool).await?;
    Ok(())
}
