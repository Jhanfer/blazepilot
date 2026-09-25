use crate::{
    core::{
        blaze_state::BlazeCoreState,
        bootstrap::configs::{
            config_manager::with_configs,
            platform::linux::conf_structs::{
                DetailedColumn, OrderingDirection, OrderingKind, OrderingMode,
            },
        },
        files::{blaze_motor::motor_structs::FileEntry, file_extension::StrExtension},
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
        custom_components::text_edit::BlazeTextEdit,
        icons_cache::thumbnails::thumbnails_manager::Thumbnail,
        modules::{
            custom_context_menu::context_state::ContextMenuKind,
            utilities::{ensure_min_lightness, git_dot_color, resolve_icon, text_color_for_git},
        },
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
    utils::formating::{format_date, format_size},
};
use egui::{
    Button, Color32, ColorImage, CursorIcon, FontId, Id, Key, Modifiers, PointerButton, Rect,
    RichText, ScrollArea, Sense, Stroke, StrokeKind, TextureOptions, Ui, pos2,
    scroll_area::ScrollSource, vec2,
};
use std::{borrow::Cow, collections::HashMap, path::Path, sync::Arc};
use tracing::info;

fn new_ff_logic(state: &mut BlazeCoreState, ui: &mut Ui) {
    if let Some(item_type) = state.creating_new.clone() {
        let creating_new_id = Id::new("creating_new");

        ui.horizontal(|ui| {
            let response =
                ui.add(BlazeTextEdit::singleline(&mut state.new_item_buffer).id(creating_new_id));

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

#[allow(clippy::too_many_arguments)]
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
        state.set_selection(i, !currently);
        state.set_last_selected_index(Some(i));

        if file.is_dir() {
            state.add_tab_from_file(&file.full_path);
        }
    }

    if response.drag_started_by(PointerButton::Primary) {
        if !state.is_selected(i) {
            state.deselect_all();
            state.resize_selection(files.len());
            state.set_selection(i, true);
            state.set_last_selected_index(Some(i));
            state.set_selection_anchor(Some(i));
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
            let tab_id = state.active_id();
            let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));

            if let Some(target) = state.row_view.drop_target.take() {
                let sources = state.get_selected_paths(files);

                dispatcher
                    .send(UiEvent::SureTo(SureTo::SureToMove {
                        files: sources,
                        dest: target,
                    }))
                    .ok();
            } else {
                let cwd = state.cwd();
                let sources = state.get_selected_paths(files);

                if sources.iter().all(|(_, p)| p.parent() == Some(&cwd)) {
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

        state.row_view.is_dragging_files = false;
    }

    if response.secondary_clicked() {
        state.resize_selection(files.len());

        if ui.input(|i| i.modifiers.ctrl) {
            let currently = state.is_selected(i);
            state.set_selection(i, !currently);
            state.set_last_selected_index(Some(i));
        } else if !state.is_selected(i) {
            state.deselect_all();
            state.resize_selection(files.len());
            state.set_selection(i, true);
            state.set_last_selected_index(Some(i));
        }
    }

    let cwd = state.cwd();
    let is_in_trash = get_backend().etched_in_trash_path(&cwd);

    if is_in_trash {
        let tab_id = state.active_id();
        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
        if response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(response);
            ui_state.context_menu_state.target_sender = Some(dispatcher);
            ui_state.context_menu_state.kind = ContextMenuKind::FileTrash;
            ui_state.context_menu_state.source_path = Some(cwd);
        }
    } else {
        let tab_id = state.active_id();
        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
        if response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(response);
            ui_state.context_menu_state.target_file = Some(file.clone());
            ui_state.context_menu_state.target_sender = Some(dispatcher);
            ui_state.context_menu_state.kind = ContextMenuKind::FileNormal;
            ui_state.context_menu_state.source_path = Some(cwd);
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
        BlazeTextEdit::singleline(&mut state.rename_buffer)
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

fn col_sort_label(col: &DetailedColumn, order: &OrderingMode) -> Box<str> {
    let i18n = with_configs(|c| c.get_i18n());

    let (kind, dir) = (order.kind, order.direction);
    let arrow = match dir {
        OrderingDirection::Asc => "↑",
        OrderingDirection::Desc => "↓",
    };

    let base = match col {
        DetailedColumn::Name => i18n.t("tools.name"),
        DetailedColumn::Size => i18n.t("tools.size"),
        DetailedColumn::Extension => i18n.t("tools.extension"),
        DetailedColumn::Modified => i18n.t("tools.modified"),
        DetailedColumn::Created => i18n.t("tools.created"),
        DetailedColumn::Accessed => i18n.t("tools.accessed"),
        DetailedColumn::Owner => i18n.t("tools.owner"),
        DetailedColumn::Group => i18n.t("tools.group"),
        DetailedColumn::Permissions => i18n.t("tools.permissions"),
        DetailedColumn::Dimensions => i18n.t("tools.dimensions"),
        DetailedColumn::SymlinkTarget => i18n.t("tools.symlink"),
    };

    let ordering_kind = col_ordering_kind(col);
    if ordering_kind == Some(kind) {
        format!("{}{}", base, arrow).into()
    } else {
        base
    }
}

fn col_ordering_kind(col: &DetailedColumn) -> Option<OrderingKind> {
    match col {
        DetailedColumn::Name => Some(OrderingKind::Name),
        DetailedColumn::Size => Some(OrderingKind::Size),
        DetailedColumn::Modified => Some(OrderingKind::Date),
        _ => None,
    }
}

fn col_handle_sort_click(col: &DetailedColumn, order: &OrderingMode, state: &mut BlazeCoreState) {
    let Some(kind) = col_ordering_kind(col) else {
        return;
    };
    with_configs(|c| {
        c.set_ordering_mode(match (order.kind == kind, order.direction) {
            (true, OrderingDirection::Asc) => OrderingMode {
                kind,
                direction: OrderingDirection::Desc,
                ..*order
            },
            _ => OrderingMode {
                kind,
                direction: OrderingDirection::Asc,
                ..*order
            },
        });
    });
    state.refresh();
}

pub fn render_detailed_row_scrollview(
    ui: &mut Ui,
    files: &[Arc<FileEntry>],
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    row_height: f32,
    total_rows: usize,
    content_rect: Rect,
) {
    let current_theme = with_theme(|t| t.current());

    let icon_size = state.row_view.icon_size;

    let i18n = with_configs(|c| c.get_i18n());

    ui_state.evict_thumbnail_cache_if_dir_changed(&state.cwd());
    ui_state.enforce_texture_cache_limit(500);

    let current_order = with_configs(|c| c.get_ordering_mode());

    // --- Header ---
    let detailed_columns_configs = with_configs(|c| c.get_detailed_view_config());
    let columns = &detailed_columns_configs.visible_columns;
    let available = ui.available_width() * 0.8;

    let other_cols_totals: f32 = columns
        .iter()
        .filter(|c| **c != DetailedColumn::Name)
        .map(|c| {
            let id = ui.id().with(format!("col_{:?}", c));
            ui.data(|d| d.get_temp(id).unwrap_or(c.default_width()))
                .max(c.min_width())
        })
        .sum();

    let name_id = ui.id().with("col_Name");
    let name_w: f32 = ui
        .data(|d| d.get_temp(name_id).unwrap_or(available - other_cols_totals))
        .max(DetailedColumn::Name.min_width());

    let header_height = 24.0;
    let (header_rect, _) = ui.allocate_exact_size(vec2(available, header_height), Sense::hover());

    let painter = ui.painter_at(header_rect);
    painter.rect_filled(header_rect, 0.0, Color32::TRANSPARENT);

    let handle_w = 4.0;
    let mut c_x = header_rect.min.x;

    for col in columns.iter() {
        let col_w = if *col == DetailedColumn::Name {
            name_w
        } else {
            let id = ui.id().with(format!("col_{:?}", col));
            ui.data(|d| d.get_temp(id).unwrap_or(col.default_width()))
                .max(col.min_width())
        };

        let btn_rect =
            Rect::from_min_size(pos2(c_x, header_rect.min.y), vec2(col_w, header_height));

        let label = col_sort_label(col, &current_order);

        if ui
            .put(
                btn_rect,
                Button::new(
                    RichText::new(label)
                        .size(11.0)
                        .color(current_theme.semantic.text_primary.to_color()),
                )
                .fill(current_theme.semantic.bg_container.to_color())
                .stroke(Stroke::NONE)
                .frame(true),
            )
            .clicked()
        {
            col_handle_sort_click(col, &current_order, state);
        }

        let handle_x = c_x + col_w;
        let handle_rect = Rect::from_min_size(
            pos2(handle_x - handle_w / 2.0, header_rect.min.y),
            vec2(handle_w, header_height),
        );

        let col_id = ui.id().with(format!("col_{:?}", col));
        let handle_id = ui.id().with(("resize_handle", col_id));
        let handle_response = ui.interact(handle_rect, handle_id, Sense::click_and_drag());

        if handle_response.hovered() || handle_response.dragged() {
            ui.set_cursor_icon(CursorIcon::ResizeColumn);
            painter.rect_filled(handle_rect, 0.0, current_theme.semantic.accent.to_color());
        } else {
            painter.rect_filled(
                handle_rect,
                0.0,
                current_theme.semantic.text_muted.to_color(),
            );
        }

        if handle_response.dragged() {
            let new_w = (col_w + handle_response.drag_delta().x).max(col.min_width());
            ui.data_mut(|d| d.insert_temp(col_id, new_w));
        }

        c_x += col_w;
    }

    let col_widths: Vec<(DetailedColumn, f32)> = columns
        .iter()
        .map(|col| {
            let width = if *col == DetailedColumn::Name {
                name_w
            } else {
                let id = ui.id().with(format!("col_{:?}", col));
                ui.data(|d| d.get_temp(id).unwrap_or(col.default_width()))
                    .max(col.min_width())
            };
            (col.clone(), width)
        })
        .collect();

    //Creacion de carpetas nuevas
    new_ff_logic(state, ui);

    if let Some(target_row) = state.pending_scroll_to.take()
        && !files.is_empty()
    {
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

    let scroll_area = ScrollArea::vertical()
        .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
        .auto_shrink([false, false])
        .vertical_scroll_offset(state.scroll_offset);

    let scroll_output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
        let mut should_repaint = false;

        let is_recursive = {
            let motor = state.motor();
            let tab = motor.active_tab();
            tab.sources
                .iter()
                .find(|s| s.cwd == state.cwd())
                .map(|s| s.is_recursive_active)
                .unwrap_or(false)
        };

        ui.spacing_mut().item_spacing.y = 0.0;

        state.row_view.first_visible = row_range.start;
        state.row_view.last_visible = row_range.end;

        let info_snapshot: HashMap<Arc<Path>, ExtendedInfo> = {
            match state.extended_info_manager.info_map.read() {
                Ok(map) => row_range
                    .clone()
                    .filter_map(|i| {
                        let path = &files[i].full_path;
                        map.peek(path).map(|v| (path.clone(), v.clone()))
                    })
                    .collect::<HashMap<Arc<Path>, ExtendedInfo>>(),
                Err(_) => HashMap::new(),
            }
        };

        ui_state.color_snapshot = {
            let color_map = &ui_state.folder_color_manager.cache_manager.color_cache;

            row_range
                .clone()
                .filter_map(|i| {
                    files[i]
                        .unique_id
                        .as_ref()
                        .and_then(|id| color_map.lock().get(id).map(|c| (*id, c.color)))
                })
                .collect()
        };

        let thumbnail_snapshot: HashMap<Arc<Path>, Arc<Thumbnail>> = {
            let guard = ui_state.thumbnail_manager.thumb_map.read();
            row_range
                .clone()
                .filter_map(|i| {
                    let p = &files[i].full_path;
                    guard.peek(p).cloned().map(|t| (p.clone(), t))
                })
                .collect()
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
                ui.painter().rect_filled(
                    rect,
                    5.0,
                    current_theme.components.list_item.bg_hover.to_color(),
                );
            }

            if state.is_selected(i) {
                ui.painter().rect_filled(
                    rect,
                    5.0,
                    Color32::from_rgba_unmultiplied(
                        current_theme
                            .components
                            .list_item
                            .bg_selected
                            .to_color()
                            .r(),
                        current_theme
                            .components
                            .list_item
                            .bg_selected
                            .to_color()
                            .g(),
                        current_theme
                            .components
                            .list_item
                            .bg_selected
                            .to_color()
                            .b(),
                        60,
                    ),
                );
            }

            // Drop target highlight
            if let Some(ref target) = state.row_view.drop_target.clone() {
                if *file.full_path == **target {
                    ui.painter().rect_stroke(
                        rect,
                        5.0,
                        Stroke::new(2.0, Color32::from_rgb(150, 150, 255)),
                        StrokeKind::Outside,
                    );
                }
            } else if let Some(ref target_invalid) = state.row_view.drop_invalid_target.clone()
                && *file.full_path == **target_invalid
            {
                ui.painter().rect_stroke(
                    rect,
                    5.0,
                    Stroke::new(2.0, Color32::from_rgb(255, 150, 150)),
                    StrokeKind::Outside,
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
                    if let Some(anchor) = state.selection_anchor() {
                        let start = anchor.min(i);
                        let end = anchor.max(i);
                        state.select_range(start, end);
                    } else {
                        state.deselect_all();
                        state.resize_selection(files.len());
                        state.set_selection(i, true);
                        state.set_selection_anchor(Some(i));
                    }

                    state.set_last_selected_index(Some(i));
                } else if modifiers.ctrl {
                    let currently = state.is_selected(i);
                    state.resize_selection(files.len());
                    state.set_selection(i, !currently);
                    state.set_selection_anchor(Some(i));
                    state.set_last_selected_index(Some(i));
                } else {
                    state.deselect_all();
                    state.resize_selection(files.len());
                    state.set_selection(i, true);
                    state.set_selection_anchor(Some(i));
                    state.set_last_selected_index(Some(i));
                }
            }

            let extended = info_snapshot.get(&file.full_path);
            let git = extended.and_then(|e| e.git_status.as_ref());
            let name_color = text_color_for_git(git);
            let dot_color = git_dot_color(git);

            let mut cell_x = rect.min.x;

            for (col, col_w) in col_widths.iter() {
                let cell_rect =
                    Rect::from_min_max(pos2(cell_x, rect.min.y), pos2(cell_x + col_w, rect.max.y));

                let text_color = current_theme.semantic.text_primary.to_color();

                if *col == DetailedColumn::Name {
                    let dot_size = icon_size / 8.0;
                    let icon_spacing = icon_size / 4.0;
                    let left_padding = icon_size / 6.0;
                    let name_start_x =
                        cell_rect.min.x + left_padding + icon_size + dot_size + icon_spacing * 2.0;

                    let icon_rect = Rect::from_min_size(
                        pos2(
                            cell_rect.min.x + left_padding,
                            rect.center().y - icon_size / 2.0,
                        ),
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
                            vec2(icon_size, icon_size),
                        );

                        ui.painter().image(
                            icon.id(),
                            rounded_rect,
                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }

                    let dot_center = pos2(
                        rect.min.x + left_padding + icon_size + icon_spacing + dot_size / 2.0,
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

                    let name_font_size = (icon_size / 2.0 * 1.2).clamp(13.0, 25.0);

                    let display_name: Cow<str> = if is_recursive {
                        file.full_path
                            .strip_prefix(state.cwd())
                            .unwrap_or(&file.full_path)
                            .to_string_lossy()
                    } else {
                        Cow::Borrowed(&file.name)
                    };

                    let name_galley = ui.fonts_mut(|f| {
                        f.layout_no_wrap(
                            display_name.to_string(),
                            FontId::proportional(name_font_size),
                            name_color,
                        )
                    });

                    ui.painter().with_clip_rect(cell_rect).galley(
                        pos2(name_start_x, rect.center().y - name_galley.size().y / 2.0),
                        name_galley,
                        ensure_min_lightness(name_color),
                    );
                } else {
                    let text = match col {
                        DetailedColumn::Size => {
                            let display_size = if file.is_dir() {
                                state
                                    .sizer_manager
                                    .cache_manager
                                    .get_cached_size(&file.full_path)
                            } else {
                                Some(file.size)
                            };

                            match display_size {
                                None => "...",
                                Some(0) if file.is_dir() => "-",
                                Some(s) => &format_size(s),
                            }
                        }
                        DetailedColumn::Modified => &format_date(file.modified),
                        DetailedColumn::Created => &format_date(file.created),
                        DetailedColumn::Accessed => &format_date(file.accessed),
                        DetailedColumn::Extension if file.is_dir() => "",
                        DetailedColumn::Extension => file.extension.extension(),
                        DetailedColumn::Owner => &extended
                            .and_then(|e| e.owner.clone())
                            .unwrap_or("...".to_owned()),
                        DetailedColumn::Group => &extended
                            .and_then(|e| e.group_name.clone())
                            .unwrap_or("...".to_owned()),
                        DetailedColumn::Permissions => {
                            #[cfg(unix)]
                            {
                                &format!("{}", file.permissions)
                            }
                            #[cfg(windows)]
                            {
                                &format!("{}", file.attributes)
                            }
                        }
                        DetailedColumn::Dimensions => &extended
                            .and_then(|e| e.dimensions)
                            .map(|(w, h)| format!("{}×{}", w, h))
                            .unwrap_or("_".to_owned()),
                        DetailedColumn::SymlinkTarget => &extended
                            .and_then(|e| e.symlink_target.clone())
                            .map(|p| p.display().to_string())
                            .unwrap_or("...".to_owned()),
                        DetailedColumn::Name => unreachable!(),
                    };

                    let font_size = (icon_size / 2.0 * 1.2).clamp(12.0, 24.0);
                    let galley = ui.fonts_mut(|f| {
                        f.layout_no_wrap(
                            text.to_owned(),
                            FontId::proportional(font_size),
                            text_color,
                        )
                    });

                    ui.painter().with_clip_rect(cell_rect).galley(
                        pos2(
                            cell_rect.min.x + 4.0,
                            cell_rect.center().y - galley.size().y / 2.0,
                        ),
                        galley,
                        text_color,
                    );
                }
                cell_x += col_w;
            }
        }

        // Context menu
        let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);
        match ctx_menu.kind {
            ContextMenuKind::FileNormal => ctx_menu.file_context_menu(ui, state, ui_state),
            ContextMenuKind::FileTrash => ctx_menu.file_context_menu_in_trash(ui, state, ui_state),
            _ => {}
        }
        ui_state.context_menu_state = ctx_menu;

        if ui_state.needs_repaint || should_repaint {
            ui.request_repaint();
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
