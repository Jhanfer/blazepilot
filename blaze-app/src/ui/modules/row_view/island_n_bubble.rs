use std::sync::Arc;

use egui::{Align2, Area, Color32, Context, CornerRadius, Frame, Rect, ScrollArea, Sense, pos2, vec2};
use crate::{core::{blaze_state::BlazeCoreState, files::motor::FileEntry}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons, task_manager::task_manager::TaskStatus}, utils::formating::format_size};

pub fn render_island_bubble(state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>, ctx: &Context) {
    let island_rect = Area::new("blaze_island".into())
        .anchor(Align2::CENTER_BOTTOM, [0.0, -30.0])
        .order(egui::Order::Middle)
        .show(ctx, |ui| {

            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(Color32::from_rgb(36, 42, 47))
                .corner_radius(CornerRadius::same(20))
                .show(ui, |ui| {
                    ui.set_width(150.0);
                    ui.set_min_height(20.0);
                    ui.set_max_height(20.0);
                    
                    ui.centered_and_justified(|ui|{
                        ui.horizontal_centered(|ui| {
                            let icon_size = egui::vec2(14.0, 14.0);
                            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
                            let (icon_name, icon_bytes) = ("file", icons::ICON_FILE);
                            let icon = ui_state.icon_cache.get_or_load(ctx, icon_name, icon_bytes, Color32::GRAY);
                            
                            ui.painter().image(
                                icon.id(),
                                icon_rect,
                                Rect::from_min_max(egui::pos2(0.0, 0.0), 
                                pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );

                            ui.add_space(1.0);
                            
                            let total = files.len();
                            ui.label(format!("{}", total));
                            
                            ui.add_space(5.0);
                            

                            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());

                            let (icon_name, icon_bytes) = ("list", icons::ICON_LIST);
                            let icon = ui_state.icon_cache.get_or_load(ctx, icon_name, icon_bytes, Color32::GRAY);

                            ui.painter().image(
                                icon.id(),
                                icon_rect,
                                Rect::from_min_max(egui::pos2(0.0, 0.0), 
                                pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );


                            ui.add_space(1.0);
                            
                            let selected_count = state.selected_count(files.len());

                            let selected_size: u64 = files.iter()
                                .enumerate()
                                .filter(|(i, _)| state.is_selected(*i))
                                .map(|(_, f)| f.size)
                                .sum();

                            ui.label(format!("{}", selected_count));
                            
                            ui.add_space(5.0);
                            
                            let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
                            let (icon_name, icon_bytes) = ("server", icons::ICON_SERVER);
                            let icon = ui_state.icon_cache.get_or_load(ctx, icon_name, icon_bytes, Color32::GRAY);
                            
                            ui.painter().image(
                                icon.id(),
                                icon_rect,
                                Rect::from_min_max(egui::pos2(0.0, 0.0), 
                                pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );

                            ui.add_space(1.0);
                            
                            ui.label(format_size(selected_size));

                        });
                    });
                });
        }).response.rect;






        let tasks = state.task_manager.get_tasks();
        let has_tasks = !tasks.is_empty();

        let anim = ctx.animate_bool_with_time(
            egui::Id::new("processing_bubble"), 
            has_tasks, 
            0.2
        );

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
            .show(ctx, |ui| {

                Frame::NONE
                    .inner_margin(egui::Margin::same(10))
                    .fill(Color32::from_rgb(36, 42, 47))
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

                                ui.vertical(|ui|{
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
                                        let (bar_rect, _) = ui.allocate_exact_size(vec2(bar_width, bar_height), Sense::hover());

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