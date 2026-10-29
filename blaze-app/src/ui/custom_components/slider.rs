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
    Color32, Id, Rect, Response, Sense, Stroke, Ui, emath::Numeric, lerp, pos2, remap_clamp, vec2,
};
use std::ops::RangeInclusive;

/// Widget de control deslizante numérico con soporte para tipos númericos y colores
/// personalizados
pub(crate) struct BlazeSlider<'a, Num: Numeric> {
    id: Id,
    value: &'a mut Num,
    range: RangeInclusive<Num>,
    height: f32,
    width: f32,
    bg_color: Color32,
    accent: Color32,
}

impl<'a, Num: Numeric> BlazeSlider<'a, Num> {
    /// Genera una nueva instancia de [`BlazeSlider`]
    ///
    /// # Parámetros:
    /// - `ui`: Es el componente de egui que se utiliza para colocar widgets. Representa una región de la pantalla con un tipo de disposición (horizontal o vertical)
    /// - `value`: Referencia mutable al valor numérico que se actualizará con el deslizador
    /// - `range`: Rango inclusivo que delimita los valores mínimos y máximos permitidos por el deslizador
    pub(crate) fn new(
        ui: &mut Ui,
        value: &'a mut Num,
        range: impl Into<RangeInclusive<Num>>,
    ) -> Self {
        Self {
            id: ui.id().with("Slider"),
            value,
            range: range.into(),
            height: 30.0,
            width: 80.0,
            bg_color: Color32::GRAY,
            accent: Color32::LIGHT_GRAY,
        }
    }

    /// Establece el tamaño del widget
    pub fn size(mut self, size: impl Into<(f32, f32)>) -> Self {
        let (w, h) = size.into();
        self.width = w;
        self.height = h;
        self
    }

    /// Establece el color de fondo
    pub fn background_color(mut self, color: Color32) -> Self {
        self.bg_color = color;
        self
    }

    /// Establece el color de acento de la barra
    pub fn accent(mut self, color: Color32) -> Self {
        self.accent = color;
        self
    }

    /// Calcula el rango horizontal útil (en pixeles) considerando el radio del manipulador
    /// para evitar que este se sobresalga del rect al alcanzar los extremos
    fn handle_position_range(&self, rect: &Rect, handle_radius: f32) -> RangeInclusive<f32> {
        (rect.left() + handle_radius)..=(rect.right() - handle_radius)
    }

    /// Convierte una posición X de pixeles de la pantalla al valor numérico correspondiente
    /// dentro del rango dinámico configurado [`Self::range`]
    fn value_from_x(&self, x: f32, position_range: RangeInclusive<f32>) -> Num {
        let start_f64 = self.range.start().to_f64();
        let end_f64 = self.range.end().to_f64();

        // mapea la posición "x" normalizadaa a [0.0, 1.0] dentro del rango en pixeles
        let t = remap_clamp(x, position_range, 0.0..=1.0) as f64;

        let val_f64 = lerp(start_f64..=end_f64, t);

        Num::from_f64(val_f64)
    }

    /// Convierte el valor numérico actual a una coordenada horizontal X relativa al
    /// rango visual
    fn x_from_value(&self, position_range: RangeInclusive<f32>) -> f32 {
        let start_f64 = self.range.start().to_f64();
        let end_f64 = self.range.end().to_f64();
        let val_f64 = self.value.to_f64();

        let t = remap_clamp(val_f64, start_f64..=end_f64, 0.0..=1.0);

        lerp(position_range, t as f32)
    }

    /// Renderiza el slider y procesa las interacciones de arrastre o click
    ///
    /// # Retorno
    /// Devuelve un [`Response`] que refleja el estado de interacción y marca el cambio
    /// (`response.mark_changed()`) si el valor vinculado ha sido modificado
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(self.width, self.height), Sense::click_and_drag());

        response.id = self.id;

        let handle_radius = self.height / 2.0;
        let position_range = self.handle_position_range(&rect, handle_radius);

        // actualiza el valor mutable cuando el usuario arrastra o hace click sobre el slider
        if (response.dragged() || response.clicked())
            && let Some(pos) = response.interact_pointer_pos()
        {
            let new_val = self.value_from_x(pos.x, position_range.clone());
            if *self.value != new_val {
                *self.value = new_val;
                response.mark_changed();
            }
        }

        let target_x = self.x_from_value(position_range);
        let animation_time = if response.dragged() { 0.0 } else { 0.5 };

        // usa como clave dinámica "target_x" para reaccionar ante cambios bruscos de valor
        let animated_x = ui.animate_value_with_time(
            self.id.with(format!("slider_animated_{target_x}")),
            target_x,
            animation_time,
        );

        if ui.is_rect_visible(rect) {
            let radius = self.height / 2.5;

            let progress_rect = Rect::from_min_size(
                pos2(rect.min.x, rect.min.y + (rect.height() - self.height) / 2.0),
                vec2(rect.width(), self.height),
            );

            // dibuja el fondo de la barra
            ui.painter()
                .rect_filled(progress_rect, self.height / 2.0, self.bg_color);

            let center = pos2(animated_x, progress_rect.center().y);

            let filled_rect = Rect::from_min_max(
                progress_rect.min,
                pos2(center.x + radius, progress_rect.max.y),
            );

            // dibuja el relleno de progreso hasta la posición animada
            ui.painter()
                .rect_filled(filled_rect, self.height / 2.0, self.accent);

            // renderiza el indicador o botón circular del deslizador
            ui.painter().circle_filled(center, radius, Color32::WHITE);
            ui.painter()
                .circle_stroke(center, radius, Stroke::new(1.5, Color32::BLACK));
        }

        response
    }
}
