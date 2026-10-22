use crate::core::bootstrap::configs::{
    config_manager::with_configs, platform::linux::conf_structs::DisplayBackend,
};
use crate::ui::custom_components::UiExt;
use crate::ui::icons_cache::icon_cache::IconCache;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use egui::{Color32, Label, RichText, Ui};

pub(crate) fn render_backend_settings(ui: &mut Ui, _query: &str, icon_cache: &mut IconCache) {
    let current_theme = with_theme(|t| t.current());
    let is_dark_theme = current_theme.semantic.text_primary.to_color().r() > 128;
    let i18n = with_configs(|c| c.get_i18n());

    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_backend.title"))
                    .strong()
                    .color(current_theme.semantic.text_primary.to_color()),
            )
            .selectable(false),
        );

        ui.add_space(8.0);

        let current = with_configs(|c| c.get_display_backend());

        let cbg = current_theme.components.button.bg_hover.to_color();
        let hover_bg = if is_dark_theme {
            Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 80)
        } else {
            Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 10)
        };

        ui.label_ns(i18n.t("configs_backend.selection"));

        ui.horizontal(|ui| {
            ui.blaze_combobox(Some(icon_cache))
                .selected_text(current.name())
                .text_color(current_theme.semantic.text_primary.to_color())
                .size((150.0, 30.0))
                .background_color(current_theme.components.button.bg.to_color())
                .hover_background_color(hover_bg)
                .show(ui, |ui| {
                    let w = ui.available_width();

                    for (i, (name, backend)) in [
                        ("Auto", DisplayBackend::Auto),
                        ("Wayland", DisplayBackend::Wayland),
                        ("X11", DisplayBackend::X11),
                    ]
                    .iter()
                    .enumerate()
                    {
                        let label = RichText::new(*name)
                            .color(current_theme.semantic.text_primary.to_color());

                        let bg = if current == *backend {
                            current_theme.semantic.accent_glow.to_color()
                        } else {
                            current_theme.components.button.bg.to_color()
                        };

                        if ui
                            .blaze_button(label, None)
                            .id(format!("backend_{i}"))
                            .size((w, 20.0))
                            .background_color(bg)
                            .hover_background_color(hover_bg)
                            .stroke_color(Color32::TRANSPARENT)
                            .radius(20)
                            .show(ui)
                            .clicked()
                        {
                            with_configs(|c| {
                                c.set_display_backend(backend.clone());
                            });
                        }
                    }
                });
        });

        ui.add_space(12.0);

        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_backend.hint"))
                    .color(current_theme.semantic.text_primary.to_color()),
            )
            .selectable(false),
        );

        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_backend.message"))
                    .color(current_theme.semantic.text_secondary.to_color()),
            )
            .selectable(false),
        );
    });
}
