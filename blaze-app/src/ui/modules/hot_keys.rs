use crate::{
    core::{
        blaze_state::BlazeCoreState,
        blaze_state::state_structs::{LayoutMode, NewItemType, ViewMode},
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::{
            bus_structs::{QuickTagEvent, SureTo, UiEvent},
            event_bus::with_event_bus,
        },
        system::{
            cache::cache_manager::CacheManager,
            clipboard_text::keyboard_state::{KeyboardAction, with_keyboard_state},
            operationstate::operation_manager::with_history,
            trash_manager::manager::get_backend,
        },
    },
    ui::blaze_ui_state::BlazeUiState,
};
use egui::{Color32, Key, PointerButton, Ui};
use std::{path::Path, sync::Arc};
use tracing::warn;

/// Inputs que capturan el teclado. Si alguno tiene foco, las hotkeys
/// se desactivan
const TEXT_EDIT_IDS: &[&str] = &[
    "search_bar",
    "search_ctx_menu",
    "creating_new",
    "rename_space",
    "quick_tag_name",
    "save_theme_placeholder",
    "route_bar",
    "config_search_bar",
];

fn any_focused(ui: &mut Ui) -> bool {
    ui.memory(|m| TEXT_EDIT_IDS.iter().any(|id| m.has_focus((*id).into())))
}

