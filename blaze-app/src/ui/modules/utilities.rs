use crate::{
    core::{
        bootstrap::quick_access_manager::platform::structs::QuickLinks,
        files::{
            blaze_motor::motor_structs::FileEntry,
            file_extension::{DocType, FileExtension},
        },
        system::extended_info::extended_info_manager::GitStatus,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::{icons::*, thumbnails::thumbnails_manager::Thumbnail},
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{
    Align, Align2, Color32, ColorImage, CornerRadius, CursorIcon, Layout, Rect, Sense, Stroke,
    StrokeKind, TextStyle, TextureOptions, Ui, Vec2, lerp, pos2, vec2,
};
use file_id::FileId;
use std::{collections::HashMap, path::Path, sync::Arc};

pub fn resolve_icon(
    file: &Arc<FileEntry>,
    color_snapshot: &HashMap<FileId, Color32>,
) -> (String, &'static [u8], Color32) {
    let current_theme = with_theme(|t| t.current());
    if file.is_dir() {
        let (color, cache_key) = if let Some(file_id) = &file.unique_id {
            let color = color_snapshot
                .get(file_id)
                .copied()
                .unwrap_or(current_theme.file_theme.folder_default.to_color());
            let cache_key = format!("folder-{:?}", file_id);
            (color, cache_key)
        } else {
            (
                current_theme.file_theme.folder_default.to_color(),
                "folder-unknown".to_string(),
            )
        };
        (cache_key, ICON_FOLDER_OPEN, color)
    } else {
        match &file.extension {
            FileExtension::Image(_) => (
                "image".to_string(),
                ICON_IMAGE,
                current_theme.file_theme.image.to_color(),
            ),
            FileExtension::Document(DocType::Pdf) => (
                "pdf".to_string(),
                ICON_PDF,
                current_theme.file_theme.pdf.to_color(),
            ),
            FileExtension::Document(_) => (
                "doc".to_string(),
                ICON_DOC,
                current_theme.file_theme.document.to_color(),
            ),
            FileExtension::Video(_) => (
                "video".to_string(),
                ICON_VIDEO,
                current_theme.file_theme.video.to_color(),
            ),
            FileExtension::Audio(_) => (
                "audio".to_string(),
                ICON_VIDEO,
                current_theme.file_theme.audio.to_color(),
            ),
            FileExtension::Archive(_) => (
                "archive".to_string(),
                ICON_ARCHIVE,
                current_theme.file_theme.archive.to_color(),
            ),
            FileExtension::Code(_) => (
                "code".to_string(),
                ICON_CODE,
                current_theme.file_theme.code.to_color(),
            ),
            FileExtension::Font(_) => (
                "font".to_string(),
                ICON_FONT,
                current_theme.file_theme.font.to_color(),
            ),
            FileExtension::Executable(_) => (
                "exe".to_string(),
                ICON_EXE,
                current_theme.file_theme.executable.to_color(),
            ),
            FileExtension::Unknown => (
                "file".to_string(),
                ICON_FILE,
                current_theme.file_theme.fallback.to_color(),
            ),
        }
    }
}

pub fn text_color_for_git(git: Option<&GitStatus>) -> Color32 {
    let current_theme = with_theme(|t| t.current());
    match git {
        Some(GitStatus::Modified) => Color32::from_rgb(255, 200, 80),
        Some(GitStatus::Staged) => Color32::from_rgb(100, 220, 100),
        Some(GitStatus::Untracked) => Color32::from_rgb(160, 160, 160),
        Some(GitStatus::Ignored) => Color32::from_rgb(100, 100, 100),
        Some(GitStatus::Conflict) => Color32::from_rgb(255, 80, 80),
        Some(GitStatus::Deleted) => Color32::from_rgb(255, 60, 60),
        Some(GitStatus::Clean) | None => current_theme.text_primary.to_color(),
    }
}

pub fn git_dot_color(git: Option<&GitStatus>) -> Option<Color32> {
    match git {
        Some(GitStatus::Modified) => Some(Color32::from_rgb(255, 200, 80)),
        Some(GitStatus::Staged) => Some(Color32::from_rgb(100, 220, 100)),
        Some(GitStatus::Untracked) => Some(Color32::from_rgb(160, 160, 160)),
        Some(GitStatus::Ignored) => Some(Color32::from_rgb(80, 80, 80)),
        Some(GitStatus::Conflict) => Some(Color32::from_rgb(255, 80, 80)),
        Some(GitStatus::Deleted) => Some(Color32::from_rgb(255, 60, 60)),
        Some(GitStatus::Clean) | None => None,
    }
}

pub fn ensure_min_lightness(color: Color32) -> Color32 {
    let min_lightness = with_theme(|t| t.current().luminance);

    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if l >= min_lightness {
        return color;
    }

    let delta = max - min;
    let s = if delta < 1e-6 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let h = if delta < 1e-6 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let new_l = min_lightness;
    let c = (1.0 - (2.0 * new_l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = new_l - c / 2.0;

    let (r1, g1, b1) = match h {
        0.0..=59.0 => (c, x, 0.0),
        60.0..=119.0 => (x, c, 0.0),
        120.0..=179.0 => (0.0, c, x),
        180.0..=239.0 => (0.0, x, c),
        240.0..=299.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color32::from_rgb(
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

///----------- Componentes --------------
pub fn render_quicklink_icon(
    ui: &mut Ui,
    item: &QuickLinks,
    thumb_snapshot: &HashMap<Arc<Path>, Thumbnail>,
    ui_state: &mut BlazeUiState,
    icon_rect: Rect,
    icon_size: Vec2,
) {
    if let Some(thumb) = thumb_snapshot.get(&item.path) {
        let tex = ui_state
            .thumb_texture_cache
            .entry(item.path.clone())
            .or_insert_with_key(|path| {
                let img = ColorImage::from_rgba_unmultiplied(
                    [thumb.width as usize, thumb.height as usize],
                    &thumb.pixels,
                );
                ui.load_texture(
                    format!("thumb:{}", path.to_string_lossy()),
                    img,
                    TextureOptions::LINEAR,
                )
            });

        ui.painter().image(
            tex.id(),
            icon_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        return;
    }

    let dummy_entry = FileEntry {
        full_path: item.path.to_owned(),
        name: item.name.to_owned(),
        extension: FileExtension::from_path(&item.path),
        kind: item.kind.to_owned(),
        ..Default::default()
    };

    let (icon_name, icon_bytes, color) = resolve_icon(&Arc::from(dummy_entry), &Default::default());
    let rounded_rect = Rect::from_min_max(
        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
    );
    let icon = ui_state
        .icon_cache
        .get_or_load(ui, &icon_name, icon_bytes, color, icon_size);

    ui.painter().image(
        icon.id(),
        rounded_rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

pub fn render_button<F, C>(
    ui: &mut Ui,
    label: &str,
    mut fill_color: Color32,
    accent_color: Color32,
    mut callback: Option<F>,
    dispatch: Option<C>,
) where
    F: FnMut(),
    C: Fn(&mut Ui),
{
    let current_theme = with_theme(|t| t.current());

    let mut font = TextStyle::Button.resolve(ui.style());

    let galley = ui.fonts_mut(|f| {
        f.layout_no_wrap(
            label.to_string(),
            font.clone(),
            current_theme.text_primary.to_color(),
        )
    });

    let clickable = callback.is_some();

    let size = vec2(galley.size().x + 18.0, 25.0);

    let (rect, resp) = ui.allocate_exact_size(
        size,
        if clickable {
            Sense::click()
        } else {
            Sense::hover()
        },
    );

    let hover_t = ui.animate_bool(resp.id.with("hover"), resp.hovered());

    let press_t = ui.animate_bool(resp.id.with("press"), resp.is_pointer_button_down_on());

    let scale = lerp(1.0..=1.03, hover_t) * lerp(1.0..=0.94, press_t);

    let mut animated_rect = rect;

    if resp.hovered() && clickable {
        animated_rect = rect.scale_from_center(scale);

        font.size *= scale;

        ui.set_cursor_icon(egui::CursorIcon::PointingHand);
        fill_color = Color32::from_rgb(
            fill_color.r().saturating_add_signed(50),
            fill_color.g().saturating_add_signed(50),
            fill_color.b().saturating_add_signed(50),
        );
    }

    if let Some(dp) = dispatch {
        resp.context_menu(|ui| dp(ui));
    }

    if resp.clicked()
        && let Some(cb) = callback.as_mut()
    {
        cb();
    }

    ui.painter().rect(
        animated_rect,
        CornerRadius::same(20),
        fill_color,
        Stroke::new(0.9, accent_color),
        StrokeKind::Outside,
    );

    ui.painter().text(
        animated_rect.center(),
        Align2::CENTER_CENTER,
        label,
        font,
        accent_color,
    );
}

pub fn render_op_buttons<F, C>(
    ui: &mut Ui,
    ui_state: &mut BlazeUiState,
    tag_color: Color32,
    editcallback: F,
    deletecallback: C,
) where
    F: Fn(),
    C: Fn(),
{
    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
        ui.add_space(60.0);
        ui.horizontal(|ui| {
            let total_width = 40.0;
            let (container_rect, resp1) =
                ui.allocate_exact_size(vec2(total_width, 25.0), Sense::click());

            let gap = 6.0;
            let second_pos = pos2(container_rect.right() + gap, container_rect.top());
            let second_container = Rect::from_min_size(second_pos, vec2(total_width, 25.0));

            let base_id = ui.id().with("resp_btn2");

            let resp2 = ui.interact(
                second_container,
                resp1.id.with(base_id.with("btn2")),
                Sense::click(),
            );

            let hover_t_bt2 = ui.animate_bool(resp2.id.with("hover_t_bt2"), resp2.hovered());
            let press_t_bt2 = ui.animate_bool(
                resp2.id.with("press_t_bt2"),
                resp2.is_pointer_button_down_on(),
            );

            let bg_color1 = tag_color;

            let accent1 = Color32::from_rgb(
                bg_color1.r().saturating_add(120),
                bg_color1.g().saturating_add(120),
                bg_color1.b().saturating_add(120),
            );

            let hover_t_bt1 = ui.animate_bool(resp1.id.with("hover_t_bt1"), resp1.hovered());
            let press_t_bt1 = ui.animate_bool(
                resp1.id.with("press_t_bt1"),
                resp1.is_pointer_button_down_on(),
            );

            let merged_hover = hover_t_bt1.max(hover_t_bt2);
            let merged_press = press_t_bt1.max(press_t_bt2);

            let scale = lerp(1.0..=1.03, merged_hover) * lerp(1.0..=0.94, merged_press);

            let mut animated_rect1 = container_rect;
            let mut animated_rect2 = second_container;

            let icon_size = vec2(18.0, 18.0);

            let icon1_pos = pos2(
                animated_rect1.center().x - (icon_size.y / 2.0),
                animated_rect1.center().y - (icon_size.y / 2.0),
            );
            let icon1_rect = Rect::from_min_size(icon1_pos, icon_size);

            let icon2_pos = pos2(
                animated_rect2.center().x - (icon_size.y / 2.0),
                animated_rect2.center().y - (icon_size.y / 2.0),
            );

            let icon2_rect = Rect::from_min_size(icon2_pos, icon_size);

            let mut animated_icon_rect1 = icon1_rect;
            let mut animated_icon_rect2 = icon2_rect;

            if resp1.hovered() {
                ui.set_cursor_icon(CursorIcon::PointingHand);
                animated_rect1 = container_rect.scale_from_center(scale);
                animated_icon_rect1 = icon1_rect.scale_from_center(scale);
            }

            if resp1.clicked() {
                editcallback();
            }

            if resp2.hovered() {
                ui.set_cursor_icon(CursorIcon::PointingHand);
                animated_rect2 = second_container.scale_from_center(scale);
                animated_icon_rect2 = icon2_rect.scale_from_center(scale);
            }

            if resp2.clicked() {
                deletecallback();
            }

            ui.painter().rect(
                animated_rect1,
                CornerRadius::same(20),
                bg_color1,
                Stroke::new(0.8, accent1),
                StrokeKind::Outside,
            );

            let bg_color2 = Color32::from_rgb(165, 42, 42);

            let accent2 = Color32::from_rgb(
                bg_color2.r().saturating_add(70),
                bg_color2.g().saturating_add(70),
                bg_color2.b().saturating_add(70),
            );

            ui.painter().rect(
                animated_rect2,
                CornerRadius::same(20),
                bg_color2,
                Stroke::new(0.8, accent2),
                StrokeKind::Outside,
            );

            let icon_edit = ui_state
                .icon_cache
                .get_or_load(ui, "edit", ICON_EDIT, accent1, icon_size);

            ui.painter().image(
                icon_edit.id(),
                animated_icon_rect1,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            let icon_trash = ui_state
                .icon_cache
                .get_or_load(ui, "trash", ICON_TRASH, accent2, icon_size);

            ui.painter().image(
                icon_trash.id(),
                animated_icon_rect2,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        });
    });
}
