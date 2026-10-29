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
    Area, Color32, CornerRadius, Id, Key, LayerId, Order, Pos2, Response, Sense, Stroke,
    StrokeKind, Ui, UiBuilder, pos2, vec2,
};

/// Ventana o menú emergente flotante con posición editable, reestrición al marco
/// de la pantalla y detección de clicks externos para cierre automático
pub(crate) struct BlazePopup {
    id: Id,
    width: f32,
    height: f32,
    radius: CornerRadius,
    inner_margin: f32,
    position: Pos2,
    bg_color: Color32,
    stroke_color: Color32,
    toggle_requested: bool,
}

impl BlazePopup {
    /// Crea una nueva instancia de [`BlazePopup`] con
    /// dimesiones y colores por defecto
    pub(crate) fn new(ui: &mut Ui) -> Self {
        Self {
            id: ui.id().with("popup"),
            width: 80.0,
            height: 30.0,
            radius: 20.into(),
            inner_margin: 5.0,
            position: pos2(0.0, 0.0),
            bg_color: Color32::LIGHT_GRAY,
            stroke_color: Color32::GRAY,
            toggle_requested: false,
        }
    }

    /// Sobrescribe el identificador único del widget
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = id.into();
        self
    }

    /// Establece el margen interior del marco
    pub fn inner_margin(mut self, inner: f32) -> Self {
        self.inner_margin = inner;
        self
    }

    /// Establece el tamaño del componente
    pub fn size(mut self, size: impl Into<(f32, f32)>) -> Self {
        let (w, h) = size.into();
        self.width = w;
        self.height = h;
        self
    }

    /// Establece la posición de apertura
    pub fn pos(mut self, pos: impl Into<Pos2>) -> Self {
        self.position = pos.into();
        self
    }

    /// Alterna el estado de la apertura si el [`Response`]
    /// proporcionado detecta un click
    pub fn toggle(mut self, resp: &Response) -> Self {
        if resp.clicked() {
            self.toggle_requested = true;
        }
        self.position = resp.rect.left_bottom();
        self
    }

    /// Renderiza el widget en primer plano si se encuentra abierto
    ///
    /// # Parámetros:
    /// - `ui`: Es el componente de egui que se utiliza para colocar widgets. Representa una región de la pantalla con un tipo de disposición (horizontal o vertical)
    /// - `callback`: Es el *closure* que recibel el contexto [`Ui`] recortado al margen interior para renderizar el contenido interno
    ///
    /// # Retorno:
    /// Devuelve `Some(InnerResponse)` con el resultado del *closure* si el widget ha estado visible
    /// o `None` si el widget permanece cerrado
    pub fn show<R>(
        &mut self,
        ui: &mut Ui,
        mut callback: impl FnMut(&mut Ui) -> R,
    ) -> Option<egui::InnerResponse<R>> {
        // recupera o crea el estado de apertura del widget
        let memory_id = self.id.with("picker_state");
        let mut open: bool = ui
            .data(|d| d.get_temp::<bool>(memory_id))
            .unwrap_or_default();

        if self.toggle_requested {
            open = !open;
        }

        // si el widget permanece cerrado, persiste su estado y se omite lo demás
        if !open {
            ui.data_mut(|d| d.insert_temp(memory_id, open));
            return None;
        }

        // permite verificar si la ventana ha estado activa en el fotograma anterior para evitar cierres inmediatos por eventos de click registrados en el mismo frane
        let was_open_last_frame = ui.read_response(self.id).is_some();

        let screen = ui.content_rect();
        let pos = self.position;

        // renderizado en una capa encima de los widgets, restringida a los límites visibles del componente
        let a_resp = Area::new(self.id)
            .order(Order::Foreground)
            .fixed_pos(pos)
            .constrain_to(screen)
            .show(ui, |ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(self.width, self.height), Sense::all());
                ui.painter().rect(
                    rect,
                    self.radius,
                    self.bg_color,
                    Stroke::new(0.5, self.stroke_color),
                    StrokeKind::Middle,
                );
                let inner_rect = rect.shrink(self.inner_margin);
                let mut child_ui = ui.new_child(UiBuilder::new().max_rect(inner_rect).layer_id(
                    LayerId::new(Order::Foreground, self.id.with("child_ui_popup")),
                ));

                callback(&mut child_ui)
            });

        // detecta interacciones de cierre con clicks fuera del componente o tecla de esc
        let clicked_outside = was_open_last_frame && a_resp.response.clicked_elsewhere();
        let esc_pressed = ui.input(|i| i.key_pressed(Key::Escape));

        if clicked_outside || esc_pressed {
            open = false
        }

        ui.data_mut(|data| {
            data.insert_temp(memory_id, open);
        });

        Some(a_resp)
    }
}
