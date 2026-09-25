use crate::core::files::blaze_motor::motor_structs::FileLoadingMessage;
use std::{path::Path, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MotorError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("Error en trhead Tokio: {0}")]
    ThreadError(#[from] tokio::task::JoinError),

    #[error("SenderError: {0}")]
    SendFileBatchError(#[from] crossbeam_channel::SendError<FileLoadingMessage>),

    #[error("Error de canal oneshot: {0}")]
    OneshotRecv(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Directorio no existe: {0}")]
    InvalidPath(Arc<Path>),

    #[error("No se ha encontrado FileSource para esta ruta: {0}")]
    NotFileSourceFound(Arc<Path>),

    #[error("{0}")]
    Error(String),
}

impl From<&str> for MotorError {
    fn from(value: &str) -> Self {
        let message = value.to_owned();
        self::MotorError::Error(message)
    }
}

pub type MotorResult<T> = Result<T, MotorError>;
