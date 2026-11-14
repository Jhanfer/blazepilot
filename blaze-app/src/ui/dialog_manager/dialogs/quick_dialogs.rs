// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    core::{
        bootstrap::{
            configs::config_manager::with_configs, quick_access_manager::manager::with_quick_tags,
        },
        runtime::bus_structs::QuickTagEvent,
    },
    ui::{
        custom_components::{UiExt, text_edit::BlazeTextEdit},
        dialog_manager::{
            dialogs::{DialogButtonKind, dialog_button, dialog_button_spacing, dialog_frame},
            manager::ModalDialog,
        },
        themes::theme_manager::with_theme,
    },
};
use egui::{Color32, Margin, Modal, Order, RichText, Ui, Window};

pub struct QuickAccDialog {
    event: Option<QuickTagEvent>,
    selected_tag_index: usize,
    pub show_modal: bool,
    show_warn: bool,
    warn_message: String,
}

impl ModalDialog for QuickAccDialog {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui) -> bool {
        self.process_events(ui)
    }
}

impl QuickAccDialog {
    pub fn new() -> Self {
        Self {
            event: None,
            show_modal: false,
            selected_tag_index: 0,
            show_warn: false,
            warn_message: String::new(),
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self, event: QuickTagEvent) {
        self.event = Some(event);
        self.show_modal = true;
    }

    fn show_warn(&mut self, message: &str, ui: &mut Ui) {
        let i18n = with_configs(|c| c.get_i18n());
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);

        Modal::new("warn_modal".into())
            .frame(dialog_frame(
                &palette,
                current_theme.global.panel_stroke_width,
            ))
            .show(ui, |ui| {
                ui.label_ns(RichText::new(message).color(palette.fg_primary));
                ui.add_space(8.0);

                if dialog_button(
                    ui,
                    i18n.t("general_dialog.close"),
                    &palette,
                    DialogButtonKind::Secondary,
                )
                .show()
                .clicked()
                {
                    self.show_warn = false;
                }
            });
    }

