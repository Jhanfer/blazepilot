use std::sync::Arc;
use egui::{Color32, Context, Painter, Rect, vec2};
use crate::core::{blaze_state::BlazeCoreState, files::motor::FileEntry};

pub fn drag_files(state: &mut BlazeCoreState, files: &[Arc<FileEntry>], clipped_painter: &Painter, ctx: &Context, content_rect: Rect, row_height: f32) {
    if let Some(pos) = state.row_view.drag_ghost_pos {
        let count = state.selected_count(files.len());

        let ghost_size = vec2(220.0, 28.0);

        let layers = count.min(3);
        for layer in (0..layers).rev() {
            let offset = layer as f32 * 3.0;

            let layer_rect = Rect::from_min_size(
                pos + vec2(offset, offset),
                ghost_size,
            );

            clipped_painter.rect_filled(
                layer_rect,
                4.0,
                Color32::from_rgba_unmultiplied(80, 80, 200, 122)
            );
        }

        let first_name = files.iter()
            .enumerate()
            .find(|(i, _)| state.is_selected(*i))
            .map(|(_, f)| f.name.to_string())
            .unwrap_or("archivo".to_string());

        clipped_painter.text(
            pos + vec2(10.0, 14.0),
            egui::Align2::LEFT_CENTER,
            if count > 1 { format!("{} y {} más", first_name, count - 1) } else { first_name.to_string() },
            egui::FontId::default(),
            Color32::WHITE,
        );

    }


    if state.row_view.is_dragging_files {
        state.row_view.drop_target = None;
        state.row_view.drop_invalid_target = None;
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let max_x = content_rect.min.x + content_rect.width() * 0.80;

            if pos.x <= max_x {
                let relative_y = pos.y - state.row_view.scroll_area_origin_y + state.scroll_offset;
                let hovered_index = (relative_y / row_height).floor() as usize;
                if hovered_index < files.len() {
                    let file = &files[hovered_index];
                    if file.is_dir && !state.is_selected(hovered_index) {
                        state.row_view.drop_target = Some(file.full_path.clone());
                    } else if !file.is_dir {
                        state.row_view.drop_invalid_target = Some(file.full_path.clone());
                    }
                }
            }
        }
    }
    
}
