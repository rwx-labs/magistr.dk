use ::tracing::debug;
use figment::{
    Figment,
    providers::{Env, Serialized},
};
use miette::IntoDiagnostic;

mod config;
mod database;
mod error;
mod http;
mod models;
mod templates;
mod tracing;

pub use error::Error;

use crate::config::Config;

#[derive(Clone)]
struct AppState {
    pub database: database::Database,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Env::prefixed("MAGISTR_").split("_"))
        .extract()
        .into_diagnostic()?;

    tracing::try_init(&config.tracing)?;

    debug!("preparing database");
    let database = database::connect(&config.database).await?;
    let app_state = AppState {
        database: database.clone(),
    };

    database::migrate(database.clone()).await?;
    debug!("finished database preparations");

    http::serve(&config.http, app_state).await?;

    Ok(())
}
