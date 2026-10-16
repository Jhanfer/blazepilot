use crate::core::bootstrap::configs::{
    config_manager::with_configs, platform::linux::conf_structs::DisplayBackend,
};
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use egui::{ComboBox, Label, RichText, Ui};

pub(crate) fn render_backend_settings(ui: &mut Ui, _query: &str) {
    let current_theme = with_theme(|t| t.current());

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

        ComboBox::from_label(
            RichText::new(i18n.t("configs_backend.selection"))
                .color(current_theme.semantic.text_primary.to_color()),
        )
        .selected_text(current.name())
        .show_ui(ui, |ui| {
            for (name, backend) in [
                ("Auto", DisplayBackend::Auto),
                ("Wayland", DisplayBackend::Wayland),
                ("X11", DisplayBackend::X11),
            ] {
                let label =
                    RichText::new(name).color(current_theme.semantic.text_primary.to_color());

                if ui.selectable_label(current == backend, label).clicked() {
                    with_configs(|c| {
                        c.set_display_backend(backend);
                    });
                }
            }
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
