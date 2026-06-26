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

use crate::core::{
    bootstrap::configs::{
        config_manager::with_configs, platform::linux::conf_structs::DisplayBackend,
    },
    system::{
        clipboard::global_clipboard::TOKIO_RUNTIME,
        terminal_opener::terminal_manager::GLOBAL_TERMINAL_MANAGER,
    },
};
use crate::ui::dialog_manager::manager::ModalDialog;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use core::f32;
use egui::{
    CentralPanel, Color32, ComboBox, CornerRadius, Frame, Id, Key, Label, Margin, OpenUrl, Order,
    Panel, RichText, ScrollArea, TextEdit, Ui, Window,
};
use std::time::Duration;
use tracing::{info, warn};

#[derive(PartialEq, Clone, Copy)]
enum CurrentConfigTab {
    General,
    Terminal,
    Backend,
    Appearance,
    Behavior,
    Language,
}

impl CurrentConfigTab {
    pub fn name(&self) -> Box<str> {
        let i18n = with_configs(|c| c.get_i18n());
        match self {
            CurrentConfigTab::General => i18n.t("configs.general"),
            CurrentConfigTab::Terminal => i18n.t("configs.terminal"),
            CurrentConfigTab::Backend => i18n.t("configs.backend"),
            CurrentConfigTab::Appearance => i18n.t("configs.appearance"),
            CurrentConfigTab::Behavior => i18n.t("configs.behavior"),
            CurrentConfigTab::Language => i18n.t("configs.language"),
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let i18n = with_configs(|c| c.get_i18n());
        if query.is_empty() {
            return true;
        }

        let q = query.to_lowercase();

        let name_matches = self.name().to_lowercase().contains(&q);

        let content_matches = match self {
            CurrentConfigTab::General => false,

            CurrentConfigTab::Terminal => {
                let keywords = [
                    i18n.t("search_keywords_terminal.terminal"),
                    i18n.t("search_keywords_terminal.shell"),
                    "bash".into(),
                    "zsh".into(),
                    "cmd".into(),
                    "powershell".into(),
                    i18n.t("search_keywords_terminal.prompt"),
                    i18n.t("search_keywords_terminal.font"),
                    i18n.t("search_keywords_terminal.size"),
                    i18n.t("search_keywords_terminal.command"),
                ];
                keywords
                    .iter()
                    .any(|k| k.contains(&q) || q.contains(&k.to_string()))
            }
            CurrentConfigTab::Backend => {
                let keywords = [
                    i18n.t("search_keywords_backend.backend"),
                    "gpu".into(),
                    "cpu".into(),
                    i18n.t("search_keywords_backend.render"),
                    "vulkan".into(),
                    "opengl".into(),
                    "direui".into(),
                    "metal".into(),
                    i18n.t("search_keywords_backend.renderer"),
                    i18n.t("search_keywords_backend.acceleration"),
                    i18n.t("search_keywords_backend.display_protocol"),
                ];
                keywords
                    .iter()
                    .any(|k| k.contains(&q) || q.contains(&k.to_string()))
            }
            CurrentConfigTab::Appearance => {
                let keywords = [
                    i18n.t("search_keywords_appearance.color"),
                    i18n.t("search_keywords_appearance.theme"),
                    i18n.t("search_keywords_appearance.dark"),
                    i18n.t("search_keywords_appearance.light"),
                    i18n.t("search_keywords_appearance.font"),
                    i18n.t("search_keywords_appearance.icon"),
                    i18n.t("search_keywords_appearance.background"),
                ];
                keywords
                    .iter()
                    .any(|k| k.contains(&q) || q.contains(&k.to_string()))
            }
            CurrentConfigTab::Behavior => {
                let keywords = [
                    i18n.t("search_keywords_behavior.auto"),
                    i18n.t("search_keywords_behavior.save"),
                    i18n.t("search_keywords_behavior.backup"),
                    i18n.t("search_keywords_behavior.behavior"),
                    i18n.t("search_keywords_behavior.confirm"),
                    i18n.t("search_keywords_behavior.undo"),
                ];
                keywords
                    .iter()
                    .any(|k| k.contains(&q) || q.contains(&k.to_string()))
            }
            CurrentConfigTab::Language => {
                let keywords = [
                    i18n.t("search_keywords_language.language"),
                    i18n.t("search_keywords_language.idiom"),
                ];
                keywords
                    .iter()
                    .any(|k| k.contains(&q) || q.contains(&k.to_string()))
            }
        };

        name_matches || content_matches
    }
}

pub struct ConfigDialog {
    current_config_tab: CurrentConfigTab,
    pub config_search: String,
    pub show_modal: bool,

