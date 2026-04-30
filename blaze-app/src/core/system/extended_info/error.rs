use git2::Error as GitErr;
use thiserror::Error;

use crate::utils::channel_pool::FileOperation;


#[derive(Error, Debug)]
pub enum ExtendedInfoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error en thread Tokio: {0}")]
    ThreadError(#[from] tokio::task::JoinError),

    #[error("Error leyendo metadatos de archivo")]
    MetadataError,

    #[error("Error obteniendo estado git: {0}")]
    GitError(#[from] GitErr),

    #[error("Error resolviendo symlink")]
    SymlinkError,

    #[error("Error obteniendo dimensiones de imagen")]
    DimensionError,

    #[error("Error de lock envenenado")]
    PoisonedLock,

    #[error("SenderError: {0}")]
    SendError(#[from] crossbeam_channel::SendError<FileOperation>),

    #[error("Error dividiendo el path: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError)
}

pub type ExtendedInfoResult<T> = Result<T, ExtendedInfoError>;