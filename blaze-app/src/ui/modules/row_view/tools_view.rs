use std::sync::Arc;
use egui::{Color32, Context, Rect, Sense, Ui, pos2};
use crate::{core::{blaze_state::{BlazeCoreState, NewItemType}, configs::config_state::with_configs, files::motor::FileEntry}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons}, utils::channel_pool::{SureTo, UiEvent}};

pub fn tools(state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>, ui: &mut Ui) {
    ui.horizontal(|ui|{
        ui.visuals_mut().button_frame = false;

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.visuals_mut().button_frame = false;


            let (icon_plus_fol, icon_bytes_plus_fol) = ("plus-folder", icons::ICON_PLUS_FOLDER);

            let icon_size = egui::vec2(16.0, 16.0);
            let (icon_rect, new_fol) = ui.allocate_exact_size(icon_size, Sense::click());
            
            let icon = ui_state.icon_cache.get_or_load(ui, icon_plus_fol, icon_bytes_plus_fol, Color32::GRAY);
            
            ui.painter().image(
                icon.id(),
                icon_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), 
                pos2(1.0, 1.0)),
                Color32::WHITE,
            );


            if new_fol.clicked() {
                state.creating_new = Some(NewItemType::Folder);
                state.new_item_buffer = "nueva carpeta".to_string(); 
            }


            let (icon_plus_file, icon_bytes_plus_file) = ("plus-file", icons::ICON_PLUS_FILE);

            let icon_size = egui::vec2(16.0, 16.0);
            let (icon_rect, new_file) = ui.allocate_exact_size(icon_size, Sense::click());
            
            let icon = ui_state.icon_cache.get_or_load(ui, icon_plus_file, icon_bytes_plus_file, Color32::GRAY);
            
            ui.painter().image(
                icon.id(),
                icon_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), 
                pos2(1.0, 1.0)),
                Color32::WHITE,
            );


            if new_file.clicked() {
                state.creating_new = Some(NewItemType::File);
                state.new_item_buffer = "nuevo archivo".to_string(); 
            }

            ui.separator();

            if new_fol.hovered() || new_file.hovered() {
                ui.set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        });




        let has_selection = state.selected_count(files.len()) > 0;
        let has_clipboard = state.clipboard.clipboard_has_files();

        let (icon_cut, icon_bytes_cut) = if has_selection {
            ("scissors", icons::ICON_SCISSORS)
        } else {
            ("scissors-disable", icons::ICON_SCISSORS_DISABLE)
        };

        let icon_size = egui::vec2(16.0, 16.0);
        let (icon_rect, cut_resp) = ui.allocate_exact_size(icon_size, Sense::click());
        
        let icon = ui_state.icon_cache.get_or_load(ui, icon_cut, icon_bytes_cut, Color32::GRAY);
        
        ui.painter().image(
            icon.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), 
            pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if cut_resp.clicked() {
            state.cut(files);
        }


        let (icon_copy, icon_bytes_copy) = if has_selection {
            ("copy", icons::ICON_COPY)
        } else {
            ("copy-disable", icons::ICON_COPY_DISABLE)
        };

        let icon_size = egui::vec2(16.0, 16.0);
        let (icon_rect, cop_resp) = ui.allocate_exact_size(icon_size, Sense::click());

        
        let icon = ui_state.icon_cache.get_or_load(ui, icon_copy, icon_bytes_copy, Color32::GRAY);
        
        ui.painter().image(
            icon.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), 
            pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if cop_resp.clicked() {
            state.copy(files);
        }


        let (icon_paste, icon_bytes_paste) = if has_clipboard {
            ("clipboard", icons::ICON_CLIPBOARD)
        } else {
            ("clipboard-disable", icons::ICON_CLIPBOARD_DISABLE)
        };

        let icon_size = egui::vec2(16.0, 16.0);
        let (icon_rect, pas_resp) = ui.allocate_exact_size(icon_size, Sense::click());

        
        let icon = ui_state.icon_cache.get_or_load(ui, icon_paste, icon_bytes_paste, Color32::GRAY);
        
        ui.painter().image(
            icon.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), 
            pos2(1.0, 1.0)),
            Color32::WHITE,
        );


        if pas_resp.clicked() {
            let cwd = state.cwd.clone();
            state.paste(cwd);
        }


        let (icon_trash, icon_bytes_trash) = if has_selection {
            ("trash", icons::ICON_TRASH)
        } else {
            ("trash-disable", icons::ICON_TRASH_DISABLED)
        };

        let icon_size = egui::vec2(16.0, 16.0);
        let (icon_rect, del_resp) = ui.allocate_exact_size(icon_size, Sense::click());

        
        let icon = ui_state.icon_cache.get_or_load(ui, icon_trash, icon_bytes_trash, Color32::GRAY);
        
        ui.painter().image(
            icon.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), 
            pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if del_resp.clicked() {
            let cwd = state.cwd.clone();
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

        ui.add_space(8.0);



        let (icon_name, icon_bytes) = if state.select_all_mode {
            ("deselect", icons::ICON_DESELECT)
        } else {
            ("select-all", icons::ICON_SELECTALL)
        };

        let icon_size = egui::vec2(16.0, 16.0);
        let (icon_rect, select_resp) = ui.allocate_exact_size(icon_size, Sense::click());
        

        let icon = ui_state.icon_cache.get_or_load(ui, icon_name, icon_bytes, Color32::GRAY);
        
        ui.painter().image(
            icon.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), 
            pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if select_resp.clicked() {
            state.toggle_select_all(files.len());
        }


        ui.separator();


        let (icon_refresh, icon_bytes_refresh) = ("refresh", icons::ICON_REFRESH);

        let icon_size = egui::vec2(16.0, 16.0);
        let (icon_rect, refresh_resp) = ui.allocate_exact_size(icon_size, Sense::click());
        
        let icon = ui_state.icon_cache.get_or_load(ui, icon_refresh, icon_bytes_refresh, Color32::GRAY);
        
        ui.painter().image(
            icon.id(),
            icon_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), 
            pos2(1.0, 1.0)),
            Color32::WHITE,
        );


        if refresh_resp.clicked() {
            state.refresh();
        }



        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.visuals_mut().button_frame = false;

            let is_hidden = with_configs(|c| {c.configs.show_hidden_files.clone()});


            let (icon_refresh, icon_bytes_refresh) = if is_hidden {
                ("eye", icons::ICON_EYE)
            } else {
                ("eye-closed", icons::ICON_EYE_CLOSED)
            };

            let icon_size = egui::vec2(16.0, 16.0);
            let (icon_rect, hidd_resp) = ui.allocate_exact_size(icon_size, Sense::click());
            
            let icon = ui_state.icon_cache.get_or_load(ui, icon_refresh, icon_bytes_refresh, Color32::GRAY);
            
            ui.painter().image(
                icon.id(),
                icon_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), 
                pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            if hidd_resp.clicked() {
                with_configs(|c| {
                    c.set_show_hidden_files(!is_hidden);
                });
                state.refresh();
            };

            ui.separator();

            if hidd_resp.hovered() {
                ui.set_cursor_icon(egui::CursorIcon::PointingHand);
            }

        });


        let show_hand = select_resp.hovered()
            || refresh_resp.hovered()

            || (has_selection && (del_resp.hovered() || cop_resp.hovered() || cut_resp.hovered()))

            || (has_clipboard && pas_resp.hovered());

        if show_hand {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

    });
}