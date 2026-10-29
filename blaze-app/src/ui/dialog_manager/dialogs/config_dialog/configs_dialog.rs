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

use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::dialogs::config_dialog::appareance::render_appearance_settings;
use crate::ui::dialog_manager::dialogs::config_dialog::backend::render_backend_settings;
use crate::ui::dialog_manager::dialogs::config_dialog::general::render_general_settings;
use crate::ui::dialog_manager::dialogs::config_dialog::language::render_language_settings;
use crate::ui::dialog_manager::dialogs::config_dialog::terminal::render_terminal_settings;
use crate::ui::dialog_manager::manager::ModalDialog;
use crate::ui::icons_cache::icon_cache::IconCache;
use crate::ui::themes::theme_manager::with_theme;
use egui::{
    Align2, CentralPanel, Color32, CornerRadius, Frame, Id, Key, Margin, Order, Panel, RichText,
    Stroke, Ui, Vec2, Window,
};

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

    pub(crate) available_terminals: Vec<String>,
    pub(crate) loading_terminals: bool,
    pub(crate) terminal_rx: Option<tokio::sync::mpsc::Receiver<Vec<String>>>,
    pub(crate) terminals_loaded: bool,
    pub(crate) no_terminals_error: bool,
    pub(crate) retry_count: u8,
    pub(crate) max_retries: u8,
    pub(crate) custom_theme_name: String,
}

impl ModalDialog for ConfigDialog {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui, icon_cache: &mut IconCache) -> bool {
        self.render_dialog(ui, icon_cache)
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

    fn render_config_sidebar(&mut self, ui: &mut Ui, icon_cache: &mut IconCache) {
        let current_theme = with_theme(|t| t.current());
        ui.add_space(10.0);

        let search_id = egui::Id::new("config_search_bar");

        let w = ui.available_width();

        let resp = ui
            .blaze_input("Buscar...", None, &mut self.config_search, Some(icon_cache))
            .id(search_id)
            .size((w, 30.0))
            .radius(20)
            .text_color(current_theme.global.text_primary())
            .background_color(current_theme.sidebar_left.bg_elevated())
            .stroke_color(Color32::TRANSPARENT)
            .x_color(current_theme.sidebar_left.accent())
            .show(ui, || {});

        if resp.clicked() || resp.gained_focus() {
            ui.memory_mut(|mem| {
                mem.request_focus(search_id);
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

            let text_color = if is_selected {
                current_theme.global.text_primary()
            } else {
                current_theme.global.text_secondary()
            };
            let label = RichText::new(tab.name()).color(text_color);

            let hover_bg = current_theme.sidebar_left.bg_hover();
            let bg = if is_selected {
                current_theme.sidebar_left.bg_active()
            } else {
                current_theme.sidebar_left.bg_elevated()
            };

            if ui
                .blaze_button(label, Some(icon_cache))
                .size((w, 20.0))
                .background_color(bg)
                .hover_background_color(hover_bg)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .stroke_width(current_theme.global.components_stroke_width)
                .show(ui)
                .clicked()
            {
                self.current_config_tab = *tab;
            }
        }

        if filtered.is_empty() && !query.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label_ns(
                    RichText::new("No se encontraron resultados")
                        .weak()
                        .color(current_theme.global.text_secondary()),
                );
            });
        }
    }

    pub fn render_dialog(&mut self, ui: &mut Ui, icon_cache: &mut IconCache) -> bool {
        let current_theme = with_theme(|t| t.current());
        let i18n = with_configs(|c| c.get_i18n());

        let mut should_close = false;
        let mut is_open = self.show_modal;

        if !self.show_modal {
            return false;
        }

        let custom_frame = Frame::NONE
            .fill(current_theme.sidebar_left.bg_panel())
            .stroke(Stroke::new(
                current_theme.global.panel_stroke_width,
                current_theme.sidebar_left.border(),
            ))
            .corner_radius(CornerRadius::same(15))
            .inner_margin(Margin::same(5));

        let screen_rect = ui.viewport_rect();
        let desired_width = screen_rect.width() * 0.88;
        let desired_height = screen_rect.height() * 0.85;

        Window::new(
            RichText::new(i18n.t("configs.title")).color(current_theme.global.text_primary()),
        )
        .id(Id::new("config_modal_window"))
        .frame(custom_frame)
        .order(Order::Foreground)
        .default_size([desired_width, desired_height])
        .min_size([300.0, 200.0])
        .max_size([screen_rect.width() * 0.9, screen_rect.height() * 0.9])
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .open(&mut is_open)
        .show(ui, |ui| {
            let h = ui.available_height();
            ui.set_width(ui.available_width());
            ui.set_height(h);

            ui.horizontal(|ui| {
                ui.set_width(ui.available_width());
                ui.set_height(h);

                Panel::left("configs_sidebar")
                    .exact_size(160.0)
                    .resizable(false)
                    .show(ui, |ui| {
                        self.render_config_sidebar(ui, icon_cache);
                    });

                CentralPanel::default().show(ui, |ui| {
                    let query = self.config_search.trim().to_lowercase();

                    match self.current_config_tab {
                        CurrentConfigTab::General => render_general_settings(ui, &query),
                        CurrentConfigTab::Terminal => {
                            render_terminal_settings(self, ui, &query, icon_cache)
                        }
                        CurrentConfigTab::Backend => {
                            render_backend_settings(ui, &query, icon_cache)
                        }
                        CurrentConfigTab::Appearance => {
                            render_appearance_settings(self, ui, &query, icon_cache)
                        }
                        CurrentConfigTab::Behavior => {}
                        CurrentConfigTab::Language => {
                            render_language_settings(ui, &query, icon_cache)
                        }
                    }
                });
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
