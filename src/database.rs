use sqlx::{
    migrate::Migrator,
    postgres::{PgPool, PgPoolOptions},
};

use crate::Error;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn connect(url: &str) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .map_err(Error::DatabaseOpenError)?;

    Ok(pool)
}

pub async fn migrate(pool: PgPool) -> Result<(), Error> {
    MIGRATOR
        .run(&pool)
        .await
        .map_err(Error::DatabaseMigrationError)
}
