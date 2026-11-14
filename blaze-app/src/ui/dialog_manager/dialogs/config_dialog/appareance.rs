use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::dialogs::config_dialog::configs_dialog::ConfigDialog;
use crate::ui::examples::examples_components::{ExamplesUiState, render_examples_ui};
use crate::ui::icons_cache::icons;
use crate::ui::themes::{platform::color_ops, theme_manager::with_theme};
use egui::scroll_area::ScrollSource;
use egui::{CollapsingHeader, Color32, Id, Label, RichText, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};
use tracing::{info, warn};

macro_rules! color_field {
    ($ui:expr, $i18n:expr, $key:expr, $c_theme:expr, $($path:tt)*) => {{
        let label = $i18n.t($key);
        $ui.horizontal(|ui| {
            let palette = $c_theme.dialogs.palette(&$c_theme.global);
            let current_hex = &$c_theme.$($path)*;
            let mut color = Color32::from_hex(current_hex)
                .unwrap_or(Color32::DEBUG_COLOR);

            ui.label_ns(
                RichText::new(&*label)
                    .color(palette.fg_primary),
            );

            if ui.color_picker(&mut color, (&*label, stringify!($($path)*)))
                .background_color(palette.bg_elevated)
                .show()
                .changed() {
                with_theme(|t| {
                    t.update_theme(
                        |theme, c| {
                            let [r, g, b, a] = c.to_srgba_unmultiplied();
                            theme.$($path)* = format!(
                                "#{:02X}{:02X}{:02X}{:02X}",
                                r, g, b, a
                            ).into();
                        },
                        color,
                    );
                });
            }
        });
    }};
}

