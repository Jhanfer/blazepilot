use crate::{
    core::{
        blaze_state::{BlazeCoreState, LayoutMode, NewItemType, ViewMode},
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::{
            bus_structs::{SureTo, UiEvent},
            event_bus::with_event_bus,
        },
        system::trash_manager::manager::get_backend,
    },
    ui::{blaze_ui_state::BlazeUiState, icons_cache::icons, themes::colors::*},
};
use egui::{
    containers::Frame, pos2, vec2, Align, Color32, CornerRadius, CursorIcon, Layout, Margin, Rect,
    Sense, Stroke, Ui,
};
use std::sync::Arc;
use tracing::warn;

fn render_tag_button(ui: &mut Ui, state: &mut BlazeCoreState) {
    let (rect_toggle, resp_toggle) = ui.allocate_exact_size(vec2(45.0, 25.0), Sense::click());

    if resp_toggle.clicked() {
        state.view_mode = match &state.view_mode {
            ViewMode::Normal(layout) => ViewMode::Tags(layout.to_owned()),
            ViewMode::Tags(layout) => ViewMode::Normal(layout.to_owned()),
        };
    }

    let anim = ui.animate_bool(
        "view_mode_toggle".into(),
        state.view_mode == ViewMode::Tags(LayoutMode::Row)
            || state.view_mode == ViewMode::Tags(LayoutMode::Grid),
    );

    let bg_color = COLOR_ACCENT_PURPLE;

    ui.painter().rect_filled(rect_toggle, 20.0, bg_color);

    let padding = 3.0;
    let radius = (rect_toggle.height() / 2.0) - padding;
    let x_left = rect_toggle.min.x + padding + radius;
    let x_right = rect_toggle.max.x - padding - radius;
    let cx = x_left + (x_right - x_left) * anim;
    let cy = rect_toggle.center().y;

    ui.painter()
        .circle_filled(pos2(cx, cy), radius, Color32::WHITE);
}

