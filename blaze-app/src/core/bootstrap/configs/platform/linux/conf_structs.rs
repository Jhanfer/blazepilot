use std::{path::Path, sync::Arc};
use serde::{Serialize, Deserialize};


//--__--__--__--__ Modo de ordenado __--__--__--__--__--__--__
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum OrderingMode {
    #[default]
    Az,
    Za,
    SizeAsc,
    SizeDesc,
    DateAsc,
    DateDesc,
}




//--__--__--__--__ Backends  __--__--__--__--__--__--__
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum DisplayBackend {
    #[default]
    Auto,
    X11,
    Wayland
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


//--__--__--__--__ Enlaces Favs  __--__--__--__--__--__--__
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct FavoriteLinks {
    pub name: String,
    pub path: Arc<Path>,
    pub is_dir: bool,
}
