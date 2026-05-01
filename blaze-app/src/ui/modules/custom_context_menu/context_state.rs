use std::{cell::Cell, path::PathBuf, sync::Arc};

use egui::{Align2, Area, Color32, CursorIcon, FontId, Frame, Id, Key, Order, Pos2, Rect, Response, Sense, Stroke, TextEdit, Ui, UiBuilder, pos2, vec2};
use tracing::{info, warn};

use crate::{core::{blaze_state::{BlazeCoreState, NewItemType}, configs::config_state::with_configs, files::blaze_motor::motor_structs::FileEntry, runtime::{bus_structs::{FileOperation, SureTo, UiEvent}, event_bus::{Dispatcher, with_event_bus}}, system::{clipboard::TOKIO_RUNTIME, disk_reader::disk::Disk}}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons, image_preview::image_preview::ImagePreviewState}};


#[derive(Default, PartialEq)]
pub enum ContextMenuKind {
    #[default]
    None,
    FileNormal,
    FileTrash,
    BackgroundNormal,
    BackgroundTrash,
    DrivesPanel,
}

#[derive(Default)]
pub struct ContextMenuState {
    pub open: bool,
    pub position: Pos2,
    pub kind: ContextMenuKind,
    pub target_file: Option<Arc<FileEntry>>,
    pub target_sender: Option<Dispatcher>,
    pub target_drive: Option<Disk>,
    just_opened: bool,
}


impl ContextMenuState {
    pub fn new() -> Self {
        Self { 
            open: false, 
            position: pos2(0.0, 0.0), 
            target_file: None,
            target_sender: None,
            kind: ContextMenuKind::None,
            target_drive: None,
            just_opened: false,
        }
    }


    pub fn handle_response(&mut self, response: &Response) {
        if response.secondary_clicked() {
            self.open = true;
            self.just_opened = true;
            self.position = response.ctx.input(|i| i.pointer.latest_pos()).unwrap_or_default();
            self.clear_targets();
        }

        if self.open {
            let text_focused = response.ctx.memory(|m| m.focused().is_some());
            if !text_focused {
                let clicked_elsewhere = response.ctx.input(|i| i.pointer.primary_clicked()) || response.drag_started();
                if clicked_elsewhere {
                    self.open = false;
                }
            }
        }
    }
    
    fn handle_internal_response(&mut self, ui: &mut Ui, menu_rect: Rect) {
        let text_edit_focused = ui.ctx().memory(|m| m.focused().is_some());

        if text_edit_focused {
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                self.open = false;
            }
            return;
        }

        let clicked_outside = ui.input(|i| {
            (i.pointer.primary_clicked() || i.pointer.secondary_clicked()) &&
            i.pointer.latest_pos()
                .map(|p| !menu_rect.contains(p))
                .unwrap_or(true)
        });

