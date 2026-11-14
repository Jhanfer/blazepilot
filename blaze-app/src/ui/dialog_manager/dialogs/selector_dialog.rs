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

use egui::{
    ColorImage, Image, Order, RichText, ScrollArea, TextureHandle, TextureOptions, Ui, Window,
};
use fast_image_resize as fr;
use parking_lot::Mutex;
use std::{path::Path, sync::Arc};
use tracing::warn;

use crate::{
    core::{
        bootstrap::configs::config_manager::with_configs,
        system::{
            clipboard::global_clipboard::TOKIO_RUNTIME,
            fileopener_module::{
                GLOBAL_FILE_OPENER,
                platform::{
                    linux::backend::DesktopApp,
                    opener_trait::{AppIconSource, AppInfo},
                },
            },
        },
    },
    ui::{
        custom_components::UiExt,
        dialog_manager::{
            dialogs::{DialogButtonKind, dialog_button, dialog_button_spacing, dialog_frame},
            manager::ModalDialog,
        },
        themes::{platform::palette::SectionPalette, theme_manager::with_theme},
    },
};

pub enum SelectorState {
    Loading,
    Ready(SelectorData),
}

pub struct SelectorData {
    pub path: Arc<Path>,
    pub apps: Vec<AppInfo>,
    pub textures: Vec<Option<egui::TextureHandle>>,
}

type PendingApps = (Arc<Path>, Arc<Mutex<Option<Vec<AppInfo>>>>);

pub struct AppSelectorDialog {
    pub show_modal: bool,
    pub state: Option<SelectorState>,
    pending_apps: Option<PendingApps>,
}

impl ModalDialog for AppSelectorDialog {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui) -> bool {
        self.render_app_selector(ui)
    }
}

impl AppSelectorDialog {
    pub fn new() -> Self {
        Self {
            show_modal: false,
            state: None,
            pending_apps: None,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
        self.state = None;
        self.pending_apps = None;
    }

    pub fn open(&mut self, path: Arc<Path>) {
        self.show_modal = true;
        self.state = Some(SelectorState::Loading);

        let slot: Arc<Mutex<Option<Vec<AppInfo>>>> = Arc::new(Mutex::new(None));
        let opener = GLOBAL_FILE_OPENER.clone();
        let path_clone = path.clone();
        let slot_clone = slot.clone();

        let mut manager = opener.lock();
        match manager.get_all_apps(path_clone) {
            Ok(apps) => {
                let mut guard = slot_clone.lock();
                *guard = Some(apps);
            }
            Err(e) => warn!("Error obteniendo apps en segundo plano: {}", e),
        }
        self.pending_apps = Some((path, slot));
    }

    fn poll_pending_apps(&mut self) {
        let Some((path, slot)) = self.pending_apps.take() else {
            return;
        };

        let slot_clone = slot.clone();

        match slot_clone.try_lock() {
            Some(mut guard) => {
                if let Some(apps) = guard.take() {
                    let count = apps.len();

                    self.state = Some(SelectorState::Ready(SelectorData {
                        path,
                        apps,
                        textures: vec![None; count],
                    }));
                    self.pending_apps = None;
                }
            }
            _ => self.pending_apps = Some((path, slot)),
        };
    }

    fn load_svg_as_texture(ctx: &mut Ui, path: &Path, size: u32) -> Option<TextureHandle> {
        let svg_data = std::fs::read(path).ok()?;

        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(&svg_data, &opt).ok()?;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;

        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(
                size as f32 / tree.size().width(),
                size as f32 / tree.size().height(),
            ),
            &mut pixmap.as_mut(),
        );

        let image =
            ColorImage::from_rgba_unmultiplied([size as usize, size as usize], pixmap.data());

        Some(ctx.load_texture(path.to_string_lossy(), image, TextureOptions::LINEAR))
    }

