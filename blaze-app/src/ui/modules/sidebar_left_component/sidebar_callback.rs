use crate::{
    core::{
        blaze_state::BlazeCoreState,
        bootstrap::configs::config_manager::with_configs,
        system::{
            clipboard::global_clipboard::TOKIO_RUNTIME,
            knowndirs::knowndirs_manager::KnownDirsManager,
            trash_manager::manager::{get_backend, TrashDestination},
        },
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        modules::{
            custom_context_menu::context_state::ContextMenuKind,
            sidebar_left_component::sidebar_components::{
                render_drives_button, render_header_text, render_local_buttons,
            },
        },
        themes::colors::*,
    },
};
use egui::{
    scroll_area::ScrollBarVisibility, CornerRadius, Frame, Margin, Panel, ScrollArea, Stroke, Ui,
};

pub fn sidebar_left_component(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
) {
    let i18n = with_configs(|c| c.get_i18n());

    let custom_frame = Frame::NONE.fill(COLOR_BG_MAIN).inner_margin(Margin {
        left: 15,
        right: 0,
        top: 0,
        bottom: 10,
    });

    Panel::left("LeftSidePanel")
        .show_separator_line(false)
        .resizable(false)
        .frame(custom_frame)
        .show_inside(ui, |ui| {
            ui.set_width(200.0);

            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(COLOR_BG_PANEL)
                .corner_radius(CornerRadius::same(20))
                .stroke(Stroke {
                    width: 0.5,
                    color: COLOR_ACCENT_GLOW,
                })
                .show(ui, |ui| {
                    ui.set_width(200.0);
                    ui.set_height(ui.available_height());

                    ScrollArea::vertical()
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            render_header_text(
                                "locals",
                                &i18n.t("left_sidebar.locals"),
                                ui,
                                ui_state,
                            );

                            ui.add_space(10.0);

                            let kdm = KnownDirsManager::get();
                            let mut dirs = kdm.sidebar_dirs();

                            let Some(trash) =
                                get_backend().get_trash_files(&TrashDestination::Home).ok()
                            else {
                                return;
                            };

                            dirs.push(("trash", i18n.t("left_sidebar.trash"), &trash));

                            for (key, label, path) in dirs {
                                if path.exists() {
                                    render_local_buttons(
                                        key,
                                        &label,
                                        path.to_owned(),
                                        state,
                                        ui,
                                        ui_state,
                                    );
                                    ui.add_space(2.0);
                                }
                            }

                            ui.add_space(20.0);
                            ui.separator();

                            ui.add_space(10.0);
                            render_header_text(
                                "disks",
                                &i18n.t("left_sidebar.disks"),
                                ui,
                                ui_state,
                            );
                            ui.add_space(10.0);

                            let manager = state.motor.borrow_mut().disk_manager.clone();
                            let drives = TOKIO_RUNTIME.block_on(async {
                                let manager = manager.lock().await;
                                manager.get_partitions().await
                            });

                            ui.vertical(|ui| {
                                for drive in drives {
                                    render_drives_button(ui, state, drive, ui_state);
                                    ui.add_space(2.0);
                                }
                            });

                            let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);

                            if ctx_menu.kind == ContextMenuKind::DrivesPanel {
                                ctx_menu.render_drives_context(ui, state, ui_state)
                            }

                            ui_state.context_menu_state = ctx_menu;
                        });
                });
        });
}
