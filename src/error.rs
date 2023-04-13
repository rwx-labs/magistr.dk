//! Error types

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("Cannot connect to database database")]
    #[diagnostic(code(redirekt::db_open))]
    DatabaseOpenError(#[source] sqlx::Error),

    #[error("Database migration failed")]
    DatabaseMigrationError(#[source] sqlx::migrate::MigrateError),
}
