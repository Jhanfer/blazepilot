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

use crate::core::blaze_state::BlazeCoreState;
use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::bootstrap::configs::platform::linux::conf_structs::{
    OrderingDirection, OrderingKind, OrderingMode,
};
use crate::core::files::blaze_motor::motor_structs::FileEntry;
use crate::core::system::extended_info::extended_info_manager::ExtendedInfo;
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::icons_cache::icons;
use crate::ui::icons_cache::thumbnails::thumbnails_manager::Thumbnail;
use crate::ui::modules::utilities::{ensure_min_lightness, resolve_icon};
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use crate::utils::formating::{format_date, format_size};
use egui::{
    pos2, vec2, Align, Button, Color32, ColorImage, CornerRadius, Frame, Grid, Label, Layout,
    Margin, Panel, Rect, RichText, Sense, Stroke, TextEdit, TextureOptions, Ui,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::error;

pub fn render_ordering_btn<F>(
    ui: &mut Ui,
    ui_state: &mut BlazeUiState,
    icons: (&str, &[u8], &[u8]),
    mode: OrderingMode,
    mut callback: F,
) where
    F: FnMut(),
{
    let current_theme = with_theme(|t| t.current());

    let icon_size = vec2(18.0, 18.0);

    let (icon_rect, resp) = ui.allocate_exact_size(icon_size, Sense::click());

    let (icon_name, icon_bytes) = match mode.kind {
        OrderingKind::Name => match mode.direction {
            OrderingDirection::Asc => (format!("{}-asc", icons.0), icons.1),
            OrderingDirection::Desc => (format!("{}-desc", icons.0), icons.2),
        },
        OrderingKind::Size => match mode.direction {
            OrderingDirection::Asc => (format!("{}-asc", icons.0), icons.1),
            OrderingDirection::Desc => (format!("{}-desc", icons.0), icons.2),
        },
        OrderingKind::Date => match mode.direction {
            OrderingDirection::Asc => (format!("{}-asc", icons.0), icons.1),
            OrderingDirection::Desc => (format!("{}-desc", icons.0), icons.2),
        },
    };

    if resp.clicked() {
        callback();
    }

    let mut color = current_theme.tools_secondary.to_color();

    if resp.hovered() {
        ui.set_cursor_icon(egui::CursorIcon::PointingHand);
        color = current_theme.tools_primary.to_color();
    }

    let rounded_rect = Rect::from_min_max(
        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
    );
    let icon =
        ui_state
            .icon_cache
            .get_or_load(ui, &icon_name, icon_bytes, Color32::GRAY, icon_size);

    ui.painter().image(
        icon.id(),
        rounded_rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        color,
    );
}

pub fn sidebar_right_component(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    files: &[Arc<FileEntry>],
) {
    let current_theme = with_theme(|t| t.current());

    let i18n = with_configs(|c| c.get_i18n());
    let custom_frame = Frame::NONE
        .fill(current_theme.bg_main.to_color())
        .inner_margin(Margin {
            left: 0,
            right: 15,
            top: 0,
            bottom: 0,
        });

    Panel::right("info_panel")
        .resizable(false)
        .frame(custom_frame)
        .show_separator_line(false)
        .show_inside(ui, |ui| {

            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(current_theme.bg_panel.to_color())
                .corner_radius(CornerRadius::same(20))
                .stroke(Stroke {
                    width: 0.5,
                    color: current_theme.accent_glow.to_color(),
                })
                .show(ui, |ui| {

                    Frame::NONE
                        .fill(current_theme.bg_main.to_color())
                        .stroke(
                            Stroke::new(
                                0.5,
                                current_theme.accent_glow.to_color()
                            )
                        )
                        .corner_radius(CornerRadius::same(99))
                        .inner_margin(Margin::symmetric(10, 6))
                        .show(ui, |ui| {

                            ui.horizontal(|ui| {
                                let icon_size = vec2(14.0, 14.0);
                                let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
                                let (icon_name, icon_bytes) = ("search", icons::ICON_SEARCH);
                                let rounded_rect = Rect::from_min_max(
                                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                                );
                                let icon = ui_state.icon_cache.get_or_load(
                                    ui,
                                    icon_name,
                                    icon_bytes,
                                    Color32::GRAY,
                                    icon_size,
                                );

                                ui.painter().image(
                                    icon.id(),
                                    rounded_rect,
                                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                    Color32::WHITE,
                                );

                                let mut search = state.search_filter.clone();

                                let response = ui.add(
                                    TextEdit::singleline(&mut search)
                                        .id("search_bar".into())
                                        .frame(Frame::NONE)
                                        .desired_width(150.0)
                                        .hint_text(i18n.t("search.placeholder"))
                                );

                                if response.changed() {
                                    state.set_search(search);
                                    response.request_focus();
                                }

                                if ui
                                    .add_enabled(!state.search_filter.is_empty(), Button::new("X"))
                                    .clicked()
                                {
                                    state.clean_search();
                                }

                            });
                    });

                    ui.add_space(5.0);

                    let frame_width = 110.0;
                    ui.with_layout(
                        Layout::top_down(Align::Center),
                        |ui| {
                            Frame::NONE
                                .fill(current_theme.border_panel.to_color())
                                .stroke(
                                    Stroke::new(0.5, current_theme.accent_glow.to_color())
                                )
                                .corner_radius(CornerRadius::same(99))
                                .inner_margin(Margin::symmetric(10, 6))
                                .show(ui, |ui| {
                                    ui.set_max_height(25.0);
                                    ui.set_width(frame_width);

                                    ui.horizontal_centered(|ui| {
                                        let icon_size_f = 18.0;
                                        let button_count = 3;
                                        let available = frame_width - 20.0;
                                        let total_buttons = icon_size_f * button_count as f32;
                                        let spacing = (available - total_buttons) / (button_count as f32 + 1.0);

                                        ui.add_space(spacing);

                                        let mut current_ordering = with_configs(|c| c.get_ordering_mode());

                                        let mut needs_update = false;

                                        render_ordering_btn(
                                            ui,
                                            ui_state,
                                            (
                                                "alphabet",
                                                icons::ICON_SORT_LETTERS_UP,
                                                icons::ICON_SORT_LETTERS_DOWN
                                            ),
                                            current_ordering,
                                            || {
                                                current_ordering = match (current_ordering.kind, current_ordering.direction) {
                                                    (OrderingKind::Name, OrderingDirection::Asc) => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Name,
                                                            direction: OrderingDirection::Desc,
                                                        }
                                                    },
                                                    (OrderingKind::Name, OrderingDirection::Desc) => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Name,
                                                            direction: OrderingDirection::Asc,
                                                        }
                                                    },
                                                    _ => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Name,
                                                            direction: OrderingDirection::Asc,
                                                        }
                                                    }
                                                };
                                                needs_update = true;
                                            }
                                        );

                                        ui.add_space(spacing);

                                        render_ordering_btn(
                                            ui,
                                            ui_state,
                                            (
                                                "size",
                                                icons::ICON_SORT_SHAPES_UP,
                                                icons::ICON_SORT_SHAPES_DOWN
                                            ),
                                            current_ordering,
                                            || {
                                                current_ordering = match (current_ordering.kind, current_ordering.direction) {
                                                    (OrderingKind::Size, OrderingDirection::Asc) => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Size,
                                                            direction: OrderingDirection::Desc,
                                                        }
                                                    },
                                                    (OrderingKind::Size, OrderingDirection::Desc) => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Size,
                                                            direction: OrderingDirection::Asc,
                                                        }
                                                    },
                                                    _ => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Size,
                                                            direction: OrderingDirection::Asc,
                                                        }
                                                    }
                                                };
                                                needs_update = true;
                                            }
                                        );

                                        ui.add_space(spacing);

                                        render_ordering_btn(
                                            ui,
                                            ui_state,
                                            (
                                                "date",
                                                icons::ICON_CALENDAR_UP,
                                                icons::ICON_CALENDAR_DOWN
                                            ),
                                            current_ordering,
                                            || {
                                                current_ordering = match (current_ordering.kind, current_ordering.direction) {
                                                    (OrderingKind::Date,
                                                        OrderingDirection::Asc) => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Date,
                                                            direction: OrderingDirection::Desc,
                                                        }
                                                    },
                                                    (OrderingKind::Date, OrderingDirection::Desc) => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Date,
                                                            direction: OrderingDirection::Asc,
                                                        }
                                                    },
                                                    _ => {
                                                        OrderingMode {
                                                            kind: OrderingKind::Date,
                                                            direction: OrderingDirection::Asc,
                                                        }
                                                    }
                                                };
                                                needs_update = true;
                                            }
                                        );

                                        ui.add_space(spacing);

                                        if needs_update {
                                            with_configs(|c| c.set_ordering_mode(current_ordering));
                                            state.refresh();
                                        }
                                    });
                            });
                    });

                    ui.add_space(10.0);

                    let has_selection = (0..files.len()).any(|i| state.is_selected(i));
                    let target = if has_selection { 1.0 } else { 0.0 };
                    let anim_id = ui.id().with("file_info_expand");

                    let anim = ui.animate_value_with_time(
                        anim_id,
                        target,
                        0.10
                    );

                    if anim > 0.001 {
                        if anim != target {
                            ui.request_repaint();
                        }

                        let max_height = 350.0;
                        let current_height = max_height * anim;

                        let available = ui.available_rect_before_wrap();
                        let clip_rect = Rect::from_min_size(
                            available.min,
                            vec2(available.width(), current_height)
                        );

                        let prev_clip = ui.clip_rect();
                        ui.set_clip_rect(clip_rect);

                        let animated_stroke = Stroke::new(
                            (0.5 * anim).clamp(0.0, 0.5),
                            current_theme.accent_glow.to_color().linear_multiply(anim)
                        );

                        let animated_radius = CornerRadius::same((20.0 * anim) as u8);

                        if let Some(first_selected_idx) = (0..files.len()).find(|&i| state.is_selected(i)) {

                            Frame::NONE
                                .outer_margin(Margin::symmetric(1, 1))
                                .inner_margin(Margin::same(10))
                                .stroke(animated_stroke)
                                .fill(current_theme.bg_main.to_color())
                                .corner_radius(animated_radius)
                                .show(ui, |ui| {

                                    let file = &files[first_selected_idx];

                                    ui.add(
                                        Label::new(
                                            RichText::new(i18n.t("right_sidebar.info"))
                                                .heading()
                                                .color(current_theme.text_primary.to_color().linear_multiply(anim))
                                        )
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

                                        let thumbnail_snapshot: HashMap<Arc<Path>, Thumbnail> = {
                                            match ui_state.thumbnail_manager.thumb_map.try_write() {
                                                Ok(mut guard) => {
                                                    let mut map = HashMap::new();
                                                    if let Some(thumb) = guard.get(&file.full_path).cloned() {
                                                        map.insert(file.full_path.clone(), thumb);
                                                    }
                                                    map
                                                }
                                                Err(_) => {
                                                    ui_state.needs_repaint = true;
                                                    HashMap::new()
                                                }
                                            }
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
                                            let (icon_name, icon_bytes, color) = resolve_icon(file, &ui_state.color_snapshot);

                                            let rounded_rect = Rect::from_min_max(
                                                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                                                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                                            );

                                            let icon = ui_state.icon_cache.get_or_load(
                                                ui,
                                                &icon_name,
                                                icon_bytes,
                                                Color32::GRAY,
                                                icon_size,
                                            );

                                            let normalized_color = ensure_min_lightness(color);

                                            ui.painter().image(
                                                icon.id(),
                                                rounded_rect,
                                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                                normalized_color.linear_multiply(anim),
                                            );
                                        }

                                        ui.allocate_ui_with_layout(
                                            vec2(100.0, 0.0),
                                            Layout::top_down(Align::Center),
                                            |ui| {
                                                ui.add(
                                                    Label::new(
                                                        RichText::new(file.name.clone())
                                                            .color(current_theme.text_primary.to_color().linear_multiply(anim))
                                                    ).wrap()
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
                                        .show(ui, |ui| {

                                            let row = |ui: &mut Ui, label: &str, value: &str| {
                                                ui.label(label);
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.label(value);
                                                });
                                                ui.end_row();
                                            };

                                            row(ui, "Modificado:", &format_date(file.modified));

                                            if file.is_dir() {
                                                row(ui, &i18n.t("right_sidebar.ext"), "Folder");
                                            } else {
                                                row(ui, &i18n.t("right_sidebar.ext"), &format!("{:?}", file.extension));
                                            };

                                            row(ui, &i18n.t("right_sidebar.size"), &format_size(file.size));


                                            let extended_info =
                                                if state.calculating_extended_info.contains(&file.full_path) {
                                                    None
                                                } else if state.calculated_extended_info.contains(&file.full_path) {
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
                                                    row(ui, &i18n.t("right_sidebar.type"), &symlink_target.display().to_string());
                                                }
                                                if let Some(dimensions) = extended_info.dimensions {
                                                    let w_str = dimensions.0.to_string();
                                                    let h_str = dimensions.1.to_string();
                                                    row(ui, &i18n.t("right_sidebar.dimensions"), &format!("{}x{}", w_str, h_str));
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
                });
        });
}
