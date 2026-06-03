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
        bootstrap::quick_access_manager::manager::with_quick_tags,
        runtime::bus_structs::QuickTagEvent,
    },
    ui::{dialog_manager::dialog_manager::ModalDialog, themes::colors::COLOR_BG_MAIN},
};
use egui::{Color32, CornerRadius, Frame, Margin, Modal, Order, TextEdit, Ui, Window};
use tracing::info;

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
    fn render(&mut self, ui: &mut Ui) {
        self.process_events(ui);
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
        Modal::new("warn_modal".into()).show(ui, |ui| {
            ui.label(message);
            if ui.button("Cerrar").clicked() {
                self.show_warn = false;
            }
        });
    }

    fn process_events(&mut self, ui: &mut Ui) {
        let Some(event) = self.event.take() else {
            return;
        };

        match event {
            QuickTagEvent::AddQuickLinkToTag { quicks } => {
                let mut accepted = false;
                let mut show_warn = self.show_warn;
                let mut message = std::mem::take(&mut self.warn_message);

                let tags = with_quick_tags(|qtm| qtm.get_tags());

                let mut selected_index = self.selected_tag_index;

                self.render_dialog(ui, "Añadir al tag", |ui, mut should_close| {
                    ui.set_min_width(250.0);
                    ui.set_min_height(100.0);

                    ui.vertical_centered(|ui| {
                        egui::ComboBox::from_label("Seleccionar tag")
                            .selected_text(
                                tags.get(selected_index)
                                    .map(|t| t.title.as_ref())
                                    .unwrap_or("Seleccionar tag"),
                            )
                            .show_ui(ui, |ui| {
                                for (i, tag) in tags.iter().enumerate() {
                                    if ui
                                        .selectable_label(selected_index == i, &*tag.title)
                                        .clicked()
                                    {
                                        selected_index = i;
                                    }
                                }
                            });

                        ui.add_space(8.0);
                    });

                    ui.add_space(50.0);

                    ui.horizontal(|ui| {
                        let width = ui.available_width();
                        let button_width = 120.0;
                        let spacing = (width - button_width * 2.0) / 2.0;

                        ui.add_space(spacing);
                        if ui.button("Cerrar").clicked() {
                            should_close = true;
                        }

                        if ui.button("Aceptar").clicked() {
                            with_quick_tags(|qtm| {
                                let tag = tags.get(selected_index);
                                if let Some(tag) = tag {
                                    let added = qtm.add_quicks_to_tag(tag.id, &quicks);
                                    if !added {
                                        show_warn = true;
                                        message = "Ya existe dentro del tag".into();
                                    } else {
                                        accepted = true;
                                        should_close = true;
                                    }
                                }
                            });
                        }
                    });

                    should_close
                });

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
            }

            QuickTagEvent::CreateNewTag {
                mut title,
                mut temp_color,
            } => {
                let mut accepted = false;
                let mut show_warn = self.show_warn;
                let mut message = std::mem::take(&mut self.warn_message);

                self.render_dialog(ui, "Crear nuevo tag", |ui, mut should_close| {
                    ui.set_min_width(250.0);
                    ui.set_min_height(100.0);

                    ui.vertical_centered(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut title)
                                .id("quick_tag_name".into())
                                .hint_text("Nombre del tag:")
                                .margin(Margin::symmetric(8, 4)),
                        );
                        ui.add_space(8.0);

                        let mut rgb: [f32; 3] = [
                            temp_color.r() as f32 / 255.0,
                            temp_color.g() as f32 / 255.0,
                            temp_color.b() as f32 / 255.0,
                        ];

                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                            temp_color = Color32::from_rgb(
                                (rgb[0] * 255.0) as u8,
                                (rgb[1] * 255.0) as u8,
                                (rgb[2] * 255.0) as u8,
                            );
                        }
                    });

                    ui.add_space(50.0);

                    ui.horizontal(|ui| {
                        let width = ui.available_width();
                        let button_width = 120.0;
                        let spacing = (width - button_width * 2.0) / 2.0;

                        ui.add_space(spacing);
                        if ui.button("Cerrar").clicked() {
                            should_close = true;
                        }

                        if ui.button("Aceptar").clicked() {
                            if title.trim().is_empty() {
                                show_warn = true;
                                message = "El título está vacío".into();
                            } else {
                                let created =
                                    with_quick_tags(|qtm| qtm.create_tag(&title, temp_color));

                                if !created {
                                    show_warn = true;
                                    message = "Ya existe tag con ese título".into();
                                } else {
                                    accepted = true;
                                    should_close = true;
                                }
                            }
                        }
                    });

                    should_close
                });

                self.show_warn = show_warn;
                self.warn_message = message.clone();

                if show_warn {
                    self.show_warn(&message, ui);
                }

                if accepted {
                    info!("creando");
                } else if self.show_modal {
                    self.event = Some(QuickTagEvent::CreateNewTag { title, temp_color });
                }
            }

            QuickTagEvent::EditCurrentTag {
                id,
                mut title,
                mut temp_color,
            } => {
                let mut accepted = false;
                let mut show_warn = self.show_warn;
                let mut message = std::mem::take(&mut self.warn_message);

                self.render_dialog(ui, "Editar Tag", |ui, mut should_close| {
                    ui.vertical_centered(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut title)
                                .id("quick_tag_name".into())
                                .hint_text("Nombre del tag:")
                                .margin(Margin::symmetric(8, 4)),
                        );
                        ui.add_space(8.0);

                        let mut rgb: [f32; 3] = [
                            temp_color.r() as f32 / 255.0,
                            temp_color.g() as f32 / 255.0,
                            temp_color.b() as f32 / 255.0,
                        ];

                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                            temp_color = Color32::from_rgb(
                                (rgb[0] * 255.0) as u8,
                                (rgb[1] * 255.0) as u8,
                                (rgb[2] * 255.0) as u8,
                            );
                        }
                    });

                    ui.add_space(50.0);

                    ui.horizontal(|ui| {
                        let width = ui.available_width();
                        let button_width = 120.0;
                        let spacing = (width - button_width * 2.0) / 2.0;

                        ui.add_space(spacing);
                        if ui.button("Cerrar").clicked() {
                            should_close = true;
                        }

                        if ui.button("Guardar").clicked() {
                            if title.trim().is_empty() {
                                show_warn = true;
                                message = "El título está vacío".into();
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
                                    message = "Error al actualizar el tag".into();
                                }
                            }
                        }
                    });

                    should_close
                });

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
            }

            QuickTagEvent::DeleteCurrentTag { id, title } => {
                let mut accepted = false;
                self.render_dialog(
                    ui,
                    &format!("¿Desea eliminar el tag: '{}'?", title),
                    |ui, mut should_close| {
                        ui.add_space(50.0);

                        ui.horizontal(|ui| {
                            let width = ui.available_width();
                            let button_width = 120.0;
                            let spacing = (width - button_width * 2.0) / 2.0;

                            ui.add_space(spacing);
                            if ui.button("Cerrar").clicked() {
                                should_close = true;
                            }

                            if ui.button("Eliminar").clicked() {
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
            }

            QuickTagEvent::DeleteQuickLink {
                tag_id,
                quick_title,
                quick_id,
            } => {
                let mut accepted = false;
                self.render_dialog(
                    ui,
                    &format!("¿Desea eliminar el link: '{}'?", quick_title),
                    |ui, mut should_close| {
                        ui.add_space(50.0);

                        ui.horizontal(|ui| {
                            let width = ui.available_width();
                            let button_width = 120.0;
                            let spacing = (width - button_width * 2.0) / 2.0;

                            ui.add_space(spacing);
                            if ui.button("Cerrar").clicked() {
                                should_close = true;
                            }

                            if ui.button("Eliminar").clicked() {
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

                self.render_dialog(ui, "Editar Quick", |ui, mut should_close| {
                    ui.vertical_centered(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut title)
                                .id("quick_tag_name".into())
                                .hint_text("Nombre del quick:")
                                .margin(Margin::symmetric(8, 4)),
                        );
                        ui.add_space(8.0);

                        let mut rgb: [f32; 3] = [
                            temp_color.r() as f32 / 255.0,
                            temp_color.g() as f32 / 255.0,
                            temp_color.b() as f32 / 255.0,
                        ];

                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                            temp_color = Color32::from_rgb(
                                (rgb[0] * 255.0) as u8,
                                (rgb[1] * 255.0) as u8,
                                (rgb[2] * 255.0) as u8,
                            );
                        }
                    });

                    ui.add_space(50.0);

                    ui.horizontal(|ui| {
                        let width = ui.available_width();
                        let button_width = 120.0;
                        let spacing = (width - button_width * 2.0) / 2.0;

                        ui.add_space(spacing);
                        if ui.button("Cerrar").clicked() {
                            should_close = true;
                        }

                        if ui.button("Guardar").clicked() {
                            if title.trim().is_empty() {
                                show_warn = true;
                                message = "El nombre está vacío".into();
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
                                    message = "Error al actualizar el tag".into();
                                }
                            }
                        }
                    });

                    should_close
                });

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
            }
        }
    }

    pub fn render_dialog<F>(&mut self, ui: &mut Ui, title: &str, mut callback: F)
    where
        F: FnMut(&mut Ui, bool) -> bool,
    {
        let mut should_close = false;

        let custom_frame = Frame::NONE
            .fill(COLOR_BG_MAIN)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        Window::new(title)
            .frame(custom_frame)
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
    }
}
