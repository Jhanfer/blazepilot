use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileTheme {
    pub folder_default: String,
    pub image: String,
    pub pdf: String,
    pub document: String,
    pub video: String,
    pub audio: String,
    pub archive: String,
    pub code: String,
    pub font: String,
    pub executable: String,
    pub fallback: String,
}

impl Default for FileTheme {
    fn default() -> Self {
        Self::default_dark()
    }
}

impl FileTheme {
    pub fn default_dark() -> Self {
        Self {
            folder_default: "#FFFF00FF".into(),
            image: "#64C8FFFF".into(),
            pdf: "#FF5050FF".into(),
            document: "#648CFFFF".into(),
            video: "#C864FFFF".into(),
            audio: "#FFC850FF".into(),
            archive: "#FFA03CFF".into(),
            code: "#64FF96FF".into(),
            font: "#C8C8C8FF".into(),
            executable: "#FF6464FF".into(),
            fallback: "#FFFFFFFF".into(),
        }
    }

    pub fn default_light() -> Self {
        Self {
            folder_default: "#C49A45".into(),
            image: "#0066CC".into(),
            pdf: "#D32F2F".into(),
            document: "#1A56DB".into(),
            video: "#7B1FA2".into(),
            audio: "#E65100".into(),
            archive: "#A0522D".into(),
            code: "#007F3E".into(),
            font: "#4A4A4A".into(),
            executable: "#C62828".into(),
            fallback: "#2D2D2D".into(),
        }
    }

