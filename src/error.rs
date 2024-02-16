use std::{io, env, process::ExitStatus};

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

  #[error("executing command failed")]
  CmdError(ExitStatus)
}
