use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("axum got an error: {0}")]
    Axum(#[from] axum::Error),
    #[error("tcp got an error: {0}")]
    Tcp(#[from] io::Error),
}
