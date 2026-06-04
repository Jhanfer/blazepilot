use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::warn;

pub fn parse_initial_path() -> Option<Arc<Path>> {
    let arg = std::env::args().nth(1)?;

    let raw_path = if let Some(stripped) = arg.strip_prefix("file://") {
        PathBuf::from(
            urlencoding::decode(stripped)
                .unwrap_or(std::borrow::Cow::Borrowed(stripped))
                .as_ref(),
        )
    } else {
        PathBuf::from(&arg)
    };

    let absolute_path = if raw_path.is_absolute() {
        raw_path
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(raw_path),
            Err(e) => {
                warn!("No se ha podido obtener el directorio actual: {}", e);
                return None;
            }
        }
    };

    let final_path = match absolute_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            warn!(
                "No se ha podido canonicalizar la ruta: '{}': {}",
                absolute_path.display(),
                e
            );
            absolute_path
        }
    };

    if final_path.is_dir() {
        Some(final_path.into())
    } else {
        warn!("BlazePilot: '{}' no es un directorio válido", arg);
        None
    }
}
