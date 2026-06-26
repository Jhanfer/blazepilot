use crate::{
    core::{
        blaze_state::BlazeCoreState,
        files::blaze_motor::motor_structs::FileEntry,
        runtime::event_bus::with_event_bus,
        system::{
            extended_info::extended_info_manager::ExtendedInfoMessages,
            sizer_manager::manager::SizerMessages, trash_manager::manager::get_backend,
        },
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::thumbnails::thumbnails_manager::ThumbnailMessages,
        modules::{
            custom_context_menu::context_state::ContextMenuKind,
            drag_drop_logic::drag_files,
            hot_keys::hot_keys_logic,
            island_n_bubble::render_island_bubble,
            row_view::{
                new_scroll_view::new_render_scrollview, rubber_band_logic::render_row_rubberband,
            },
        },
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{CornerRadius, Frame, Margin, Rect, Stroke, Ui};
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
fn background_response_logic(
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    files: &[Arc<FileEntry>],
    ui: &mut Ui,
    panel_top: f32,
    total_rows: usize,
    row_height: f32,
    content_rect: Rect,
) {
    let bg_id = ui.id().with("background_interact");
    let bg_response = ui.interact(
        ui.available_rect_before_wrap(),
        bg_id,
        egui::Sense::click_and_drag(),
    );

    let dragged_by = bg_response.drag_started_by(egui::PointerButton::Primary)
        || bg_response.drag_started_by(egui::PointerButton::Secondary);

    if dragged_by {
        if let Some(orgin) = ui.input(|i| i.pointer.press_origin()) {
            state.rubber_band.rubber_band_start_content_y =
                orgin.y - panel_top + state.scroll_offset;
            state.rubber_band.rubber_band_start = Some(orgin);
        }
        state.rubber_band.is_rubber_banding = true;
    }

    if bg_response.dragged() {
        state.rubber_band.rubber_band_current = ui.input(|i| i.pointer.interact_pos());

        if let Some(current) = state.rubber_band.rubber_band_current {
            let total_content_height = total_rows as f32 * row_height;
            let max_scroll = (total_content_height - content_rect.height()).max(0.0) + 80.0;

            if total_content_height <= content_rect.height() {
                state.scroll_offset = 0.0;
            } else {
                let scroll_speed = 14.0;
                let scroll_zone = 60.0;

                if current.y > content_rect.max.y - scroll_zone {
                    let distance = (current.y - (content_rect.max.y - scroll_zone)) / scroll_zone;
                    let acceleration = distance * distance;
                    state.scroll_offset += scroll_speed * acceleration;
                }

                if current.y < content_rect.min.y + scroll_zone {
                    let distance = ((content_rect.min.y + scroll_zone) - current.y) / scroll_zone;
                    let acceleration = distance * distance;
                    state.scroll_offset -= scroll_speed * acceleration;
                }

                state.scroll_offset = state.scroll_offset.clamp(0.0, max_scroll);
            }
        }
    }

    if bg_response.drag_stopped() {
        state.rubber_band.is_rubber_banding = false;
        state.rubber_band.rubber_band_start = None;
        state.rubber_band.rubber_band_current = None;
    }

    let cwd = state.cwd.clone();
    let is_in_trash = get_backend().etched_in_trash_path(&cwd);

    if is_in_trash {
        let tab_id = state.active_id;
        let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
        if bg_response.secondary_clicked() {
            ui_state.context_menu_state.handle_response(&bg_response);
            ui_state.context_menu_state.kind = ContextMenuKind::BackgroundTrash;
            ui_state.context_menu_state.target_sender = Some(dispatcher);
        }
    } else {
        if bg_response.secondary_clicked() {
            state.deselect_all();
            state.resize_selection(files.len());
            ui_state.context_menu_state.handle_response(&bg_response);
            ui_state.context_menu_state.kind = ContextMenuKind::BackgroundNormal;
        }
    }

    let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);
    match ctx_menu.kind {
        ContextMenuKind::BackgroundNormal => ctx_menu.background_context_menu(ui, state, ui_state),
        ContextMenuKind::BackgroundTrash => {
            ctx_menu.background_context_menu_in_trash(ui, state, ui_state, files)
        }
        _ => {}
    }
    ui_state.context_menu_state = ctx_menu;

    if bg_response.clicked() {
        state.deselect_all();
    }
}

pub fn row_panel_frame(
    ui: &mut Ui,
    files: &[Arc<FileEntry>],
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    bottom_padding: i8,
    tabs_height: i8,
) {
    let current_theme = with_theme(|t| t.current());

    Frame::NONE
        .inner_margin(Margin {
            left: 10,
            right: 10,
            top: 0,
            bottom: 10,
        })
        .fill(current_theme.bg_panel.to_color())
        .corner_radius(CornerRadius {
            nw: 0,
            ne: 0,
            sw: 20,
            se: 20,
        })
        .stroke(Stroke {
            width: 0.5,
            color: current_theme.accent_glow.to_color(),
        })
        .show(ui, |ui| {
            let original_clip = ui.clip_rect();
            let frame_rect = ui.max_rect();
            ui.set_clip_rect(frame_rect);

            let content_rect = ui.viewport_rect();
            let panel_top = content_rect.min.y;
            let clipped_painter = &ui.painter_at(content_rect);

            let icon_size = state.row_view.icon_size;

            let row_height = (icon_size + 8.0).clamp(28.0, 64.0);
            let total_rows = files.len();

            //Drag
            if state.row_view.is_dragging_files {
                drag_files(ui, state, files, clipped_painter, content_rect, row_height);
            }

            //Rubberband
            if state.rubber_band.is_rubber_banding {
                render_row_rubberband(
                    state,
                    files,
                    clipped_painter,
                    panel_top,
                    content_rect,
                    row_height,
                );
            }

            if !ui.memory(|m| m.focused().is_some()) {
                ui.memory_mut(|m| m.request_focus(ui.id()));
            }

            //Background
            background_response_logic(
                state,
                ui_state,
                files,
                ui,
                panel_top,
                total_rows,
                row_height,
                content_rect,
            );

            let tab_id = state.active_id;
            let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));

            //Disparador de sizer
            for file in files
                .iter()
                .take(state.row_view.last_visible.min(files.len()))
                .skip(state.row_view.first_visible)
            {
                if file.is_dir()
                    && !state.calculating_dir_sizes.contains(&file.full_path)
                    && !state.calculated_dir_sizes.contains(&file.full_path)
                {
                    state.calculating_dir_sizes.insert(file.full_path.clone());
                    if let Err(e) = dispatcher.send(SizerMessages::StartCal(
                        file.full_path.to_owned(),
                        Uuid::new_v4(),
                    )) {
                        warn!("Error enviando Sizer: {}", e);
                    }
                }
            }

            //Disparador de Info extendida
            for file in files
                .iter()
                .take(state.row_view.last_visible.min(files.len()))
                .skip(state.row_view.first_visible)
            {
                if !state.calculating_extended_info.contains(&file.full_path)
                    && !state.calculated_extended_info.contains(&file.full_path)
                {
                    state
                        .calculating_extended_info
                        .insert(file.full_path.clone());
                    if let Err(e) =
                        dispatcher.send(ExtendedInfoMessages::StartScan(file.full_path.to_owned()))
                    {
                        warn!("Error enviando Sizer: {}", e);
                    }
                }
            }

            //disparador de thumbnails
            for file in files
                .iter()
                .take(state.row_view.last_visible.min(files.len()))
                .skip(state.row_view.first_visible)
            {
                let img_vid = file.extension.is_image() || file.extension.is_video();

                if !ui_state.calculating_thumbnails.contains(&file.full_path)
                    && !ui_state.calculated_thumbnails.contains(&file.full_path)
                    && img_vid
                {
                    let sent = dispatcher
                        .send(ThumbnailMessages::RequestThumb(file.full_path.clone()))
                        .is_ok();
                    if sent {
                        ui_state
                            .calculating_thumbnails
                            .insert(file.full_path.clone());
                    }
                }
            }

            //Scrollview
            new_render_scrollview(
                ui,
                files,
                state,
                ui_state,
                row_height,
                total_rows,
                content_rect,
            );

            if state.row_view.is_dragging_files && !ui.input(|i| i.pointer.any_down()) {
                state.row_view.is_dragging_files = false;
                state.row_view.drag_ghost_pos = None;
                state.row_view.drop_target = None;
                state.row_view.drop_invalid_target = None;
            }

            //Isla y burbuja y las tabs
            render_island_bubble(ui, state, ui_state, files, bottom_padding, tabs_height);

            //hotkeys
            hot_keys_logic(state, ui_state, files, ui, total_rows);

            ui.set_clip_rect(original_clip);
        });
}