    available_terminals: Vec<String>,
    loading_terminals: bool,
    terminal_rx: Option<tokio::sync::mpsc::Receiver<Vec<String>>>,
    terminals_loaded: bool,
    no_terminals_error: bool,
    retry_count: u8,
    max_retries: u8,
    custom_theme_name: String,
}

impl ModalDialog for ConfigDialog {
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

impl ConfigDialog {
    pub fn new() -> Self {
        Self {
            current_config_tab: CurrentConfigTab::General,
            config_search: String::new(),
            show_modal: false,

            available_terminals: Vec::new(),
            loading_terminals: false,
            terminal_rx: None,
            terminals_loaded: false,
            no_terminals_error: false,
            retry_count: 0,
            max_retries: 3,
            custom_theme_name: String::new(),
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self) {
        self.show_modal = true;
    }

    fn render_config_sidebar(&mut self, ui: &mut Ui) {
        let current_theme = with_theme(|t| t.current());

        let frame = Frame::new().inner_margin(20.0);
        frame.show(ui, |ui| {
            ui.add_space(6.0);

            let search_id = egui::Id::new("config_search_bar");

            let resp = ui.add(
                TextEdit::singleline(&mut self.config_search)
                    .id(search_id)
                    .hint_text("Buscar configs...")
                    .desired_width(f32::INFINITY)
                    .margin(Margin::symmetric(8, 4)),
            );

            if resp.clicked() || resp.gained_focus() {
                ui.memory_mut(|mem| {
                    mem.request_focus(search_id);
                });
            }

            if !self.config_search.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() - 24.0);
                    if ui.small_button("X").clicked() {
                        self.config_search.clear();
                        ui.memory_mut(|mem| mem.request_focus(search_id));
                    }
                });
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            let query = self.config_search.trim().to_lowercase();

            let all_tabs = [
                CurrentConfigTab::General,
                CurrentConfigTab::Terminal,
                CurrentConfigTab::Backend,
                CurrentConfigTab::Appearance,
                CurrentConfigTab::Behavior,
                CurrentConfigTab::Language,
            ];

            let filtered: Vec<CurrentConfigTab> = all_tabs
                .iter()
                .copied()
                .filter(|&tab| tab.matches_search(&query))
                .collect();

            for tab in &filtered {
                let is_selected = self.current_config_tab == *tab;

                let label = RichText::new(tab.name()).color(current_theme.text_primary.to_color());

                if ui.selectable_label(is_selected, label).clicked() {
                    self.current_config_tab = *tab;
                }
            }

            if filtered.is_empty() && !query.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("No se encontraron resultados")
                            .weak()
                            .color(current_theme.text_primary.to_color()),
                    );
                });
            }
        });
    }

    fn render_general_settings(&self, ui: &mut Ui, _query: &str, frame: Frame) {
        let current_theme = with_theme(|t| t.current());

        let i18n = with_configs(|c| c.get_i18n());

        ui.add_space(10.0);
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.add_space(20.0);

                ui.add(
                    Label::new(
                        RichText::new("BlazePilot")
                            .strong()
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );
                ui.add_space(4.0);

                ui.add(
                    Label::new(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .color(current_theme.text_muted.to_color()),
                    )
                    .selectable(false),
                );

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(20.0);

                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_general.gratitude"))
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );

                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_general.made_for"))
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.link("GitHub").clicked() {
                        ui.open_url(OpenUrl::new_tab("https://github.com/Jhanfer/blazepilot"));
                    }

                    ui.add(
                        Label::new(
                            RichText::new("•").color(current_theme.text_secondary.to_color()),
                        )
                        .selectable(false),
                    );

                    if ui.link(i18n.t("configs_general.report")).clicked() {
                        ui.open_url(OpenUrl::new_tab(
                            "https://github.com/Jhanfer/blazepilot/issues",
                        ));
                    }
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);

                ui.collapsing(
                    RichText::new(format!("OS: {}", std::env::consts::OS))
                        .color(current_theme.text_secondary.to_color()),
                    |ui| {
                        ui.monospace(
                            RichText::new(format!("OS: {}", std::env::consts::OS))
                                .color(current_theme.text_secondary.to_color()),
                        );
                        ui.monospace(
                            RichText::new(format!("Arch: {}", std::env::consts::ARCH))
                                .color(current_theme.text_secondary.to_color()),
                        );
                    },
                );

                ui.add_space(20.0);
                ui.label(
                    RichText::new(i18n.t("configs_general.disclaimer"))
                        .color(current_theme.text_muted.to_color())
                        .italics()
                        .weak(),
                );
            });
        });
    }

    fn get_selected_terminal_text(&self, label: &str) -> String {
        with_configs(|c| {
            if c.get_default_terminal().trim().is_empty() {
                label.to_string()
            } else {
                c.get_default_terminal()
            }
        })
    }

    fn is_terminal_selected(&self, term: &str) -> bool {
        with_configs(|c| c.get_default_terminal() == term)
    }

    fn reset_terminal_loading_state(&mut self) {
        self.available_terminals.clear();
        self.loading_terminals = false;
        self.terminals_loaded = false;
        self.no_terminals_error = false;
        self.terminal_rx = None;
    }

    fn render_terminal_settings(&mut self, ui: &mut Ui, _query: &str, frame: Frame) {
        let current_theme = with_theme(|t| t.current());
        let i18n = with_configs(|c| c.get_i18n());

        ui.add_space(10.0);

        if self.available_terminals.is_empty()
            && !self.loading_terminals
            && !self.terminals_loaded
            && (self.retry_count < self.max_retries)
        {
            self.loading_terminals = true;
            self.retry_count += 1;

            let (tx, rx) = tokio::sync::mpsc::channel(1);
            self.terminal_rx = Some(rx);

            let tm_manager = GLOBAL_TERMINAL_MANAGER.clone();

            TOKIO_RUNTIME.spawn(async move {
                let result = tokio::time::timeout(Duration::from_secs(5), async {
                    let mut manager = tm_manager.lock().await;
                    manager.request_load_terminals().await
                })
                .await;

                match result {
                    Ok(terminals) => {
                        tx.send(terminals).await.ok();
                    }
                    Err(_) => {
                        tx.send(Vec::new()).await.ok();
                    }
                }
            });
        }

        if let Some(rx) = &mut self.terminal_rx {
            if let Ok(terminals) = rx.try_recv() {
                self.available_terminals = terminals.clone();
                self.loading_terminals = false;
                self.terminals_loaded = true;
                self.terminal_rx = None;

                if terminals.is_empty() {
                    self.no_terminals_error = true;
                    if self.retry_count >= self.max_retries {
                        warn!("Máximo de reintentos alcanzado para cargar terminales");
                    }
                } else {
                    self.no_terminals_error = false;
                }
            }
        }

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_terminal.title"))
                            .strong()
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );

                ui.add_space(8.0);

                if self.loading_terminals {
                    ui.horizontal(|ui| {
                        ui.spinner();

                        ui.add(
                            Label::new(
                                RichText::new(i18n.t_args(
                                    "configs_terminal.loading_message",
                                    &[
                                        ("query", &self.retry_count.to_string()),
                                        ("query", &self.max_retries.to_string()),
                                    ],
                                ))
                                .color(current_theme.text_primary.to_color()),
                            )
                            .selectable(false),
                        );
                    });
                    return;
                }

                if self.available_terminals.is_empty() {
                    let mut show_retry = false;

                    if self.no_terminals_error {
                        ui.colored_label(
                            current_theme.error.to_color(),
                            i18n.t("configs_terminal.no_terminals_error"),
                        );

                        if self.retry_count < self.max_retries {
                            show_retry = true;

                            if ui.button(i18n.t("configs_terminal.retry")).clicked() {
                                self.reset_terminal_loading_state();
                            }
                        } else {
                            ui.colored_label(
                                current_theme.warn.to_color(),
                                i18n.t("configs_terminal.max_retries_error"),
                            );

                            ui.add(
                                Label::new(
                                    RichText::new(
                                        i18n.t("configs_terminal.verify_terminals_message"),
                                    )
                                    .color(current_theme.text_secondary.to_color()),
                                )
                                .selectable(false),
                            );
                        }
                    } else {
                        ui.add(
                            Label::new(
                                RichText::new(i18n.t("configs_terminal.no_terminals_error"))
                                    .color(current_theme.text_secondary.to_color()),
                            )
                            .selectable(false),
                        );
                    }

                    if show_retry {
                        ui.add_space(8.0);
                        ui.colored_label(
                            current_theme.warn.to_color(),
                            i18n.t("configs_terminal.suggestion_message"),
                        );
                    }

                    return;
                }

                ComboBox::from_label(
                    RichText::new(i18n.t("configs_terminal.combo_box_label"))
                        .color(current_theme.text_primary.to_color()),
                )
                .selected_text(
                    self.get_selected_terminal_text(&i18n.t("configs_terminal.selection")),
                )
                .show_ui(ui, |ui| {
                    for term in &self.available_terminals {
                        let is_selected = self.is_terminal_selected(term);

                        let label =
                            RichText::new(term).color(current_theme.text_primary.to_color());

                        if ui.selectable_label(is_selected, label).clicked() {
                            with_configs(|c| {
                                c.set_default_terminal(term.to_string());
                            });
                        }
                    }
                });

                ui.add_space(12.0);
                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_terminal.message"))
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );
            });
        });
    }

    fn render_backend_settings(&mut self, ui: &mut Ui, _query: &str, frame: Frame) {
        let current_theme = with_theme(|t| t.current());

        let i18n = with_configs(|c| c.get_i18n());

        ui.add_space(10.0);
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_backend.title"))
                            .strong()
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );

                ui.add_space(8.0);

                let current = with_configs(|c| c.get_display_backend());

                ComboBox::from_label(
                    RichText::new(i18n.t("configs_backend.selection"))
                        .color(current_theme.text_primary.to_color()),
                )
                .selected_text(current.name())
                .show_ui(ui, |ui| {
                    for (name, backend) in [
                        ("Auto", DisplayBackend::Auto),
                        ("Wayland", DisplayBackend::Wayland),
                        ("X11", DisplayBackend::X11),
                    ] {
                        let label =
                            RichText::new(name).color(current_theme.text_primary.to_color());

                        if ui.selectable_label(current == backend, label).clicked() {
                            with_configs(|c| {
                                c.set_display_backend(backend);
                            });
                        }
                    }
                });
            });

            ui.add_space(12.0);

            ui.add(
                Label::new(
                    RichText::new(i18n.t("configs_backend.hint"))
                        .color(current_theme.text_primary.to_color()),
                )
                .selectable(false),
            );

            ui.add(
                Label::new(
                    RichText::new(i18n.t("configs_backend.message"))
                        .color(current_theme.text_secondary.to_color()),
                )
                .selectable(false),
            );
        });
    }

    fn render_language_settings(&mut self, ui: &mut Ui, _query: &str, frame: Frame) {
        let current_theme = with_theme(|t| t.current());
        let i18n = with_configs(|c| c.get_i18n());

        ui.add_space(10.0);
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_language.title"))
                            .strong()
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );

                ui.add_space(8.0);

                let current = with_configs(|c| c.get_locale());

                let languages = ["en", "es", "it", "ru", "fr", "de"];

                ComboBox::from_label(
                    RichText::new(i18n.t("configs_language.selection"))
                        .color(current_theme.text_primary.to_color()),
                )
                .selected_text(&current)
                .show_ui(ui, |ui| {
                    for lang in languages {
                        let label =
                            RichText::new(lang).color(current_theme.text_primary.to_color());

                        if ui.selectable_label(current == lang.into(), label).clicked() {
                            with_configs(|c| {
                                c.set_locale(lang);
                            });
                        }
                    }
                });
            });

            ui.add(
                Label::new(
                    RichText::new(i18n.t("configs_language.message"))
                        .color(current_theme.text_secondary.to_color()),
                )
                .selectable(false),
            );
        });
    }

    fn render_appearance_settings(&mut self, ui: &mut Ui, _query: &str, frame: Frame) {
        let i18n = with_configs(|c| c.get_i18n());
        let (available_themes, current_theme) =
            with_theme(|t| (t.get_available_themes_names(), t.current().clone()));

        ui.add_space(10.0);

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // Titulo
                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_appearance.title"))
                            .strong()
                            .color(current_theme.text_primary.to_color()),
                    )
                    .selectable(false),
                );
                ui.add_space(8.0);

                // Recargar
                if ui
                    .button(
                        RichText::new(i18n.t("configs_appearance.reload_theme"))
                            .color(current_theme.text_primary.to_color()),
                    )
                    .clicked()
                {
                    with_theme(|t| match t.reload() {
                        Ok(_) => info!("Tema recargado con éxito."),
                        Err(e) => warn!("Error al recargar el tema: {e}."),
                    });
                }

                // Selector de temas
                ComboBox::from_label(
                    RichText::new(i18n.t("configs_appearance.selection"))
                        .color(current_theme.text_primary.to_color()),
                )
                .selected_text(&*current_theme.name)
                .show_ui(ui, |ui| {
                    for theme in available_themes {
                        let label =
                            RichText::new(&*theme).color(current_theme.text_primary.to_color());
                        if ui
                            .selectable_label(current_theme.name == theme, label)
                            .clicked()
                        {
                            with_configs(|c| c.set_current_theme_name(&theme));
                            with_theme(|t| t.set_theme(&theme));
                        }
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Editor
                ui.label(
                    RichText::new(i18n.t("configs_appearance.color_editor"))
                        .strong()
                        .color(current_theme.text_primary.to_color()),
                );
                ui.add_space(6.0);

                ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                    macro_rules! color_field {
                        ($label:expr, $($path:tt)*) => {{
                            ui.horizontal(|ui| {
                                let current_hex = &current_theme.$($path)*;
                                let mut color = Color32::from_hex(current_hex)
                                    .unwrap_or(Color32::DEBUG_COLOR);

                                ui.label(
                                    RichText::new($label)
                                        .color(current_theme.text_primary.to_color()),
                                );

                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    with_theme(|t| {
                                        t.update_theme(
                                            |theme, c| {
                                                theme.$($path)* = format!(
                                                    "#{:02X}{:02X}{:02X}{:02X}",
                                                    c.r(), c.g(), c.b(), c.a()
                                                );
                                            },
                                            color,
                                        );
                                    });

                                }
                            });
                        }};
                    }

                    // --- Estados ---
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.states"))
                            .strong()
                            .color(current_theme.text_secondary.to_color()),
                    );
                    color_field!(i18n.t("configs_appearance.error"), error);
                    color_field!(i18n.t("configs_appearance.success"), success);
                    color_field!(i18n.t("configs_appearance.warn"), warn);
                    ui.add_space(8.0);

                    // --- Fondos ---
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.background"))
                            .strong()
                            .color(current_theme.text_secondary.to_color()),
                    );
                    color_field!(i18n.t("configs_appearance.main_bg"), bg_main);
                    color_field!(i18n.t("configs_appearance.panel_bg"), bg_panel);
                    color_field!(i18n.t("configs_appearance.container_bg"), bg_container);
                    color_field!(i18n.t("configs_appearance.hover_bg"), bg_hover);
                    ui.add_space(8.0);

                    // --- Bordes y botones ---
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.border-buttons"))
                            .strong()
                            .color(current_theme.text_secondary.to_color()),
                    );
                    color_field!(i18n.t("configs_appearance.border_panel"), border_panel);
                    color_field!(i18n.t("configs_appearance.main_buttons"), main_buttons);
                    ui.add_space(8.0);

                    // --- Acentos ---
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.accent"))
                            .strong()
                            .color(current_theme.text_secondary.to_color()),
                    );
                    color_field!(i18n.t("configs_appearance.accent"), accent);
                    color_field!(i18n.t("configs_appearance.accent_glow"), accent_glow);
                    color_field!(i18n.t("configs_appearance.rubberband"), rubberband);
                    color_field!(i18n.t("configs_appearance.selected_item"), item_selected);
                    ui.add_space(8.0);

                    // --- Textos ---
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.texts"))
                            .strong()
                            .color(current_theme.text_secondary.to_color()),
                    );
                    color_field!(i18n.t("configs_appearance.primary_text"), text_primary);
                    color_field!(i18n.t("configs_appearance.secondary_text"), text_secondary);
                    color_field!(i18n.t("configs_appearance.muted_text"), text_muted);
                    ui.add_space(8.0);

                    // --- Herramientas ---
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.tools"))
                            .strong()
                            .color(current_theme.text_secondary.to_color()),
                    );
                    color_field!(
                        i18n.t("configs_appearance.primary-tool-color"),
                        tools_primary
                    );
                    color_field!(
                        i18n.t("configs_appearance.secondary-tool-color"),
                        tools_secondary
                    );
                    color_field!(
                        i18n.t("configs_appearance.tool-btn-active"),
                        tool_btn_active
                    );
                    color_field!(
                        i18n.t("configs_appearance.tool-btn-inactive"),
                        tool_btn_inactive
                    );
                    color_field!(
                        i18n.t("configs_appearance.tool-btn-hovered"),
                        tool_btn_hovered
                    );
                    ui.add_space(8.0);

                    // --- Iconos de archivo ---
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.files"))
                            .strong()
                            .color(current_theme.text_secondary.to_color()),
                    );
                    color_field!(
                        i18n.t("configs_appearance.folder"),
                        file_theme.folder_default
                    );
                    color_field!(i18n.t("configs_appearance.image"), file_theme.image);
                    color_field!(i18n.t("configs_appearance.pdf"), file_theme.pdf);
                    color_field!(i18n.t("configs_appearance.doc"), file_theme.document);
                    color_field!(i18n.t("configs_appearance.video"), file_theme.video);
                    color_field!(i18n.t("configs_appearance.audio"), file_theme.audio);
                    color_field!(i18n.t("configs_appearance.archive"), file_theme.archive);
                    color_field!(i18n.t("configs_appearance.code"), file_theme.code);
                    color_field!(i18n.t("configs_appearance.font"), file_theme.font);
                    color_field!(i18n.t("configs_appearance.exe"), file_theme.executable);
                    color_field!(i18n.t("configs_appearance.fallback"), file_theme.fallback);
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Input para nombre de tema personalizado
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(i18n.t("configs_appearance.theme-name"))
                            .color(current_theme.text_primary.to_color()),
                    );

                    let text_edit = TextEdit::singleline(&mut self.custom_theme_name)
                        .id("save_theme_placeholder".into());

                    ui.add(text_edit);

                    if ui
                        .button(
                            RichText::new(i18n.t("configs_appearance.save-as-custom"))
                                .color(current_theme.text_primary.to_color()),
                        )
                        .clicked()
                    {
                        if !self.custom_theme_name.trim().is_empty() {
                            with_theme(|t| {
                                match t.save_as_custom_theme(self.custom_theme_name.trim()) {
                                    Ok(_) => info!("Tema personalizado guardado"),
                                    Err(e) => warn!("Error al guardar tema: {e}"),
                                }
                            });
                        } else {
                            warn!("El nombre del tema no puede estar vacío");
                        }
                    }
                });

                ui.add_space(8.0);

                // Botone de reset
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new(i18n.t("configs_appearance.reset-defaults"))
                                .color(current_theme.error.to_color()),
                        )
                        .clicked()
                    {
                        with_theme(|t| match t.reset_to_default() {
                            Ok(_) => info!("Tema reseteado a valores por defecto"),
                            Err(e) => warn!("Error al resetear tema: {e}"),
                        });
                    }
                });

                ui.add_space(8.0);
                ui.label(
                    RichText::new(i18n.t("configs_appearance.message"))
                        .color(current_theme.text_secondary.to_color()),
                );
            });
        });
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let current_theme = with_theme(|t| t.current());

        let i18n = with_configs(|c| c.get_i18n());

        let mut should_close = false;

        let mut is_open = self.show_modal;

        if !self.show_modal {
            return false;
        }

        let custom_frame = Frame::window(ui.style())
            .fill(Color32::from_rgba_unmultiplied(
                current_theme.bg_main.to_color().r(),
                current_theme.bg_main.to_color().g(),
                current_theme.bg_main.to_color().b(),
                122,
            ))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        let screen_rect = ui.viewport_rect();
        let desired_width = screen_rect.width() * 0.68;
        let desired_height = screen_rect.height() * 0.65;

        let frame = Frame::new()
            .corner_radius(CornerRadius::same(10))
            .fill(current_theme.bg_main.to_color())
            .outer_margin(Margin::same(5));

        Window::new(
            RichText::new(i18n.t("configs.title")).color(current_theme.text_primary.to_color()),
        )
        .id(Id::new("config_modal_window"))
        .frame(custom_frame)
        .order(Order::Foreground)
        .default_size([desired_width, desired_height])
        .min_size([300.0, 200.0])
        .max_size([screen_rect.width() * 0.9, screen_rect.height() * 0.9])
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut is_open)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            Panel::left("config_left_panel")
                .show_separator_line(false)
                .resizable(false)
                .frame(frame)
                .show_inside(ui, |ui| {
                    ui.set_width(160.0);
                    ui.add_space(8.0);

                    self.render_config_sidebar(ui);
                });

            CentralPanel::default().frame(frame).show_inside(ui, |ui| {
                ui.set_max_width(ui.available_width());
                ui.set_max_height(ui.available_height());

                let query = self.config_search.trim().to_lowercase();
                let frame = Frame::new().inner_margin(20.0);

                match self.current_config_tab {
                    CurrentConfigTab::General => self.render_general_settings(ui, &query, frame),
                    CurrentConfigTab::Terminal => self.render_terminal_settings(ui, &query, frame),
                    CurrentConfigTab::Backend => self.render_backend_settings(ui, &query, frame),
                    CurrentConfigTab::Appearance => {
                        self.render_appearance_settings(ui, &query, frame)
                    }
                    CurrentConfigTab::Behavior => {}
                    CurrentConfigTab::Language => self.render_language_settings(ui, &query, frame),
                }
            });
        });

        if !is_open {
            should_close = true;
        }

        if ui.input(|i| i.key_pressed(Key::Escape)) {
            should_close = true;
        }

        if should_close {
            self.show_modal = false;
        }

        should_close
    }
}
