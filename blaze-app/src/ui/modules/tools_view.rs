use crate::{
    core::{
        blaze_state::{
            BlazeCoreState,
            state_structs::{LayoutMode, NewItemType, ViewMode},
        },
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::{
            bus_structs::{SureTo, UiEvent},
            event_bus::with_event_bus,
        },
        system::trash_manager::manager::get_backend,
    },
    ui::{
        custom_components::UiExt,
        icons_cache::icons::*,
        modules::utilities::{ensure_min_lightness, shade_color},
        themes::theme_manager::with_theme,
    },
};
use egui::{
    Align, Color32, CornerRadius, CursorIcon, Layout, Margin, Rect, Sense, Stroke, Ui,
    containers::Frame, pos2, vec2,
};
use std::sync::Arc;
use tracing::warn;

fn render_tag_button(ui: &mut Ui, state: &mut BlazeCoreState) {
    let current_theme = with_theme(|t| t.current());
    let palette = current_theme.main_area.palette(&current_theme.global);

    let (container_rect, resp_toggle) = ui.allocate_exact_size(vec2(50.0, 25.0), Sense::click());

    if resp_toggle.clicked() {
        let new_mode = match &state.view_mode {
            ViewMode::Normal(layout) => ViewMode::Tags(layout.to_owned()),
            ViewMode::Tags(layout) => ViewMode::Normal(layout.to_owned()),
        };

        state.set_view_mode(new_mode);
    }

    if resp_toggle.hovered() {
        ui.set_cursor_icon(CursorIcon::PointingHand);
    }

    ui.painter()
        .rect_filled(container_rect, 20.0, Color32::TRANSPARENT);

    let view_mode_id = ui.make_persistent_id("view_mode_toggle_state");
    let animation_id = ui.make_persistent_id("view_mode_toggle_animation");

    let is_tags = matches!(state.view_mode, ViewMode::Tags(_));

    let previous_is_tags = ui.memory(|memory| memory.data.get_temp::<bool>(view_mode_id));

    if let Some(previous_is_tags) = previous_is_tags
        && previous_is_tags != is_tags
    {
        ui.memory_mut(|mem| {
            let current = mem.data.get_temp::<bool>(animation_id).unwrap_or(false);
            mem.data.insert_temp(animation_id, !current);
        });
    }

    ui.memory_mut(|memory| {
        memory.data.insert_temp(view_mode_id, is_tags);
    });

    let animation_target =
        ui.memory(|mem| mem.data.get_temp::<bool>(animation_id).unwrap_or(false));

    let anim = ui.animate_bool_with_time(animation_id, animation_target, 0.2);

    let padding = 3.0;
    let radius = (container_rect.height() / 2.0) - padding;

    let x_left = container_rect.min.x + padding + radius;
    let x_right = container_rect.max.x - padding - radius;
    let cx = x_left + (x_right - x_left) * anim;

    let stretch = anim * (1.0 - anim) * 4.0;
    let capsule_width = radius * 2.0 * (1.0 + stretch * 0.8);
    let capsule_height = radius * 2.0;

    let capsule_rect = Rect::from_center_size(
        pos2(cx, container_rect.center().y),
        vec2(capsule_width, capsule_height),
    );

    ui.painter().rect_stroke(
        capsule_rect,
        radius,
        Stroke::new(current_theme.global.panel_stroke_width, palette.accent),
        egui::StrokeKind::Middle,
    );

    let icon_size = vec2(16.0, 16.0);

    let icon1_rect = Rect::from_min_size(
        pos2(
            x_left - 7.5,
            container_rect.center().y - (icon_size.y / 2.0),
        ),
        icon_size,
    );

    let icon2_rect = Rect::from_min_size(
        pos2(
            x_right - 6.5,
            container_rect.center().y - (icon_size.y / 2.0),
        ),
        icon_size,
    );

    let icon1_name = match (&state.view_mode, is_tags) {
        (ViewMode::Normal(LayoutMode::Row), _) => ("layout-list", ICON_LAYOUT_LIST),
        (ViewMode::Normal(LayoutMode::RowDetailed), _) => {
            ("layout-detailed", ICON_LAYOUT_LIST_DETAILED)
        }
        (ViewMode::Normal(LayoutMode::Grid), _) => ("layout-grid", ICON_LAYOUT_GRID),
        (ViewMode::Normal(LayoutMode::Miller), _) => ("layout-miller", ICON_LAYOUT_MILLER),
        (ViewMode::Normal(LayoutMode::Compact), _) => ("layout-compact", ICON_LAYOUT_LIST_COMPACT),
        (ViewMode::Tags(LayoutMode::Row), _) => ("layout-row", ICON_LAYOUT_LIST),
        (ViewMode::Tags(LayoutMode::RowDetailed), _) => {
            ("layout-detailed", ICON_LAYOUT_LIST_DETAILED)
        }
        (ViewMode::Tags(LayoutMode::Grid), _) => ("layout-grid", ICON_LAYOUT_GRID),
        (ViewMode::Tags(LayoutMode::Miller), _) => ("layout-miller", ICON_LAYOUT_MILLER),
        (ViewMode::Tags(LayoutMode::Compact), _) => ("layout-compact", ICON_LAYOUT_LIST_COMPACT),
    };

    let icons = [
        (icon1_name.0, icon1_name.1, icon1_rect, anim == 0.0),
        ("tag", ICON_TAG, icon2_rect, anim == 1.0),
    ];

    let color = palette.accent;

    for (name, bytes, rect, is_active) in icons {
        let base_color = if is_active { color } else { shade_color(color) };

        let color = ensure_min_lightness(base_color);

        ui.draw_icon((name, bytes))
            .size(icon_size)
            .tint(color)
            .show_at(rect);
    }
}

