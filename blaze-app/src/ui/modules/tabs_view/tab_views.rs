use crate::{
    core::blaze_state::BlazeCoreState,
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::icons,
        modules::tabs_view::tabs::{
            FLOAT_GAP, INACTIVE_MIN_VISIBLE_W, INACTIVE_TOP_INSET, MIN_VISIBLE_W, R_INACTIVE,
            R_TAB, TAB_H, TAB_MAX_WIDTH, TAB_MIN_WIDTH, TAB_SPACING, draw_tab_bar, draw_tab_merged,
        },
        themes::theme_manager::with_theme,
    },
};
use egui::{
    Align, Color32, CornerRadius, CursorIcon, FontId, Id, Layout, PointerButton, Rect, Response,
    ScrollArea, Sense, Stroke, StrokeKind, TextFormat, Ui, pos2,
    scroll_area::{ScrollBarVisibility, ScrollSource},
    text::{LayoutJob, TextWrapping},
    vec2,
};

#[allow(clippy::too_many_arguments)]
pub fn render_label(
    ui: &mut Ui,
    ui_state: &mut BlazeUiState,
    state: &mut BlazeCoreState,
    label: String,
    (tab_count, i): (usize, usize),
    rect: Rect,
    resp: &Response,
    visible_w: f32,
) {
    let current_theme = with_theme(|t| t.current());

    let icon_size = 16.0;
    let icon_padding = 13.0;
    let x_size = 14.0;
    let x_padding = 10.0;

    let show_icon = visible_w >= 55.0;
    let show_label = visible_w >= 50.0;
    let show_close = tab_count > 1 && visible_w >= 30.0;

    let content_left = rect.left() + icon_padding + icon_size + 4.0;

    let content_right = if tab_count > 1 {
        rect.right() - x_size - x_padding * 2.0
    } else {
        rect.right() - x_padding
    };

    let max_text_width = (content_right - content_left).max(0.0);

    // Icono de pestaña
    if show_icon {
        let t_icon_rect = Rect::from_min_size(
            pos2(
                rect.left() + icon_padding,
                rect.center().y - icon_size + FLOAT_GAP / 1.5,
            ),
            vec2(icon_size, icon_size),
        );
        let (icon_n, icon_b) = ("folder-open", icons::ICON_FOLDER_OPEN);

        let icon = ui_state.icon_cache.get_or_load(
            ui,
            icon_n,
            icon_b,
            current_theme.top_bar.accent(),
            vec2(icon_size, icon_size),
        );

        let rounded_rect = Rect::from_min_max(
            pos2(t_icon_rect.min.x.round(), t_icon_rect.min.y.round()),
            pos2(t_icon_rect.max.x.round(), t_icon_rect.max.y.round()),
        );

        ui.painter().image(
            icon.id(),
            rounded_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    let x_rect = Rect::from_min_size(
        pos2(
            rect.right() - x_size - x_padding,
            rect.center().y - x_size + FLOAT_GAP / 1.5,
        ),
        vec2(x_size, x_size),
    );

    // Label
    if show_label {
        let mut job = LayoutJob::single_section(
            label,
            TextFormat {
                font_id: FontId::default(),
                color: current_theme.global.text_primary(),
                ..Default::default()
            },
        );
        job.wrap = TextWrapping::truncate_at_width(max_text_width);

        let galley = ui.painter().layout_job(job);
        ui.painter().galley(
            pos2(
                rect.left() + icon_padding + icon_size + 4.0,
                rect.center().y - galley.size().y + FLOAT_GAP / 1.5,
            ),
            galley,
            current_theme.global.text_primary(),
        );
    }

    // icono de X
    if show_close && tab_count > 1 {
        let resp_x = ui.interact(x_rect, ui.id().with("tab_x").with(i), Sense::click());

        let cc = current_theme.top_bar.bg_hover();
        let circ_color = Color32::from_rgba_unmultiplied(cc.r(), cc.g(), cc.b(), 80);

        ui.painter()
            .rect_filled(x_rect, CornerRadius::same(20), circ_color);

        let (icon_n, icon_b) = ("x", icons::ICON_X);

        let icon = ui_state.icon_cache.get_or_load(
            ui,
            icon_n,
            icon_b,
            Color32::WHITE,
            vec2(icon_size, icon_size),
        );

        let rounded_rect = Rect::from_min_max(
            pos2(x_rect.min.x.round(), x_rect.min.y.round()),
            pos2(x_rect.max.x.round(), x_rect.max.y.round()),
        );

        let xc = current_theme.top_bar.accent();
        let x_color = Color32::from_rgba_unmultiplied(xc.r(), xc.g(), xc.b(), 200);

        ui.painter().image(
            icon.id(),
            rounded_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            x_color,
        );

        if resp.hovered() || resp_x.hovered() {
            ui.set_cursor_icon(CursorIcon::PointingHand);
        }

        if resp_x.clicked() {
            state.close_tab(i);
        }
    }
}

pub fn tabs(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    bar_width: f32,
) -> Rect {
    let current_theme = with_theme(|t| t.current());

    let tab_count = state.motor.borrow().tabs.len();
    let tab_h = TAB_H;

    let tab_w = if tab_count == 0 {
        TAB_MAX_WIDTH
    } else {
        let spacing_width = TAB_SPACING * tab_count.saturating_sub(1) as f32;
        let width_per_tab = (bar_width - spacing_width) / tab_count as f32;

        width_per_tab.clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH)
    };

    let active_tab_index = state.motor().active_tab_index;
    let mut active_rect: Option<Rect> = None;
    let mut viewport = Rect::NOTHING;

    ui.allocate_ui(vec2(bar_width, tab_h), |ui| {
        let view = ui.available_rect_before_wrap();

        viewport = ScrollArea::horizontal()
            .scroll_source(ScrollSource::MOUSE_WHEEL)
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .id_salt(Id::new("tabs_scroll"))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_height(tab_h);
                ui.spacing_mut().item_spacing.x = TAB_SPACING;

                ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
                    for i in 0..tab_count {
                        let is_active = i == active_tab_index;

                        let size = vec2(tab_w, tab_h);
                        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

                        if is_active {
                            active_rect = Some(rect);
                        }

                        let visible_w = rect.intersect(view).width().max(0.0);

                        if !is_active && visible_w < INACTIVE_MIN_VISIBLE_W {
                            continue;
                        }

                        if !is_active {
                            let floating = Rect::from_min_max(
                                pos2(rect.min.x, rect.min.y + INACTIVE_TOP_INSET),
                                pos2(rect.max.x, rect.max.y - FLOAT_GAP),
                            );

                            let (bg, stc) = if resp.hovered() {
                                ui.set_cursor_icon(CursorIcon::PointingHand);
                                let b = current_theme.top_bar.bg_hover();
                                (
                                    Color32::from_rgba_unmultiplied(b.r(), b.g(), b.b(), 80),
                                    current_theme.top_bar.border(),
                                )
                            } else {
                                (
                                    current_theme.top_bar.bg_panel(),
                                    current_theme.top_bar.border(),
                                )
                            };

                            ui.painter().rect(
                                floating,
                                R_INACTIVE,
                                bg,
                                Stroke::new(current_theme.global.panel_stroke_width, stc),
                                StrokeKind::Middle,
                            );

                            let label = state.tab_title(i).clone();
                            render_label(
                                ui,
                                ui_state,
                                state,
                                label,
                                (tab_count, i),
                                rect,
                                &resp,
                                visible_w,
                            );

                            let middle_clicked = ui.input(|i| {
                                i.pointer.button_pressed(PointerButton::Middle)
                                    && i.pointer
                                        .interact_pos()
                                        .map(|p| rect.contains(p))
                                        .unwrap_or(false)
                            });

                            if middle_clicked {
                                state.close_tab(i);
                            }

                            if resp.clicked() && !is_active {
                                state.switch_to_tab(i);
                                state.refresh();
                            }
                        }

                        if resp.clicked() && !is_active {
                            state.switch_to_tab(i);
                            state.refresh();
                        }
                    }
                });
            })
            .inner_rect;
    });

    let bar_h = TAB_H + 10.0;

    let (bar_rect, _) = ui.allocate_exact_size(vec2(bar_width, bar_h), Sense::click());

    let visible_tab = active_rect.and_then(|t| {
        let clipped = t.intersect(viewport);
        if clipped.width() >= MIN_VISIBLE_W * R_TAB {
            Some(clipped)
        } else {
            None
        }
    });

    let clip = Rect::from_min_max(
        pos2(bar_rect.left(), bar_rect.top() - TAB_H - 2.0),
        bar_rect.max,
    );

    let painter = ui.painter().with_clip_rect(clip);

    let bg = current_theme.top_bar.bg_panel();
    let stc = current_theme.top_bar.border();

    match visible_tab {
        Some(tab) => {
            draw_tab_merged(
                &painter,
                bar_rect,
                tab,
                bg,
                Stroke::new(current_theme.global.panel_stroke_width, stc),
            );

            let resp = ui.interact(tab, ui.id().with("active_label"), Sense::all());

            let middle_clicked = ui.input(|i| {
                i.pointer.button_pressed(PointerButton::Middle)
                    && i.pointer
                        .interact_pos()
                        .map(|p| tab.contains(p))
                        .unwrap_or(false)
            });

            if middle_clicked && tab_count > 1 {
                state.close_tab(active_tab_index);
            }

            let label = state.tab_title(active_tab_index).clone();
            let visible_w = tab.width();

            render_label(
                ui,
                ui_state,
                state,
                label,
                (tab_count, active_tab_index),
                tab,
                &resp,
                visible_w,
            );
        }
        None => {
            draw_tab_bar(
                &painter,
                bar_rect,
                bg,
                Stroke::new(current_theme.global.panel_stroke_width, stc),
            );
        }
    }

    bar_rect
}
