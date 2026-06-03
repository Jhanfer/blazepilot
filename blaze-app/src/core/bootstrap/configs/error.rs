use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error en thread Tokio: {0}")]
    ThreadError(#[from] tokio::task::JoinError),

    #[error("Error dividiendo el path: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),

    #[error("No se ha encontrado la ruta de configuración")]
    ProjectDirsNotFound,

    #[error("Error al deserializar el 'config.json'")]
    Deserialize,

    #[error("Error al serializar el 'config.json'")]
    Serialize,
}

pub type ConfigResult<T> = Result<T, ConfigError>;
