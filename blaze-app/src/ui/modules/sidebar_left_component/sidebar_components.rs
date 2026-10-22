use crate::{
    core::{blaze_state::BlazeCoreState, system::disk_reader::disk::Disk},
    ui::{
        blaze_ui_state::BlazeUiState,
        custom_components::UiExt,
        icons_cache::icons::*,
        modules::{
            custom_context_menu::context_state::ContextMenuKind, utilities::ensure_min_lightness,
        },
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{Align2, Color32, FontId, PointerButton, Rect, Sense, Ui, pos2, vec2};
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

    let icon: &egui::TextureHandle = &ui_state
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
    let current_theme = with_theme(|t| t.current());

    let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());

    let (icon_name, icon_bytes) = get_header_icon(icon_key);

    render_icon(
        ui,
        ui_state,
        icon_name,
        current_theme.semantic.text_primary.to_color(),
        icon_bytes,
        rect,
    );

    ui.painter().text(
        rect.left_center() + vec2(34.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(20.0),
        current_theme.semantic.text_primary.to_color(),
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
    let current_theme = with_theme(|t| t.current());
    let (icon_name, icon_bytes) = get_folder_icon(icon_key);

    let is_dark_theme = current_theme.semantic.text_primary.to_color().r() > 128;

    let cbg = current_theme.components.button.bg_hover.to_color();
    let bg = current_theme.components.button.bg.to_color();
    let hover_bg = if is_dark_theme {
        Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 80)
    } else {
        Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 10)
    };

    let icon_color = current_theme.components.button.label_active.to_color();
    let hover_icon = ensure_min_lightness(current_theme.components.button.label_hover.to_color());

    let btn_id = ui.id().with(label);
    let btn_resp = ui
        .blaze_button(label, Some(&mut ui_state.icon_cache))
        .id(btn_id)
        .size((ui.available_width(), 35.0))
        .background_color(bg)
        .hover_background_color(hover_bg)
        .radius(20)
        .stroke_color(Color32::TRANSPARENT)
        .hover_stroke_color(hover_icon)
        .icon(icon_name, icon_bytes)
        .icon_color(icon_color)
        .hover_icon_color(hover_icon)
        .icon_size(16_f32)
        .text_color(current_theme.semantic.text_primary.to_color())
        .show(ui);

    let middle_clicked = ui.input(|i| {
        i.pointer.button_pressed(PointerButton::Middle)
            && i.pointer
                .interact_pos()
                .map(|p| btn_resp.rect.contains(p))
                .unwrap_or(false)
    });

    if middle_clicked {
        state.add_tab_from_file(&path);
    }

    if btn_resp.clicked() {
        if !state.is_miller() {
            state.navigate_to(path);
        } else {
            state.miller_reset_to(path);
        }
    }
}

pub fn render_drives_button(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    drive: Disk,
    ui_state: &mut BlazeUiState,
) {
    let current_theme = with_theme(|t| t.current());

    let used = drive.used_percent / 100.0;
    let is_mounted = drive.mountpoint.is_some();
    let used_percentage = format!("Usado {}%", drive.used_percent as i32);
    let is_removable = drive.is_removable;
    let is_system = drive.is_system;

    let display_name = if drive.mountpoint == Some("/".to_owned()) {
        "Root".to_owned()
    } else {
        drive.display_name.clone()
    };

    let (icon_name, icon_bytes) = if is_system {
        ("system", ICON_DEVICE_PC)
    } else if is_removable {
        ("usb", ICON_USB)
    } else {
        ("server", ICON_SERVER)
    };

    let is_dark_theme = current_theme.semantic.text_primary.to_color().r() > 128;

    let cbg = current_theme.components.button.bg_hover.to_color();
    let bg = current_theme.components.button.bg.to_color();
    let hover_bg = if is_dark_theme {
        Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 80)
    } else {
        Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 10)
    };

    let icon_color = current_theme.components.button.label_active.to_color();
    let hover_icon = ensure_min_lightness(current_theme.components.button.label_hover.to_color());

    let btn_id = ui.id().with(&display_name);
    let btn_resp = ui
        .blaze_button(&display_name, Some(&mut ui_state.icon_cache))
        .id(btn_id)
        .size((ui.available_width(), if is_mounted { 50.0 } else { 35.0 }))
        .background_color(bg)
        .hover_background_color(hover_bg)
        .radius(20)
        .stroke_color(Color32::TRANSPARENT)
        .hover_stroke_color(hover_icon)
        .icon(icon_name, icon_bytes)
        .icon_color(icon_color)
        .hover_icon_color(hover_icon)
        .icon_size(16_f32)
        .text_color(current_theme.semantic.text_primary.to_color())
        .content_offset_y(if is_mounted { -8.0 } else { 0.0 })
        .show(ui);

    if is_mounted {
        let padding_x = 34.0;
        let padding_y = 10.0;
        let height = 6.0;

        let progress_rect = Rect::from_min_size(
            pos2(
                btn_resp.rect.min.x + padding_x,
                btn_resp.rect.min.y - height + btn_resp.rect.height() - padding_y,
            ),
            vec2(btn_resp.rect.width() - (padding_x * 2.0), height),
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

        //relleno
        ui.painter()
            .rect_filled(filled_rect, 10.0, current_theme.semantic.accent.to_color());
    }

    if btn_resp.clicked() && is_mounted {
        let path_string = drive.mountpoint.clone().unwrap_or_default();
        let path = PathBuf::from(path_string);
        state.navigate_to(path.as_path().into());
    }

    if btn_resp.secondary_clicked() {
        ui_state.context_menu_state.handle_response(&btn_resp);
        ui_state.context_menu_state.kind = ContextMenuKind::DrivesPanel;
        ui_state.context_menu_state.target_drive = Some(drive);
    }

    btn_resp.on_hover_text(used_percentage);
}
