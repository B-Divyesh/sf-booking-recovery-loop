use sqlx::{Executor, SqlitePool};

const UP_1: &str = include_str!("../migrations/0001_demo_workspaces.up.sql");
const UP_2: &str = include_str!("../migrations/0002_demo_token_aliases.up.sql");
#[cfg(test)]
const DOWN_1: &str = include_str!("../migrations/0001_demo_workspaces.down.sql");
#[cfg(test)]
const DOWN_2: &str = include_str!("../migrations/0002_demo_token_aliases.down.sql");

pub(crate) async fn up(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    transaction
        .execute(
            "CREATE TABLE IF NOT EXISTS brl_schema_migrations (\
             version INTEGER PRIMARY KEY NOT NULL, applied_at INTEGER NOT NULL)",
        )
        .await?;
    for (version, sql) in [(1_i64, UP_1), (2_i64, UP_2)] {
        let applied: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM brl_schema_migrations WHERE version = ?")
                .bind(version)
                .fetch_one(&mut *transaction)
                .await?;
        if applied == 0 {
            transaction.execute(sql).await?;
            sqlx::query(
                "INSERT INTO brl_schema_migrations (version, applied_at) \
             VALUES (?, CAST(strftime('%s', 'now') AS INTEGER))",
            )
            .bind(version)
            .execute(&mut *transaction)
            .await?;
        }
    }
    transaction.commit().await?;
    Ok(())
}

#[cfg(test)]
pub(crate) async fn down(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    transaction.execute(DOWN_2).await?;
    transaction.execute(DOWN_1).await?;
    transaction
        .execute("DROP TABLE IF EXISTS brl_schema_migrations")
        .await?;
    transaction.commit().await?;
    Ok(())
}