    fn load_textures(&mut self, ui: &mut Ui) {
        let Some(SelectorState::Ready(data)) = &mut self.state else {
            return;
        };
        let mut needs_repaint = false;
        for app in data.apps.iter_mut() {
            if let AppIconSource::Unresolved(name) = &app.icon {
                app.icon = DesktopApp::resolve_icon_path(&name.clone());
                needs_repaint = true;
                break;
            }
        }

        for (i, app) in data.apps.iter_mut().enumerate() {
            if data.textures[i].is_some() {
                continue;
            }

            let AppIconSource::Path(icon_path) = &app.icon else {
                continue;
            };

            let texture = match icon_path.extension().and_then(|e| e.to_str()) {
                Some("png") | Some("jpg") | Some("jpeg") => match std::fs::File::open(icon_path) {
                    Ok(mut file) => {
                        let mut buffer = Vec::new();
                        use std::io::Read;

                        if file.read_to_end(&mut buffer).is_ok() {
                            match stb_image::image::load_from_memory_with_depth(&buffer, 4, false) {
                                stb_image::image::LoadResult::ImageU8(image) => {
                                    let (w, h) = (image.width as u32, image.height as u32);
                                    let raw_rgba = if w == 64 && h == 64 {
                                        image.data
                                    } else {
                                        let src_image =
                                            fr::images::Image::new(64, 64, fr::PixelType::U8x4);

                                        let mut dst_image =
                                            fr::images::Image::new(64, 64, fr::PixelType::U8x4);

                                        let mut resizer = fr::Resizer::new();

                                        resizer
                                            .resize(
                                                &src_image,
                                                &mut dst_image,
                                                &fr::ResizeOptions::new(),
                                            )
                                            .unwrap();

                                        dst_image.into_vec()
                                    };

                                    Some(ui.ctx().load_texture(
                                        format!("icon_{}", app.id),
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            [64, 64],
                                            &raw_rgba,
                                        ),
                                        egui::TextureOptions::LINEAR,
                                    ))
                                }

                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                Some("svg") => Self::load_svg_as_texture(ui, icon_path, 64),
                _ => None,
            };

            if texture.is_some() {
                data.textures[i] = texture;
                needs_repaint = true;
            }
        }

        if needs_repaint {}
    }

    fn render_selector_button(
        ui: &mut egui::Ui,
        app: &AppInfo,
        data: &SelectorData,
        index: usize,
        palette: &SectionPalette,
    ) -> bool {
        let i18n = with_configs(|c| c.get_i18n());

        let mut should_close = false;
        let opener = GLOBAL_FILE_OPENER.clone();

        ui.horizontal(|ui| {
            if let Some(texture) = &data.textures[index] {
                ui.add(Image::new(texture).fit_to_exact_size(egui::vec2(20.0, 20.0)));
            } else {
                ui.label_ns("🖼");
            }

            let button_width = ui.available_width();
            let br = ui
                .blaze_button(&*app.name)
                .size((button_width, 24.0))
                .text_color(palette.fg_primary)
                .background_color(palette.bg_elevated)
                .hover_background_color(palette.bg_hover)
                .stroke_color(egui::Color32::TRANSPARENT)
                .radius(8)
                .show();

            if br.clicked() {
                let app_id = app.id.clone();
                let path_owned = data.path.clone();
                let opener_clone = opener.clone();

                TOKIO_RUNTIME.spawn_blocking(move || {
                    let opener = opener_clone.lock();
                    if let Err(e) = opener.open_with(&app_id, path_owned) {
                        warn!("Error abriendo: {}", e);
                    }
                });

                should_close = true;
            }

            br.context_menu(|ui| {
                if ui
                    .button(i18n.t("selector_dialog.select_as_default"))
                    .clicked()
                {
                    let app_id = app.id.clone();
                    let path_owned = data.path.clone();
                    let opener_clone = opener.clone();

                    TOKIO_RUNTIME.spawn_blocking(move || {
                        let mut opener = opener_clone.lock();
                        // save_to_system = false: solo guarda en BlazePilot,
                        // no toca mimeapps.list del sistema
                        if let Err(e) = opener.set_default_app(path_owned.clone(), &app_id, false) {
                            warn!("Error estableciendo default: {}", e);
                            return; // no intentar abrir si falló el set
                        }
                        if let Err(e) = opener.open_with(&app_id, path_owned) {
                            warn!("Error abriendo: {}", e);
                        }
                    });

                    should_close = true;
                }
            });
        });

        should_close
    }

    pub fn render_app_selector(&mut self, ui: &mut Ui) -> bool {
        self.poll_pending_apps();

        if let Some(SelectorState::Ready(_)) = &self.state {
            self.load_textures(ui);
        }

        let i18n = with_configs(|c| c.get_i18n());

        let mut should_close = self.show_modal;

        match &self.state {
            None => false,
            Some(SelectorState::Loading) => {
                let current_theme = with_theme(|t| t.current());
                let palette = current_theme.dialogs.palette(&current_theme.global);
                let mut open = self.show_modal;

                Window::new(
                    RichText::new(i18n.t("selector_dialog.select_app")).color(palette.fg_primary),
                )
                .frame(dialog_frame(
                    &palette,
                    current_theme.global.panel_stroke_width,
                ))
                .order(Order::Foreground)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut open)
                .show(ui, |ui| {
                    ui.set_min_width(250.0);
                    ui.set_max_width(300.0);
                    ui.set_min_height(100.0);

                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label_ns(
                            RichText::new(i18n.t("selector_dialog.searching_apps"))
                                .color(palette.fg_primary),
                        );
                    });
                });

                self.show_modal = open;
                !open
            }
            Some(SelectorState::Ready(data)) => {
                let current_theme = with_theme(|t| t.current());
                let palette = current_theme.dialogs.palette(&current_theme.global);

                let file_name = data
                    .path
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "unknown".to_string());

                let mut close_requested = false;

                Window::new(
                    RichText::new(
                        i18n.t_args("selector_dialog.open_with", &[("query", &file_name)]),
                    )
                    .color(palette.fg_primary),
                )
                .frame(dialog_frame(
                    &palette,
                    current_theme.global.panel_stroke_width,
                ))
                .order(Order::Foreground)
                .collapsible(false)
                .resizable(false)
                .default_size([480.0, 520.0])
                .min_size([400.0, 300.0])
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut should_close)
                .show(ui, |ui| {
                    ui.label_ns(
                        RichText::new(i18n.t("selector_dialog.select_app"))
                            .color(palette.fg_primary)
                            .size(18.0),
                    );
                    ui.separator();

                    let height = 320.0;

                    ScrollArea::vertical().max_height(height).show(ui, |ui| {
                        ui.label_ns(
                            RichText::new(i18n.t("selector_dialog.recomends"))
                                .color(palette.fg_primary)
                                .strong(),
                        );
                        ui.add_space(6.0);

                        for (i, app) in data.apps.iter().enumerate() {
                            if app.is_recommended {
                                close_requested |=
                                    Self::render_selector_button(ui, app, data, i, &palette);
                            }
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(12.0);

                        ui.label_ns(
                            RichText::new(i18n.t("selector_dialog.all_apps"))
                                .color(palette.fg_primary)
                                .strong(),
                        );
                        ui.add_space(6.0);

                        for (i, app) in data.apps.iter().enumerate() {
                            if !app.is_recommended {
                                close_requested |=
                                    Self::render_selector_button(ui, app, data, i, &palette);
                            }
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        let spacing = dialog_button_spacing(ui, 1);

                        ui.add_space(spacing);

                        if dialog_button(
                            ui,
                            i18n.t("general_dialog.cancel"),
                            &palette,
                            DialogButtonKind::Secondary,
                        )
                        .show()
                        .clicked()
                        {
                            close_requested = true;
                        }

                        ui.add_space(spacing);
                    });
                });

                self.show_modal = should_close;
                close_requested
            }
        }
    }
}
