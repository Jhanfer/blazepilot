use std::{path::{Path, PathBuf}, sync::Arc};
use egui::{Align2, Color32, CursorIcon, FontId, PointerButton, Rect, Sense, Ui, pos2, vec2};
use tracing::info;
use crate::{core::{blaze_state::BlazeCoreState, bootstrap::configs::{config_manager::with_configs, platform::linux::conf_structs::FavoriteLinks}, files::file_extension::{DocType, FileExtension}, system::disk_reader::disk::Disk}, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons::*, modules::custom_context_menu::context_state::ContextMenuKind}};



fn get_folder_icon(label: &str) -> (&'static str, &'static [u8]) {
    let label_lower = label.to_lowercase();
    match label_lower.as_str() {
        "home" => ("home", ICON_HOME),
        "escritorio" | "desktop" => ("desktop", ICON_DESKTOP),
        "descargas" | "downloads" => ("downloads", ICON_DOWNLOADS),
        "documentos" | "documents" => ("documents", ICON_ARCHIVE),
        "imágenes" | "imagenes" | "pictures" | "images" => ("pictures", ICON_POLAROID),
        "papelera" | "trash" | "basura" => ("trash", ICON_TRASH),
        "videos" | "vídeos" => ("videos", ICON_VIDEO),
        "public" | "público" | "publico" => ("public", ICON_PUBLIC),
        "musica" | "music" | "música" => ("music", ICON_MUSIC),
        _ => ("default", ICON_HOME),
    }
}


fn get_herader_icon(label: &str) -> (&'static str, &'static [u8]) {
    let label_lower = label.to_lowercase();
    match label_lower.as_str() {
        "locales" | "locals" => ("locales", ICON_USER),
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
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(20.0),
        Color32::from_rgb(255, 255, 255),
    );
}



pub fn render_local_buttons(label:&str, path: Arc<Path>, state: &mut BlazeCoreState, ui: &mut Ui, ui_state: &mut BlazeUiState) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 30.0),
        Sense::click_and_drag()
    );
    let bg_color = if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(100, 100, 255, 60)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    };

    ui.painter().rect_filled(rect, 5.0, bg_color);


    let middle_clicked = ui.input(|i| {
        i.pointer.button_pressed(PointerButton::Middle)
        && i.pointer.interact_pos()
            .map(|p| rect.contains(p))
            .unwrap_or(false)
    });

    if middle_clicked {
        state.add_tab_from_file(&*path);
    }

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
        Align2::LEFT_CENTER,
        label,
        FontId::default(),
        ui.visuals().text_color(),
    );

}


pub fn render_fav_buttons(ui: &mut Ui, fav: FavoriteLinks, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), 30.0),
        Sense::click()
    );

    let bg_color = if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(100, 100, 255, 60)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    };

    ui.painter()
        .rect_filled(
            rect,
            5.0,
            bg_color
        );
    
    let ext = FileExtension::from_path(&fav.path);

    if response.clicked() {
        if fav.is_dir {
            let path = fav.path.to_owned();
            state.navigate_to(path);
        } else {
            info!("Intentando abrir");
            state.open_file_by_path(fav.path.to_owned());
        }
    }

    response.context_menu(|ui|{
        if ui.button("Eliminar de favoritos").clicked() {
            with_configs(|c| {
                c.delete_from_favorites(&fav.name, &fav.path);
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

    let (name, icon, color) = if fav.is_dir {
        ("folder".to_owned(), ICON_FOLDER, Color32::YELLOW)
    } else {
        match &ext {
            FileExtension::Image(_) => ("image".to_owned(), ICON_IMAGE, Color32::from_rgb(100, 200, 255)),
            FileExtension::Document(DocType::Pdf) => ("pdf".to_owned(), ICON_PDF, Color32::from_rgb(255, 80,  80)),
            FileExtension::Document(_) => ("doc".to_owned(), ICON_DOC, Color32::from_rgb(100, 140, 255)),
            FileExtension::Video(_) => ("video".to_owned(), ICON_VIDEO, Color32::from_rgb(200, 100, 255)),
            FileExtension::Audio(_) => ("audio".to_owned(), ICON_VIDEO, Color32::from_rgb(255, 200, 80)),
            FileExtension::Archive(_) => ("archive".to_owned(), ICON_ARCHIVE, Color32::from_rgb(255, 160, 60)),
            FileExtension::Code(_) => ("code".to_owned(), ICON_CODE, Color32::from_rgb(100, 255, 150)),
            FileExtension::Font(_) => ("font".to_owned(), ICON_FONT, Color32::from_rgb(200, 200, 200)),
            FileExtension::Executable(_) => ("exe".to_owned(), ICON_EXE, Color32::from_rgb(255, 100, 100)),
            FileExtension::Unknown => ("file".to_owned(), ICON_FILE, Color32::WHITE),
        }
    };

    render_icon(
        ui,
        ui_state,
        &name,
        color,
        icon,
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
    let used = drive.used_percent as f32 / 100.0;
    let is_mounted = !drive.mountpoint.is_none();
    let used_percentage = format!("Usado {}%", drive.used_percent as i32);
    let is_removable = drive.is_removable;
    let is_system = drive.is_system;

    let btn_h = if is_mounted {
        50.0
    } else {
        30.0
    };

    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), btn_h),
        Sense::click_and_drag()
    );


    if response.clicked() && is_mounted {
        let path_string = drive.mountpoint.clone().unwrap_or_default();
        let path = PathBuf::from(path_string);
        state.navigate_to(path.as_path().into());
    }


    let display_name = if drive.mountpoint == Some("/".to_owned()) {
        "Root".to_owned()
    } else {
        drive.display_name.clone()
    };


    if response.secondary_clicked() {
        ui_state.context_menu_state.handle_response(&response);
        ui_state.context_menu_state.kind = ContextMenuKind::DrivesPanel;
        ui_state.context_menu_state.target_drive = Some(drive);
    }


    let bg_color = if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(100, 100, 255, 60)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    };

    ui.painter()
        .rect_filled(rect, 5.0, bg_color);

    response.on_hover_text(used_percentage);

    if is_mounted {
        let padding_x = 34.0;
        let padding_y = 10.0;
        let height = 6.0;

        let progress_rect = Rect::from_min_size(
            pos2(rect.min.x + padding_x, rect.min.y - height + rect.height() - padding_y),
            vec2(rect.width() - (padding_x * 2.0), height),
        );

        //Fondo de la barra
        ui.painter()
            .rect_filled(progress_rect, 10.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30));


        let filled_width = progress_rect.width() * used.clamp(0.0, 1.0);
        let filled_rect = Rect::from_min_size(
            progress_rect.min,
            vec2(filled_width, progress_rect.height()),
        );


        let progress_color = if used >= 0.90 {
            Color32::from_rgb(239, 68, 68)
        } else if used >= 0.75 {
            Color32::from_rgb(249, 115, 22)
        } else if used >= 0.50 {
            Color32::from_rgb(250, 204, 21)
        } else if used >= 0.25 {
            Color32::from_rgb(56, 189, 248)
        } else {
            Color32::from_rgb(94, 234, 212)
        };

        //relleno
        ui.painter().rect_filled(filled_rect, 10.0, progress_color);

    }


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


    let y = if is_mounted {
        -8.0
    } else {
        0.0
    };

    ui.painter().text(
        rect.left_center() + vec2(34.0, y), 
        Align2::LEFT_CENTER,
        display_name,
        FontId::default(),
        ui.visuals().text_color(),
    );

}