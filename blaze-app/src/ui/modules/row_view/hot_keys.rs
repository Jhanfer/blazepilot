use std::{path::PathBuf, sync::Arc};
use egui::{ Key, Modifiers, PointerButton, Ui};
use tracing::info;
use crate::{core::{blaze_state::{BlazeCoreState, NewItemType}, files::motor::FileEntry}, utils::channel_pool::{FileOperation, SureTo, UiEvent}};

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


    //Recargar
    if (input.key_pressed(Key::F5) || 
    (input.modifiers.command && input.key_pressed(Key::R))) && disable_keys {
        state.refresh();
    }


    //Botones del ratón
    if input.pointer.button_pressed(PointerButton::Extra1) && disable_keys {
        state.back();
    }

    if input.pointer.button_pressed(PointerButton::Extra2) && disable_keys {
        state.forward();
    }


    //eliminar 
    if (input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace)) && disable_keys {
        let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
        let trash = state.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();

        if trash == cwd {
            let Some(sender) = state.sender().cloned() else {return;};
            let tab_id = state.motor.borrow_mut().active_tab().id;
            let sources = state.get_selected_paths(files);
            sender.send_ui_event(
                UiEvent::SureTo(
                    SureTo::SureToDelete { 
                        files: sources, 
                        tab_id 
                    }
                )
            ).ok();
            
        } else {
            state.move_to_trash(files);
        }
    }


    let (mut do_copy, mut do_cut, mut do_paste) = (false, false, false);

    ui.ctx().input(|i| {
        for event in &i.events {
            match event {
                egui::Event::Copy => do_copy = true,
                egui::Event::Cut => do_cut = true,
                egui::Event::Paste(_) => do_paste = true,
                _ => {}
            }
        }
    });

    //copiar
    if do_copy && disable_keys { 
        state.copy(files);
    }

    //cortar
    if do_cut && disable_keys { 
        state.cut(files);
    }

    //pegar
    if do_paste && disable_keys { 
        let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
        state.paste(cwd);
    }
    
    //creación de nueva carpeta
    if input.modifiers.command && input.modifiers.shift && input.key_pressed(Key::N) {
        state.creating_new = Some(NewItemType::Folder);
        state.new_item_buffer = "nueva carpeta".to_string(); 
    }

    //creación de nuevo archivo
    if input.modifiers.command && input.modifiers.shift && input.key_pressed(Key::F) {
        state.creating_new = Some(NewItemType::File);
        state.new_item_buffer = "nuevo archivo".to_string();
    }


    if input.modifiers.alt && input.key_pressed(Key::T) {
        state.open_terminal_here();
    }


    // ---- Pestañas ----

    // nueva pestaña
    if input.modifiers.command && input.key_pressed(Key::N) && disable_keys {
        info!("pestaña nueva")
    }

    // cerrar pestaña actual
    if input.modifiers.command && input.key_pressed(Key::W) {
        info!("cerrar pestaña")
    }

    // cambiar de pestaña
    if input.modifiers.command && input.key_pressed(Key::Tab) {
        info!("cambiar pestaña")
    }

}