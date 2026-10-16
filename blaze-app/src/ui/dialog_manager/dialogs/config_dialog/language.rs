use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use egui::{ComboBox, Label, RichText, Ui};

pub(crate) fn render_language_settings(ui: &mut Ui, _query: &str) {
    let current_theme = with_theme(|t| t.current());
    let i18n = with_configs(|c| c.get_i18n());

    ui.add_space(10.0);
    ui.vertical(|ui| {
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_language.title"))
                    .strong()
                    .color(current_theme.semantic.text_primary.to_color()),
            )
            .selectable(false),
        );

        ui.add_space(8.0);

        let current = with_configs(|c| c.get_locale());

        let languages = ["en", "es", "it", "ru", "fr", "de"];

        ComboBox::from_label(
            RichText::new(i18n.t("configs_language.selection"))
                .color(current_theme.semantic.text_primary.to_color()),
        )
        .selected_text(&current)
        .show_ui(ui, |ui| {
            for lang in languages {
                let label =
                    RichText::new(lang).color(current_theme.semantic.text_primary.to_color());

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
                .color(current_theme.semantic.text_secondary.to_color()),
        )
        .selectable(false),
    );
}
