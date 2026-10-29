use crate::core::bootstrap::configs::{
    config_manager::with_configs, platform::linux::conf_structs::DisplayBackend,
};
use crate::ui::custom_components::UiExt;
use crate::ui::icons_cache::icon_cache::IconCache;
use crate::ui::themes::theme_manager::with_theme;
use egui::{Color32, Label, RichText, Ui};

pub(crate) fn render_backend_settings(ui: &mut Ui, _query: &str, icon_cache: &mut IconCache) {
    let current_theme = with_theme(|t| t.current());
    let i18n = with_configs(|c| c.get_i18n());

    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_backend.title"))
                    .strong()
                    .color(current_theme.global.text_primary()),
            )
            .selectable(false),
        );

        ui.add_space(8.0);

        let current = with_configs(|c| c.get_display_backend());

        let hover_bg = current_theme.sidebar_left.bg_hover();

        ui.label_ns(i18n.t("configs_backend.selection"));

        ui.horizontal(|ui| {
            ui.blaze_combobox(Some(icon_cache))
                .selected_text(current.name())
                .text_color(current_theme.global.text_primary())
                .size((150.0, 30.0))
                .background_color(current_theme.sidebar_left.bg_elevated())
                .hover_background_color(current_theme.sidebar_left.bg_hover())
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
                        let is_selected = current == *backend;
                        let text_color = if is_selected {
                            current_theme.global.text_primary()
                        } else {
                            current_theme.global.text_secondary()
                        };
                        let label = RichText::new(*name).color(text_color);

                        let bg = if is_selected {
                            current_theme.sidebar_left.bg_active()
                        } else {
                            Color32::TRANSPARENT
                        };

                        if ui
                            .blaze_button(label, None)
                            .id(format!("backend_{i}"))
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
                    .color(current_theme.global.text_primary()),
            )
            .selectable(false),
        );

        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_backend.message"))
                    .color(current_theme.global.text_secondary()),
            )
            .selectable(false),
        );
    });
}
