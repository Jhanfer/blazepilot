use egui::Color32;
use parking_lot::Mutex;
use std::sync::{Arc, LazyLock};
use tracing::warn;

use crate::ui::themes::platform::{ColorsTrait, PlatformTheme, structs::Theme};

pub static GLOBAL_THEMES: LazyLock<Mutex<ThemeManager>> =
    LazyLock::new(|| Mutex::new(ThemeManager::new()));

pub fn with_theme<R>(t: impl FnOnce(&mut ThemeManager) -> R) -> R {
    t(&mut GLOBAL_THEMES.lock())
}

pub struct ThemeManager {
    platform: PlatformTheme,
}

impl ThemeManager {
    fn new() -> Self {
        let mut manager = Self {
            platform: PlatformTheme::init(),
        };

        if let Err(e) = manager.platform.load() {
            warn!("Ha fallado la carga de los temas. Usando colores por defecto: {e}");
        }

        manager
    }

    //--__--__--__-- Getters __--__--__--__--__

    pub fn current(&self) -> Arc<Theme> {
        self.platform.current_theme()
    }

    pub fn get_available_themes_names(&self) -> Vec<Box<str>> {
        self.platform.available_themes()
    }

    //--__--__--__-- Setters __--__--__--__--__

    pub fn update_theme(&mut self, mutator: fn(&mut Theme, Color32), value: Color32) {
        self.platform.update_theme(mutator, value);
        self.save().ok();
    }

    pub fn set_theme(&mut self, name: &str) {
        self.platform.set_theme(name);
        self.save().ok();
    }

    //--__--__--__-- System __--__--__--__--__

    pub fn save_as_custom_theme(&mut self, new_name: &str) -> Result<(), String> {
        self.platform.save_as_custom_theme(new_name)
    }

    pub fn reset_to_default(&mut self) -> Result<(), String> {
        self.platform.reset_to_default()
    }

    pub fn save(&mut self) -> Result<(), String> {
        self.platform.save()
    }

    pub fn reload(&mut self) -> Result<(), String> {
        self.platform.reload()
    }
}
