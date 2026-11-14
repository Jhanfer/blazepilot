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

use std::{path::Path, str::FromStr, sync::Arc};

use crate::{
    core::{
        bootstrap::configs::config_manager::with_configs,
        files::{
            file_extension::{FileExtension, sniff_magic_bytes},
            utils::write_to_media,
        },
        system::clipboard::global_clipboard::TOKIO_RUNTIME,
    },
    ui::{
        custom_components::UiExt,
        dialog_manager::{
            dialogs::{DialogButtonKind, dialog_button, dialog_button_spacing, dialog_frame},
            manager::ModalDialog,
        },
        themes::theme_manager::with_theme,
    },
};
use egui::{Order, RichText, Ui, Window};
use tracing::{debug, warn};

pub struct WantToDonwloadDialog {
    pub mime: Option<Box<str>>,
    pub url: Option<Box<str>>,
    pub cwd: Option<Arc<Path>>,
    pub show_modal: bool,
}

impl ModalDialog for WantToDonwloadDialog {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui) -> bool {
        self.render_dialog(ui)
    }
}

impl WantToDonwloadDialog {
    pub fn new() -> Self {
        Self {
            mime: None,
            url: None,
            cwd: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self, mime: &str, url: &str, cwd: Arc<Path>) {
        self.mime = Some(mime.into());
        self.url = Some(url.into());
        self.cwd = Some(cwd);
        self.show_modal = true;
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let i18n = with_configs(|c| c.get_i18n());

        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);

        let mut should_close = false;

        let (Some(mime), Some(url), Some(cwd)) =
            (self.mime.as_ref(), self.url.as_ref(), self.cwd.as_ref())
        else {
            return false;
        };

        let is_html = mime.starts_with("text/html");

        let (title, message) = if is_html {
            (
                i18n.t("want_download_dialog.title_web"),
                i18n.t("want_download_dialog.message_web"),
            )
        } else {
            (
                i18n.t("want_download_dialog.title_img"),
                i18n.t("want_download_dialog.message_url"),
            )
        };

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
                ui.set_min_width(250.0);
                ui.set_max_width(300.0);
                ui.set_min_height(100.0);

                ui.vertical_centered(|ui| {
                    ui.label_ns(RichText::new(message).color(palette.fg_primary));
                    ui.add_space(8.0);
                });

                ui.add_space(20.0);

                let cwd = cwd.clone();

                ui.horizontal(|ui| {
                    let spacing = dialog_button_spacing(ui, 2);

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
                        if is_html {
                            debug!("abriendo url: {url}");
                            ui.open_url(egui::OpenUrl::new_tab(url));
                            std::thread::sleep(std::time::Duration::from_millis(16));
                        } else {
                            let url_clone = url.clone();
                            let mime_clone = mime.clone();
                            TOKIO_RUNTIME.spawn(async move {
                                let uri = match url::Url::from_str(&url_clone) {
                                    Ok(uri) => uri,
                                    Err(e) => {
                                        warn!("Ha ocurrido un error al parsear url: {e}");
                                        return;
                                    }
                                };

                                let response = match reqwest::get(uri).await {
                                    Ok(resp) => resp,
                                    Err(e) => {
                                        warn!(
                                            "No se ha podido obtener el contenido de {}: {e}",
                                            url_clone
                                        );
                                        return;
                                    }
                                };

                                let bytes = match response.bytes().await {
                                    Ok(bt) => bt,
                                    Err(e) => {
                                        warn!("No se ha podido obtener los bytes: {e}");
                                        return;
                                    }
                                };

                                let data: Vec<u8> = bytes.to_vec();
                                let file_ext = sniff_magic_bytes(&bytes)
                                    .unwrap_or_else(|| FileExtension::from_mime(&mime_clone));

                                write_to_media(file_ext, cwd, data);
                            });
                        }
                        should_close = true;
                    }

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
                        should_close = true;
                    }

                    ui.add_space(spacing);
                });
            });

        should_close
    }
}
