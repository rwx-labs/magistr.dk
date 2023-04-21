use std::ops::Deref;

use cached::proc_macro::{cached, once};
use sqlx::{
    migrate::Migrator,
    postgres::{PgPool, PgPoolOptions},
};
use tracing::{instrument, trace};

use crate::models;
use crate::Error;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Debug, Clone)]
pub struct Database(PgPool);

impl Deref for Database {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn connect(url: &str) -> Result<Database, Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .map_err(Error::DatabaseOpenError)
        .map(Database)?;

    Ok(pool)
}

pub async fn migrate(pool: Database) -> Result<(), Error> {
    let mut conn = pool.acquire().await.map_err(Error::DatabaseConnAcqError)?;

    MIGRATOR
        .run(&mut conn)
        .await
        .map_err(Error::DatabaseMigrationError)
}

#[once(time = 120, option = true, sync_writes = true)]
pub async fn get_quotes(database: &Database) -> Option<Vec<models::Quote>> {
    trace!("reading all quotes from database");

    let quotes = sqlx::query_as("SELECT * FROM quotes ORDER BY id DESC")
        .fetch_all(&database.0)
        .await
        .ok();

    if let Some(ref qs) = quotes {
        trace!(num_results = qs.len());
    }

    quotes
}

#[cached(time = 1800, key = "i32", convert = r#"{ quote_id }"#)]
pub async fn get_quote(database: &Database, quote_id: i32) -> Option<models::Quote> {
    trace!("querying specific quote");

    let quote: Option<models::Quote> = sqlx::query_as("SELECT * FROM quotes WHERE id = $1")
        .bind(quote_id)
        .fetch_optional(&database.0)
        .await
        .ok()
        .flatten();

    trace!(result = ?quote);

    quote
}

impl Database {
    #[instrument(err, skip_all)]
    pub async fn create_quote(&self, quote: models::NewQuote) -> Result<(), Error> {
        trace!("querying specific quote");

        sqlx::query("INSERT INTO quotes (date, text) VALUES ($1, $2)")
            .bind(quote.date)
            .bind(quote.text)
            .execute(&self.0)
            .await?;

        Ok(())
    }
}
