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
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::custom_components::UiExt;
use crate::ui::icons_cache::icons;
use crate::ui::modules::sidebar_right_component::components::animated_section::render_animated_section;
use crate::ui::modules::sidebar_right_component::components::clipboard_manager::render_clipboard_menu;
use crate::ui::modules::sidebar_right_component::rightbar_state::{
    MIN_PANEL_WIDTH_TOLERANCE, PANEL_TAB_WIDTH,
};
use crate::ui::themes::platform::structs::BlazeAppTheme;
use crate::ui::themes::theme_manager::with_theme;
use egui::{
    Align, Area, Color32, CornerRadius, Frame, Id, Layout, Margin, Panel, Rect, Sense, Stroke, Ui,
    Vec2, pos2,
};
use std::sync::Arc;

fn get_sort_icon(kind: OrderingKind, current_order: OrderingMode) -> (&'static str, &'static [u8]) {
    match kind {
        OrderingKind::Name => match current_order.direction {
            OrderingDirection::Asc => ("alphabet-asc", icons::ICON_SORT_LETTERS_UP),
            OrderingDirection::Desc => ("alphabet-desc", icons::ICON_SORT_LETTERS_DOWN),
        },
        OrderingKind::Size => match current_order.direction {
            OrderingDirection::Asc => ("size-asc", icons::ICON_SORT_SHAPES_UP),
            OrderingDirection::Desc => ("size-desc", icons::ICON_SORT_SHAPES_DOWN),
        },
        OrderingKind::Date => match current_order.direction {
            OrderingDirection::Asc => ("date-asc", icons::ICON_CALENDAR_UP),
            OrderingDirection::Desc => ("date-desc", icons::ICON_CALENDAR_DOWN),
        },
    }
}

fn render_expanded_panel(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    current_theme: Arc<BlazeAppTheme>,
    current_order: OrderingMode,
) {
    let i18n = with_configs(|c| c.get_i18n());
    let palette = current_theme.sidebar_right.palette(&current_theme.global);
    let custom_frame = Frame::NONE
        .fill(current_theme.global.bg_main())
        .inner_margin(Margin {
            left: 0,
            right: 15,
            top: 0,
            bottom: 0,
        });

    let mut inner_width: f32 = 0.0;

    ui.memory_mut(|mem| {
        mem.data
            .insert_persisted(Id::new("info_panel"), current_order.rightpanel_state.width);
    });

    Panel::right("info_panel")
        .resizable(true)
        .frame(custom_frame)
        .show_separator_line(false)
        .show(ui, |ui| {
            inner_width = ui.available_width();

            let search_id: Id = Id::new("search_bar");
            let has_focus = ui.memory(|mem| mem.has_focus(search_id));
            Frame::NONE
                .inner_margin(Margin::same(10))
                .fill(palette.bg_panel)
                .corner_radius(CornerRadius::same(20))
                .stroke(Stroke::new(
                    current_theme.global.panel_stroke_width,
                    palette.border_panel,
                ))
                .show(ui, |ui| {
                    let search_color = if has_focus {
                        palette.border_focus
                    } else {
                        palette.border_component
                    };

                    let (icon_name, icon_bytes) = ("search", icons::ICON_SEARCH);

                    let mut search = state.current_search_filter().to_string();

                    let response = ui
                        .blaze_input(&i18n.t("search.placeholder"), Some("Ctrl + K"), &mut search)
                        .id(search_id)
                        .hint_background_color(palette.accent_soft)
                        .hint_stroke_color(palette.accent_text)
                        .hint_text_color(palette.accent_text)
                        .size((200.0, 30.0))
                        .radius(20)
                        .text_color(palette.fg_primary)
                        .background_color(palette.bg_elevated)
                        .stroke_color(search_color)
                        .x_color(palette.accent_text)
                        .icon(icon_name, icon_bytes)
                        .icon_color(if has_focus {
                            palette.accent_text
                        } else {
                            palette.icon_secondary
                        })
                        .stroke_width(current_theme.global.components_stroke_width)
                        .show(|| {
                            state.clean_search();
                        });

                    if response.changed() {
                        state.set_search(search);
                        response.request_focus();
                    }

                    ui.add_space(5.0);

                    let frame_width = 110.0;
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        Frame::NONE
                            .fill(Color32::TRANSPARENT)
                            .inner_margin(Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.set_max_height(25.0);
                                ui.set_width(frame_width);

                                ui.horizontal_centered(|ui| {
                                    let icon_size_f = 20.0;
                                    let button_count = 3;
                                    let available = frame_width - 20.0;
                                    let total_buttons = icon_size_f * button_count as f32;
                                    let spacing =
                                        (available - total_buttons) / (button_count as f32 + 1.0);

                                    ui.add_space(spacing);

                                    let mut c_mode = with_configs(|c| c.get_ordering_mode());
                                    let mut needs_update = false;

                                    let (i, b) = get_sort_icon(OrderingKind::Name, c_mode);
                                    if ui
                                        .blaze_button("")
                                        .id(i.to_string())
                                        .size(Vec2::splat(icon_size_f))
                                        .icon(i, b)
                                        .icon_size(icon_size_f)
                                        .background_color(Color32::TRANSPARENT)
                                        .hover_background_color(Color32::TRANSPARENT)
                                        .stroke_color(Color32::TRANSPARENT)
                                        .icon_color(palette.icon_primary)
                                        .hover_icon_color(palette.icon_hover)
                                        .show()
                                        .clicked()
                                    {
                                        c_mode = c_mode.toggle(OrderingKind::Name);
                                        needs_update = true;
                                    }

                                    ui.add_space(spacing);

                                    let (i, b) = get_sort_icon(OrderingKind::Size, c_mode);
                                    if ui
                                        .blaze_button("")
                                        .id(i.to_string())
                                        .size(Vec2::splat(icon_size_f))
                                        .icon(i, b)
                                        .icon_size(icon_size_f)
                                        .background_color(Color32::TRANSPARENT)
                                        .hover_background_color(Color32::TRANSPARENT)
                                        .stroke_color(Color32::TRANSPARENT)
                                        .icon_color(palette.icon_primary)
                                        .hover_icon_color(palette.icon_hover)
                                        .show()
                                        .clicked()
                                    {
                                        c_mode = c_mode.toggle(OrderingKind::Size);
                                        needs_update = true;
                                    }

                                    ui.add_space(spacing);

                                    let (i, b) = get_sort_icon(OrderingKind::Date, c_mode);
                                    if ui
                                        .blaze_button("")
                                        .id(i.to_string())
                                        .size(Vec2::splat(icon_size_f))
                                        .icon(i, b)
                                        .icon_size(icon_size_f)
                                        .background_color(Color32::TRANSPARENT)
                                        .hover_background_color(Color32::TRANSPARENT)
                                        .stroke_color(Color32::TRANSPARENT)
                                        .icon_color(palette.icon_primary)
                                        .hover_icon_color(palette.icon_hover)
                                        .show()
                                        .clicked()
                                    {
                                        c_mode = c_mode.toggle(OrderingKind::Date);
                                        needs_update = true;
                                    }

                                    if needs_update {
                                        with_configs(|c| c.set_ordering_mode(c_mode));
                                        state.refresh();
                                    }
                                });
                            });
                    });

                    render_animated_section(ui, state, ui_state, current_theme.clone());
                });

            render_clipboard_menu(ui, state, ui_state, current_theme.clone());
        });

    let stroke_total = 3.0 * current_theme.global.panel_stroke_width;

    let min_width = MIN_PANEL_WIDTH_TOLERANCE + stroke_total;

    let prev_width = current_order.rightpanel_state.width;
    let was_open = prev_width >= min_width;
    let is_growing = inner_width > prev_width;

    if inner_width < min_width {
        if was_open && !is_growing {
            with_configs(|c| {
                let mut o = c.get_ordering_mode();
                o.rightpanel_state.collapsed = true;
                c.set_ordering_mode(o);
            });
        } else if !was_open && is_growing {
            with_configs(|c| {
                let mut o = c.get_ordering_mode();
                o.rightpanel_state.collapsed = false;
                o.rightpanel_state.width = min_width;
                c.set_ordering_mode(o);
            });
        }
    } else {
        with_configs(|c| {
            let mut o = c.get_ordering_mode();
            o.rightpanel_state.width = inner_width;
            o.rightpanel_state.collapsed = false;
            c.set_ordering_mode(o);
        });
    }
}

