use std::{path::Path, sync::Arc};

use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Label, Margin, Rect, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, Ui, pos2, vec2,
};

use crate::{
    core::files::{blaze_motor::motor_structs::FileEntry, file_extension::FileExtension},
    ui::{
        custom_components::UiExt,
        icons_cache::{icon_cache::IconCache, icons},
        modules::utilities::{contrats_to_text_color, ensure_min_lightness, resolve_icon},
        themes::theme_manager::with_theme,
    },
    utils::formating::{format_date, format_size},
};

#[derive(Default, Clone)]
pub struct ExamplesUiState<'a> {
    pub is_initialized: bool,
    example_combobox: Vec<&'a str>,
}

impl<'a> ExamplesUiState<'a> {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            example_combobox: vec!["test1", "test2", "test3"],
        }
    }
}

/// Hace una demostración de alguno de los elementos de la UI:
/// - Paneles
/// - Botones (Variantes normales, con hover y acento)
/// - Inputs (Campos de texto con hint, icono y borrado)
/// - List items)
/// - ComboBox
/// - Indicadores semánticos (Error, Warning, Success)
pub fn render_examples_ui(ui: &mut Ui, state: &mut ExamplesUiState, icon_cache: &mut IconCache) {
    if !state.is_initialized {
        state.is_initialized = true;
    }
    let current_theme = with_theme(|t| t.current().clone());

    // Contenedor principal
    Frame::NONE
        .fill(current_theme.main_area.bg_panel())
        .stroke(Stroke {
            width: 0.5,
            color: current_theme.main_area.border(),
        })
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.add(
                Label::new(
                    RichText::new("1. Tipografía").color(current_theme.global.text_primary()),
                )
                .selectable(false),
            );

            ui.separator();

            ui.add(
                Label::new(
                    RichText::new("Primary text")
                        .heading()
                        .color(current_theme.global.text_primary()),
                )
                .selectable(false),
            );

            ui.add(
                Label::new(
                    RichText::new("Texto secundario para rutas o descripciones")
                        .small()
                        .color(current_theme.global.text_secondary()),
                )
                .selectable(false),
            );

            ui.add(
                Label::new(
                    RichText::new("Texto silenciado")
                        .small()
                        .color(current_theme.global.text_secondary()),
                )
                .selectable(false),
            );

            ui.separator();

            ui.add(
                Label::new(
                    RichText::new("2. Controles básicos")
                        .color(current_theme.global.text_primary()),
                )
                .selectable(false),
            );

            ui.horizontal(|ui| {
                ui.blaze_button("Normal", None)
                    .size((80.0, 20.0))
                    .background_color(current_theme.main_area.bg_elevated())
                    .radius(20)
                    .stroke_width(current_theme.global.components_stroke_width)
                    .show(ui);

                ui.blaze_button("Hover", None)
                    .size((80.0, 20.0))
                    .background_color(current_theme.main_area.bg_hover())
                    .radius(20)
                    .stroke_width(current_theme.global.components_stroke_width)
                    .show(ui);

                ui.blaze_button("Active", None)
                    .size((80.0, 20.0))
                    .background_color(current_theme.main_area.bg_active())
                    .radius(20)
                    .stroke_width(current_theme.global.components_stroke_width)
                    .show(ui);
            });

            ui.blaze_combobox(Some(icon_cache))
                .selected_text("Test")
                .text_color(current_theme.global.text_primary())
                .size((150.0, 30.0))
                .background_color(current_theme.main_area.bg_elevated())
                .show(ui, |ui| {
                    let w = ui.available_width();

                    for (i, name) in state.example_combobox.iter().enumerate() {
                        let label =
                            RichText::new(&**name).color(current_theme.global.text_primary());

                        let bg = if "test1" == *name {
                            current_theme.main_area.accent()
                        } else {
                            current_theme.main_area.bg_elevated()
                        };

                        ui.blaze_button(label, None)
                            .id(format!("example_btn_{i}"))
                            .size((w, 20.0))
                            .background_color(bg)
                            .hover_background_color(current_theme.main_area.bg_hover())
                            .stroke_color(Color32::TRANSPARENT)
                            .radius(20)
                            .stroke_width(current_theme.global.components_stroke_width)
                            .show(ui);
                    }
                });

            ui.separator();

            ui.add(
                Label::new(RichText::new("3. Inputs").color(current_theme.global.text_primary()))
                    .selectable(false),
            );

            let c = current_theme.main_area.accent();
            let hint_stroke_color = Color32::from_rgba_unmultiplied(
                c.r().saturating_add(20),
                c.g().saturating_add(20),
                c.b().saturating_add(20),
                120,
            );

            let hint_color = Color32::from_rgba_unmultiplied(
                c.r().saturating_add(60),
                c.g().saturating_add(60),
                c.b().saturating_add(60),
                160,
            );

            let (icon_name, icon_bytes) = ("search", icons::ICON_SEARCH);
            let mut test = String::new();

            ui.blaze_input("test iddle", Some("hint"), &mut test, Some(icon_cache))
                .id("test_input_1")
                .hint_background_color(current_theme.main_area.bg_elevated())
                .hint_stroke_color(hint_stroke_color)
                .hint_text_color(hint_color)
                .size((200.0, 30.0))
                .radius(20)
                .text_color(current_theme.global.text_secondary())
                .background_color(current_theme.main_area.bg_elevated())
                .stroke_color(ensure_min_lightness(current_theme.main_area.border()))
                .x_color(current_theme.main_area.accent())
                .icon(icon_name, icon_bytes)
                .icon_color(ensure_min_lightness(current_theme.main_area.border()))
                .stroke_width(current_theme.global.components_stroke_width)
                .show(ui, || {});

            ui.blaze_input("test focused", Some("hint"), &mut test, Some(icon_cache))
                .id("test_input_2")
                .hint_background_color(current_theme.main_area.bg_elevated())
                .hint_stroke_color(hint_stroke_color)
                .hint_text_color(hint_color)
                .size((200.0, 30.0))
                .radius(20)
                .text_color(current_theme.global.text_secondary())
                .background_color(current_theme.main_area.bg_elevated())
                .stroke_color(ensure_min_lightness(current_theme.main_area.accent()))
                .x_color(current_theme.main_area.accent())
                .icon(icon_name, icon_bytes)
                .icon_color(ensure_min_lightness(current_theme.main_area.accent()))
                .stroke_width(current_theme.global.components_stroke_width)
                .show(ui, || {});

            ui.blaze_input("test no icon", None, &mut test, Some(icon_cache))
                .id("test_input_3")
                .size((200.0, 30.0))
                .radius(20)
                .text_color(current_theme.global.text_secondary())
                .background_color(current_theme.main_area.bg_elevated())
                .stroke_color(Color32::TRANSPARENT)
                .x_color(current_theme.main_area.accent())
                .stroke_width(current_theme.global.components_stroke_width)
                .show(ui, || {});

            ui.separator();

            ui.add(
                Label::new(
                    RichText::new("4. Estados semánticos")
                        .color(current_theme.global.text_primary()),
                )
                .selectable(false),
            );

            let state_size = vec2(80.0, 30.0);
            ui.horizontal(|ui| {
                let (rect1, _) = ui.allocate_exact_size(state_size, Sense::hover());
                ui.painter()
                    .rect_filled(rect1, 20, current_theme.global.error());

                let text_color = contrats_to_text_color(current_theme.global.error());
                ui.painter().text(
                    rect1.center(),
                    Align2::CENTER_CENTER,
                    "Error",
                    FontId::proportional(12.0),
                    text_color,
                );

                let (rect2, _) = ui.allocate_exact_size(state_size, Sense::hover());
                ui.painter()
                    .rect_filled(rect2, 20, current_theme.global.warn());

                let text_color = contrats_to_text_color(current_theme.global.warn());
                ui.painter().text(
                    rect2.center(),
                    Align2::CENTER_CENTER,
                    "Warn",
                    FontId::proportional(12.0),
                    text_color,
                );

                let (rect3, _) = ui.allocate_exact_size(state_size, Sense::hover());
                ui.painter()
                    .rect_filled(rect3, 20, current_theme.global.success());

                let text_color = contrats_to_text_color(current_theme.global.success());
                ui.painter().text(
                    rect3.center(),
                    Align2::CENTER_CENTER,
                    "Success",
                    FontId::proportional(12.0),
                    text_color,
                );
            });

            ui.separator();

            ui.add(
                Label::new(RichText::new("5. Archivos").color(current_theme.global.text_primary()))
                    .selectable(false),
            );

            ui.vertical(|ui| {
                ui.set_height(300.0);
                let available = ui.available_width();
                let date_w = 90.0_f32;
                let size_w = 60.0_f32;
                let name_w = (available - date_w - size_w - 20.0).max(60.0);

                let scroll_area = ScrollArea::vertical()
                    .scroll_source(
                        egui::scroll_area::ScrollSource::MOUSE_WHEEL
                            | egui::scroll_area::ScrollSource::SCROLL_BAR,
                    )
                    .auto_shrink([false, false])
                    .id_salt("scroll_example");

                let row_height = 28.0;
                let files = [
                    Arc::new(FileEntry::placeholder_dir("dir", Path::new(""))),
                    Arc::new(FileEntry::placeholder_file(
                        "img",
                        Path::new(""),
                        FileExtension::Image(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "pdf",
                        Path::new(""),
                        FileExtension::Document(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "doc",
                        Path::new(""),
                        FileExtension::Document(crate::core::files::file_extension::DocType::Doc),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "vid",
                        Path::new(""),
                        FileExtension::Video(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "aud",
                        Path::new(""),
                        FileExtension::Audio(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "arch",
                        Path::new(""),
                        FileExtension::Archive(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "code",
                        Path::new(""),
                        FileExtension::Code(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "font",
                        Path::new(""),
                        FileExtension::Font(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "exe",
                        Path::new(""),
                        FileExtension::Executable(Default::default()),
                    )),
                    Arc::new(FileEntry::placeholder_file(
                        "file",
                        Path::new(""),
                        FileExtension::Unknown,
                    )),
                ];
                scroll_area.show_rows(ui, row_height, files.len(), |ui: &mut Ui, row_range| {
                    for i in row_range.clone() {
                        let file = &files[i];

                        let (rect, response) =
                            ui.allocate_exact_size(vec2(available, row_height), Sense::hover());

                        // Hover: única interacción conservada.
                        if response.hovered() {
                            ui.painter()
                                .rect_filled(rect, 5.0, current_theme.main_area.bg_hover());
                        }

                        let text_color = current_theme.global.text_primary();

                        // --- Icono ---
                        let icon_size = 16.0;
                        let left_padding = icon_size / 6.0;

                        let icon_rect = Rect::from_min_size(
                            pos2(rect.min.x + left_padding, rect.center().y - icon_size / 2.0),
                            vec2(icon_size, icon_size),
                        );

                        let (icon_name, icon_bytes, color) = resolve_icon(file, None);

                        let icon = icon_cache.get_or_load(
                            ui,
                            &icon_name,
                            icon_bytes,
                            ensure_min_lightness(color),
                            vec2(icon_size, icon_size),
                        );

                        ui.painter().image(
                            icon.id(),
                            icon_rect,
                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );

                        // --- Nombre ---

                        let name_font_size = (icon_size / 2.0 * 1.2).clamp(13.0, 25.0);

                        let name_start_x = icon_rect.max.x + 8.0;
                        let name_end_x = rect.min.x + name_w;

                        let name_rect = Rect::from_min_max(
                            pos2(name_start_x, rect.min.y),
                            pos2(name_end_x, rect.max.y),
                        );

                        let name_galley = ui.fonts_mut(|f| {
                            f.layout_no_wrap(
                                file.name.to_string(),
                                FontId::proportional(name_font_size),
                                text_color,
                            )
                        });

                        ui.painter().with_clip_rect(name_rect).galley(
                            pos2(name_start_x, rect.center().y - name_galley.size().y / 2.0),
                            name_galley,
                            text_color,
                        );

                        // --- Fecha ---

                        let date_rect = Rect::from_min_max(
                            pos2(rect.min.x + name_w, rect.min.y),
                            pos2(rect.min.x + name_w + date_w, rect.max.y),
                        );

                        let date_font_size = (icon_size / 2.0 * 1.2).clamp(12.0, 25.0);

                        let date_galley = ui.fonts_mut(|f| {
                            f.layout_no_wrap(
                                format_date(file.modified).to_string(),
                                FontId::proportional(date_font_size),
                                text_color,
                            )
                        });

                        ui.painter().with_clip_rect(date_rect).galley(
                            pos2(
                                date_rect.min.x + 4.0,
                                rect.center().y - date_galley.size().y / 2.0,
                            ),
                            date_galley,
                            text_color,
                        );

                        // --- Peso aleatorio de ejemplo ---

                        let size_rect = Rect::from_min_max(
                            pos2(rect.min.x + name_w + date_w, rect.min.y),
                            rect.max,
                        );

                        let size_text = format_size(file.size);

                        let size_galley = ui.fonts_mut(|f| {
                            f.layout_no_wrap(
                                size_text,
                                FontId::proportional(date_font_size),
                                text_color,
                            )
                        });

                        ui.painter().with_clip_rect(size_rect).galley(
                            pos2(
                                size_rect.min.x + 4.0,
                                rect.center().y - size_galley.size().y / 2.0,
                            ),
                            size_galley,
                            text_color,
                        );
                    }
                });
            });

            ui.separator();

            ui.add(
                Label::new(RichText::new("6. Paneles").color(current_theme.global.text_primary()))
                    .selectable(false),
            );

            let state_size = vec2(100.0, 100.0);
            ui.horizontal(|ui| {
                let (rect1, _) = ui.allocate_exact_size(state_size, Sense::hover());
                ui.painter()
                    .rect_filled(rect1, 20, current_theme.global.bg_main());
                let text_color = contrats_to_text_color(current_theme.global.bg_main());
                ui.painter().text(
                    rect1.center(),
                    Align2::CENTER_CENTER,
                    "Main BG",
                    FontId::proportional(12.0),
                    text_color,
                );

                let (rect2, _) = ui.allocate_exact_size(state_size, Sense::hover());
                ui.painter()
                    .rect_filled(rect2, 20, current_theme.main_area.bg_elevated());
                let text_color = contrats_to_text_color(current_theme.main_area.bg_elevated());
                ui.painter().text(
                    rect2.center(),
                    Align2::CENTER_CENTER,
                    "BG Container",
                    FontId::proportional(12.0),
                    text_color,
                );

                let (rect3, _) = ui.allocate_exact_size(state_size, Sense::hover());
                ui.painter().rect(
                    rect3,
                    20,
                    current_theme.main_area.bg_panel(),
                    Stroke::new(
                        current_theme.global.panel_stroke_width,
                        current_theme.main_area.border(),
                    ),
                    StrokeKind::Middle,
                );
                let text_color = contrats_to_text_color(current_theme.main_area.bg_panel());
                ui.painter().text(
                    rect3.center(),
                    Align2::CENTER_CENTER,
                    "Panel y borde",
                    FontId::proportional(12.0),
                    text_color,
                );
            });
        });
}
