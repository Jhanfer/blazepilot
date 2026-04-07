// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.



use std::path::PathBuf;

use dirs::{desktop_dir, document_dir, download_dir, home_dir, picture_dir};
use egui::{Color32, CornerRadius, FontId, Frame, Margin, RichText, ScrollArea, Sense, SidePanel, Ui, pos2, scroll_area::ScrollSource, vec2};
use tracing::info;
use crate::core::{blaze_state::BlazeCoreState, configs::config_state::{FavoriteLinks, with_configs}, system::{clipboard::TOKIO_RUNTIME, disk_reader::disk::Disk}};


pub fn render_local_buttons(label:&str, path: PathBuf, state: &mut BlazeCoreState, ui: &mut Ui) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 30.0),
        Sense::click_and_drag()
    );
    let bg_color = if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        egui::Color32::from_rgba_unmultiplied(100, 100, 255, 60)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    };

    ui.painter().rect_filled(rect, 5.0, bg_color);


    if response.clicked() {
        state.navigate_to(path);
    }

    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::default(),
        ui.visuals().text_color(),
    );

}


pub fn render_fav_buttons(fav: FavoriteLinks, state: &mut BlazeCoreState, ui: &mut Ui) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 30.0),
        Sense::click()
    );

    let bg_color = if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        egui::Color32::from_rgba_unmultiplied(100, 100, 255, 60)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    };

    ui.painter()
        .rect_filled(
            rect,
            5.0,
            bg_color
        );
    
    if response.clicked() {
        if fav.is_dir {
            let path = fav.path.clone();
            state.navigate_to(path);
        } else {
            info!("Intentando abrir");
            state.open_file_by_path(fav.path.clone());
        }
    }

    response.context_menu(|ui|{
        if ui.button("Eliminar de favoritos").clicked() {
            with_configs(|c| {
                c.delete_from_favorites(fav.name.clone(), fav.path);
            });
        }
    });


    let galley = ui.fonts_mut(|f| {
        f.layout_no_wrap(
            fav.name.clone(),
            FontId::proportional(14.0),
            ui.visuals().text_color()
        )
    });

    ui.painter().with_clip_rect(rect)
        .galley(
            rect.left_center() + vec2(8.0, -galley.size().y / 2.0),
            galley,
            ui.visuals().text_color()
        );

}


pub fn render_drives_button(state: &mut BlazeCoreState, ui: &mut Ui, drive: Disk) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 30.0),
        Sense::click_and_drag()
    );

    if response.clicked() && !drive.mountpoint.is_none() {
        let path_string = drive.mountpoint.clone().unwrap_or_default();
        let path = PathBuf::from(path_string);
        state.motor.borrow_mut().active_tab().navigate_to(path);
        state.refresh();
    }

    response.context_menu(|ui|{
        if drive.mountpoint.is_none() {
            if ui.button("Montar").clicked() {
                TOKIO_RUNTIME.block_on(async {
                    let mut manager = state.disk_manager.lock().await;
                    manager.mount_disk(&drive).await.ok();
                });
            }
        } else {
            if ui.button("Abrir").clicked() {
                let path_string = drive.mountpoint.clone().unwrap_or_default();
                let path = PathBuf::from(path_string);
                state.motor.borrow_mut().active_tab().navigate_to(path);
                state.refresh();
            }
        }

        if drive.is_removable && drive.mountpoint.is_none() {
            ui.separator();
            if ui.button("Expulsar").clicked() {
                TOKIO_RUNTIME.block_on(async {
                    let mut manager = state.disk_manager.lock().await;
                    manager.eject_disk(&drive).await.ok();
                });
            }
        }

        if !drive.mountpoint.is_none() {
            ui.separator();

            if ui.button("Desmontar").clicked() {
                TOKIO_RUNTIME.block_on(async {
                    let mut manager = state.disk_manager.lock().await;
                    manager.unmount_disk(&drive).await.ok();
                });
            }
        }
    });


    let bg_color = if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        egui::Color32::from_rgba_unmultiplied(100, 100, 255, 60)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    };

    ui.painter().rect_filled(rect, 5.0, bg_color);


    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        drive.display_name,
        FontId::default(),
        ui.visuals().text_color(),
    );
}


pub fn render_header_text(label: &str,ui: &mut Ui) {
    let (rect, _resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), 24.0),
        Sense::hover(),
    );

    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(20.0),
        Color32::from_rgb(255, 255, 255),
    );
}


pub fn sidebar_left_component(ctx: &egui::Context, state: &mut BlazeCoreState) {
    let custom_frame = Frame::NONE
        .fill(Color32::from_rgb(16, 21, 25))
        .inner_margin(Margin::same(10));

    SidePanel::left("LeftSidePanel")
    .resizable(false)
    .frame(custom_frame)
    .show(ctx, |ui| {
        
        ui.add_space(10.0);

        Frame::NONE
        .inner_margin(egui::Margin::same(10))
        .fill(Color32::from_rgb(27, 31, 35))
        .corner_radius(CornerRadius::same(20))
        .show(ui, |ui|{

            render_header_text("Locales", ui);

            ui.add_space(10.0);

            let home = home_dir().unwrap();
            render_local_buttons("Home", home, state, ui);

            let desk = desktop_dir().unwrap();
            render_local_buttons("Escritorio", desk, state, ui);

            let donw = download_dir().unwrap();
            render_local_buttons("Descargas", donw, state, ui);

            let docs = document_dir().unwrap();
            render_local_buttons("Documentos", docs, state, ui);

            let imgs = picture_dir().unwrap();
            render_local_buttons("Imágenes", imgs, state, ui);

            let trsh = state.motor.borrow_mut().active_tab().get_trash_dir().unwrap();
            render_local_buttons("Papelera", trsh, state, ui);


            ui.add_space(20.0);
            ui.separator();
        

            ui.add_space(10.0);
            render_header_text("Favoritos", ui);
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
                        render_fav_buttons(fav, state, ui);
                    }
            });



            ui.add_space(20.0);
            ui.separator();

            ui.add_space(10.0);
            render_header_text("Discos", ui);
            ui.add_space(10.0);

            let drives = TOKIO_RUNTIME.block_on(async {
                let manager = state.disk_manager.lock().await;
                manager.get_partitions().await
            });   

            for drive in drives {
                render_drives_button(state, ui, drive);
            }

        });

    });

}