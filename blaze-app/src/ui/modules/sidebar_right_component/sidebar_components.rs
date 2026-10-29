use crate::core::bootstrap::configs::platform::linux::conf_structs::{
    OrderingDirection, OrderingKind, OrderingMode,
};
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::themes::theme_manager::with_theme;
use egui::{Color32, CursorIcon, Rect, Sense, Ui, pos2, vec2};

pub fn render_ordering_btn<F>(
    ui: &mut Ui,
    ui_state: &mut BlazeUiState,
    icons: (&str, &[u8], &[u8]),
    mode: OrderingMode,
    mut callback: F,
) where
    F: FnMut(),
{
    let current_theme = with_theme(|t| t.current());

    let icon_size = vec2(18.0, 18.0);

    let (icon_rect, resp) = ui.allocate_exact_size(icon_size, Sense::click());

    let (icon_name, icon_bytes) = match mode.kind {
        OrderingKind::Name => match mode.direction {
            OrderingDirection::Asc => (format!("{}-asc", icons.0), icons.1),
            OrderingDirection::Desc => (format!("{}-desc", icons.0), icons.2),
        },
        OrderingKind::Size => match mode.direction {
            OrderingDirection::Asc => (format!("{}-asc", icons.0), icons.1),
            OrderingDirection::Desc => (format!("{}-desc", icons.0), icons.2),
        },
        OrderingKind::Date => match mode.direction {
            OrderingDirection::Asc => (format!("{}-asc", icons.0), icons.1),
            OrderingDirection::Desc => (format!("{}-desc", icons.0), icons.2),
        },
    };

    if resp.clicked() {
        callback();
    }

    let mut color = current_theme.sidebar_right.accent();

    if resp.hovered() {
        ui.set_cursor_icon(CursorIcon::PointingHand);
        color = current_theme.sidebar_right.bg_hover();
    }

    let rounded_rect = Rect::from_min_max(
        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
    );
    let icon = ui_state
        .icon_cache
        .get_or_load(ui, &icon_name, icon_bytes, color, icon_size);

    ui.painter().image(
        icon.id(),
        rounded_rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}