fn render_collapsed_tab(ui: &mut Ui, current_theme: Arc<BlazeAppTheme>) {
    let area_id = Id::new("right_tab");
    let hover_state_id = Id::new("right_tab_hover_state");
    let palette = current_theme.sidebar_right.palette(&current_theme.global);

    let mut is_hovered = ui.data(|data| data.get_temp::<bool>(hover_state_id).unwrap_or(false));

    Panel::right("info_panel_tab")
        .exact_size(PANEL_TAB_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .show(ui, |ui| {
            let rect = ui.max_rect();
            let area_height = 20.0;

            let visual_pos = pos2(rect.center().x - 30.0, rect.center().y - area_height / 2.0);

            let area_response = Area::new(area_id)
                .fixed_pos(visual_pos)
                .sense(Sense::all())
                .show(ui, |ui| {
                    let area_rect = ui.max_rect();
                    let frame_width = if is_hovered { 20.0 } else { 5.0 };

                    Frame::NONE
                        .inner_margin(Margin::same(10))
                        .fill(if is_hovered {
                            palette.bg_hover
                        } else {
                            palette.bg_panel
                        })
                        .stroke(Stroke::new(
                            current_theme.global.panel_stroke_width,
                            palette.border_panel,
                        ))
                        .corner_radius(CornerRadius::same(20))
                        .show(ui, |ui| {
                            ui.set_width(frame_width);
                            ui.set_height(area_height / 2.0);

                            let area_center = area_rect.center();

                            let icon_size = Vec2::splat(18.0);

                            let t_icon_rect = Rect::from_center_size(area_center, icon_size);
                            let (icon_n, icon_b) = ("right_sidebar", icons::ICON_SIDEBAR_RIGHT);

                            ui.draw_icon((icon_n, icon_b))
                                .size(icon_size)
                                .tint(if is_hovered {
                                    palette.icon_hover
                                } else {
                                    palette.icon_secondary
                                })
                                .show_at(t_icon_rect);
                        });
                })
                .response;

            if area_response.clicked() {
                with_configs(|c| {
                    let mut current_ordering = c.get_ordering_mode();
                    current_ordering.rightpanel_state.collapsed = false;
                    c.set_ordering_mode(current_ordering);
                });
            }

            let new_hovered = area_response.hovered();

            if new_hovered != is_hovered {
                is_hovered = new_hovered;
                ui.data_mut(|data| {
                    data.insert_temp(hover_state_id, is_hovered);
                });
            }
        });
}

pub fn sidebar_right_component(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
) {
    let current_theme = with_theme(|t| t.current());
    let current_order = with_configs(|c| c.get_ordering_mode());

    if current_order.rightpanel_state.collapsed {
        render_collapsed_tab(ui, current_theme);
    } else {
        render_expanded_panel(ui, state, ui_state, current_theme, current_order);
    }
}
