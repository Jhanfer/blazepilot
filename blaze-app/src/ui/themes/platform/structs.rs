use egui::{Color32, Rgba, ecolor::ParseHexColorError, epaint::Hsva};
use serde::{Deserialize, Serialize};

//Traits
pub trait ThemeDefault: Sized {
    fn default_dark() -> Self;
    fn default_light() -> Self;
    fn default_vscode_dark() -> Self;
    fn default_vscode_light() -> Self;
}

//Nuevo sistemad e temas más especializado
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LegacyTheme {
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
    #[serde(default = "default_text_primary")]
    pub text: String,
    #[serde(default = "default_text_muted")]
    pub placeholder: String,
    pub border_idle: String,
    pub border_focus: String,
}

fn default_text_primary() -> String {
    "#FFFFFF".into()
}

fn default_text_muted() -> String {
    "#858585".into()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ButtonTokens {
    pub bg: String,
    pub bg_hover: String,
    #[serde(default = "default_button_bg_active")]
    pub bg_active: String,
    #[serde(default = "default_text_primary")]
    pub text: String,
    pub label_hover: String,
    pub label_active: String,
    pub label_inactive: String,
    #[serde(default = "default_button_border")]
    pub border: String,
}

fn default_button_bg_active() -> String {
    "#007ACC".into()
}

fn default_button_border() -> String {
    "#00000000".into()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SidebarItemTokens {
    #[serde(default = "default_transparent")]
    pub bg: String,
    pub bg_hover: String,
    #[serde(default = "default_sidebar_bg_selected")]
    pub bg_selected: String,
    #[serde(default = "default_text_primary")]
    pub text: String,
    #[serde(default = "default_text_primary")]
    pub text_selected: String,
    #[serde(default = "default_text_primary")]
    pub icon_idle: String,
    #[serde(default = "default_text_primary")]
    pub icon_hover: String,
    #[serde(default = "default_text_primary")]
    pub icon_selected: String,
}

fn default_transparent() -> String {
    "#00000000".into()
}

fn default_sidebar_bg_selected() -> String {
    "#372942".into()
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

/// Migración de los temas legacy
impl From<LegacyTheme> for BlazeAppTheme {
    fn from(old: LegacyTheme) -> Self {
        BlazeAppTheme {
            meta: ThemeMeta {
                name: old.name,
                autor: old.autor,
                version: old.version,
                luminance: old.luminance,
            },

            global: GlobalTokens {
                panel_stroke_width: 0.8,
                components_stroke_width: 0.8,
                bg_main: old.semantic.bg_main.clone().into(),
                rubberband: old.semantic.rubberband.into(),
                text_primary: old.semantic.text_primary.clone().into(),
                text_secondary: old.semantic.text_secondary.clone().into(),
                error: old.semantic.error.into(),
                warn: old.semantic.warn.into(),
                success: old.semantic.success.into(),
            },

            file_icons: old.file_theme,

            sidebar_left: SectionTheme {
                bg_panel: old.semantic.bg_container.clone().into(),
                bg_elevated: old.components.sidebar_item.bg.into(),
                bg_hover: old.components.sidebar_item.bg_hover.into(),
                bg_active: old.components.sidebar_item.bg_selected.into(),
                icon_primary: old.components.sidebar_item.icon_idle.into(),
                icon_secondary: old.semantic.text_muted.into(),
                accent: old.semantic.accent.clone().into(),
                border: old.components.panel.border.clone().into(),
            },

            top_bar: SectionTheme {
                bg_panel: old.semantic.bg_container.clone().into(),
                bg_elevated: old.components.input.bg.clone().into(),
                bg_hover: old.components.button.bg_hover.clone().into(),
                bg_active: old.components.button.bg_active.clone().into(),
                icon_primary: old.semantic.text_primary.clone().into(),
                icon_secondary: old.components.button.label_inactive.into(),
                accent: old.semantic.accent.clone().into(),
                border: old.components.panel.border.clone().into(),
            },

            main_area: SectionTheme {
                bg_panel: old.semantic.bg_main.clone().into(),
                bg_elevated: old.semantic.bg_container.clone().into(),
                bg_hover: old.components.list_item.bg_hover.clone().into(),
                bg_active: old.components.list_item.bg_selected.clone().into(),
                icon_primary: old.semantic.text_primary.clone().into(),
                icon_secondary: old.semantic.text_secondary.clone().into(),
                accent: old.semantic.accent.clone().into(),
                border: old.semantic.separator.clone().into(),
            },

            sidebar_right: SectionTheme {
                bg_panel: old.components.tools.bg.into(),
                bg_elevated: old.components.input.bg.into(),
                bg_hover: old.components.button.bg_hover.into(),
                bg_active: old.components.button.bg_active.into(),
                icon_primary: old.semantic.text_primary.clone().into(),
                icon_secondary: old.components.tools.label_inactive.into(),
                accent: old.semantic.accent.clone().into(),
                border: old.components.panel.border.into(),
            },

            context_menu: SectionTheme {
                bg_panel: old.semantic.bg_main.into(),
                bg_elevated: old.semantic.bg_container.into(),
                bg_hover: old.components.list_item.bg_hover.into(),
                bg_active: old.components.list_item.bg_selected.into(),
                icon_primary: old.semantic.text_primary.into(),
                icon_secondary: old.semantic.text_secondary.into(),
                accent: old.semantic.accent.into(),
                border: old.semantic.separator.into(),
            },
        }
    }
}

pub type ColorResult<T> = Result<T, ParseHexColorError>;
/// Traits para transformar hex en [`Color32`]
pub trait AsColor {
    fn as_color(&self) -> ColorResult<Color32>;
}

impl AsColor for Color32 {
    fn as_color(&self) -> ColorResult<Color32> {
        Ok(*self)
    }
}

impl AsColor for str {
    fn as_color(&self) -> ColorResult<Color32> {
        Color32::from_hex(self)
    }
}
/// Deref cubre Box<str>, &str, String
impl<T: AsColor + ?Sized> AsColor for &T {
    fn as_color(&self) -> ColorResult<Color32> {
        (**self).as_color()
    }
}

fn as_color_unwrap(c: ColorResult<Color32>) -> Color32 {
    c.unwrap_or_default()
}

/// Estructura base de la metadata de los temas
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeMeta {
    pub name: Box<str>,
    pub autor: Box<str>,
    pub version: Box<str>,
    pub luminance: f32,
}
//Tema de los iconos
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileTheme {
    pub folder_default: Box<str>,
    pub image: Box<str>,
    pub pdf: Box<str>,
    pub document: Box<str>,
    pub video: Box<str>,
    pub audio: Box<str>,
    pub archive: Box<str>,
    pub code: Box<str>,
    pub font: Box<str>,
    pub executable: Box<str>,
    pub fallback: Box<str>,
}

/// Getters de los colores default para los iconos
impl FileTheme {
    pub fn folder_default(&self) -> Color32 {
        as_color_unwrap(self.folder_default.as_color())
    }

    pub fn image(&self) -> Color32 {
        as_color_unwrap(self.image.as_color())
    }

    pub fn pdf(&self) -> Color32 {
        as_color_unwrap(self.pdf.as_color())
    }

    pub fn document(&self) -> Color32 {
        as_color_unwrap(self.document.as_color())
    }

    pub fn video(&self) -> Color32 {
        as_color_unwrap(self.video.as_color())
    }

    pub fn audio(&self) -> Color32 {
        as_color_unwrap(self.audio.as_color())
    }

    pub fn archive(&self) -> Color32 {
        as_color_unwrap(self.archive.as_color())
    }

    pub fn code(&self) -> Color32 {
        as_color_unwrap(self.code.as_color())
    }

    pub fn font(&self) -> Color32 {
        as_color_unwrap(self.font.as_color())
    }

    pub fn executable(&self) -> Color32 {
        as_color_unwrap(self.executable.as_color())
    }

    pub fn fallback(&self) -> Color32 {
        as_color_unwrap(self.fallback.as_color())
    }
}

/// Tokens globales para toda la app
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalTokens {
    pub bg_main: Box<str>,
    pub panel_stroke_width: f32,
    pub components_stroke_width: f32,
    pub rubberband: Box<str>,
    pub text_primary: Box<str>,
    pub text_secondary: Box<str>,
    pub error: Box<str>,
    pub warn: Box<str>,
    pub success: Box<str>,
}

impl GlobalTokens {
    /// Fondo principal de la app
    pub fn bg_main(&self) -> Color32 {
        as_color_unwrap(self.bg_main.as_color())
    }

    /// Color de la rubberband
    pub fn rubberband(&self) -> Color32 {
        as_color_unwrap(self.rubberband.as_color())
    }

    /// Texto primario de la app
    pub fn text_primary(&self) -> Color32 {
        as_color_unwrap(self.text_primary.as_color())
    }

    /// Texto secundario de la app
    pub fn text_secondary(&self) -> Color32 {
        as_color_unwrap(self.text_secondary.as_color())
    }

    /// Color de error global
    pub fn error(&self) -> Color32 {
        as_color_unwrap(self.error.as_color())
    }

    /// Color de advertencia global
    pub fn warn(&self) -> Color32 {
        as_color_unwrap(self.warn.as_color())
    }

    /// Color de éxito global
    pub fn success(&self) -> Color32 {
        as_color_unwrap(self.success.as_color())
    }
}

/// Base para las secciones y componentes de la app
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SectionTheme {
    pub bg_panel: Box<str>,
    pub bg_elevated: Box<str>,
    pub bg_hover: Box<str>,
    pub bg_active: Box<str>,
    pub icon_primary: Box<str>,
    pub icon_secondary: Box<str>,
    pub accent: Box<str>,
    pub border: Box<str>,
}

impl SectionTheme {
    /// Detecta según la luminosidad si el fondo es oscuro
    fn is_dark_bg(&self, bg_color: Color32) -> bool {
        let rgb = Rgba::from(bg_color);
        // fórmula de luminancia relativa básica
        let luminance = rgb.r() * 0.299 + rgb.g() * 0.587 + rgb.b() * 0.114;
        luminance < 0.5
    }

    fn shift_to_hover(&self, target_c: impl AsColor, bg: impl AsColor) -> Color32 {
        let target_c = as_color_unwrap(target_c.as_color());
        let mut hsva = Hsva::from(target_c);

        if self.is_dark_bg(as_color_unwrap(bg.as_color())) {
            hsva.v = (hsva.v + 0.15).min(1.0); // aclara el color
            hsva.s = (hsva.s - 0.1).max(0.0); // reducir la saturación
        } else {
            hsva.v = (hsva.v - 0.15).max(0.0); // oscurece el color
            hsva.s = (hsva.s + 0.1).min(1.0); // aumentar la saturación
        }

        hsva.into()
    }

    pub fn icon_hover_on_color(&self, bg: Color32) -> Color32 {
        self.shift_to_hover(&*self.icon_primary, bg)
    }

    pub fn color_hover_on_color(&self, color: Color32, bg: Color32) -> Color32 {
        self.shift_to_hover(color, bg)
    }

    /// Color primario para el icono
    pub fn icon_primary(&self) -> Color32 {
        as_color_unwrap(self.icon_primary.as_color())
    }

    /// Color secundario para el icono
    pub fn icon_secondary(&self) -> Color32 {
        as_color_unwrap(self.icon_secondary.as_color())
    }

    /// Fondo para los paneles (ej: sidepanels, modals)
    pub fn bg_panel(&self) -> Color32 {
        as_color_unwrap(self.bg_panel.as_color())
    }

    /// Fondo para los componentes encima de panel (ej: botones, combo boxes, inputs)
    pub fn bg_elevated(&self) -> Color32 {
        as_color_unwrap(self.bg_elevated.as_color())
    }

    /// Color de hover para los componentes del panel
    pub fn bg_hover(&self) -> Color32 {
        as_color_unwrap(self.bg_hover.as_color())
    }

    /// Color de los elementos seleccionados o clicados
    pub fn bg_active(&self) -> Color32 {
        as_color_unwrap(self.bg_active.as_color())
    }

    /// Color de acento de la sección
    pub fn accent(&self) -> Color32 {
        as_color_unwrap(self.accent.as_color())
    }

    /// Color de los bordes del panel o separadores
    pub fn border(&self) -> Color32 {
        as_color_unwrap(self.border.as_color())
    }
}

/// Tematización global basada en roles de sección
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlazeAppTheme {
    pub meta: ThemeMeta,
    pub global: GlobalTokens,
    pub file_icons: FileTheme,
    pub sidebar_left: SectionTheme,
    pub sidebar_right: SectionTheme,
    pub main_area: SectionTheme,
    pub context_menu: SectionTheme,
    pub top_bar: SectionTheme,
}
