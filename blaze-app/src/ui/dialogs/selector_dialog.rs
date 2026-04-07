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





use std::path::PathBuf;
use egui::{Color32, ColorImage, Context, Frame, Margin, Order, RichText, ScrollArea, TextureOptions, Window, scroll_area::ScrollSource};
use tracing::info;

use crate::{core::system::{clipboard::TOKIO_RUNTIME, fileopener_module::{AppAssociation, GLOBAL_FILE_OPENER, platform::linux::linux::AppsIconData}}, ui::blaze_ui_state::ModalDialog};

pub struct SelectorData {
    pub path: PathBuf,
    pub mime: String,
    pub apps: Vec<AppAssociation>,
    pub icon_data: Vec<AppsIconData>,
    pub textures: Vec<Option<egui::TextureHandle>>,
    pub show_all_apps: bool,
}

pub struct AppSelectorDialog {
    pub selector_data: Option<SelectorData>,
    pub show_modal: bool,
}

impl ModalDialog for AppSelectorDialog {
    fn is_open(&self) -> bool { self.show_modal }
    fn close(&mut self) { self.close() }
    fn render(&mut self, ctx: &Context) { self.render_app_selector(ctx); }
}

impl AppSelectorDialog {
    pub fn new() -> Self {
        Self {
            selector_data: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false; 
        self.selector_data = None;
    }

    pub fn open(&mut self, path: PathBuf, mime: String, apps: Vec<AppAssociation>, icon_data: Vec<AppsIconData>, show_all_apps: bool) {
        let textures = vec![None; apps.len()];
        self.selector_data = Some(
            SelectorData {
                path, 
                mime, 
                apps, 
                icon_data, 
                textures, 
                show_all_apps 
            }
        );

        self.show_modal = true;
    }

    fn load_textures(&mut self, ctx: &Context) {
        let Some(app_icon_data) = &mut self.selector_data else {return;};
        
        for (i, icon) in app_icon_data.icon_data.iter().enumerate(){
            if app_icon_data.textures[i].is_some() {
                continue;
            }

            if let AppsIconData::Rgba { data, width, height } = icon {
                let color_image = ColorImage::from_rgba_unmultiplied([*width as usize, *height as usize], &data);

                let texture = ctx.load_texture(
                    format!("icon_{}", app_icon_data.apps[i].id),
                    color_image, 
                    TextureOptions::NEAREST,
                );

                app_icon_data.textures[i] = Some(texture);
            }
        }
    }

    fn render_selector_button(ui: &mut egui::Ui, app: &AppAssociation, data: &SelectorData, index: usize,) -> bool {
        let mut should_close = false;
        let opener = GLOBAL_FILE_OPENER.clone();

        ui.horizontal(|ui| {
            if let Some(texture) = &data.textures[index] {
                ui.image(texture);
            } else {
                ui.label("🖼");
            }

            let br = ui.button(&app.name);
            if br.clicked() {

                let app_owned = app.clone(); 
                let path_owned = data.path.clone();
                let opener_clone = opener.clone();

                TOKIO_RUNTIME.spawn(async move {
                    let mut opener = opener_clone.lock().await;
                    opener.request_launch(&app_owned, &path_owned).await;
                });

                should_close = true;
                info!("Abrir con {}", app.name);
            }

            br.context_menu(|ui|{
                if ui.button("Seleccionar default").clicked() {
                    let app_name_owned = app.name.clone();
                    let app_owned = app.clone();
                    let mime_owned = data.mime.clone();
                    let opener_clone = opener.clone();

                    TOKIO_RUNTIME.spawn(async move {
                        let mut opener = opener_clone.lock().await;
                        opener.set_pending_default_app_name(app_name_owned).await;
                        opener.set_association(&mime_owned, app_owned).await;
                    });

                    info!("Seleccionado como default {}", app.name);

                    let path_owned = data.path.clone();
                    let app_owned = app.clone();
                    let opener_clone = opener.clone();
                    TOKIO_RUNTIME.spawn(async move {
                        let mut opener = opener_clone.lock().await;
                        opener.request_launch(&app_owned, &path_owned).await;
                    });

                    should_close = true;
                }
            });
        });
        should_close
    }


    pub fn render_app_selector(&mut self, ctx: &Context) {
        if self.selector_data.is_none() {return;}

        let mut should_close = false;

        self.load_textures(ctx);

        let Some(data) = &mut self.selector_data else { return; };
        
        let custom_frame = Frame::NONE
            .fill(Color32::from_rgb(16, 21, 25))
            .inner_margin(Margin::same(20));

        let file_name = data.path.file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| data.mime.clone());

        Window::new(format!("Abrir «{}» con...", file_name))
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .default_size([480.0, 520.0])
            .min_size([400.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal)
            .show(ctx, |ui|{
                ui.heading("Seleccionar aplicación");
                ui.separator();

                let height = if data.show_all_apps {320.0} else {50.0};

                ScrollArea::vertical()
                    .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
                    .auto_shrink([false, false])
                    .max_height(height)
                    .show(ui, |ui| {

                        ui.label(RichText::new("Recomendadas").strong());
                        ui.add_space(6.0);

                        for (i, app) in data.apps.iter().enumerate() {
                            if app.is_recommended {
                                should_close |= Self::render_selector_button(ui, app, data, i);
                            }
                        }


                        if data.show_all_apps {
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(12.0);

                            ui.label(egui::RichText::new("Todas las apps").strong()); 
                            ui.add_space(6.0);

                            for (i, app) in data.apps.iter().enumerate() {
                                if !app.is_recommended {
                                    should_close |= Self::render_selector_button(ui, &app, data, i);
                                }
                            }
                        }
                });


                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cerrar").clicked() {
                        should_close = true;
                    }
                });
            });

        if should_close {
            info!("Se cierra");
            self.close();
        }
    }
}