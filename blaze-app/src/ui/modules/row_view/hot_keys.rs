use std::sync::Arc;
use egui::{ Key, PointerButton, Ui};
use tracing::warn;
use crate::{core::{blaze_state::{BlazeCoreState, NewItemType}, files::blaze_motor::motor_structs::FileEntry, runtime::{bus_structs::{SureTo, UiEvent}, event_bus::with_event_bus}, system::{cache::cache_manager::CacheManager, operationstate::operation_manager::with_history, trash_manager::trash_manager::get_backend}}, ui::blaze_ui_state::BlazeUiState};


fn get_focus(ui: &mut Ui, id: &'static str) -> bool {
    ui.memory(|m| m.has_focus(id.into()))
}


pub fn hot_keys_logic(state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>, ui: &mut Ui, _total_rows: usize) {
    let input = ui.input(|i| i.clone());
    let disable_keys = state.renaming_file.is_none() && state.creating_new.is_none();

    let dispatcher = with_event_bus(|e| e.dispatcher(state.active_id));

    let has_clipboard = match state.clipboard.clipboard_has_files() {
        Ok(has_files) => has_files,
        Err(e) => {
            warn!("Error en el clipboard: {}", e);
            false
        }
    };

    //tecla de arriba
    if input.key_pressed(Key::ArrowUp) && disable_keys {
        let prev = if let Some(i) = state.last_selected_index {
            if i == 0 {
                0
            } else {
                i - 1
            }
        } else {
            state.row_view.last_visible.min(files.len().saturating_sub(1))
        };
        
        state.deselect_all();
        state.resize_selection(files.len());

        if files.is_empty() {
            state.last_selected_index = None;
            state.selection_anchor = None;
        } else {
            let safe_prev = prev.min(files.len() - 1);
            state.selection.set(safe_prev, true);
            state.selection_anchor = Some(safe_prev);
            state.last_selected_index = Some(safe_prev);
            state.pending_scroll_to = Some(safe_prev);
        }
    }


    //tecla de abajo
    if input.key_pressed(Key::ArrowDown) && disable_keys {
        let next = if let Some(i) = state.last_selected_index {
            ( i + 1).min(files.len().saturating_sub(1))
        } else {
            state.row_view.first_visible.min(files.len().saturating_sub(1))
        };

        state.deselect_all();
        state.resize_selection(files.len());
        
        if files.is_empty() {
            state.last_selected_index = None;
            state.selection_anchor = None;
        } else {
            let safe_next = next.min(files.len() - 1);
            state.selection.set(safe_next, true);
            state.selection_anchor = Some(safe_next);
            state.last_selected_index = Some(safe_next);
            state.pending_scroll_to = Some(safe_next);
        }
    }



    //tecla enter
    if ui.input(|i| i.key_pressed(Key::Enter)) && disable_keys {
        if let Some(idx) = state.last_selected_index {
            if idx < files.len() {
                let file = &files[idx];
                if file.is_dir() {
                    state.navigate_to(file.full_path.to_owned());
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
        {
            let motor = state.motor.borrow();
            let tab = motor.active_tab();

            let file_guard = tab.files.read().unwrap();

            for file in file_guard.iter().filter(|f| f.is_dir()) {
                state.calculated_dir_sizes.remove(&file.full_path);
                state.calculating_dir_sizes.remove(&file.full_path);
                CacheManager::global().invalidate(&file.full_path);
            }
        }
        ui.ctx().request_repaint();
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
    if input.key_pressed(Key::Delete) && disable_keys {
        let sources = state.get_selected_paths(files);
        let cwd = state.cwd.clone();
        let is_in_trash = get_backend().etched_in_trash_path(&cwd);

        if is_in_trash && !sources.is_empty() {
            let tab_id = state.active_id;
            let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
            let tab_id = state.motor.borrow_mut().active_tab().id;
            dispatcher.send(
                UiEvent::SureTo(
                    SureTo::SureToDelete { 
                        files: sources, 
                        tab_id 
                    }
                )
            ).ok();
            
        } else if !sources.is_empty(){
            let items = files
                .iter()
                .enumerate()
                .filter(|(index, _)| state.is_selected(*index))
                .map(|(_, f)| (Arc::from(f.name.to_owned()), f.full_path.to_owned()))
                .collect();
            state.move_to_trash(items);
        }
    }


    let (mut do_copy, mut do_cut, mut do_paste) = (false, false, false);
    for event in &input.events {
        match event {
            egui::Event::Copy => do_copy = true,
            egui::Event::Cut => do_cut = true,
            egui::Event::Paste(_) => do_paste = true,
            _ => {}
        }
    }
    
    //Undo
    if input.modifiers.ctrl && input.key_pressed(egui::Key::Z) {
        with_history(|h| h.undo_last(&dispatcher));
    }

    //copiar
    if do_copy && disable_keys {
        state.copy(files);
    }

    //cortar
    if do_cut && disable_keys {
        state.cut(files);
    }

    //pegar
    if do_paste && disable_keys && has_clipboard {
        let cwd = state.cwd.clone();
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


    if input.modifiers.alt && input.key_pressed(Key::R) {
        if state.search_filter.is_empty() || !ui.memory(|m| m.has_focus("search_bar".into())) {
            state.set_search("rec:".to_owned());
            
            ui.ctx().memory_mut(|mem| {
                mem.request_focus("search_bar".into());
            });
        }
    }


    // ---- Pestañas ----

    // nueva pestaña
    if input.modifiers.command && !input.modifiers.shift && input.key_pressed(Key::N) && disable_keys {
        state.create_tab();
    }

    // cerrar pestaña actual
    if input.modifiers.command && input.key_pressed(Key::W) {
        let index = state.motor.borrow().active_tab_index;
        state.close_tab(index);
        state.refresh();
    }

    // cambiar de pestaña y encender búsqueda

    let text_edit_focused = get_focus(ui, "search_bar") || get_focus(ui, "search_ctx_menu") || get_focus(ui, "creating_new") || get_focus(ui, "rename_space");

    if !text_edit_focused {
        for event in &input.events {
            match event {
                egui::Event::Key { key, pressed, modifiers , ..} => {
                    if !pressed {
                        return;
                    }

                    match key {
                        Key::Tab => {
                            if modifiers.shift {
                                state.prev_tab();
                                state.refresh();
                            } else {
                                state.next_tab();
                                state.refresh();
                            }
                        }
                        Key::Num1 if modifiers.ctrl => {state.switch_to_tab(0); state.refresh();},
                        Key::Num2 if modifiers.ctrl => {state.switch_to_tab(1); state.refresh();},
                        Key::Num3 if modifiers.ctrl => {state.switch_to_tab(2); state.refresh();},
                        Key::Num4 if modifiers.ctrl => {state.switch_to_tab(3); state.refresh();},
                        Key::Num5 if modifiers.ctrl => {state.switch_to_tab(4); state.refresh();},

                        Key::ArrowLeft if modifiers.ctrl => {state.prev_tab();},
                        Key::ArrowRight if modifiers.ctrl => {state.next_tab();},

                        Key::A | Key::B | Key::C | Key::D | Key::E | Key::F | Key::G | Key::H | Key::I |
                        Key::J | Key::K | Key::L | Key::M | Key::N | Key::O | Key::P | Key::Q | Key::R |
                        Key::S | Key::T | Key::U | Key::V | Key::W | Key::X | Key::Y | Key::Z 
                        if !modifiers.ctrl && !modifiers.shift && !modifiers.alt => {

                            let config_search_has_focus = ui.memory(|m| m.has_focus(egui::Id::new("config_search_bar")));

                            let context_menu_open = ui_state.context_menu_state.open;

                            if !config_search_has_focus
                            && !context_menu_open
                            && (state.search_filter.is_empty() || !ui.memory(|m| m.has_focus("search_bar".into())))
                            {
                                state.set_search(key.name().to_lowercase());
                                
                                ui.ctx().memory_mut(|mem| {
                                    mem.request_focus("search_bar".into());
                                });
                            }
                        },
                        _ => {}
                    }
                },
                _ => {},
            }
            
        }
    }
}