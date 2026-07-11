use tracing::{debug, warn};
use uuid::Uuid;

use crate::core::files::file_extension::FileExtension;
use crate::core::runtime::bus_structs::{SureTo, UiEvent};
use crate::core::runtime::event_bus::with_event_bus;
use crate::core::system::clipboard::global_clipboard::TOKIO_RUNTIME;
use crate::{
    core::files::{
        file_extension::sniff_magic_bytes,
        utils::{write_to_media, write_to_text},
    },
    platform::wayland::{
        reader::DroppedData,
        wayland_dnd::{DndEvent, WaylandDndReceiver},
    },
};
use std::sync::Arc;
use std::{path::Path, time::Duration};

pub trait EventTrait {
    fn process_event(&self, cwd: Arc<Path>, active_id: Uuid);
}

#[derive(Debug)]
pub enum URLType {
    WebUrl(String),
    DirectImageUrl(String),
}

impl AsRef<str> for URLType {
    fn as_ref(&self) -> &str {
        match self {
            URLType::WebUrl(url) | URLType::DirectImageUrl(url) => url,
        }
    }
}

async fn probe_url(url: &str) -> Result<URLType, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.head(url).send().await.map_err(|e| e.to_string())?;

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if content_type.starts_with("image/") {
        Ok(URLType::DirectImageUrl(content_type.into()))
    } else {
        Ok(URLType::WebUrl(content_type.into()))
    }
}

impl EventTrait for WaylandDndReceiver {
    fn process_event(&self, cwd: Arc<Path>, active_id: Uuid) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                DndEvent::Hovered(paths) => {
                    debug!("Hover sobre: {:?}", paths);
                }
                DndEvent::Leaving => {
                    debug!("Ratón abandonó la ventana");
                }
                DndEvent::Dropped(data) => match data {
                    DroppedData::Files(path_bufs) => {
                        let dispatcher = with_event_bus(|e| e.dispatcher(active_id));
                        let cwd = cwd.clone();

                        dispatcher
                            .send(UiEvent::SureTo(SureTo::SureToMove {
                                files: path_bufs,
                                dest: cwd.clone(),
                            }))
                            .ok();
                    }
                    DroppedData::MediaBytes { mime, bytes } => {
                        let file_ext = sniff_magic_bytes(&bytes)
                            .unwrap_or_else(|| FileExtension::from_mime(&mime));

                        let cwd = cwd.clone();
                        TOKIO_RUNTIME.spawn_blocking(move || {
                            write_to_media(file_ext, cwd, bytes);
                        });
                    }
                    DroppedData::TextSnippet(content) => {
                        let cwd = cwd.clone();
                        TOKIO_RUNTIME.spawn_blocking(move || {
                            write_to_text(cwd, &content);
                        });
                    }
                    DroppedData::Unknown => {
                        warn!("Dropped Data desconocido");
                    }
                    DroppedData::RemoteUrl(url) => {
                        let dispatcher = with_event_bus(|e| e.dispatcher(active_id));
                        debug!("Url detectada: {url}");
                        let cwd = cwd.clone();
                        TOKIO_RUNTIME.spawn(async move {
                            match probe_url(&url).await {
                                Ok(mime) => {
                                    dispatcher
                                        .send(UiEvent::WantToDownload {
                                            mime: mime.as_ref().into(),
                                            url: url.into(),
                                            cwd,
                                        })
                                        .ok();
                                }
                                Err(e) => warn!("Ha ocurrido un error detectando URL: {e}"),
                            }
                        });
                    }
                },
            }
        }
    }
}
