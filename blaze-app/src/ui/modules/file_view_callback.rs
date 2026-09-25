use crate::{
    core::{
        blaze_state::BlazeCoreState,
        blaze_state::state_structs::{LayoutMode, ViewMode},
        bootstrap::configs::config_manager::with_configs,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        modules::{
            compact_view::compact_row_panel::compact_row_panel_frame,
            detailed_view::detailed_row_panel::detailed_row_panel_frame,
            grid_view::render_grid_panel_view::grid_panel_frame,
            miller_view::miller_panel_view::miller_panel_frame, render_tags_view::tag_views,
            row_view::render_row_panel_view::row_panel_frame, tools_view::tools,
        },
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{CentralPanel, Event, Frame, Margin, MouseWheelUnit, Ui};

pub fn render_views(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    let current_theme = with_theme(|t| t.current());

    let tabs_height: i8 = if state.motor.borrow_mut().tabs.len() > 1 {
        50
    } else {
        0
    };

    let bottom_padding = 10.0 as i8;

    let custom_frame = Frame::NONE
        .fill(current_theme.semantic.bg_main.to_color())
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

    const ROW_MIN_ICON_SIZE: f32 = 10.0;
    const ROW_TO_MILLER_THRESHOLD: f32 = 14.0;

    const MILLER_MIN_ICON_SIZE: f32 = 12.0;
    const MILLER_MAX_ICON_SIZE: f32 = 48.0;
    const MILLER_TO_ROW_THRESHOLD: f32 = 20.0;

    const ROW_TO_GRID_THRESHOLD: f32 = 40.0;
    const GRID_TO_ROW_THRESHOLD: f32 = 36.0;

    if ctrl_scroll != 0.0 {
        match &state.view_mode {
            ViewMode::Normal(LayoutMode::Row) => {
                let new_size =
                    (state.row_view.icon_size + ctrl_scroll * 0.3).clamp(ROW_MIN_ICON_SIZE, 48.0);

                state.row_view.icon_size = new_size;
                with_configs(|c| c.set_row_icon_size(new_size));
            }

            ViewMode::Normal(LayoutMode::Compact) => {
                let new_size =
                    (state.row_view.icon_size + ctrl_scroll * 0.3).clamp(ROW_MIN_ICON_SIZE, 48.0);

                state.row_view.icon_size = new_size;
                with_configs(|c| c.set_compact_icon_size(new_size));
            }

            ViewMode::Normal(LayoutMode::RowDetailed) => {
                let new_size =
                    (state.row_view.icon_size + ctrl_scroll * 0.3).clamp(ROW_MIN_ICON_SIZE, 48.0);

                state.row_view.icon_size = new_size;
                with_configs(|c| c.set_detailed_icon_size(new_size));
            }

            ViewMode::Normal(LayoutMode::Grid) => {
                let new_size =
                    (state.grid_view.base.icon_size + ctrl_scroll * 0.3).clamp(32.0, 128.0);

                state.grid_view.base.icon_size = new_size;
                with_configs(|c| c.set_grid_icon_size(new_size));
            }

            ViewMode::Normal(LayoutMode::Miller) => {
                let new_size = (state.miller_view.base.icon_size + ctrl_scroll * 0.3)
                    .clamp(MILLER_MIN_ICON_SIZE, MILLER_MAX_ICON_SIZE);

                state.miller_view.base.icon_size = new_size;
                with_configs(|c| c.set_miller_icon_size(new_size));
            }

            _ => {}
        }

        match &state.view_mode {
            ViewMode::Normal(LayoutMode::Row)
                if state.row_view.icon_size <= ROW_TO_MILLER_THRESHOLD =>
            {
                state.miller_view.base.icon_size = MILLER_MIN_ICON_SIZE;

                let new_mode = ViewMode::Normal(LayoutMode::Miller);
                state.set_view_mode(new_mode.clone());
            }

            ViewMode::Normal(LayoutMode::Compact)
                if state.row_view.icon_size <= ROW_TO_MILLER_THRESHOLD =>
            {
                state.miller_view.base.icon_size = MILLER_MIN_ICON_SIZE;

                let new_mode = ViewMode::Normal(LayoutMode::Miller);
                state.set_view_mode(new_mode.clone());
            }

            ViewMode::Normal(LayoutMode::RowDetailed)
                if state.row_view.icon_size <= ROW_TO_MILLER_THRESHOLD =>
            {
                state.miller_view.base.icon_size = MILLER_MIN_ICON_SIZE;

                let new_mode = ViewMode::Normal(LayoutMode::Miller);
                state.set_view_mode(new_mode.clone());
            }

            ViewMode::Normal(LayoutMode::Row)
                if state.row_view.icon_size >= ROW_TO_GRID_THRESHOLD =>
            {
                state.grid_view.base.icon_size = state.row_view.icon_size.clamp(32.0, 128.0);

                let new_mode = ViewMode::Normal(LayoutMode::Grid);
                state.set_view_mode(new_mode.clone());
            }

            ViewMode::Normal(LayoutMode::Compact)
                if state.row_view.icon_size >= ROW_TO_GRID_THRESHOLD =>
            {
                state.grid_view.base.icon_size = state.row_view.icon_size.clamp(32.0, 128.0);

                let new_mode = ViewMode::Normal(LayoutMode::Grid);
                state.set_view_mode(new_mode.clone());
            }

            ViewMode::Normal(LayoutMode::RowDetailed)
                if state.row_view.icon_size >= ROW_TO_GRID_THRESHOLD =>
            {
                state.grid_view.base.icon_size = state.row_view.icon_size.clamp(32.0, 128.0);

                let new_mode = ViewMode::Normal(LayoutMode::Grid);
                state.set_view_mode(new_mode.clone());
            }

            ViewMode::Normal(LayoutMode::Miller)
                if state.miller_view.base.icon_size >= MILLER_TO_ROW_THRESHOLD =>
            {
                state.row_view.icon_size = MILLER_TO_ROW_THRESHOLD;

                let new_mode = ViewMode::Normal(LayoutMode::Row);
                state.set_view_mode(new_mode.clone());
            }

            ViewMode::Normal(LayoutMode::Grid)
                if state.grid_view.base.icon_size <= GRID_TO_ROW_THRESHOLD =>
            {
                state.row_view.icon_size = state.grid_view.base.icon_size.clamp(12.0, 48.0);

                let new_mode = ViewMode::Normal(LayoutMode::Row);
                state.set_view_mode(new_mode.clone());
            }

            _ => {}
        }
    }

    CentralPanel::default().frame(custom_frame).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        tools(state, ui_state, ui);

        match &state.view_mode {
            ViewMode::Normal(layout) => match layout.to_owned() {
                LayoutMode::Row => {
                    row_panel_frame(ui, state, ui_state, bottom_padding, tabs_height);
                }

                LayoutMode::RowDetailed => {
                    detailed_row_panel_frame(ui, state, ui_state, bottom_padding, tabs_height);
                }

                LayoutMode::Grid => {
                    grid_panel_frame(ui, state, ui_state, bottom_padding, tabs_height);
                }

                LayoutMode::Compact => {
                    compact_row_panel_frame(ui, state, ui_state, bottom_padding, tabs_height);
                }

                LayoutMode::Miller => {
                    miller_panel_frame(ui, state, ui_state, bottom_padding, tabs_height)
                }
            },
            ViewMode::Tags(_) => {
                tag_views(ui, state, ui_state, bottom_padding, tabs_height);
            }
        }
    });
}
