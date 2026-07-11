use std::{path::Path, sync::Arc};

use egui::DroppedFile;
use uuid::Uuid;

use crate::core::{
    files::{
        file_extension::FileExtension,
        utils::{write_to_media, write_to_text},
    },
    runtime::{
        bus_structs::{SureTo, UiEvent},
        event_bus::with_event_bus,
    },
    system::clipboard::global_clipboard::TOKIO_RUNTIME,
};

pub fn process_event_x11(cwd: Arc<Path>, active_id: Uuid, dropped_files: &[DroppedFile]) {
    let mut paths = Vec::new();

    for dropped in dropped_files {
        if let Some(path) = &dropped.path {
            paths.push(path.clone().into());
        }
    }

    if !paths.is_empty() {
        let dispatcher = with_event_bus(|e| e.dispatcher(active_id));

        dispatcher
            .send(UiEvent::SureTo(SureTo::SureToMove {
                files: paths,
                dest: cwd.clone(),
            }))
            .ok();
    }

    for dropped in dropped_files {
        let file_ext = FileExtension::from_mime(&dropped.mime);
        let cwd = cwd.clone();

        if file_ext.is_video() || file_ext.is_image() || file_ext.is_audio() {
            let bytes_option = dropped.bytes.clone();

            TOKIO_RUNTIME.spawn_blocking(move || {
                if let Some(bytes) = bytes_option {
                    let bytes: Vec<u8> = bytes.to_vec();
                    write_to_media(file_ext, cwd, bytes);
                }
            });
        } else {
            let text = dropped
                .bytes
                .clone()
                .as_deref()
                .and_then(|b| std::str::from_utf8(b).ok())
                .map(str::to_owned);

            TOKIO_RUNTIME.spawn_blocking(move || {
                if let Some(text) = text {
                    write_to_text(cwd, &text);
                }
            });
        }
    }
}
