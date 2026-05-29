use std::sync::Arc;
use egui::{
    CentralPanel, 
    Frame, 
    Margin, 
    Ui, 
};
use crate::{
    core::{
        blaze_state::{
            BlazeCoreState,
            ViewMode
        },
        files::blaze_motor::motor_structs::FileEntry, 
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        modules::row_view::{
                render_row_panel_view::row_panel_frame,
                render_tags_view::tag_views,
                tools_view::tools,
            },
        themes::colors::*,
    },
};



pub fn render_row_view(ui: &mut Ui, files: &Vec<Arc<FileEntry>>, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    state.resize_selection(files.len());

    let tabs_height: i8 = if state.motor.borrow_mut().tabs.len() > 1 {
        50
    } else {
        0
    };

    let bottom_padding = 10.0 as i8;
    
    let custom_frame = Frame::NONE
        .fill(COLOR_BG_MAIN)
        .inner_margin(Margin {
            left: 15,
            right: 15,
            top: 0,
            bottom: bottom_padding + tabs_height,
        });

    CentralPanel::default()
        .frame(custom_frame)
        .show_inside(ui, |ui| {
        
        ui.spacing_mut().item_spacing.y = 0.0;

        tools(state, ui_state, files, ui);

        match state.view_mode {
            ViewMode::Normal => {
                row_panel_frame(ui, files, state, ui_state, bottom_padding, tabs_height);
            },
            ViewMode::Tags => {
                tag_views(ui, state, ui_state, bottom_padding, tabs_height);
            },
        }


    });
}