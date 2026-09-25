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
        custom_components::text_edit::BlazeTextEdit,
        icons_cache::thumbnails::thumbnails_manager::Thumbnail,
        modules::{
            custom_context_menu::context_state::ContextMenuKind,
            utilities::{ensure_min_lightness, git_dot_color, resolve_icon, text_color_for_git},
        },
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{
    Align, Align2, Color32, ColorImage, CursorIcon, FontId, Id, Key, Layout, Modifiers,
    PointerButton, Rect, ScrollArea, Sense, TextureOptions, Ui, pos2, scroll_area::ScrollSource,
    vec2,
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
    col_index: usize,
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
            state.set_selection_anchor(Some(i));
            state.set_last_selected_index(Some(i));
        }
        state.miller_view.base.is_dragging_files = true;
    }

    if response.dragged_by(PointerButton::Primary) {
        state.miller_view.base.drag_ghost_pos = ui.input(|i| i.pointer.interact_pos());
    }

    if response.drag_stopped() && state.miller_view.base.is_dragging_files {
        state.miller_view.base.drag_ghost_pos = None;

        let drop_in_file_area = ui
            .input(|i| i.pointer.interact_pos())
            .map(|p| p.x <= content_rect.min.x + content_rect.width() * 0.80)
            .unwrap_or(false);

        if let Some(invalid_target) = state.miller_view.base.drop_invalid_target.take() {
            info!("No es posible mover a {:?}", invalid_target);
        }

        if drop_in_file_area {
            let tab_id = state.active_id();
            let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));

            if let Some(target) = state.miller_view.base.drop_target.take() {
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

        state.miller_view.base.is_dragging_files = false;
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
            ui_state.context_menu_state.target_col = Some(col_index);
            ui_state.context_menu_state.target_row_col = Some(i);
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
            ui_state.context_menu_state.target_col = Some(col_index);
            ui_state.context_menu_state.target_row_col = Some(i);
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

pub fn render_miller_scrollview(
    ui: &mut Ui,
    files: &[Arc<FileEntry>],
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    col_index: usize,
    content_rect: Rect,
    col_path: Arc<Path>,
) {
    if state.miller_view.columns.is_empty() {
        return;
    }

    ui_state.evict_thumbnail_cache_if_dir_changed(&state.cwd());
    ui_state.enforce_texture_cache_limit(500);

    let current_theme = with_theme(|t| t.current());
    let i18n = with_configs(|c| c.get_i18n());
    let col_id = ui.id().with(("miller_col", col_index));

    let icon_size = state.miller_view.base.icon_size;
    let row_height = (icon_size + 8.0).clamp(28.0, 64.0);
    let total_rows = files.len();

    let col_width = 220.0;

    let available = ui.available_width() * 0.8;

    ui.allocate_ui_with_layout(
        vec2(col_width, ui.available_height()),
        Layout::top_down(Align::Min),
        |ui| {
            //Creacion de carpetas nuevas
            new_ff_logic(state, ui);

            let mut scroll_offset = state
                .miller_view
                .scroll_offsets
                .get(&col_index)
                .copied()
                .unwrap_or(0.0);

            if let Some(target_row) = state.pending_scroll_to.take()
                && !files.is_empty()
            {
                let target_row = target_row.min(files.len() - 1);
                let row_top = target_row as f32 * row_height;
                let row_bottom = row_top + row_height;

                let viewport_top = scroll_offset;
                let viewport_bottop = scroll_offset + state.row_view.viewport_height;

                if row_top < viewport_top {
                    scroll_offset = row_top;
                } else if row_bottom > viewport_bottop {
                    scroll_offset = row_bottom - state.row_view.viewport_height;
                }
            }

            let scroll_area = ScrollArea::vertical()
                .id_salt(col_id)
                .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
                .auto_shrink([false, false])
                .vertical_scroll_offset(scroll_offset);

            let scroll_output =
                scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
                    let mut should_repaint = false;

                    let is_recursive = {
                        let motor = state.motor();
                        let tab = motor.active_tab();
                        tab.sources
                            .iter()
                            .find(|s| s.cwd == col_path)
                            .map(|s| s.is_recursive_active)
                            .unwrap_or(false)
                    };

                    // insertar los rangos visibles
                    state
                        .miller_view
                        .visible_ranges
                        .insert(col_index, (row_range.start, row_range.end));

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

                    for i in row_range.clone() {
                        let file = &files[i];
                        let is_renaming = state.renaming_file.as_deref() == Some(&file.full_path);

                        if is_renaming {
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(available, row_height), Sense::hover());

                            if i == row_range.start {
                                state.row_view.scroll_area_origin_y =
                                    rect.min.y + state.scroll_offset - (i as f32 * row_height);
                            }

                            render_rename_field(ui, file, state, rect, is_renaming);
                            continue;
                        }

                        let (rect, resp) = ui.allocate_exact_size(
                            vec2(col_width, row_height),
                            Sense::click_and_drag(),
                        );

                        // Aquí quizá ponga highjight drop

                        // doble click para abrir files que no sean directorios
                        if resp.double_clicked_by(PointerButton::Primary) && !file.is_dir() {
                            state.open_file(file);
                        }

                        // clicks de selección y navegación
                        if resp.clicked_by(PointerButton::Primary) {
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

                            if file.is_dir() {
                                // Cargar siguiente columna
                                state.navigate_to_from_col(file.full_path.clone(), col_index);
                                state.miller_view.set_opened(col_index, i);
                            }
                        }

                        let is_opened = state.miller_view.is_opened(col_index, i);

                        let is_selected = state.is_selected_for(&col_path, i);

                        // painter de la selección y directorio abierto
                        if is_opened || is_selected {
                            let color = if is_opened {
                                let c = current_theme.components.list_item.bg_selected.to_color();
                                Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 200)
                            } else {
                                let c = current_theme.components.list_item.bg_selected.to_color();
                                Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 60)
                            };

                            ui.painter().rect_filled(rect, 5.0, color);
                        }

                        // painter del hover
                        if resp.hovered() {
                            ui.set_cursor_icon(CursorIcon::PointingHand);

                            let mut c = current_theme.components.list_item.bg_selected.to_color();
                            c = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 60);

                            ui.painter().rect_filled(rect, 5.0, c);
                        }

                        // interacción por cada row
                        handle_row_interactions(
                            ui,
                            &resp,
                            i,
                            file,
                            state,
                            ui_state,
                            files,
                            content_rect,
                            rect,
                            col_index,
                        );

                        let extended = info_snapshot.get(&file.full_path);
                        let git = extended.and_then(|e| e.git_status.as_ref());
                        let name_color = text_color_for_git(git);
                        let dot_color = git_dot_color(git);

                        let dot_size = icon_size / 8.0;

                        let icon_spacing = icon_size / 4.0;
                        let left_padding = icon_size / 6.0;

                        // Icono
                        let icon_rect = Rect::from_min_size(
                            pos2(rect.min.x + 4.0, rect.center().y - icon_size / 2.0),
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

                        if let Some(dot) = dot_color {
                            let dot_center = pos2(
                                rect.min.x
                                    + left_padding
                                    + icon_size
                                    + icon_spacing
                                    + dot_size / 2.0,
                                rect.center().y,
                            );

                            ui.painter().circle_filled(dot_center, 3.5, dot);
                            let dot_rect =
                                Rect::from_center_size(dot_center, vec2(dot_size, dot_size));
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
                        let name_font_size = (icon_size / 2.0 * 1.2).clamp(13.0, 25.0);

                        // muestra la ruta o el nombre dependiendo del modod de búsqueda
                        let display_name: Cow<str> = if is_recursive {
                            file.full_path
                                .strip_prefix(state.cwd())
                                .unwrap_or(&file.full_path)
                                .to_string_lossy()
                        } else {
                            Cow::Borrowed(&file.name)
                        };

                        let name_start_x = icon_rect.max.x + 15.0;
                        let name_end_x = rect.max.x - 16.0; // espacio para chevron
                        let name_rect = Rect::from_min_max(
                            pos2(name_start_x, rect.min.y),
                            pos2(name_end_x, rect.max.y),
                        );

                        let name_galley = ui.fonts_mut(|f| {
                            f.layout_no_wrap(
                                display_name.to_string(),
                                FontId::proportional(name_font_size),
                                name_color,
                            )
                        });

                        ui.painter().with_clip_rect(name_rect).galley(
                            pos2(name_start_x, rect.center().y - name_galley.size().y / 2.0),
                            name_galley,
                            ensure_min_lightness(name_color),
                        );

                        // Chevron para directorios
                        if file.is_dir() {
                            ui.painter().text(
                                rect.right_center() - vec2(8.0, 0.0),
                                Align2::RIGHT_CENTER,
                                "›",
                                FontId::proportional(14.0),
                                current_theme.semantic.text_secondary.to_color(),
                            );
                        }
                    }

                    // Context menu
                    let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);
                    match ctx_menu.kind {
                        ContextMenuKind::FileNormal => {
                            if ctx_menu.target_col == Some(col_index) {
                                ctx_menu.file_context_menu(ui, state, ui_state);
                            }
                        }
                        ContextMenuKind::FileTrash => {
                            ctx_menu.file_context_menu_in_trash(ui, state, ui_state)
                        }
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

            state
                .miller_view
                .scroll_offsets
                .insert(col_index, scroll_output.state.offset.y);
        },
    );
}
