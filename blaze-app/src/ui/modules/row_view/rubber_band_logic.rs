use std::sync::Arc;
use egui::{Color32, Painter, Rect, Stroke, pos2, vec2};
use crate::core::{blaze_state::BlazeCoreState, files::motor::FileEntry};


pub fn render_rubberband(state: &mut BlazeCoreState, files: &Vec<Arc<FileEntry>>, clipped_painter: &Painter, panel_top: f32, content_rect: Rect, row_height: f32) {
    if let (Some(start), Some(current)) = (
        state.rubber_band.rubber_band_start,
        state.rubber_band.rubber_band_current
    ) {

        let start_screen_y = panel_top + state.rubber_band.rubber_band_start_content_y - state.scroll_offset;
        let start_screen = pos2(start.x, start_screen_y);
        let rect = Rect::from_two_pos(start_screen, current);


        clipped_painter.rect_filled(
            rect, 
            10.0, 
            Color32::from_rgba_unmultiplied(100, 100, 255, 40)
        );

        let mut points = Vec::new();

        let stroke = Stroke::new(3.0, Color32::from_rgba_unmultiplied(150, 150, 255, 200));
        let radius = (rect.width().min(rect.height()) / 2.0).min(10.0);
        let steps = 8;

        let mut add_arc = |center: egui::Pos2, start_angle: f32| {
            for i in 0..=steps {
                let angle = start_angle + (i as f32 / steps as f32) * std::f32::consts::FRAC_PI_2;
                points.push(center + egui::vec2(angle.cos() * radius, angle.sin() * radius));
            }
        };

        add_arc(rect.right_top() + vec2(-radius, radius), -std::f32::consts::FRAC_PI_2);
        add_arc(rect.right_bottom() + vec2(-radius, -radius), 0.0);
        add_arc(rect.left_bottom() + vec2(radius, -radius), std::f32::consts::FRAC_PI_2);
        add_arc(rect.left_top() + vec2(radius, radius), std::f32::consts::PI);


        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            clipped_painter.add(egui::Shape::dashed_line(
                &[p1, p2],
                stroke,
                4.0, // largo del guion
                4.0  // espacio
            ));
        }

        state.deselect_all();
        state.resize_selection(files.len());

        for (i, _) in files.iter().enumerate() {
            let file_y_min = state.row_view.scroll_area_origin_y + i as f32 * row_height - state.scroll_offset;
            let file_y_max = file_y_min + row_height;

            let file_rect = Rect::from_min_max(
                pos2(content_rect.min.x, file_y_min),
                pos2(content_rect.min.x + content_rect.width() * 0.80, file_y_max),
            );

            if rect.intersects(file_rect) {
                state.selection.set(i, true);
            }
        }

        state.last_selected_index = if state.selected_count(files.len()) > 0 {
            Some(files.len().saturating_sub(1))
        } else {
            None
        };
    }
}