use std::f32::consts::{FRAC_PI_2, PI};

use crate::{
    core::blaze_state::BlazeCoreState,
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::icons,
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{
    Align, Color32, CornerRadius, FontId, Id, LayerId, Layout, Order, Pos2, Rect, ScrollArea,
    Sense, Shape, Stroke, StrokeKind, TextFormat, Ui, pos2,
    scroll_area::ScrollBarVisibility,
    text::{LayoutJob, TextWrapping},
    vec2,
};

pub fn tabs_bar(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    let current_theme = with_theme(|t| t.current());

    let tab_count = state.motor.borrow().tabs.len();
    let tab_h = 30.0;

    const TAB_MAX_WIDTH: f32 = 150.0;
    const TAB_MIN_WIDTH: f32 = 80.0;
    const TAB_SPACING: f32 = 5.0;

    let cornet_pad = 20.0;
    let bar_width = ui.available_width() - cornet_pad;

    let available_width = bar_width - 20.0;

    let tab_w = if tab_count == 0 {
        TAB_MAX_WIDTH
    } else {
        let spacing_width = TAB_SPACING * tab_count.saturating_sub(1) as f32;
        let width_per_tab = (available_width - spacing_width) / tab_count as f32;

        width_per_tab.clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH)
    };

    ui.allocate_ui(vec2(bar_width, tab_h), |ui| {
        ScrollArea::horizontal()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .id_salt(Id::new("tabs_scroll"))
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_height(tab_h);
                ui.spacing_mut().item_spacing.x = TAB_SPACING;

                ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
                    ui.add_space(1.0);

                    let active_tab_index = state.motor.borrow_mut().active_tab_index;

                    for i in 0..tab_count {
                        let is_active = i == active_tab_index;

                        let corner_radius = if is_active {
                            CornerRadius {
                                nw: 16,
                                ne: 16,
                                sw: 0,
                                se: 0,
                            }
                        } else {
                            CornerRadius::same(12)
                        };

                        let label = state.tab_title(i).clone();

                        let (rect, resp) = ui.allocate_exact_size(vec2(tab_w, tab_h), Sense::all());

                        let (background, stroke_color): (Color32, Color32) = if is_active {
                            (
                                current_theme.components.panel.bg.to_color(),
                                current_theme.components.panel.border.to_color(),
                            )
                        } else if resp.hovered() {
                            let b = current_theme.components.button.bg_hover.to_color();
                            (
                                Color32::from_rgba_unmultiplied(b.r(), b.g(), b.b(), 80),
                                current_theme.components.panel.border.to_color(),
                            )
                        } else {
                            (
                                current_theme.components.panel.bg.to_color(),
                                current_theme.components.panel.border.to_color(),
                            )
                        };

                        ui.painter().rect(
                            rect,
                            corner_radius,
                            background,
                            Stroke::new(0.5, stroke_color),
                            StrokeKind::Outside,
                        );

                        if is_active {
                            let gap = 10.5;
                            let cover = 10.9;
                            let r = 6.0;
                            let stroke = Stroke::new(0.5, stroke_color);
                            let segments = 8;

                            let vis = ui.clip_rect();

                            let clip = Rect::from_min_max(
                                pos2(vis.left(), rect.top() - 5.0),
                                pos2(vis.right(), rect.bottom() + gap + 5.0),
                            );

                            let painter = ui
                                .layer_painter(LayerId::new(
                                    Order::Foreground,
                                    Id::new("tab_bridge").with(i),
                                ))
                                .with_clip_rect(clip);

                            let bridge_rect = Rect::from_min_max(
                                pos2(rect.left() + 0.5, rect.bottom() - 1.0),
                                pos2(rect.right() - 0.5, rect.bottom() + cover),
                            );

                            painter.rect_filled(bridge_rect, 0.0, background);

                            let draw_side =
                                |edge_x: f32, dir: f32, theta_start: f32, theta_end: f32| {
                                    let center = pos2(edge_x + dir * r, rect.bottom() + gap - r);
                                    let corner = pos2(edge_x, rect.bottom() + cover);

                                    let arc: Vec<Pos2> = (0..=segments)
                                        .map(|k| {
                                            let t = k as f32 / segments as f32;
                                            let theta = theta_start + (theta_end - theta_start) * t;
                                            center + vec2(theta.cos(), theta.sin()) * r
                                        })
                                        .collect();

                                    for w in arc.windows(2) {
                                        painter.add(Shape::convex_polygon(
                                            vec![corner, w[0], w[1]],
                                            background,
                                            Stroke::NONE,
                                        ));
                                    }

                                    let mut path = vec![pos2(edge_x, rect.bottom())];
                                    path.extend(arc.iter().copied());
                                    painter.add(Shape::line(path, stroke));
                                };

                            if i != 0 {
                                draw_side(rect.left(), -1.0, 0.0, FRAC_PI_2);
                            } else {
                                let dx = 0.52_f32;
                                painter.line_segment(
                                    [
                                        pos2(rect.left() - dx, rect.bottom()),
                                        pos2(rect.left() - dx, rect.bottom() + gap),
                                    ],
                                    stroke,
                                );
                            }

                            draw_side(rect.right(), 1.0, PI, FRAC_PI_2);
                        }

                        let icon_size = 16.0;
                        let icon_padding = 8.0;

                        let t_icon_rect = Rect::from_min_size(
                            pos2(
                                rect.left() + icon_padding,
                                rect.center().y - icon_size / 2.0,
                            ),
                            vec2(icon_size, icon_size),
                        );
                        let (icon_n, icon_b) = ("folder-open", icons::ICON_FOLDER_OPEN);

                        let icon = ui_state.icon_cache.get_or_load(
                            ui,
                            icon_n,
                            icon_b,
                            Color32::GRAY,
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

                        let x_size = 14.0;
                        let x_padding = 10.0;

                        let x_rect = Rect::from_min_size(
                            pos2(
                                rect.right() - x_size - x_padding,
                                rect.center().y - x_size / 2.0,
                            ),
                            vec2(x_size, x_size),
                        );

                        let max_text_width = rect.width() - x_size - x_padding * 2.0 - 8.0;

                        let mut job = LayoutJob::single_section(
                            label,
                            TextFormat {
                                font_id: FontId::default(),
                                color: current_theme.semantic.text_primary.to_color(),
                                ..Default::default()
                            },
                        );
                        job.wrap = TextWrapping::truncate_at_width(max_text_width);

                        let galley = ui.painter().layout_job(job);
                        ui.painter().galley(
                            pos2(
                                rect.left() + icon_padding + icon_size + 4.0,
                                rect.center().y - galley.size().y / 2.0,
                            ),
                            galley,
                            Color32::WHITE,
                        );

                        if tab_count > 1 {
                            let resp_x =
                                ui.interact(x_rect, ui.id().with("tab_x").with(i), Sense::click());

                            let cc = current_theme.components.button.bg_hover.to_color();
                            let circ_color =
                                Color32::from_rgba_unmultiplied(cc.r(), cc.g(), cc.b(), 80);

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

                            let xc = current_theme.semantic.accent_glow.to_color();
                            let x_color =
                                Color32::from_rgba_unmultiplied(xc.r(), xc.g(), xc.b(), 200);

                            ui.painter().image(
                                icon.id(),
                                rounded_rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                x_color,
                            );

                            if resp.hovered() || resp_x.hovered() {
                                ui.set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            if resp_x.clicked() {
                                state.close_tab(i);
                            }
                        }

                        let middle_clicked = ui.input(|i| {
                            i.pointer.button_pressed(egui::PointerButton::Middle)
                                && i.pointer
                                    .interact_pos()
                                    .map(|p| rect.contains(p))
                                    .unwrap_or(false)
                        });

                        if middle_clicked {
                            state.close_tab(i);
                        }

                        if resp.clicked() {
                            state.switch_to_tab(i);
                            state.refresh();
                        }
                    }
                });
            });
    });
}
