use crate::ui::themes::theme_manager::with_theme;
use crate::{core::bootstrap::configs::config_manager::with_configs, ui::custom_components::UiExt};
use egui::{Color32, Label, RichText, Ui};

pub(crate) fn render_language_settings(ui: &mut Ui, _query: &str) {
    let current_theme = with_theme(|t| t.current());
    let palette = current_theme.dialogs.palette(&current_theme.global);
    let i18n = with_configs(|c| c.get_i18n());

    ui.add_space(10.0);
    ui.vertical(|ui| {
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_language.title"))
                    .strong()
                    .color(palette.fg_primary),
            )
            .selectable(false),
        );

        ui.add_space(8.0);

        let current = with_configs(|c| c.get_locale());

        let languages = ["en", "es", "it", "ru", "fr", "de"];

        ui.label_ns(
            RichText::new(i18n.t("configs_language.selection")).color(palette.fg_secondary),
        );

        ui.horizontal(|ui| {
            ui.blaze_combobox()
                .selected_text(&current)
                .text_color(palette.fg_primary)
                .size((150.0, 30.0))
                .background_color(palette.bg_elevated)
                .hover_background_color(palette.bg_hover)
                .show(|ui| {
                    let w = ui.available_width();

                    for (i, lang) in languages.iter().enumerate() {
                        let lang: Box<str> = lang.to_owned().into();
                        let is_selected = current == lang;

                        let text_color = if is_selected {
                            palette.fg_primary
                        } else {
                            palette.fg_secondary
                        };
                        let label = RichText::new(&*lang).color(text_color);

                        let bg = if is_selected {
                            palette.bg_selected
                        } else {
                            Color32::TRANSPARENT
                        };

                        if ui
                            .blaze_button(label)
                            .id(format!("lang_{i}"))
                            .size((w, 20.0))
                            .background_color(bg)
                            .hover_background_color(palette.bg_hover)
                            .stroke_color(Color32::TRANSPARENT)
                            .radius(20)
                            .stroke_width(current_theme.global.components_stroke_width)
                            .show()
                            .clicked()
                        {
                            with_configs(|c| {
                                c.set_locale(&lang);
                            });
                        }
                    }
                });
        });
    });

    ui.add(
        Label::new(RichText::new(i18n.t("configs_language.message")).color(palette.fg_secondary))
            .selectable(false),
    );
}
