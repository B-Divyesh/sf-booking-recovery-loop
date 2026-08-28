use sqlx::{Executor, SqlitePool};

const UP: &str = include_str!("../migrations/0001_demo_workspaces.up.sql");
#[cfg(test)]
const DOWN: &str = include_str!("../migrations/0001_demo_workspaces.down.sql");

pub(crate) async fn up(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    transaction
        .execute(
            "CREATE TABLE IF NOT EXISTS brl_schema_migrations (\
             version INTEGER PRIMARY KEY NOT NULL, applied_at INTEGER NOT NULL)",
        )
        .await?;
    let applied: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM brl_schema_migrations WHERE version = 1")
            .fetch_one(&mut *transaction)
            .await?;
    if applied == 0 {
        transaction.execute(UP).await?;
        sqlx::query(
            "INSERT INTO brl_schema_migrations (version, applied_at) \
             VALUES (1, CAST(strftime('%s', 'now') AS INTEGER))",
        )
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

#[cfg(test)]
pub(crate) async fn down(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    transaction.execute(DOWN).await?;
    transaction
        .execute("DROP TABLE IF EXISTS brl_schema_migrations")
        .await?;
    transaction.commit().await?;
    Ok(())
}
