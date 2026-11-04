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

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, CursorIcon, Id, Rect, Response, Sense, Stroke, StrokeKind, TextStyle,
    TextWrapMode, Ui, Vec2, WidgetText, pos2, vec2,
};
use parking_lot::Mutex;

use crate::ui::icons_cache::icon_cache::IconCache;

/// Botón personalizable con texto e icono
///
/// Permite configurar dimensiones, colores, bordes, radio de las
/// esquinas y posición del contenido. Usa [`IconCache`] para cargar y
/// reutilizar los iconos
///
/// El método `show` dibuja el botón y devuelve su [`Response`] para
/// gestionar las interacciones egui
pub(crate) struct BlazeButton<'a> {
    id: Id,
    ui: &'a mut Ui,
    is_active: bool,
    text: WidgetText,
    width: f32,
    height: f32,
    content_offset_y: f32,
    corner_radius: CornerRadius,
    stroke_width: f32,
    text_color: Color32,
    background_color: Color32,
    hover_background_color: Option<Color32>,
    stroke_color: Color32,
    hover_stroke_color: Option<Color32>,
    icon: Option<(&'a str, &'a [u8])>,
    icon_color: Color32,
    hover_icon_color: Option<Color32>,
    icon_size: f32,
    icon_cache: Option<Arc<Mutex<IconCache>>>,
}

impl<'a> BlazeButton<'a> {
    /// Crea un botón con valores predeterminados e identidad basada en texto
    pub fn new<T: Into<WidgetText>>(
        ui: &'a mut Ui,
        text: T,
        icon_cache: Option<Arc<Mutex<IconCache>>>,
    ) -> Self {
        let widget = text.into();
        let salt = widget.text();
        Self {
            id: ui.id().with(salt),
            ui,
            is_active: true,
            text: widget,
            width: 20.0,
            height: 15.0,
            content_offset_y: 0.0,
            corner_radius: CornerRadius::same(5),
            stroke_width: 0.8,
            background_color: Color32::GRAY,
            hover_background_color: None,
            stroke_color: Color32::GRAY,
            hover_stroke_color: None,
            icon_color: Color32::WHITE,
            text_color: Color32::WHITE,
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

    /// Establece si el botón está activo
    pub fn set_is_active(mut self, bool: bool) -> Self {
        self.is_active = bool;
        self
    }

    /// Establece el tamaño del botón
    pub fn size(mut self, size: impl Into<(f32, f32)>) -> Self {
        let (w, h) = size.into();
        self.width = w;
        self.height = h;
        self
    }

    /// Establece el ancho de la linea del borde
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Ajusta la posición vertical del contenido
    pub fn content_offset_y(mut self, y: f32) -> Self {
        self.content_offset_y = y;
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

    /// Dibuja el botón gestionando su apariencia e interacción
    ///
    /// Adapta el contenido al ancho disponible y devuelve el [`Response`]
    /// de egui para consultar eventos: clicks y estados de interacción
    pub fn show(&mut self) -> Response {
        let icon_size = self.icon_size;
        let gap = 6.0;
        let padding_x = 10_f32;

        let has_text = !self.text.is_empty();
        let has_icon = self.icon.is_some();

        let gap_eff = if has_icon && has_text { gap } else { 0.0 };
        let icon_w = if has_icon { icon_size + gap_eff } else { 0.0 };

        let available_w = self.ui.available_width();
        let button_w = self.width.max(0.0).min(available_w);
        let text_max_w = (button_w - icon_w - padding_x * 2.0).max(0.0);

        let galley = self.text.clone().into_galley(
            self.ui,
            Some(TextWrapMode::Truncate),
            text_max_w,
            TextStyle::Button,
        );

        let text_size = if has_text { galley.size() } else { Vec2::ZERO };
        let show_text = has_text && text_max_w > 0.0 && text_size.x <= text_max_w;

        let content_w = icon_w + text_size.x;
        let content_h = text_size.y.max(if has_icon { icon_size } else { 0.0 });

        let min_w = content_w + padding_x * 2.0;
        let min_h = content_h + 12.0;

        let desired_size = if has_text {
            Vec2::new(self.width.max(min_w), self.height.max(min_h))
        } else {
            Vec2::new(self.width.max(0.0), self.height.max(0.0))
        };

        let rect = self.ui.allocate_space(desired_size).1;

        let sense = if self.is_active {
            Sense::click()
        } else {
            Sense::hover()
        };
        let resp = self.ui.interact(rect, self.id, sense);

        let hovered = self.is_active && resp.hovered();

        let (mut bg, mut st_c, mut ic_c, mut tx_c) = if hovered {
            self.ui.set_cursor_icon(CursorIcon::PointingHand);
            (
                self.hover_background_color.unwrap_or(self.background_color),
                self.hover_stroke_color.unwrap_or(self.stroke_color),
                self.hover_icon_color.unwrap_or(self.icon_color),
                self.text_color,
            )
        } else {
            (
                self.background_color,
                self.stroke_color,
                self.icon_color,
                self.text_color,
            )
        };

        if !self.is_active {
            let dim = 0.45;
            bg = bg.gamma_multiply(dim);
            st_c = st_c.gamma_multiply(dim);
            ic_c = ic_c.gamma_multiply(dim);
            tx_c = tx_c.gamma_multiply(dim);
        }

        self.ui.painter().rect(
            rect,
            self.corner_radius,
            bg,
            Stroke::new(self.stroke_width, st_c),
            StrokeKind::Middle,
        );

        let mut cursor_x = rect.min.x + if has_text { padding_x } else { 0.0 };

        if let Some((name, bytes)) = self.icon
            && let Some(ref icon_cache) = self.icon_cache
        {
            let center = if has_text {
                pos2(
                    cursor_x + icon_size / 2.0,
                    rect.center().y + self.content_offset_y,
                )
            } else {
                rect.center() + vec2(0.0, self.content_offset_y)
            };

            let icon_rect = Rect::from_center_size(center, Vec2::splat(icon_size));

            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );

            let icon = icon_cache
                .lock()
                .get_or_load(self.ui, name, bytes, Vec2::splat(icon_size));

            self.ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                ic_c,
            );

            cursor_x = icon_rect.max.x + gap;
        }

        if show_text && has_text {
            let text_pos = pos2(
                cursor_x,
                rect.center().y - galley.size().y / 2.0 + self.content_offset_y,
            );

            self.ui.painter().galley(text_pos, galley, tx_c);
        }

        resp
    }
}
