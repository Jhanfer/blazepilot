use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("FfmpegError: {0:?}")]
    FfmpegError(#[from] ffmpeg_next::util::error::Error),

    #[error("CpalError: {0}")]
    CpalError(#[from] cpal::Error),

    #[error("No se ha encontrado dispositivo de audio.")]
    NoDevice,
}

pub type AudioResult<T> = Result<T, AudioError>;

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("FfmpegError: {0:?}")]
    FfmpegError(#[from] ffmpeg_next::util::error::Error),

    #[error("Error crossbeam: {0}")]
    CrossBeamError(#[from] crossbeam_channel::RecvError),
}

pub type VideoResult<T> = Result<T, VideoError>;
