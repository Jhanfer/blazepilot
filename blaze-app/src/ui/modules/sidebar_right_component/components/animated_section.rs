use crate::core::blaze_state::BlazeCoreState;
use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::files::blaze_motor::motor_structs::FileEntry;
use crate::core::system::extended_info::extended_info_manager::ExtendedInfo;
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::custom_components::UiExt;
use crate::ui::icons_cache::thumbnails::thumbnails_manager::Thumbnail;
use crate::ui::modules::utilities::{ensure_min_lightness, resolve_icon};
use crate::ui::themes::platform::structs::ToColor;
use crate::utils::formating::{format_date, format_size};
use egui::{
    Align, Color32, ColorImage, CornerRadius, Frame, Grid, Label, Layout, Margin, Rect, RichText,
    Sense, Stroke, TextureOptions, Ui, pos2, vec2,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::error;

pub(crate) fn render_animated_section(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    current_theme: Arc<crate::ui::themes::platform::structs::NewTheme>,
) {
    ui.add_space(10.0);

    let i18n = with_configs(|c| c.get_i18n());
    let path = state.cwd();
    let files: &[Arc<FileEntry>] = &state.get_files_for(&path);

    let has_selection = (0..files.len()).any(|i| state.is_selected(i));
    let target = if has_selection { 1.0 } else { 0.0 };
    let anim_id = ui.id().with("file_info_expand");

    let anim = ui.animate_value_with_time(anim_id, target, 0.10);

    if anim > 0.001 {
        if anim != target {
            ui.request_repaint();
        }

        let max_height = 350.0;
        let current_height = max_height * anim;

        let available = ui.available_rect_before_wrap();
        let clip_rect = Rect::from_min_size(available.min, vec2(available.width(), current_height));

        let prev_clip = ui.clip_rect();
        ui.set_clip_rect(clip_rect);

        let animated_stroke = Stroke::new(
            (0.5 * anim).clamp(0.0, 0.5),
            current_theme
                .components
                .panel
                .border
                .to_color()
                .linear_multiply(anim),
        );

        let animated_radius = CornerRadius::same((20.0 * anim) as u8);

        if let Some(first_selected_idx) = (0..files.len()).find(|&i| state.is_selected(i)) {
            Frame::NONE
                .outer_margin(Margin::symmetric(1, 1))
                .inner_margin(Margin::same(10))
                .stroke(animated_stroke)
                .fill(current_theme.semantic.bg_main.to_color())
                .corner_radius(animated_radius)
                .show(ui, |ui| {
                    let file = &files[first_selected_idx];

                    ui.add(
                        Label::new(
                            RichText::new(i18n.t("right_sidebar.info")).heading().color(
                                current_theme
                                    .semantic
                                    .text_primary
                                    .to_color()
                                    .linear_multiply(anim),
                            ),
                        )
                        .selectable(false),
                    );

                    ui.separator();

                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        let mut should_repaint = false;

                        let slide = (1.0 - anim) * -15.0;
                        if slide.abs() > 0.5 {
                            ui.add_space(slide);
                        }
                        let icon_size = vec2(70.0, 70.0);

                        let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::click());

                        let thumbnail_snapshot: HashMap<Arc<Path>, Arc<Thumbnail>> = {
                            let guard = ui_state.thumbnail_manager.thumb_map.read();
                            let mut map = HashMap::new();
                            if let Some(thumb) = guard.peek(&file.full_path).cloned() {
                                map.insert(file.full_path.clone(), thumb);
                            }
                            map
                        };

                        if let Some(thumb) = thumbnail_snapshot.get(&file.full_path) {
                            let tex = ui_state
                                .thumb_texture_cache
                                .entry(file.full_path.to_owned())
                                .or_insert_with_key(|path| {
                                    let color_image = ColorImage::from_rgba_unmultiplied(
                                        [thumb.width as usize, thumb.height as usize],
                                        &thumb.pixels,
                                    );

                                    should_repaint = true;

                                    ui.load_texture(
                                        format!("thumb:{}", path.to_string_lossy()),
                                        color_image,
                                        TextureOptions::LINEAR,
                                    )
                                });

                            ui.painter().image(
                                tex.id(),
                                icon_rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else if ui_state
                            .newly_calculated_thumbnails
                            .contains(&file.full_path)
                        {
                            should_repaint = true;
                        } else {
                            let snapshot_color = file
                                .unique_id
                                .as_ref()
                                .and_then(|id| ui_state.color_snapshot.get(id))
                                .copied();

                            let (icon_name, icon_bytes, color) = resolve_icon(file, snapshot_color);

                            let rounded_rect = Rect::from_min_max(
                                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                            );

                            let normalized_color = ensure_min_lightness(color);

                            let icon = ui_state.icon_cache.get_or_load(
                                ui,
                                &icon_name,
                                icon_bytes,
                                normalized_color,
                                icon_size,
                            );

                            ui.painter().image(
                                icon.id(),
                                rounded_rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }

                        ui.allocate_ui_with_layout(
                            vec2(100.0, 0.0),
                            Layout::top_down(Align::Center),
                            |ui| {
                                ui.add(
                                    Label::new(
                                        RichText::new(file.name.clone()).color(
                                            current_theme
                                                .semantic
                                                .text_primary
                                                .to_color()
                                                .linear_multiply(anim),
                                        ),
                                    )
                                    .wrap()
                                    .selectable(false),
                                );
                            },
                        );

                        ui.add_space(10.0);

                        if should_repaint {
                            ui.request_repaint();
                        }
                    });

                    Grid::new("file_info_grid")
                        .num_columns(2)
                        .striped(true)
                        .max_col_width(100.0)
                        .min_col_width(50.0)
                        .show(ui, |ui| {
                            let row = |ui: &mut Ui, label: &str, value: &str| {
                                ui.label_ns(label);
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label_ns(value);
                                });
                                ui.end_row();
                            };

                            row(ui, "Modificado:", &format_date(file.modified));

                            if file.is_dir() {
                                row(ui, &i18n.t("right_sidebar.ext"), "Folder");
                            } else {
                                row(
                                    ui,
                                    &i18n.t("right_sidebar.ext"),
                                    &format!("{:?}", file.extension),
                                );
                            };

                            row(ui, &i18n.t("right_sidebar.size"), &format_size(file.size));

                            let extended_info = if state
                                .extended_info
                                .is_calculating(&file.full_path)
                            {
                                None
                            } else if state.extended_info.is_calculated(&file.full_path) {
                                match state.extended_info_manager.info_map.write() {
                                    Ok(mut map) => map.get(&file.full_path).cloned(),
                                    Err(e) => {
                                        error!(
                                            "Ha ocurrio un error intentando leer ExtendedInfo: {}",
                                            e
                                        );
                                        None
                                    }
                                }
                            } else {
                                state
                                    .extended_info_manager
                                    .cache_manager
                                    .get_cached_extended_info(&file.full_path)
                                    .map(|c| ExtendedInfo {
                                        owner: c.owner,
                                        group_name: c.group_name,
                                        symlink_target: c.symlink_target,
                                        dimensions: c.dimensions,
                                        git_status: c.git_status,
                                    })
                            };

                            if let Some(extended_info) = extended_info {
                                if let Some(owner) = extended_info.owner {
                                    row(ui, &i18n.t("right_sidebar.owner"), &owner);
                                }
                                if let Some(group_name) = extended_info.group_name {
                                    row(ui, &i18n.t("right_sidebar.group_name"), &group_name);
                                }
                                if let Some(symlink_target) = extended_info.symlink_target {
                                    row(
                                        ui,
                                        &i18n.t("right_sidebar.type"),
                                        &symlink_target.display().to_string(),
                                    );
                                }
                                if let Some(dimensions) = extended_info.dimensions {
                                    let w_str = dimensions.0.to_string();
                                    let h_str = dimensions.1.to_string();
                                    row(
                                        ui,
                                        &i18n.t("right_sidebar.dimensions"),
                                        &format!("{}x{}", w_str, h_str),
                                    );
                                }
                                if let Some(git_status) = extended_info.git_status {
                                    let status_debug = format!("{:?}", git_status);
                                    row(ui, &i18n.t("right_sidebar.git_status"), &status_debug);
                                }
                            }
                        });
                });
        }

        ui.set_clip_rect(prev_clip);
    }
}
