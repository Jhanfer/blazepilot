use std::sync::Arc;

use egui::{Align2, Area, Color32, CornerRadius, Frame};

use crate::{core::{blaze_state::BlazeCoreState, files::motor::FileEntry}, ui::{blaze_ui_state::BlazeUiState, modules::{row_view::row_view_callback::render_row_view, sidebar_left::sidebar_left_component, sidebar_right::sidebar_right_component, toolbar::toolbar_component}}};


pub fn connect_ui_components_callback(ctx: &egui::Context, files: &Vec<Arc<FileEntry>>, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {

    // -------------------------------
    //  Toolbar     
    // -------------------------------
    toolbar_component(ctx, state);


    // -------------------------------
    //  sidebar left     
    // -------------------------------
    sidebar_left_component(ctx, state, ui_state);


    // -------------------------------
    //  sidebar right     
    // -------------------------------
    sidebar_right_component(state, files, ctx);


    // -------------------------------
    //  file view 
    // -------------------------------
    render_row_view(ctx, files, state, ui_state);

}