pub(crate) fn render_appearance_settings(c_dialog: &mut ConfigDialog, ui: &mut Ui, _query: &str) {
    let i18n = with_configs(|c| c.get_i18n());
    let (available_themes, current_theme) =
        with_theme(|t| (t.get_available_themes_names(), t.current().clone()));
    let palette = current_theme.dialogs.palette(&current_theme.global);

    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.set_height(ui.available_height());
        // Titulo
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_appearance.title"))
                    .strong()
                    .color(palette.fg_primary),
            )
            .selectable(false),
        );
        ui.add_space(8.0);

        ui.label_ns(
            RichText::new(&*i18n.t("configs_appearance.selection")).color(palette.fg_secondary),
        );
        ui.add_space(4.0);

        let hover_bg = palette.bg_hover;

        ui.horizontal(|ui| {
            ui.blaze_combobox()
                .selected_text(&*current_theme.meta.name)
                .text_color(palette.fg_primary)
                .size((150.0, 30.0))
                .background_color(palette.bg_elevated)
                .hover_background_color(palette.bg_hover)
                .show(|ui| {
                    let w = ui.available_width();

                    for (i, theme) in available_themes.iter().enumerate() {
                        let is_selected = current_theme.meta.name == *theme;
                        let text_color = if is_selected {
                            palette.fg_primary
                        } else {
                            palette.fg_secondary
                        };
                        let label = RichText::new(&**theme).color(text_color);

                        let bg = if is_selected {
                            palette.bg_selected
                        } else {
                            Color32::TRANSPARENT
                        };

                        if ui
                            .blaze_button(label)
                            .id(format!("theme_btn_{i}"))
                            .size((w, 20.0))
                            .background_color(bg)
                            .hover_background_color(hover_bg)
                            .stroke_color(Color32::TRANSPARENT)
                            .radius(20)
                            .show()
                            .clicked()
                        {
                            with_configs(|c| c.set_current_theme_name(theme));
                            with_theme(|t| t.set_theme(theme));
                        }
                    }
                });

            // Recargar
            if ui
                .blaze_button("")
                .id("reload_theme")
                .size((30.0, 30.0))
                .background_color(palette.bg_elevated)
                .hover_background_color(hover_bg)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("reset", icons::ICON_RESTORE)
                .icon_color(palette.accent)
                .stroke_width(current_theme.global.components_stroke_width)
                .show()
                .on_hover_text(&*i18n.t("configs_appearance.reload_theme"))
                .clicked()
            {
                with_theme(|t| match t.reload() {
                    Ok(_) => info!("Tema recargado con éxito."),
                    Err(e) => warn!("Error al recargar el tema: {e}."),
                });
            }
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        // Editor
        ui.label_ns(
            RichText::new(i18n.t("configs_appearance.color_editor"))
                .strong()
                .color(palette.fg_primary),
        );
        ui.add_space(6.0);

        let footer_h = 75.0;
        let content_h = (ui.available_height() - footer_h).max(100.0);

        StripBuilder::new(ui)
            .size(Size::exact(content_h))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    StripBuilder::new(ui)
                        .size(Size::initial(240.0).at_least(240.0).at_most(300.0))
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ScrollArea::vertical()
                                    .id_salt("appareance_color_scroll")
                                    .auto_shrink([false, false])
                                    .max_height(content_h)
                                    .scroll_source(
                                        ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR,
                                    )
                                    .show(ui, |ui| {
                                        // --- 1. Globales ---
                                        CollapsingHeader::new(
                                            RichText::new(
                                                &*i18n.t("configs_appearance.section_global"),
                                            )
                                            .color(palette.fg_primary),
                                        )
                                        .default_open(true) // Mantenemos el primero abierto por defecto
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.main_bg",
                                                current_theme,
                                                global.bg_main
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.rubberband",
                                                current_theme,
                                                global.rubberband
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.primary_text",
                                                current_theme,
                                                global.text_primary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.secondary_text",
                                                current_theme,
                                                global.text_secondary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.error",
                                                current_theme,
                                                global.error
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.warn",
                                                current_theme,
                                                global.warn
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.success",
                                                current_theme,
                                                global.success
                                            );

                                            ui.label_ns(
                                                RichText::new(
                                                    &*i18n.t("configs_appearance.stroke_width"),
                                                )
                                                .color(palette.fg_primary),
                                            );

                                            let mut panel_stroke =
                                                current_theme.global.panel_stroke_width;

                                            if ui
                                                .blaze_slider(&mut panel_stroke, 0.0..=1.0)
                                                .size((80.0, 20.0))
                                                .background_color(palette.bg_elevated)
                                                .accent(palette.accent)
                                                .handle_color(palette.accent)
                                                .handle_stroke_color(palette.bg_elevated)
                                                .show()
                                                .changed()
                                            {
                                                with_theme(|t| {
                                                    t.update_theme(
                                                        |theme, val| {
                                                            theme.global.panel_stroke_width = val;
                                                        },
                                                        panel_stroke,
                                                    );
                                                });
                                            }

                                            let mut component_stroke =
                                                current_theme.global.components_stroke_width;

                                            if ui
                                                .blaze_slider(&mut component_stroke, 0.0..=1.0)
                                                .size((80.0, 20.0))
                                                .background_color(palette.bg_elevated)
                                                .accent(palette.accent)
                                                .handle_color(palette.accent)
                                                .handle_stroke_color(palette.bg_elevated)
                                                .show()
                                                .changed()
                                            {
                                                with_theme(|t| {
                                                    t.update_theme(
                                                        |theme, val| {
                                                            theme.global.components_stroke_width =
                                                                val;
                                                        },
                                                        component_stroke,
                                                    );
                                                });
                                            }
                                        });

                                        // --- 2. Iconos de Archivos ---
                                        CollapsingHeader::new(
                                            RichText::new(&*i18n.t("configs_appearance.files"))
                                                .color(palette.fg_primary),
                                        )
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.folder",
                                                current_theme,
                                                file_icons.folder_default
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.image",
                                                current_theme,
                                                file_icons.image
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.pdf",
                                                current_theme,
                                                file_icons.pdf
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.doc",
                                                current_theme,
                                                file_icons.document
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.video",
                                                current_theme,
                                                file_icons.video
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.audio",
                                                current_theme,
                                                file_icons.audio
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.archive",
                                                current_theme,
                                                file_icons.archive
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.code",
                                                current_theme,
                                                file_icons.code
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.font",
                                                current_theme,
                                                file_icons.font
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.exe",
                                                current_theme,
                                                file_icons.executable
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.fallback",
                                                current_theme,
                                                file_icons.fallback
                                            );
                                        });

                                        // --- 3. Secciones (Ejemplo: Barra Lateral Izquierda) ---
                                        CollapsingHeader::new(
                                            RichText::new(
                                                &*i18n.t("configs_appearance.section_sidebar_left"),
                                            )
                                            .color(palette.fg_primary),
                                        )
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_bg",
                                                current_theme,
                                                sidebar_left.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_elevated",
                                                current_theme,
                                                sidebar_left.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.hover_bg",
                                                current_theme,
                                                sidebar_left.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_active",
                                                current_theme,
                                                sidebar_left.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_primary",
                                                current_theme,
                                                sidebar_left.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_secondary",
                                                current_theme,
                                                sidebar_left.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.accent",
                                                current_theme,
                                                sidebar_left.accent
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_border",
                                                current_theme,
                                                sidebar_left.panel_border
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.component_border",
                                                current_theme,
                                                sidebar_left.component_border
                                            );
                                        });

                                        CollapsingHeader::new(
                                            RichText::new(
                                                &*i18n.t("configs_appearance.section_top_bar"),
                                            )
                                            .color(palette.fg_primary),
                                        )
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_bg",
                                                current_theme,
                                                top_bar.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_elevated",
                                                current_theme,
                                                top_bar.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.hover_bg",
                                                current_theme,
                                                top_bar.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_active",
                                                current_theme,
                                                top_bar.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_primary",
                                                current_theme,
                                                top_bar.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_secondary",
                                                current_theme,
                                                top_bar.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.accent",
                                                current_theme,
                                                top_bar.accent
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_border",
                                                current_theme,
                                                top_bar.panel_border
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.component_border",
                                                current_theme,
                                                top_bar.component_border
                                            );
                                        });

                                        CollapsingHeader::new(
                                            RichText::new(
                                                &*i18n.t("configs_appearance.section_main_area"),
                                            )
                                            .color(palette.fg_primary),
                                        )
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_bg",
                                                current_theme,
                                                main_area.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_elevated",
                                                current_theme,
                                                main_area.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.hover_bg",
                                                current_theme,
                                                main_area.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_active",
                                                current_theme,
                                                main_area.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_primary",
                                                current_theme,
                                                main_area.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_secondary",
                                                current_theme,
                                                main_area.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.accent",
                                                current_theme,
                                                main_area.accent
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_border",
                                                current_theme,
                                                main_area.panel_border
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.component_border",
                                                current_theme,
                                                main_area.component_border
                                            );
                                        });

                                        CollapsingHeader::new(
                                            RichText::new(
                                                &*i18n
                                                    .t("configs_appearance.section_sidebar_right"),
                                            )
                                            .color(palette.fg_primary),
                                        )
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_bg",
                                                current_theme,
                                                sidebar_right.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_elevated",
                                                current_theme,
                                                sidebar_right.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.hover_bg",
                                                current_theme,
                                                sidebar_right.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_active",
                                                current_theme,
                                                sidebar_right.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_primary",
                                                current_theme,
                                                sidebar_right.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_secondary",
                                                current_theme,
                                                sidebar_right.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.accent",
                                                current_theme,
                                                sidebar_right.accent
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_border",
                                                current_theme,
                                                sidebar_right.panel_border
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.component_border",
                                                current_theme,
                                                sidebar_right.component_border
                                            );
                                        });

                                        CollapsingHeader::new(
                                            RichText::new(
                                                &*i18n.t("configs_appearance.section_context_menu"),
                                            )
                                            .color(palette.fg_primary),
                                        )
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_bg",
                                                current_theme,
                                                context_menu.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_elevated",
                                                current_theme,
                                                context_menu.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.hover_bg",
                                                current_theme,
                                                context_menu.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_active",
                                                current_theme,
                                                context_menu.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_primary",
                                                current_theme,
                                                context_menu.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_secondary",
                                                current_theme,
                                                context_menu.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.accent",
                                                current_theme,
                                                context_menu.accent
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_border",
                                                current_theme,
                                                context_menu.panel_border
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.component_border",
                                                current_theme,
                                                context_menu.component_border
                                            );
                                        });

                                        CollapsingHeader::new(
                                            RichText::new(
                                                &*i18n.t("configs_appearance.section_dialogs"),
                                            )
                                            .color(palette.fg_primary),
                                        )
                                        .show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_bg",
                                                current_theme,
                                                dialogs.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_elevated",
                                                current_theme,
                                                dialogs.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.hover_bg",
                                                current_theme,
                                                dialogs.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.bg_active",
                                                current_theme,
                                                dialogs.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_primary",
                                                current_theme,
                                                dialogs.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.icon_secondary",
                                                current_theme,
                                                dialogs.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.accent",
                                                current_theme,
                                                dialogs.accent
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.panel_border",
                                                current_theme,
                                                dialogs.panel_border
                                            );
                                            color_field!(
                                                ui,
                                                i18n,
                                                "configs_appearance.component_border",
                                                current_theme,
                                                dialogs.component_border
                                            );
                                        });
                                    });
                            });

                            strip.cell(|ui| {
                                ui.vertical(|ui| {
                                    let mut examples_state = ui.data_mut(|d| {
                                        d.get_temp::<ExamplesUiState>(Id::new(
                                            "appearance_examples_state",
                                        ))
                                        .unwrap_or(ExamplesUiState::new())
                                    });

                                    ScrollArea::vertical().show(ui, |ui| {
                                        render_examples_ui(ui, &mut examples_state);
                                    });

                                    ui.data_mut(|d| {
                                        d.insert_temp(
                                            Id::new("appearance_examples_state"),
                                            examples_state,
                                        );
                                    });
                                });
                            });
                        });
                });
            });

        ui.horizontal(|ui| {
            // Input para nombre de tema personalizado
            ui.blaze_input(
                &i18n.t("configs_appearance.theme-name"),
                None,
                &mut c_dialog.custom_theme_name,
            )
            .id("save_theme_placeholder")
            .size((200.0, 30.0))
            .radius(20)
            .text_color(palette.fg_secondary)
            .background_color(palette.bg_elevated)
            .stroke_color(Color32::TRANSPARENT)
            .x_color(palette.accent)
            .stroke_width(current_theme.global.components_stroke_width)
            .show(|| {});

            ui.add_space(8.0);

            let hover_bg = palette.bg_hover;
            let bg = palette.bg_elevated;

            if ui
                .blaze_button("")
                .id("save_btn")
                .size((30.0, 30.0))
                .background_color(bg)
                .hover_background_color(hover_bg)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("server", icons::ICON_SERVER)
                .icon_color(palette.accent)
                .show()
                .on_hover_text(&*i18n.t("configs_appearance.save-as-custom"))
                .clicked()
            {
                if !c_dialog.custom_theme_name.trim().is_empty() {
                    with_theme(|t| {
                        match t.save_as_custom_theme(c_dialog.custom_theme_name.trim()) {
                            Ok(_) => info!("Tema personalizado guardado"),
                            Err(e) => warn!("Error al guardar tema: {e}"),
                        }
                    });
                } else {
                    warn!("El nombre del tema no puede estar vacío");
                }
            }

            ui.add_space(8.0);
            // Botone de reset
            let error = current_theme.global.error();
            let c = color_ops::with_alpha(error, 120);
            let hover = color_ops::with_alpha(error, 160);
            if ui
                .blaze_button("")
                .id("reset_btn")
                .size((30.0, 30.0))
                .background_color(c)
                .hover_background_color(hover)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("reset", icons::ICON_RESTORE)
                .icon_color(error)
                .show()
                .on_hover_text(&*i18n.t("configs_appearance.reset-defaults"))
                .clicked()
            {
                with_theme(|t| match t.reset_to_default() {
                    Ok(_) => info!("Tema reseteado a valores por defecto"),
                    Err(e) => warn!("Error al resetear tema: {e}"),
                });
            }
        });

        ui.add_space(8.0);
        ui.label_ns(
            RichText::new(i18n.t("configs_appearance.message")).color(palette.fg_secondary),
        );
    });
}
