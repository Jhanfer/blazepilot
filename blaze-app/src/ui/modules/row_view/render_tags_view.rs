use crate::{
    core::{
        blaze_state::{BlazeCoreState, TagViewFilter, ViewMode},
        bootstrap::{
            configs::config_manager::with_configs, quick_access_manager::manager::with_quick_tags,
        },
        runtime::{
            bus_structs::{FileOperation, QuickTagEvent, UiEvent},
            event_bus::with_event_bus,
        },
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::thumbnails::thumbnails_manager::Thumbnail,
        modules::row_view::{
            island_n_bubble::render_tags_island_bubble,
            utilities::{
                ensure_min_lightness, render_button, render_op_buttons, render_quicklink_icon,
            },
        },
        themes::colors::*,
    },
    utils::formating::{format_date, format_size},
};
use egui::{
    lerp, pos2, vec2, Align, Color32, CornerRadius, Frame, Label, Layout, Margin, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui, UiBuilder,
};
use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
use tracing::info;
use uuid::Uuid;

pub fn tag_views(
    ui: &mut Ui,
    state: &mut BlazeCoreState,
    ui_state: &mut BlazeUiState,
    bottom_padding: i8,
    tabs_height: i8,
) {
    let mut tag_len: usize = 0;

    let i18n = with_configs(|c| c.get_i18n());

    Frame::NONE
        .inner_margin(Margin {
            left: 10,
            right: 10,
            top: 0,
            bottom: 10,
        })
        .fill(COLOR_BG_PANEL)
        .corner_radius(CornerRadius {
            nw: 0,
            ne: 0,
            sw: 20,
            se: 20,
        })
        .stroke(Stroke {
            width: 0.5,
            color: COLOR_ACCENT_GLOW,
        })
        .show(ui, |ui| {
            ScrollArea::vertical().id_salt("all_tags").show(ui, |ui| {
                if !ui.memory(|m| m.focused().is_some()) {
                    ui.memory_mut(|m| m.request_focus(ui.id()));
                }

                let tab_id = state.active_id;
                let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));

                let tags = with_quick_tags(|qtm| qtm.get_tags());

                let (tags_names, tags_colors, tags_ids, tags_items_len): (
                    Vec<String>,
                    Vec<Color32>,
                    Vec<Uuid>,
                    Vec<usize>,
                ) = tags
                    .iter()
                    .map(|t| (t.title.to_string(), t.color, t.id, t.items.len()))
                    .clone()
                    .collect();

                tag_len = tags.len();
                let total_items: usize = tags_items_len.iter().sum();

                if let TagViewFilter::All {
                    ref mut all_items_len,
                } = state.tag_filter
                {
                    *all_items_len = total_items;
                }

                ui.add_space(10.0);

                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 38.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        render_button(
                            ui,
                            &i18n.t("tags_quick.new"),
                            COLOR_BG_MAIN,
                            COLOR_ACCENT_GLOW,
                            Some(|| {
                                dispatcher
                                    .send(UiEvent::QuickTagEvent(QuickTagEvent::CreateNewTag {
                                        title: String::new(),
                                        temp_color: Color32::GRAY,
                                    }))
                                    .ok();
                            }),
                            None::<fn(&mut Ui)>,
                        );

                        render_button(
                            ui,
                            &i18n.t("tags_quick.all"),
                            COLOR_BG_MAIN,
                            COLOR_ACCENT_GLOW,
                            Some(|| {
                                state.tag_filter = TagViewFilter::All {
                                    all_items_len: total_items,
                                };
                            }),
                            None::<fn(&mut Ui)>,
                        );

                        ScrollArea::horizontal()
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                            )
                            .max_height(28.0)
                            .id_salt("tag_scroll")
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for (tag_name, ((tag_color, tag_id), items_len)) in
                                        tags_names.iter().zip(
                                            tags_colors
                                                .iter()
                                                .zip(tags_ids.iter())
                                                .zip(tags_items_len.iter()),
                                        )
                                    {
                                        let vivid_color = ensure_min_lightness(*tag_color, 0.45);

                                        let color = Color32::from_rgba_unmultiplied(
                                            vivid_color.r(),
                                            vivid_color.g(),
                                            vivid_color.b(),
                                            60,
                                        );

                                        let text_color = ensure_min_lightness(color, 0.70);

                                        render_button::<_, _>(
                                            ui,
                                            tag_name,
                                            color,
                                            text_color,
                                            Some(|| {
                                                state.tag_filter = TagViewFilter::Tag {
                                                    name: tag_name.clone(),
                                                    items_len: *items_len,
                                                };
                                            }),
                                            Some(|ui: &mut Ui| {
                                                if ui.button(&i18n.t("tags_quick.edit")).clicked() {
                                                    dispatcher
                                                        .send(UiEvent::QuickTagEvent(
                                                            QuickTagEvent::EditCurrentTag {
                                                                id: *tag_id,
                                                                title: tag_name.to_string(),
                                                                temp_color: *tag_color,
                                                            },
                                                        ))
                                                        .ok();
                                                }

                                                if ui.button(&i18n.t("tags_quick.delete")).clicked()
                                                {
                                                    dispatcher
                                                        .send(UiEvent::QuickTagEvent(
                                                            QuickTagEvent::DeleteCurrentTag {
                                                                title: tag_name.to_owned().into(),
                                                                id: *tag_id,
                                                            },
                                                        ))
                                                        .ok();
                                                }
                                            }),
                                        );
                                    }
                                });
                            });
                    },
                );

                if tags.is_empty() {
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), 80.0),
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.label(
                                egui::RichText::new(&*i18n.t("tags_quick.empty"))
                                    .color(COLOR_TEXT_SECONDARY)
                                    .size(14.0)
                                    .strong(),
                            );
                        },
                    );
                }

                ui.add_space(10.0);

                ui.vertical(|ui| {
                    for tag in tags.iter().filter(|t| match &state.tag_filter {
                        TagViewFilter::All { .. } => true,
                        TagViewFilter::Tag { name, .. } => *t.title == *name,
                    }) {
                        let vivid_color = ensure_min_lightness(tag.color, 0.45);

                        let color = Color32::from_rgba_unmultiplied(
                            vivid_color.r(),
                            vivid_color.g(),
                            vivid_color.b(),
                            60,
                        );

                        let text_color = ensure_min_lightness(color, 0.70);

                        Frame::NONE
                            .corner_radius(CornerRadius::same(20))
                            .fill(color)
                            .stroke(Stroke::new(0.8, vivid_color))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(15.0);

                                    render_button::<_, _>(
                                        ui,
                                        &tag.title,
                                        color,
                                        text_color,
                                        None::<fn()>,
                                        None::<fn(&mut Ui)>,
                                    );

                                    ui.add(
                                        Label::new(
                                            RichText::new(&*i18n.t_args(
                                                "tags_quick.count",
                                                &[("query", &tag.items.len().to_string())],
                                            ))
                                            .color(COLOR_TEXT_SECONDARY)
                                            .size(11.0),
                                        )
                                        .selectable(false),
                                    );

                                    render_op_buttons(
                                        ui,
                                        ui_state,
                                        color,
                                        || {
                                            dispatcher
                                                .send(UiEvent::QuickTagEvent(
                                                    QuickTagEvent::EditCurrentTag {
                                                        id: tag.id,
                                                        title: tag.title.to_string(),
                                                        temp_color: tag.color,
                                                    },
                                                ))
                                                .ok();
                                        },
                                        || {
                                            dispatcher
                                                .send(UiEvent::QuickTagEvent(
                                                    QuickTagEvent::DeleteCurrentTag {
                                                        title: tag.title.clone(),
                                                        id: tag.id,
                                                    },
                                                ))
                                                .ok();
                                        },
                                    );
                                });

                                ui.add_space(10.0);

                                let thumb_snapshot: HashMap<Arc<Path>, Thumbnail> = {
                                    match ui_state.thumbnail_manager.thumb_map.try_write() {
                                        Ok(mut guard) => tags
                                            .iter()
                                            .flat_map(|t| &t.items)
                                            .filter_map(|item| {
                                                guard
                                                    .get(&item.path)
                                                    .cloned()
                                                    .map(|t| (item.path.clone(), t))
                                            })
                                            .collect(),
                                        Err(_) => HashMap::new(),
                                    }
                                };

                                ui.vertical(|ui| {
                                    let items: Vec<_> = tag.items.iter().collect();
                                    let total = items.len();

                                    if tag.items.is_empty() {
                                        ui.allocate_ui_with_layout(
                                            vec2(ui.available_width(), 40.0),
                                            Layout::centered_and_justified(
                                                egui::Direction::BottomUp,
                                            ),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(
                                                        &*i18n.t("tags_quick.add_something"),
                                                    )
                                                    .color(COLOR_TEXT_SECONDARY)
                                                    .size(14.0)
                                                    .strong(),
                                                );
                                            },
                                        );
                                    }

                                    for (i, item) in items.iter().enumerate() {
                                        if item.needs_refresh(Duration::from_secs(30)) {
                                            item.refresh_meta();
                                        }

                                        let row_height = 40.0;
                                        let full_width = ui.available_width();

                                        let (rect, response) = ui.allocate_exact_size(
                                            vec2(full_width, row_height),
                                            Sense::click(),
                                        );

                                        let hover_t = ui.animate_bool(
                                            response.id.with("hover_rect"),
                                            response.hovered(),
                                        );
                                        let press_t = ui.animate_bool(
                                            response.id.with("press_rect"),
                                            response.is_pointer_button_down_on(),
                                        );

                                        let bg_alpha = lerp(0.0..=30.0, hover_t) as u8;
                                        let bg_alpha = if press_t > 0.0 {
                                            (bg_alpha as f32 * 0.6) as u8
                                        } else {
                                            bg_alpha
                                        };

                                        if response.hovered() {
                                            ui.set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }

                                        let is_first = i == 0;
                                        let is_last = i == total - 1;
                                        let is_only = total == 1;

                                        let corner_radius = match (is_first, is_last) {
                                            _ if is_only => CornerRadius {
                                                nw: 0,
                                                ne: 0,
                                                sw: 20,
                                                se: 20,
                                            },
                                            (true, _) => CornerRadius::same(0),
                                            (_, true) => CornerRadius {
                                                nw: 0,
                                                ne: 0,
                                                sw: 20,
                                                se: 20,
                                            },
                                            _ => CornerRadius::same(0),
                                        };

                                        response.context_menu(|ui| {
                                            if ui.button(&i18n.t("tags_quick.edit")).clicked() {
                                                dispatcher
                                                    .send(UiEvent::QuickTagEvent(
                                                        QuickTagEvent::EditCurrentQuickLink {
                                                            tag_id: tag.id,
                                                            quick_id: item.id,
                                                            title: item.name.to_string(),
                                                            temp_color: item.color,
                                                        },
                                                    ))
                                                    .ok();
                                            }

                                            if ui.button(&i18n.t("tags_quick.delete")).clicked() {
                                                dispatcher
                                                    .send(UiEvent::QuickTagEvent(
                                                        QuickTagEvent::DeleteQuickLink {
                                                            tag_id: tag.id,
                                                            quick_id: item.id,
                                                            quick_title: item.name.to_owned(),
                                                        },
                                                    ))
                                                    .ok();
                                            }
                                        });

                                        if response.clicked() {
                                            if item.is_dir {
                                                let path = item.path.to_owned();
                                                dispatcher
                                                    .send(FileOperation::NavigateTo(path))
                                                    .ok();
                                            } else {
                                                info!("Intentando abrir");
                                                let path = item.path.to_owned();
                                                dispatcher
                                                    .send(FileOperation::OpenFileByPath(path))
                                                    .ok();
                                            }
                                        }

                                        if response.clicked_by(egui::PointerButton::Middle)
                                            && item.is_dir
                                        {
                                            let path = item.path.to_owned();
                                            dispatcher.send(FileOperation::NavigateTo(path)).ok();
                                        }

                                        ui.painter().rect(
                                            rect,
                                            corner_radius,
                                            Color32::from_rgb(
                                                color.r().saturating_add(bg_alpha),
                                                color.g().saturating_add(bg_alpha),
                                                color.b().saturating_add(bg_alpha),
                                            ),
                                            Stroke::new(0.8, color),
                                            StrokeKind::Outside,
                                        );

                                        let n = 3;
                                        let spacing = ui.available_width() / (n as f32 + 1.0);

                                        ui.scope_builder(
                                            UiBuilder::new().layout(*ui.layout()).max_rect(rect),
                                            |ui| {
                                                ui.horizontal_centered(|ui| {
                                                    ui.add_space(15.0);

                                                    let dot_size = vec2(8.0, 8.0);
                                                    let (dot_rect, _) = ui.allocate_exact_size(
                                                        dot_size,
                                                        Sense::hover(),
                                                    );

                                                    let icon_size = vec2(18.0, 18.0);
                                                    let icon_rect = Rect::from_min_size(
                                                        pos2(
                                                            dot_rect.right() + 16.0
                                                                - (icon_size.y / 2.0),
                                                            dot_rect.center().y
                                                                - (icon_size.y / 2.0),
                                                        ),
                                                        icon_size,
                                                    );

                                                    let item_vivid =
                                                        ensure_min_lightness(item.color, 0.45);

                                                    let color = Color32::from_rgba_unmultiplied(
                                                        item_vivid.r(),
                                                        item_vivid.g(),
                                                        item_vivid.b(),
                                                        60,
                                                    );

                                                    let fixed_item_color =
                                                        ensure_min_lightness(color, 0.70);

                                                    let glow = egui::epaint::Shadow {
                                                        offset: [1, 1],
                                                        blur: 3,
                                                        spread: 40,
                                                        color: fixed_item_color,
                                                    };

                                                    ui.painter().rect_filled(
                                                        dot_rect.expand(glow.blur.into()),
                                                        20.0 + glow.spread as f32,
                                                        glow.color.linear_multiply(0.3),
                                                    );

                                                    ui.painter().rect_filled(
                                                        dot_rect,
                                                        20.0,
                                                        fixed_item_color,
                                                    );

                                                    render_quicklink_icon(
                                                        ui,
                                                        item,
                                                        &thumb_snapshot,
                                                        ui_state,
                                                        icon_rect,
                                                        icon_size,
                                                    );

                                                    ui.add_space(20.0);

                                                    ui.label(item.name.to_owned());

                                                    if let Ok(guard) = item.meta.lock() {
                                                        if let Some(meta) = guard.as_ref() {
                                                            let modified = meta
                                                                .modified
                                                                .duration_since(UNIX_EPOCH)
                                                                .unwrap_or_default()
                                                                .as_secs();

                                                            ui.add_space(spacing);
                                                            ui.label(format_date(modified));

                                                            ui.add_space(spacing);
                                                            ui.label(format_size(meta.size));
                                                        }
                                                    }
                                                });
                                            },
                                        );
                                    }
                                });
                            });

                        ui.add_space(10.0);
                    }
                });
            });

            render_tags_island_bubble(ui, state, ui_state, bottom_padding, tabs_height, tag_len);
        });

    //Tags
    if ui.input(|i| i.modifiers.ctrl) && ui.input(|i| i.key_pressed(egui::Key::T)) {
        state.view_mode = match state.view_mode {
            ViewMode::Normal => ViewMode::Tags,
            ViewMode::Tags => ViewMode::Normal,
        };
    }
}
