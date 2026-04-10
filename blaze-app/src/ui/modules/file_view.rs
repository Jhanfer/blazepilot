// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.






use std::sync::Arc;
use egui::{Align2, Area, Button, CentralPanel, Color32, CornerRadius, FontId, Frame, Key, Margin, Painter, Rect, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, UiBuilder, pos2, scroll_area::ScrollSource, vec2};
use tracing::{error, info};
use crate::{core::{blaze_state::{BlazeCoreState, NewItemType}, configs::config_state::with_configs, files::{file_extension::{DocType, FileExtension}, motor::{FileEntry, TabState}}, system::clipboard::TOKIO_RUNTIME}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons::{self}, modules::sidebar_right::sidebar_right_component, task_manager::task_manager::TaskStatus}, utils::{channel_pool::{FileOperation, SureTo, UiEvent}, formating::{format_date, format_size}}};



pub fn drag_files(state: &mut BlazeCoreState, clipped_painter: &Painter) {
    if let Some(pos) = state.row_view.drag_ghost_pos {
        let count = state.selected_files.len();
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

        let first_name = state.selected_files.iter().next()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("archivo");

        clipped_painter.text(
            pos + vec2(10.0, 14.0),
            egui::Align2::LEFT_CENTER,
            if count > 1 { format!("{} y {} más", first_name, count - 1) } else { first_name.to_string() },
            egui::FontId::default(),
            Color32::WHITE,
        );

    }
    
}


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

        add_arc(rect.right_top() + egui::vec2(-radius, radius), -std::f32::consts::FRAC_PI_2);
        add_arc(rect.right_bottom() + egui::vec2(-radius, -radius), 0.0);
        add_arc(rect.left_bottom() + egui::vec2(radius, -radius), std::f32::consts::FRAC_PI_2);
        add_arc(rect.left_top() + egui::vec2(radius, radius), std::f32::consts::PI);


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

        state.selected_files.clear(); 

        for (i, file) in files.iter().enumerate() {
            let file_y_min = state.row_view.scroll_area_origin_y + i as f32 * row_height - state.scroll_offset;
            let file_y_max = file_y_min + row_height;

            let file_rect = Rect::from_min_max(
                pos2(content_rect.min.x, file_y_min),
                pos2(content_rect.min.x + content_rect.width() * 0.80, file_y_max),
            );

            if rect.intersects(file_rect) {
                state.selected_files.insert(file.full_path.clone());
            }
        }
    }
}




