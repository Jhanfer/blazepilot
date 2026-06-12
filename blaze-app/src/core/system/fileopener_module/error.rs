use std::{path::Path, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenerError {
    #[error("{msg}: {path} - {source}")]
    Io {
        path: Arc<Path>,
        msg: Box<str>,
        #[source]
        source: std::io::Error,
    },

    #[error("{msg}: {path} - {source}")]
    TomlError {
        path: Arc<Path>,
        msg: Box<str>,
        #[source]
        source: toml::ser::Error,
    },

    #[error("Ejecutable bloqueado por política de seguridad: '{name}'")]
    BlockedExecutable { name: Box<str> },

    #[error("Error parseando argumentos de Exec: {error}")]
    ExecParseFailed { error: shell_words::ParseError },

    #[error("Exec vacío")]
    ArgsEmpty,

    #[error("No se han encontrado la aplicación")]
    NoAppFound,

    #[error("xdg-mime ha retornado error")]
    XDGMimeError,
}

pub type OpenerResult<T> = Result<T, OpenerError>;
