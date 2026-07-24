use crate::core::system::clipboard_text::text_clipboard::ClipboardBackend;
use crossbeam_channel::Sender;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct WaylandClipboard {
    pub copy_tx: Option<Sender<String>>,
    pub clipboard_text: Arc<Mutex<Option<String>>>,
}

impl ClipboardBackend for WaylandClipboard {
    fn set(&mut self, text: String) {
        if let Some(copy_tx) = self.copy_tx.as_ref() {
            copy_tx.send(text).ok();
        }
    }

    fn get(&self) -> Option<String> {
        self.clipboard_text.lock().clone()
    }
}

impl WaylandClipboard {
    pub fn new() -> Self {
        Self {
            copy_tx: None,
            clipboard_text: Arc::new(Mutex::new(None)),
        }
    }
}
