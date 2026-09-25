use crate::{
    core::{
        blaze_state::BlazeCoreState,
        blaze_state::state_structs::{LayoutMode, NewItemType, ViewMode},
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::{
            bus_structs::{SureTo, UiEvent},
            event_bus::with_event_bus,
        },
        system::trash_manager::manager::get_backend,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::icons::*,
        modules::utilities::ensure_min_lightness,
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{
    Align, Color32, CornerRadius, CursorIcon, Layout, Margin, Rect, Sense, Stroke, Ui,
    containers::Frame, pos2, vec2,
};
use std::sync::Arc;
use tracing::warn;

fn render_tag_button(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    let current_theme = with_theme(|t| t.current());

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
        Stroke::new(1.0, current_theme.components.panel.border.to_color()),
        egui::StrokeKind::Outside,
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

    for (name, bytes, rect, is_active) in icons {
        let base_color = if is_active {
            current_theme.components.button.label_active.to_color()
        } else {
            current_theme.components.button.label_inactive.to_color()
        };

        let color = ensure_min_lightness(base_color);

        let icon = ui_state
            .icon_cache
            .get_or_load(ui, name, bytes, color, icon_size);

        ui.painter().image(
            icon.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

fn render_views_mode(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    let current_theme = with_theme(|t| t.current());

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

            let base_color = if is_active {
                current_theme.components.button.label_active.to_color()
            } else {
                current_theme.components.button.label_inactive.to_color()
            };

            let (new_name, new_bytes) = if is_list_button {
                match current_layout {
                    LayoutMode::Compact => ("layout-compact", ICON_LAYOUT_LIST_COMPACT),
                    LayoutMode::RowDetailed => ("layout-detailed", ICON_LAYOUT_LIST_DETAILED),
                    _ => ("layout-list", ICON_LAYOUT_LIST),
                }
            } else {
                (*name, *bytes)
            };

            let icon = ui_state.icon_cache.get_or_load(
                ui,
                new_name,
                new_bytes,
                ensure_min_lightness(base_color),
                icon_size,
            );

            let (rect, response) =
                ui.allocate_exact_size(icon_size + vec2(6.0, 6.0), Sense::click());

            if response.hovered() {
                ui.set_cursor_icon(CursorIcon::PointingHand);
                ui.painter()
                    .rect_filled(rect, 4.0, current_theme.semantic.bg_container.to_color());
            }

            if is_active {
                ui.painter().rect_filled(
                    rect,
                    4.0,
                    current_theme
                        .semantic
                        .accent
                        .to_color()
                        .linear_multiply(0.2),
                );
            }

            let icon_rect = Rect::from_center_size(rect.center(), icon_size);

            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

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

pub fn tools(state: &mut BlazeCoreState, ui_state: &mut BlazeUiState, ui: &mut Ui) {
    let path = state.cwd();
    let files: &[Arc<FileEntry>] = &state.get_files_for(&path);

    let current_theme = with_theme(|t| t.current());

    Frame::new()
        .fill(current_theme.components.tools.bg.to_color())
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
        .stroke(Stroke {
            width: 0.5,
            color: current_theme.components.panel.border.to_color(),
        })
        .show(ui, |ui| {
            let toolbar_height = 25.0;
            ui.set_min_height(toolbar_height);
            ui.set_max_height(toolbar_height);

            ui.horizontal_centered(|ui| {
                ui.visuals_mut().button_frame = false;

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.visuals_mut().button_frame = false;

                    let (icon_plus_fol, icon_bytes_plus_fol) = ("plus-folder", ICON_PLUS_FOLDER);

                    let icon_size = vec2(18.0, 18.0);
                    let (icon_rect, new_fol) = ui.allocate_exact_size(icon_size, Sense::click());
                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    let plus_color = if new_fol.hovered() {
                        current_theme.components.tools.label_hover.to_color()
                    } else {
                        current_theme.components.tools.label_active.to_color()
                    };

                    let icon = ui_state.icon_cache.get_or_load(
                        ui,
                        icon_plus_fol,
                        icon_bytes_plus_fol,
                        ensure_min_lightness(plus_color),
                        icon_size,
                    );

                    ui.painter().image(
                        icon.id(),
                        rounded_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    if new_fol.clicked() {
                        state.creating_new = Some(NewItemType::Folder);
                        state.new_item_buffer = "nueva carpeta".to_string();
                    }

                    let (icon_plus_file, icon_bytes_plus_file) = ("plus-file", ICON_PLUS_FILE);

                    let icon_size = vec2(18.0, 18.0);
                    let (icon_rect, new_file) = ui.allocate_exact_size(icon_size, Sense::click());
                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    let plus_color = if new_file.hovered() {
                        current_theme.components.tools.label_hover.to_color()
                    } else {
                        current_theme.components.tools.label_active.to_color()
                    };

                    let icon = ui_state.icon_cache.get_or_load(
                        ui,
                        icon_plus_file,
                        icon_bytes_plus_file,
                        ensure_min_lightness(plus_color),
                        icon_size,
                    );

                    ui.painter().image(
                        icon.id(),
                        rounded_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    if new_file.clicked() {
                        state.creating_new = Some(NewItemType::File);
                        state.new_item_buffer = "nuevo archivo".to_string();
                    }

                    ui.separator();

                    if new_fol.hovered() || new_file.hovered() {
                        ui.set_cursor_icon(CursorIcon::PointingHand);
                    }
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

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, cut_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let sissors_color = if cut_resp.hovered() && has_selection {
                    current_theme.components.tools.label_hover.to_color()
                } else {
                    current_theme.components.tools.label_active.to_color()
                };

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_cut,
                    icon_bytes_cut,
                    ensure_min_lightness(sissors_color),
                    icon_size,
                );

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                if cut_resp.clicked() && has_selection {
                    state.cut(files);
                }

                let (icon_copy, icon_bytes_copy) = if has_selection {
                    ("copy", ICON_COPY)
                } else {
                    ("copy-disable", ICON_COPY_DISABLE)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, cop_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let clip_color = if cop_resp.hovered() && has_selection {
                    current_theme.components.tools.label_hover.to_color()
                } else {
                    current_theme.components.tools.label_active.to_color()
                };

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_copy,
                    icon_bytes_copy,
                    ensure_min_lightness(clip_color),
                    icon_size,
                );

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                if cop_resp.clicked() && has_selection {
                    state.copy(files);
                }

                let (icon_paste, icon_bytes_paste) = if has_clipboard {
                    ("clipboard", ICON_CLIPBOARD)
                } else {
                    ("clipboard-disable", ICON_CLIPBOARD_DISABLE)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, pas_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let pas_color = if pas_resp.hovered() && has_clipboard {
                    current_theme.components.tools.label_hover.to_color()
                } else {
                    current_theme.components.tools.label_active.to_color()
                };

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_paste,
                    icon_bytes_paste,
                    ensure_min_lightness(pas_color),
                    icon_size,
                );

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                if pas_resp.clicked() && has_clipboard {
                    let cwd = state.cwd();
                    state.paste(cwd);
                }

                let (icon_trash, icon_bytes_trash) = if has_selection {
                    ("trash", ICON_TRASH)
                } else {
                    ("trash-disable", ICON_TRASH_DISABLED)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, del_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let del_color = if del_resp.hovered() && has_selection {
                    current_theme.components.tools.label_hover.to_color()
                } else {
                    current_theme.components.tools.label_active.to_color()
                };

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_trash,
                    icon_bytes_trash,
                    ensure_min_lightness(del_color),
                    icon_size,
                );

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                if del_resp.clicked() && has_selection {
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

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, select_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let sel_color = if select_resp.hovered() {
                    current_theme.components.tools.label_hover.to_color()
                } else {
                    current_theme.components.tools.label_active.to_color()
                };

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_name,
                    icon_bytes,
                    ensure_min_lightness(sel_color),
                    icon_size,
                );

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                if select_resp.clicked() {
                    state.toggle_select_all(files.len());
                }

                ui.separator();

                let (icon_refresh, icon_bytes_refresh) = ("refresh", ICON_REFRESH);

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, refresh_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let ref_color = if refresh_resp.hovered() {
                    current_theme.components.tools.label_hover.to_color()
                } else {
                    current_theme.components.tools.label_active.to_color()
                };

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_refresh,
                    icon_bytes_refresh,
                    ensure_min_lightness(ref_color),
                    icon_size,
                );

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                if refresh_resp.clicked() {
                    state.refresh();
                }

                ui.separator();

                ui.vertical(|ui| {
                    render_tag_button(ui, state, ui_state);
                });

                ui.separator();

                ui.vertical(|ui| {
                    render_views_mode(ui, state, ui_state);
                });

                ui.separator();

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.visuals_mut().button_frame = false;

                    let is_hidden = with_configs(|c| c.get_show_hidden_files());

                    let (icon_refresh, icon_bytes_refresh) = if is_hidden {
                        ("eye", ICON_EYE)
                    } else {
                        ("eye-closed", ICON_EYE_CLOSED)
                    };

                    let icon_size = vec2(18.0, 18.0);
                    let (icon_rect, hidd_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    let hidd_color = if hidd_resp.hovered() {
                        current_theme.components.tools.label_hover.to_color()
                    } else {
                        current_theme.components.tools.label_active.to_color()
                    };

                    let icon = ui_state.icon_cache.get_or_load(
                        ui,
                        icon_refresh,
                        icon_bytes_refresh,
                        ensure_min_lightness(hidd_color),
                        icon_size,
                    );

                    ui.painter().image(
                        icon.id(),
                        rounded_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    if hidd_resp.clicked() {
                        with_configs(|c| {
                            c.set_show_hidden_files(!is_hidden);
                        });
                        state.refresh();
                    };

                    ui.separator();

                    if hidd_resp.hovered() {
                        ui.set_cursor_icon(CursorIcon::PointingHand);
                    }
                });

                let show_hand = select_resp.hovered()
                    || refresh_resp.hovered()
                    || (has_selection
                        && (del_resp.hovered() || cop_resp.hovered() || cut_resp.hovered()))
                    || (has_clipboard && pas_resp.hovered());

                if show_hand {
                    ui.set_cursor_icon(CursorIcon::PointingHand);
                }
            });
        });
}
