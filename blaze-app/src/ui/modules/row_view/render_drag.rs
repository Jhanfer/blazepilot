use std::sync::Arc;

use egui::{Color32, Painter, Rect, Stroke, StrokeKind, pos2, vec2};

use crate::core::{blaze_state::BlazeCoreState, files::blaze_motor::motor_structs::FileEntry};

pub fn render_drag_files(state: &mut BlazeCoreState, files: &Vec<Arc<FileEntry>>, clipped_painter: &Painter, content_rect: Rect, row_height: f32) {
    //drag de archivos
    if let Some(ref target) = state.row_view.drop_target.clone() {
        if let Some(idx) = files.iter().position(|f| &f.full_path == target) {
            let y = state.row_view.scroll_area_origin_y + idx as f32 * row_height - state.scroll_offset;
            let target_rect = Rect::from_min_size(
                pos2(content_rect.min.x, y),
                vec2(content_rect.width() * 0.80, row_height)
            );

            clipped_painter.rect_stroke(
                target_rect, 4.0,
                Stroke::new(2.0, Color32::from_rgb(150, 150, 255)),
                StrokeKind::Outside
            );
        }
    } else if let Some(ref target_invalid) = state.row_view.drop_invalid_target.clone() {
        if let Some(idx) = files.iter().position(|f| &f.full_path == target_invalid) {
            let y = state.row_view.scroll_area_origin_y + idx as f32 * row_height - state.scroll_offset;
            let target_rect = Rect::from_min_size(
                pos2(content_rect.min.x, y),
                vec2(content_rect.width() * 0.80, row_height)
            );

            clipped_painter.rect_stroke(
                target_rect, 4.0,
                Stroke::new(2.0, Color32::from_rgb(255, 150, 150)),
                StrokeKind::Outside
            );
        }
    }
}