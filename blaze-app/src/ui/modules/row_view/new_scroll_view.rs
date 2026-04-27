use std::{collections::HashMap, path::PathBuf, sync::Arc};
use egui::{Color32, CursorIcon, FontId, Key, PointerButton, Rect, RichText, ScrollArea, Sense, Ui, pos2, scroll_area::ScrollSource, vec2};
use file_id::FileId;
use tracing::{error, info};
use crate::{core::{blaze_state::BlazeCoreState, configs::config_state::{OrderingMode, with_configs}, files::{file_extension::{DocType, FileExtension}, motor::FileEntry}, system::{extended_info::extended_info_manager::{ExtendedInfo, GitStatus}}}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons, modules::custom_context_menu::context_state::ContextMenuKind}, utils::{channel_pool::{SureTo, UiEvent}, formating::{format_date, format_size}}};



fn handle_row_interactions(ui: &mut Ui, response: &egui::Response, i: usize, file: &Arc<FileEntry>,state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>,content_rect: Rect, rect: Rect) {

    //Habilitar seleccion con rueda de ratón
    let middle_clicked = ui.input(|i| {
        i.pointer.button_pressed(PointerButton::Middle)
        && i.pointer.interact_pos()
            .map(|p| rect.contains(p))
            .unwrap_or(false)
    });

    if middle_clicked {
        state.resize_selection(files.len());
        let currently = state.is_selected(i);
        state.selection.set(i, !currently);
        state.last_selected_index = Some(i);
    }

    if response.drag_started_by(PointerButton::Primary) {
        if !state.is_selected(i) {
            state.deselect_all();
            state.resize_selection(files.len());
            state.selection.set(i, true);
            state.last_selected_index = Some(i);
            state.selection_anchor = Some(i);
        }
        state.row_view.is_dragging_files = true;
    }

    if response.dragged_by(PointerButton::Primary) {
        state.row_view.drag_ghost_pos = ui.input(|i| i.pointer.interact_pos());
    }


    if response.drag_stopped() && state.row_view.is_dragging_files {
        state.row_view.drag_ghost_pos = None;

        let drop_in_file_area = ui.input(|i| i.pointer.interact_pos())
            .map(|p| p.x <= content_rect.min.x + content_rect.width() * 0.80)
            .unwrap_or(false);

        if let Some(invalid_target) = state.row_view.drop_invalid_target.take() {
            info!("No es posible mover a {:?}", invalid_target);  
        }

        if drop_in_file_area {
            let Some(sender) = state.sender().cloned() else {return;};
            let tab_id = state.motor.borrow_mut().active_tab().id;

            if let Some(target) = state.row_view.drop_target.take() {
                let sources = state.get_selected_paths(files);   
                
                sender.send_ui_event(UiEvent::SureTo(
                        SureTo::SureToMove { 
                            files: sources, 
                            dest: target,
                            tab_id,
                        }
                    )).ok();

            } else {
                let cwd = state.cwd.clone();
                let sources = state.get_selected_paths(files);   

                if sources.iter().all(|p| p.parent() == Some(&cwd)) {
                    return;
                }

                sender.send_ui_event(UiEvent::SureTo(SureTo::SureToMove { 
                    files: sources, 
                    dest: cwd,
                    tab_id
                })).ok();
            }
        }

        state.row_view.is_dragging_files = false; 
    }

    if response.secondary_clicked() {
        state.resize_selection(files.len());

        if ui.input(|i| i.modifiers.ctrl) {
            let currently = state.is_selected(i);    
            state.selection.set(i, !currently);
            state.last_selected_index = Some(i);
        } else if !state.is_selected(i) {
            state.deselect_all();
            state.resize_selection(files.len());
            state.selection.set(i, true);
            state.last_selected_index = Some(i);
        }
    }

    let cwd = state.cwd.clone();
    let trash = state.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();

    if trash == cwd {
        let Some(sender) = state.sender().cloned() else {return;};
        if response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(&response);
            ui_state.context_menu_state.target_sender = Some(sender);
            ui_state.context_menu_state.kind = ContextMenuKind::FileTrash;
        }
    } else {
        if response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(&response);
            ui_state.context_menu_state.target_file = Some(file.clone());
            ui_state.context_menu_state.kind = ContextMenuKind::FileNormal;
        }
    }
}

