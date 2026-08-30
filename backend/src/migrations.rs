use sqlx::{migrate::Migrator, SqlitePool};

// SQLx records migration checksums in the product's durable SQLite file.
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub(crate) async fn up(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

#[cfg(test)]
pub(crate) async fn down(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.undo(pool, 0).await
}
