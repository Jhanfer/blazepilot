use std::{path::Path, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenerError {
    #[error("No se ha podido leer el archivo: {path} - {source}")]
    Io {
        path: Arc<Path>,
        #[source]
        source: std::io::Error,
    },

    #[error("Error de Serde {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Ejecutable bloqueado por política de seguridad: '{name}'")]
    BlockedExecutable { name: String },

    #[error("Ejecutable no encontrado o sin permisos: '{name}'")]
    ExecutableNotFound { name: String },

    #[error("Exec vacío o inválido en desktop file: {desktop_id}")]
    InvalidExec { desktop_id: String },

    #[error("No se pudo parsear el archivo de escritorio: {path}")]
    DesktopParsedFaild { path: Arc<Path> },

    #[error("Error parseando argumentos de Exec: {raw}")]
    ExecParseFailed { raw: String },

    #[error("Error en el task de tokio: {0}")]
    ThreadError(#[from] tokio::task::JoinError),

    #[error("Formato no soportado")]
    UnsuportedFormat,

    #[error("Error leyendo imagen: {0}")]
    ImageError(#[from] image::error::ImageError),

    #[error("Error leyendo imagen: {0}")]
    SvgError(#[from] resvg::usvg::Error),

    #[error("No hay dimensiones disponibles")]
    TargetDimensionError,
}

pub type OpenerResult<T> = Result<T, OpenerError>;
