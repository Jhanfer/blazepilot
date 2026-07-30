use egui::Color32;
use serde::{Deserialize, Serialize};

#[deprecated(
    since = "0.18.0",
    note = "Esta función será removida en la versión 0.20.0. Usar 'NewTheme' en su lugar."
)]
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

//Traits
pub trait ThemeDefault: Sized {
    fn default_dark() -> Self;
    fn default_light() -> Self;
    fn default_vscode_dark() -> Self;
    fn default_vscode_light() -> Self;
}
pub trait ToColor {
    fn to_color(&self) -> Color32;
}

impl ToColor for String {
    fn to_color(&self) -> Color32 {
        Color32::from_hex(self).unwrap_or(Color32::DEBUG_COLOR)
    }
}

//Tema de los iconos
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

//Nuevo sistemad e temas más especializado
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewTheme {
    pub name: Box<str>,
    pub autor: Box<str>,
    pub version: Box<str>,
    pub luminance: f32,
    pub semantic: SemanticTokens,
    pub components: ComponentTokens,
    pub file_theme: FileTheme,
}

//semánticos para globalizar y distribuir mejor los colores
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SemanticTokens {
    pub bg_main: String,
    pub bg_container: String,
    pub separator: String,
    pub accent: String,
    pub accent_glow: String,
    pub rubberband: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub error: String,
    pub success: String,
    pub warn: String,
}
//distribución de colores por componentes
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComponentTokens {
    pub panel: PanelTokens,
    pub input: InputTokens,
    pub button: ButtonTokens,
    pub sidebar_item: SidebarItemTokens,
    pub list_item: ListItemTokens,
    pub tools: ToolsViewTokens,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PanelTokens {
    pub bg: String,
    pub border: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputTokens {
    pub bg: String,
    pub border_idle: String,
    pub border_focus: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ButtonTokens {
    pub bg: String,
    pub bg_hover: String,
    pub label_hover: String,
    pub label_active: String,
    pub label_inactive: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SidebarItemTokens {
    pub bg_hover: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListItemTokens {
    pub bg_hover: String,
    pub bg_selected: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolsViewTokens {
    pub bg: String,
    pub label_hover: String,
    pub label_active: String,
    pub label_inactive: String,
}
