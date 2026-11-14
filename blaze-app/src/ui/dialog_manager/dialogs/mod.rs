pub mod config_dialog;
pub mod error_dialog;
pub mod folder_color_selector_dialog;
pub mod image_preview_dialog;
pub mod media_player_dialog;
pub mod quick_dialogs;
pub mod selector_dialog;
pub mod show_generic_message;
pub mod sure_to_delete;
pub mod sure_to_move_to;
pub mod update_dialog;
pub mod want_donwload;
pub mod want_to_install;

use crate::ui::{
    custom_components::{
        UiExt,
        button::{BlazeButton, ContentAlign},
    },
    themes::{
        platform::{color_ops, palette::SectionPalette},
        theme_manager::with_theme,
    },
};
use egui::{Color32, CornerRadius, Frame, Margin, Stroke, Ui, WidgetText};

/// Estilo visual de un botón de diálogo
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DialogButtonKind {
    Primary,
    Secondary,
    Danger,
}

/// Marco común de los diálogos
pub(crate) fn dialog_frame(palette: &SectionPalette, stroke_width: f32) -> Frame {
    Frame::NONE
        .fill(palette.bg_panel)
        .stroke(Stroke::new(stroke_width, palette.border_panel))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::same(10))
}

/// Crea un botón de diálogo con el estilo y tamaño habituales
pub(crate) fn dialog_button<'a, T: Into<WidgetText>>(
    ui: &'a mut Ui,
    text: T,
    palette: &SectionPalette,
    kind: DialogButtonKind,
) -> BlazeButton<'a> {
    dialog_button_sized(ui, text, palette, kind, (80.0, 30.0))
}

/// Crea un botón de diálogo permitiendo un tamaño personalizado
pub(crate) fn dialog_button_sized<'a, T: Into<WidgetText>>(
    ui: &'a mut Ui,
    text: T,
    palette: &SectionPalette,
    kind: DialogButtonKind,
    size: (f32, f32),
) -> BlazeButton<'a> {
    let (text_color, background_color, hover_background_color) = match kind {
        DialogButtonKind::Primary => (palette.fg_primary, palette.bg_elevated, palette.bg_hover),
        DialogButtonKind::Secondary => {
            (palette.fg_secondary, palette.bg_elevated, palette.bg_hover)
        }
        DialogButtonKind::Danger => {
            let error = with_theme(|t| t.current().global.error());
            let hover =
                color_ops::shift_lightness(error, if palette.is_dark { 0.08 } else { -0.08 });
            (palette.fg_primary, error, hover)
        }
    };

    ui.blaze_button(text)
        .text_color(text_color)
        .size(size)
        .background_color(background_color)
        .hover_background_color(hover_background_color)
        .stroke_color(Color32::TRANSPARENT)
        .radius(20)
        .content_align(ContentAlign::Center)
}

/// Espacio a ambos lados de una fila de botones centrada
pub(crate) fn dialog_button_spacing(ui: &Ui, count: usize) -> f32 {
    let total = 80.0 * count as f32;
    ((ui.available_width() - total) / (count as f32 + 1.0)).max(0.0)
}
