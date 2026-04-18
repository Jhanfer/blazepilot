use std::{path::PathBuf, sync::Arc};
use egui::{Button, CentralPanel, Color32, Context, CornerRadius, Frame, Key, Label, Margin, Rect, RichText, Sense, TextEdit, Ui, pos2};
use tracing::error;
use crate::{core::{blaze_state::{BlazeCoreState, NewItemType}, files::motor::FileEntry, system::{clipboard::TOKIO_RUNTIME, sizer_manager::sizer_manager::SizerMessages}}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons, modules::row_view::{drag_drop_logic::drag_files, hot_keys::hot_keys_logic, island_n_bubble::render_island_bubble, render_drag::render_drag_files, rubber_band_logic::render_rubberband, srcoll_view::render_scrollview, tools_view::tools}}, utils::channel_pool::{FileOperation, SureTo, UiEvent}};


fn new_ff_logic(state: &mut BlazeCoreState, ui: &mut Ui) {
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
}

fn background_response_logic(state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>, ui: &mut Ui, ctx: &Context, panel_top: f32, total_rows: usize, row_height: f32, content_rect: Rect) {
    let bg_id = ui.id().with("background_interact");
    let bg_response = ui.interact(ui.available_rect_before_wrap(), bg_id, egui::Sense::click_and_drag());

    let dragged_by  = bg_response.drag_started_by(egui::PointerButton::Primary) || bg_response.drag_started_by(egui::PointerButton::Secondary);
    
    if dragged_by {
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
            let sources = state.get_selected_paths(files);
            let file_names: Vec<String> = sources.iter()
                .map(|p| PathBuf::from(p)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned())
                .collect();

            
            ui.horizontal(|ui|{
                let (icon, bytes) = ("restore", icons::ICON_RESTORE);

                let icon = ui_state.icon_cache.get_or_load(ctx, icon, bytes, Color32::GRAY);

                let res = ui.add_enabled(
                    !sources.is_empty(),
                    egui::Button::image_and_text(
                        icon,
                        "Restaurar"
                    )
                );

                if res.clicked() {
                    sender.send_fileop(
                        FileOperation::RestoreDeletedFiles {
                            file_names
                        }
                    ).ok();
                }

                //Poner hotkey para esto
            });

            ui.separator();
            
            ui.horizontal(|ui|{
                let (icon, bytes) = ("trash-forever", icons::ICON_TRASH);

                let icon = ui_state.icon_cache.get_or_load(ctx, icon, bytes, Color32::RED);

                let del = ui.add_enabled(
                    !sources.is_empty(),
                    egui::Button::image_and_text(
                        icon,
                        RichText::new("Eliminar")
                                .color(Color32::RED)
                    )
                );

                if del.clicked() {
                    sender.send_ui_event(
                        UiEvent::SureTo(
                            SureTo::SureToDelete {
                                files: sources, 
                                tab_id 
                            }
                        )
                    ).ok();
                }
                
                ui.add(
                    Label::new(
                        RichText::new("Supr")
                                .color(Color32::RED)
                                .small()
                                .weak()
                    )
                    .selectable(false)
                );
            });
        });
    } else {
        bg_response.context_menu(|ui| {
            state.deselect_all();
            state.resize_selection(files.len());

            ui.horizontal(|ui|{
                let (icon, bytes) = ("tab-icon", icons::ICON_TAB_ICON);    

                let icon = ui_state.icon_cache.get_or_load(ctx, icon, bytes, Color32::GRAY);

                let paste = ui.add(
                    egui::Button::image_and_text(
                        icon,
                        "Nueva pestaña"
                    )
                );

                if paste.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if paste.clicked() {
                    state.create_tab();
                    ui.close();
                }

                ui.add(
                    Label::new(
                        RichText::new("Ctrl + N")
                                .small()
                                .weak()
                    )
                    .selectable(false)
                );
            });
            
            ui.separator();

            ui.horizontal(|ui|{
                let has_clipboard = state.clipboard.clipboard_has_files();
                let (icon, bytes) = if has_clipboard {
                    ("clipboard", icons::ICON_CLIPBOARD)
                } else {
                    ("clipboard-disabled", icons::ICON_CLIPBOARD_DISABLE)
                };

                let icon = ui_state.icon_cache.get_or_load(ctx, icon, bytes, Color32::GRAY);


                let paste = ui.add_enabled(
                    has_clipboard,
                    egui::Button::image_and_text(
                        icon,
                        "Pegar aquí"
                    )
                );

                if paste.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if paste.clicked() {
                    let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                    state.paste(cwd);
                    ui.close();
                }

                ui.add(
                    Label::new(
                        RichText::new("Ctrl + V")
                                .small()
                                .weak()
                    )
                    .selectable(false)
                );
            });

            ui.separator();


            ui.horizontal(|ui|{
                let (icon, bytes) = ("terminal", icons::ICON_TERMINAL);

                let icon = ui_state.icon_cache.get_or_load(ctx, icon, bytes, Color32::GRAY);

                let term = ui.add(
                    egui::Button::image_and_text(
                        icon,
                        "Abrir terminal aqui"
                    )
                );

                if term.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if term.clicked() {
                    state.open_terminal_here();
                    ui.close();
                }

                ui.add(
                    Label::new(
                        RichText::new("Alt + T")
                                .small()
                                .weak()
                    )
                    .selectable(false)
                );
            });


            ui.separator();


            ui.horizontal(|ui|{
                let (icon, bytes) = ("plus-folder", icons::ICON_PLUS_FOLDER);

                let icon = ui_state.icon_cache.get_or_load(ctx, icon, bytes, Color32::GRAY);

                let new_fol = ui.add(
                    egui::Button::image_and_text(
                        icon,
                        "Nueva carpeta"
                    )
                );

                if new_fol.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if new_fol.clicked() {
                    state.creating_new = Some(NewItemType::Folder);
                    state.new_item_buffer = "nueva carpeta".to_string(); 
                    ui.close();
                }

                ui.add(
                    Label::new(
                        RichText::new("Ctrl + Shfit + N")
                                .small()
                                .weak()
                    )
                    .selectable(false)
                );
            });

            
            ui.horizontal(|ui|{
                let (icon, bytes) = ("plus-file", icons::ICON_PLUS_FILE);

                let icon = ui_state.icon_cache.get_or_load(ctx, icon, bytes, Color32::GRAY);

                let new_file = ui.add(
                    egui::Button::image_and_text(
                        icon,
                        "Nuevo archivo"
                    )
                );

                if new_file.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if new_file.clicked() {
                    state.creating_new = Some(NewItemType::File);
                    state.new_item_buffer = "nuevo archivo".to_string();
                    ui.close();
                }

                ui.add(
                    Label::new(
                        RichText::new("Ctrl + Shfit + F")
                                .small()
                                .weak()
                    )
                    .selectable(false)
                );
            });
        });
    }



    if bg_response.clicked() {
        state.deselect_all();
    }
}


