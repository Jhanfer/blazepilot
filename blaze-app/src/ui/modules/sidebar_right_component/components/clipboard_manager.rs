use crate::core::blaze_state::BlazeCoreState;
use crate::core::system::clipboard::ClipboardMode;
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::custom_components::UiExt;
use crate::ui::icons_cache::icons;
use crate::ui::modules::utilities::ensure_min_lightness;
use crate::ui::themes::platform::structs::ToColor;
use crate::utils::easing::ease_in_out_bounce;
use egui::scroll_area::ScrollSource;
use egui::{
    Align, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Rect, ScrollArea, Stroke,
    StrokeKind, Ui, pos2, vec2,
};
use std::sync::Arc;

pub fn render_clipboard_menu(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    current_theme: Arc<crate::ui::themes::platform::structs::NewTheme>,
) {
    let has_clipboard = state.clipboard.clipboard_has_files().unwrap_or(false);

    let items = state.clipboard.get_items().unwrap_or(vec![]);

    let clipboard_mode = state.clipboard.get_mode();

    let has_items = has_clipboard && !items.is_empty();

    let show = ui_state.show_clipboard_manager;

    let visible = has_items || show;

    let presence =
        ui.animate_bool_with_easing(Id::new("clipboard_presence"), visible, ease_in_out_bounce);

    let expansion =
        ui.animate_bool_with_easing(Id::new("clipboard_expansion"), show, ease_in_out_bounce);

    let target_width = 150.0;
    let expanded_extra_height = 186.0;
    let header_height = 14.0;

    let current_w = target_width * presence;
    let current_h = header_height * presence + expanded_extra_height * expansion;

    let m_h = (10.0 * presence).round() as i8;
    let m_v = (6.0 * presence).round() as i8;
    let radius = (15.0 * presence).round() as u8;

    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        ui.add_space(20.0);

        Frame::NONE
            .fill(current_theme.components.panel.bg.to_color())
            .stroke(Stroke::new(
                0.5 * expansion,
                current_theme.components.panel.border.to_color(),
            ))
            .corner_radius(CornerRadius::same(radius))
            .inner_margin(Margin::symmetric(m_h, m_v))
            .show(ui, |ui| {
                ui.set_width(current_w);
                ui.set_height(current_h);

                let icon_size = 16.0 * expansion;
                let c = current_theme.semantic.text_primary.to_color();
                let eased_color = Color32::from_rgba_unmultiplied_const(
                    c.r(),
                    c.g(),
                    c.b(),
                    (255.0 * expansion) as u8,
                );
                let bg = Color32::TRANSPARENT;
                let hover_bg = current_theme.components.button.bg_hover.to_color();

                if expansion > 0.0 && show {
                    let btn_resp = ui
                        .blaze_button("Limpiar", Some(&mut ui_state.icon_cache))
                        .size((current_w, 14.0))
                        .radius(20)
                        .stroke_color(Color32::TRANSPARENT)
                        .background_color(bg)
                        .hover_background_color(hover_bg)
                        .icon("trash", icons::ICON_TRASH)
                        .icon_size(icon_size)
                        .text_color(eased_color)
                        .show(ui);

                    if btn_resp.clicked() {
                        state.clear_clipboard();
                        ui_state.toggle_clipboard();
                    }

                    ui.separator();

                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ScrollArea::vertical()
                            .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(current_w);
                                ui.add_space(10.0);

                                let (icon_name, icon_bytes) = ("x", icons::ICON_X);

                                for item in items {
                                    let hover_stroke =
                                        Color32::from_rgba_unmultiplied(178, 34, 34, 200);

                                    let btn = ui
                                        .blaze_button(item.clone(), Some(&mut ui_state.icon_cache))
                                        .size((current_w - 5.0, 14.0))
                                        .radius(20)
                                        .stroke_color(Color32::TRANSPARENT)
                                        .background_color(bg)
                                        .hover_background_color(hover_bg)
                                        .hover_stroke_color(hover_stroke)
                                        .icon(icon_name, icon_bytes)
                                        .hover_icon_color(hover_stroke)
                                        .text_color(eased_color)
                                        .show(ui);

                                    if btn.clicked() {
                                        state.delete_one_from_clipboard(item.clone());
                                    }
                                }

                                ui.add_space(10.0);
                            });
                    });
                    ui.separator();
                    ui.add_space(5.0);
                }

                let icon_size_f = 20.0 * presence;
                let c = current_theme.semantic.text_primary.to_color();
                let eased_color = Color32::from_rgba_unmultiplied_const(
                    c.r(),
                    c.g(),
                    c.b(),
                    (255.0 * presence) as u8,
                );

                let (icon_name, icon_bytes) = match clipboard_mode {
                    Ok(ClipboardMode::Copy) => ("copy", icons::ICON_COPY),
                    Ok(ClipboardMode::Cut) => ("scissors", icons::ICON_SCISSORS),
                    _ => ("", icons::ICON_CLIPBOARD),
                };

                let label = match clipboard_mode {
                    Ok(ClipboardMode::Copy | ClipboardMode::Cut) => "Cola",
                    _ if show => "Cola",
                    _ => "",
                };

                if current_w > 0.8 {
                    let btn_resp = ui
                        .blaze_button(label, Some(&mut ui_state.icon_cache))
                        .background_color(bg)
                        .hover_background_color(hover_bg)
                        .size((current_w, 10.0 * presence))
                        .radius(20)
                        .stroke_color(Color32::TRANSPARENT)
                        .icon(icon_name, icon_bytes)
                        .icon_size(icon_size_f)
                        .text_color(eased_color)
                        .show(ui);

                    if btn_resp.clicked() {
                        ui_state.toggle_clipboard();
                    }

                    let hint = "Ctrl + B";
                    let c = current_theme.semantic.accent_glow.to_color();

                    let hint_stroke_color = Color32::from_rgba_unmultiplied(
                        c.r().saturating_add(20),
                        c.g().saturating_add(20),
                        c.b().saturating_add(20),
                        120 * presence as u8,
                    );

                    let hint_color = Color32::from_rgba_unmultiplied(
                        c.r().saturating_add(60),
                        c.g().saturating_add(60),
                        c.b().saturating_add(60),
                        160 * presence as u8,
                    );

                    let hint_galley = ui.painter().layout_no_wrap(
                        hint.to_string(),
                        FontId::proportional(10.0 * presence),
                        ensure_min_lightness(hint_color),
                    );

                    let hint_size = hint_galley.size();
                    let hint_width = hint_galley.size().x;

                    let hint_pos = pos2(
                        btn_resp.rect.right() - 4.0 - hint_width - 10.0,
                        btn_resp.rect.center().y - hint_galley.size().y / 2.0,
                    );

                    let pad = vec2(4.0, 2.0);
                    let hint_rect = Rect::from_min_size(hint_pos, hint_size).expand2(pad);

                    ui.painter().rect(
                        hint_rect,
                        3.0,
                        current_theme.semantic.bg_container.to_color(),
                        Stroke::new(1.0, hint_stroke_color),
                        StrokeKind::Middle,
                    );

                    ui.painter().galley(hint_pos, hint_galley, hint_color);
                }
            });
    });
}
