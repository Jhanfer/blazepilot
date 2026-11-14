use crate::ui::themes::platform::structs::BlazeAppTheme;
use std::sync::Arc;

/// Expone las funciones básicas para cada sistema
pub trait ColorsTrait {
    fn init() -> Self;
    fn update_theme<V>(&mut self, mutator: fn(&mut BlazeAppTheme, V), value: V);
    fn set_theme(&mut self, name: &str);
    fn current_theme(&self) -> Arc<BlazeAppTheme>;
    fn available_themes(&self) -> Vec<Box<str>>;
    fn save(&mut self) -> Result<(), String>;
    fn load(&mut self) -> Result<(), String>;
    fn reload(&mut self) -> Result<(), String>;
    fn reset_to_default(&mut self) -> Result<(), String>;
    fn save_as_custom_theme(&mut self, new_name: &str) -> Result<(), String>;
}
