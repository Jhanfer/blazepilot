use parking_lot::Mutex;
use std::sync::{
    LazyLock,
    atomic::{AtomicU64, Ordering},
};

pub static GLOBAL_KEYBOARD_STATE: LazyLock<Mutex<KeyboardState>> =
    LazyLock::new(|| Mutex::new(KeyboardState::new()));

pub fn with_keyboard_state<R>(k: impl FnOnce(&mut KeyboardState) -> R) -> R {
    k(&mut GLOBAL_KEYBOARD_STATE.lock())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardAction {
    Copy,
    Cut,
    Paste(String),
}

pub struct KeyboardState {
    action: Option<KeyboardAction>,
    generation: AtomicU64,
    created_at: AtomicU64,
    last_selection: Option<String>,
}

impl KeyboardState {
    fn new() -> Self {
        Self {
            action: None,
            generation: AtomicU64::new(0),
            created_at: AtomicU64::new(0),
            last_selection: None,
        }
    }

    pub fn update_selection(&mut self, selected: String) {
        self.last_selection = Some(selected);
    }

    pub fn take_selection(&self) -> Option<String> {
        self.last_selection.clone()
    }

    pub fn set_action(&mut self, action: KeyboardAction, current_frame: u64) {
        self.action = Some(action);
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.created_at.store(current_frame, Ordering::SeqCst);
    }

    pub fn get(&self, current_frame: u64) -> Option<KeyboardAction> {
        let created = self.created_at.load(Ordering::SeqCst);
        if created > 0 && current_frame <= created + 1 {
            self.action.clone()
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.action = None;
        self.generation.store(0, Ordering::SeqCst);
        self.created_at.store(0, Ordering::SeqCst);
    }
}
