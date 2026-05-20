use std::{path::{PathBuf, Path}, sync::Arc};
use tracing::warn;

pub fn parse_initial_path() -> Option<Arc<Path>> {
    let arg = std::env::args().nth(1)?;

    let path = if let Some(stripped) = arg.strip_prefix("file://") {
        PathBuf::from(
            urlencoding::decode(stripped)
                .unwrap_or(std::borrow::Cow::Borrowed(stripped)
            )
            .as_ref()
        )
    } else {
        PathBuf::from(&arg)
    };

    let path_ref = path.as_path();

    if path.is_dir() {
        Some(path_ref.into())
    } else {
        warn!("BlazePilot: '{}' no es un directorio válido", arg);
        None
    }
}