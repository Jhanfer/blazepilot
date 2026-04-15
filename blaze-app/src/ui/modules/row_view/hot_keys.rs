use std::sync::Arc;
use egui::{ Key, Ui};
use crate::core::{blaze_state::BlazeCoreState, files::motor::FileEntry};

pub fn hot_keys_logic(state: &mut BlazeCoreState, files: &Vec<Arc<FileEntry>>, ui: &mut Ui, total_rows: usize) {
    let input = ui.input(|i| i.clone());
    let disable_keys = state.renaming_file.is_none() && state.creating_new.is_none();

    //tecla de abajo
    if input.key_pressed(Key::ArrowDown) && disable_keys {
        let next = state.last_selected_index
            .map(|i| (i + 1).min(total_rows - 1))
            .unwrap_or(state.row_view.first_visible);

        state.deselect_all();
        state.resize_selection(files.len());
        state.selection.set(next, true);
        state.selection_anchor = Some(next);
        state.last_selected_index = Some(next);
        state.pending_scroll_to = Some(next);
    }

    //tecla de arriba
    if input.key_pressed(Key::ArrowUp) && disable_keys {
        let prev = state.last_selected_index
            .map(|i| i.saturating_sub(1))
            .unwrap_or(state.row_view.last_visible);
        
        state.deselect_all();
        state.resize_selection(files.len());
        state.selection.set(prev, true);
        state.selection_anchor = Some(prev);
        state.last_selected_index = Some(prev);
        state.pending_scroll_to = Some(prev);
    }

    //tecla enter
    if ui.input(|i| i.key_pressed(Key::Enter)) && disable_keys {
        if let Some(idx) = state.last_selected_index {
            if idx < files.len() {
                let file = &files[idx];
                if file.is_dir {
                    state.navigate_to(file.full_path.clone());
                } else {
                    state.open_file(&file);
                }
            } else {
                state.last_selected_index = None;
            }
        }
    }

    //seleccionar todo
    if input.modifiers.command && input.key_pressed(Key::A) {
        state.toggle_select_all(files.len());
    }
}