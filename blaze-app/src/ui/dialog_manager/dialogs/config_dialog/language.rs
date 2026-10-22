use crate::ui::icons_cache::icon_cache::IconCache;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use crate::{core::bootstrap::configs::config_manager::with_configs, ui::custom_components::UiExt};
use egui::{Color32, Label, RichText, Ui};

pub(crate) fn render_language_settings(ui: &mut Ui, _query: &str, icon_cache: &mut IconCache) {
    let current_theme = with_theme(|t| t.current());
    let is_dark_theme = current_theme.semantic.text_primary.to_color().r() > 128;
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

        let cbg = current_theme.components.button.bg_hover.to_color();
        let hover_bg = if is_dark_theme {
            Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 80)
        } else {
            Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 10)
        };

        ui.label_ns(i18n.t("configs_language.selection"));

        ui.horizontal(|ui| {
            ui.blaze_combobox(Some(icon_cache))
                .selected_text(&current)
                .text_color(current_theme.semantic.text_primary.to_color())
                .size((150.0, 30.0))
                .background_color(current_theme.components.button.bg.to_color())
                .hover_background_color(hover_bg)
                .show(ui, |ui| {
                    let w = ui.available_width();

                    for (i, lang) in languages.iter().enumerate() {
                        let lang: Box<str> = lang.to_owned().into();

                        let label = RichText::new(&*lang)
                            .color(current_theme.semantic.text_primary.to_color());

                        let bg = if current == lang {
                            current_theme.semantic.accent_glow.to_color()
                        } else {
                            current_theme.components.button.bg.to_color()
                        };

                        if ui
                            .blaze_button(label, None)
                            .id(format!("lang_{i}"))
                            .size((w, 20.0))
                            .background_color(bg)
                            .hover_background_color(hover_bg)
                            .stroke_color(Color32::TRANSPARENT)
                            .radius(20)
                            .show(ui)
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
        Label::new(
            RichText::new(i18n.t("configs_language.message"))
                .color(current_theme.semantic.text_secondary.to_color()),
        )
        .selectable(false),
    );
}
