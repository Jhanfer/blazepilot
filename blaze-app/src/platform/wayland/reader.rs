use std::fs::File;
use std::io::Read;
use std::os::fd::AsFd;
use std::path::Path;
use std::sync::Arc;
use wayland_client::protocol::wl_data_offer::WlDataOffer;

pub fn receive_raw_bytes(offer: &WlDataOffer, mime: &str) -> std::sync::mpsc::Receiver<Vec<u8>> {
    let (read_fd, write_fd) = rustix::pipe::pipe().unwrap();
    offer.receive(mime.to_string(), write_fd.as_fd());
    drop(write_fd); // cerrar el extremo de escritura

    let (tx, rx) = std::sync::mpsc::channel();
    let mut file = File::from(read_fd);

    //let mut buf = [0u8; 16];

    std::thread::spawn(move || {
        // se lee el pipe y enviamos los datos por el canal
        let mut buf: Vec<u8> = Vec::new();

        match file.read_to_end(&mut buf) {
            Ok(_) => {}
            Err(e) => tracing::warn!("Ha ocurrido un error al leer el pipe: {e}"),
        }
        let _ = tx.send(buf);
    });

    rx
}

#[derive(Debug)]
pub enum DroppedData {
    Files(Vec<Arc<Path>>),
    MediaBytes { mime: String, bytes: Vec<u8> },
    TextSnippet(String),
    RemoteUrl(String),
    Unknown,
}

pub fn parse_payload(mime: &str, raw: Vec<u8>) -> DroppedData {
    match mime {
        "text/uri-list" => {
            let text = String::from_utf8_lossy(&raw);
            let paths: Vec<Arc<Path>> = text
                .lines()
                .filter(|l| l.starts_with("file://"))
                .filter_map(|l| {
                    let path = url::Url::parse(l).ok()?.to_file_path().ok()?;
                    Some(Arc::from(path))
                })
                .collect();
            DroppedData::Files(paths)
        }

        "text/plain" | "text/plain;charset=utf-8" => {
            let text = String::from_utf8_lossy(&raw);
            let text = text.trim();
            match url::Url::parse(text) {
                Ok(url) => match url.scheme() {
                    "file" => {
                        if let Ok(path) = url.to_file_path() {
                            DroppedData::Files(vec![path.into()])
                        } else {
                            DroppedData::TextSnippet(text.into())
                        }
                    }

                    "http" | "https" => DroppedData::RemoteUrl(url.to_string()),

                    _ => DroppedData::TextSnippet(text.into()),
                },
                Err(_) => DroppedData::TextSnippet(text.into()),
            }
        }

        m if m.starts_with("image/") || m.starts_with("video/") || m.starts_with("audio/") => {
            DroppedData::MediaBytes {
                mime: mime.into(),
                bytes: raw,
            }
        }

        _ => DroppedData::Unknown,
    }
}