        if clicked_outside || ui.input(|i| i.key_pressed(Key::Escape)) {
            self.open = false;
        }
    }

    fn clear_targets(&mut self) {
        self.kind = ContextMenuKind::None;
        self.target_file = None;
        self.target_sender = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.clear_targets();
    }


    fn show_menu<F>(&mut self, ui: &mut Ui, menu_id: &str, mut callback: F) 
        where F: FnMut(&mut Ui) {
            let screen = ui.content_rect();
            let pos = self.position;

            self.handle_internal_response(ui, screen);

            let area_response = egui::Area::new(Id::new(menu_id))
                .order(Order::Foreground)
                .fixed_pos(pos)
                .constrain_to(screen)
                .show(ui.ctx(), |ui| {
                    Frame::new()
                        .fill(Color32::from_rgb(40, 40, 50))
                        .corner_radius(12.0)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(46, 5, 63)))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_min_width(190.0);
                            ui.set_max_width(190.0);
                            callback(ui);
                        });
                });

            let menu_rect = area_response.response.rect;
            let clicked_outside = ui.input(|i| {
                i.pointer.primary_clicked() &&
                i.pointer.latest_pos()
                    .map(|p| !menu_rect.contains(p))
                    .unwrap_or(true)
            });

            if clicked_outside || ui.input(|i| i.key_pressed(Key::Escape)) {
                self.open = false;
            }
        }


    fn render_small_context<I>(ui: &mut Ui, state: &mut BlazeUiState, rect: Rect, hint_pos: Pos2, popup_id: Id, mut callback: I)
        where I: FnMut(&mut Ui, &mut BlazeUiState) {
            let popup_size = vec2(120.0, 80.0);
            let screen_rect = ui.content_rect();
            
            let anchor_x = hint_pos.x;
            let goes_right = anchor_x + popup_size.x + 15.0 < screen_rect.right();
            
            let popup_pos = if goes_right {
                pos2(anchor_x + 15.0, rect.top())
            } else {
                pos2(anchor_x - popup_size.x - 15.0, rect.top())
            };
            
            let popup_rect = Rect::from_min_size(popup_pos, popup_size);

            Area::new(popup_id.with("area"))
                .order(Order::TOP)
                .fixed_pos(popup_pos)
                .show(ui.ctx(), |ui| {
                    Frame::popup(ui.style())
                        .fill(Color32::from_rgb(45, 45, 55))
                        .show(ui, |ui| {
                            ui.set_min_size(popup_size);
                            ui.set_max_size(popup_size);
                            callback(ui, state);
                        });
                });

            if ui.input(|i| i.pointer.any_click()) {
                let mouse_pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(pos) = mouse_pos {
                    if !popup_rect.contains(pos) && !rect.contains(pos) {
                        ui.memory_mut(|m| m.data.insert_temp(popup_id, false));
                    }
                }
            }
        }

    fn render_context_button<J, I>(ui: &mut Ui, ui_state: &mut BlazeUiState, label: &str, hint: &str, icon: (&str, &[u8]), enabled: bool, mut callback_one: J, callback_two: Option<I>)
        where J: FnMut(),
            I: FnMut(&mut Ui, &mut BlazeUiState)
        {
            let (rect, response) = ui.allocate_exact_size(
                vec2(ui.available_width() - 2.0, 30.0),
                Sense::click_and_drag()
            );

            let h_padding = 4.0;
            let paint_rect = rect.shrink2(vec2(h_padding, 0.0));


            let text_color = if enabled {
                ui.visuals().text_color()
            } else {
                ui.visuals().weak_text_color()
            };

            let popup_id = response.id.with("popup");
            let mut is_open = ui.memory_mut(|m| m.data.get_temp::<bool>(popup_id).unwrap_or(false));


            let bg_color = if response.hovered() && enabled {
                ui.set_cursor_icon(CursorIcon::PointingHand);
                Color32::from_rgba_unmultiplied(100, 100, 255, 60)
            } else {
                Color32::TRANSPARENT
            };

            ui.painter().rect_filled(paint_rect, 12.0, bg_color);


            let icon_size = vec2(16.0, 16.0);
            let padding = 8.0;

            let icon_pos = rect.left_top() + vec2(padding, (rect.height() - icon_size.x) / 2.0);
            let icon_rect = Rect::from_min_size(icon_pos, icon_size);
            let icon = ui_state.icon_cache.get_or_load(ui, icon.0, icon.1, Color32::GRAY);
            
            let painter = ui.painter();

            painter.image(
                icon.id(),
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), 
                pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            let text_pos = pos2(
                icon_rect.right() + 6.0,
                rect.center().y
            );

            
            ui.painter().text(
                text_pos,
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(14.0),
                text_color,
            );

            let hint_galley = ui.painter().layout_no_wrap(
                hint.to_string(),
                FontId::proportional(10.0),
                ui.visuals().weak_text_color(),
            );

            let hint_width = hint_galley.size().x;

            let hint_pos = pos2(
                rect.right() - padding - hint_width,
                rect.center().y - hint_galley.size().y / 2.0,
            );

            ui.painter().galley(hint_pos, hint_galley, ui.visuals().weak_text_color());

            if enabled {
                if response.clicked() {
                    callback_one();
                    is_open = false;
                }

                if response.secondary_clicked() {
                    is_open = !is_open;
                }
            }

            ui.memory_mut(|m| m.data.insert_temp(popup_id, is_open));


            if let Some(mut callback) = callback_two {
                if is_open {
                    Self::render_small_context(ui, ui_state, rect, hint_pos, popup_id, |ui: &mut Ui, state: &mut BlazeUiState|{
                        callback(ui, state)
                    })
                }
            }
        }

    
    //Menú de los discos
    pub fn render_drives_context(&mut self, ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
        if !self.open { return; }
        let Some(drive) = self.target_drive.clone() else {return;};

        let mut should_close = false;
        let is_removable = drive.is_removable;
        let is_mounted = drive.mountpoint.is_none();
        let is_system = drive.is_system;

        self.show_menu(ui, "custom_ctx_drives", |ui| {
            ui.horizontal(|ui|{
                
                let icon = if is_mounted {
                    ("mount", icons::ICON_OPEN_ARROW_UP)
                } else {
                    ("folder-open", icons::ICON_FOLDER_OPEN)
                };

                let label = if is_mounted {"Montar"} else {"Abrir"};
                let hint = "";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        if is_mounted {
                            let manager = state.motor.borrow_mut().disk_manager.clone();
                            TOKIO_RUNTIME.block_on(async {
                                let mut manager = manager.lock().await;
                                manager.mount_disk(&drive).await.ok();
                            });
                        } else {
                            let path_string = drive.mountpoint.clone().unwrap_or_default();
                            let path = PathBuf::from(path_string);
                            state.navigate_to(path);
                        }
                        should_close = true;
                    }
                    _ => {}
                }
            });

            
            if is_mounted && is_removable {
                ui.horizontal(|ui|{
                    let icon = ("eject", icons::ICON_EJECT_FILLED);
                    let label = "Expulsar";
                    let hint = "";
                    
                    let action: Cell<Option<u8>> = Cell::new(None);
                    
                    Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>);

                    match action.get() {
                        Some(0) => {
                            let manager = state.motor.borrow_mut().disk_manager.clone();
                            TOKIO_RUNTIME.block_on(async {
                                let mut manager = manager.lock().await;
                                manager.eject_disk(&drive).await.ok();
                            });
                            should_close = true;
                        }
                        _ => {}
                    }
                });
            }


            if !is_mounted {
                ui.horizontal(|ui|{
                    let icon = ("unmount", icons::ICON_OPEN_ARROW_DOWN);
                    let label = "Desmontar";
                    let hint = "";
                    
                    let action: Cell<Option<u8>> = Cell::new(None);
                    
                    Self::render_context_button(ui, ui_state, label, hint, icon, !is_system,|| {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>);

                    match action.get() {
                        Some(0) => {
                            //protección básica
                            if drive.is_system {
                                warn!("¡No intentes desmontar raiz!"); 
                                should_close = true; 
                                return;
                            }

                            let manager = state.motor.borrow_mut().disk_manager.clone();
                            TOKIO_RUNTIME.block_on(async {
                                let mut manager = manager.lock().await;
                                manager.unmount_disk(&drive).await.ok();
                            });
                            should_close = true;
                        }
                        _ => {}
                    }
                });
            }
        });

        if should_close {
            self.close();
        }
    }






    //Menús del fondo
    pub fn background_context_menu_in_trash(&mut self, ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>) {
        if !self.open { return; }
        let Some(sender) = self.target_sender.clone() else { return; };

        let mut should_close = false;

        self.show_menu(ui, "custom_ctx_background_trash", |ui| {
            let sources = state.get_selected_paths(files);
            let file_names: Vec<String> = sources.iter()
                .map(|p| PathBuf::from(p)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned())
                .collect();


            ui.horizontal(|ui|{
                let icon = ("restore", icons::ICON_RESTORE);

                let label = "Restaurar";
                let hint = "";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        sender.send(
                            FileOperation::RestoreDeletedFiles {
                                file_names
                            }
                        ).ok();
                        should_close = true;
                    }
                    _ => {}
                }
            });
            ui.separator();


            ui.horizontal(|ui|{
                let icon = ("trash-forever", icons::ICON_TRASH);
                let label = "Eliminar";
                let hint = "Supr";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        sender.send(
                            UiEvent::SureTo(
                                SureTo::SureToDelete {
                                    files: sources, 
                                    tab_id: sender.tab_id
                                }
                            )
                        ).ok();
                        should_close = true;
                    }
                    _ => {}
                }
            });
        });

    }


    pub fn background_context_menu(&mut self, ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
        if !self.open { return; }

        let mut should_close = false;
        let mut just_opened = Cell::new(self.just_opened);
        self.show_menu(ui, "custom_ctx_menu_background", |ui| {
            
            ui.horizontal(|ui| {
                let icon = ("search", icons::ICON_ZOOM);
                let icon_size = vec2(16.0, 16.0);
                let padding = 8.0;

                let (rect, _) = ui.allocate_exact_size(
                    vec2(ui.available_width() - 2.0, 30.0),
                    Sense::hover()
                );

                let paint_rect = rect.shrink2(vec2(4.0, 0.0));
                ui.painter().rect_filled(paint_rect, 12.0, Color32::from_rgb(30, 30, 40));

                let icon_pos = rect.left_top() + vec2(padding, (rect.height() - icon_size.y) / 2.0);
                let icon_rect = Rect::from_min_size(icon_pos, icon_size);
                let icon = ui_state.icon_cache.get_or_load(ui, icon.0, icon.1, Color32::GRAY);

                ui.painter().image(
                    icon.id(),
                    icon_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                let text_rect = Rect::from_min_max(
                    pos2(icon_rect.right() + 6.0, rect.min.y + 4.0),
                    pos2(rect.max.x - padding, rect.max.y - 4.0),
                );

                let mut child_ui = ui.new_child(UiBuilder::new().max_rect(text_rect));

                let text_edit = TextEdit::singleline(&mut state.search_filter)
                    .hint_text("Buscar...")
                    .font(FontId::proportional(13.0))
                    .desired_width(text_rect.width());

                let te_response = child_ui.add(text_edit);

                if te_response.clicked() {
                    te_response.request_focus();
                    just_opened = true.into();
                }
            });

            ui.horizontal(|ui|{
                let icon = ("tab-icon", icons::ICON_TAB_ICON);

                let label = "Nueva pestaña";
                let hint = "Ctrl + N";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.create_tab();
                        should_close = true;
                    }
                    _ => {}
                }
            });

            ui.separator();


            ui.horizontal(|ui|{
                let enable = state.clipboard.clipboard_has_files();

                let icon = if enable {
                    ("clipboard", icons::ICON_CLIPBOARD)
                } else {
                    ("clipboard-disabled", icons::ICON_CLIPBOARD_DISABLE)
                };

                let label = "Pegar aquí";
                let hint = "Ctrl + V";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, enable,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        let cwd = state.cwd.clone();
                        state.paste(cwd);
                        should_close = true;
                    }
                    _ => {}
                }
            });


            ui.separator();

            ui.horizontal(|ui|{
                let icon = ("terminal", icons::ICON_TERMINAL);

                let label = "Abrir terminal aqui";
                let hint = "Alt + T";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.open_terminal_here();
                        should_close = true;
                    }
                    _ => {}
                }
            });
            ui.separator();

            ui.horizontal(|ui|{
                let icon = ("plus-folder", icons::ICON_PLUS_FOLDER);

                let label = "Nueva carpeta";
                let hint = "Ctrl + Shfit + N";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.creating_new = Some(NewItemType::Folder);
                        state.new_item_buffer = "nueva carpeta".to_string();
                        should_close = true;
                    }
                    _ => {}
                }
            });


            ui.horizontal(|ui|{
                let icon = ("plus-file", icons::ICON_PLUS_FILE);

                let label = "Nuevo archivo";
                let hint = "Ctrl + Shfit + F";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.creating_new = Some(NewItemType::File);
                        state.new_item_buffer = "nuevo archivo".to_string();
                        should_close = true;
                    }
                    _ => {}
                }
            });
        });

        if should_close {
            self.close();
        }

        self.just_opened = just_opened.get();
    }




    ///Menus files
    pub fn file_context_menu_in_trash(&mut self, ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>) {
        if !self.open { return; }
        let Some(sender) = self.target_sender.clone() else { return; };

        let mut should_close = false;

        self.show_menu(ui, "custom_ctx_menu_files_trash", |ui| {

            let sources = state.get_selected_paths(files);
            let file_names: Vec<String> = sources.iter()
                .map(|p| PathBuf::from(p)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned())
                .collect();


            ui.horizontal(|ui|{
                let icon = ("restore", icons::ICON_RESTORE);

                let label = "Restaurar";
                let hint = "";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        sender.send(
                            FileOperation::RestoreDeletedFiles {
                                file_names
                            }
                        ).ok();
                        should_close = true;
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });

            ui.separator();

            ui.horizontal(|ui|{
                let icon = ("trash-forever", icons::ICON_TRASH);

                let label = "Eliminar";
                let hint = "Supr";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        sender.send(
                            UiEvent::SureTo(
                                SureTo::SureToDelete { 
                                    files: sources, 
                                    tab_id: sender.tab_id,
                                }
                            )
                        ).ok();
                        should_close = true;
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });
        });

        if should_close {
            self.close();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.open = false;
        }

    }





    pub fn file_context_menu(&mut self, ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, files: &Vec<Arc<FileEntry>>) {
        if !self.open { return; }
        let (Some(file), Some(sender)) = (self.target_file.clone(), self.target_sender.clone()) else { return; };

        let mut should_close = false;

        self.show_menu(ui, "custom_ctx_menu_files", |ui| {

            if file.extension.is_image() {
                ui.horizontal(|ui|{
                    let icon = ("polaroid", icons::ICON_POLAROID);

                    let label = "Previsualizar";
                    let hint = "";
                    
                    let action: Cell<Option<u8>> = Cell::new(None);
                    
                    Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>);

                    match action.get() {
                        Some(0) => {

                            let all_images: Vec<PathBuf> = files.iter()
                                .filter(|f| f.extension.is_image())
                                .map(|f| f.full_path.clone())
                                .collect();

                            let pvw = ImagePreviewState::new(
                                file.full_path.clone(),
                                all_images
                            );

                            sender.send(
                                UiEvent::ShowImagePvw { pvw: Some(pvw) }
                            ).ok();

                            should_close = true;
                        }
                        _ => {}
                    }

                    //Añadirle hotkey
                });
                ui.separator();
            }


            ui.horizontal(|ui|{
                if !file.is_dir {
                    let icon = ("external-link",icons::ICON_EXTERNAL_LINK);
                    let label = "Abrir";
                    let hint = "Enter";
                    
                    let action: Cell<Option<u8>> = Cell::new(None);
                    
                    Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                        action.set(Some(0));
                    }, 
                    Some(|ui: &mut Ui, ui_state: &mut BlazeUiState| {

                        let icon = ("external-link",icons::ICON_EXTERNAL_LINK);
                        let label = "Abrir con...";
                        let hint = "";

                        Self::render_context_button(ui, ui_state, label, hint, icon, true, || {
                            info!("Se ejecuta");
                            action.set(Some(1));
                        }, None::<fn(&mut Ui, &mut BlazeUiState)>);
                        
                    }));

                    match action.get() {
                        Some(0) => {
                            if file.is_dir {
                                state.navigate_to(file.full_path.clone());
                            } else {
                                state.open_file(&file);
                            }
                            should_close = true;
                        }
                        Some(1) => {
                            state.open_file_with(&file);
                            should_close = true;
                        }
                        _ => {}
                    }

                } else {

                    let icon = ("folder-open", icons::ICON_FOLDER_OPEN);
                    let label = "Abrir";
                    let hint = "Enter";
                    
                    let action: Cell<Option<u8>> = Cell::new(None);
                    
                    Self::render_context_button(ui, ui_state, label, hint, icon , true,|| {
                        action.set(Some(0));
                    }, 
                    Some(|ui: &mut Ui, ui_state: &mut BlazeUiState| {

                        let icon = ("folder-open", icons::ICON_FOLDER_OPEN);
                        let label = "Abrir nueva pestaña";
                        let hint = "";

                        Self::render_context_button(ui, ui_state, label, hint, icon , true, || {
                            action.set(Some(1));
                        }, None::<fn(&mut Ui, &mut BlazeUiState)>);
                        
                    }));

                    match action.get() {
                        Some(0) => {
                            state.navigate_to(file.full_path.clone());
                            should_close = true;
                        }
                        Some(1) => {
                            state.motor.borrow_mut().add_tab(file.full_path.clone());
                            state.refresh();
                            should_close = true;
                        }
                        _ => {}
                    }
                }

            });

            //Pegar
            ui.horizontal(|ui|{
                let enable = state.clipboard.clipboard_has_files() && file.is_dir;

                let icon = if enable {
                    ("clipboard", icons::ICON_CLIPBOARD)
                } else {
                    ("clipboard-disable", icons::ICON_CLIPBOARD_DISABLE)
                };

                let label = "Pegar aquí";
                let hint = "";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, enable,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.paste(file.full_path.clone());
                        should_close = true;
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });


            // Copiar
            ui.horizontal(|ui|{
                let icon = ("copy", icons::ICON_COPY);

                let label = "Copiar";
                let hint = "Ctrl + C";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.copy(files);
                        should_close = true;
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });


            //Cortar
            ui.horizontal(|ui|{
                let icon = ("scissors", icons::ICON_SCISSORS);

                let label = "Cortar";
                let hint = "Ctrl + X";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.cut(files);
                        should_close = true;
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });


            if file.extension.is_archive() {
                ui.horizontal(|ui|{
                    let icon = ("extract", icons::ICON_EXTRACT);

                    let label = "Extraer aqui";
                    let hint = "";
                    
                    let action: Cell<Option<u8>> = Cell::new(None);
                    
                    Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>);

                    match action.get() {
                        Some(0) => {
                            let cwd = state.cwd.clone();
                            sender.send(
                                FileOperation::ExtractHere { 
                                    entry: file.clone(), 
                                    dest_dir: cwd,
                                }
                            ).ok();
                            should_close = true;
                        }
                        _ => {}
                    }
                });
            }

            ui.separator();


            let is_in_fav = with_configs(|c| {
                c.is_in_favorite(&file.full_path)
            });

            if file.is_dir {
                //Color de carpeta
                ui.horizontal(|ui|{
                    let icon = ("palette", icons::ICON_PALETTE);

                    let label = "Color de carpeta";
                    let hint = "";
                    
                    let action: Cell<Option<u8>> = Cell::new(None);
                    
                    Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>);

                    match action.get() {
                        Some(0) => {
                            let tab_id = state.active_id;
                            let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
                            let Some(folder_id) = file.unique_id else { return; };
                            dispatcher.send(
                                UiEvent::ShowFolderColorSelector { folder_id: folder_id }
                            ).ok();
                            
                            should_close = true;
                        }
                        _ => {}
                    }

                    //Añadirle hotkey
                });
            }
            

            //Agregar a favoritos
            ui.horizontal(|ui|{
                let icon = if !is_in_fav {
                    ("star-row", icons::ICON_STAR)
                } else {
                    ("star-disable", icons::ICON_STAR_DISABLE)
                };

                let label = if !is_in_fav {
                    "Agregar a favoritos"
                } else {
                    "Quitar a favoritos"
                };
                let hint = "";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        if !is_in_fav {
                            with_configs(|c| {
                                c.add_to_favorites(file.name.to_string(),file.full_path.clone(), file.is_dir)
                            });
                            should_close = true;
                        } else {
                        with_configs(|c| {
                            c.delete_from_favorites(file.name.to_string(),file.full_path.clone())
                        });
                        should_close = true;
                        }
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });
            

            ui.separator();


            //Mover a Papelera
            ui.horizontal(|ui|{
                let icon = ("trash", icons::ICON_TRASH);

                let label = "Mover a Papelera";
                let hint = "Supr";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.move_to_trash(files);
                        should_close = true;
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });


            //Mover a Papelera
            ui.horizontal(|ui|{
                let icon = ("edit", icons::ICON_EDIT);

                let label = "Renombrar";
                let hint = "";
                
                let action: Cell<Option<u8>> = Cell::new(None);
                
                Self::render_context_button(ui, ui_state, label, hint, icon, true,|| {
                    action.set(Some(0));
                },
                None::<fn(&mut Ui, &mut BlazeUiState)>);

                match action.get() {
                    Some(0) => {
                        state.renaming_file = Some(file.full_path.clone());
                        state.rename_buffer = file.name.to_ascii_lowercase();
                        should_close = true;
                    }
                    _ => {}
                }

                //Añadirle hotkey
            });
        });


        if should_close {
            self.close();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.open = false;
        }
    }
}