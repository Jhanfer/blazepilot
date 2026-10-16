use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use crate::{core::bootstrap::configs::config_manager::with_configs, ui::custom_components::UiExt};
use egui::{Label, OpenUrl, RichText, Ui};

pub(crate) fn render_general_settings(ui: &mut Ui, _query: &str) {
    let current_theme = with_theme(|t| t.current());
    let i18n = with_configs(|c| c.get_i18n());

    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.add(
            Label::new(
                RichText::new("BlazePilot")
                    .strong()
                    .color(current_theme.semantic.text_primary.to_color()),
            )
            .selectable(false),
        );
        ui.add_space(4.0);

        ui.add(
            Label::new(
                RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .color(current_theme.semantic.text_muted.to_color()),
            )
            .selectable(false),
        );

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_general.gratitude"))
                    .color(current_theme.semantic.text_primary.to_color()),
            )
            .selectable(false),
        );

        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_general.made_for"))
                    .color(current_theme.semantic.text_primary.to_color()),
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
                    RichText::new("•").color(current_theme.semantic.text_secondary.to_color()),
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
                .color(current_theme.semantic.text_secondary.to_color()),
            |ui| {
                ui.monospace(
                    RichText::new(format!("OS: {}", std::env::consts::OS))
                        .color(current_theme.semantic.text_secondary.to_color()),
                );
                ui.monospace(
                    RichText::new(format!("Arch: {}", std::env::consts::ARCH))
                        .color(current_theme.semantic.text_secondary.to_color()),
                );
            },
        );

        ui.add_space(20.0);
        ui.label_ns(
            RichText::new(i18n.t("configs_general.disclaimer"))
                .color(current_theme.semantic.text_muted.to_color())
                .italics()
                .weak(),
        );
    });
}
