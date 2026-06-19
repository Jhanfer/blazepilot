use crate::{
    core::{
        blaze_state::BlazeCoreState,
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::{
            bus_structs::{SureTo, UiEvent},
            event_bus::with_event_bus,
        },
        system::{
            extended_info::extended_info_manager::{ExtendedInfo, GitStatus},
            trash_manager::manager::get_backend,
        },
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::thumbnails::thumbnails_manager::Thumbnail,
        modules::{
            custom_context_menu::context_state::ContextMenuKind,
            utilities::{git_dot_color, resolve_icon, text_color_for_git},
        },
        themes::colors::COLOR_MAIN_BUTTONS,
    },
    utils::formating::{format_date, format_size},
};
use egui::{
    pos2, scroll_area::ScrollSource, vec2, Color32, ColorImage, CursorIcon, FontId, Id, Key,
    Modifiers, PointerButton, Rect, ScrollArea, Sense, Stroke, StrokeKind, TextEdit,
    TextureOptions, Ui,
};
use file_id::FileId;
use std::{collections::HashMap, path::Path, sync::Arc};
use tracing::info;

fn grid_file_creation(
    state: &mut BlazeCoreState,
    ui: &mut Ui,
    available_width: f32,
    ui_state: &mut BlazeUiState,
) {
    if let Some(item_type) = state.creating_new.clone() {
        let icon_size = state.grid_view.icon_size;
        let cell_size = state.grid_view.cell_size;
        let row_height = state.grid_view.row_height;
        let cell_padding = 8.0_f32;

        let (icon_name, icon_bytes, color) = match item_type {
            crate::core::blaze_state::NewItemType::Folder => (
                "folder".to_string(),
                crate::ui::icons_cache::icons::ICON_FOLDER,
                Color32::YELLOW,
            ),
            crate::core::blaze_state::NewItemType::File => (
                "file".to_string(),
                crate::ui::icons_cache::icons::ICON_FILE,
                Color32::WHITE,
            ),
        };

        let (row_rect, _) =
            ui.allocate_exact_size(vec2(available_width, row_height), Sense::hover());
        let cell_x = row_rect.min.x + 0.0 * cell_size;
        let rect = Rect::from_min_size(pos2(cell_x, row_rect.min.y), vec2(cell_size, row_height));

        let icon_top = rect.min.y + cell_padding;
        let icon_x = rect.min.x + (cell_size - icon_size) / 2.0;
        let icon_rect = Rect::from_min_size(pos2(icon_x, icon_top), vec2(icon_size, icon_size));

        let rounded_rect = Rect::from_min_max(
            pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
            pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
        );

        let icon = ui_state.icon_cache.get_or_load(
            ui,
            &icon_name,
            icon_bytes,
            color,
            vec2(icon_size, icon_size),
        );
        ui.painter().image(
            icon.id(),
            rounded_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let creating_new_id = Id::new("creating_new");

        let text_rect = Rect::from_min_size(
            pos2(rect.min.x + 4.0, icon_rect.max.y + 4.0),
            vec2(rect.width() - 8.0, 22.0),
        );

        let response = ui.put(
            text_rect,
            TextEdit::singleline(&mut state.new_item_buffer).id(creating_new_id),
        );

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
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_grid_interactions(
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
        state.grid_view.is_dragging_files = true;
    }

    if response.dragged_by(PointerButton::Primary) {
        state.grid_view.drag_ghost_pos = ui.input(|i| i.pointer.interact_pos());
    }

    if response.drag_stopped() && state.grid_view.is_dragging_files {
        state.grid_view.drag_ghost_pos = None;

        let drop_in_file_area = ui
            .input(|i| i.pointer.interact_pos())
            .map(|p| p.x <= content_rect.min.x + content_rect.width() * 0.80)
            .unwrap_or(false);

        if let Some(invalid_target) = state.grid_view.drop_invalid_target.take() {
            info!("No es posible mover a {:?}", invalid_target);
        }

        if drop_in_file_area {
            let tab_id = state.active_id;
            let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));

            if let Some(target) = state.grid_view.drop_target.take() {
                let sources = state.get_selected_paths(files);

                dispatcher
                    .send(UiEvent::SureTo(SureTo::SureToMove {
                        files: sources,
                        dest: target,
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
                    }))
                    .ok();
            }
        }

        state.grid_view.is_dragging_files = false;
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

pub fn render_grid_scrollview(
    ui: &mut Ui,
    files: &[Arc<FileEntry>],
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    content_rect: Rect,
) {
    let i18n = with_configs(|c| c.get_i18n());

    ui_state.evict_thumbnail_cache_if_dir_changed(&state.cwd);
    ui_state.enforce_texture_cache_limit(500);

    let cell_padding = 8.0_f32;

    let icon_size = state.grid_view.icon_size;
    let cell_size = icon_size + 40.0;

    let row_height = cell_size + 28.0;

    state.grid_view.row_height = row_height;

    let available_width = ui.available_width();

    let cols = ((available_width / cell_size).floor() as usize).max(1);

    state.grid_view.cols = cols;
    state.grid_view.cell_size = cell_size;

    let total_rows = files.len().div_ceil(cols);

    // --- Scroll-to ---
    if let Some(target_idx) = state.pending_scroll_to.take() {
        if !files.is_empty() {
            let target_idx = target_idx.min(files.len() - 1);
            let target_row = target_idx / cols;
            let row_top = target_row as f32 * row_height;
            let row_bottom = row_top + row_height;

            let viewport_top = state.scroll_offset;
            let viewport_bottom = state.scroll_offset + state.grid_view.viewport_height;

            if row_top < viewport_top {
                state.scroll_offset = row_top;
            } else if row_bottom > viewport_bottom {
                state.scroll_offset = row_bottom - state.grid_view.viewport_height;
            }
        }
    }

    let scroll_area = ScrollArea::vertical()
        .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
        .auto_shrink([false, false])
        .vertical_scroll_offset(state.scroll_offset);

    let scroll_output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
        // --- Creación de carpeta/archivo nuevo ---
        grid_file_creation(state, ui, available_width, ui_state);

        let mut should_repaint = false;

        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        state.grid_view.first_visible = row_range.start * cols;
        state.grid_view.last_visible = (row_range.end * cols).min(files.len());

        // --- Snapshots de datos externos ---
        let file_indices: Vec<usize> = row_range
            .clone()
            .flat_map(|row| {
                let start = row * cols;
                let end = (start + cols).min(files.len());
                start..end
            })
            .collect();

        let info_snapshot: HashMap<Arc<Path>, ExtendedInfo> = {
            match state.extended_info_manager.info_map.write() {
                Ok(mut map) => file_indices
                    .iter()
                    .filter_map(|&i| {
                        let path = &files[i].full_path;
                        map.get(path).map(|v| (path.clone(), v.clone()))
                    })
                    .collect(),
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
                Ok(guard) => file_indices
                    .iter()
                    .filter_map(|&i| {
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
                Ok(mut guard) => file_indices
                    .iter()
                    .filter_map(|&i| {
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

        // --- Renderizado fila por fila del grid ---
        for row in row_range.clone() {
            let (row_rect, _) =
                ui.allocate_exact_size(vec2(available_width, row_height), Sense::hover());

            if row == row_range.start && row_range.start == 0 {
                state.grid_view.actual_origin = row_rect.min;
            }

            if row == row_range.start {
                state.grid_view.scroll_area_origin_y =
                    row_rect.min.y + state.scroll_offset - (row as f32 * row_height);
            }

            for col in 0..cols {
                let i = row * cols + col;
                if i >= files.len() {
                    break;
                }

                let file = &files[i];

                let cell_x = row_rect.min.x + col as f32 * cell_size;
                let rect =
                    Rect::from_min_size(pos2(cell_x, row_rect.min.y), vec2(cell_size, row_height));

                // --- Rename inline ---
                let is_renaming = state.renaming_file.as_deref() == Some(&file.full_path);
                if is_renaming {
                    let rename_rect = Rect::from_min_max(
                        pos2(rect.min.x + cell_padding, rect.max.y - 28.0),
                        pos2(rect.max.x - cell_padding, rect.max.y - 2.0),
                    );
                    render_rename_field(ui, file, state, rename_rect, is_renaming);
                }

                // --- Interacción de la celda ---
                let cell_id = ui.id().with(("grid_cell", i));
                let response = ui.interact(rect, cell_id, Sense::click_and_drag());

                if response.hovered() && !is_renaming {
                    ui.set_cursor_icon(CursorIcon::PointingHand);
                    let margin = 4.0;
                    let selection_rect = Rect::from_min_max(
                        pos2(rect.min.x + margin, rect.min.y + margin),
                        pos2(rect.max.x - margin, rect.max.y - margin),
                    );

                    ui.painter()
                        .rect_filled(selection_rect, 8.0, COLOR_MAIN_BUTTONS);
                }

                if state.is_selected(i) {
                    let margin = 4.0;
                    let selection_rect = Rect::from_min_max(
                        pos2(rect.min.x + margin, rect.min.y + margin),
                        pos2(rect.max.x - margin, rect.max.y - margin),
                    );

                    ui.painter().rect_filled(
                        selection_rect,
                        8.0,
                        Color32::from_rgba_unmultiplied(100, 100, 255, 60),
                    );
                }

                // Drop target highlight
                if let Some(ref target) = state.grid_view.drop_target.clone() {
                    if *file.full_path == **target {
                        ui.painter().rect_stroke(
                            rect,
                            8.0,
                            Stroke::new(2.0, Color32::from_rgb(150, 150, 255)),
                            StrokeKind::Outside,
                        );
                    }
                } else if let Some(ref target_invalid) = state.grid_view.drop_invalid_target.clone()
                {
                    if *file.full_path == **target_invalid {
                        ui.painter().rect_stroke(
                            rect,
                            8.0,
                            Stroke::new(2.0, Color32::from_rgb(255, 150, 150)),
                            StrokeKind::Outside,
                        );
                    }
                }

                handle_grid_interactions(
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

                // Double-click para abrir
                if response.double_clicked_by(PointerButton::Primary) {
                    if file.is_dir() {
                        state.navigate_to(file.full_path.to_owned());
                        state.deselect_all();
                        state.resize_selection(files.len());
                    } else {
                        state.open_file(file);
                    }
                }

                // Click primario con modificadores
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

                let extended = info_snapshot.get(&file.full_path);
                let git = extended.and_then(|e| e.git_status.as_ref());
                let name_color = text_color_for_git(git);
                let dot_color = git_dot_color(git);

                let icon_top = rect.min.y + cell_padding;
                let icon_x = rect.min.x + (cell_size - icon_size) / 2.0;
                let icon_rect =
                    Rect::from_min_size(pos2(icon_x, icon_top), vec2(icon_size, icon_size));

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

                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    ui.painter().image(
                        tex.id(),
                        rounded_rect,
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

                    let rounded_rect = Rect::from_min_max(
                        pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                        pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
                    );

                    let icon = ui_state.icon_cache.get_or_load(
                        ui,
                        &icon_name,
                        icon_bytes,
                        color,
                        vec2(icon_size, icon_size),
                    );
                    ui.painter().image(
                        icon.id(),
                        rounded_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                if let Some(dot) = dot_color {
                    let dot_center = pos2(icon_rect.max.x - 4.0, icon_rect.min.y + 4.0);
                    ui.painter().circle_filled(dot_center, 4.0, dot);

                    let dot_rect = Rect::from_center_size(dot_center, vec2(10.0, 10.0));
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

                if !is_renaming {
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

                    let name_top = icon_rect.max.y + 4.0;
                    let name_padding = 3.0;
                    let name_rect = Rect::from_min_max(
                        pos2(rect.min.x + name_padding, name_top),
                        pos2(rect.max.x - name_padding, rect.max.y - 2.0),
                    );

                    let name_font_size = (icon_size / 4.5).clamp(13.0, 25.0);

                    let mut name_galley = ui.fonts_mut(|f| {
                        f.layout(
                            display_name.clone(),
                            FontId::proportional(name_font_size),
                            name_color,
                            name_rect.width(),
                        )
                    });

                    let max_height = name_rect.height();

                    if name_galley.size().y > max_height && !name_galley.is_empty() {
                        let line_height = name_galley.size().y / name_galley.rows.len() as f32;
                        let max_lines = (max_height / line_height).floor() as usize;
                        let max_lines = max_lines.max(1);

                        if name_galley.rows.len() > max_lines {
                            let last_row_idx = max_lines - 1;

                            let mut byte_idx = 0;
                            if let Some(glyph) = name_galley.rows[last_row_idx].glyphs.last() {
                                byte_idx = glyph.chr.len_utf8();
                            }

                            let mut absolute_byte_idx = 0;
                            for row_idx in 0..=last_row_idx {
                                if row_idx < last_row_idx {
                                    for glyph in &name_galley.rows[row_idx].glyphs {
                                        absolute_byte_idx += glyph.chr.len_utf8();
                                    }
                                } else {
                                    absolute_byte_idx += byte_idx;
                                }
                            }

                            let mut truncated = display_name.clone();
                            if absolute_byte_idx > 0 && absolute_byte_idx <= truncated.len() {
                                truncated.truncate(absolute_byte_idx);
                            }

                            for _ in 0..3 {
                                match truncated.pop() {
                                    Some(_) => continue,
                                    None => break,
                                }
                            }
                            truncated.push_str("...");

                            name_galley = ui.fonts_mut(|f| {
                                f.layout(
                                    truncated,
                                    FontId::proportional(name_font_size),
                                    name_color,
                                    name_rect.width(),
                                )
                            });
                        }
                    }

                    let text_y = name_rect.min.y;
                    let text_x = name_rect.min.x + (name_rect.width() - name_galley.size().x) / 2.0;

                    ui.painter().with_clip_rect(name_rect).galley(
                        pos2(text_x, text_y),
                        name_galley,
                        name_color,
                    );
                }

                if response.hovered() {
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

                    let tooltip = format!(
                        "{}\n{}: {}",
                        file.name,
                        i18n.t("tools.modified"),
                        format_date(file.modified),
                    );
                    let tooltip = if file.is_dir() {
                        tooltip
                    } else {
                        format!("{}\n{}: {}", tooltip, i18n.t("tools.size"), size_text)
                    };

                    response.on_hover_text(tooltip);
                }
            }
        }

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

    state.grid_view.viewport_height = scroll_output.inner_rect.height();
}
