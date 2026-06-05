use crate::{
    core::{
        blaze_state::BlazeCoreState,
        bootstrap::configs::{
            config_manager::with_configs, platform::linux::conf_structs::OrderingMode,
        },
        files::blaze_motor::motor_structs::FileEntry,
        runtime::{
            bus_structs::{SureTo, UiEvent},
            event_bus::with_event_bus,
        },
        system::{
            extended_info::extended_info_manager::{ExtendedInfo, GitStatus},
            trash_manager::trash_manager::get_backend,
        },
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::thumbnails::thumbnails_manager::Thumbnail,
        modules::{
            custom_context_menu::context_state::ContextMenuKind,
            row_view::utilities::{git_dot_color, resolve_icon, text_color_for_git},
        },
        themes::colors::COLOR_MAIN_BUTTONS,
    },
    utils::formating::{format_date, format_size},
};
use egui::{
    pos2, scroll_area::ScrollSource, vec2, Button, Color32, ColorImage, CursorIcon, FontId, Id,
    Key, Modifiers, PointerButton, Rect, RichText, ScrollArea, Sense, TextEdit, TextureOptions, Ui,
};
use file_id::FileId;
use std::{collections::HashMap, path::Path, sync::Arc};
use tracing::info;

fn new_ff_logic(state: &mut BlazeCoreState, ui: &mut Ui) {
    if let Some(item_type) = state.creating_new.clone() {
        let creating_new_id = Id::new("creating_new");

        ui.horizontal(|ui| {
            let response =
                ui.add(TextEdit::singleline(&mut state.new_item_buffer).id(creating_new_id));

            if !state.focus_requested {
                response.request_focus();
                state.focus_requested = true;
            }

            if ui.input(|i| i.key_pressed(Key::Enter)) && !state.new_item_buffer.trim().is_empty() {
                state.create_new(item_type);
                state.creating_new = None;
                state.refresh();
                state.focus_requested = false;
            }

            if ui.input(|i| i.key_pressed(Key::Escape))
                || (response.lost_focus() && !ui.input(|i| i.key_pressed(Key::Enter)))
            {
                state.creating_new = None;
                state.focus_requested = false;
            }
        });
    }
}

fn handle_row_interactions(
    ui: &mut Ui,
    response: &egui::Response,
    i: usize,
    file: &Arc<FileEntry>,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    files: &[Arc<FileEntry>],
    content_rect: Rect,
    rect: Rect,
) {
    //Habilitar seleccion con rueda de ratón
    let middle_clicked = ui.input(|i| {
        i.pointer.button_pressed(PointerButton::Middle)
            && i.pointer
                .interact_pos()
                .map(|p| rect.contains(p))
                .unwrap_or(false)
    });

    if middle_clicked {
        state.resize_selection(files.len());
        let currently = state.is_selected(i);
        state.selection.set(i, !currently);
        state.last_selected_index = Some(i);

        if file.is_dir() {
            state.add_tab_from_file(&file.full_path);
        }
    }

    if response.drag_started_by(PointerButton::Primary) {
        if !state.is_selected(i) {
            state.deselect_all();
            state.resize_selection(files.len());
            state.selection.set(i, true);
            state.last_selected_index = Some(i);
            state.selection_anchor = Some(i);
        }
        state.row_view.is_dragging_files = true;
    }

    if response.dragged_by(PointerButton::Primary) {
        state.row_view.drag_ghost_pos = ui.input(|i| i.pointer.interact_pos());
    }

    if response.drag_stopped() && state.row_view.is_dragging_files {
        state.row_view.drag_ghost_pos = None;

        let drop_in_file_area = ui
            .input(|i| i.pointer.interact_pos())
            .map(|p| p.x <= content_rect.min.x + content_rect.width() * 0.80)
            .unwrap_or(false);

        if let Some(invalid_target) = state.row_view.drop_invalid_target.take() {
            info!("No es posible mover a {:?}", invalid_target);
        }

        if drop_in_file_area {
            let tab_id = state.active_id;
            let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
            let tab_id = state.motor.borrow_mut().active_tab().id;

            if let Some(target) = state.row_view.drop_target.take() {
                let sources = state.get_selected_paths(files);

                dispatcher
                    .send(UiEvent::SureTo(SureTo::SureToMove {
                        files: sources,
                        dest: target,
                        tab_id,
                    }))
                    .ok();
            } else {
                let cwd = state.cwd.clone();
                let sources = state.get_selected_paths(files);

                if sources.iter().all(|p| p.parent() == Some(&cwd)) {
                    return;
                }

                dispatcher
                    .send(UiEvent::SureTo(SureTo::SureToMove {
                        files: sources,
                        dest: cwd,
                        tab_id,
                    }))
                    .ok();
            }
        }

        state.row_view.is_dragging_files = false;
    }

    if response.secondary_clicked() {
        state.resize_selection(files.len());

        if ui.input(|i| i.modifiers.ctrl) {
            let currently = state.is_selected(i);
            state.selection.set(i, !currently);
            state.last_selected_index = Some(i);
        } else if !state.is_selected(i) {
            state.deselect_all();
            state.resize_selection(files.len());
            state.selection.set(i, true);
            state.last_selected_index = Some(i);
        }
    }

    let cwd = state.cwd.clone();
    let is_in_trash = get_backend().etched_in_trash_path(&cwd);

    if is_in_trash {
        let tab_id = state.active_id;
        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
        if response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(response);
            ui_state.context_menu_state.target_sender = Some(dispatcher);
            ui_state.context_menu_state.kind = ContextMenuKind::FileTrash;
        }
    } else {
        let tab_id = state.active_id;
        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
        if response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(response);
            ui_state.context_menu_state.target_file = Some(file.clone());
            ui_state.context_menu_state.target_sender = Some(dispatcher);
            ui_state.context_menu_state.kind = ContextMenuKind::FileNormal;
        }
    }
}

