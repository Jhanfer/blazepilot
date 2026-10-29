use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::dialogs::config_dialog::configs_dialog::ConfigDialog;
use crate::ui::examples::examples_components::{ExamplesUiState, render_examples_ui};
use crate::ui::icons_cache::icon_cache::IconCache;
use crate::ui::icons_cache::icons;
use crate::ui::themes::theme_manager::with_theme;
use egui::scroll_area::ScrollSource;
use egui::{CollapsingHeader, Color32, Id, Label, RichText, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};
use tracing::{info, warn};

macro_rules! color_field {
    ($ui:expr, $label:expr, $c_theme:expr, $($path:tt)*) => {{
        $ui.horizontal(|ui| {
            let current_hex = &$c_theme.$($path)*;
            let mut color = Color32::from_hex(current_hex)
                .unwrap_or(Color32::DEBUG_COLOR);

            ui.label_ns(
                RichText::new($label)
                    .color($c_theme.global.text_primary()),
            );

            if ui.color_picker(&mut color, ($label, stringify!($($path)*)))
                .background_color($c_theme.main_area.bg_panel())
                .show(ui)
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

pub(crate) fn render_appearance_settings(
    c_dialog: &mut ConfigDialog,
    ui: &mut Ui,
    _query: &str,
    icon_cache: &mut IconCache,
) {
    let i18n = with_configs(|c| c.get_i18n());
    let (available_themes, current_theme) =
        with_theme(|t| (t.get_available_themes_names(), t.current().clone()));

    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.set_height(ui.available_height());
        // Titulo
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_appearance.title"))
                    .strong()
                    .color(current_theme.global.text_primary()),
            )
            .selectable(false),
        );
        ui.add_space(8.0);

        let hover_bg = current_theme.sidebar_left.bg_hover();

        // i18n.t("configs_appearance.selection")
        ui.horizontal(|ui| {
            ui.blaze_combobox(Some(icon_cache))
                .selected_text(&*current_theme.meta.name)
                .text_color(current_theme.global.text_primary())
                .size((150.0, 30.0))
                .background_color(current_theme.sidebar_left.bg_elevated())
                .hover_background_color(current_theme.sidebar_left.bg_hover())
                .show(ui, |ui| {
                    let w = ui.available_width();

                    for (i, theme) in available_themes.iter().enumerate() {
                        let is_selected = current_theme.meta.name == *theme;
                        let text_color = if is_selected {
                            current_theme.global.text_primary()
                        } else {
                            current_theme.global.text_secondary()
                        };
                        let label = RichText::new(&**theme).color(text_color);

                        let bg = if is_selected {
                            current_theme.sidebar_left.bg_active()
                        } else {
                            Color32::TRANSPARENT
                        };

                        if ui
                            .blaze_button(label, None)
                            .id(format!("theme_btn_{i}"))
                            .size((w, 20.0))
                            .background_color(bg)
                            .hover_background_color(hover_bg)
                            .stroke_color(Color32::TRANSPARENT)
                            .radius(20)
                            .show(ui)
                            .clicked()
                        {
                            with_configs(|c| c.set_current_theme_name(theme));
                            with_theme(|t| t.set_theme(theme));
                        }
                    }
                });

            // Recargar
            if ui
                .blaze_button("", Some(icon_cache))
                .id("reload_theme")
                .size((30.0, 30.0))
                .background_color(current_theme.sidebar_left.bg_elevated())
                .hover_background_color(hover_bg)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("reset", icons::ICON_RESTORE)
                .icon_color(current_theme.sidebar_left.accent())
                .stroke_width(current_theme.global.components_stroke_width)
                .show(ui)
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
                .color(current_theme.global.text_primary()),
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
                                        CollapsingHeader::new("Globales")
                                            .default_open(true) // Mantenemos el primero abierto por defecto
                                            .show(ui, |ui| {
                                                color_field!(
                                                    ui,
                                                    "Fondo Principal",
                                                    current_theme,
                                                    global.bg_main
                                                );
                                                color_field!(
                                                    ui,
                                                    "Efecto Rebote",
                                                    current_theme,
                                                    global.rubberband
                                                );
                                                color_field!(
                                                    ui,
                                                    "Texto Principal",
                                                    current_theme,
                                                    global.text_primary
                                                );
                                                color_field!(
                                                    ui,
                                                    "Texto Secundario",
                                                    current_theme,
                                                    global.text_secondary
                                                );
                                                color_field!(
                                                    ui,
                                                    "Error",
                                                    current_theme,
                                                    global.error
                                                );
                                                color_field!(
                                                    ui,
                                                    "Advertencia",
                                                    current_theme,
                                                    global.warn
                                                );
                                                color_field!(
                                                    ui,
                                                    "Éxito",
                                                    current_theme,
                                                    global.success
                                                );

                                                ui.label_ns(
                                                    RichText::new("Stroke Width")
                                                        .color(current_theme.global.text_primary()),
                                                );

                                                let mut panel_stroke =
                                                    current_theme.global.panel_stroke_width;

                                                if ui
                                                    .blaze_slider(&mut panel_stroke, 0.0..=1.0)
                                                    .size((80.0, 20.0))
                                                    .background_color(
                                                        current_theme.main_area.bg_panel(),
                                                    )
                                                    .accent(current_theme.main_area.accent())
                                                    .show(ui)
                                                    .changed()
                                                {
                                                    with_theme(|t| {
                                                        t.update_theme(
                                                            |theme, val| {
                                                                theme.global.panel_stroke_width =
                                                                    val;
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
                                                    .background_color(
                                                        current_theme.main_area.bg_panel(),
                                                    )
                                                    .accent(current_theme.main_area.accent())
                                                    .show(ui)
                                                    .changed()
                                                {
                                                    with_theme(|t| {
                                                        t.update_theme(
                                                            |theme, val| {
                                                                theme
                                                                    .global
                                                                    .components_stroke_width = val;
                                                            },
                                                            component_stroke,
                                                        );
                                                    });
                                                }
                                            });

                                        // --- 2. Iconos de Archivos ---
                                        CollapsingHeader::new("Iconos de Archivos").show(
                                            ui,
                                            |ui| {
                                                color_field!(
                                                    ui,
                                                    "Carpeta (Por defecto)",
                                                    current_theme,
                                                    file_icons.folder_default
                                                );
                                                color_field!(
                                                    ui,
                                                    "Imagen",
                                                    current_theme,
                                                    file_icons.image
                                                );
                                                color_field!(
                                                    ui,
                                                    "PDF",
                                                    current_theme,
                                                    file_icons.pdf
                                                );
                                                color_field!(
                                                    ui,
                                                    "Documento",
                                                    current_theme,
                                                    file_icons.document
                                                );
                                                color_field!(
                                                    ui,
                                                    "Vídeo",
                                                    current_theme,
                                                    file_icons.video
                                                );
                                                color_field!(
                                                    ui,
                                                    "Audio",
                                                    current_theme,
                                                    file_icons.audio
                                                );
                                                color_field!(
                                                    ui,
                                                    "Archivo (Zip/Rar)",
                                                    current_theme,
                                                    file_icons.archive
                                                );
                                                color_field!(
                                                    ui,
                                                    "Código",
                                                    current_theme,
                                                    file_icons.code
                                                );
                                                color_field!(
                                                    ui,
                                                    "Fuente",
                                                    current_theme,
                                                    file_icons.font
                                                );
                                                color_field!(
                                                    ui,
                                                    "Ejecutable",
                                                    current_theme,
                                                    file_icons.executable
                                                );
                                                color_field!(
                                                    ui,
                                                    "Desconocido (Fallback)",
                                                    current_theme,
                                                    file_icons.fallback
                                                );
                                            },
                                        );

                                        // --- 3. Secciones (Ejemplo: Barra Lateral Izquierda) ---
                                        CollapsingHeader::new("Barra lateral izquierda").show(
                                            ui,
                                            |ui| {
                                                color_field!(
                                                    ui,
                                                    "Fondo del Panel",
                                                    current_theme,
                                                    sidebar_left.bg_panel
                                                );
                                                color_field!(
                                                    ui,
                                                    "Fondo Elevado",
                                                    current_theme,
                                                    sidebar_left.bg_elevated
                                                );
                                                color_field!(
                                                    ui,
                                                    "Fondo Hover",
                                                    current_theme,
                                                    sidebar_left.bg_hover
                                                );
                                                color_field!(
                                                    ui,
                                                    "Fondo Activo",
                                                    current_theme,
                                                    sidebar_left.bg_active
                                                );
                                                color_field!(
                                                    ui,
                                                    "Icono Principal",
                                                    current_theme,
                                                    sidebar_left.icon_primary
                                                );
                                                color_field!(
                                                    ui,
                                                    "Icono Secundario",
                                                    current_theme,
                                                    sidebar_left.icon_secondary
                                                );
                                                color_field!(
                                                    ui,
                                                    "Acento",
                                                    current_theme,
                                                    sidebar_left.accent
                                                );
                                                color_field!(
                                                    ui,
                                                    "Borde",
                                                    current_theme,
                                                    sidebar_left.border
                                                );
                                            },
                                        );

                                        CollapsingHeader::new("Panel central").show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                "Fondo del Panel",
                                                current_theme,
                                                main_area.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                "Fondo Elevado",
                                                current_theme,
                                                main_area.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                "Fondo Hover",
                                                current_theme,
                                                main_area.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                "Fondo Activo",
                                                current_theme,
                                                main_area.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                "Icono Principal",
                                                current_theme,
                                                main_area.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                "Icono Secundario",
                                                current_theme,
                                                main_area.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                "Acento",
                                                current_theme,
                                                main_area.accent
                                            );
                                            color_field!(
                                                ui,
                                                "Borde",
                                                current_theme,
                                                main_area.border
                                            );
                                        });

                                        CollapsingHeader::new("Barra lateral derecha").show(
                                            ui,
                                            |ui| {
                                                color_field!(
                                                    ui,
                                                    "Fondo del Panel",
                                                    current_theme,
                                                    sidebar_right.bg_panel
                                                );
                                                color_field!(
                                                    ui,
                                                    "Fondo Elevado",
                                                    current_theme,
                                                    sidebar_right.bg_elevated
                                                );
                                                color_field!(
                                                    ui,
                                                    "Fondo Hover",
                                                    current_theme,
                                                    sidebar_right.bg_hover
                                                );
                                                color_field!(
                                                    ui,
                                                    "Fondo Activo",
                                                    current_theme,
                                                    sidebar_right.bg_active
                                                );
                                                color_field!(
                                                    ui,
                                                    "Icono Principal",
                                                    current_theme,
                                                    sidebar_right.icon_primary
                                                );
                                                color_field!(
                                                    ui,
                                                    "Icono Secundario",
                                                    current_theme,
                                                    sidebar_right.icon_secondary
                                                );
                                                color_field!(
                                                    ui,
                                                    "Acento",
                                                    current_theme,
                                                    sidebar_right.accent
                                                );
                                                color_field!(
                                                    ui,
                                                    "Borde",
                                                    current_theme,
                                                    sidebar_right.border
                                                );
                                            },
                                        );

                                        CollapsingHeader::new("Menu contextual").show(ui, |ui| {
                                            color_field!(
                                                ui,
                                                "Fondo del Panel",
                                                current_theme,
                                                context_menu.bg_panel
                                            );
                                            color_field!(
                                                ui,
                                                "Fondo Elevado",
                                                current_theme,
                                                context_menu.bg_elevated
                                            );
                                            color_field!(
                                                ui,
                                                "Fondo Hover",
                                                current_theme,
                                                context_menu.bg_hover
                                            );
                                            color_field!(
                                                ui,
                                                "Fondo Activo",
                                                current_theme,
                                                context_menu.bg_active
                                            );
                                            color_field!(
                                                ui,
                                                "Icono Principal",
                                                current_theme,
                                                context_menu.icon_primary
                                            );
                                            color_field!(
                                                ui,
                                                "Icono Secundario",
                                                current_theme,
                                                context_menu.icon_secondary
                                            );
                                            color_field!(
                                                ui,
                                                "Acento",
                                                current_theme,
                                                context_menu.accent
                                            );
                                            color_field!(
                                                ui,
                                                "Borde",
                                                current_theme,
                                                context_menu.border
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
                                        render_examples_ui(ui, &mut examples_state, icon_cache);
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
                "save",
                None,
                &mut c_dialog.custom_theme_name,
                Some(icon_cache),
            )
            .id("save_theme_placeholder")
            .size((200.0, 30.0))
            .radius(20)
            .text_color(current_theme.global.text_secondary())
            .background_color(current_theme.sidebar_left.bg_elevated())
            .stroke_color(Color32::TRANSPARENT)
            .x_color(current_theme.sidebar_left.accent())
            .stroke_width(current_theme.global.components_stroke_width)
            .show(ui, || {});

            ui.add_space(8.0);

            let hover_bg = current_theme.sidebar_left.bg_hover();
            let bg = current_theme.sidebar_left.bg_elevated();

            if ui
                .blaze_button("", Some(icon_cache))
                .id("save_btn")
                .size((30.0, 30.0))
                .background_color(bg)
                .hover_background_color(hover_bg)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("server", icons::ICON_SERVER)
                .icon_color(current_theme.sidebar_left.accent())
                .show(ui)
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
            let c = Color32::from_rgba_unmultiplied(255, 0, 0, 120);
            let hover = Color32::from_rgba_unmultiplied(255, 0, 0, 160);
            if ui
                .blaze_button("", Some(icon_cache))
                .id("reset_btn")
                .size((30.0, 30.0))
                .background_color(c)
                .hover_background_color(hover)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("reset", icons::ICON_RESTORE)
                .icon_color(Color32::RED)
                .show(ui)
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
            RichText::new(i18n.t("configs_appearance.message"))
                .color(current_theme.global.text_secondary()),
        );
    });
}
// eliminar fondo activo
