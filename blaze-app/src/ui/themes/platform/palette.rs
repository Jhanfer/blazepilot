//! Paleta derivada de una sección
//!
//! Los temas guardan solo los colores base (`bg_panel`, `bg_hover`, `accent`,
//! `icon_primary`, etc.). Esta paleta expande esos valores a los roles que
//! consumen los componentes, calculando los estados y variantes que faltan
//!
//! Así no hace falta crear un token nuevo cada vez que un componente necesita
//! "otro color": se deriva de los seeds de la sección y del tema global

use egui::Color32;

use super::{
    color_ops,
    structs::{GlobalTokens, SectionTheme},
};

/// Roles de color listos para usar por los componentes de una sección
///
/// No todos los roles se consumen todavía. Se dejan disponibles para que los
/// componentes nuevos no tengan que inventar colores propios
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SectionPalette {
    /// Fondo base del panel
    pub bg_panel: Color32,
    /// Superficie elevada (botones, inputs, tarjetas)
    pub bg_elevated: Color32,
    /// Fondo al pasar el cursor
    pub bg_hover: Color32,
    /// Fondo de un elemento seleccionado
    pub bg_selected: Color32,
    /// Texto principal
    pub fg_primary: Color32,
    /// Texto secundario
    pub fg_secondary: Color32,
    /// Texto silenciado, placeholders
    pub fg_muted: Color32,
    /// Color base de los iconos
    pub icon_primary: Color32,
    /// Color de iconos secundarios
    pub icon_secondary: Color32,
    /// Color de iconos en hover
    pub icon_hover: Color32,
    /// Borde de paneles, marcos y separadores
    pub border_panel: Color32,
    /// Borde idle de controles (botones, inputs, combos)
    pub border_component: Color32,
    /// Borde de foco o énfasis
    pub border_focus: Color32,
    /// Acento para rellenos
    pub accent: Color32,
    /// Acento para hover
    pub accent_hover: Color32,
    /// Acento tenue precompuesto sobre el panel (badges, hints)
    pub accent_soft: Color32,
    /// Acento legible como texto sobre el panel
    pub accent_text: Color32,
    /// Texto legible sobre un relleno de acento
    pub on_accent: Color32,
    /// Si el panel es oscuro
    pub is_dark: bool,
}

impl SectionTheme {
    /// Expande los colores base de la sección a una paleta de roles
    pub fn palette(&self, global: &GlobalTokens) -> SectionPalette {
        let bg_panel = self.bg_panel();
        let is_dark = color_ops::is_dark(bg_panel);

        let fg_primary = global.text_primary();
        let fg_secondary = global.text_secondary();
        let fg_muted =
            color_ops::ensure_contrast(color_ops::mix(fg_secondary, bg_panel, 0.35), bg_panel, 3.0);

        let accent = self.accent();
        let accent_hover = color_ops::shift_lightness(accent, if is_dark { 0.08 } else { -0.08 });
        let accent_soft = color_ops::mix(bg_panel, accent, 0.16);
        let accent_text = color_ops::ensure_contrast(accent, bg_panel, 4.5);
        let on_accent = color_ops::contrast_color(accent);

        // Si el tema no define un color, se deriva
        let bg_elevated = if self.bg_elevated().a() == 0 {
            color_ops::elevate(bg_panel, is_dark)
        } else {
            self.bg_elevated()
        };

        let bg_hover = if self.bg_hover().a() == 0 {
            color_ops::hover_from(bg_panel, fg_primary)
        } else {
            self.bg_hover()
        };

        let bg_selected = if self.bg_active().a() == 0 {
            color_ops::selected_from(bg_panel, accent)
        } else {
            self.bg_active()
        };

        let icon_primary = self.icon_primary();
        let icon_secondary = self.icon_secondary();
        // En hover se usa el acento, garantizando contraste sobre el fondo de hover
        // Así también funciona cuando el icono base ya es blanco (que no se puede aclarar más)
        let icon_hover = color_ops::ensure_contrast(accent, bg_hover, 3.0);

        let border_panel = self.panel_border();
        let border_component = self.component_border();

        let border_focus = accent;

        SectionPalette {
            bg_panel,
            bg_elevated,
            bg_hover,
            bg_selected,
            fg_primary,
            fg_secondary,
            fg_muted,
            icon_primary,
            icon_secondary,
            icon_hover,
            border_panel,
            border_component,
            border_focus,
            accent,
            accent_hover,
            accent_soft,
            accent_text,
            on_accent,
            is_dark,
        }
    }
}
