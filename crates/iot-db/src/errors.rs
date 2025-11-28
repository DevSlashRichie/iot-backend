use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("got sqlx error: {0}")]
    Sqlx(#[from] sqlx::error::Error),
    #[error("could not insert")]
    NotInserted,
    #[error("could not migrate: {0}")]
    Migrate(#[from] MigrateError),
}
