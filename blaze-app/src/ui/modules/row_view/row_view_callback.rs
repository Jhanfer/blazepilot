use std::sync::Arc;
use egui::{CentralPanel, Color32, CornerRadius, Frame, Key, Margin, Rect, TextEdit, Ui};
use tracing::{error};
use crate::{core::{blaze_state::{BlazeCoreState, NewItemType}, files::blaze_motor::motor_structs::FileEntry, system::{extended_info::extended_info_manager::ExtendedInfoMessages, sizer_manager::sizer_manager::SizerMessages}}, ui::{blaze_ui_state::BlazeUiState, icons_cache::thumbnails::thumbnails_manager::ThumbnailMessages, modules::{custom_context_menu::context_state::ContextMenuKind, row_view::{drag_drop_logic::drag_files, hot_keys::hot_keys_logic, island_n_bubble::render_island_bubble, new_scroll_view::new_render_scrollview, render_drag::render_drag_files, rubber_band_logic::render_rubberband, tools_view::tools}}}};


fn new_ff_logic(state: &mut BlazeCoreState, ui: &mut Ui) {
    if let Some(item_type) = state.creating_new.clone() {
        ui.horizontal(|ui|{
            let response = ui.add(TextEdit::singleline(&mut state.new_item_buffer));

            if !state.focus_requested {
                response.request_focus();
                state.focus_requested = true;
            }

            if ui.input(|i| i.key_pressed(Key::Enter)) && !state.new_item_buffer.trim().is_empty() {
                let cwd = state.cwd.clone();

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

fn background_response_logic(state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>, ui: &mut Ui, panel_top: f32, total_rows: usize, row_height: f32, content_rect: Rect) {
    let bg_id = ui.id().with("background_interact");
    let bg_response = ui.interact(ui.available_rect_before_wrap(), bg_id, egui::Sense::click_and_drag());

    let dragged_by  = bg_response.drag_started_by(egui::PointerButton::Primary) || bg_response.drag_started_by(egui::PointerButton::Secondary);
    
    if dragged_by {
        if let Some(orgin) = ui.input(|i| i.pointer.press_origin()) {
            state.rubber_band.rubber_band_start_content_y = orgin.y - panel_top + state.scroll_offset;
            state.rubber_band.rubber_band_start = Some(orgin);
        }
        state.rubber_band.is_rubber_banding = true;
    }

    if bg_response.dragged() {
        state.rubber_band.rubber_band_current = ui.input(|i| i.pointer.interact_pos());

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



    let cwd = state.cwd.clone();
    let trash = state.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();

    if trash == cwd {
        let Some(sender) = state.sender().cloned() else {return;};
        if bg_response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(&bg_response);
            ui_state.context_menu_state.kind = ContextMenuKind::BackgroundTrash;
            ui_state.context_menu_state.target_sender = Some(sender);
        }
    } else {
        if bg_response.secondary_clicked() {
            state.deselect_all();
            state.resize_selection(files.len());
            ui_state.context_menu_state.handle_response(&bg_response);
            ui_state.context_menu_state.kind = ContextMenuKind::BackgroundNormal;
        }
    }

    let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);
    match ctx_menu.kind {
        ContextMenuKind::BackgroundNormal => ctx_menu.background_context_menu(ui, state, ui_state),
        ContextMenuKind::BackgroundTrash  => ctx_menu.background_context_menu_in_trash(ui, state, ui_state, files),
        _ => {},
    }
    ui_state.context_menu_state = ctx_menu;

    if bg_response.clicked() {
        state.deselect_all();
    }
}


pub fn render_row_view(ui: &mut Ui, files: &Vec<Arc<FileEntry>>, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
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
            left: 5,
            right: 20,
            top: 0,
            bottom: bottom_padding + tabs_height,
        });

    CentralPanel::default()
        .frame(custom_frame)
        .show_inside(ui, |ui| {
        
        ui.set_width(ui.available_width() + 20.0);

        Frame::NONE
            .inner_margin(egui::Margin::same(10))
            .fill(Color32::from_rgb(27, 31, 35))
            .corner_radius(CornerRadius::same(20))
            .show(ui, |ui| {
                

                tools(state, ui_state, files, ui);


                ui.add_space(10.0);


                let content_rect = ui.available_rect_before_wrap();
                let panel_top = content_rect.min.y;
                let clipped_painter = &ui.painter_at(content_rect);

                let row_height = 30.0;
                let total_rows = files.len();


                //Drag
                if state.row_view.is_dragging_files {
                    drag_files(ui, state, files, clipped_painter, content_rect, row_height);
                }
                
                //Rubberband 
                if state.rubber_band.is_rubber_banding {
                    render_rubberband(state, files, clipped_painter, panel_top, content_rect, row_height);
                }


                if !ui.memory(|m| m.focused().is_some()) {
                    ui.memory_mut(|m| m.request_focus(ui.id()));
                }

                //Creacion de carpetas nuevas
                new_ff_logic(state, ui);


                //Background
                background_response_logic(state, ui_state, files, ui, panel_top, total_rows, row_height, content_rect);

            
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


                //Disparador de Info extendida
                if let Some(sender) = state.sender().cloned() {
                    for i in state.row_view.first_visible..state.row_view.last_visible.min(files.len()) {
                        let file = &files[i];
                        if !state.calculating_extended_info.contains(&file.full_path) && !state.calculated_extended_info.contains(&file.full_path) {
                            state.calculating_extended_info.insert(file.full_path.clone());
                            sender.send_extended_info(ExtendedInfoMessages::StartScan(file.full_path.clone())).ok();
                        }
                    }
                }

                //disparador de thumbnails
                if let Some(sender) = state.sender().cloned() {
                    for i in state.row_view.first_visible..state.row_view.last_visible.min(files.len()) {
                        let file = &files[i];

                        let img_vid = file.extension.is_image() || file.extension.is_video();

                        if !ui_state.calculating_thumbnails.contains(&file.full_path) && !ui_state.calculated_thumbnails.contains(&file.full_path) && img_vid {
                            let sent = sender.send_thumbnails(
                                ThumbnailMessages::RequestThumb(file.full_path.clone())
                            ).is_ok();
                            if sent {
                                ui_state.calculating_thumbnails.insert(file.full_path.clone());
                            }
                        }
                    }
                }


                //Scrollview
                new_render_scrollview(ui, files, state, ui_state, row_height, total_rows, content_rect);


                if state.row_view.is_dragging_files && !ui.input(|i| i.pointer.any_down()) {
                    state.row_view.is_dragging_files = false;
                    state.row_view.drag_ghost_pos = None;
                    state.row_view.drop_target = None;
                    state.row_view.drop_invalid_target = None;
                }

                //Renderizado de archivos drag
                render_drag_files(state, files, clipped_painter, content_rect, row_height);
                



                //Isla y burbuja y las tabs
                render_island_bubble(ui, state, ui_state, files, bottom_padding, tabs_height);



                //hotkeys
                hot_keys_logic(state, ui_state, files, ui, total_rows);

        });
    });
}

