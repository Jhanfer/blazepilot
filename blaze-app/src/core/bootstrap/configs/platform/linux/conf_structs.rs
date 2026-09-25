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

//--__--__--__--__  Columnas  __--__--__--__--__--__--__
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DetailedViewConfig {
    pub visible_columns: Vec<DetailedColumn>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum DetailedColumn {
    Name,
    Size,
    Extension,
    Modified,
    Created,
    Accessed,
    Owner,
    Group,
    Permissions,
    Dimensions,
    SymlinkTarget,
}

impl Default for DetailedViewConfig {
    fn default() -> Self {
        Self {
            visible_columns: vec![
                DetailedColumn::Name,
                DetailedColumn::Size,
                DetailedColumn::Modified,
                DetailedColumn::Accessed,
                DetailedColumn::Extension,
            ],
        }
    }
}

impl DetailedColumn {
    pub fn default_width(&self) -> f32 {
        match self {
            Self::Name => 200.0,
            Self::Size => 70.0,
            Self::Extension => 110.0,
            Self::Modified => 110.0,
            Self::Created => 110.0,
            Self::Accessed => 70.0,
            Self::Owner => 90.0,
            Self::Group => 90.0,
            Self::Permissions => 90.0,
            Self::Dimensions => 90.0,
            Self::SymlinkTarget => 140.0,
        }
    }

    pub fn min_width(&self) -> f32 {
        match self {
            Self::Name => 80.0,
            _ => 50.0,
        }
    }
}
