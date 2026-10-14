use std::sync::Arc;

use crate::{
    core::{
        blaze_state::{BlazeCoreState, state_structs::TagViewFilter},
        files::blaze_motor::motor_structs::FileEntry,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        custom_components::UiExt,
        icons_cache::icons,
        task_manager::tasks::TaskStatus,
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
    utils::{easing::ease_in_out_bounce, formating::format_size},
};
use egui::{
    Align2, Area, Color32, CornerRadius, Frame, Margin, Order, Rect, ScrollArea, Sense, Stroke, Ui,
    pos2, vec2,
};

pub fn generic_island<F>(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    bottom_padding: i8,
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
            [0.0, -(bottom_padding as f32 + ISLAND_GAP)],
        )
        .order(egui::Order::Middle)
        .show(ui, |ui| {
            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(current_theme.components.panel.bg.to_color())
                .stroke(Stroke::new(
                    0.5,
                    current_theme.components.panel.border.to_color(),
                ))
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
) {
    let current_theme = with_theme(|t| t.current());

    let island_rect = generic_island(
        ui,
        state,
        ui_state,
        bottom_padding,
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
            ui.label_ns(format!("{}", total));

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

            ui.label_ns(format!("{}", selected_count));

            ui.add_space(5.0);

            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
            let (icon_name, icon_bytes) = ("database", icons::ICON_DATABASE);
            let _rect = Rect::from_min_max(
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
                _rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            ui.add_space(1.0);

            ui.label_ns(format_size(selected_size));
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
        .order(Order::Background)
        .show(ui, |ui| {
            Frame::NONE
                .inner_margin(Margin::same(10))
                .fill(current_theme.components.panel.bg.to_color())
                .stroke(Stroke::new(
                    0.5,
                    current_theme.components.panel.border.to_color(),
                ))
                .corner_radius(CornerRadius::same(20))
                .show(ui, |ui| {
                    let inner_w = (current_w - 20.0).max(0.05);
                    let inner_h = (current_h - 20.0).max(0.05);

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
                                        ui.label_ns(icon);
                                        ui.label_ns(&task.text);
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
}

pub fn render_tags_island_bubble(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    bottom_padding: i8,
    tag_len: usize,
) {
    generic_island(
        ui,
        state,
        ui_state,
        bottom_padding,
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

            ui.label_ns(format!("{}", tag_len));

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

            ui.label_ns(format!("{}", items_len));
        },
    );
}
