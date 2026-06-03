use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuickAccError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No se ha encontrado la ruta de configuración")]
    ProjectDirsNotFound,

    #[error("Error al deserializar el 'config.json'")]
    Deserialize,

    #[error("Error al serializar el 'config.json'")]
    Serialize,
}

pub type QuickAccResult<T> = Result<T, QuickAccError>;
