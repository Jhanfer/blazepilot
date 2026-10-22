use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::dialogs::config_dialog::configs_dialog::ConfigDialog;
use crate::ui::examples::examples_components::{ExamplesUiState, render_examples_ui};
use crate::ui::icons_cache::icon_cache::IconCache;
use crate::ui::icons_cache::icons;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use egui::scroll_area::ScrollSource;
use egui::{Color32, Id, Label, RichText, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};
use tracing::{info, warn};

macro_rules! color_field {
    ($ui:expr_2021, $current_theme:expr_2021, $label:expr_2021, $($path:tt)*) => {{
        $ui.horizontal(|ui| {
            let current_hex = &$current_theme.$($path)*;
            let mut color = Color32::from_hex(current_hex)
                .unwrap_or(Color32::DEBUG_COLOR);

            ui.label_ns(
                RichText::new($label)
                    .color($current_theme.semantic.text_primary.to_color()),
            );

            if ui.color_picker(
                &mut color,
                &*$label,
            )
            .show(ui)
            .changed() {
                with_theme(|t| {
                    t.update_theme(
                        |theme, c| {
                            let [r, g, b, a] = c.to_srgba_unmultiplied();
                            theme.$($path)* = format!(
                                "#{:02X}{:02X}{:02X}{:02X}",
                                r, g, b, a
                            );
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

    let is_dark_theme = current_theme.semantic.text_primary.to_color().r() > 128;

    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.set_height(ui.available_height());
        // Titulo
        ui.add(
            Label::new(
                RichText::new(i18n.t("configs_appearance.title"))
                    .strong()
                    .color(current_theme.semantic.text_primary.to_color()),
            )
            .selectable(false),
        );
        ui.add_space(8.0);

        let cbg = current_theme.components.button.bg_hover.to_color();
        let hover_bg = if is_dark_theme {
            Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 80)
        } else {
            Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 10)
        };

        // i18n.t("configs_appearance.selection")
        ui.horizontal(|ui| {
            ui.blaze_combobox(Some(icon_cache))
                .selected_text(&*current_theme.name)
                .text_color(current_theme.semantic.text_primary.to_color())
                .size((150.0, 30.0))
                .background_color(current_theme.components.button.bg.to_color())
                .hover_background_color(hover_bg)
                .show(ui, |ui| {
                    let w = ui.available_width();

                    for (i, theme) in available_themes.iter().enumerate() {
                        let label = RichText::new(&**theme)
                            .color(current_theme.semantic.text_primary.to_color());

                        let bg = if current_theme.name == *theme {
                            current_theme.semantic.accent_glow.to_color()
                        } else {
                            current_theme.components.button.bg.to_color()
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
                .background_color(current_theme.components.button.bg.to_color())
                .hover_background_color(hover_bg)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("reset", icons::ICON_RESTORE)
                .icon_color(current_theme.semantic.accent_glow.to_color())
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
                .color(current_theme.semantic.text_primary.to_color()),
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
                                        // --- 1. Fondos ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.background"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.main_bg"),
                                            semantic.bg_main
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.container_bg"),
                                            semantic.bg_container
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.separator"),
                                            semantic.separator
                                        );
                                        ui.add_space(8.0);

                                        // --- 2. Textos ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.texts"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.primary_text"),
                                            semantic.text_primary
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.secondary_text"),
                                            semantic.text_secondary
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.muted_text"),
                                            semantic.text_muted
                                        );
                                        ui.add_space(8.0);

                                        // --- 3. Estados Semánticos ---
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.error"),
                                            semantic.error
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.success"),
                                            semantic.success
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.warn"),
                                            semantic.warn
                                        );
                                        ui.add_space(8.0);

                                        // --- 4. Acentos ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.accent"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.accent"),
                                            semantic.accent
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.accent_glow"),
                                            semantic.accent_glow
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.rubberband"),
                                            semantic.rubberband
                                        );
                                        ui.add_space(8.0);

                                        // --- 5. Paneles ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.panels"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.panel_bg"),
                                            components.panel.bg
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.panel_border"),
                                            components.panel.border
                                        );
                                        ui.add_space(8.0);

                                        // --- 6. Botones ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.buttons"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.button_bg"),
                                            components.button.bg
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.button_hover"),
                                            components.button.bg_hover
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.button_label_hover"),
                                            components.button.label_hover
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.button_label_active"),
                                            components.button.label_active
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.button_label_inactive"),
                                            components.button.label_inactive
                                        );
                                        ui.add_space(8.0);

                                        // --- 7. Barra de búsqueda (Input) ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.input"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.input_bg"),
                                            components.input.bg
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.input_border_idle"),
                                            components.input.border_idle
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.input_border_focus"),
                                            components.input.border_focus
                                        );
                                        ui.add_space(8.0);

                                        // --- 8. Sidebar ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.sidebar"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.sidebar_hover"),
                                            components.sidebar_item.bg_hover
                                        );
                                        ui.add_space(8.0);

                                        // --- 9. Lista de archivos ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.file_list"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.list_hover"),
                                            components.list_item.bg_hover
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.list_selected"),
                                            components.list_item.bg_selected
                                        );
                                        ui.add_space(8.0);

                                        // --- 10. Tools View ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.tools"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.tools_bg"),
                                            components.tools.bg
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.tools_label_hover"),
                                            components.tools.label_hover
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.tools_label_active"),
                                            components.tools.label_active
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.tools_label_inactive"),
                                            components.tools.label_inactive
                                        );
                                        ui.add_space(8.0);

                                        // --- 11. Iconos de archivo ---
                                        ui.label_ns(
                                            RichText::new(i18n.t("configs_appearance.files"))
                                                .strong()
                                                .color(
                                                    current_theme
                                                        .semantic
                                                        .text_secondary
                                                        .to_color(),
                                                ),
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.folder"),
                                            file_theme.folder_default
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.image"),
                                            file_theme.image
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.pdf"),
                                            file_theme.pdf
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.doc"),
                                            file_theme.document
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.video"),
                                            file_theme.video
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.audio"),
                                            file_theme.audio
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.archive"),
                                            file_theme.archive
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.code"),
                                            file_theme.code
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.font"),
                                            file_theme.font
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.exe"),
                                            file_theme.executable
                                        );
                                        color_field!(
                                            ui,
                                            current_theme,
                                            i18n.t("configs_appearance.fallback"),
                                            file_theme.fallback
                                        );
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
            ui.blaze_input("save", None, &mut c_dialog.custom_theme_name, icon_cache)
                .id("save_theme_placeholder")
                .size((200.0, 30.0))
                .radius(20)
                .text_color(current_theme.semantic.text_secondary.to_color())
                .background_color(current_theme.semantic.bg_container.to_color())
                .stroke_color(Color32::TRANSPARENT)
                .x_color(current_theme.semantic.accent_glow.to_color())
                .show(ui, || {});

            ui.add_space(8.0);

            let cbg = current_theme.components.button.bg_hover.to_color();
            let bg = current_theme.components.button.bg.to_color();
            let hover_bg = if is_dark_theme {
                Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 80)
            } else {
                Color32::from_rgba_unmultiplied(cbg.r(), cbg.g(), cbg.b(), 10)
            };

            if ui
                .blaze_button("", Some(icon_cache))
                .id("save_btn")
                .size((30.0, 30.0))
                .background_color(bg)
                .hover_background_color(hover_bg)
                .stroke_color(Color32::TRANSPARENT)
                .radius(20)
                .icon("server", icons::ICON_SERVER)
                .icon_color(current_theme.semantic.accent_glow.to_color())
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
                .color(current_theme.semantic.text_secondary.to_color()),
        );
    });
}

// // Layout horizontal a dos columnas: Editor (Izq) | Preview (Der)
// ui.horizontal(|ui| {
//     ui.set_width(ui.available_width());
//     ui.set_height(content_h);

//     let avail_w = ui.available_width();
//     let spacing = 16.0;
//     // La columna izquierda toma 280px para los campos y color pickers;
//     // la derecha toma el resto disponible holgadamente.
//     let left_w = (avail_w * 0.20).clamp(240.0, 300.0);
//     let right_w = (avail_w - left_w - spacing).max(230.0);

//     tracing::debug!("content_h: {content_h}");
//     tracing::debug!("left_w: {left_w}");
//     tracing::debug!("right_w: {right_w}");

//     // 1. Columna izquierda: Editor de colores con scroll
//     ui.vertical(|ui| {
//         ui.set_width(left_w);
//         ui.set_height(content_h);

//     // Separador vertical sutil entre ambas secciones
//     ui.separator();

//     // 2. Columna derecha: Vista previa interactiva

// });