    pub fn blaze_light() -> Self {
        Self {
            folder_default: "#FFA3B6".into(),
            image: "#4EA8DE".into(),
            pdf: "#E63946".into(),
            document: "#48CAE4".into(),
            video: "#B5179E".into(),
            audio: "#FFB703".into(),
            archive: "#FB8500".into(),
            code: "#2A9D8F".into(),
            font: "#6D6875".into(),
            executable: "#E63946".into(),
            fallback: "#4A4E69".into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Theme {
    pub name: Box<str>,
    pub autor: Box<str>,
    pub version: Box<str>,
    pub luminance: f32,
    pub error: String,
    pub success: String,
    pub warn: String,
    pub bg_main: String,
    pub bg_panel: String,
    pub bg_container: String,
    pub border_panel: String,
    pub main_buttons: String,
    pub bg_hover: String,
    pub accent: String,
    pub accent_glow: String,
    pub rubberband: String,
    pub item_selected: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub tools_primary: String,
    pub tools_secondary: String,
    pub tool_btn_active: String,
    pub tool_btn_inactive: String,
    pub tool_btn_hovered: String,
    pub file_theme: FileTheme,
}

impl Default for Theme {
    fn default() -> Self {
        Self::blaze_dark()
    }
}

pub trait ToColor {
    fn to_color(&self) -> Color32;
}

impl ToColor for String {
    fn to_color(&self) -> Color32 {
        Color32::from_hex(self).unwrap_or(Color32::DEBUG_COLOR)
    }
}

impl Theme {
    pub fn blaze_dark() -> Self {
        Self {
            name: "Blaze Dark".into(),
            autor: "Jhanfer".into(),
            version: "1.0.0".into(),
            luminance: 0.7,
            error: "#C44D4D".into(),
            success: "#73C44D".into(),
            warn: "#C4BC4D".into(),
            bg_main: "#0D0614".into(),
            bg_panel: "#1B1124".into(),
            bg_container: "#251731".into(),
            border_panel: "#251731".into(),
            main_buttons: "#281E30".into(),
            bg_hover: "#372942".into(),
            accent: "#8C4BF7".into(),
            accent_glow: "#BA6EFF".into(),
            rubberband: "#8C4BF73E".into(),
            item_selected: "#75479C".into(),
            text_primary: "#FFFFFF".into(),
            text_secondary: "#E2D5ED".into(),
            text_muted: "#7A6A85".into(),
            tools_primary: "#F8F8F8".into(),
            tools_secondary: "#B6B6B6".into(),
            tool_btn_active: "#B78BDD".into(),
            tool_btn_inactive: "#858585".into(),
            tool_btn_hovered: "#d3a1ff".into(),
            file_theme: FileTheme::default(),
        }
    }

    pub fn blaze_light() -> Self {
        Self {
            name: "Blaze Light".into(),
            autor: "Jhanfer".into(),
            version: "1.0.0".into(),
            luminance: 0.30,
            error: "#D32F2F".into(),
            success: "#388E3C".into(),
            warn: "#FBC02D".into(),
            bg_main: "#FFF5F7".into(),
            bg_panel: "#FFFFFF".into(),
            bg_container: "#FFE3E9".into(),
            border_panel: "#FFD1DC".into(),
            main_buttons: "#FFE9EE".into(),
            bg_hover: "#FFC2D1".into(),
            accent: "#FF6584".into(),
            accent_glow: "#FF85A2".into(),
            rubberband: "#FF658426".into(),
            item_selected: "#E4B4BD".into(),
            text_primary: "#3A1A22".into(),
            text_secondary: "#704852".into(),
            text_muted: "#A3858D".into(),
            tools_primary: "#3A1A22".into(),
            tools_secondary: "#704852".into(),
            tool_btn_active: "#FF6584".into(),
            tool_btn_inactive: "#A3858D".into(),
            tool_btn_hovered: "#FF85A2".into(),
            file_theme: FileTheme::blaze_light(),
        }
    }

    pub fn vscode_dark() -> Self {
        Self {
            name: "VS Code Dark".into(),
            autor: "Jhanfer".into(),
            version: "1.0.0".into(),
            luminance: 0.7,
            error: "#C44D4D".into(),
            success: "#73C44D".into(),
            warn: "#C4BC4D".into(),
            bg_main: "#1E1E1E".into(),
            bg_panel: "#252526".into(),
            bg_container: "#3C3C3C".into(),
            border_panel: "#3C3C3C".into(),
            main_buttons: "#2D2D2D".into(),
            bg_hover: "#2A2D2E".into(),
            accent: "#007ACC".into(),
            accent_glow: "#1C97EA".into(),
            rubberband: "#007ACC33".into(),
            item_selected: "#5E5E69".into(),
            text_primary: "#CCCCCC".into(),
            text_secondary: "#858585".into(),
            text_muted: "#6A6A6A".into(),
            tools_primary: "#F5F5F5".into(),
            tools_secondary: "#CFCFCF".into(),
            tool_btn_active: "#E5B567".into(),
            tool_btn_inactive: "#858585".into(),
            tool_btn_hovered: "#F1C987".into(),
            file_theme: FileTheme::default(),
        }
    }

    pub fn vscode_light() -> Self {
        Self {
            name: "VS Code Light".into(),
            autor: "Jhanfer".into(),
            version: "1.0.0".into(),
            luminance: 0.4,
            error: "#C44D4D".into(),
            success: "#73C44D".into(),
            warn: "#C4BC4D".into(),
            bg_main: "#F3F3F3".into(),
            bg_panel: "#FFFFFF".into(),
            bg_container: "#E4E4E4".into(),
            border_panel: "#E4E4E4".into(),
            main_buttons: "#F3F3F3".into(),
            bg_hover: "#E4E6F1".into(),
            accent: "#007ACC".into(),
            accent_glow: "#0062A3".into(),
            rubberband: "#007ACC26".into(),
            item_selected: "#8D8D8D".into(),
            text_primary: "#555555".into(),
            text_secondary: "#636363".into(),
            text_muted: "#969696".into(),
            tools_primary: "#333333".into(),
            tools_secondary: "#717171".into(),
            tool_btn_active: "#007ACC".into(),
            tool_btn_inactive: "#858585".into(),
            tool_btn_hovered: "#E5B567".into(),
            file_theme: FileTheme::default_light(),
        }
    }
}
