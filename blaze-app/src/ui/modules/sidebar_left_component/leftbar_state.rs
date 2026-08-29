use serde::{Deserialize, Serialize};

pub const MIN_PANEL_WIDTH_TOLERANCE: f32 = 100.0;
pub const PANEL_TAB_WIDTH: f32 = 5.0;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct LeftPanelState {
    pub width: f32,
    pub collapsed: bool,
}

impl Default for LeftPanelState {
    fn default() -> Self {
        Self {
            width: 190.0,
            collapsed: false,
        }
    }
}
