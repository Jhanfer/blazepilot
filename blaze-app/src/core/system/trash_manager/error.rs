use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrashError {
    #[error("Error de E/S: {0}")]
    Io(#[from] io::Error),
    
    #[error("Directorio no escribible: {path:?}")]
    DirNotWritable { path: PathBuf },
    
    #[error("Ruta inválida (contiene nulo): {0:?}")]
    InvalidPath(PathBuf),
    
    #[error("Operación no soportada en esta plataforma")]
    PlatformNotSupported,
    
    #[error("Archivo no encontrado en papelera: {path:?}")]
    TrashEntryNotFound { path: PathBuf },
    
    #[error("Fallo al restaurar: nombre original inválido")]
    RestoreInvalidName,
    
    #[cfg(target_os = "linux")]
    #[error("Directorio .Trash en montaje externo no tiene sticky bit correcto")]
    TrashDirInvalidPermissions { path: PathBuf },
}

pub type TrashResult<T> = Result<T, TrashError>;