fn render_views_mode(ui: &mut Ui, state: &mut BlazeCoreState) {
    let current_theme = with_theme(|t| t.current());
    let palette = current_theme.main_area.palette(&current_theme.global);

    let layouts: &[(&str, &[u8], LayoutMode)] = &[
        ("layout-list", ICON_LAYOUT_LIST, LayoutMode::Row),
        ("layout-grid", ICON_LAYOUT_GRID, LayoutMode::Grid),
        ("layout-miller", ICON_LAYOUT_MILLER, LayoutMode::Miller),
    ];

    let current_layout = match &state.view_mode {
        ViewMode::Normal(l) | ViewMode::Tags(l) => l.clone(),
    };

    ui.horizontal(|ui| {
        for (name, bytes, layout) in layouts {
            let is_list_button = *layout == LayoutMode::Row;
            let is_active = if is_list_button {
                matches!(
                    current_layout,
                    LayoutMode::Row | LayoutMode::Compact | LayoutMode::RowDetailed
                )
            } else {
                current_layout == *layout
            };
            let icon_size = vec2(18.0, 18.0);

            let color = palette.accent;

            let base_color = if is_active { color } else { shade_color(color) };

            let (new_name, new_bytes) = if is_list_button {
                match current_layout {
                    LayoutMode::Compact => ("layout-compact", ICON_LAYOUT_LIST_COMPACT),
                    LayoutMode::RowDetailed => ("layout-detailed", ICON_LAYOUT_LIST_DETAILED),
                    _ => ("layout-list", ICON_LAYOUT_LIST),
                }
            } else {
                (*name, *bytes)
            };

            let (rect, response) =
                ui.allocate_exact_size(icon_size + vec2(6.0, 6.0), Sense::click());

            if response.hovered() {
                ui.set_cursor_icon(CursorIcon::PointingHand);
                ui.painter().rect_filled(rect, 4.0, palette.bg_hover);
            }

            if is_active {
                ui.painter().rect_filled(rect, 4.0, palette.bg_selected);
            }

            let icon_rect = Rect::from_center_size(rect.center(), icon_size);

            ui.draw_icon((new_name, new_bytes))
                .size(icon_size)
                .tint(ensure_min_lightness(base_color))
                .show_at(icon_rect);

            if response.clicked() {
                let new_layout = if is_list_button {
                    match current_layout {
                        LayoutMode::Row => LayoutMode::Compact,
                        LayoutMode::Compact => LayoutMode::RowDetailed,
                        LayoutMode::RowDetailed => LayoutMode::Row,
                        _ => LayoutMode::Row,
                    }
                } else {
                    layout.to_owned()
                };

                let new_mode = match &state.view_mode {
                    ViewMode::Normal(_) => ViewMode::Normal(new_layout),
                    ViewMode::Tags(_) => ViewMode::Tags(new_layout),
                };

                state.set_view_mode(new_mode);
            }
        }
    });
}