fn render_rename_field(ui: &mut Ui, file: &Arc<FileEntry>, state: &mut BlazeCoreState, rect: Rect, is_renaming: bool) {
    let response = ui.put(rect,
        egui::TextEdit::singleline(&mut state.rename_buffer)
            .margin(vec2(0.0, 5.0))
            .font(egui::FontId::default())
    );

    if is_renaming && !response.has_focus() {
        response.request_focus();
    }

    if !response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
        if let Err(e) = state.clipboard.rename_file(&file.name, &state.rename_buffer) {
            error!("Error renombrando: {}", e);
        }
        state.renaming_file = None;
    }

    let cancel = ui.input(|i| i.key_pressed(Key::Escape)) || ui.input(|i| i.pointer.any_click());


    if response.lost_focus() || cancel {
        state.renaming_file = None;
    }
}

fn resolve_icon(file: &Arc<FileEntry>, color_snapshot: &HashMap<FileId, Color32>) -> (String, &'static [u8], Color32) {
    if file.is_dir {
        let (color, cache_key) = if let Some(file_id) = &file.unique_id {
                let color = color_snapshot.get(file_id).copied().unwrap_or(Color32::YELLOW);
                let cache_key = format!("folder-{:?}", file_id);
                (color, cache_key)
            } else {
                (Color32::YELLOW, "folder-unknown".to_string())
            };
        return (cache_key, icons::ICON_FOLDER, color);
    } else {
        match &file.extension {
            FileExtension::Image(_) => ("image".to_string(), icons::ICON_IMAGE,    Color32::from_rgb(100, 200, 255)),
            FileExtension::Document(DocType::Pdf) => ("pdf".to_string(),      icons::ICON_PDF, Color32::from_rgb(255, 80,  80)),
            FileExtension::Document(_) => ("doc".to_string(), icons::ICON_DOC, Color32::from_rgb(100, 140, 255)),
            FileExtension::Video(_) => ("video".to_string(), icons::ICON_VIDEO,    Color32::from_rgb(200, 100, 255)),
            FileExtension::Audio(_) => ("audio".to_string(), icons::ICON_VIDEO,    Color32::from_rgb(255, 200, 80)),
            FileExtension::Archive(_) => ("archive".to_string(), icons::ICON_ARCHIVE,  Color32::from_rgb(255, 160, 60)),
            FileExtension::Code(_) => ("code".to_string(), icons::ICON_CODE,     Color32::from_rgb(100, 255, 150)),
            FileExtension::Font(_) => ("font".to_string(), icons::ICON_FONT,     Color32::from_rgb(200, 200, 200)),
            FileExtension::Executable(_) => ("exe".to_string(), icons::ICON_EXE,      Color32::from_rgb(255, 100, 100)),
            FileExtension::Unknown => ("file".to_string(), icons::ICON_FILE, Color32::WHITE),
        }
    }
}


fn text_color_for_git(git: Option<&GitStatus>) -> Color32 {
    match git {
        Some(GitStatus::Modified)  => Color32::from_rgb(255, 200, 80),
        Some(GitStatus::Staged)    => Color32::from_rgb(100, 220, 100),
        Some(GitStatus::Untracked) => Color32::from_rgb(160, 160, 160),
        Some(GitStatus::Ignored)   => Color32::from_rgb(100, 100, 100),
        Some(GitStatus::Conflict)  => Color32::from_rgb(255, 80, 80),
        Some(GitStatus::Deleted)   => Color32::from_rgb(255, 60, 60),
        Some(GitStatus::Clean) | None => Color32::from_rgb(189, 189, 189),
    }
}

fn git_dot_color(git: Option<&GitStatus>) -> Option<Color32> {
    match git {
        Some(GitStatus::Modified)  => Some(Color32::from_rgb(255, 200, 80)),
        Some(GitStatus::Staged)    => Some(Color32::from_rgb(100, 220, 100)),
        Some(GitStatus::Untracked) => Some(Color32::from_rgb(160, 160, 160)),
        Some(GitStatus::Ignored)   => Some(Color32::from_rgb(80, 80, 80)),
        Some(GitStatus::Conflict)  => Some(Color32::from_rgb(255, 80, 80)),
        Some(GitStatus::Deleted)   => Some(Color32::from_rgb(255, 60, 60)),
        Some(GitStatus::Clean) | None => None,
    }
}


