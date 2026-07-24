use std::{fs::File, io::Read, os::fd::AsFd, path::Path, sync::Arc, sync::mpsc::Receiver};
use wayland_client::protocol::wl_data_offer::WlDataOffer;

pub fn receive_raw_bytes(offer: &WlDataOffer, mime: &str) -> Result<Receiver<Vec<u8>>, String> {
    let (read_fd, write_fd) =
        rustix::pipe::pipe().map_err(|e| format!("Error creando pipe: {e}"))?;
    offer.receive(mime.to_string(), write_fd.as_fd());
    drop(write_fd); // cerrar el extremo de escritura

    let (tx, rx) = std::sync::mpsc::channel();
    let mut file = File::from(read_fd);
    std::thread::spawn(move || {
        // se lee el pipe y enviamos los datos por el canal
        let mut buf: Vec<u8> = Vec::new();

        match file.read_to_end(&mut buf) {
            Ok(_) => {}
            Err(e) => tracing::warn!("Ha ocurrido un error al leer el pipe: {e}"),
        }
        let _ = tx.send(buf);
    });

    Ok(rx)
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
            let text = decode_text(raw);
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
            let text = decode_text(raw);
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

pub fn decode_text(raw: Vec<u8>) -> String {
    // UTF-8 válido
    if let Ok(s) = String::from_utf8(raw.clone()) {
        return s;
    }

    // quítar BOM UTF-8,
    if raw.starts_with(&[0xEF, 0xBB, 0xBF])
        && let Ok(s) = String::from_utf8(raw[3..].to_vec())
    {
        return s;
    }

    // UTF-16LE con BOM
    if raw.starts_with(&[0xFF, 0xFE]) {
        let u16s: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&u16s);
    }

    // UTF-16BE con BOM
    if raw.starts_with(&[0xFE, 0xFF]) {
        let u16s: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&u16s);
    }

    // UTF-16LE sin BOM
    if raw.len().is_multiple_of(2) {
        let u16s: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let decoded = String::from_utf16_lossy(&u16s);
        // si tiene caracteres nulos intercalados es UTF-16
        if decoded.contains('\0') {
            return decoded.replace('\0', "");
        }
    }

    // fallback latin-1/ISO-8859-1
    raw.into_iter().map(|b| b as char).collect()
}
