use std::sync::Arc;

use crate::{
    core::{
        blaze_state::BlazeCoreState,
        bootstrap::configs::{
            config_manager::with_configs, platform::linux::conf_structs::OrderingMode,
        },
        system::{
            clipboard::global_clipboard::TOKIO_RUNTIME,
            knowndirs::knowndirs_manager::KnownDirsManager,
            trash_manager::manager::{TrashDestination, get_backend},
        },
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::icons,
        modules::{
            custom_context_menu::context_state::ContextMenuKind,
            sidebar_left_component::sidebar_components::{
                render_drives_button, render_header_text, render_local_buttons,
            },
        },
        themes::{platform::structs::ToColor, theme_manager::with_theme},
    },
};
use egui::{
    Area, Color32, CornerRadius, Frame, Id, Margin, Panel, Rect, ScrollArea, Sense, Stroke, Ui,
    pos2, scroll_area::ScrollBarVisibility, vec2,
};

use crate::ui::modules::sidebar_left_component::leftbar_state::{
    MIN_PANEL_WIDTH_TOLERANCE, PANEL_TAB_WIDTH,
};

fn render_expanded_panel(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    current_order: OrderingMode,
) {
    let current_theme = with_theme(|t| t.current());
    let i18n = with_configs(|c| c.get_i18n());

    let custom_frame = Frame::NONE
        .fill(current_theme.semantic.bg_main.to_color())
        .inner_margin(Margin {
            left: 15,
            right: 0,
            top: 0,
            bottom: 10,
        });

    let left_response = Panel::left("LeftSidePanel")
        .show_separator_line(false)
        .min_size(PANEL_TAB_WIDTH)
        .max_size(400.0)
        .default_size(current_order.leftpanel_state.width)
        .resizable(true)
        .frame(custom_frame)
        .show(ui, |ui| {
            //ui.set_width(current_order.leftpanel_state.width);

            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(current_theme.components.panel.bg.to_color())
                .corner_radius(CornerRadius::same(20))
                .stroke(Stroke {
                    width: 0.5,
                    color: current_theme.components.panel.border.to_color(),
                })
                .show(ui, |ui| {
                    //ui.set_width(current_order.leftpanel_state.width);
                    ui.set_height(ui.available_height());

                    ScrollArea::vertical()
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            render_header_text(
                                "locals",
                                &i18n.t("left_sidebar.locals"),
                                ui,
                                ui_state,
                            );

                            ui.add_space(10.0);

                            let kdm = KnownDirsManager::get();
                            let mut dirs = kdm.sidebar_dirs();

                            let Some(trash) =
                                get_backend().get_trash_files(&TrashDestination::Home).ok()
                            else {
                                return;
                            };

                            dirs.push(("trash", i18n.t("left_sidebar.trash"), &trash));

                            for (key, label, path) in dirs {
                                if path.exists() {
                                    render_local_buttons(
                                        key,
                                        &label,
                                        path.to_owned(),
                                        state,
                                        ui,
                                        ui_state,
                                    );
                                    ui.add_space(2.0);
                                }
                            }

                            ui.add_space(20.0);
                            ui.separator();

                            ui.add_space(10.0);
                            render_header_text(
                                "disks",
                                &i18n.t("left_sidebar.disks"),
                                ui,
                                ui_state,
                            );
                            ui.add_space(10.0);

                            let manager = state.motor.borrow_mut().disk_manager.clone();
                            let drives = TOKIO_RUNTIME.block_on(async {
                                let manager = manager.lock().await;
                                manager.get_partitions().await
                            });

                            ui.vertical(|ui| {
                                for drive in drives {
                                    render_drives_button(ui, state, drive, ui_state);
                                    ui.add_space(2.0);
                                }
                            });

                            let mut ctx_menu = std::mem::take(&mut ui_state.context_menu_state);

                            if ctx_menu.kind == ContextMenuKind::DrivesPanel {
                                ctx_menu.render_drives_context(ui, state, ui_state)
                            }

                            ui_state.context_menu_state = ctx_menu;
                        });
                });
        })
        .response;

    let new_width = left_response.rect.width();

    if new_width < MIN_PANEL_WIDTH_TOLERANCE
        && current_order.leftpanel_state.width >= MIN_PANEL_WIDTH_TOLERANCE
    {
        with_configs(|c| {
            let mut current_ordering = c.get_ordering_mode();
            current_ordering.leftpanel_state.collapsed = true;
            c.set_ordering_mode(current_ordering);
        });
    } else if new_width >= MIN_PANEL_WIDTH_TOLERANCE {
        with_configs(|c| {
            let mut current_ordering = c.get_ordering_mode();
            current_ordering.leftpanel_state.width = new_width;
            c.set_ordering_mode(current_ordering);
        });
    }
}

