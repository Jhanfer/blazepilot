use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::system::{
    clipboard::global_clipboard::TOKIO_RUNTIME,
    terminal_opener::terminal_manager::GLOBAL_TERMINAL_MANAGER,
};
use crate::ui::dialog_manager::dialogs::config_dialog::configs_dialog::ConfigDialog;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use egui::{ComboBox, Label, RichText, Ui};
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

pub(crate) fn render_terminal_settings(c_dialog: &mut ConfigDialog, ui: &mut Ui, _query: &str) {
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
                    .color(current_theme.semantic.text_primary.to_color()),
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
                        .color(current_theme.semantic.text_primary.to_color()),
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
                    current_theme.semantic.error.to_color(),
                    i18n.t("configs_terminal.no_terminals_error"),
                );

                if c_dialog.retry_count < c_dialog.max_retries {
                    show_retry = true;

                    if ui.button(i18n.t("configs_terminal.retry")).clicked() {
                        reset_terminal_loading_state(c_dialog);
                    }
                } else {
                    ui.colored_label(
                        current_theme.semantic.warn.to_color(),
                        i18n.t("configs_terminal.max_retries_error"),
                    );

                    ui.add(
                        Label::new(
                            RichText::new(i18n.t("configs_terminal.verify_terminals_message"))
                                .color(current_theme.semantic.text_secondary.to_color()),
                        )
                        .selectable(false),
                    );
                }
            } else {
                ui.add(
                    Label::new(
                        RichText::new(i18n.t("configs_terminal.no_terminals_error"))
                            .color(current_theme.semantic.text_secondary.to_color()),
                    )
                    .selectable(false),
                );
            }

            if show_retry {
                ui.add_space(8.0);
                ui.colored_label(
                    current_theme.semantic.warn.to_color(),
                    i18n.t("configs_terminal.suggestion_message"),
                );
            }

            return;
        }

        ComboBox::from_label(
            RichText::new(i18n.t("configs_terminal.combo_box_label"))
                .color(current_theme.semantic.text_primary.to_color()),
        )
        .selected_text(get_selected_terminal_text(
            &i18n.t("configs_terminal.selection"),
        ))
        .show_ui(ui, |ui| {
            for term in &c_dialog.available_terminals {
                let is_selected = is_terminal_selected(term);

                let label =
                    RichText::new(term).color(current_theme.semantic.text_primary.to_color());

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
                    .color(current_theme.semantic.text_primary.to_color()),
            )
            .selectable(false),
        );
    });
}