pub fn hot_keys_logic(
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    files: &[Arc<FileEntry>],
    ui: &mut Ui,
) {
    // Unificación de todas las llamadas de input
    let input = ui.input(|i| i.clone());
    let mods_key = input.modifiers;
    let cmd = mods_key.command;

    // Estados de ui
    let editing = state.renaming_file.is_some() || state.creating_new.is_some();
    let text_focused = any_focused(ui);
    let context_menu_open = ui_state.context_menu_state.open;

    let keys_enabled = !editing && !text_focused && !context_menu_open;

    // Mod + K para iniciar la búsqueda
    // Funciona incluso cuando se está escribiendo, salvo nombrando archivos
    if cmd && input.key_pressed(Key::K) && !editing {
        ui.memory_mut(|m| m.request_focus("search_bar".into()))
    }

    // Desactiva todo si hay foco de texto o renombrado
    if !keys_enabled {
        return;
    }

    let has_clipboard = state.clipboard.clipboard_has_files().unwrap_or_else(|e| {
        warn!("Error en el portapapeles: {e}");
        false
    });

    // Navegación con flechas
    if input.key_pressed(Key::ArrowUp) {
        let prev = match state.last_selected_index() {
            Some(0) => 0,
            Some(i) => i - 1,
            None => {
                let last_visible = match state.view_mode {
                    ViewMode::Normal(LayoutMode::Grid) => state
                        .grid_view
                        .base
                        .last_visible
                        .min(files.len().saturating_sub(1)),
                    ViewMode::Tags(LayoutMode::Miller) => state
                        .miller_view
                        .base
                        .last_visible
                        .min(files.len().saturating_sub(1)),
                    ViewMode::Normal(_) | ViewMode::Tags(_) => state
                        .row_view
                        .last_visible
                        .min(files.len().saturating_sub(1)),
                };
                last_visible.min(files.len().saturating_sub(1))
            }
        };

        state.deselect_all();
        state.resize_selection(files.len());

        if files.is_empty() {
            state.set_last_selected_index(None);
            state.set_selection_anchor(None);
        } else {
            let safe = prev.min(files.len() - 1);
            state.set_selection(safe, true);
            state.set_selection_anchor(Some(safe));
            state.set_last_selected_index(Some(safe));
            state.pending_scroll_to = Some(safe);
        }
    }

    if input.key_pressed(Key::ArrowDown) {
        let next = match state.last_selected_index() {
            Some(i) => (i + 1).min(files.len().saturating_sub(1)),
            None => match state.view_mode {
                ViewMode::Normal(LayoutMode::Grid) => state
                    .grid_view
                    .base
                    .last_visible
                    .min(files.len().saturating_sub(1)),
                ViewMode::Tags(LayoutMode::Miller) => state
                    .miller_view
                    .base
                    .last_visible
                    .min(files.len().saturating_sub(1)),
                ViewMode::Normal(_) | ViewMode::Tags(_) => state
                    .row_view
                    .last_visible
                    .min(files.len().saturating_sub(1)),
            },
        };

        state.deselect_all();
        state.resize_selection(files.len());

        if files.is_empty() {
            state.set_last_selected_index(None);
            state.set_selection_anchor(None);
        } else {
            let safe = next.min(files.len() - 1);
            state.set_selection(safe, true);
            state.set_selection_anchor(Some(safe));
            state.set_last_selected_index(Some(safe));
            state.pending_scroll_to = Some(safe);
        }
    }

    // Tecla Enter
    if input.key_pressed(Key::Enter) {
        if let Some(idx) = state.last_selected_index()
            && let Some(file) = files.get(idx)
        {
            if file.is_dir() {
                if !state.is_miller() {
                    state.navigate_to(file.full_path.to_owned());
                } else {
                    let col_index = state.miller_view.columns.len().saturating_sub(1);
                    state.navigate_to_from_col(file.full_path.to_owned(), col_index);
                    state.miller_view.set_opened(col_index, idx);
                }
            } else {
                state.open_file(file);
            }
        } else {
            state.set_last_selected_index(None);
        }
    }

    // Selección y recarga
    if cmd && input.key_pressed(Key::A) {
        state.toggle_select_all(files.len());
    }

    if input.key_pressed(Key::F5) || (cmd && input.key_pressed(Key::R)) {
        let paths_to_invalidate: Vec<Arc<Path>> = {
            let mut motor = state.motor_mut();
            let tab = motor.active_tab_mut();
            let focused = tab.focused.clone();

            match tab.sources.iter_mut().find(|s| s.cwd == focused) {
                Some(souce) => souce
                    .files
                    .read()
                    .iter()
                    .filter(|f| f.is_dir())
                    .map(|f| f.full_path.to_owned())
                    .collect(),
                None => {
                    warn!("no hay source con cwd: {focused:?} en la tab activa");
                    Vec::new()
                }
            }
        };

        for path in &paths_to_invalidate {
            state.dir_sizes.remove_one(path);
            CacheManager::global().invalidate(path);
        }

        ui.request_repaint();
        state.reload_all();
    }

    // Botones laterales del ratón
    if input.pointer.button_pressed(PointerButton::Extra1) {
        state.deselect_all();
        state.resize_selection(files.len());
        state.back();
    }
    if input.pointer.button_pressed(PointerButton::Extra2) {
        state.deselect_all();
        state.resize_selection(files.len());
        state.forward();
    }

    // Eliminar (papelera o eliminación permanente)
    if input.key_pressed(Key::Delete) {
        let sources = state.get_selected_paths(files);

        if !sources.is_empty() {
            let cwd = state.cwd();
            if get_backend().etched_in_trash_path(&cwd) {
                let tab_id = state.active_id();
                with_event_bus(|e| {
                    e.dispatcher(tab_id)
                        .send(UiEvent::SureTo(SureTo::SureToDelete {
                            files: sources,
                            tab_id,
                        }))
                })
                .ok();
            } else {
                let items = files
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| state.is_selected(*i))
                    .map(|(_, f)| (f.name.clone(), f.full_path.to_owned()))
                    .collect();
                state.move_to_trash(items);
            }
        }
    }

    // Portapapeles
    if mods_key.ctrl && input.key_pressed(Key::B) {
        ui_state.toggle_clipboard();
    }

    let (mut do_copy, mut do_cut, mut do_paste) = (false, false, false);
    with_keyboard_state(|k| match k.get(ui.cumulative_frame_nr()) {
        Some(KeyboardAction::Copy) => do_copy = true,
        Some(KeyboardAction::Cut) => do_cut = true,
        Some(KeyboardAction::Paste(_)) => do_paste = true,
        _ => {}
    });

    if do_copy {
        tracing::info!("Se manda comando copia");
        state.copy(files);
    }
    if do_cut {
        tracing::info!("Se manda comando corta");
        state.cut(files);
    }
    if do_paste && has_clipboard {
        tracing::info!("Se manda comando pega");
        let cwd = state.cwd();
        state.paste(cwd);
    }

    // Undo
    if cmd && input.key_pressed(Key::Z) {
        let tab_id = state.active_id();
        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
        with_history(|h| h.undo_last(&dispatcher));
    }

    // Creación de items
    if cmd && mods_key.shift && input.key_pressed(Key::N) {
        state.creating_new = Some(NewItemType::Folder);
        state.new_item_buffer = "nueva carpeta".to_string();
    }
    if cmd && mods_key.shift && input.key_pressed(Key::F) {
        state.creating_new = Some(NewItemType::File);
        state.new_item_buffer = "nuevo archivo".to_string();
    }

    // Terminal
    if mods_key.alt && input.key_pressed(Key::T) {
        state.open_terminal_here();
    }

    // Búsqueda recursiva
    if mods_key.alt && input.key_pressed(Key::R) {
        state.set_search("rec:".to_owned());
        ui.memory_mut(|m| m.request_focus("search_bar".into()));
    }

    // Vistas y tags
    if cmd && !mods_key.shift && input.key_pressed(Key::T) {
        state.view_mode = match &state.view_mode {
            ViewMode::Normal(l) => ViewMode::Tags(l.to_owned()),
            ViewMode::Tags(l) => ViewMode::Normal(l.to_owned()),
        };
    }

    if cmd && !mods_key.shift && input.key_pressed(Key::L) {
        state.view_mode = match &state.view_mode {
            ViewMode::Normal(LayoutMode::Row) => {
                state.grid_view.base.icon_size = 56.0;
                with_configs(|c| c.set_grid_icon_size(56.0));
                ViewMode::Normal(LayoutMode::Grid)
            }
            ViewMode::Normal(LayoutMode::Grid) => {
                state.row_view.icon_size = 20.0;
                with_configs(|c| c.set_row_icon_size(20.0));
                ViewMode::Normal(LayoutMode::Row)
            }
            actual => actual.to_owned(),
        };
    }

    if cmd && mods_key.shift && input.key_pressed(Key::T) {
        let tab_id = state.active_id();
        with_event_bus(|e| {
            e.dispatcher(tab_id)
                .send(UiEvent::QuickTagEvent(QuickTagEvent::CreateNewTag {
                    title: String::new(),
                    temp_color: Color32::GRAY,
                }))
        })
        .ok();
    }

    // Pestañas
    if cmd && !mods_key.shift && input.key_pressed(Key::N) {
        state.create_tab();
    }

    if cmd && input.key_pressed(Key::W) {
        let index = state.motor.borrow().active_tab_index;
        state.close_tab(index);
        state.refresh();
    }

    if input.key_pressed(Key::Tab) {
        if mods_key.shift {
            state.prev_tab();
        } else {
            state.next_tab();
        }
        state.refresh();
    }

    if cmd && !mods_key.shift {
        for (key, index) in [
            (Key::Num1, 0),
            (Key::Num2, 1),
            (Key::Num3, 2),
            (Key::Num4, 3),
            (Key::Num5, 4),
        ] {
            if input.key_pressed(key) {
                state.switch_to_tab(index);
                state.refresh();
            }
        }
    }

    if cmd && input.key_pressed(Key::ArrowLeft) {
        state.prev_tab();
    }
    if cmd && input.key_pressed(Key::ArrowRight) {
        state.next_tab();
    }
}
