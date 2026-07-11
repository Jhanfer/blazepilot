use std::{path::Path, sync::Arc};

use crate::core::files::file_extension::{FileExtension, StrExtension};

fn make_file_name<T: StrExtension>(ty: &T) -> String {
    format!(
        "dropped_{}.{}",
        chrono::Local::now().format("%Y%m%d_%H%M%S"),
        ty.extension()
    )
}

pub fn write_to_media(file_ext: FileExtension, cwd: Arc<Path>, bytes: Vec<u8>) {
    match file_ext {
        FileExtension::Image(image_type) => {
            let dest = cwd.join(make_file_name(&image_type));

            match std::fs::write(&dest, bytes) {
                Ok(_) => {}
                Err(e) => tracing::warn!("ha ocurrido un error: {e}"),
            }
        }
        FileExtension::Video(video_type) => {
            let dest = cwd.join(make_file_name(&video_type));

            match std::fs::write(&dest, bytes) {
                Ok(_) => {}
                Err(e) => tracing::warn!("ha ocurrido un error: {e}"),
            }
        }
        FileExtension::Audio(audio_type) => {
            let dest = cwd.join(make_file_name(&audio_type));

            match std::fs::write(&dest, bytes) {
                Ok(_) => {}
                Err(e) => tracing::warn!("ha ocurrido un error: {e}"),
            }
        }
        FileExtension::Unknown => {
            tracing::warn!("FileExtension Desconocido");
        }
        _ => {}
    }
}

pub fn write_to_text(cwd: Arc<Path>, text: &str) {
    let file_name = format!(
        "dropped_{}.txt",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );

    let dest = cwd.join(file_name);

    match std::fs::write(&dest, text) {
        Ok(_) => {}
        Err(e) => tracing::warn!("Ha ocurrido un error escribiendo: {e}"),
    }
}
