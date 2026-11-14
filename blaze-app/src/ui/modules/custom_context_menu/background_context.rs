use std::{cell::Cell, path::PathBuf, sync::Arc};

use egui::{Id, Ui};
use tracing::warn;

use crate::{
    core::{
        blaze_state::{BlazeCoreState, state_structs::NewItemType},
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::bus_structs::{FileOperation, SureTo, UiEvent},
    },
    ui::{custom_components::UiExt, icons_cache::icons, themes::theme_manager::with_theme},
};

use crate::ui::modules::custom_context_menu::context_state::ContextMenuState;

impl ContextMenuState {
    //Menús del fondo
    pub fn background_context_menu_in_trash(&mut self, ui: &mut Ui, state: &mut BlazeCoreState) {
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

        self.show_menu(ui, "custom_ctx_background_trash", |ui| {
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
                    &label,
                    hint,
                    icon,
                    enable,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui)>,
                );

                if let Some(0) = action.get() {
                    sender
                        .send(FileOperation::RestoreDeletedFiles { file_names })
                        .ok();
                    should_close = true;
                }
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
                    &label,
                    hint,
                    icon,
                    enable,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui)>,
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
            });
        });
    }

    pub fn background_context_menu(&mut self, ui: &mut Ui, state: &mut BlazeCoreState) {
        let i18n = with_configs(|c| c.get_i18n());
        let current_theme = with_theme(|f| f.current());
        let palette = current_theme.context_menu.palette(&current_theme.global);

        if !self.open {
            return;
        }

        let mut should_close = false;
        self.show_menu(ui, "custom_ctx_menu_background", |ui| {
            let w = ui.available_width();
            let search_id = Id::new("search_ctx_menu");
            let has_focus = ui.memory(|mem| mem.has_focus(search_id));

            let search_color = if has_focus {
                palette.border_focus
            } else {
                palette.border_component
            };

            let (icon_name, icon_bytes) = ("search", icons::ICON_SEARCH);

            let mut search = state.current_search_filter().to_string();

            let response = ui
                .blaze_input(&i18n.t("search.placeholder"), None, &mut search)
                .id(search_id)
                .hint_background_color(palette.accent_soft)
                .hint_stroke_color(palette.accent_text)
                .hint_text_color(palette.accent_text)
                .size((w, 30.0))
                .radius(20)
                .text_color(palette.fg_primary)
                .background_color(palette.bg_elevated)
                .stroke_color(search_color)
                .x_color(palette.accent_text)
                .icon(icon_name, icon_bytes)
                .icon_color(if has_focus {
                    palette.accent_text
                } else {
                    palette.icon_secondary
                })
                .stroke_width(current_theme.global.components_stroke_width)
                .show(|| {
                    state.clean_search();
                });

            if response.changed() {
                state.set_search(search);
                response.request_focus();
            }

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                let icon = ("tab-icon", icons::ICON_TAB_ICON);

                let label = i18n.t("bg_ctx_menu.new_tab");
                let hint = "Ctrl + N";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui)>,
                );

                if let Some(0) = action.get() {
                    state.create_tab();
                    should_close = true;
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                let enable = match state.clipboard.clipboard_has_files() {
                    Ok(has_files) => has_files,
                    Err(e) => {
                        warn!("Error en el clipboard: {}", e);
                        false
                    }
                };

                let icon = if enable {
                    ("clipboard", icons::ICON_CLIPBOARD)
                } else {
                    ("clipboard-disabled", icons::ICON_CLIPBOARD_DISABLE)
                };

                let label = i18n.t("file_ops.paste_here");
                let hint = "Ctrl + V";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    &label,
                    hint,
                    icon,
                    enable,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui)>,
                );

                if let Some(0) = action.get() {
                    let cwd = state.cwd();
                    state.paste(cwd);
                    should_close = true;
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                let icon = ("terminal", icons::ICON_TERMINAL);

                let label = i18n.t("bg_ctx_menu.open_terminal_here");
                let hint = "Alt + T";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui)>,
                );

                if let Some(0) = action.get() {
                    state.open_terminal_here();
                    should_close = true;
                }
            });
            ui.separator();

            ui.horizontal(|ui| {
                let icon = ("plus-folder", icons::ICON_PLUS_FOLDER);

                let label = i18n.t("bg_ctx_menu.new_folder");
                let hint = "Ctrl + Shfit + N";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui)>,
                );

                if let Some(0) = action.get() {
                    state.creating_new = Some(NewItemType::Folder);
                    state.new_item_buffer = label.to_string();
                    should_close = true;
                }
            });

            ui.horizontal(|ui| {
                let icon = ("plus-file", icons::ICON_PLUS_FILE);

                let label = i18n.t("bg_ctx_menu.new_file");
                let hint = "Ctrl + Shfit + F";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    &label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui)>,
                );

                if let Some(0) = action.get() {
                    state.creating_new = Some(NewItemType::File);
                    state.new_item_buffer = label.to_string();
                    should_close = true;
                }
            });
        });

        if should_close {
            self.close();
        }
    }
}