pub fn tools(
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    files: &[Arc<FileEntry>],
    ui: &mut Ui,
) {
    Frame::new()
        .fill(COLOR_BG_CONTAINER)
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
            color: COLOR_ACCENT_GLOW,
        })
        .show(ui, |ui| {
            let toolbar_height = 25.0;
            ui.set_min_height(toolbar_height);
            ui.set_max_height(toolbar_height);

            ui.horizontal_centered(|ui| {
                ui.visuals_mut().button_frame = false;

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.visuals_mut().button_frame = false;

                    let (icon_plus_fol, icon_bytes_plus_fol) =
                        ("plus-folder", icons::ICON_PLUS_FOLDER);

                    let icon_size = vec2(18.0, 18.0);
                    let (icon_rect, new_fol) = ui.allocate_exact_size(icon_size, Sense::click());
                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    let icon = ui_state.icon_cache.get_or_load(
                        ui,
                        icon_plus_fol,
                        icon_bytes_plus_fol,
                        Color32::GRAY,
                        icon_size,
                    );

                    let plus_color = if new_fol.hovered() {
                        COLOR_TOOLS_PRIMARY
                    } else {
                        COLOR_TOOLS_SECONDARY
                    };

                    ui.painter().image(
                        icon.id(),
                        rounded_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        plus_color,
                    );

                    if new_fol.clicked() {
                        state.creating_new = Some(NewItemType::Folder);
                        state.new_item_buffer = "nueva carpeta".to_string();
                    }

                    let (icon_plus_file, icon_bytes_plus_file) =
                        ("plus-file", icons::ICON_PLUS_FILE);

                    let icon_size = vec2(18.0, 18.0);
                    let (icon_rect, new_file) = ui.allocate_exact_size(icon_size, Sense::click());
                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    let icon = ui_state.icon_cache.get_or_load(
                        ui,
                        icon_plus_file,
                        icon_bytes_plus_file,
                        Color32::GRAY,
                        icon_size,
                    );

                    let plus_color = if new_file.hovered() {
                        COLOR_TOOLS_PRIMARY
                    } else {
                        COLOR_TOOLS_SECONDARY
                    };

                    ui.painter().image(
                        icon.id(),
                        rounded_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        plus_color,
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
                    ("scissors", icons::ICON_SCISSORS)
                } else {
                    ("scissors-disable", icons::ICON_SCISSORS_DISABLE)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, cut_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_cut,
                    icon_bytes_cut,
                    Color32::GRAY,
                    icon_size,
                );

                let sissors_color = if cut_resp.hovered() && has_selection {
                    COLOR_TOOLS_PRIMARY
                } else {
                    COLOR_TOOLS_SECONDARY
                };

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    sissors_color,
                );

                if cut_resp.clicked() {
                    state.cut(files);
                }

                let (icon_copy, icon_bytes_copy) = if has_selection {
                    ("copy", icons::ICON_COPY)
                } else {
                    ("copy-disable", icons::ICON_COPY_DISABLE)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, cop_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_copy,
                    icon_bytes_copy,
                    Color32::GRAY,
                    icon_size,
                );

                let clip_color = if cop_resp.hovered() && has_selection {
                    COLOR_TOOLS_PRIMARY
                } else {
                    COLOR_TOOLS_SECONDARY
                };

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    clip_color,
                );

                if cop_resp.clicked() {
                    state.copy(files);
                }

                let (icon_paste, icon_bytes_paste) = if has_clipboard {
                    ("clipboard", icons::ICON_CLIPBOARD)
                } else {
                    ("clipboard-disable", icons::ICON_CLIPBOARD_DISABLE)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, pas_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_paste,
                    icon_bytes_paste,
                    Color32::GRAY,
                    icon_size,
                );

                let pas_color = if pas_resp.hovered() && has_clipboard {
                    COLOR_TOOLS_PRIMARY
                } else {
                    COLOR_TOOLS_SECONDARY
                };

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    pas_color,
                );

                if pas_resp.clicked() {
                    let cwd = state.cwd.clone();
                    state.paste(cwd);
                }

                let (icon_trash, icon_bytes_trash) = if has_selection {
                    ("trash", icons::ICON_TRASH)
                } else {
                    ("trash-disable", icons::ICON_TRASH_DISABLED)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, del_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_trash,
                    icon_bytes_trash,
                    Color32::GRAY,
                    icon_size,
                );

                let del_color = if del_resp.hovered() && has_selection {
                    COLOR_TOOLS_PRIMARY
                } else {
                    COLOR_TOOLS_SECONDARY
                };

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    del_color,
                );

                if del_resp.clicked() && has_selection {
                    let cwd = state.cwd.clone();
                    let is_in_trash = get_backend().etched_in_trash_path(&cwd);

                    if is_in_trash {
                        let tab_id = state.active_id;
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
                            .map(|(_, f)| (Arc::from(f.name.to_owned()), f.full_path.to_owned()))
                            .collect();
                        state.move_to_trash(items);
                    }
                }

                ui.add_space(8.0);

                let (icon_name, icon_bytes) = if state.select_all_mode {
                    ("deselect", icons::ICON_DESELECT)
                } else {
                    ("select-all", icons::ICON_SELECTALL)
                };

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, select_resp) = ui.allocate_exact_size(icon_size, Sense::click());
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

                let sel_color = if select_resp.hovered() {
                    COLOR_TOOLS_PRIMARY
                } else {
                    COLOR_TOOLS_SECONDARY
                };

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    sel_color,
                );

                if select_resp.clicked() {
                    state.toggle_select_all(files.len());
                }

                ui.separator();

                let (icon_refresh, icon_bytes_refresh) = ("refresh", icons::ICON_REFRESH);

                let icon_size = vec2(18.0, 18.0);
                let (icon_rect, refresh_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                let rounded_rect = Rect::from_min_max(
                    pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                    pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                );

                let icon = ui_state.icon_cache.get_or_load(
                    ui,
                    icon_refresh,
                    icon_bytes_refresh,
                    Color32::GRAY,
                    icon_size,
                );

                let ref_color = if refresh_resp.hovered() {
                    COLOR_TOOLS_PRIMARY
                } else {
                    COLOR_TOOLS_SECONDARY
                };

                ui.painter().image(
                    icon.id(),
                    rounded_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    ref_color,
                );

                if refresh_resp.clicked() {
                    state.refresh();
                }

                ui.separator();

                render_tag_button(ui, state);

                ui.separator();

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.visuals_mut().button_frame = false;

                    let is_hidden = with_configs(|c| c.get_show_hidden_files());

                    let (icon_refresh, icon_bytes_refresh) = if is_hidden {
                        ("eye", icons::ICON_EYE)
                    } else {
                        ("eye-closed", icons::ICON_EYE_CLOSED)
                    };

                    let icon_size = vec2(18.0, 18.0);
                    let (icon_rect, hidd_resp) = ui.allocate_exact_size(icon_size, Sense::click());
                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    let icon = ui_state.icon_cache.get_or_load(
                        ui,
                        icon_refresh,
                        icon_bytes_refresh,
                        Color32::GRAY,
                        icon_size,
                    );

                    let hidd_color = if hidd_resp.hovered() {
                        COLOR_TOOLS_PRIMARY
                    } else {
                        COLOR_TOOLS_SECONDARY
                    };

                    ui.painter().image(
                        icon.id(),
                        rounded_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        hidd_color,
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
