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
use egui::{Color32, CornerRadius, FontId, Frame, Margin, Panel, Rect, ScrollArea, Sense, Ui, pos2, scroll_area::ScrollSource, vec2};
use tracing::info;
use crate::{core::{blaze_state::BlazeCoreState, configs::config_state::{FavoriteLinks, with_configs}, system::{clipboard::TOKIO_RUNTIME, disk_reader::disk::Disk}}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons::{ICON_ARCHIVE, ICON_DESKTOP, ICON_DEVICE_PC, ICON_DOWNLOADS, ICON_HOME, ICON_POLAROID, ICON_SERVER, ICON_STAR, ICON_TRASH, ICON_USB, ICON_USER}, modules::custom_context_menu::context_state::ContextMenuKind}};




fn get_folder_icon(label: &str) -> (&'static str, &'static [u8]) {
    let label_lower = label.to_lowercase();
    match label_lower.as_str() {
        "home" => ("home", ICON_HOME),
        "escritorio" | "desktop" => ("desktop", ICON_DESKTOP),
        "descargas" | "downloads" => ("downloads", ICON_DOWNLOADS),
        "documentos" | "documents" => ("documents", ICON_ARCHIVE),
        "imágenes" | "imagenes" | "pictures" | "images" => ("pictures", ICON_POLAROID),
        "papelera" | "trash" | "basura" => ("trash", ICON_TRASH),
        _ => ("default", ICON_HOME),
    }
}


fn get_herader_icon(label: &str) -> (&'static str, &'static [u8]) {
    let label_lower = label.to_lowercase();
    match label_lower.as_str() {
        "locales" => ("locales", ICON_USER),
        "favoritos" | "favorites" => ("favorites", ICON_STAR),
        "discos" | "disks" => ("disks", ICON_SERVER),
        _ => ("default", ICON_HOME),
    }
}


pub fn render_icon(ui: &mut Ui, ui_state: &mut BlazeUiState, icon_name: &str, color: Color32, icon_bytes: &[u8], rect: Rect) {
    let icon = ui_state.icon_cache.get_or_load(ui, icon_name, icon_bytes, color);

    let icon_size = vec2(16.0, 16.0);
    let icon_pos = rect.left_center() - vec2(-10.0, icon_size.y / 2.0);
    let icon_rect = Rect::from_min_size(icon_pos, icon_size);

    ui.painter().image(
        icon.id(),
        icon_rect,
        Rect::from_min_max(pos2(0.0, 0.0),
        pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}


pub fn render_local_buttons(label:&str, path: PathBuf, state: &mut BlazeCoreState, ui: &mut Ui, ui_state: &mut BlazeUiState) {
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

    let (icon_name, icon_bytes) = get_folder_icon(label);
    let color = Color32::WHITE;

    render_icon(
        ui,
        ui_state,
        icon_name,
        color,
        icon_bytes,
        rect
    );


    ui.painter().text(
        rect.left_center() + vec2(34.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::default(),
        ui.visuals().text_color(),
    );

}


pub fn render_fav_buttons(ui: &mut Ui, fav: FavoriteLinks, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
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

    let color = Color32::YELLOW;

    render_icon(
        ui,
        ui_state,
        "star",
        color,
        ICON_STAR,
        rect,
    );

    ui.painter().with_clip_rect(rect)
        .galley(
            rect.left_center() + vec2(34.0, -galley.size().y / 2.0),
            galley,
            ui.visuals().text_color()
        );
}


pub fn render_drives_button(ui: &mut Ui, state: &mut BlazeCoreState, drive: Disk, ui_state: &mut BlazeUiState) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 30.0),
        Sense::click_and_drag()
    );

    if response.clicked() && !drive.mountpoint.is_none() {
        let path_string = drive.mountpoint.clone().unwrap_or_default();
        let path = PathBuf::from(path_string);
        state.navigate_to(path);
    }


    let root_symbol = "/".to_string();
    let display_name = if drive.mountpoint == Some(root_symbol.clone()) {
        root_symbol
    } else {
        drive.display_name.clone()
    };

    let is_removable = drive.is_removable;
    let is_system = drive.is_system;

    if response.secondary_clicked() {
        ui_state.context_menu_state.handle_response(&response);
        ui_state.context_menu_state.kind = ContextMenuKind::DrivesPanel;
        ui_state.context_menu_state.target_drive = Some(drive);
    }


    let bg_color = if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        egui::Color32::from_rgba_unmultiplied(100, 100, 255, 60)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    };

    ui.painter().rect_filled(rect, 5.0, bg_color);

    let color = Color32::WHITE;

    let (icon, bytes) = if is_system {
        ("system", ICON_DEVICE_PC)
    } else if is_removable {
        ("usb", ICON_USB)
    } else {
        ("server", ICON_SERVER)
    };


    render_icon(
        ui,
        ui_state,
        icon,
        color,
        bytes,
        rect
    );


    ui.painter().text(
        rect.left_center() + vec2(34.0, 0.0),
        egui::Align2::LEFT_CENTER,
        display_name,
        FontId::default(),
        ui.visuals().text_color(),
    );

}


pub fn render_header_text(label: &str,ui: &mut Ui, ui_state: &mut BlazeUiState) {
    let (rect, _resp) = ui.allocate_exact_size(
        vec2(ui.available_width(), 24.0),
        Sense::hover(),
    );


    let (icon_name, icon_bytes) = get_herader_icon(label);
    let color = Color32::WHITE;

    render_icon(
        ui,
        ui_state,
        icon_name,
        color,
        icon_bytes,
        rect
    );

    ui.painter().text(
        rect.left_center() + vec2(34.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(20.0),
        Color32::from_rgb(255, 255, 255),
    );
}


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

            let dirs = [
                ("Home",        home_dir()),
                ("Escritorio",  desktop_dir()),
                ("Descargas",   download_dir()),
                ("Documentos",  document_dir()),
                ("Imágenes",    picture_dir()),
                ("Papelera",    state.motor.borrow_mut().get_trash_dir(None)),
            ];

            for (label, path) in dirs {
                if let Some(path) = path {
                    render_local_buttons(label, path, state, ui, ui_state);
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