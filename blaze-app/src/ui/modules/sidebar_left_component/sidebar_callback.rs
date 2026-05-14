use egui::{Color32, CornerRadius, Frame, Margin, Panel, ScrollArea, Ui, scroll_area::ScrollSource};
use crate::{core::{blaze_state::BlazeCoreState, configs::config_state::with_configs, system::{clipboard::clipboard::TOKIO_RUNTIME, knowndirs::knowndirs_manager::KnownDirsManager, trash_manager::trash_manager::{TrashDestination, get_backend}}}, ui::{blaze_ui_state::BlazeUiState, modules::{custom_context_menu::context_state::ContextMenuKind, sidebar_left_component::sidebar_components::{render_drives_button, render_fav_buttons, render_header_text, render_local_buttons}}}};

pub fn sidebar_left_component(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    let custom_frame = Frame::NONE
        .fill(Color32::from_rgb(16, 21, 25))
        .inner_margin(Margin {
            left: 5,
            right: 0,
            top: 0,
            bottom: 10,
        });

    Panel::left("LeftSidePanel")
    .show_separator_line(false)
    .resizable(false)
    .frame(custom_frame)
    .show_inside(ui, |ui| {


        Frame::NONE
        .inner_margin(egui::Margin::same(10))
        .fill(Color32::from_rgb(27, 31, 35))
        .corner_radius(CornerRadius::same(20))
        .show(ui, |ui|{

            render_header_text("Locales", ui, ui_state);

            ui.add_space(10.0);

            let kdm = KnownDirsManager::get();
            let mut dirs = kdm.sidebar_dirs();


            let Some(trash) = get_backend().get_trash_files(&TrashDestination::Home).ok() else {return;};
            
            dirs.push(("Papelera", &trash));

            for (label, path) in dirs {
                if path.exists() {
                    render_local_buttons(label, path.to_owned(), state, ui, ui_state);
                }
            }


            ui.add_space(20.0);
            ui.separator();
        

            ui.add_space(10.0);
            render_header_text("Favoritos", ui, ui_state);
            ui.add_space(10.0);

            ScrollArea::vertical()
                .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
                .auto_shrink(true)
                .max_height(200.0)
                .animated(true)
                .show(ui, |ui|{
                    let favorites = with_configs(|c| {
                        c.configs.favorite_list.clone()
                    });

                    if favorites.is_empty() {
                        ui.label("No hay favoritos.");
                    }

                    for fav in favorites.clone() {
                        render_fav_buttons(ui, fav, state, ui_state);
                    }
            });



            ui.add_space(20.0);
            ui.separator();

            ui.add_space(10.0);
            render_header_text("Discos", ui, ui_state);
            ui.add_space(10.0);

            let manager = state.motor.borrow_mut().disk_manager.clone();
            let drives = TOKIO_RUNTIME.block_on(async {
                let manager = manager.lock().await;
                manager.get_partitions().await
            });

            for drive in drives {
                render_drives_button(ui, state, drive, ui_state);
            }

            let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);
            
            match ctx_menu.kind {
                ContextMenuKind::DrivesPanel => ctx_menu.render_drives_context(ui, state, ui_state),
                _ => {}
            }
            ui_state.context_menu_state = ctx_menu;

        });

    });

}