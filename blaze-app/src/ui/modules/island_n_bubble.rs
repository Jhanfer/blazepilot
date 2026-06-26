use std::sync::Arc;

use crate::{
    core::{
        blaze_state::{BlazeCoreState, TagViewFilter},
        files::blaze_motor::motor_structs::FileEntry,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::icons,
        task_manager::tasks::TaskStatus,
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
    utils::formating::format_size,
};
use egui::{
    pos2,
    text::{LayoutJob, TextWrapping},
    vec2, Align2, Area, Color32, CornerRadius, FontId, Frame, Margin, Rect, RichText, ScrollArea,
    Sense, Stroke, TextFormat, Ui,
};

fn ease_out_bounce(t: f32) -> f32 {
    let n1 = 7.5625_f32;
    let d1 = 2.75_f32;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

pub fn generic_island<F>(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    bottom_padding: i8,
    tabs_height: i8,
    width: f32,
    mut callback: F,
) -> Rect
where
    F: FnMut(&mut Ui, &mut BlazeUiState, &mut BlazeCoreState),
{
    let current_theme = with_theme(|t| t.current());

    const ISLAND_GAP: f32 = 30.0;

    Area::new("blaze_island".into())
        .anchor(
            Align2::CENTER_BOTTOM,
            [
                0.0,
                -(bottom_padding as f32 + tabs_height as f32 + ISLAND_GAP),
            ],
        )
        .order(egui::Order::Middle)
        .show(ui, |ui| {
            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(current_theme.bg_panel.to_color())
                .stroke(Stroke::new(0.8, current_theme.accent_glow.to_color()))
                .corner_radius(CornerRadius::same(20))
                .show(ui, |ui| {
                    ui.set_width(width);
                    ui.set_min_height(20.0);
                    ui.set_max_height(20.0);

                    ui.centered_and_justified(|ui| {
                        ui.horizontal_centered(|ui| {
                            callback(ui, ui_state, state);
                        });
                    });
                });
        })
        .response
        .rect
}

pub fn render_island_bubble(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    files: &[Arc<FileEntry>],
    bottom_padding: i8,
    tabs_height: i8,
) {
    let current_theme = with_theme(|t| t.current());

    let island_rect = generic_island(
        ui,
        state,
        ui_state,
        bottom_padding,
        tabs_height,
        150.0,
        |ui, ui_state, state| {
            let icon_size = egui::vec2(14.0, 14.0);
            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
            let (icon_name, icon_bytes) = ("file", icons::ICON_FILE);
            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );
            let icon = ui_state.icon_cache.get_or_load(
                ui,
                icon_name,
                icon_bytes,
                Color32::GRAY,
                icon_size,
            );

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            ui.add_space(1.0);

            let total = files.len();
            ui.label(format!("{}", total));

            ui.add_space(5.0);

            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );
            let (icon_name, icon_bytes) = ("list", icons::ICON_LIST);
            let icon = ui_state.icon_cache.get_or_load(
                ui,
                icon_name,
                icon_bytes,
                Color32::GRAY,
                icon_size,
            );

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            ui.add_space(1.0);

            let selected_count = state.selected_count(files.len());

            let selected_size: u64 = files
                .iter()
                .enumerate()
                .filter(|(i, _)| state.is_selected(*i))
                .map(|(_, f)| {
                    if f.is_dir() {
                        state
                            .sizer_manager
                            .cache_manager
                            .get_cached_size(&f.full_path)
                            .unwrap_or(0)
                    } else {
                        f.size
                    }
                })
                .sum();

            ui.label(format!("{}", selected_count));

            ui.add_space(5.0);

            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
            let (icon_name, icon_bytes) = ("database", icons::ICON_DATABASE);
            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );
            let icon = ui_state.icon_cache.get_or_load(
                ui,
                icon_name,
                icon_bytes,
                Color32::GRAY,
                icon_size,
            );

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            ui.add_space(1.0);

            ui.label(format_size(selected_size));
        },
    );

    let tasks = state.task_manager.get_tasks();
    let has_tasks = !tasks.is_empty();

    let anim = ui.animate_bool_with_time(egui::Id::new("processing_bubble"), has_tasks, 0.2);

    let eased = ease_in_out_bounce(anim);

    let target_width = 300.0;
    let target_height = 100.0;

    let pivot = island_rect.center_top();

    let current_w = target_width * eased;
    let current_h = target_height * eased;
    let offset_y = -(eased * (target_height + 10.0));

    Area::new("processing_bubble".into())
        .fixed_pos(pivot + vec2(-current_w / 2.0, offset_y))
        .order(egui::Order::Background)
        .show(ui, |ui| {
            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(current_theme.bg_panel.to_color())
                .stroke(Stroke::new(0.8, current_theme.accent_glow.to_color()))
                .corner_radius(CornerRadius::same(20))
                .show(ui, |ui| {
                    let inner_w = (current_w - 20.0).max(0.1);
                    let inner_h = (current_h - 20.0).max(0.1);

                    ui.set_min_size(vec2(inner_w, inner_h));
                    ui.set_max_size(vec2(inner_w, inner_h));

                    ScrollArea::vertical()
                        .id_salt("tasks_scroll")
                        .max_height(inner_h)
                        .show(ui, |ui| {
                            ui.set_min_width(inner_w);

                            ui.vertical(|ui| {
                                for task in &tasks {
                                    ui.horizontal(|ui| {
                                        let icon = match task.status {
                                            TaskStatus::Running => "⏳",
                                            TaskStatus::FinishedSuccess => "✅",
                                            TaskStatus::FinishedError => "❌",
                                        };
                                        ui.label(icon);
                                        ui.label(&task.text);
                                    });

                                    let bar_width = (current_w - 40.0).max(0.0);
                                    let bar_height = 4.0;
                                    let (bar_rect, _) = ui.allocate_exact_size(
                                        vec2(bar_width, bar_height),
                                        Sense::hover(),
                                    );

                                    ui.painter().rect_filled(
                                        bar_rect,
                                        CornerRadius::same(2),
                                        Color32::from_rgba_unmultiplied(255, 255, 255, 30),
                                    );

                                    let filled = Rect::from_min_size(
                                        bar_rect.min,
                                        vec2(bar_rect.width() * task.progress, bar_height),
                                    );

                                    ui.painter().rect_filled(
                                        filled,
                                        CornerRadius::same(2),
                                        Color32::from_rgb(100, 200, 100),
                                    );

                                    ui.add_space(6.0);
                                }
                            });
                        });
                });
        });

    let screen_size = ui.content_rect();
    let dist_from_bottom = screen_size.bottom() - island_rect.bottom();

    let tabs_width = ui.viewport_rect().width() / 3.0;

    let enabled = state.motor.borrow_mut().tabs.len() > 1;

    if enabled {
        let tabs_bar = Area::new("tabs_bar".into())
            .anchor(Align2::CENTER_BOTTOM, [0.0, -(dist_from_bottom - 80.0)])
            .order(egui::Order::Middle)
            .show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(10, 4))
                    .fill(current_theme.bg_panel.to_color())
                    .stroke(Stroke::new(0.8, current_theme.accent_glow.to_color()))
                    .corner_radius(CornerRadius::same(20))
                    .show(ui, |ui| {
                        ui.set_width(tabs_width);
                        ui.set_min_width(tabs_width);
                        ui.set_max_width(tabs_width);

                        ui.set_height(24.0);
                        ui.set_min_height(24.0);
                        ui.set_max_height(24.0);

                        let tab_count = state.motor.borrow_mut().tabs.len();
                        let tab_w = (tabs_width / tab_count as f32).clamp(80.0, 150.0);
                        let tab_h = 30.0;

                        ScrollArea::horizontal()
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                            )
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;

                                    let active_tab_index =
                                        state.motor.borrow_mut().active_tab_index;

                                    for i in 0..tab_count {
                                        let is_active = i == active_tab_index;

                                        let label = state.tab_title(i).clone();

                                        let color: Color32 = if is_active {
                                            Color32::from_rgb(27, 31, 35)
                                        } else {
                                            Color32::from_rgb(56, 61, 68)
                                        };

                                        let (rect, resp) = ui.allocate_exact_size(
                                            vec2(tab_w, tab_h),
                                            Sense::click(),
                                        );

                                        ui.painter().rect_filled(
                                            rect,
                                            CornerRadius::same(20),
                                            color,
                                        );

                                        let icon_size = 14.0;
                                        let icon_padding = 6.0;

                                        let t_icon_rect = Rect::from_min_size(
                                            pos2(
                                                rect.left() + icon_padding,
                                                rect.center().y - icon_size / 2.0,
                                            ),
                                            vec2(icon_size, icon_size),
                                        );
                                        let (icon_n, icon_b) = ("tab-icon", icons::ICON_TAB_ICON);

                                        let icon = ui_state.icon_cache.get_or_load(
                                            ui,
                                            icon_n,
                                            icon_b,
                                            Color32::GRAY,
                                            vec2(icon_size, icon_size),
                                        );

                                        let rounded_rect = Rect::from_min_max(
                                            pos2(
                                                t_icon_rect.min.x.round(),
                                                t_icon_rect.min.y.round(),
                                            ),
                                            pos2(
                                                t_icon_rect.max.x.round(),
                                                t_icon_rect.max.y.round(),
                                            ),
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

                                        let max_text_width =
                                            rect.width() - x_size - x_padding * 2.0 - 8.0;

                                        let mut job = LayoutJob::single_section(
                                            label,
                                            TextFormat {
                                                font_id: FontId::default(),
                                                color: current_theme.text_primary.to_color(),
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

                                        let resp_x = ui.interact(
                                            x_rect,
                                            ui.id().with("tab_x").with(i),
                                            Sense::click(),
                                        );

                                        ui.painter().rect_filled(
                                            x_rect,
                                            CornerRadius::same(20),
                                            Color32::WHITE,
                                        );

                                        let (icon_n, icon_b) = ("x", icons::ICON_X);

                                        let icon = ui_state.icon_cache.get_or_load(
                                            ui,
                                            icon_n,
                                            icon_b,
                                            Color32::GRAY,
                                            vec2(icon_size, icon_size),
                                        );

                                        let rounded_rect = Rect::from_min_max(
                                            pos2(x_rect.min.x.round(), x_rect.min.y.round()),
                                            pos2(x_rect.max.x.round(), x_rect.max.y.round()),
                                        );

                                        ui.painter().image(
                                            icon.id(),
                                            rounded_rect,
                                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                            Color32::WHITE,
                                        );

                                        if resp.hovered() || resp_x.hovered() {
                                            ui.set_cursor_icon(egui::CursorIcon::PointingHand);
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

                                        if resp_x.clicked() {
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
            })
            .response
            .rect;

        Area::new("hints".into())
            .anchor(Align2::CENTER_BOTTOM, [0.0, -(dist_from_bottom - 45.0)])
            .order(egui::Order::Background)
            .show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(10, 4))
                    .fill(current_theme.bg_panel.to_color())
                    .stroke(Stroke::new(0.8, current_theme.accent_glow.to_color()))
                    .corner_radius(CornerRadius {
                        nw: 10,
                        ne: 10,
                        sw: 0,
                        se: 0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                RichText::new("Ctrl + 1-5, Ctrl + <-/->, Tab, Shift + Tab")
                                    .color(current_theme.text_secondary.to_color())
                                    .size(10.0),
                            );
                        });
                    });
            });

        let gap = 8.0;
        Area::new("new_tab_btn".into())
            .current_pos(pos2(tabs_bar.right() + gap, tabs_bar.center().y - 16.0))
            .show(ui, |ui| {
                Frame::new()
                    .corner_radius(CornerRadius::same(20))
                    .fill(Color32::from_rgba_unmultiplied(100, 100, 255, 90))
                    .show(ui, |ui| {
                        ui.set_width(35.0);
                        ui.set_height(35.0);

                        let frame_rect = ui.available_rect_before_wrap();
                        let icon_size = 16.0;

                        let icon_rect =
                            Rect::from_center_size(frame_rect.center(), vec2(icon_size, icon_size));

                        let resp = ui.interact(icon_rect, ui.id().with("plus"), Sense::click());

                        let (icon_n, icon_b) = ("plus", icons::ICON_PLUS);

                        let rounded_rect = Rect::from_min_max(
                            pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                            pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                        );

                        let icon = ui_state.icon_cache.get_or_load(
                            ui,
                            icon_n,
                            icon_b,
                            Color32::GRAY,
                            vec2(icon_size, icon_size),
                        );

                        ui.painter().image(
                            icon.id(),
                            rounded_rect,
                            Rect::from_min_max(egui::pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );

                        if resp.hovered() {
                            ui.set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        if resp.clicked() {
                            state.create_tab();
                        }
                    });
            });
    }
}