pub fn render_row_view(ctx: &egui::Context, files: &Vec<Arc<FileEntry>>, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    state.resize_selection(files.len());

    let tabs_height: i8 = if state.motor.borrow_mut().tabs.len() > 1 {
        50
    } else {
        0
    };

    let bottom_padding = 20.0 as i8;
    
    let custom_frame = Frame::NONE
        .fill(Color32::from_rgb(16, 21, 25))
        .inner_margin(Margin {
            left: 20,
            right: 20,
            top: 20,
            bottom: bottom_padding + tabs_height,
        });

    CentralPanel::default()
        .frame(custom_frame)
        .show(ctx, |ui| {
        
        ui.set_width(ui.available_width() + 20.0);

        Frame::NONE
            .inner_margin(egui::Margin::same(10))
            .fill(Color32::from_rgb(27, 31, 35))
            .corner_radius(CornerRadius::same(20))
            .show(ui, |ui| {
                

                tools(state, ui_state, files, ui, ctx);


                ui.add_space(10.0);


                let content_rect = ui.available_rect_before_wrap();
                let panel_top = content_rect.min.y;
                let clipped_painter = &ui.painter_at(content_rect);

                let row_height = 30.0;
                let total_rows = files.len();


                //Drag
                if state.row_view.is_dragging_files {
                    drag_files(state, files, clipped_painter, ctx, content_rect, row_height);
                }
                
                //Rubberband 
                if state.rubber_band.is_rubber_banding {
                    render_rubberband(state, files, clipped_painter, panel_top, content_rect, row_height);
                }


                if !ctx.memory(|m| m.focused().is_some()) {
                    ui.memory_mut(|m| m.request_focus(ui.id()));
                }

                //Creacion de carpetas nuevas
                new_ff_logic(state, ui);


                //Background
                background_response_logic(state, ui_state, files, ui, ctx, panel_top, total_rows, row_height, content_rect);

            
                //Disparador de sizer
                if let Some(sender) = state.sender().cloned() {
                    for i in state.row_view.first_visible..state.row_view.last_visible.min(files.len()) {
                        let file = &files[i];

                        if file.is_dir 
                            && !state.calculating_dir_sizes.contains(&file.full_path) 
                            && !state.calculated_dir_sizes.contains(&file.full_path) 
                        {
                            state.calculating_dir_sizes.insert(file.full_path.clone());
                            sender.send_sizer(SizerMessages::StartCal(file.full_path.clone())).ok();
                        }
                    }
                }


                //Scrollview
                render_scrollview(ctx, files, state, ui_state, ui, row_height, total_rows, content_rect);


                if state.row_view.is_dragging_files && !ctx.input(|i| i.pointer.any_down()) {
                    state.row_view.is_dragging_files = false;
                    state.row_view.drag_ghost_pos = None;
                    state.row_view.drop_target = None;
                    state.row_view.drop_invalid_target = None;
                }

                //Renderizado de archivos drag
                render_drag_files(state, files, clipped_painter, content_rect, row_height);
                



                //Isla y burbuja y las tabs
                render_island_bubble(state, ui_state, files, ctx, bottom_padding, tabs_height);



                //hotkeys
                hot_keys_logic(state, files, ui, total_rows);

        });
    });
}