    fn process_events(&mut self, ui: &mut Ui) -> bool {
        let Some(event) = self.event.take() else {
            return false;
        };

        let current_theme = with_theme(|t| t.current().clone());
        let palette = current_theme.dialogs.palette(&current_theme.global);
        let i18n = with_configs(|c| c.get_i18n());

        match event {
            QuickTagEvent::AddQuickLinkToTag { quicks } => {
                let mut accepted = false;
                let mut show_warn = self.show_warn;
                let mut message = std::mem::take(&mut self.warn_message);

                let tags = with_quick_tags(|qtm| qtm.get_tags());

                let mut selected_index = self.selected_tag_index;

                let should_close = self.render_dialog(
                    ui,
                    &i18n.t("quick_dialog_quick_to_tag.title"),
                    |ui, mut should_close| {
                        ui.set_min_width(250.0);
                        ui.set_max_width(300.0);
                        ui.set_min_height(100.0);

                        ui.vertical_centered(|ui| {
                            let hover_bg = palette.bg_hover;

                            ui.label_ns(
                                RichText::new(i18n.t("quick_dialog_quick_to_tag.select_tag"))
                                    .color(palette.fg_primary),
                            );

                            ui.horizontal(|ui| {
                                ui.blaze_combobox()
                                    .selected_text(
                                        tags.get(selected_index)
                                            .map(|t| t.title.as_ref())
                                            .unwrap_or(
                                                &i18n.t("quick_dialog_quick_to_tag.select_tag"),
                                            ),
                                    )
                                    .text_color(palette.fg_primary)
                                    .size((150.0, 30.0))
                                    .background_color(palette.bg_elevated)
                                    .hover_background_color(palette.bg_hover)
                                    .show(|ui| {
                                        let w = ui.available_width();

                                        for (i, tag) in tags.iter().enumerate() {
                                            let is_selected = selected_index == i;
                                            let text_color = if is_selected {
                                                palette.fg_primary
                                            } else {
                                                palette.fg_secondary
                                            };
                                            let label =
                                                RichText::new(&*tag.title).color(text_color);

                                            let bg = if is_selected {
                                                palette.bg_selected
                                            } else {
                                                Color32::TRANSPARENT
                                            };

                                            if ui
                                                .blaze_button(label)
                                                .id(format!("quick_tag{i}"))
                                                .size((w, 20.0))
                                                .background_color(bg)
                                                .hover_background_color(hover_bg)
                                                .stroke_color(Color32::TRANSPARENT)
                                                .radius(20)
                                                .stroke_width(
                                                    current_theme.global.components_stroke_width,
                                                )
                                                .show()
                                                .clicked()
                                            {
                                                selected_index = i;
                                            }
                                        }
                                    });
                            });

                            ui.add_space(8.0);
                        });

                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            let spacing = dialog_button_spacing(ui, 2);

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.close"),
                                &palette,
                                DialogButtonKind::Secondary,
                            )
                            .show()
                            .clicked()
                            {
                                should_close = true;
                            }

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.accept"),
                                &palette,
                                DialogButtonKind::Primary,
                            )
                            .show()
                            .clicked()
                            {
                                with_quick_tags(|qtm| {
                                    let tag = tags.get(selected_index);
                                    if let Some(tag) = tag {
                                        let added = qtm.add_quicks_to_tag(tag.id, &quicks);
                                        if !added {
                                            show_warn = true;
                                            message = i18n
                                                .t("quick_dialog_quick_to_tag.tag_already_exists")
                                                .into();
                                        } else {
                                            accepted = true;
                                            should_close = true;
                                        }
                                    }
                                });
                            }
                        });

                        should_close
                    },
                );

                self.selected_tag_index = selected_index;
                self.show_warn = show_warn;
                self.warn_message = message.clone();

                if show_warn {
                    self.show_warn(&message, ui);
                }

                if accepted {
                } else if self.show_modal {
                    self.event = Some(QuickTagEvent::AddQuickLinkToTag { quicks });
                }

                should_close
            }

            QuickTagEvent::CreateNewTag {
                mut title,
                mut temp_color,
            } => {
                let mut accepted = false;
                let mut show_warn = self.show_warn;
                let mut message = std::mem::take(&mut self.warn_message);

                let should_close = self.render_dialog(
                    ui,
                    &i18n.t("quick_dialog_create.title"),
                    |ui, mut should_close| {
                        ui.set_min_width(250.0);
                        ui.set_min_height(100.0);

                        ui.vertical_centered(|ui| {
                            ui.add(
                                BlazeTextEdit::singleline(&mut title)
                                    .id("quick_tag_name".into())
                                    .hint_text(i18n.t("quick_dialog_create.text_edit_hint"))
                                    .margin(Margin::symmetric(8, 4)),
                            );
                            ui.add_space(8.0);

                            ui.color_picker(&mut temp_color, "quick_tag_color")
                                .background_color(palette.bg_elevated)
                                .show();
                        });

                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            let spacing = dialog_button_spacing(ui, 2);

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.close"),
                                &palette,
                                DialogButtonKind::Secondary,
                            )
                            .show()
                            .clicked()
                            {
                                should_close = true;
                            }

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.accept"),
                                &palette,
                                DialogButtonKind::Primary,
                            )
                            .show()
                            .clicked()
                            {
                                if title.trim().is_empty() {
                                    show_warn = true;
                                    message = i18n.t("quick_dialog_create.title_empty").into();
                                } else {
                                    let created =
                                        with_quick_tags(|qtm| qtm.create_tag(&title, temp_color));

                                    if !created {
                                        show_warn = true;
                                        message = i18n
                                            .t("quick_dialog_create.title_on_tag_already_exists")
                                            .into();
                                    } else {
                                        accepted = true;
                                        should_close = true;
                                    }
                                }
                            }
                        });

                        should_close
                    },
                );

                self.show_warn = show_warn;
                self.warn_message = message.clone();

                if show_warn {
                    self.show_warn(&message, ui);
                }

                if accepted {
                } else if self.show_modal {
                    self.event = Some(QuickTagEvent::CreateNewTag { title, temp_color });
                }

                should_close
            }

            QuickTagEvent::EditCurrentTag {
                id,
                mut title,
                mut temp_color,
            } => {
                let mut accepted = false;
                let mut show_warn = self.show_warn;
                let mut message = std::mem::take(&mut self.warn_message);

                let should_close = self.render_dialog(
                    ui,
                    &i18n.t("quick_dialog_edit_tag.title"),
                    |ui, mut should_close| {
                        ui.vertical_centered(|ui| {
                            ui.add(
                                BlazeTextEdit::singleline(&mut title)
                                    .id("quick_tag_name".into())
                                    .hint_text(i18n.t("quick_dialog_edit_tag.text_edit_hint"))
                                    .margin(Margin::symmetric(8, 4)),
                            );
                            ui.add_space(8.0);

                            ui.color_picker(&mut temp_color, "quick_tag_color")
                                .background_color(palette.bg_elevated)
                                .show();
                        });

                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            let spacing = dialog_button_spacing(ui, 2);

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.close"),
                                &palette,
                                DialogButtonKind::Secondary,
                            )
                            .show()
                            .clicked()
                            {
                                should_close = true;
                            }

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.save"),
                                &palette,
                                DialogButtonKind::Primary,
                            )
                            .show()
                            .clicked()
                            {
                                if title.trim().is_empty() {
                                    show_warn = true;
                                    message = i18n.t("quick_dialog_edit_tag.title_empty").into();
                                } else {
                                    let updated = with_quick_tags(|qtm| {
                                        qtm.update_tag_callback(id, |tag| {
                                            tag.title = title.trim().into();
                                            tag.color = temp_color;
                                        })
                                    });

                                    if updated {
                                        accepted = true;
                                        should_close = true;
                                    } else {
                                        show_warn = true;
                                        message =
                                            i18n.t("quick_dialog_edit_tag.update_error").into();
                                    }
                                }
                            }
                        });

                        should_close
                    },
                );

                self.show_warn = show_warn;
                self.warn_message = message.clone();

                if show_warn {
                    self.show_warn(&message, ui);
                }

                if !accepted && self.show_modal {
                    self.event = Some(QuickTagEvent::EditCurrentTag {
                        id,
                        title,
                        temp_color,
                    });
                }

                should_close
            }

            QuickTagEvent::DeleteCurrentTag { id, title } => {
                let mut accepted = false;
                let should_close = self.render_dialog(
                    ui,
                    &i18n.t_args("quick_dialog_delete_tag.title", &[("query", &title)]),
                    |ui, mut should_close| {
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            let spacing = dialog_button_spacing(ui, 2);

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.close"),
                                &palette,
                                DialogButtonKind::Secondary,
                            )
                            .show()
                            .clicked()
                            {
                                should_close = true;
                            }

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.delete"),
                                &palette,
                                DialogButtonKind::Danger,
                            )
                            .show()
                            .clicked()
                            {
                                with_quick_tags(|qtm| {
                                    qtm.remove_tag(id);
                                });
                                should_close = true;
                                accepted = true;
                            }
                        });

                        should_close
                    },
                );

                if !accepted && self.show_modal {
                    self.event = Some(QuickTagEvent::DeleteCurrentTag { id, title });
                }

                should_close
            }

            QuickTagEvent::DeleteQuickLink {
                tag_id,
                quick_title,
                quick_id,
            } => {
                let mut accepted = false;
                let should_close = self.render_dialog(
                    ui,
                    &i18n.t_args(
                        "quick_dialog_delete_quick.title",
                        &[("query", &quick_title)],
                    ),
                    |ui, mut should_close| {
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            let spacing = dialog_button_spacing(ui, 2);

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.close"),
                                &palette,
                                DialogButtonKind::Secondary,
                            )
                            .show()
                            .clicked()
                            {
                                should_close = true;
                            }

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.delete"),
                                &palette,
                                DialogButtonKind::Danger,
                            )
                            .show()
                            .clicked()
                            {
                                with_quick_tags(|qtm| {
                                    qtm.remove_quick_to_tag(tag_id, quick_id);
                                });
                                should_close = true;
                                accepted = true;
                            }
                        });

                        should_close
                    },
                );

                if !accepted && self.show_modal {
                    self.event = Some(QuickTagEvent::DeleteQuickLink {
                        tag_id,
                        quick_title,
                        quick_id,
                    });
                }

                should_close
            }

            QuickTagEvent::EditCurrentQuickLink {
                tag_id,
                quick_id,
                mut title,
                mut temp_color,
            } => {
                let mut accepted = false;
                let mut show_warn = self.show_warn;
                let mut message = std::mem::take(&mut self.warn_message);

                let should_close = self.render_dialog(
                    ui,
                    &i18n.t("quick_dialog_edit_quick.title"),
                    |ui, mut should_close| {
                        ui.vertical_centered(|ui| {
                            ui.add(
                                BlazeTextEdit::singleline(&mut title)
                                    .id("quick_tag_name".into())
                                    .hint_text(i18n.t("quick_dialog_edit_quick.text_edit_hint"))
                                    .margin(Margin::symmetric(8, 4)),
                            );
                            ui.add_space(8.0);

                            ui.color_picker(&mut temp_color, "quick_tag_color")
                                .background_color(palette.bg_elevated)
                                .show();
                        });

                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            let spacing = dialog_button_spacing(ui, 2);

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.close"),
                                &palette,
                                DialogButtonKind::Secondary,
                            )
                            .show()
                            .clicked()
                            {
                                should_close = true;
                            }

                            ui.add_space(spacing);

                            if dialog_button(
                                ui,
                                i18n.t("general_dialog.save"),
                                &palette,
                                DialogButtonKind::Primary,
                            )
                            .show()
                            .clicked()
                            {
                                if title.trim().is_empty() {
                                    show_warn = true;
                                    message = i18n.t("quick_dialog_edit_quick.name_empty").into();
                                } else {
                                    let updated = with_quick_tags(|qtm| {
                                        qtm.update_quick_callback(tag_id, quick_id, |quick| {
                                            quick.name = title.trim().into();
                                            quick.color = temp_color;
                                        })
                                    });

                                    if updated {
                                        accepted = true;
                                        should_close = true;
                                    } else {
                                        show_warn = true;
                                        message =
                                            i18n.t("quick_dialog_edit_quick.update_error").into();
                                    }
                                }
                            }
                        });

                        should_close
                    },
                );

                self.show_warn = show_warn;
                self.warn_message = message.clone();

                if show_warn {
                    self.show_warn(&message, ui);
                }

                if !accepted && self.show_modal {
                    self.event = Some(QuickTagEvent::EditCurrentQuickLink {
                        tag_id,
                        quick_id,
                        title,
                        temp_color,
                    });
                }

                should_close
            }
        }
    }

    pub fn render_dialog<F>(&mut self, ui: &mut Ui, title: &str, mut callback: F) -> bool
    where
        F: FnMut(&mut Ui, bool) -> bool,
    {
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);

        let mut should_close = false;

        Window::new(RichText::new(title).color(palette.fg_primary))
            .frame(dialog_frame(
                &palette,
                current_theme.global.panel_stroke_width,
            ))
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal)
            .show(ui, |ui| {
                should_close = callback(ui, should_close);
            });

        if should_close {
            self.close();
        }

        should_close
    }
}
