use std::{path::Path, sync::Arc};
use thiserror::Error;
use crate::core::system::trash_manager::error::TrashError;


#[derive(Debug, Error)]
pub enum ClipBoardError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error con el nombre: {0}")]
    InvalidName(Arc<str>),

    #[error("El archivo {0} ya existe")]
    AlreadyExist(Arc<str>),

    #[error("Error con la papelera: {0}")]
    TrashError(#[from] TrashError),

    #[error("Permiso denegado: {0}")]
    PermissionDenied(Arc<Path>),

    #[error("Rename falló: No es Cross-Device")]
    CrossDeviceError,

    #[error("No hay destino")]
    NoDestError,

    #[error("Error de lock envenenado")]
    PoisonedLock,
}

pub type ClipBoardResult<T> = Result<T, ClipBoardError>;