pub fn render_tags_island_bubble(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    bottom_padding: i8,
    tabs_height: i8,
    tag_len: usize,
) {
    generic_island(
        ui,
        state,
        ui_state,
        bottom_padding,
        tabs_height,
        90.0,
        |ui, ui_state, state| {
            let (tag_len, items_len) = match state.tag_filter {
                TagViewFilter::All { all_items_len } => (tag_len, all_items_len),
                TagViewFilter::Tag { items_len, .. } => (1, items_len),
            };

            let icon_size = vec2(14.0, 14.0);
            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
            let (icon_name, icon_bytes) = ("tag", icons::ICON_TAG);
            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );
            let icon = ui_state.icon_cache.get_or_load(
                ui,
                icon_name,
                icon_bytes,
                Color32::GRAY,
                icon_size,
            );

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            ui.add_space(1.0);

            ui.label(format!("{}", tag_len));

            ui.add_space(5.0);

            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());

            let (icon_name, icon_bytes) = ("list", icons::ICON_LIST);
            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );
            let icon = ui_state.icon_cache.get_or_load(
                ui,
                icon_name,
                icon_bytes,
                Color32::GRAY,
                icon_size,
            );

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            ui.add_space(1.0);

            ui.label(format!("{}", items_len));
        },
    );
}
