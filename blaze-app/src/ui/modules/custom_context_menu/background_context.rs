use std::{cell::Cell, path::PathBuf, sync::Arc};

use egui::{Color32, FontId, Id, Rect, Sense, Ui, UiBuilder, pos2, vec2};
use tracing::warn;

use crate::{
    core::{
        blaze_state::{BlazeCoreState, state_structs::NewItemType},
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::bus_structs::{FileOperation, SureTo, UiEvent},
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        custom_components::text_edit::BlazeTextEdit,
        icons_cache::icons,
        modules::utilities::ensure_min_lightness,
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};

use crate::ui::modules::custom_context_menu::context_state::ContextMenuState;

impl ContextMenuState {
    //Menús del fondo
    pub fn background_context_menu_in_trash(
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
            });
        });
    }

    pub fn background_context_menu(
        &mut self,
        ui: &mut Ui,
        state: &mut BlazeCoreState,
        ui_state: &mut BlazeUiState,
    ) {
        let i18n = with_configs(|c| c.get_i18n());
        let current_theme = with_theme(|f| f.current());

        if !self.open {
            return;
        }

        let mut should_close = false;
        let mut just_opened = Cell::new(self.just_opened);
        self.show_menu(ui, "custom_ctx_menu_background", |ui| {
            ui.horizontal(|ui| {
                let icon = ("search", icons::ICON_ZOOM);
                let icon_size = vec2(16.0, 16.0);
                let padding = 8.0;

                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width() - 2.0, 30.0), Sense::hover());

                let search_id = Id::new("search_ctx_menu");
                let has_focus = ui.memory(|mem| mem.has_focus(search_id));

                let search_color = if has_focus {
                    ensure_min_lightness(current_theme.components.input.border_focus.to_color())
                } else {
                    ensure_min_lightness(current_theme.components.input.border_idle.to_color())
                };

                let paint_rect = rect.shrink2(vec2(4.0, 0.0));
                ui.painter().rect_filled(
                    paint_rect,
                    12.0,
                    current_theme.components.input.bg.to_color(),
                );

                let icon_pos = rect.left_top() + vec2(padding, (rect.height() - icon_size.y) / 2.0);
                let icon_rect = Rect::from_min_size(icon_pos, icon_size);

                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let icon =
                    ui_state
                        .icon_cache
                        .get_or_load(ui, icon.0, icon.1, search_color, icon_size);

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                let text_rect = Rect::from_min_max(
                    pos2(rounded_rect.right() + 6.0, rect.min.y + 4.0),
                    pos2(rect.max.x - padding, rect.max.y - 4.0),
                );

                let mut child_ui = ui.new_child(UiBuilder::new().max_rect(text_rect));

                let text_edit = BlazeTextEdit::singleline(&mut state.search_filter)
                    .id(search_id)
                    .hint_text(i18n.t("search.placeholder"))
                    .font(FontId::proportional(13.0))
                    .desired_width(text_rect.width());

                let te_response = child_ui.add(text_edit);

                if te_response.clicked() {
                    te_response.request_focus();
                    just_opened = true.into();
                }
            });

            ui.horizontal(|ui| {
                let icon = ("tab-icon", icons::ICON_TAB_ICON);

                let label = i18n.t("bg_ctx_menu.new_tab");
                let hint = "Ctrl + N";

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
                    state.creating_new = Some(NewItemType::File);
                    state.new_item_buffer = label.to_string();
                    should_close = true;
                }
            });
        });

        if should_close {
            self.close();
        }

        self.just_opened = just_opened.get();
    }
}
