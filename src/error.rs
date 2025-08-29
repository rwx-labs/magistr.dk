//! Error types

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("cannot connect to database: {0}")]
    #[diagnostic(code(magistr::db_open))]
    DatabaseOpenError(#[source] sqlx::Error),

    #[error("could not acquire a connection from the connection pool")]
    DatabaseConnAcqError(#[source] sqlx::Error),

    #[error("database migration failed")]
    DatabaseMigrationError(#[source] sqlx::migrate::MigrateError),

    #[error("database query failed")]
    DatabaseQueryFailed(#[from] sqlx::Error),

    #[error("missing expected environment variable `{0}'")]
    MissingEnv(String),
    #[error("could not install global tracing subscriber")]
    TracingTryInit(#[from] tracing_subscriber::util::TryInitError),
    #[error("could not build opentelemetry span exporter")]
    BuildOtelExporter(#[source] Box<dyn std::error::Error + Sync + Send>),
}
