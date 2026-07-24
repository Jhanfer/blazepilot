use parking_lot::Mutex;
use std::sync::LazyLock;
use tracing::debug;

pub static GLOBAL_TEXT_CLIPBOARD: LazyLock<Mutex<TextClipboard>> =
    LazyLock::new(|| Mutex::new(TextClipboard::new()));

pub fn with_text_clipboard<R>(f: impl FnOnce(&mut TextClipboard) -> R) -> R {
    f(&mut GLOBAL_TEXT_CLIPBOARD.lock())
}

pub trait ClipboardBackend: Send {
    fn set(&mut self, text: String);
    fn get(&self) -> Option<String>;
}

pub struct TextClipboard {
    backend: Option<Box<dyn ClipboardBackend>>,
}

impl TextClipboard {
    pub fn new() -> Self {
        Self { backend: None }
    }

    pub fn init(&mut self, backend: impl ClipboardBackend + 'static) {
        self.backend = Some(Box::new(backend));
    }

    pub fn copy(&mut self, text: String) {
        debug!("El backend está None: {}", self.backend.is_none());
        if let Some(backend) = self.backend.as_mut() {
            backend.set(text);
        }
    }

    pub fn paste(&self) -> Option<String> {
        self.backend.as_ref()?.get()
    }
}
