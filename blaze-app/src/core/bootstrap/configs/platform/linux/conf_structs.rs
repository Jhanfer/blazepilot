use serde::{Deserialize, Serialize};

use crate::ui::modules::{
    sidebar_left_component::leftbar_state::LeftPanelState,
    sidebar_right_component::rightbar_state::RightPanelState,
};

//--__--__--__--__ Modo de ordenado __--__--__--__--__--__--__

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Copy)]
pub enum OrderingKind {
    #[default]
    Name,
    Size,
    Date,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Copy)]
pub enum OrderingDirection {
    #[default]
    Asc,
    Desc,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Copy)]
pub struct OrderingMode {
    pub kind: OrderingKind,
    pub direction: OrderingDirection,
    pub rightpanel_state: RightPanelState,
    pub leftpanel_state: LeftPanelState,
}

//--__--__--__--__ Backends  __--__--__--__--__--__--__
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum DisplayBackend {
    #[default]
    Auto,
    X11,
    Wayland,
}

impl DisplayBackend {
    pub fn name(&self) -> &'static str {
        match self {
            DisplayBackend::Auto => "Auto",
            DisplayBackend::X11 => "X11",
            DisplayBackend::Wayland => "Wayland",
        }
    }
}
