use egui::{Color32, Key, Ui};
use std::{cell::Cell, path::PathBuf, sync::Arc};
use tracing::{info, warn};

use crate::{
    core::{
        blaze_state::BlazeCoreState,
        bootstrap::{
            configs::config_manager::with_configs,
            quick_access_manager::platform::structs::QuickLinks,
        },
        files::blaze_motor::motor_structs::FileEntry,
        runtime::{
            bus_structs::{FileOperation, QuickTagEvent, SureTo, UiEvent},
            event_bus::with_event_bus,
        },
    },
    ui::{
        blaze_ui_state::BlazeUiState, icons_cache::icons,
        image_preview::image_preview_handler::ImagePreviewState,
    },
};

use crate::ui::modules::custom_context_menu::context_state::ContextMenuState;

impl ContextMenuState {
    ///Menus files
    pub fn file_context_menu_in_trash(
        &mut self,
        ui: &mut Ui,
        state: &mut BlazeCoreState,
        ui_state: &mut BlazeUiState,
    ) {
        let i18n = with_configs(|c| c.get_i18n());

        if !self.open {
            return;
        }
        let (Some(sender), Some(source_path)) =
            (self.target_sender.clone(), self.source_path.clone())
        else {
            return;
        };

        let files: &[Arc<FileEntry>] = &state.get_files_for(&source_path);

        let mut should_close = false;

        self.show_menu(ui, "custom_ctx_menu_files_trash", |ui| {
            let sources = state.get_selected_paths(files);
            let file_names: Vec<String> = sources
                .iter()
                .map(|(_, p)| {
                    PathBuf::from(p.as_ref())
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                })
                .collect();

            ui.horizontal(|ui| {
                let enable = !file_names.is_empty();

                let icon = ("restore", icons::ICON_RESTORE);

                let label = i18n.t("file_ops.restore");
                let hint = "";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    enable,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    sender
                        .send(FileOperation::RestoreDeletedFiles { file_names })
                        .ok();
                    should_close = true;
                }

                //Añadirle hotkey
            });

            ui.separator();

            ui.horizontal(|ui| {
                let enable = !sources.is_empty();

                let icon = ("trash-forever", icons::ICON_TRASH);

                let label = i18n.t("file_ops.delete");
                let hint = "Supr";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    enable,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    sender
                        .send(UiEvent::SureTo(SureTo::SureToDelete {
                            files: sources,
                            tab_id: sender.tab_id,
                        }))
                        .ok();
                    should_close = true;
                }

                //Añadirle hotkey
            });
        });

        if should_close {
            self.close();
        }

        if ui.input(|i| i.key_pressed(Key::Escape)) {
            self.open = false;
        }
    }

    pub fn file_context_menu(
        &mut self,
        ui: &mut Ui,
        state: &mut BlazeCoreState,
        ui_state: &mut BlazeUiState,
    ) {
        let i18n = with_configs(|c| c.get_i18n());

        if !self.open {
            return;
        }
        let (Some(file), Some(sender), Some(source_path)) = (
            self.target_file.clone(),
            self.target_sender.clone(),
            self.source_path.clone(),
        ) else {
            return;
        };

        let files: &[Arc<FileEntry>] = &state.get_files_for(&source_path);

        let (col_index_opt, row_col_opt) = (self.target_col, self.target_row_col);

        let mut should_close = false;
        let is_media = file.extension.is_video() || file.extension.is_audio();

        self.show_menu(ui, "custom_ctx_menu_files", |ui| {
            if file.extension.is_image() {
                ui.horizontal(|ui| {
                    let icon = ("polaroid", icons::ICON_POLAROID);

                    let label = i18n.t("file_ctx_menu.preview");
                    let hint = "";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        &label,
                        hint,
                        icon,
                        true,
                        || {
                            action.set(Some(0));
                        },
                        None::<fn(&mut Ui, &mut BlazeUiState)>,
                    );

                    if let Some(0) = action.get() {
                        let all_images = files
                            .iter()
                            .filter(|f| f.extension.is_image())
                            .map(|f| f.full_path.to_path_buf())
                            .collect();

                        let pvw = ImagePreviewState::new(file.full_path.to_path_buf(), all_images);

                        sender
                            .send(UiEvent::ShowImagePvw {
                                pvw: Some(Box::new(pvw)),
                            })
                            .ok();

                        should_close = true;
                    }

                    //Añadirle hotkey
                });
                ui.separator();
            } else if is_media {
                ui.horizontal(|ui| {
                    let icon = if file.extension.is_video() {
                        ("video", icons::ICON_VIDEO)
                    } else {
                        ("music", icons::ICON_MUSIC)
                    };

                    let label = if file.extension.is_video() {
                        i18n.t("file_ctx_menu.preview_video")
                    } else {
                        i18n.t("file_ctx_menu.preview_audio")
                    };

                    let hint = "";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        &label,
                        hint,
                        icon,
                        true,
                        || {
                            action.set(Some(0));
                        },
                        None::<fn(&mut Ui, &mut BlazeUiState)>,
                    );

                    if let Some(0) = action.get() {
                        let media_path = file.full_path.clone();
                        let media_name = file.name.clone();
                        sender
                            .send(UiEvent::OpenMediaPlayer {
                                media_path,
                                media_name,
                                is_audio_only: file.extension.is_audio(),
                            })
                            .ok();
                        should_close = true;
                    }

                    //Añadirle hotkey
                });
                ui.separator();
            }

            ui.horizontal(|ui| {
                if !file.is_dir() {
                    let icon = ("external-link", icons::ICON_EXTERNAL_LINK);
                    let label = i18n.t("file_ctx_menu.open");
                    let hint = "Enter";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        &label,
                        hint,
                        icon,
                        true,
                        || {
                            action.set(Some(0));
                        },
                        Some(|ui: &mut Ui, ui_state: &mut BlazeUiState| {
                            let icon = ("external-link", icons::ICON_EXTERNAL_LINK);
                            let label = i18n.t("file_ctx_menu.open_with");
                            let hint = "";

                            Self::render_context_button(
                                ui,
                                ui_state,
                                &label,
                                hint,
                                icon,
                                true,
                                || {
                                    info!("Se ejecuta");
                                    action.set(Some(1));
                                },
                                None::<fn(&mut Ui, &mut BlazeUiState)>,
                            );
                        }),
                    );

                    match action.get() {
                        Some(0) => {
                            if state.miller_view.columns.is_empty() {
                                if file.is_dir() {
                                    state.navigate_to(file.full_path.to_owned());
                                    state.deselect_all();
                                    state.resize_selection(files.len());
                                } else {
                                    state.open_file(&file);
                                }
                            } else {
                                let (Some(col_index), Some(i)) = (col_index_opt, row_col_opt)
                                else {
                                    return;
                                };

                                state.miller_view.set_opened(col_index, i);

                                if file.is_dir() {
                                    state.navigate_to(file.full_path.clone());
                                }
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
                    let label = i18n.t("file_ctx_menu.open");
                    let hint = "Enter";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        &label,
                        hint,
                        icon,
                        true,
                        || {
                            action.set(Some(0));
                        },
                        Some(|ui: &mut Ui, ui_state: &mut BlazeUiState| {
                            let icon = ("folder-open", icons::ICON_FOLDER_OPEN);
                            let label = i18n.t("file_ctx_menu.open_in_new_tab");
                            let hint = "";

                            Self::render_context_button(
                                ui,
                                ui_state,
                                &label,
                                hint,
                                icon,
                                true,
                                || {
                                    action.set(Some(1));
                                },
                                None::<fn(&mut Ui, &mut BlazeUiState)>,
                            );
                        }),
                    );

                    match action.get() {
                        Some(0) => {
                            if state.miller_view.columns.is_empty() {
                                if file.is_dir() {
                                    state.navigate_to(file.full_path.to_owned());
                                    state.deselect_all();
                                    state.resize_selection(files.len());
                                } else {
                                    state.open_file(&file);
                                }
                            } else {
                                let (Some(col_index), Some(i)) = (col_index_opt, row_col_opt)
                                else {
                                    return;
                                };

                                state.miller_view.set_opened(col_index, i);

                                if file.is_dir() {
                                    state.navigate_to(file.full_path.clone());
                                }
                            }
                            should_close = true;
                        }
                        Some(1) => {
                            state.add_tab_from_file(&file.full_path);
                            should_close = true;
                        }
                        _ => {}
                    }
                }
            });

            //Pegar
            ui.horizontal(|ui| {
                let enable = match state.clipboard.clipboard_has_files() {
                    Ok(has_files) => has_files && file.is_dir(),
                    Err(e) => {
                        warn!("Error en el clipboard: {}", e);
                        false
                    }
                };

                let icon = if enable {
                    ("clipboard", icons::ICON_CLIPBOARD)
                } else {
                    ("clipboard-disable", icons::ICON_CLIPBOARD_DISABLE)
                };

                let label = i18n.t("file_ops.paste_here");
                let hint = "";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    enable,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    state.paste(file.full_path.to_owned());
                    should_close = true;
                }

                //Añadirle hotkey
            });

            // Copiar
            ui.horizontal(|ui| {
                let icon = ("copy", icons::ICON_COPY);

                let label = i18n.t("file_ops.copy");
                let hint = "Ctrl + C";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    state.copy(files);
                    should_close = true;
                }

                //Añadirle hotkey
            });

            //Cortar
            ui.horizontal(|ui| {
                let icon = ("scissors", icons::ICON_SCISSORS);

                let label = i18n.t("file_ops.cut");
                let hint = "Ctrl + X";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    state.cut(files);
                    should_close = true;
                }

                //Añadirle hotkey
            });

            if file.extension.is_archive() {
                ui.horizontal(|ui| {
                    let icon = ("extract", icons::ICON_EXTRACT);

                    let label = i18n.t("file_ops.extract_here");
                    let hint = "";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        &label,
                        hint,
                        icon,
                        true,
                        || {
                            action.set(Some(0));
                        },
                        None::<fn(&mut Ui, &mut BlazeUiState)>,
                    );

                    if let Some(0) = action.get() {
                        let cwd = state.cwd();
                        sender
                            .send(FileOperation::ExtractHere {
                                entry: file.clone(),
                                dest_dir: cwd,
                            })
                            .ok();
                        should_close = true;
                    }
                });
            }

            ui.separator();

            if file.is_dir() {
                //Color de carpeta
                ui.horizontal(|ui| {
                    let icon = ("palette", icons::ICON_PALETTE);

                    let label = i18n.t("file_ops.folder_color");
                    let hint = "";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        &label,
                        hint,
                        icon,
                        true,
                        || {
                            action.set(Some(0));
                        },
                        None::<fn(&mut Ui, &mut BlazeUiState)>,
                    );

                    if let Some(0) = action.get() {
                        let tab_id = state.active_id();
                        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
                        let Some(folder_id) = file.unique_id else {
                            return;
                        };
                        dispatcher
                            .send(UiEvent::ShowFolderColorSelector { folder_id })
                            .ok();

                        should_close = true;
                    }

                    //Añadirle hotkey
                });
            }

            //Agregar a Tags
            ui.horizontal(|ui| {
                let icon = ("star-row", icons::ICON_STAR);

                let label = i18n.t("file_ctx_menu.add_to_tag");
                let hint = "";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    let qick = QuickLinks::new(file.name.clone(), Color32::GRAY)
                        .with_path(file.full_path.clone())
                        .with_kind(file.kind.clone())
                        .with_is_dir(file.is_dir());

                    sender
                        .send(UiEvent::QuickTagEvent(QuickTagEvent::AddQuickLinkToTag {
                            quicks: vec![qick],
                        }))
                        .ok();

                    should_close = true;
                }

                //Añadirle hotkey
            });

            ui.separator();

            //Mover a Papelera
            ui.horizontal(|ui| {
                let icon = ("trash", icons::ICON_TRASH);

                let label = i18n.t("file_ops.move_to_trash");
                let hint = "Supr";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    let items = files
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| state.is_selected(*index))
                        .map(|(_, f)| (f.name.clone(), f.full_path.to_owned()))
                        .collect();
                    state.move_to_trash(items);
                    should_close = true;
                }

                //Añadirle hotkey
            });

            //Mover a Papelera
            ui.horizontal(|ui| {
                let icon = ("edit", icons::ICON_EDIT);

                let label = i18n.t("file_ops.rename");
                let hint = "";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    state.renaming_file = Some(file.full_path.to_path_buf());
                    state.rename_buffer = file.name.to_string();
                    should_close = true;
                }

                //Añadirle hotkey
            });
        });

        if should_close {
            self.close();
        }

        if ui.input(|i| i.key_pressed(Key::Escape)) {
            self.open = false;
        }
    }
}
