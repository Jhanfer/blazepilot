use crate::{
    core::{
        blaze_state::{BlazeCoreState, LayoutMode, ViewMode},
        bootstrap::configs::config_manager::with_configs,
        files::blaze_motor::motor_structs::FileEntry,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        modules::{
            grid_view::render_grid_panel_view::grid_panel_frame, render_tags_view::tag_views,
            row_view::render_row_panel_view::row_panel_frame, tools_view::tools,
        },
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{CentralPanel, Event, Frame, Margin, MouseWheelUnit, Ui};
use std::sync::Arc;

pub fn render_views(
    ui: &mut Ui,
    files: &[Arc<FileEntry>],
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
) {
    let current_theme = with_theme(|t| t.current());

    state.resize_selection(files.len());

    let tabs_height: i8 = if state.motor.borrow_mut().tabs.len() > 1 {
        50
    } else {
        0
    };

    let bottom_padding = 10.0 as i8;

    let custom_frame = Frame::NONE
        .fill(current_theme.bg_main.to_color())
        .inner_margin(Margin {
            left: 15,
            right: 15,
            top: 0,
            bottom: bottom_padding + tabs_height,
        });

    let ctrl_scroll = ui.input(|i| {
        if i.modifiers.ctrl {
            i.events
                .iter()
                .filter_map(|e| match e {
                    Event::MouseWheel {
                        unit: MouseWheelUnit::Line,
                        delta,
                        ..
                    } => Some(delta.y * 10.0),
                    _ => None,
                })
                .sum()
        } else {
            0.0
        }
    });

    if ctrl_scroll != 0.0 {
        match &state.view_mode {
            ViewMode::Normal(LayoutMode::Row) => {
                let new_size = (state.row_view.icon_size + ctrl_scroll * 0.3).clamp(12.0, 48.0);
                state.row_view.icon_size = new_size;
                with_configs(|c| c.set_row_icon_size(new_size));
            }
            ViewMode::Normal(LayoutMode::Grid) => {
                let new_size = (state.grid_view.icon_size + ctrl_scroll * 0.3).clamp(32.0, 128.0);
                state.grid_view.icon_size = new_size;
                with_configs(|c| c.set_grid_icon_size(new_size));
            }
            _ => {}
        }
    }

    const SWITCH_TO_GRID_THRESHOLD: f32 = 33.0;
    const SWITCH_TO_ROW_THRESHOLD: f32 = 32.0;

    match &state.view_mode {
        ViewMode::Normal(LayoutMode::Row)
            if state.row_view.icon_size >= SWITCH_TO_GRID_THRESHOLD =>
        {
            state.grid_view.icon_size = state.row_view.icon_size.clamp(32.0, 128.0);
            let new_mode = ViewMode::Normal(LayoutMode::Grid);
            state.view_mode = new_mode.clone();
            with_configs(|c| c.set_view_mode(new_mode));
        }
        ViewMode::Normal(LayoutMode::Grid)
            if state.grid_view.icon_size <= SWITCH_TO_ROW_THRESHOLD =>
        {
            state.row_view.icon_size = state.grid_view.icon_size.clamp(12.0, 48.0);
            let new_mode = ViewMode::Normal(LayoutMode::Row);
            state.view_mode = new_mode.clone();
            with_configs(|c| c.set_view_mode(new_mode));
        }
        _ => {}
    }

    CentralPanel::default()
        .frame(custom_frame)
        .show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            tools(state, ui_state, files, ui);

            match &state.view_mode {
                ViewMode::Normal(layout) => match layout.to_owned() {
                    LayoutMode::Row => {
                        row_panel_frame(ui, files, state, ui_state, bottom_padding, tabs_height);
                    }
                    LayoutMode::Grid => {
                        grid_panel_frame(ui, files, state, ui_state, bottom_padding, tabs_height);
                    }
                },
                ViewMode::Tags(_) => {
                    tag_views(ui, state, ui_state, bottom_padding, tabs_height);
                }
            }
        });
}
