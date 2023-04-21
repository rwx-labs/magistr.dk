use std::ops::Deref;

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

impl Database {
    #[instrument(err, skip(self))]
    pub async fn get_quote(&self, quote_id: i32) -> Result<Option<models::Quote>, Error> {
        trace!("querying specific quote");

        let quote: Option<models::Quote> = sqlx::query_as("SELECT * FROM quotes WHERE id = $1")
            .bind(quote_id)
            .fetch_optional(&self.0)
            .await?;

        trace!(result = ?quote);

        Ok(quote)
    }

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

    #[instrument(err, skip_all)]
    pub async fn get_quotes(&self) -> Result<Vec<models::Quote>, Error> {
        trace!("reading all quotes from database");

        let quotes: Vec<models::Quote> = sqlx::query_as("SELECT * FROM quotes ORDER BY id DESC")
            .fetch_all(&self.0)
            .await?;

        trace!(num_results = quotes.len());

        Ok(quotes)
    }
}