pub fn new_render_scrollview(ui: &mut Ui, files: &Vec<Arc<FileEntry>>, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, row_height: f32, total_rows: usize, content_rect: Rect) {
    let current_order = with_configs(|c| c.configs.app_ordering_mode.clone());

    // --- Header manual ---
    let id_name_w  = ui.id().with("col_name_w");
    let id_date_w  = ui.id().with("col_date_w");
    let id_size_w  = ui.id().with("col_size_w");

    let available = ui.available_width();
    let name_w: f32 = ui.data(|d| d.get_temp(id_name_w).unwrap_or(available - 180.0)).max(80.0);
    let date_w: f32 = ui.data(|d| d.get_temp(id_date_w).unwrap_or(110.0_f32)).max(60.0);
    let size_w: f32 = ui.data(|d| d.get_temp(id_size_w).unwrap_or(70.0_f32)).max(40.0);

    let name_w = name_w.min(available - date_w - size_w - 20.0).max(80.0);

    let header_height = 24.0;
    let (header_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), header_height),
        Sense::hover(),
    );

    let painter = ui.painter_at(header_rect);
    painter.rect_filled(header_rect, 0.0, Color32::from_rgb(20, 24, 28));

    // Botón Nombre
    let name_btn_rect = Rect::from_min_size(header_rect.min, egui::vec2(name_w, header_height));
    let name_label = match current_order {
        OrderingMode::Az => "Nombre ↑",
        OrderingMode::Za => "Nombre ↓",
        _ => "Nombre",
    };
    if ui.put(name_btn_rect, egui::Button::new(
        RichText::new(name_label).size(11.0).color(Color32::from_rgb(140, 140, 160))
    ).frame(false)).clicked() {
        with_configs(|c| {
            c.set_ordering_mode(match current_order { OrderingMode::Az => OrderingMode::Za, _ => OrderingMode::Az });
        });
        state.refresh();
    }

    // Botón Modificado
    let date_x = header_rect.min.x + name_w;
    let date_btn_rect = Rect::from_min_size(pos2(date_x, header_rect.min.y), egui::vec2(date_w, header_height));
    let date_label = match current_order {
        OrderingMode::DateAsc  => "Modificado ↑",
        OrderingMode::DateDesc => "Modificado ↓",
        _ => "Modificado",
    };
    if ui.put(date_btn_rect, egui::Button::new(
        RichText::new(date_label).size(11.0).color(Color32::from_rgb(140, 140, 160))
    ).frame(false)).clicked() {
        with_configs(|c| {
            c.set_ordering_mode(match current_order { OrderingMode::DateAsc => OrderingMode::DateDesc, _ => OrderingMode::DateAsc });
        });
        state.refresh();
    }

    // Botón Tamaño
    let size_x = date_x + date_w;
    let size_btn_rect = Rect::from_min_size(pos2(size_x, header_rect.min.y), egui::vec2(size_w, header_height));
    let size_label = match current_order {
        OrderingMode::SizeAsc  => "Tamaño ↑",
        OrderingMode::SizeDesc => "Tamaño ↓",
        _ => "Tamaño",
    };
    if ui.put(size_btn_rect, egui::Button::new(
        RichText::new(size_label).size(11.0).color(Color32::from_rgb(140, 140, 160))
    ).frame(false)).clicked() {
        with_configs(|c| {
            c.set_ordering_mode(match current_order { OrderingMode::SizeAsc => OrderingMode::SizeDesc, _ => OrderingMode::SizeAsc });
        });
        state.refresh();
    }


    let handle_w = 4.0;
    for (x, id_w, current_w) in [
        (header_rect.min.x + name_w, id_name_w, name_w),
    ] {
        let handle_rect = Rect::from_min_size(
            pos2(x - handle_w / 2.0, header_rect.min.y),
            egui::vec2(handle_w, header_height),
        );
        let handle_id = ui.id().with(("resize_handle", id_w));
        let handle_response = ui.interact(handle_rect, handle_id, Sense::click_and_drag());

        if handle_response.hovered() || handle_response.dragged() {
            ui.set_cursor_icon(egui::CursorIcon::ResizeColumn);
            painter.rect_filled(handle_rect, 0.0, Color32::from_rgb(100, 100, 140));
        } else {
            painter.rect_filled(handle_rect, 0.0, Color32::from_rgb(50, 50, 60));
        }

        if handle_response.dragged() {
            let new_w = (current_w + handle_response.drag_delta().x).max(80.0);
            ui.data_mut(|d| d.insert_temp(id_w, new_w));
        }
    }


    let scroll_area = ScrollArea::vertical()
        .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
        .auto_shrink([false, false])
        .vertical_scroll_offset(state.scroll_offset);

    let scroll_output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
        ui.spacing_mut().item_spacing.y = 0.0;

        state.row_view.first_visible = row_range.start.clone();
        state.row_view.last_visible = row_range.end;

        let info_snapshot: HashMap<PathBuf, ExtendedInfo> = {
            match state.extended_info_manager.info_map.try_read() {
                Ok(map) => {
                    row_range.clone()
                        .filter_map(|i| {
                            let path = &files[i].full_path;
                            map.get(path).map(|v| (path.clone(), v.clone()))
                        })
                        .collect()
                },
                Err(_) => HashMap::new(),
            }
        };

        let color_snapshot: HashMap<FileId, Color32> = {
            match ui_state.folder_color_manager.cache_manager.color_cache.try_read() {
                Ok(guard) => {
                    row_range.clone()
                        .filter_map(|i|{
                            files[i].unique_id.as_ref()
                                .and_then(|id| guard.get(id).map(|c| (id.clone(), c.color)))
                        })
                        .collect()
                },
                Err(_) => HashMap::new(),
            }
        };

        for i in row_range.clone() {
            let file = &files[i];
            let is_renaming = state.renaming_file.as_deref() == Some(&file.full_path);

            if is_renaming {
                let (rect, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), row_height),
                    Sense::hover(),
                );

                if i == row_range.start {
                    state.row_view.scroll_area_origin_y = rect.min.y + state.scroll_offset - (i as f32 * row_height);
                }

                render_rename_field(ui, file, state, rect, is_renaming);
                continue;
            }


            let (rect, response) = ui.allocate_exact_size(
                vec2(ui.available_width(), row_height),
                Sense::click_and_drag(),
            );

            // --- Corrección de la rubberband ---
            if i == row_range.start {
                state.row_view.scroll_area_origin_y = rect.min.y + state.scroll_offset - (i as f32 * row_height);
            }

            // --- Selección y hover ---
            if state.is_selected(i) {
                ui.painter().rect_filled(rect, 5.0, Color32::from_rgba_unmultiplied(100, 100, 255, 60));
            }
            if response.hovered() {
                ui.set_cursor_icon(CursorIcon::PointingHand);
                ui.painter().rect_filled(rect, 5.0, Color32::from_rgba_unmultiplied(255, 255, 255, 15));
            }

            // --- Toda la lógica de interacción original ---
            handle_row_interactions(ui, &response, i, file, state, ui_state, files, content_rect, rect);

            // clicks y selección
            if response.double_clicked() {
                if file.is_dir { state.navigate_to(file.full_path.clone()); }
                else { state.open_file(&file); }
            }

            if response.clicked_by(egui::PointerButton::Primary) {
                let modifiers = ui.input(|i| i.modifiers);
                if modifiers.shift {
                    if let Some(anchor) = state.selection_anchor {
                        let start = anchor.min(i);
                        let end = anchor.max(i);
                        state.select_range(start, end);
                    } else {
                        state.deselect_all();
                        state.resize_selection(files.len());
                        state.selection.set(i, true);
                        state.selection_anchor = Some(i);
                    }

                    state.last_selected_index = Some(i);

                } else if modifiers.ctrl {
                    let currently = state.is_selected(i);
                    state.resize_selection(files.len());
                    state.selection.set(i, !currently);
                    state.selection_anchor = Some(i);
                    state.last_selected_index = Some(i);
                } else {
                    state.deselect_all();
                    state.resize_selection(files.len());
                    state.selection.set(i, true);
                    state.selection_anchor = Some(i);
                    state.last_selected_index = Some(i);
                }
            }


            // --- Contenido de la fila ---
            let extended = info_snapshot.get(&file.full_path);
            let git = extended.and_then(|e| e.git_status.as_ref());
            let name_color = text_color_for_git(git);
            let dot_color  = git_dot_color(git);

            // Columna Filename
            let icon_size    = 16.0;
            let dot_size     = 8.0;
            let icon_spacing = 4.0;
            let name_start_x = rect.min.x + icon_size + dot_size + icon_spacing * 2.0;
            let name_end_x   = rect.min.x + name_w;

            let (icon_name, icon_bytes, color) = resolve_icon(file, &color_snapshot);
            let icon = ui_state.icon_cache.get_or_load(ui, &icon_name, icon_bytes, color);

            let icon_rect = Rect::from_min_size(
                pos2(rect.min.x, rect.center().y - icon_size / 2.0),
                vec2(icon_size, icon_size),
            );
            ui.painter().image(icon.id(), icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);

            // Dot git
            let dot_center = pos2(rect.min.x + icon_size + icon_spacing + dot_size / 2.0, rect.center().y);
            if let Some(dot) = dot_color {
                ui.painter().circle_filled(dot_center, 3.5, dot);
                let dot_rect = Rect::from_center_size(dot_center, vec2(dot_size, dot_size));
                if let Some(git_status) = git {
                    let label = match git_status {
                        GitStatus::Modified  => "Modificado",
                        GitStatus::Staged    => "Preparado",
                        GitStatus::Untracked => "No rastreado",
                        GitStatus::Ignored   => "Ignorado",
                        GitStatus::Conflict  => "Conflicto",
                        GitStatus::Deleted   => "Eliminado",
                        GitStatus::Clean     => "Limpio",
                    };
                    ui.interact(dot_rect, ui.id().with(("dot", i)), Sense::hover())
                        .on_hover_text(label);
                }
            }

            // Nombre
            let motor = state.motor.borrow_mut();
            let display_name = if motor.active_tab().is_recursive_active {
                file.full_path.strip_prefix(&motor.active_tab().cwd)
                    .unwrap_or(&file.full_path).to_string_lossy().to_string()
            } else {
                file.name.to_string()
            };
            drop(motor);

            let name_rect = Rect::from_min_max(
                pos2(name_start_x, rect.min.y),
                pos2(name_end_x, rect.max.y),
            );
            let name_galley = ui.fonts_mut(|f| f.layout_no_wrap(
                display_name,
                FontId::proportional(13.0),
                name_color,
            ));
            let name_painter = ui.painter().with_clip_rect(name_rect);
            name_painter.galley(
                pos2(name_start_x, rect.center().y - name_galley.size().y / 2.0),
                name_galley, name_color,
            );

            // Columna Modified
            let date_rect = Rect::from_min_max(
                pos2(rect.min.x + name_w, rect.min.y),
                pos2(rect.min.x + name_w + date_w, rect.max.y),
            );
            let date_galley = ui.fonts_mut(|f| f.layout_no_wrap(
                format_date(file.modified),
                FontId::proportional(12.0),
                Color32::from_rgb(109, 108, 111),
            ));
            ui.painter().with_clip_rect(date_rect).galley(
                pos2(date_rect.min.x + 4.0, rect.center().y - date_galley.size().y / 2.0),
                date_galley, Color32::WHITE,
            );

            // Columna Size
            let size_rect = Rect::from_min_max(
                pos2(rect.min.x + name_w + date_w, rect.min.y),
                pos2(rect.max.x, rect.max.y),
            );
            let display_size = if file.is_dir {
                if state.calculating_dir_sizes.contains(&file.full_path) { None }
                else if state.calculated_dir_sizes.contains(&file.full_path) { Some(file.size) }
                else { state.sizer_manager.cache_manager.get_cached_size(&file.full_path) }
            } else {
                Some(file.size)
            };
            let size_text = match display_size {
                None => "...".to_string(),
                Some(0) if file.is_dir => "-".to_string(),
                Some(size) => format_size(size),
            };
            let size_galley = ui.fonts_mut(|f| f.layout_no_wrap(
                size_text,
                FontId::proportional(12.0),
                Color32::from_rgb(109, 108, 111),
            ));
            ui.painter().with_clip_rect(size_rect).galley(
                pos2(size_rect.min.x + 4.0, rect.center().y - size_galley.size().y / 2.0),
                size_galley, Color32::WHITE,
            );
        }

        

        // Context menu
        let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);
        match ctx_menu.kind {
            ContextMenuKind::FileNormal => ctx_menu.file_context_menu(ui, state, ui_state, files),
            ContextMenuKind::FileTrash  => ctx_menu.file_context_menu_in_trash(ui, state, ui_state, files),
            _ => {}
        }
        ui_state.context_menu_state = ctx_menu;
        
    });

    if !state.rubber_band.is_rubber_banding {
        state.scroll_offset = scroll_output.state.offset.y;
    }
}