use sqlx::{migrate::Migrator, AnyPool};

// SQLx records migration checksums and applies one ordered set to SQLite for
// local/demo work and PostgreSQL for the production shared store.
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub(crate) async fn up(pool: &AnyPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

#[cfg(test)]
pub(crate) async fn down(pool: &AnyPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.undo(pool, 0).await
}