fn render_collapsed_tab(
    ui: &mut Ui,
    ui_state: &mut BlazeUiState,
    current_theme: Arc<crate::ui::themes::platform::structs::NewTheme>,
) {
    let area_id = Id::new("left_tab");
    let hover_state_id = Id::new("left_tab_hover_state");

    let mut is_hovered = ui.data(|data| data.get_temp::<bool>(hover_state_id).unwrap_or(false));

    Panel::left("left_panel_tab")
        .exact_size(PANEL_TAB_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .show(ui, |ui| {
            let rect: Rect = ui.max_rect();
            let area_height = 20.0;

            let visual_pos = pos2(rect.left() - 2.0, rect.center().y - area_height / 2.0);

            let area_response = Area::new(area_id)
                .fixed_pos(visual_pos)
                .sense(Sense::all())
                .show(ui, |ui| {
                    let area_rect = ui.max_rect();
                    let frame_width = if is_hovered { 20.0 } else { 5.0 };

                    Frame::NONE
                        .inner_margin(Margin::same(10))
                        .fill(current_theme.components.panel.bg.to_color())
                        .stroke(Stroke::new(
                            0.5,
                            current_theme.components.panel.border.to_color(),
                        ))
                        .corner_radius(CornerRadius::same(20))
                        .show(ui, |ui| {
                            ui.set_width(frame_width);
                            ui.set_height(area_height / 2.0);

                            let area_center = area_rect.center();

                            let icon_size = 18.0;

                            let t_icon_rect =
                                Rect::from_center_size(area_center, vec2(icon_size, icon_size));
                            let (icon_n, icon_b) = ("left_sidebar", icons::ICON_SIDEBAR_LEFT);

                            let icon = ui_state.icon_cache.get_or_load(
                                ui,
                                icon_n,
                                icon_b,
                                Color32::GRAY,
                                vec2(icon_size, icon_size),
                            );

                            let rounded_rect = Rect::from_min_max(
                                pos2(t_icon_rect.min.x.round(), t_icon_rect.min.y.round()),
                                pos2(t_icon_rect.max.x.round(), t_icon_rect.max.y.round()),
                            );

                            ui.painter().image(
                                icon.id(),
                                rounded_rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        });
                })
                .response;

            if area_response.clicked() {
                with_configs(|c| {
                    let mut current_ordering = c.get_ordering_mode();
                    current_ordering.leftpanel_state.collapsed = false;
                    c.set_ordering_mode(current_ordering);
                });
            }

            let new_hovered = area_response.hovered();

            if new_hovered != is_hovered {
                is_hovered = new_hovered;
                ui.data_mut(|data| {
                    data.insert_temp(hover_state_id, is_hovered);
                });
                ui.request_repaint();
            }
        });
}

pub fn sidebar_left_component(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
) {
    let current_theme = with_theme(|t| t.current());
    let current_order = with_configs(|c| c.get_ordering_mode());

    if current_order.leftpanel_state.collapsed {
        render_collapsed_tab(ui, ui_state, current_theme);
    } else {
        render_expanded_panel(ui, state, ui_state, current_order);
    }
}