pub fn tools(state: &mut BlazeCoreState, ui: &mut Ui) {
    let path = state.cwd();
    let files: &[Arc<FileEntry>] = &state.get_files_for(&path);

    let current_theme = with_theme(|t| t.current());
    let palette = current_theme.main_area.palette(&current_theme.global);

    Frame::new()
        .fill(palette.bg_elevated)
        .inner_margin(Margin {
            left: 15,
            right: 15,
            top: 15,
            bottom: 15,
        })
        .corner_radius(CornerRadius {
            nw: 20,
            ne: 20,
            sw: 0,
            se: 0,
        })
        .stroke(Stroke::new(
            current_theme.global.panel_stroke_width,
            palette.border_panel,
        ))
        .show(ui, |ui| {
            let toolbar_height = 25.0;
            ui.set_min_height(toolbar_height);
            ui.set_max_height(toolbar_height);

            ui.horizontal_centered(|ui| {
                ui.visuals_mut().button_frame = false;

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.visuals_mut().button_frame = false;

                    let btn_name = "plus-folder";
                    let id = ui.id().with(btn_name);
                    if ui
                        .blaze_button("")
                        .id(id)
                        .size((15.0, 15.0))
                        .icon_size(19_f32)
                        .radius(25)
                        .icon(btn_name, ICON_PLUS_FOLDER)
                        .icon_color(palette.icon_primary)
                        .hover_icon_color(palette.icon_hover)
                        .background_color(Color32::TRANSPARENT)
                        .hover_background_color(Color32::TRANSPARENT)
                        .stroke_color(Color32::TRANSPARENT)
                        .show()
                        .clicked()
                    {
                        state.creating_new = Some(NewItemType::Folder);
                        state.new_item_buffer = "nueva carpeta".to_string();
                    }

                    let btn_name = "plus-file";
                    let id = ui.id().with(btn_name);
                    if ui
                        .blaze_button("")
                        .id(id)
                        .size((15.0, 15.0))
                        .icon_size(19_f32)
                        .radius(25)
                        .icon(btn_name, ICON_PLUS_FILE)
                        .icon_color(palette.icon_primary)
                        .hover_icon_color(palette.icon_hover)
                        .background_color(Color32::TRANSPARENT)
                        .hover_background_color(Color32::TRANSPARENT)
                        .stroke_color(Color32::TRANSPARENT)
                        .show()
                        .clicked()
                    {
                        state.creating_new = Some(NewItemType::File);
                        state.new_item_buffer = "nuevo archivo".to_string();
                    }

                    ui.separator();
                });

                let has_selection = state.selected_count(files.len()) > 0;

                let has_clipboard = match state.clipboard.clipboard_has_files() {
                    Ok(has_files) => has_files,
                    Err(e) => {
                        warn!("Error en el clipboard: {}", e);
                        false
                    }
                };

                let (icon_cut, icon_bytes_cut) = if has_selection {
                    ("scissors", ICON_SCISSORS)
                } else {
                    ("scissors-disable", ICON_SCISSORS_DISABLE)
                };

                let id = ui.id().with(icon_cut);
                if ui
                    .blaze_button("")
                    .id(id)
                    .set_is_active(has_selection)
                    .size((15.0, 15.0))
                    .icon_size(19_f32)
                    .radius(25)
                    .icon(icon_cut, icon_bytes_cut)
                    .icon_color(palette.icon_primary)
                    .hover_icon_color(palette.icon_hover)
                    .background_color(Color32::TRANSPARENT)
                    .hover_background_color(Color32::TRANSPARENT)
                    .stroke_color(Color32::TRANSPARENT)
                    .show()
                    .clicked()
                {
                    state.cut(files);
                }

                let (icon_copy, icon_bytes_copy) = if has_selection {
                    ("copy", ICON_COPY)
                } else {
                    ("copy-disable", ICON_COPY_DISABLE)
                };

                let id = ui.id().with(icon_copy);
                if ui
                    .blaze_button("")
                    .id(id)
                    .set_is_active(has_selection)
                    .size((15.0, 15.0))
                    .icon_size(19_f32)
                    .radius(25)
                    .icon(icon_copy, icon_bytes_copy)
                    .icon_color(palette.icon_primary)
                    .hover_icon_color(palette.icon_hover)
                    .background_color(Color32::TRANSPARENT)
                    .hover_background_color(Color32::TRANSPARENT)
                    .stroke_color(Color32::TRANSPARENT)
                    .show()
                    .clicked()
                {
                    state.copy(files);
                }

                let (icon_paste, icon_bytes_paste) = if has_clipboard {
                    ("clipboard", ICON_CLIPBOARD)
                } else {
                    ("clipboard-disable", ICON_CLIPBOARD_DISABLE)
                };

                let id = ui.id().with(icon_paste);
                if ui
                    .blaze_button("")
                    .id(id)
                    .set_is_active(has_clipboard)
                    .size((15.0, 15.0))
                    .icon_size(19_f32)
                    .radius(25)
                    .icon(icon_paste, icon_bytes_paste)
                    .icon_color(palette.icon_primary)
                    .hover_icon_color(palette.icon_hover)
                    .background_color(Color32::TRANSPARENT)
                    .hover_background_color(Color32::TRANSPARENT)
                    .stroke_color(Color32::TRANSPARENT)
                    .show()
                    .clicked()
                {
                    let cwd = state.cwd();
                    state.paste(cwd);
                }

                let (icon_trash, icon_bytes_trash) = if has_selection {
                    ("trash", ICON_TRASH)
                } else {
                    ("trash-disable", ICON_TRASH_DISABLED)
                };

                let id = ui.id().with(icon_trash);
                if ui
                    .blaze_button("")
                    .id(id)
                    .set_is_active(has_selection)
                    .size((15.0, 15.0))
                    .icon_size(19_f32)
                    .radius(25)
                    .icon(icon_trash, icon_bytes_trash)
                    .icon_color(palette.icon_primary)
                    .hover_icon_color(palette.icon_hover)
                    .background_color(Color32::TRANSPARENT)
                    .hover_background_color(Color32::TRANSPARENT)
                    .stroke_color(Color32::TRANSPARENT)
                    .show()
                    .clicked()
                {
                    let cwd = state.cwd();
                    let is_in_trash = get_backend().etched_in_trash_path(&cwd);

                    if is_in_trash {
                        let tab_id = state.active_id();
                        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));

                        let sources = state.get_selected_paths(files);

                        dispatcher
                            .send(UiEvent::SureTo(SureTo::SureToDelete {
                                files: sources,
                                tab_id,
                            }))
                            .ok();
                    } else {
                        let items = files
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| state.is_selected(*index))
                            .map(|(_, f)| (f.name.clone(), f.full_path.to_owned()))
                            .collect();
                        state.move_to_trash(items);
                    }
                }

                ui.add_space(8.0);

                let (icon_name, icon_bytes) = if state.select_all_mode() {
                    ("deselect", ICON_DESELECT)
                } else {
                    ("select-all", ICON_SELECTALL)
                };

                let id = ui.id().with(icon_name);
                if ui
                    .blaze_button("")
                    .id(id)
                    .size((15.0, 15.0))
                    .icon_size(19_f32)
                    .radius(25)
                    .icon(icon_name, icon_bytes)
                    .icon_color(palette.icon_primary)
                    .hover_icon_color(palette.icon_hover)
                    .background_color(Color32::TRANSPARENT)
                    .hover_background_color(Color32::TRANSPARENT)
                    .stroke_color(Color32::TRANSPARENT)
                    .show()
                    .clicked()
                {
                    state.toggle_select_all(files.len());
                }

                ui.separator();

                let (icon_refresh, icon_bytes_refresh) = ("refresh", ICON_REFRESH);

                let id = ui.id().with(icon_refresh);
                if ui
                    .blaze_button("")
                    .id(id)
                    .size((15.0, 15.0))
                    .icon_size(19_f32)
                    .radius(25)
                    .icon(icon_refresh, icon_bytes_refresh)
                    .icon_color(palette.icon_primary)
                    .hover_icon_color(palette.icon_hover)
                    .background_color(Color32::TRANSPARENT)
                    .hover_background_color(Color32::TRANSPARENT)
                    .stroke_color(Color32::TRANSPARENT)
                    .show()
                    .clicked()
                {
                    state.refresh();
                }

                ui.separator();

                ui.vertical(|ui| {
                    render_tag_button(ui, state);
                });

                ui.separator();

                ui.vertical(|ui| {
                    render_views_mode(ui, state);
                });

                ui.separator();

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.visuals_mut().button_frame = false;

                    let is_hidden = with_configs(|c| c.get_show_hidden_files());

                    let (icon_eye, icon_bytes_eye) = if is_hidden {
                        ("eye", ICON_EYE)
                    } else {
                        ("eye-closed", ICON_EYE_CLOSED)
                    };

                    let id = ui.id().with(icon_eye);
                    if ui
                        .blaze_button("")
                        .id(id)
                        .size((15.0, 15.0))
                        .icon_size(19_f32)
                        .radius(25)
                        .icon(icon_eye, icon_bytes_eye)
                        .icon_color(palette.icon_primary)
                        .hover_icon_color(palette.icon_hover)
                        .background_color(Color32::TRANSPARENT)
                        .hover_background_color(Color32::TRANSPARENT)
                        .stroke_color(Color32::TRANSPARENT)
                        .show()
                        .clicked()
                    {
                        with_configs(|c| {
                            c.set_show_hidden_files(!is_hidden);
                        });
                        state.refresh();
                    }

                    ui.separator();
                });
            });
        });
}
