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

use crate::ui::dialog_manager::dialog_manager::ModalDialog;
use crate::{
    core::{
        bootstrap::configs::{
            config_manager::with_configs, platform::linux::conf_structs::DisplayBackend,
        },
        system::{
            clipboard::clipboard::TOKIO_RUNTIME,
            terminal_opener::terminal_manager::GLOBAL_TERMINAL_MANAGER,
        },
    },
    ui::themes::colors::COLOR_BG_MAIN,
};
use core::f32;
use egui::{
    pos2, Area, CentralPanel, Color32, ComboBox, CornerRadius, Frame, Key, Margin, OpenUrl, Order,
    Panel, RichText, TextEdit, Ui, Window,
};
use std::time::Duration;
use tracing::warn;

#[derive(PartialEq, Clone, Copy)]
enum CurrentConfigTab {
    General,
    Terminal,
    Backend,
    Appearance,
    Behavior,
}

impl CurrentConfigTab {
    pub fn name(&self) -> &'static str {
        match self {
            CurrentConfigTab::General => "General",
            CurrentConfigTab::Terminal => "Terminal",
            CurrentConfigTab::Backend => "Backend",
            CurrentConfigTab::Appearance => "Apariencia",
            CurrentConfigTab::Behavior => "Comportamiento",
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let q = query.to_lowercase();

        let name_matches = self.name().to_lowercase().contains(&q);

        let content_matches = match self {
            CurrentConfigTab::General => false,

            CurrentConfigTab::Terminal => [
                "terminal",
                "shell",
                "bash",
                "zsh",
                "cmd",
                "powershell",
                "prompt",
                "font",
                "tamaño",
                "size",
                "command",
            ]
            .iter()
            .any(|&k| k.contains(&q) || q.contains(k)),

            CurrentConfigTab::Backend => [
                "backend",
                "gpu",
                "cpu",
                "render",
                "vulkan",
                "opengl",
                "direui",
                "metal",
                "renderer",
                "aceleración",
                "protocolo de pantalla",
            ]
            .iter()
            .any(|&k| k.contains(&q) || q.contains(k)),

            CurrentConfigTab::Appearance => [
                "color",
                "tema",
                "theme",
                "dark",
                "light",
                "font",
                "icono",
                "icon",
                "background",
                "fondo",
            ]
            .iter()
            .any(|&k| k.contains(&q) || q.contains(k)),

            CurrentConfigTab::Behavior => [
                "auto",
                "guardar",
                "save",
                "backup",
                "comportamiento",
                "confirm",
                "confirmar",
                "undo",
                "deshacer",
            ]
            .iter()
            .any(|&k| k.contains(&q) || q.contains(k)),
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
}

impl ModalDialog for ConfigDialog {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui) {
        self.render_dialog(ui);
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
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self) {
        self.show_modal = true;
    }

    fn render_config_sidebar(&mut self, ui: &mut Ui) {
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
            ];

            let filtered: Vec<CurrentConfigTab> = all_tabs
                .iter()
                .copied()
                .filter(|&tab| tab.matches_search(&query))
                .collect();

            for tab in &filtered {
                let is_selected = self.current_config_tab == *tab;

                if ui.selectable_label(is_selected, tab.name()).clicked() {
                    self.current_config_tab = *tab;
                }
            }

            if filtered.is_empty() && !query.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("No se encontraron resultados").weak());
                });
            }
        });
    }

    fn render_general_settings(&self, ui: &mut Ui, _query: &str, frame: Frame) {
        ui.add_space(10.0);
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.add_space(20.0);

                ui.heading("BlazePilot");
                ui.add_space(4.0);

                ui.label(RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak());

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(20.0);

                ui.label("Gracias por usar BlazePilot!");
                ui.label("Hecho con ❤️ por Jhanfer");

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.link("GitHub").clicked() {
                        ui.open_url(OpenUrl::new_tab("https://github.com/Jhanfer/blazepilot"));
                    }
                    ui.label("•");
                    if ui.link("Reportar bug").clicked() {
                        ui.open_url(OpenUrl::new_tab(
                            "https://github.com/Jhanfer/blazepilot/issues",
                        ));
                    }
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);

                ui.collapsing("Información del sistema", |ui| {
                    ui.monospace(format!("OS: {}", std::env::consts::OS));
                    ui.monospace(format!("Arch: {}", std::env::consts::ARCH));
                });

                ui.add_space(20.0);
                ui.label(
                    RichText::new("Las configuraciones son limitadas por ahora.")
                        .italics()
                        .weak(),
                );
            });
        });
    }

    fn get_selected_terminal_text(&self) -> String {
        with_configs(|c| {
            if c.get_default_terminal().trim().is_empty() {
                "Seleccionar terminal".to_string()
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
                ui.heading("Seleccionar terminal");
                ui.add_space(8.0);

                if self.loading_terminals {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("Cargando terminales... (Intento {}/{})", 
                        self.retry_count, self.max_retries));
                    });
                    return;
                }

                if self.available_terminals.is_empty() {
                    let mut show_retry = false;

                    if self.no_terminals_error {
                        ui.colored_label(Color32::RED,"⚠️ No se han encontrado terminales en el sistema.");

                        if self.retry_count < self.max_retries {
                            show_retry = true;

                            if ui.button("Reintentar").clicked() {
                                self.reset_terminal_loading_state();
                            }
                        } else {
                            ui.colored_label(Color32::from_rgb(255, 165, 0),
                                "❌ No se ha podido encontrar ninguna terminal después de varios intentos.");
                            ui.label("Por favor, verifique que tenga al menos una terminal instalada.");
                        }
                    } else {
                        ui.label("No se encontraron terminales.");
                    }

                    if show_retry {
                        ui.add_space(8.0);
                        ui.colored_label(egui::Color32::from_rgb(255, 165, 0),
                        "💡 Sugerencia: Instale una terminal como gnome-terminal, konsole, o alacritty");
                    }

                    return;
                }


                egui::ComboBox::from_label("Terminal predeterminado")
                    .selected_text(self.get_selected_terminal_text())
                    .show_ui(ui, |ui| {
                        for term in &self.available_terminals {
                            let is_selected = self.is_terminal_selected(term);

                            if ui.selectable_label(is_selected, term).clicked() {
                                with_configs(|c| {
                                    c.set_default_terminal(term.to_string());
                                });
                            }
                        }
                    });

                ui.add_space(12.0);
                ui.label("Aquí puede seleccionar la terminal predeterminada para abrir.");
            });
        });
    }

    fn render_backend_settings(&mut self, ui: &mut Ui, _query: &str, frame: Frame) {
        ui.add_space(10.0);
        frame.show(ui, |ui|{
            ui.vertical(|ui|{

            ui.heading("Protocolo de pantalla");

            ui.add_space(8.0);

            let current = with_configs(|c|c.get_display_backend());

            ComboBox::from_label("Backend")
                .selected_text(current.name())
                .show_ui(ui, |ui| {
                    for (name, backend) in [("Auto", DisplayBackend::Auto), ("Wayland", DisplayBackend::Wayland), ("X11", DisplayBackend::X11)] {

                        if ui.selectable_label(current == backend, name).clicked() {
                            with_configs(|c| {
                                c.set_display_backend(backend);
                            });
                        }
                    }
                });
            });

            ui.add_space(12.0);
            ui.label(egui::RichText::new("¿Qué backend debería usar?").strong());
            ui.label("Auto: Recomendado para la mayoría de usuarios.");
            ui.label("Wayland: Mejor rendimiento y calidad visual.");
            ui.label("X11: Mayor compatibilidad con apps antiguas. Permite el drag & drop desde otras aplicaciones.");

            ui.add_space(20.0);
            ui.label("Estos cambios requieren de reiniciar la app.");
        });
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) {
        let mut config_open = self.show_modal;

        if !self.show_modal {
            return;
        }

        let custom_frame = Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(16, 21, 25, 0))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        let screen_rect = ui.viewport_rect();
        let desired_width = screen_rect.width() * 0.68;
        let desired_height = screen_rect.height() * 0.65;

        let frame = Frame::new()
            .corner_radius(CornerRadius::same(10))
            .fill(COLOR_BG_MAIN)
            .outer_margin(Margin::same(5));

        let mut close_requested = false;

        Window::new("Configuraciones")
            .frame(custom_frame)
            .order(Order::Foreground)
            .default_size([desired_width, desired_height])
            .min_size([300.0, 200.0])
            .max_size([screen_rect.width() * 0.9, screen_rect.height() * 0.9])
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut config_open)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());

                let side = Panel::left("config_left_panel")
                    .show_separator_line(false)
                    .resizable(false)
                    .frame(frame)
                    .show_inside(ui, |ui| {
                        ui.set_width(160.0);
                        ui.add_space(8.0);

                        self.render_config_sidebar(ui);
                    });

                let central = CentralPanel::default().frame(frame).show_inside(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());

                    let query = self.config_search.trim().to_lowercase();
                    let frame = Frame::new().inner_margin(20.0);

                    match self.current_config_tab {
                        CurrentConfigTab::General => {
                            self.render_general_settings(ui, &query, frame)
                        }
                        CurrentConfigTab::Terminal => {
                            self.render_terminal_settings(ui, &query, frame)
                        }
                        CurrentConfigTab::Backend => {
                            self.render_backend_settings(ui, &query, frame)
                        }
                        CurrentConfigTab::Appearance => {}
                        CurrentConfigTab::Behavior => {}
                    }
                });

                let all_rect = side.response.rect.union(central.response.rect);

                Area::new("cerrar".into())
                    .order(Order::Middle)
                    .fixed_pos(pos2(all_rect.center().x, all_rect.bottom() + 8.0))
                    .show(ui, |ui| {
                        if ui.button("cerrar").clicked() {
                            close_requested = true;
                        }
                    });
            });

        self.show_modal = config_open;

        let input = ui.input(|i| i.clone());

        if input.key_pressed(Key::Escape) || close_requested {
            self.close();
        }
    }
}
