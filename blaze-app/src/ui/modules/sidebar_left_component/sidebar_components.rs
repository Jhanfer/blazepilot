use crate::{
    core::{blaze_state::BlazeCoreState, system::disk_reader::disk::Disk},
    ui::{
        blaze_ui_state::BlazeUiState, icons_cache::icons::*,
        modules::custom_context_menu::context_state::ContextMenuKind, themes::colors::*,
    },
};
use egui::{
    pos2, vec2, Align2, Color32, FontId, PointerButton, Rect, Sense, Stroke, StrokeKind, Ui,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

fn get_folder_icon(key: &str) -> (&'static str, &'static [u8]) {
    match key {
        "home" => ("home", ICON_HOME),
        "desktop" => ("desktop", ICON_DESKTOP),
        "downloads" => ("downloads", ICON_DOWNLOADS),
        "documents" => ("documents", ICON_ARCHIVE),
        "pictures" | "images" => ("images", ICON_POLAROID),
        "trash" => ("trash", ICON_TRASH),
        "videos" => ("videos", ICON_VIDEO),
        "public" => ("public", ICON_PUBLIC),
        "music" => ("music", ICON_MUSIC),
        _ => ("default", ICON_HOME),
    }
}

fn get_header_icon(key: &str) -> (&'static str, &'static [u8]) {
    match key {
        "locals" => ("locales", ICON_USER),
        "favorites" => ("favorites", ICON_STAR),
        "disks" => ("disks", ICON_SERVER),
        _ => ("default", ICON_HOME),
    }
}

pub fn render_icon(
    ui: &mut Ui,
    ui_state: &mut BlazeUiState,
    icon_name: &str,
    color: Color32,
    icon_bytes: &[u8],
    rect: Rect,
) {
    let icon_size = vec2(16.0, 16.0);
    let icon_pos = rect.left_center() - vec2(-10.0, icon_size.y / 2.0);
    let icon_rect = Rect::from_min_size(icon_pos, icon_size);

    let rounded_rect = Rect::from_min_max(
        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
    );

    let icon: &egui::TextureHandle = ui_state
        .icon_cache
        .get_or_load(ui, icon_name, icon_bytes, color, icon_size);

    ui.painter().image(
        icon.id(),
        rounded_rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

pub fn render_header_text(icon_key: &str, label: &str, ui: &mut Ui, ui_state: &mut BlazeUiState) {
    let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());

    let (icon_name, icon_bytes) = get_header_icon(icon_key);

    render_icon(
        ui,
        ui_state,
        icon_name,
        COLOR_TEXT_PRIMARY,
        icon_bytes,
        rect,
    );

    ui.painter().text(
        rect.left_center() + vec2(34.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(20.0),
        Color32::from_rgb(255, 255, 255),
    );
}

pub fn render_local_buttons(
    icon_key: &str,
    label: &str,
    path: Arc<Path>,
    state: &mut BlazeCoreState,
    ui: &mut Ui,
    ui_state: &mut BlazeUiState,
) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 30.0),
        Sense::click_and_drag(),
    );

    let (bg_color, icon_color) = if response.hovered() {
        ui.set_cursor_icon(egui::CursorIcon::PointingHand);
        (
            Color32::from_rgba_unmultiplied(100, 100, 255, 60),
            COLOR_ACCENT_GLOW,
        )
    } else {
        (COLOR_MAIN_BUTTONS, Color32::WHITE)
    };

    ui.painter().rect(
        rect,
        20.0,
        bg_color,
        if response.hovered() {
            Stroke::new(0.5, icon_color)
        } else {
            Stroke::NONE
        },
        StrokeKind::Outside,
    );

    let middle_clicked = ui.input(|i| {
        i.pointer.button_pressed(PointerButton::Middle)
            && i.pointer
                .interact_pos()
                .map(|p| rect.contains(p))
                .unwrap_or(false)
    });

    if middle_clicked {
        state.add_tab_from_file(&path);
    }

    if response.clicked() {
        state.navigate_to(path);
    }

    let (icon_name, icon_bytes) = get_folder_icon(icon_key);

    render_icon(ui, ui_state, icon_name, icon_color, icon_bytes, rect);

    ui.painter().text(
        rect.left_center() + vec2(34.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::default(),
        COLOR_TEXT_SECONDARY,
    );
}

pub fn render_drives_button(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    drive: Disk,
    ui_state: &mut BlazeUiState,
) {
    let used = drive.used_percent / 100.0;
    let is_mounted = drive.mountpoint.is_some();
    let used_percentage = format!("Usado {}%", drive.used_percent as i32);
    let is_removable = drive.is_removable;
    let is_system = drive.is_system;

    let btn_h = if is_mounted { 50.0 } else { 30.0 };

    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), btn_h), Sense::click_and_drag());

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

    let (bg_color, icon_color) = if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        (
            Color32::from_rgba_unmultiplied(100, 100, 255, 60),
            COLOR_ACCENT_GLOW,
        )
    } else {
        (COLOR_MAIN_BUTTONS, Color32::WHITE)
    };

    ui.painter().rect(
        rect,
        20.0,
        bg_color,
        if response.hovered() {
            Stroke::new(0.5, icon_color)
        } else {
            Stroke::NONE
        },
        StrokeKind::Outside,
    );

    response.on_hover_text(used_percentage);

    if is_mounted {
        let padding_x = 34.0;
        let padding_y = 10.0;
        let height = 6.0;

        let progress_rect = Rect::from_min_size(
            pos2(
                rect.min.x + padding_x,
                rect.min.y - height + rect.height() - padding_y,
            ),
            vec2(rect.width() - (padding_x * 2.0), height),
        );

        //Fondo de la barra
        ui.painter().rect_filled(
            progress_rect,
            10.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 30),
        );

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

    let (icon, bytes) = if is_system {
        ("system", ICON_DEVICE_PC)
    } else if is_removable {
        ("usb", ICON_USB)
    } else {
        ("server", ICON_SERVER)
    };

    render_icon(ui, ui_state, icon, icon_color, bytes, rect);

    let y = if is_mounted { -8.0 } else { 0.0 };

    ui.painter().text(
        rect.left_center() + vec2(34.0, y),
        Align2::LEFT_CENTER,
        display_name,
        FontId::default(),
        COLOR_TEXT_SECONDARY,
    );
}
