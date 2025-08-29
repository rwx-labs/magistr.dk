use std::ops::Deref;
use std::time::Duration;

use cached::proc_macro::{cached, once};
use sqlx::{
    migrate::Migrator,
    postgres::{PgPool, PgPoolOptions},
};
use tracing::{debug, instrument, trace};

use crate::Error;
use crate::config;
use crate::models;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Debug, Clone)]
pub struct Database(PgPool);

impl Deref for Database {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn connect(config: &config::Database) -> Result<Database, Error> {
    debug!("connecting to database");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .idle_timeout(config.idle_timeout)
        .connect(config.url.as_str())
        .await
        .map_err(Error::DatabaseOpenError)
        .map(Database)?;

    debug!("connected to database");

    Ok(pool)
}

pub async fn migrate(pool: Database) -> Result<(), Error> {
    debug!("running database migrations");
    let mut conn = pool.acquire().await.map_err(Error::DatabaseConnAcqError)?;

    MIGRATOR
        .run(&mut conn)
        .await
        .inspect(|()| debug!("finished running database migrations"))
        .map_err(Error::DatabaseMigrationError)
}

#[instrument(skip(database))]
#[once(time = 300, option = true, sync_writes = true)]
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

#[instrument(skip(database))]
#[cached(
    time = 1800,
    key = "i32",
    convert = r#"{ quote_id }"#,
    sync_writes = "default"
)]
pub async fn get_quote(database: &Database, quote_id: i32) -> Option<models::Quote> {
    trace!(%quote_id, "querying specific quote");

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
    pub async fn create_quote(&self, quote: models::NewQuote) -> Result<Option<i32>, Error> {
        trace!("querying specific quote");

        let quote_id =
            sqlx::query_scalar("INSERT INTO quotes (date, text) VALUES ($1, $2) RETURNING id")
                .bind(quote.date)
                .bind(quote.text)
                .fetch_optional(&self.0)
                .await?;

        Ok(quote_id)
    }
}
