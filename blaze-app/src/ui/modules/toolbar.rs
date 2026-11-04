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

use crate::{
    core::{
        blaze_state::BlazeCoreState,
        bootstrap::configs::config_manager::with_configs,
        runtime::{bus_structs::UiEvent, event_bus::with_event_bus},
        system::clipboard_text::text_clipboard::with_text_clipboard,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        custom_components::{UiExt, text_edit::BlazeTextEdit},
        icons_cache::icons::{self},
        modules::tabs_view::tab_views::tabs,
        themes::theme_manager::with_theme,
    },
};
use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Frame, Id, Key, Layout, Margin, Panel, Rect,
    RichText, Sense, Ui, UiBuilder, vec2,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub fn toolbar_component(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    let current_theme = with_theme(|t| t.current());
    let i18n = with_configs(|c| c.get_i18n());

    Panel::top("toolbar")
        .frame(
            Frame::new()
                .fill(current_theme.global.bg_main())
                .inner_margin(Margin {
                    left: 10,
                    right: 10,
                    top: 10,
                    bottom: 10,
                }),
        )
        .show_separator_line(false)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add_space(5.0);

            let bar_width = ui.available_width();

            // ----------------------
            // Tabs
            // ----------------------
            let bar_rect = tabs(ui, state, ui_state, bar_width);

            let route_h = 35.0;
            let horizontal_padding = 10.0;

            let inner_size = vec2(
                bar_rect.width() - horizontal_padding * 2.0,
                bar_rect.height(),
            );

            let inner_rect =
                Rect::from_min_size(bar_rect.min + vec2(horizontal_padding, 0.0), inner_size);

            let mut bar_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(inner_rect)
                    .layout(Layout::left_to_right(egui::Align::Center)),
            );

            bar_ui.set_clip_rect(inner_rect);
            bar_ui.set_min_size(inner_size);

            bar_ui.spacing_mut().item_spacing.x = 6.0;

            bar_ui.allocate_ui_with_layout(
                inner_size,
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add_space(1.0);
                    let back_active = state.can_go_back();

                    if ui
                        .blaze_button("")
                        .id("back_btn")
                        .set_is_active(back_active)
                        .size((route_h, route_h))
                        .icon_color(current_theme.top_bar.icon_primary())
                        .hover_icon_color(
                            current_theme
                                .top_bar
                                .icon_hover_on_color(current_theme.top_bar.bg_elevated()),
                        )
                        .icon("<", icons::ICON_ARROW_LEFT)
                        .text_color(current_theme.global.text_secondary())
                        .background_color(current_theme.top_bar.bg_elevated())
                        .stroke_color(current_theme.top_bar.component_border())
                        .stroke_width(current_theme.global.components_stroke_width)
                        .radius(25.0)
                        .show()
                        .clicked()
                    {
                        state.back()
                    }

                    let forward_active = state.can_go_forward();

                    if ui
                        .blaze_button("")
                        .id("forward_btn")
                        .set_is_active(forward_active)
                        .size((route_h, route_h))
                        .icon_color(current_theme.top_bar.icon_primary())
                        .icon(">", icons::ICON_ARROW_RIGHT)
                        .text_color(current_theme.global.text_secondary())
                        .background_color(current_theme.top_bar.bg_elevated())
                        .stroke_color(current_theme.top_bar.component_border())
                        .stroke_width(current_theme.global.components_stroke_width)
                        .radius(25.0)
                        .show()
                        .clicked()
                    {
                        state.forward()
                    }

                    let up_active = state.can_go_up();

                    if ui
                        .blaze_button("")
                        .id("up_btn")
                        .set_is_active(up_active)
                        .size((route_h, route_h))
                        .icon_color(current_theme.top_bar.icon_primary())
                        .icon("UP", icons::ICON_ARROW_UP)
                        .text_color(current_theme.global.text_secondary())
                        .background_color(current_theme.top_bar.bg_elevated())
                        .stroke_color(current_theme.top_bar.component_border())
                        .stroke_width(current_theme.global.components_stroke_width)
                        .radius(25.0)
                        .show()
                        .clicked()
                    {
                        state.up()
                    }

                    if ui
                        .blaze_button("")
                        .id("settings_btn")
                        .size((route_h, route_h))
                        .icon_color(current_theme.top_bar.icon_primary())
                        .icon("settings", icons::ICON_SETTINGS)
                        .text_color(current_theme.global.text_secondary())
                        .background_color(current_theme.top_bar.bg_elevated())
                        .stroke_color(current_theme.top_bar.component_border())
                        .stroke_width(current_theme.global.components_stroke_width)
                        .radius(25.0)
                        .show()
                        .clicked()
                    {
                        let tab_id = state.active_id();
                        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
                        dispatcher.send(UiEvent::OpenConfigs).ok();
                    }

                    let cwd = state.cwd();
                    Frame::new()
                        .fill(current_theme.top_bar.bg_elevated())
                        .corner_radius(CornerRadius::same(20))
                        .inner_margin(0)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(route_h);

                            if state.editing_cwd {
                                ui.horizontal_centered(|ui| {
                                    ui.add_space(15.0);

                                    let available = ui.available_width();

                                    let route_id = Id::new("route_bar");
                                    let has_focus = ui.memory(|mem| mem.has_focus(route_id));

                                    let mut route = state.cwd_input.clone();

                                    let response = ui.add(
                                        BlazeTextEdit::singleline(&mut route)
                                            .hint_text(i18n.t("route.placeholder"))
                                            .desired_width(available - 40.0)
                                            .id(route_id),
                                    );

                                    if !has_focus {
                                        response.request_focus();
                                    }

                                    if response.changed() {
                                        state.cwd_input = route.clone();
                                    }

                                    let enter_pressed = ui.input(|i| i.key_pressed(Key::Enter));
                                    let escape_pressed = ui.input(|i| i.key_pressed(Key::Escape));

                                    if escape_pressed {
                                        // Cancela la edicion y restaura el path original
                                        state.cwd_input = state.cwd().display().to_string();
                                        state.editing_cwd = false;
                                    } else if enter_pressed || response.lost_focus() {
                                        let new_path = PathBuf::from(&route);
                                        let path: Arc<Path> = new_path.into();
                                        if !state.is_miller() {
                                            state.navigate_to(path);
                                        } else {
                                            state.miller_reset_to(path);
                                        }
                                        state.editing_cwd = false;
                                    }
                                });
                            } else {
                                ui.horizontal_centered(|ui| {
                                    ui.add_space(15.0);

                                    let components: Vec<_> = cwd.components().collect();

                                    let reserve_for_clipboard = 40.0;
                                    let buget =
                                        (ui.available_width() - reserve_for_clipboard).max(0.0);

                                    let sep_font = FontId::proportional(16.0);
                                    let sep_color = current_theme.global.text_secondary();
                                    let sep_w = ui
                                        .painter()
                                        .layout_no_wrap(">".to_owned(), sep_font, sep_color)
                                        .rect
                                        .width()
                                        + ui.spacing().item_spacing.x;

                                    let crumb_font = FontId::proportional(14.0);
                                    let crumb_extra = 20.0;

                                    let per_crumb_max = f32::INFINITY;

                                    let mut crumbs: Vec<(usize, String, f32)> = vec![];
                                    for (i, c) in components.iter().enumerate() {
                                        let name = c.as_os_str().to_string_lossy().to_string();
                                        if name.is_empty() {
                                            continue;
                                        }

                                        let w = ui
                                            .painter()
                                            .layout_no_wrap(
                                                name.clone(),
                                                crumb_font.clone(),
                                                current_theme.global.text_primary(),
                                            )
                                            .rect
                                            .width()
                                            + crumb_extra;

                                        crumbs.push((i, name, w));
                                    }

                                    if crumbs.is_empty() {
                                        return;
                                    }

                                    let n = crumbs.len();

                                    let mut visible_from = 0_usize;
                                    {
                                        let mut total = 0.0_f32;
                                        for k in (0..n).rev() {
                                            let (_, _, w) = &crumbs[k];
                                            let need = *w + if k < n - 1 { sep_w } else { 0.0 };
                                            if total + need > buget && k != n - 1 {
                                                break;
                                            }
                                            total += need;
                                            visible_from = k;
                                        }
                                    }

                                    if visible_from > 0 {
                                        let ellipsis_w = ui
                                            .painter()
                                            .layout_no_wrap(
                                                "...".to_owned(),
                                                crumb_font.clone(),
                                                current_theme.global.text_primary(),
                                            )
                                            .rect
                                            .width()
                                            + crumb_extra
                                            + sep_w;

                                        let mut total = ellipsis_w;
                                        for (k, _) in
                                            crumbs.iter().enumerate().take(n).skip(visible_from)
                                        {
                                            let (_, _, w) = &crumbs[k];
                                            total += *w + if k < n - 1 { sep_w } else { 0.0 };
                                        }

                                        while total > buget && visible_from < n - 1 {
                                            let (_, _, w) = &crumbs[visible_from];
                                            total += *w + sep_w;
                                            visible_from += 1;
                                        }
                                    }

                                    let mut current_path = PathBuf::new();
                                    for (k, _, _) in crumbs.iter().take(visible_from) {
                                        let (i, _, _) = &crumbs[*k];
                                        current_path.push(components[*i]);
                                    }

                                    if visible_from > 0 {
                                        ui.label_ns(
                                            RichText::new("...")
                                                .color(current_theme.global.text_secondary())
                                                .size(16.0),
                                        );
                                    }

                                    for (k, _) in
                                        crumbs.iter().enumerate().take(n).skip(visible_from)
                                    {
                                        let (i, raw_name, w) = &crumbs[k];
                                        let c = components[*i];
                                        current_path.push(c);

                                        let is_last = *i == components.len() - 1;

                                        let display_name = if !is_last && *w > per_crumb_max {
                                            let max_chars = ((per_crumb_max
                                                / (crumb_font.size * 0.555))
                                                as usize)
                                                .max(3);

                                            let chars: Vec<char> = raw_name.chars().collect();
                                            if chars.len() > max_chars {
                                                let s: String = chars
                                                    [..max_chars.saturating_sub(1)]
                                                    .iter()
                                                    .collect();
                                                format!("{s}..")
                                            } else {
                                                raw_name.clone()
                                            }
                                        } else {
                                            raw_name.clone()
                                        };

                                        let mut response = ui
                                            .blaze_button(&display_name)
                                            .size((*w, 28.0))
                                            .text_color(if is_last {
                                                current_theme.global.text_primary()
                                            } else {
                                                current_theme.global.text_secondary()
                                            })
                                            .background_color(if is_last {
                                                current_theme.top_bar.accent()
                                            } else {
                                                Color32::TRANSPARENT
                                            })
                                            .stroke_color(Color32::TRANSPARENT)
                                            .radius(20.0)
                                            .show();

                                        if display_name != *raw_name {
                                            response = response.on_hover_text(raw_name);
                                        }

                                        if response.double_clicked() {
                                            state.editing_cwd = true;
                                        } else if response.clicked() && !is_last {
                                            if !state.is_miller() {
                                                state.navigate_to(current_path.to_owned().into());
                                            } else {
                                                state.miller_reset_to(
                                                    current_path.to_owned().into(),
                                                );
                                            }
                                        }

                                        if !is_last {
                                            ui.label_ns(
                                                RichText::new("›")
                                                    .color(current_theme.global.text_secondary())
                                                    .size(16.0),
                                            );
                                        }
                                    }
                                });
                            }

                            let remaining = ui.available_width() - 40.0;
                            if remaining > 0.0 {
                                let empty_space =
                                    ui.allocate_response(vec2(remaining, route_h), Sense::click());
                                if empty_space.hovered() {
                                    ui.set_cursor_icon(CursorIcon::Text);
                                }
                                if empty_space.clicked() || empty_space.double_clicked() {
                                    state.editing_cwd = true;
                                }
                            }

                            if ui
                                .blaze_button("")
                                .size((20.0, 20.0))
                                .icon("clipboard", icons::ICON_CLIPBOARD)
                                .icon_size(13.0_f32)
                                .text_color(current_theme.global.text_secondary())
                                .background_color(current_theme.top_bar.bg_panel())
                                .stroke_color(Color32::TRANSPARENT)
                                .radius(25.0)
                                .show()
                                .clicked()
                            {
                                with_text_clipboard(|c| {
                                    c.copy(cwd.to_string_lossy().to_string());
                                });
                            }
                        });
                },
            );
        });
}
