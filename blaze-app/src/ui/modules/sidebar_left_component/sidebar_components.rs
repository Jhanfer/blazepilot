use crate::{
    core::{blaze_state::BlazeCoreState, system::disk_reader::disk::Disk},
    ui::{
        blaze_ui_state::BlazeUiState, custom_components::UiExt, icons_cache::icons::*,
        modules::custom_context_menu::context_state::ContextMenuKind,
        themes::theme_manager::with_theme,
    },
    utils::formating::format_size,
};
use egui::{Align2, Color32, FontId, PointerButton, Rect, Sense, Ui, pos2, vec2};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// Separador que respeta el tema de la sección (a diferencia de `ui.separator()`).
pub fn render_separator(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::empty());
    ui.painter().rect_filled(rect, 0.0, color);
}

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

pub fn render_header_text(label: &str, ui: &mut Ui) {
    let current_theme = with_theme(|t| t.current());
    let palette = current_theme.sidebar_left.palette(&current_theme.global);

    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::empty());

    ui.painter().text(
        rect.left_center() + vec2(10.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(15.0),
        palette.fg_secondary,
    );
}

pub fn render_local_buttons(
    icon_key: &str,
    label: &str,
    path: Arc<Path>,
    state: &mut BlazeCoreState,
    ui: &mut Ui,
) {
    let current_theme = with_theme(|t| t.current());
    let palette = current_theme.sidebar_left.palette(&current_theme.global);
    let (icon_name, icon_bytes) = get_folder_icon(icon_key);

    let btn_id = ui.id().with(label);
    let available_w = ui.available_width();
    let btn_resp = ui
        .blaze_button(label)
        .id(btn_id)
        .size((available_w, 35.0))
        .background_color(palette.bg_elevated)
        .hover_background_color(palette.bg_hover)
        .radius(20)
        .stroke_color(palette.border_component)
        .hover_stroke_color(palette.border_focus)
        .icon(icon_name, icon_bytes)
        .icon_color(palette.icon_primary)
        .hover_icon_color(palette.icon_hover)
        .icon_size(16_f32)
        .text_color(palette.fg_primary)
        .stroke_width(current_theme.global.components_stroke_width)
        .show();

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
    let palette = current_theme.sidebar_left.palette(&current_theme.global);

    let used = drive.used_percent / 100.0;
    let is_mounted = drive.mountpoint.is_some();
    let used_percentage = format!("Used {}%", drive.used_percent as i32);
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

    let btn_id = ui.id().with(&display_name);
    let available_w = ui.available_width();
    let btn_resp = ui
        .blaze_button(&display_name)
        .id(btn_id)
        .size((available_w, if is_mounted { 68.0 } else { 35.0 }))
        .background_color(palette.bg_elevated)
        .hover_background_color(palette.bg_hover)
        .radius(20)
        .stroke_color(palette.border_component)
        .hover_stroke_color(palette.border_focus)
        .icon(icon_name, icon_bytes)
        .icon_color(palette.icon_primary)
        .hover_icon_color(palette.icon_hover)
        .icon_size(16_f32)
        .text_color(palette.fg_primary)
        .stroke_width(current_theme.global.components_stroke_width)
        .content_offset_y(if is_mounted { -15.0 } else { 0.0 })
        .show();

    if is_mounted {
        let padding_x = 10.0;
        let padding_y = 28.0;
        let height = 6.0;
        let text_padding = 2.0;

        let progress_rect = Rect::from_min_size(
            pos2(
                btn_resp.rect.min.x + padding_x,
                btn_resp.rect.min.y - height + btn_resp.rect.height() - padding_y,
            ),
            vec2(
                btn_resp.rect.width() - (padding_x * 2.0) + text_padding,
                height,
            ),
        );

        //Fondo de la barra
        ui.painter()
            .rect_filled(progress_rect, 10.0, palette.bg_hover);

        let filled_width = progress_rect.width() * used.clamp(0.0, 1.0);
        let filled_rect = Rect::from_min_size(
            progress_rect.min,
            vec2(filled_width, progress_rect.height()),
        );

        //relleno
        ui.painter().rect_filled(filled_rect, 10.0, palette.accent);

        let used_bytes = drive.total.saturating_sub(drive.available);
        let label = format!("{} / {}", format_size(used_bytes), format_size(drive.total));

        ui.painter().text(
            pos2(progress_rect.min.x, progress_rect.max.y + 3.0),
            Align2::LEFT_TOP,
            label,
            FontId::proportional(12.0),
            palette.fg_secondary,
        );
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
