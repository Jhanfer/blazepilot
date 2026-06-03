use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImagePreviewError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error leyendo imagen: {0}")]
    ImageError(#[from] image::error::ImageError),

    #[error("Error leyendo imagen: {0}")]
    SvgImageError(#[from] resvg::usvg::Error),

    #[error("Formato no soportado")]
    UnsuportedFormat,

    #[error("Error obteniendo dimensiones de imagen")]
    DimensionError,
}

pub type ImagePreviewResult<T> = Result<T, ImagePreviewError>;
