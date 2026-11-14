use crate::ui::icons_cache::icons;
use crate::ui::themes::theme_manager::with_theme;
use crate::{core::bootstrap::configs::config_manager::with_configs, ui::custom_components::UiExt};
use egui::{
    Color32, Frame, Label, Margin, OpenUrl, RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    vec2,
};

pub(crate) fn render_general_settings(ui: &mut Ui, _query: &str) {
    let current_theme = with_theme(|t| t.current());
    let palette = current_theme.dialogs.palette(&current_theme.global);
    let i18n = with_configs(|c| c.get_i18n());

    ui.add_space(10.0);

    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            Frame::new()
                .inner_margin(Margin::symmetric(10, 4))
                .show(ui, |ui| {
                    let (blaze_icon, bytes) = ("blaze-logo", icons::LOGO_BLAZEPILOT);

                    let icon_size = vec2(35.0, 35.0);

                    ui.draw_icon((blaze_icon, bytes))
                        .size(icon_size)
                        .tint(current_theme.sidebar_left.accent())
                        .show();

                    ui.add(
                        Label::new(
                            RichText::new("BlazePilot")
                                .strong()
                                .size(25.0)
                                .color(current_theme.global.text_primary()),
                        )
                        .selectable(false),
                    );
                });

            ui.blaze_button(format!("v{}", env!("CARGO_PKG_VERSION")))
                .size((65.0, 15.0))
                .radius(20)
                .text_color(palette.fg_primary)
                .background_color(palette.accent)
                .stroke_color(Color32::TRANSPARENT)
                .set_is_active(false)
                .show();
        });

        ui.add_space(4.0);
        ui.add_space(20.0);

        let available_w = ui.available_width();
        let border_padding = 10.0;
        let (rect, _) =
            ui.allocate_exact_size(vec2(available_w - border_padding, 200.0), Sense::empty());

        ui.painter().rect(
            rect,
            20,
            palette.bg_elevated,
            Stroke::new(
                current_theme.global.components_stroke_width,
                palette.border_component,
            ),
            StrokeKind::Middle,
        );

        let content_rext = rect.shrink(10.0);
        ui.scope_builder(UiBuilder::new().max_rect(content_rext), |ui| {
            ui.vertical_centered(|ui| {
                ui.set_width(ui.available_width());

                ui.label_ns(
                    RichText::new(i18n.t("configs_general.gratitude"))
                        .color(palette.fg_primary)
                        .size(18.0),
                );

                ui.label_ns(
                    RichText::new(i18n.t("configs_general.made_for"))
                        .color(palette.fg_primary)
                        .size(18.0),
                );

                ui.add_space(10.0);

                ui.label_ns(
                    RichText::new(i18n.t("configs_general.disclaimer"))
                        .color(palette.fg_secondary)
                        .italics()
                        .weak(),
                );

                ui.add_space(60.0);

                ui.horizontal(|ui| {
                    let spacing = ui.spacing().item_spacing.x;
                    let total_buttons_width = 100.0 * 2.0 + spacing;
                    let left_offset = ((ui.available_width() - total_buttons_width) / 2.0).max(0.0);

                    ui.add_space(left_offset);

                    if ui
                        .blaze_button("GitHub")
                        .size((100.0, 15.0))
                        .icon("github", icons::ICON_GITHUB)
                        .icon_color(palette.icon_primary)
                        .hover_background_color(palette.icon_hover)
                        .radius(20)
                        .background_color(palette.accent)
                        .hover_background_color(palette.bg_hover)
                        .text_color(palette.fg_primary)
                        .stroke_color(Color32::TRANSPARENT)
                        .show()
                        .clicked()
                    {
                        ui.open_url(OpenUrl::new_tab("https://github.com/Jhanfer/blazepilot"));
                    }

                    if ui
                        .blaze_button("Report")
                        .size((100.0, 15.0))
                        .icon("bug", icons::ICON_BUG)
                        .icon_color(palette.icon_primary)
                        .hover_background_color(palette.icon_hover)
                        .radius(20)
                        .background_color(palette.bg_elevated)
                        .hover_background_color(palette.bg_hover)
                        .text_color(palette.fg_primary)
                        .stroke_color(Color32::TRANSPARENT)
                        .show()
                        .clicked()
                    {
                        ui.open_url(OpenUrl::new_tab(
                            "https://github.com/Jhanfer/blazepilot/issues",
                        ));
                    }
                });
            });
        });
    });
}
