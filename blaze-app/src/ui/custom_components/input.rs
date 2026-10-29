#![allow(unused)]
// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Id, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2, pos2, vec2,
};

use crate::ui::{
    custom_components::text_edit::BlazeTextEdit,
    icons_cache::{icon_cache::IconCache, icons},
    modules::utilities::ensure_min_lightness,
};

/// Entrada de texto con soporte para icono, etiqueta de sugerencias y botón
/// de limpieza
pub(crate) struct BlazeInput<'a> {
    id: Id,
    input: &'a mut String,
    input_hint_text: &'a str,
    hint_text: Option<&'a str>,
    hint_text_color: Option<Color32>,
    hint_background_color: Option<Color32>,
    hint_stroke_color: Option<Color32>,
    stroke_width: f32,
    width: f32,
    height: f32,
    corner_radius: CornerRadius,
    text_color: Color32,
    x_color: Color32,
    background_color: Color32,
    hover_background_color: Option<Color32>,
    stroke_color: Color32,
    hover_stroke_color: Option<Color32>,
    icon: Option<(&'a str, &'a [u8])>,
    icon_color: Color32,
    hover_icon_color: Option<Color32>,
    icon_size: f32,
    icon_cache: Option<&'a mut IconCache>,
}

impl<'a> BlazeInput<'a> {
    /// Inicializa la estructura [`BlazeInput`] con los parámetros base y valores por defecto
    ///
    /// # Parámetros:
    /// - `input`: referencia mutable a la cadena de tecto editada por el usuario
    /// - `input_hint_text`: texto de sugerencia de hot_keys
    /// - `hint_text`: Texto explicativo que se renderiza dentro del input cuando está vacío
    /// - `icon_cache`: Pool de caché opcional para registrar o recuperar texturas de iconos y evitar rasterizar imágenes repetidas
    pub(crate) fn new<T: Into<&'a mut String>>(
        ui: &mut Ui,
        input: T,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        icon_cache: Option<&'a mut IconCache>,
    ) -> Self {
        Self {
            id: ui.id().with("input"),
            input: input.into(),
            input_hint_text,
            hint_text,
            hint_text_color: None,
            hint_background_color: None,
            hint_stroke_color: None,
            stroke_width: 0.8,
            width: 100.0,
            height: 30.0,
            corner_radius: CornerRadius::same(5),
            background_color: Color32::GRAY,
            hover_background_color: None,
            stroke_color: Color32::GRAY,
            hover_stroke_color: None,
            icon_color: Color32::WHITE,
            text_color: Color32::WHITE,
            x_color: Color32::LIGHT_GRAY,
            icon: None,
            hover_icon_color: None,
            icon_size: 16_f32,
            icon_cache,
        }
    }

    /// Establece el identificador del botón
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = id.into();
        self
    }

    /// Establece el tamaño del botón
    pub fn size(mut self, size: impl Into<(f32, f32)>) -> Self {
        let (w, h) = size.into();
        self.width = w;
        self.height = h;
        self
    }

    /// Establece el ancho de la línea del borde
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Establece el tamaño del icono
    pub fn icon_size(mut self, size: impl Into<f32>) -> Self {
        self.icon_size = size.into();
        self
    }

    /// Establece el color del icono
    pub fn icon_color(mut self, color: impl Into<Color32>) -> Self {
        self.icon_color = color.into();
        self
    }

    /// Establece el color del icono al pasar el cursor
    pub fn hover_icon_color(mut self, color: impl Into<Color32>) -> Self {
        self.hover_icon_color = Some(color.into());
        self
    }

    /// Establece el color del fonfo del botón
    pub fn background_color(mut self, color: impl Into<Color32>) -> Self {
        self.background_color = color.into();
        self
    }

    /// Establece el color del fondo del bóton al pasar el cursor
    pub fn hover_background_color(mut self, color: impl Into<Color32>) -> Self {
        self.hover_background_color = Some(color.into());
        self
    }

    /// Establece el color del texto
    pub fn text_color(mut self, color: impl Into<Color32>) -> Self {
        self.text_color = color.into();
        self
    }

    /// Establece el color del contorno del botón
    pub fn stroke_color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke_color = color.into();
        self
    }

    /// Establece el color del contorno del botón al pasar el cursor
    pub fn hover_stroke_color(mut self, color: impl Into<Color32>) -> Self {
        self.hover_stroke_color = Some(color.into());
        self
    }

    /// Establece el radio de las esquinas del botón
    pub fn radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = corner_radius.into();
        self
    }

    /// Establece el nombre y los bytes del icono
    pub fn icon(mut self, name: &'a str, bytes: &'a [u8]) -> Self {
        self.icon = Some((name, bytes));
        self
    }

    /// Establece el color del texto de las sugerencias
    pub fn hint_text_color(mut self, color: impl Into<Color32>) -> Self {
        self.hint_text_color = Some(color.into());
        self
    }

    /// Establece el color del fondo de las sugerencias
    pub fn hint_background_color(mut self, color: impl Into<Color32>) -> Self {
        self.hint_background_color = Some(color.into());
        self
    }

    /// Establece el color del borde de las sugerencias
    pub fn hint_stroke_color(mut self, color: impl Into<Color32>) -> Self {
        self.hint_stroke_color = Some(color.into());
        self
    }

    /// Establece el color del clear
    pub fn x_color(mut self, color: impl Into<Color32>) -> Self {
        self.x_color = color.into();
        self
    }

    /// Renderiza el campo entrada, calcula la distribución del contenido (iconos, texto, sugerencias) y gestiona eventos de borrado
    pub fn show<F: FnMut()>(&mut self, ui: &mut Ui, mut callback: F) -> Response {
        let icon_size = self.icon_size;
        let gap = 6.0;
        let padding_x = 10_f32;

        let list_id = Id::new("inputs");
        let input_id = self.id;

        // registra el id de la instancia en la memoria temporal para el reastro de entradas
        ui.data_mut(|data| {
            let mut ids: Vec<Id> = data.get_temp(list_id).unwrap_or_default();
            if !ids.contains(&input_id) {
                ids.push(input_id);
            }
            data.insert_temp(list_id, ids)
        });

        let size = vec2(self.width, self.height);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

        // determina la paleta de colores según el estado de interacción
        let (bg, st_c, ic_c) = if resp.hovered() {
            ui.set_cursor_icon(CursorIcon::Text);
            (
                self.hover_background_color.unwrap_or(self.background_color),
                self.hover_stroke_color.unwrap_or(self.stroke_color),
                self.hover_icon_color.unwrap_or(self.icon_color),
            )
        } else {
            (self.background_color, self.stroke_color, self.icon_color)
        };

        ui.painter().rect(
            rect,
            self.corner_radius,
            bg,
            Stroke::new(self.stroke_width, st_c),
            StrokeKind::Middle,
        );

        let mut cursor_x = rect.min.x + padding_x;

        // dibuja el icono inicial si se ha aportado
        if let Some((name, bytes)) = self.icon
            && let Some(icon_cache) = self.icon_cache.as_deref_mut()
        {
            let icon_rect = Rect::from_min_size(
                pos2(cursor_x, rect.center().y - icon_size / 2.0),
                Vec2::splat(icon_size),
            );

            // redondea las coordenadas a enteros y prevenir distorciones por subpixeles
            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );

            let icon = icon_cache.get_or_load(ui, name, bytes, ic_c, Vec2::splat(icon_size));

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            cursor_x = icon_rect.max.x + gap;
        }

        let padding_y = 6.0;
        let clear_w = 22.0;
        let gap_edit = 4.0;

        let right_edge = rect.max.x - padding_x;

        // el botón de limpieza solo si es visible cuando el campo contiene texto y existe
        // gestor de la caché para el icono
        let show_clear = !self.input.is_empty() && self.icon_cache.is_some();

        let mut text_right = right_edge;

        // calcula el área ocupada por la insignia de sugerencia si el botón de limpieza no la sobrescribe
        let mut hint_data = None;
        if !show_clear && let Some(hint) = self.hint_text {
            let (hint_color, hint_bg, hint_stc) = match (
                self.hint_text_color,
                self.hint_background_color,
                self.hint_stroke_color,
            ) {
                (Some(c), Some(bg), Some(stc)) => (c, bg, stc),
                _ => (Color32::WHITE, Color32::GRAY, Color32::LIGHT_GRAY),
            };

            let hint_galley = ui.painter().layout_no_wrap(
                hint.to_string(),
                FontId::proportional(10.0),
                ensure_min_lightness(hint_color),
            );
            let pad = vec2(8.0, 2.0);
            let badge_size = hint_galley.size() + pad * 2.0;

            // reestringe el margen derecho del área de edición para no solapar la etiqueta de sugerencia
            text_right -= badge_size.x + gap_edit;
            hint_data = Some((hint_galley, badge_size, hint_bg, hint_stc));
        }

        if show_clear {
            text_right -= clear_w + gap_edit;
        }

        let text_rect = Rect::from_min_max(
            pos2(cursor_x, rect.min.y + padding_y),
            pos2(text_right, rect.max.y - padding_y),
        );

        let response = ui.place(
            text_rect,
            BlazeTextEdit::singleline(self.input)
                .hint_text(self.input_hint_text)
                .desired_width(100.0)
                .id(self.id),
        );

        // Renderizado de la etiqueta de la sugerencia sobre el margen derecho disponible
        if let Some((hint_galley, badge_size, hint_bg, hint_stc)) = hint_data {
            let hint_rect = Rect::from_min_size(
                pos2(
                    right_edge - badge_size.x,
                    rect.center().y - badge_size.y / 2.0,
                ),
                badge_size,
            );

            ui.painter().rect(
                hint_rect,
                3.0,
                hint_bg,
                Stroke::new(1.0, hint_stc),
                StrokeKind::Middle,
            );

            let hint_pos = hint_rect.min + vec2(8.0, 2.0);
            ui.painter()
                .galley(hint_pos, hint_galley, Color32::DARK_GREEN);
        }

        // renderizado del botón de limpieza de contenido
        if show_clear && let Some(icon_cache) = self.icon_cache.as_deref_mut() {
            let c = self.background_color;
            let color = Color32::from_rgba_unmultiplied(
                c.r().saturating_add(50),
                c.g().saturating_add(50),
                c.b().saturating_add(50),
                80,
            );

            let btn_rect = Rect::from_min_size(
                pos2(
                    rect.max.x - padding_x - clear_w,
                    rect.center().y - clear_w / 2.0,
                ),
                vec2(clear_w, clear_w),
            );

            let btn = ui.interact(btn_rect, ui.id().with("clear_input"), Sense::click());

            let icon_size = 16.0;

            let icon_rect = Rect::from_min_size(
                pos2(
                    btn_rect.center().x - icon_size / 2.0,
                    btn_rect.center().y - icon_size / 2.0,
                ),
                Vec2::splat(icon_size),
            );

            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );

            let icon = icon_cache.get_or_load(ui, "x", icons::ICON_X, ic_c, Vec2::splat(icon_size));

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                self.x_color,
            );

            ui.painter().rect_filled(btn_rect, 20, color);

            if btn.hovered() {
                ui.set_cursor_icon(CursorIcon::PointingHand);
            }

            if btn.clicked() {
                self.input.clear();
                ui.memory_mut(|m| m.request_focus(self.id));
                callback()
            }
        }

        response
    }
}
