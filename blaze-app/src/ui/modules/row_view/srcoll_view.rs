use std::{path::PathBuf, sync::Arc};
use egui::{Button, Color32, FontId, Key, Rect, ScrollArea, Sense, Ui, UiBuilder, pos2, scroll_area::ScrollSource, vec2};
use tracing::{error, info};
use crate::{core::{blaze_state::BlazeCoreState, configs::config_state::with_configs, files::{file_extension::{DocType, FileExtension}, motor::FileEntry}}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons}, utils::{channel_pool::{FileOperation, SureTo, UiEvent}, formating::{format_date, format_size}}};


pub fn render_scrollview(ctx: &egui::Context, files: &Vec<Arc<FileEntry>>, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, ui: &mut Ui, row_height: f32, total_rows: usize, content_rect: Rect) {
    let scroll_area = ScrollArea::vertical()
        .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
        .auto_shrink([false, false])
        .vertical_scroll_offset(state.scroll_offset);

        let scroll_output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {

            ui.spacing_mut().item_spacing.y = 0.0;

            state.row_view.first_visible = row_range.start;
            state.row_view.last_visible = row_range.end;

            for i in row_range {
                let file = &files[i];

                let is_renaming = state.renaming_file.as_deref() == Some(&file.full_path);

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width() * 0.80, row_height),
                    Sense::click_and_drag()
                );


                if i == state.row_view.first_visible {
                    state.row_view.scroll_area_origin_y = rect.min.y + state.scroll_offset - (i as f32 * row_height);
                }

                if response.drag_started() {
                    if !state.is_selected(i) {
                        state.deselect_all();
                        state.resize_selection(files.len());
                        state.selection.set(i, true);
                        state.last_selected_index = Some(i);
                        state.selection_anchor = Some(i);
                    }
                    state.row_view.is_dragging_files = true;
                }

                if response.dragged() {
                    state.row_view.drag_ghost_pos = ctx.input(|i| i.pointer.interact_pos());
                }


                if response.drag_stopped() {
                    state.row_view.drag_ghost_pos = None;

                    let drop_in_file_area = ctx.input(|i| i.pointer.interact_pos())
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
                            let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                            let sources = state.get_selected_paths(files);   

                            if sources.iter().all(|p| p.parent() == Some(&cwd)) {
                                continue;
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

                let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                let trash = state.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();

                if trash == cwd {
                    let Some(sender) = state.sender().cloned() else {return;};
                    let tab_id = state.motor.borrow_mut().active_tab().id;

                    response.context_menu(|ui| {

                        let sources = state.get_selected_paths(files);
                        let file_names: Vec<String> = sources.iter()
                            .map(|p| PathBuf::from(p)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned())
                            .collect();

                        if ui.button("Restaurar").clicked() {
                            sender.send_fileop(
                                FileOperation::RestoreDeletedFiles {
                                    file_names
                                }
                            ).ok();
                        }

                        ui.separator();

                        if ui.button("Eliminar").clicked() {
                            sender.send_ui_event(
                                UiEvent::SureTo(
                                    SureTo::SureToDelete { 
                                        files: sources, 
                                        tab_id 
                                    }
                                )
                            ).ok();
                        }
                    });
                } else {
                    response.context_menu(|ui| {
                        if !file.is_dir {
                            let respo =  ui.menu_button("Abrir...", |ui|{
                                if ui.button("Abrir con...").clicked() {
                                    state.open_file_with(&file);
                                    ui.close();
                                }
                            }).response;

                            if respo.clicked() {
                                if file.is_dir {
                                    state.navigate_to(file.full_path.clone());
                                } else {
                                    state.open_file(&file);
                                }
                                ui.close();
                            }
                        } else {
                            if ui.button("Abrir").clicked() {
                                state.navigate_to(file.full_path.clone());
                            }
                        }

                        if ui.add_enabled(state.clipboard.clipboard_has_files() && file.is_dir, Button::new("Pegar aquí")).clicked() {
                            state.paste(file.full_path.clone());
                            ui.close();
                        }
                        
                        if ui.button("Copiar").clicked() {
                            state.copy(files);
                            ui.close();
                        }
                        if ui.button("Cortar").clicked() {
                            state.cut(files);
                            ui.close();
                        }

                        ui.separator();


                        let is_in_fav = with_configs(|c| {
                            c.is_in_favorite(file.full_path.clone())
                        });
                    
                        if !is_in_fav {
                            if ui.button("Agregar a favoritos").clicked() {
                                with_configs(|c| {
                                    c.add_to_favorites(file.name.to_string(),file.full_path.clone(), file.is_dir)
                                });
                            }
                        } else {
                            if ui.button("Quitar de favoritos").clicked() {
                                with_configs(|c| {
                                    c.delete_from_favorites(file.name.to_string(),file.full_path.clone())
                                });
                            }
                        }
                        

                        ui.separator();
                        
                        if ui.button("Borrar").clicked() {
                            state.move_to_trash(files);
                            ui.close();
                        }

                        if ui.button("Renombrar").clicked() {
                            state.renaming_file = Some(file.full_path.clone());
                            state.rename_buffer = file.name.to_ascii_lowercase();
                            ui.close();
                        }
                    });
                }



                if state.is_selected(i) {
                    ui.painter().rect_filled(rect, 5.0,egui::Color32::from_rgba_unmultiplied(100, 100, 255, 60));
                }

                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(rect, 5.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15));
                }


                if response.double_clicked() {
                    if file.is_dir {
                        state.navigate_to(file.full_path.clone());
                    } else {
                        state.open_file(&file);
                    }
                }

                if response.clicked() {
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
                
                if is_renaming {
                    let mut temp_ui = ui.new_child(UiBuilder::new().max_rect(rect));
                    let response = temp_ui.put(rect,
                        egui::TextEdit::singleline(&mut state.rename_buffer)
                            .frame(false)
                            .margin(vec2(0.0, 5.0)) 
                            .font(egui::FontId::default()) 
                    );

                    if is_renaming && !response.has_focus() {
                        response.request_focus();
                    }

                    if !response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        info!("coso");
                        if let Err(e) = state.clipboard.rename_file(&file.name, &state.rename_buffer) {
                            error!("Error renombrando: {}", e);
                        }
                        state.renaming_file = None;
                    }

                    let cancel = ui.input(|i| i.key_pressed(Key::Escape)) || ui.input(|i| i.pointer.any_click());


                    if response.lost_focus() || cancel {
                        state.renaming_file = None;
                    }

                } else {

                    //Sin renombrado

                    let (icon_name, icon_bytes, color) = if file.is_dir {
                        ("folder", icons::ICON_FOLDER, Color32::YELLOW)
                    } else {
                        match &file.extension {
                            FileExtension::Image(_) => ("image", icons::ICON_IMAGE,    Color32::from_rgb(100, 200, 255)),
                            FileExtension::Document(DocType::Pdf) => ("pdf",      icons::ICON_PDF, Color32::from_rgb(255, 80,  80)),
                            FileExtension::Document(_) => ("doc", icons::ICON_DOC, Color32::from_rgb(100, 140, 255)),
                            FileExtension::Video(_) => ("video", icons::ICON_VIDEO,    Color32::from_rgb(200, 100, 255)),
                            FileExtension::Audio(_) => ("audio", icons::ICON_VIDEO,    Color32::from_rgb(255, 200, 80)),
                            FileExtension::Archive(_) => ("archive", icons::ICON_ARCHIVE,  Color32::from_rgb(255, 160, 60)),
                            FileExtension::Code(_) => ("code", icons::ICON_CODE,     Color32::from_rgb(100, 255, 150)),
                            FileExtension::Font(_) => ("font", icons::ICON_FONT,     Color32::from_rgb(200, 200, 200)),
                            FileExtension::Executable(_) => ("exe", icons::ICON_EXE,      Color32::from_rgb(255, 100, 100)),
                            FileExtension::Unknown => ("file", icons::ICON_FILE, Color32::WHITE),
                        }
                    };

                    
                    let icon = ui_state.icon_cache.get_or_load(ctx, icon_name, icon_bytes, color);

                    let icon_size = egui::vec2(16.0, 16.0);
                    let icon_spacing = 4.0;
                    let icon_pos = rect.left_center() - egui::vec2(0.0, icon_size.y / 2.0);
                    let icon_rect = Rect::from_min_size(icon_pos, icon_size);

                    ui.painter().image(
                        icon.id(),
                        icon_rect,
                        Rect::from_min_max(egui::pos2(0.0, 0.0), 
                        pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );



                    let mut motor = state.motor.borrow_mut();
                    let display_name = if motor.active_tab().is_recursive_active {
                        file.full_path
                            .strip_prefix(&motor.active_tab().cwd)
                            .unwrap_or(&file.full_path)
                            .to_string_lossy()
                            .to_string()
                    } else {
                        file.name.to_string()
                    };

                    
                    let display_size = if file.is_dir {
                        if state.calculating_dir_sizes.contains(&file.full_path) {
                            None
                        } else {
                            state.sizer_manager.cache_manager.get_cached_size(&file.full_path)
                        }
                    } else {
                        Some(file.size)
                    };

                    let size_text = match display_size {
                        None => "...",
                        Some(0) if file.is_dir => "-",
                        Some(size) => &format_size(size),
                    };


                    let min_name_width = 40.0;
                    let date_col_width = 36.0;
                    let size_col_width = 48.0; 
                    let col_gap = 12.0;
                    let right_margin = 8.0;

                    let date_right = rect.max.x - right_margin;
                    let size_right = date_right - date_col_width - col_gap;
                    let name_right = size_right - size_col_width - col_gap;

                    let name_start_x = icon_rect.right() + icon_spacing;
                    let name_max_width = (name_right - name_start_x).max(min_name_width);

                    
                    let size_galley = ui.fonts_mut(|f| {
                        f.layout_no_wrap(
                            size_text.to_owned(),
                            FontId::proportional(12.0),
                            Color32::from_rgb(109, 108, 111),
                        )
                    });


                    let date_text = format_date(file.modified);
                    let date_galley = ui.fonts_mut(|f| {
                        f.layout_no_wrap(
                            date_text,
                            FontId::proportional(12.0),
                            Color32::from_rgb(109, 108, 111),
                        )
                    });


                    let chars: Vec<char> = display_name.chars().collect();
                    let mut lo = 0;
                    let mut hi = chars.len();

                    while lo < hi {
                        let mid = (lo + hi + 1) / 2;
                        let candidate: String = chars[..mid].iter().collect();
                        let test_text = if mid < chars.len() {
                            format!("{}…", candidate)
                        } else {
                            candidate
                        };

                        let g = ui.fonts_mut(|f| {
                            f.layout_no_wrap(
                                test_text,
                                FontId::proportional(14.0),
                                Color32::from_rgb(189, 189, 189),
                            )
                        });

                        if g.size().x <= name_max_width {
                            lo = mid;
                        } else {
                            hi = mid - 1;
                        }
                    }

                    let final_text = if lo < chars.len() {
                        format!("{}…", chars[..lo].iter().collect::<String>())
                    } else {
                        display_name.clone()
                    };


                    let name_galley = ui.fonts_mut(|f| {
                        f.layout_no_wrap(
                            final_text,
                            FontId::proportional(14.0),
                            Color32::from_rgb(189, 189, 189),
                        )
                    });

                    let y_center = rect.center().y;

                    let date_pos = pos2(
                        date_right - date_galley.size().x,
                        y_center - date_galley.size().y / 2.0,
                    );

                    let size_pos = pos2(
                        size_right - size_galley.size().x,
                        y_center - size_galley.size().y / 2.0,
                    );

                    let name_pos = pos2(
                        name_start_x,
                        y_center - name_galley.size().y / 2.0,
                    );


                    let painter = ui.painter().with_clip_rect(rect);

                    painter.galley(name_pos, name_galley, ui.visuals().text_color());
                    painter.galley(size_pos, size_galley, ui.visuals().text_color());
                    painter.galley(date_pos, date_galley, ui.visuals().text_color());
                }
            }
    });


    if !state.rubber_band.is_rubber_banding {
        state.scroll_offset = scroll_output.state.offset.y;
    }
}