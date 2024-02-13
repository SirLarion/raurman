use std::{io, env};

use thiserror::Error;
use serde_json::Error as SerdeError;

#[derive(Error, Debug)]
pub enum AppError {
  #[error(transparent)]
  IoError(#[from] io::Error),

  #[error(transparent)]
  JSONError(#[from] SerdeError),

  #[error(transparent)]
  EnvError(#[from] env::VarError),
}