fn render_rename_field(
    ui: &mut Ui,
    file: &Arc<FileEntry>,
    state: &mut BlazeCoreState,
    rect: Rect,
    is_renaming: bool,
) {
    let rename_id = Id::new("rename_space");

    let response = ui.put(
        rect,
        TextEdit::singleline(&mut state.rename_buffer)
            .id(rename_id)
            .margin(vec2(0.0, 5.0))
            .font(FontId::default()),
    );

    if is_renaming && !response.has_focus() {
        response.request_focus();
    }

    if response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
        state.rename(&file.name);

        ui.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Enter);
        });

        state.renaming_file = None;
        return;
    }

    if ui.input(|i| i.key_pressed(Key::Escape)) {
        state.renaming_file = None;
        return;
    }

    if response.lost_focus() && !response.hovered() {
        state.renaming_file = None;
    }
}

pub fn new_render_scrollview(
    ui: &mut Ui,
    files: &Vec<Arc<FileEntry>>,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    row_height: f32,
    total_rows: usize,
    content_rect: Rect,
) {
    let i18n = with_configs(|c| c.get_i18n());

    ui_state.evict_thumbnail_cache_if_dir_changed(&state.cwd);
    ui_state.enforce_texture_cache_limit(500);

    let current_order = with_configs(|c| c.get_ordering_mode());

    // --- Header manual ---
    let id_name_w = ui.id().with("col_name_w");
    let id_date_w = ui.id().with("col_date_w");
    let id_size_w = ui.id().with("col_size_w");

    let available = ui.available_width() * 0.8;
    let name_w: f32 = ui
        .data(|d| d.get_temp(id_name_w).unwrap_or(available - 180.0))
        .max(80.0);
    let date_w: f32 = ui
        .data(|d| d.get_temp(id_date_w).unwrap_or(110.0_f32))
        .max(80.0);
    let size_w: f32 = ui
        .data(|d| d.get_temp(id_size_w).unwrap_or(70.0_f32))
        .max(60.0);

    let name_w = name_w.min(available - date_w - size_w).max(80.0);

    let header_height = 24.0;
    let (header_rect, _) = ui.allocate_exact_size(vec2(available, header_height), Sense::hover());

    let painter = ui.painter_at(header_rect);
    painter.rect_filled(header_rect, 0.0, Color32::from_rgb(20, 24, 28));

    // Botón Nombre
    let name_btn_rect = Rect::from_min_size(header_rect.min, vec2(name_w, header_height));
    let name_label = match current_order {
        OrderingMode::Az => format!("{} ↑", i18n.t("tools.name")),
        OrderingMode::Za => format!("{} ↓", i18n.t("tools.name")),
        _ => i18n.t("tools.name").to_string(),
    };
    if ui
        .put(
            name_btn_rect,
            Button::new(
                RichText::new(name_label)
                    .size(11.0)
                    .color(Color32::from_rgb(140, 140, 160)),
            )
            .frame(false),
        )
        .clicked()
    {
        with_configs(|c| {
            c.set_ordering_mode(match current_order {
                OrderingMode::Az => OrderingMode::Za,
                _ => OrderingMode::Az,
            });
        });
        state.refresh();
    }

    // Botón Modificado
    let date_x = header_rect.min.x + name_w;
    let date_btn_rect =
        Rect::from_min_size(pos2(date_x, header_rect.min.y), vec2(date_w, header_height));
    let date_label = match current_order {
        OrderingMode::DateAsc => format!("{} ↑", i18n.t("tools.modified")),
        OrderingMode::DateDesc => format!("{} ↓", i18n.t("tools.modified")),
        _ => i18n.t("tools.modified").to_string(),
    };
    if ui
        .put(
            date_btn_rect,
            Button::new(
                RichText::new(date_label)
                    .size(11.0)
                    .color(Color32::from_rgb(140, 140, 160)),
            )
            .frame(false),
        )
        .clicked()
    {
        with_configs(|c| {
            c.set_ordering_mode(match current_order {
                OrderingMode::DateAsc => OrderingMode::DateDesc,
                _ => OrderingMode::DateAsc,
            });
        });
        state.refresh();
    }

    // Botón Tamaño
    let size_x = date_x + date_w;
    let size_btn_rect =
        Rect::from_min_size(pos2(size_x, header_rect.min.y), vec2(size_w, header_height));
    let size_label = match current_order {
        OrderingMode::SizeAsc => format!("{} ↑", i18n.t("tools.size")),
        OrderingMode::SizeDesc => format!("{} ↓", i18n.t("tools.size")),
        _ => i18n.t("tools.size").to_string(),
    };
    if ui
        .put(
            size_btn_rect,
            Button::new(
                RichText::new(size_label)
                    .size(11.0)
                    .color(Color32::from_rgb(140, 140, 160)),
            )
            .frame(false),
        )
        .clicked()
    {
        with_configs(|c| {
            c.set_ordering_mode(match current_order {
                OrderingMode::SizeAsc => OrderingMode::SizeDesc,
                _ => OrderingMode::SizeAsc,
            });
        });
        state.refresh();
    }

    let handle_w = 4.0;
    {
        let (x, id_w, current_w) = (header_rect.min.x + name_w, id_name_w, name_w);
        let handle_rect = Rect::from_min_size(
            pos2(x - handle_w / 2.0, header_rect.min.y),
            vec2(handle_w, header_height),
        );
        let handle_id = ui.id().with(("resize_handle", id_w));
        let handle_response = ui.interact(handle_rect, handle_id, Sense::click_and_drag());

        if handle_response.hovered() || handle_response.dragged() {
            ui.set_cursor_icon(CursorIcon::ResizeColumn);
            painter.rect_filled(handle_rect, 0.0, Color32::from_rgb(100, 100, 140));
        } else {
            painter.rect_filled(handle_rect, 0.0, Color32::from_rgb(50, 50, 60));
        }

        if handle_response.dragged() {
            let new_w = (current_w + handle_response.drag_delta().x).max(80.0);
            ui.data_mut(|d| d.insert_temp(id_w, new_w));
        }
    }

    //Creacion de carpetas nuevas
    new_ff_logic(state, ui);

    if let Some(target_row) = state.pending_scroll_to.take() {
        if !files.is_empty() {
            let target_row = target_row.min(files.len() - 1);
            let row_top = target_row as f32 * row_height;
            let row_bottom = row_top + row_height;

            let viewport_top = state.scroll_offset;
            let viewport_bottop = state.scroll_offset + state.row_view.viewport_height;

            if row_top < viewport_top {
                state.scroll_offset = row_top;
            } else if row_bottom > viewport_bottop {
                state.scroll_offset = row_bottom - state.row_view.viewport_height;
            }
        }
    }

    let scroll_area = ScrollArea::vertical()
        .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
        .auto_shrink([false, false])
        .vertical_scroll_offset(state.scroll_offset);

    let scroll_output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
        let mut should_repaint = false;

        ui.spacing_mut().item_spacing.y = 0.0;

        state.row_view.first_visible = row_range.start;
        state.row_view.last_visible = row_range.end;

        let info_snapshot: HashMap<Arc<Path>, ExtendedInfo> = {
            match state.extended_info_manager.info_map.write() {
                Ok(mut map) => row_range
                    .clone()
                    .filter_map(|i| {
                        let path = &files[i].full_path;
                        map.get(path).map(|v| (path.clone(), v.clone()))
                    })
                    .collect::<HashMap<Arc<Path>, ExtendedInfo>>(),
                Err(_) => HashMap::new(),
            }
        };

        let color_snapshot: HashMap<FileId, Color32> = {
            match ui_state
                .folder_color_manager
                .cache_manager
                .color_cache
                .try_read()
            {
                Ok(guard) => row_range
                    .clone()
                    .filter_map(|i| {
                        files[i]
                            .unique_id
                            .as_ref()
                            .and_then(|id| guard.get(id).map(|c| (*id, c.color)))
                    })
                    .collect(),
                Err(_) => HashMap::new(),
            }
        };

        let thumbnail_snapshot: HashMap<Arc<Path>, Thumbnail> = {
            match ui_state.thumbnail_manager.thumb_map.try_write() {
                Ok(mut guard) => row_range
                    .clone()
                    .filter_map(|i| {
                        let p = &files[i].full_path;
                        guard.get(p).cloned().map(|t| (p.clone(), t))
                    })
                    .collect(),
                Err(_) => {
                    ui_state.needs_repaint = true;
                    HashMap::new()
                }
            }
        };

        for i in row_range.clone() {
            let file = &files[i];
            let is_renaming = state.renaming_file.as_deref() == Some(&file.full_path);

            if is_renaming {
                let (rect, _) = ui.allocate_exact_size(vec2(available, row_height), Sense::hover());

                if i == row_range.start {
                    state.row_view.scroll_area_origin_y =
                        rect.min.y + state.scroll_offset - (i as f32 * row_height);
                }

                render_rename_field(ui, file, state, rect, is_renaming);
                continue;
            }

            let (rect, response) =
                ui.allocate_exact_size(vec2(available, row_height), Sense::click_and_drag());

            // --- Corrección de la rubberband ---
            if i == row_range.start {
                state.row_view.scroll_area_origin_y =
                    rect.min.y + state.scroll_offset - (i as f32 * row_height);
            }

            // --- Selección y hover ---
            if response.hovered() {
                ui.set_cursor_icon(CursorIcon::PointingHand);
                ui.painter().rect_filled(rect, 5.0, COLOR_MAIN_BUTTONS);
            }

            if state.is_selected(i) {
                ui.painter().rect_filled(
                    rect,
                    5.0,
                    Color32::from_rgba_unmultiplied(100, 100, 255, 60),
                );
            }

            // --- Toda la lógica de interacción original ---
            handle_row_interactions(
                ui,
                &response,
                i,
                file,
                state,
                ui_state,
                files,
                content_rect,
                rect,
            );

            // clicks y selección
            if response.double_clicked_by(PointerButton::Primary) {
                if file.is_dir() {
                    state.navigate_to(file.full_path.to_owned());
                    state.deselect_all();
                    state.resize_selection(files.len());
                } else {
                    state.open_file(file);
                }
            }

            if response.clicked_by(PointerButton::Primary) {
                if state.im_navigating() {
                    return;
                }

                let modifiers = ui.input(|i| i.modifiers);
                if modifiers.shift {
                    if let Some(anchor) = state.selection_anchor {
                        let start = anchor.min(i);
                        let end = anchor.max(i);
                        state.select_range(start, end);
                    } else {
                        state.deselect_all();
                        state.resize_selection(files.len());
                        state.selection.set(i, true);
                        state.selection_anchor = Some(i);
                    }

                    state.last_selected_index = Some(i);
                } else if modifiers.ctrl {
                    let currently = state.is_selected(i);
                    state.resize_selection(files.len());
                    state.selection.set(i, !currently);
                    state.selection_anchor = Some(i);
                    state.last_selected_index = Some(i);
                } else {
                    state.deselect_all();
                    state.resize_selection(files.len());
                    state.selection.set(i, true);
                    state.selection_anchor = Some(i);
                    state.last_selected_index = Some(i);
                }
            }

            // --- Contenido de la fila ---
            let extended = info_snapshot.get(&file.full_path);
            let git = extended.and_then(|e| e.git_status.as_ref());
            let name_color = text_color_for_git(git);
            let dot_color = git_dot_color(git);

            // Columna Filename
            let icon_size = 16.0;
            let dot_size = 8.0;
            let icon_spacing = 4.0;
            let name_start_x = rect.min.x + icon_size + dot_size + icon_spacing * 2.0;
            let name_end_x = rect.min.x + name_w;

            let icon_rect = Rect::from_min_size(
                pos2(rect.min.x, rect.center().y - icon_size / 2.0),
                vec2(icon_size, icon_size),
            );

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
                let (icon_name, icon_bytes, color) = resolve_icon(file, &color_snapshot);
                let icon = ui_state
                    .icon_cache
                    .get_or_load(ui, &icon_name, icon_bytes, color);

                ui.painter().image(
                    icon.id(),
                    icon_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }

            // Dot git
            let dot_center = pos2(
                rect.min.x + icon_size + icon_spacing + dot_size / 2.0,
                rect.center().y,
            );
            if let Some(dot) = dot_color {
                ui.painter().circle_filled(dot_center, 3.5, dot);
                let dot_rect = Rect::from_center_size(dot_center, vec2(dot_size, dot_size));
                if let Some(git_status) = git {
                    let label = match git_status {
                        GitStatus::Modified => i18n.t("git_status.modified"),
                        GitStatus::Staged => i18n.t("git_status.staged"),
                        GitStatus::Untracked => i18n.t("git_status.untracked"),
                        GitStatus::Ignored => i18n.t("git_status.ignored"),
                        GitStatus::Conflict => i18n.t("git_status.conflict"),
                        GitStatus::Deleted => i18n.t("git_status.deleted"),
                        GitStatus::Clean => i18n.t("git_status.clean"),
                    };
                    ui.interact(dot_rect, ui.id().with(("dot", i)), Sense::hover())
                        .on_hover_text(label);
                }
            }

            // Nombre
            let motor = state.motor.borrow_mut();
            let display_name = if motor.active_tab().is_recursive_active {
                file.full_path
                    .strip_prefix(&motor.active_tab().cwd)
                    .unwrap_or(&file.full_path)
                    .to_string_lossy()
                    .to_string()
            } else {
                file.name.to_string()
            };
            drop(motor);

            let name_rect =
                Rect::from_min_max(pos2(name_start_x, rect.min.y), pos2(name_end_x, rect.max.y));
            let name_galley = ui.fonts_mut(|f| {
                f.layout_no_wrap(display_name, FontId::proportional(13.0), name_color)
            });
            let name_painter = ui.painter().with_clip_rect(name_rect);
            name_painter.galley(
                pos2(name_start_x, rect.center().y - name_galley.size().y / 2.0),
                name_galley,
                name_color,
            );

            // Columna Modified
            let date_rect = Rect::from_min_max(
                pos2(rect.min.x + name_w, rect.min.y),
                pos2(rect.min.x + name_w + date_w, rect.max.y),
            );
            let date_galley = ui.fonts_mut(|f| {
                f.layout_no_wrap(
                    format_date(file.modified).to_string(),
                    FontId::proportional(12.0),
                    Color32::from_rgb(109, 108, 111),
                )
            });
            ui.painter().with_clip_rect(date_rect).galley(
                pos2(
                    date_rect.min.x + 4.0,
                    rect.center().y - date_galley.size().y / 2.0,
                ),
                date_galley,
                Color32::WHITE,
            );

            // Columna Size
            let size_rect = Rect::from_min_max(
                pos2(rect.min.x + name_w + date_w, rect.min.y),
                pos2(rect.max.x, rect.max.y),
            );
            let display_size = if file.is_dir() {
                if state.calculating_dir_sizes.contains(&file.full_path) {
                    None
                } else if state.calculated_dir_sizes.contains(&file.full_path) {
                    Some(file.size)
                } else {
                    state
                        .sizer_manager
                        .cache_manager
                        .get_cached_size(&file.full_path)
                }
            } else {
                Some(file.size)
            };
            let size_text = match display_size {
                None => "...".to_string(),
                Some(0) if file.is_dir() => "-".to_string(),
                Some(size) => format_size(size),
            };
            let size_galley = ui.fonts_mut(|f| {
                f.layout_no_wrap(
                    size_text,
                    FontId::proportional(12.0),
                    Color32::from_rgb(109, 108, 111),
                )
            });
            ui.painter().with_clip_rect(size_rect).galley(
                pos2(
                    size_rect.min.x + 4.0,
                    rect.center().y - size_galley.size().y / 2.0,
                ),
                size_galley,
                Color32::WHITE,
            );
        }

        // Context menu
        let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);
        match ctx_menu.kind {
            ContextMenuKind::FileNormal => ctx_menu.file_context_menu(ui, state, ui_state, files),
            ContextMenuKind::FileTrash => {
                ctx_menu.file_context_menu_in_trash(ui, state, ui_state, files)
            }
            _ => {}
        }
        ui_state.context_menu_state = ctx_menu;

        if ui_state.needs_repaint || should_repaint {
            ui.ctx().request_repaint();
            ui_state.needs_repaint = false;
        }

        if !ui_state.newly_calculated_thumbnails.is_empty() {
            ui_state.newly_calculated_thumbnails.clear();
        }
    });

    if !state.rubber_band.is_rubber_banding {
        state.scroll_offset = scroll_output.state.offset.y;
    }

    state.row_view.viewport_height = scroll_output.inner_rect.height();
}