pub fn file_view_component(ctx: &egui::Context, files: &Vec<Arc<FileEntry>>, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    
    let custom_frame = Frame::NONE
        .fill(Color32::from_rgb(16, 21, 25))
        .inner_margin(Margin::same(20));

    CentralPanel::default()
        .frame(custom_frame)
        .show(ctx, |ui| {
        
        ui.set_width(ui.available_width() + 20.0);

        Frame::NONE
            .inner_margin(egui::Margin::same(10))
            .fill(Color32::from_rgb(27, 31, 35))
            .corner_radius(CornerRadius::same(20))
            .show(ui, |ui|{

                ui.horizontal(|ui|{
                    ui.visuals_mut().button_frame = false;

                    if ui.button("📁 Nueva carpeta").clicked() {
                        state.creating_new = Some(NewItemType::Folder);
                        state.new_item_buffer = "nueva carpeta".to_string(); 
                    }

                    let cut = ui.add_enabled(!state.selected_files.is_empty(), Button::new("✂ Cortar"));

                    if cut.clicked() {
                        state.cut(files);
                    }

                    let cop = ui.add_enabled(!state.selected_files.is_empty(), Button::new("📋 Copiar"));

                    if cop.clicked() {
                        state.copy(files);
                    }


                    let pas = ui.add_enabled(state.clipboard.clipboard_has_files(), Button::new("📋 Pegar"));

                    if pas.clicked() {
                        let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                        state.paste(cwd);
                    }

                    let del = ui.add_enabled(!state.selected_files.is_empty(), Button::new("🗑 Borrar"));

                    if del.clicked() {
                        let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                        let trash = state.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();

                        if trash == cwd {
                            let Some(sender) = state.sender().cloned() else {return;};
                            let tab_id = state.motor.borrow_mut().active_tab().id;

                            let sources: Vec<_> = state.selected_files.iter().cloned().collect();

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

                    ui.add_space(8.0);

                    let select_all_text = if state.selected_files.len() == files.len() && !files.is_empty() {
                        "Deseleccionar todo" 
                    } else {
                        "Seleccionar todo"
                    };

                    if ui.button(select_all_text).clicked() {
                        state.toggle_select_all(files);
                    }

                    ui.separator();
                    if ui.button("🔄").clicked() {
                        state.refresh();
                    }

                    if ui.button("T").clicked() {
                        state.is_testing = !state.is_testing;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.visuals_mut().button_frame = false;

                        let mut is_hidden = with_configs(|c| {c.configs.show_hidden_files.clone()});

                        let resp = ui.checkbox(&mut is_hidden, "");

                        if resp.clicked() {
                            with_configs(|c| {
                                c.set_show_hidden_files(is_hidden);
                            });
                            state.refresh();
                        };

                        ui.separator();
                    });
                });


                ui.add_space(10.0);

                let content_rect = ui.available_rect_before_wrap();
                let panel_top = content_rect.min.y;
                let clipped_painter = &ui.painter_at(content_rect);

                let row_height = 30.0;
                let total_rows = files.len();
                let mut first_visible: usize = 0;
                let mut last_visible: usize = 0;


                //Drag
                if state.row_view.is_dragging_files {
                    drag_files(state, clipped_painter);
                }
                
                //Rubberband 
                if state.rubber_band.is_rubber_banding {
                    render_rubberband(state, files, clipped_painter, panel_top, content_rect, row_height);
                }



                if !ctx.memory(|m| m.focused().is_some()) {
                    ui.memory_mut(|m| m.request_focus(ui.id()));
                }


                let input = ui.input(|i| i.clone());
                let disable_keys = state.renaming_file.is_none() && state.creating_new.is_none();


                if let Some(item_type) = state.creating_new.clone() {
                    ui.horizontal(|ui|{
                        let response = ui.add(TextEdit::singleline(&mut state.new_item_buffer));

                        if !state.focus_requested {
                            response.request_focus();
                            state.focus_requested = true;
                        }

                        if ui.input(|i| i.key_pressed(Key::Enter)) && !state.new_item_buffer.trim().is_empty() {
                            let cwd = state.motor.borrow_mut().active_tab().cwd.clone();

                            let result = match item_type {
                                NewItemType::File => {state.clipboard.create_new_file(&state.new_item_buffer, cwd)},
                                NewItemType::Folder => {state.clipboard.create_new_dir(&state.new_item_buffer, cwd)},
                            };

                            if let Err(e) = result {
                                error!("Error creando: {}", e);
                            }

                            state.creating_new = None;
                            state.refresh();
                            state.focus_requested = false;
                        }


                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) || (response.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            state.creating_new = None;
                            state.focus_requested = false;
                        }

                    });
                }


                //tecla de abajo
                if input.key_pressed(Key::ArrowDown) && disable_keys {
                    let next = state.last_selected_index
                        .map(|i| (i + 1).min(total_rows - 1))
                        .unwrap_or(first_visible);
                    state.selected_files.clear();
                    state.selected_files.insert(files[next].full_path.clone());
                    state.last_selected_index = Some(next);
                    state.pending_scroll_to = Some(next);
                }

                //tecla de arriba
                if input.key_pressed(Key::ArrowUp) && disable_keys {
                    let prev = state.last_selected_index
                        .map(|i| i.saturating_sub(1))
                        .unwrap_or(last_visible);
                    state.selected_files.clear();
                    state.selected_files.insert(files[prev].full_path.clone());
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
                    state.select_all(files);
                }

                let bg_id = ui.id().with("background_interact");
                let bg_response = ui.interact(ui.available_rect_before_wrap(), bg_id, egui::Sense::click_and_drag());

                if bg_response.drag_started() {
                    if let Some(orgin) = ctx.input(|i| i.pointer.press_origin()) {
                        state.rubber_band.rubber_band_start_content_y = orgin.y - panel_top + state.scroll_offset;
                        state.rubber_band.rubber_band_start = Some(orgin);
                    }
                    state.rubber_band.is_rubber_banding = true;
                }

                if bg_response.dragged() {
                    state.rubber_band.rubber_band_current = ctx.input(|i| i.pointer.interact_pos());

                    if let Some(current) = state.rubber_band.rubber_band_current {
                        let total_content_height = total_rows as f32 * row_height;
                        let max_scroll = (total_content_height - content_rect.height()).max(0.0) + 80.0;

                        if total_content_height <= content_rect.height() {
                            state.scroll_offset = 0.0;
                        } else {
                            let scroll_speed = 14.0;
                            let scroll_zone = 60.0;

                            if current.y > content_rect.max.y - scroll_zone {
                                let distance = (current.y - (content_rect.max.y - scroll_zone)) / scroll_zone;
                                let acceleration = distance * distance;
                                state.scroll_offset += scroll_speed * acceleration;
                            }

                            if current.y < content_rect.min.y + scroll_zone {
                                let distance = ((content_rect.min.y + scroll_zone) - current.y) / scroll_zone;
                                let acceleration = distance * distance;
                                state.scroll_offset -= scroll_speed * acceleration;
                            }

                            state.scroll_offset = state.scroll_offset.clamp(0.0, max_scroll);
                        }
                    }
                }

                if bg_response.drag_stopped() {
                    state.rubber_band.is_rubber_banding = false;
                    state.rubber_band.rubber_band_start = None;
                    state.rubber_band.rubber_band_current = None;
                }


                let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                let trash = state.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();
                
                if trash == cwd {
                    let Some(sender) = state.sender().cloned() else {return;};
                    let tab_id = state.motor.borrow_mut().active_tab().id;

                    bg_response.context_menu(|ui| {

                        let file_names: Vec<_> = state.selected_files.iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string() ).collect();

                        let res = ui.add_enabled(state.selected_files.is_empty(), Button::new("Restaurar"));

                        if res.clicked() {
                            sender.send_fileop(
                                FileOperation::RestoreDeletedFiles {
                                    file_names
                                }
                            ).ok();
                        }

                        ui.separator();

                        let del = ui.add_enabled(state.selected_files.is_empty(), Button::new("Eliminar"));

                        if del.clicked() {

                            let sources: Vec<_> = state.selected_files.iter().cloned().collect();

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
                    bg_response.context_menu(|ui| {
                        if ui.add_enabled(state.clipboard.clipboard_has_files(), Button::new("Pegar aquí")).clicked() {
                            let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                            state.paste(cwd);
                            ui.close();
                        }

                        if ui.button("Nueva carpeta").clicked() {
                            state.creating_new = Some(NewItemType::Folder);
                            state.new_item_buffer = "nueva carpeta".to_string(); 
                            ui.close();
                        }
                        if ui.button("Nuevo archivo").clicked() {
                            state.creating_new = Some(NewItemType::File);
                            state.new_item_buffer = "nuevo archivo".to_string();
                            ui.close();
                        }
                    });
                }


                



                if bg_response.clicked() {
                    state.last_selected_index = None;
                    state.selected_files.clear();
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
                                if file.is_dir && !state.selected_files.contains(&file.full_path) {
                                    state.row_view.drop_target = Some(file.full_path.clone());
                                } else if !file.is_dir {
                                    state.row_view.drop_invalid_target = Some(file.full_path.clone());
                                }
                            }
                        }
                    }
                }


                let scroll_area = ScrollArea::vertical()
                .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
                .auto_shrink([false, false])
                .vertical_scroll_offset(state.scroll_offset);

                let scroll_output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {

                    ui.spacing_mut().item_spacing.y = 0.0;
                    first_visible = row_range.start;
                    last_visible = row_range.end;

                    for i in row_range {
                        let file = &files[i];

                        let is_renaming = state.renaming_file.as_deref() == Some(&file.full_path);

                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width() * 0.80, row_height),
                            Sense::click_and_drag()
                        );


                        if i == first_visible {
                            state.row_view.scroll_area_origin_y = rect.min.y + state.scroll_offset - (i as f32 * row_height);
                        }

                        if response.drag_started() {
                            if !state.selected_files.contains(&file.full_path) {
                                state.selected_files.clear();
                                state.selected_files.insert(file.full_path.clone());
                                state.last_selected_index = Some(i);
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
                                    let sources: Vec<_> = state.selected_files.iter().cloned().collect();
                                    
                                    sender.send_ui_event(UiEvent::SureTo(
                                            SureTo::SureToMove { 
                                                files: sources, 
                                                dest: target,
                                                tab_id,
                                            }
                                        )).ok();

                                } else {
                                    let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                                    let sources: Vec<_> = state.selected_files.iter().cloned().collect();

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
                            if ui.input(|i| i.modifiers.ctrl) {
                                state.selected_files.insert(file.full_path.clone());
                                state.last_selected_index = Some(i);
                            } else if !state.selected_files.contains(&file.full_path) {
                                state.selected_files.clear();
                                state.selected_files.insert(file.full_path.clone());
                                state.last_selected_index = Some(i);
                            }
                        }


                        if trash == cwd {
                            let Some(sender) = state.sender().cloned() else {return;};
                            let tab_id = state.motor.borrow_mut().active_tab().id;

                            response.context_menu(|ui| {

                                let file_names: Vec<_> = state.selected_files.iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string() ).collect();

                                if ui.button("Restaurar").clicked() {
                                    sender.send_fileop(
                                        FileOperation::RestoreDeletedFiles {
                                            file_names
                                        }
                                    ).ok();
                                }

                                ui.separator();

                                if ui.button("Eliminar").clicked() {

                                    let sources: Vec<_> = state.selected_files.iter().cloned().collect();

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



                        if state.selected_files.contains(&file.full_path) {
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
                            if ui.input(|i| i.modifiers.shift) {
                                if let Some(last) = state.last_selected_index {
                                    let start = last.min(i);
                                    let end = last.max(i);
                                    for idx in start..=end {
                                        state.selected_files.insert(files[idx].full_path.clone());
                                    }
                                } else {
                                    state.selected_files.insert(file.full_path.clone());
                                    state.last_selected_index = Some(i);
                                }
                            } else if ui.input(|i| i.modifiers.ctrl) {
                                if !state.selected_files.remove(&file.full_path) {
                                    state.selected_files.insert(file.full_path.clone());
                                }
                                state.last_selected_index = Some(i);
                            } else {
                                state.selected_files.clear();
                                state.selected_files.insert(file.full_path.clone());
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

                            let file_size = if file.is_dir {
                                state.dir_size_cache.get(&file.full_path).unwrap_or_else(|| &0)
                            } else {
                                &file.size
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



                            let size_text = format_size(*file_size);
                            let size_galley = ui.fonts_mut(|f| {
                                f.layout_no_wrap(
                                    size_text,
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


                if state.row_view.is_dragging_files && !ctx.input(|i| i.pointer.any_down()) {
                    state.row_view.is_dragging_files = false;
                    state.row_view.drag_ghost_pos = None;
                    state.row_view.drop_target = None;
                    state.row_view.drop_invalid_target = None;
                }


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

                if let Some(sender) = state.sender().cloned() {
                    for i in first_visible..last_visible.min(files.len()) {
                        let file = &files[i];

                        if file.is_dir && file.size == 0 {
                            if !state.calculating_dir_sizes.contains(&file.full_path) {
                                state.calculating_dir_sizes.insert(file.full_path.clone());

                                let path = file.full_path.clone();
                                let sender_clone = sender.clone();
                                let generation = state.motor.borrow_mut().active_tab().active_generation;

                                TOKIO_RUNTIME.spawn(async move {
                                    let size = TabState::get_recursive_size(&path, 12).await;

                                    sender_clone.send_fileop(
                                        FileOperation::UpdateDirSize {
                                            full_path: path,
                                            size, 
                                            gene: generation,
                                        }
                                    ).ok();
                                });
                            }
                        }
                    }
                }
        });


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

        let current_width = 150.0 + eased * (target_width - 150.0);
        let current_height = 20.0 + eased * (target_height - 20.0);
        let anchor_y = -35.0;

        Area::new("processing_bubble".into())
            .anchor(Align2::CENTER_BOTTOM, [0.0, anchor_y ])
            .default_size(vec2(current_width, current_height))
            .order(egui::Order::Background)
            .show(ctx, |ui| {

                Frame::NONE
                    .inner_margin(egui::Margin::same(10))
                    //36
                    .fill(Color32::from_rgb(122, 42, 47))
                    .corner_radius(CornerRadius::same(20))
                    .show(ui, |ui| {

                        ui.set_min_size(vec2(current_width - 20.0, current_height - 20.0));
                        ui.set_max_size(vec2(current_width - 20.0, current_height - 20.0));

                        ScrollArea::vertical()
                            .id_salt("tasks_scroll")
                            .max_height(current_height - 20.0)
                            .show(ui, |ui| {
                                ui.set_min_width(current_width - 20.0);

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


                                        let bar_width = current_width - 40.0;
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



        Area::new("blaze_island".into())
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
                                
                                let selected = state.selected_files.len();
                                ui.label(format!("{}", selected));
                                
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
                                
                                let selected_size: u64 = state.selected_files
                                    .iter()
                                    .filter_map(|selected_path| {
                                        files.iter()
                                            .find(|f| &f.full_path == selected_path)
                                            .map(|f| f.size)
                                    })
                                    .sum();
                                
                                ui.label(format_size(selected_size));

                            });
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