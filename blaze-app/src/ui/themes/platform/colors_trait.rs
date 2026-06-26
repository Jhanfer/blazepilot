use std::sync::Arc;

use crate::ui::themes::platform::structs::Theme;
use egui::Color32;

pub trait ColorsTrait {
    fn init() -> Self;
    fn update_theme(&mut self, mutator: fn(&mut Theme, Color32), value: Color32);
    fn set_theme(&mut self, name: &str);
    fn current_theme(&self) -> Arc<Theme>;
    fn available_themes(&self) -> Vec<Box<str>>;
    fn save(&mut self) -> Result<(), String>;
    fn load(&mut self) -> Result<(), String>;
    fn reload(&mut self) -> Result<(), String>;
    fn reset_to_default(&mut self) -> Result<(), String>;
    fn save_as_custom_theme(&mut self, new_name: &str) -> Result<(), String>;
}
