use crate::ui::modules::custom_context_menu::context_state::ContextMenuState;

use std::{cell::Cell, path::PathBuf};

use egui::Ui;
use tracing::warn;

use crate::{
    core::{blaze_state::BlazeCoreState, system::clipboard::global_clipboard::TOKIO_RUNTIME},
    ui::{blaze_ui_state::BlazeUiState, icons_cache::icons},
};

impl ContextMenuState {
    //Menú de los discos
    pub fn render_drives_context(
        &mut self,
        ui: &mut Ui,
        state: &mut BlazeCoreState,
        ui_state: &mut BlazeUiState,
    ) {
        if !self.open {
            return;
        }
        let Some(drive) = self.target_drive.clone() else {
            return;
        };

        let mut should_close = false;
        let is_removable = drive.is_removable;
        let is_mounted = drive.mountpoint.is_none();
        let is_system = drive.is_system;

        self.show_menu(ui, "custom_ctx_drives", |ui| {
            ui.horizontal(|ui| {
                let icon = if is_mounted {
                    ("mount", icons::ICON_OPEN_ARROW_UP)
                } else {
                    ("folder-open", icons::ICON_FOLDER_OPEN)
                };

                let label = if is_mounted { "Montar" } else { "Abrir" };
                let hint = "";

                let action: Cell<Option<u8>> = Cell::new(None);

                Self::render_context_button(
                    ui,
                    ui_state,
                    label,
                    hint,
                    icon,
                    true,
                    || {
                        action.set(Some(0));
                    },
                    None::<fn(&mut Ui, &mut BlazeUiState)>,
                );

                if let Some(0) = action.get() {
                    if is_mounted {
                        let manager = state.motor.borrow_mut().disk_manager.clone();
                        TOKIO_RUNTIME.block_on(async {
                            let mut manager = manager.lock().await;
                            manager.mount_disk(&drive).await.ok();
                        });
                    } else {
                        let path_string = drive.mountpoint.clone().unwrap_or_default();
                        let path = PathBuf::from(path_string).into();

                        if state.miller_view.columns.is_empty() {
                            state.navigate_to(path);
                        } else {
                            state.navigate_to(path.clone());
                        }
                    }
                    should_close = true;
                }
            });

            if is_mounted && is_removable {
                ui.horizontal(|ui| {
                    let icon = ("eject", icons::ICON_EJECT_FILLED);
                    let label = "Expulsar";
                    let hint = "";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        label,
                        hint,
                        icon,
                        true,
                        || {
                            action.set(Some(0));
                        },
                        None::<fn(&mut Ui, &mut BlazeUiState)>,
                    );

                    if let Some(0) = action.get() {
                        let manager = state.motor.borrow_mut().disk_manager.clone();
                        TOKIO_RUNTIME.block_on(async {
                            let mut manager = manager.lock().await;
                            manager.eject_disk(&drive).await.ok();
                        });
                        should_close = true;
                    }
                });
            }

            if !is_mounted {
                ui.horizontal(|ui| {
                    let icon = ("unmount", icons::ICON_OPEN_ARROW_DOWN);
                    let label = "Desmontar";
                    let hint = "";

                    let action: Cell<Option<u8>> = Cell::new(None);

                    Self::render_context_button(
                        ui,
                        ui_state,
                        label,
                        hint,
                        icon,
                        !is_system,
                        || {
                            action.set(Some(0));
                        },
                        None::<fn(&mut Ui, &mut BlazeUiState)>,
                    );

                    if let Some(0) = action.get() {
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
                });
            }
        });

        if should_close {
            self.close();
        }
    }
}
