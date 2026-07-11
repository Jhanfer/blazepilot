#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DragContext {
    FileSystem,
    WebBrowser,
    NativeApp,
    Unknown,
}

pub fn determine_context(available: &[String]) -> DragContext {
    let has = |mime: &str| available.iter().any(|m| m == mime || m.starts_with(mime));

    if has("text/html") {
        return DragContext::WebBrowser;
    }

    if has("x-special/gnome-copied-files")
        || has("x-special/nautilus-clipboard")
        || has("application/x-kde-")
        || has("inode/directory")
    {
        return DragContext::FileSystem;
    }

    if has("text/uri-list") {
        return DragContext::FileSystem;
    }

    let media_mime = ["image/", "video/", "audio/"];
    if available
        .iter()
        .any(|m| media_mime.iter().any(|pref| m.starts_with(pref)))
    {
        return DragContext::NativeApp;
    }

    DragContext::Unknown
}

pub fn choose_best_mime(available: &[String]) -> Option<String> {
    let context = determine_context(available);

    // helper para encontrar el formato de media disponible
    let get_best_media = || -> Option<String> {
        const PRIORITY: &[&str] = &[
            "image/png",
            "image/jpeg",
            "image/jpg",
            "image/gif",
            "image/webp",
            "video/mp4",
            "video/webm",
            "audio/mpeg",
            "audio/wav",
        ];
        for preferred in PRIORITY {
            if let Some(found) = available
                .iter()
                .find(|m| m == preferred || m.starts_with(preferred))
            {
                return Some(found.clone());
            }
        }
        None
    };

    match context {
        DragContext::FileSystem => {
            if available.iter().any(|m| m == "text/uri-list") {
                return Some("text/uri-list".to_string());
            }
            if let Some(media) = get_best_media() {
                return Some(media);
            }
        }

        DragContext::WebBrowser => {
            //evitamos ejecutar enlaces
            if let Some(media) = get_best_media() {
                return Some(media);
            }
            if available.iter().any(|m| m == "text/uri-list") {
                return Some("text/uri-list".to_string());
            }
        }

        DragContext::NativeApp => {
            //pasa directamente los bytes de ser una
            if let Some(media) = get_best_media() {
                return Some(media);
            }
        }

        DragContext::Unknown => {
            // preferimos una ruta en caso de no saber que tipo es
            if available.iter().any(|m| m == "text/uri-list") {
                return Some("text/uri-list".to_string());
            }
            if let Some(media) = get_best_media() {
                return Some(media);
            }
        }
    }

    //fallback para texto plano
    if available.iter().any(|m| m.starts_with("text/plain")) {
        return Some("text/plain".to_string());
    }
    None
}
