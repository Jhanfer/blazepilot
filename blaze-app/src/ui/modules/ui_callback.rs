use crate::{
    core::blaze_state::BlazeCoreState,
    ui::{
        blaze_ui_state::BlazeUiState,
        modules::{
            file_view_callback::render_views,
            sidebar_left_component::sidebar_callback::sidebar_left_component,
            sidebar_right_component::sidebar_right::sidebar_right_component,
            toolbar::toolbar_component,
        },
    },
};
use egui::Ui;

pub fn connect_ui_components_callback(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
) {
    // -------------------------------
    //  Toolbar
    // -------------------------------
    toolbar_component(ui, state, ui_state);

    // -------------------------------
    //  sidebar left
    // -------------------------------
    sidebar_left_component(ui, state, ui_state);

    // -------------------------------
    //  sidebar right
    // -------------------------------
    sidebar_right_component(ui, state, ui_state);

    // -------------------------------
    //  file view
    // -------------------------------
    render_views(ui, state, ui_state);
}
