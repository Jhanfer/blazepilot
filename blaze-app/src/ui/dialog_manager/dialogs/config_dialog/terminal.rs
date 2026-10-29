use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::system::{
    clipboard::global_clipboard::TOKIO_RUNTIME,
    terminal_opener::terminal_manager::GLOBAL_TERMINAL_MANAGER,
};
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::dialogs::config_dialog::configs_dialog::ConfigDialog;
use crate::ui::icons_cache::icon_cache::IconCache;
use crate::ui::themes::theme_manager::with_theme;
use egui::{Color32, Label, RichText, Ui};
use std::time::Duration;
use tracing::warn;

fn get_selected_terminal_text(label: &str) -> String {
    with_configs(|c| {
        if c.get_default_terminal().trim().is_empty() {
            label.to_string()
        } else {
            c.get_default_terminal()
        }
    })
}

fn is_terminal_selected(term: &str) -> bool {
    with_configs(|c| c.get_default_terminal() == term)
}

fn reset_terminal_loading_state(c_dialog: &mut ConfigDialog) {
    c_dialog.available_terminals.clear();
    c_dialog.loading_terminals = false;
    c_dialog.terminals_loaded = false;
    c_dialog.no_terminals_error = false;
    c_dialog.terminal_rx = None;
}

pub(crate) fn render_terminal_settings(
    c_dialog: &mut ConfigDialog,
    ui: &mut Ui,
    _query: &str,
    icon_cache: &mut IconCache,
) {
    let current_theme = with_theme(|t| t.current());
    let i18n = with_configs(|c| c.get_i18n());

    ui.add_space(10.0);

    if c_dialog.available_terminals.is_empty()
        && !c_dialog.loading_terminals
        && !c_dialog.terminals_loaded
        && (c_dialog.retry_count < c_dialog.max_retries)
    {
        c_dialog.loading_terminals = true;
        c_dialog.retry_count += 1;

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        c_dialog.terminal_rx = Some(rx);

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

    if let Some(rx) = &mut c_dialog.terminal_rx
        && let Ok(terminals) = rx.try_recv()
    {
        c_dialog.available_terminals = terminals.clone();
        c_dialog.loading_terminals = false;
        c_dialog.terminals_loaded = true;
        c_dialog.terminal_rx = None;

        if terminals.is_empty() {
            c_dialog.no_terminals_error = true;
            if c_dialog.retry_count >= c_dialog.max_retries {
                warn!("Máximo de reintentos alcanzado para cargar terminales");
            }
        } else {
            c_dialog.no_terminals_error = false;
        }
    }

    ui.vertical(|ui| {
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_terminal.title"))
                    .strong()
                    .color(current_theme.global.text_primary()),
            )
            .selectable(false),
        );

        ui.add_space(8.0);

        if c_dialog.loading_terminals {
            ui.horizontal(|ui| {
                ui.spinner();

                ui.add(
                    Label::new(
                        RichText::new(i18n.t_args(
                            "configs_terminal.loading_message",
                            &[
                                ("query", &c_dialog.retry_count.to_string()),
                                ("query", &c_dialog.max_retries.to_string()),
                            ],
                        ))
                        .color(current_theme.global.text_primary()),
                    )
                    .selectable(false),
                );
            });
            return;
        }

        if c_dialog.available_terminals.is_empty() {
            let mut show_retry = false;

            if c_dialog.no_terminals_error {
                ui.colored_label(
                    current_theme.global.error(),
                    i18n.t("configs_terminal.no_terminals_error"),
                );

                if c_dialog.retry_count < c_dialog.max_retries {
                    show_retry = true;

                    if ui.button(i18n.t("configs_terminal.retry")).clicked() {
                        reset_terminal_loading_state(c_dialog);
                    }
                } else {
                    ui.colored_label(
                        current_theme.global.warn(),
                        i18n.t("configs_terminal.max_retries_error"),
                    );

                    ui.add(
                        Label::new(
                            RichText::new(i18n.t("configs_terminal.verify_terminals_message"))
                                .color(current_theme.global.text_secondary()),
                        )
                        .selectable(false),
                    );
                }
            } else {
                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_terminal.no_terminals_error"))
                            .color(current_theme.global.text_secondary()),
                    )
                    .selectable(false),
                );
            }

            if show_retry {
                ui.add_space(8.0);
                ui.colored_label(
                    current_theme.global.warn(),
                    i18n.t("configs_terminal.suggestion_message"),
                );
            }

            return;
        }

        let hover_bg = current_theme.sidebar_left.bg_hover();

        ui.label_ns(i18n.t("configs_terminal.combo_box_label"));

        ui.horizontal(|ui| {
            ui.blaze_combobox(Some(icon_cache))
                .selected_text(get_selected_terminal_text(
                    &i18n.t("configs_terminal.selection"),
                ))
                .text_color(current_theme.global.text_primary())
                .size((150.0, 30.0))
                .background_color(current_theme.sidebar_left.bg_elevated())
                .hover_background_color(current_theme.sidebar_left.bg_hover())
                .show(ui, |ui| {
                    let w = ui.available_width();
                    for (i, term) in c_dialog.available_terminals.iter().enumerate() {
                        let is_selected = is_terminal_selected(term);
                        let text_color = if is_selected {
                            current_theme.global.text_primary()
                        } else {
                            current_theme.global.text_secondary()
                        };

                        let label = RichText::new(term).color(text_color);

                        let bg = if is_selected {
                            current_theme.sidebar_left.bg_active()
                        } else {
                            Color32::TRANSPARENT
                        };

                        if ui
                            .blaze_button(label, None)
                            .id(format!("terminal{i}"))
                            .size((w, 20.0))
                            .background_color(bg)
                            .hover_background_color(hover_bg)
                            .stroke_color(Color32::TRANSPARENT)
                            .radius(20)
                            .stroke_width(current_theme.global.components_stroke_width)
                            .show(ui)
                            .clicked()
                        {
                            with_configs(|c| {
                                c.set_default_terminal(term.to_string());
                            });
                        }
                    }
                });
        });

        ui.add_space(12.0);
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_terminal.message"))
                    .color(current_theme.global.text_primary()),
            )
            .selectable(false),
        );
    });
}
