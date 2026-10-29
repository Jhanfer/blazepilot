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
    Align, Color32, CornerRadius, Id, Key, LayerId, Layout, Order, Rect, Response, ScrollArea,
    Sense, Shape, TextWrapMode, Ui, UiBuilder, Vec2, WidgetText, emath, lerp, pos2, vec2,
};

use crate::ui::{
    custom_components::UiExt,
    icons_cache::{icon_cache::IconCache, icons},
};

/// Componente de lista desplegable personalizado con animaciones,
/// soporte de iconos de estado
pub(crate) struct BlazeComboBox<'a> {
    id: Id,
    selected_text: WidgetText,
    width: f32,
    height: f32,
    text_color: Color32,
    background_color: Color32,
    hover_background_color: Option<Color32>,
    icon_cache: Option<&'a mut IconCache>,
}

impl<'a> BlazeComboBox<'a> {
    /// Crea una instancia de [`BlazeComboBox`] con valores por defecto
    pub(crate) fn new(ui: &mut Ui, icon_cache: Option<&'a mut IconCache>) -> Self {
        Self {
            id: ui.id().with("combo_box"),
            selected_text: Default::default(),
            width: 80.0,
            height: 20.0,
            text_color: Color32::WHITE,
            background_color: Color32::GRAY,
            hover_background_color: None,
            icon_cache,
        }
    }

    /// Establece el identificador único del combobox
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = id.into();
        self
    }

    /// Establece el tamaño del componente
    pub fn size(mut self, size: impl Into<(f32, f32)>) -> Self {
        let (w, h) = size.into();
        self.width = w;
        self.height = h;
        self
    }

    /// Establece el texto seleccionado
    pub fn selected_text(mut self, selected_text: impl Into<WidgetText>) -> Self {
        self.selected_text = selected_text.into();
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

    /// Renderiza el botón del combobox y gestiona el despliegue del menú
    ///
    /// # Parámetros:
    /// - `ui`: Es el componente de egui que se utiliza para colocar widgets. Representa una región de la pantalla con un tipo de disposición (horizontal o vertical)
    /// - `icon_cache`: *closure* que define los elementos dibujados dentro del área desplegable
    ///
    /// # Retorno
    /// Devuelve un [`Response`] correspondiente al botón de activación principal
    pub fn show<R>(&mut self, ui: &mut Ui, menu_contents: impl FnOnce(&mut Ui) -> R) -> Response {
        // recupera el estado de apertura previo almacenado en la memoria temporal
        let mut open = ui.data(|data| data.get_temp::<bool>(self.id).unwrap_or(false));

        let previous_rect = ui.data(|data| data.get_temp::<Rect>(self.id.with("button_rect")));

        // alterna el estado de apertura si se detecta un click primario dentro del rectángulo
        if let Some(rect) = previous_rect {
            let clicked = ui.input(|i| {
                i.pointer.button_clicked(egui::PointerButton::Primary)
                    && i.pointer.interact_pos().is_some_and(|p| rect.contains(p))
            });

            if clicked {
                open = !open;
            }
        }

        // animación de trasición de estado
        let linear_t = ui.animate_bool_with_time(self.id.with("open_anim"), open, 0.15);

        let hover = self.hover_background_color.unwrap_or(self.background_color);

        // suavizado cúbico para la transición de las esquinas inferiores
        let radius_t = emath::easing::cubic_out((linear_t / 0.35).min(1.0));
        let radius = (20.0 * (1.0 - radius_t)).round() as u8;

        let resp = ui
            .blaze_button(self.selected_text.clone(), None)
            .text_color(self.text_color)
            .background_color(self.background_color)
            .hover_background_color(hover)
            .stroke_color(Color32::TRANSPARENT)
            .size((self.width, self.height))
            .radius(CornerRadius {
                nw: 20,
                ne: 20,
                sw: radius,
                se: radius,
            })
            .show(ui);

        let rect = resp.rect;

        // dibuja y anima la rotación del indicador tipo chevron
        if let Some(icon_cache) = self.icon_cache.as_deref_mut() {
            let icon_size = 16.0;

            let center = pos2(rect.max.x - icon_size / 2.0 - 10.0, rect.center().y);

            let icon_rect = Rect::from_center_size(center, Vec2::splat(icon_size));

            // redondeo a enteros para evitar distorsión visual
            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );

            let icon = icon_cache.get_or_load(
                ui,
                "chevron-up",
                icons::ICON_CHEVRON_UP,
                Color32::WHITE,
                Vec2::splat(icon_size),
            );

            // simula rotación vertical aplastando la escala Y e invirtiendo el mapeo UV
            let flip_t = linear_t * 2.0;
            let (scale, flipped) = if linear_t < 0.5 {
                (1.0 - flip_t, false)
            } else {
                (flip_t - 1.0, true)
            };

            let half_width = icon_size * scale * 0.5;
            let animated_rect =
                Rect::from_center_size(rounded_rect.center(), vec2(icon_size, half_width * 2.0));

            let uv = if flipped {
                Rect::from_min_max(pos2(1.0, 1.0), pos2(0.0, 0.0))
            } else {
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
            };

            ui.painter()
                .image(icon.id(), animated_rect, uv, Color32::WHITE);
        }

        // calcula el desplazamiento y la altura máxima disponible en pantalla
        let slide = (1.0 - emath::easing::cubic_out(linear_t)) * 6.0;
        let pos = pos2(rect.min.x, rect.max.y - slide);

        let screen = ui.content_rect();

        let available_height = screen.max.y - rect.min.y;
        let max_dropdown_height = 200.0_f32.min(available_height);
        let animated_height = lerp(0.0..=max_dropdown_height, radius_t);

        let fill = self.background_color.linear_multiply(radius_t);
        let mut scroll_offset = ui.data(|data| {
            data.get_temp::<f32>(self.id.with("scroll_y"))
                .unwrap_or(0.0)
        });

        if radius_t > 0.001 {
            // renderizado en capa superior para evitar que otros widgets queden por debajo
            let layer = LayerId::new(Order::Foreground, self.id.with("dropdown_layer"));

            let dropdown_rect = Rect::from_min_size(pos, vec2(rect.width(), animated_height));

            // captura el desplazamiento vertical del puntero cuando está sobre el menú desplegable
            let delta = ui.input_mut(|i| {
                let over = i
                    .pointer
                    .latest_pos()
                    .is_some_and(|p| dropdown_rect.contains(p));
                if over {
                    let d = i.smooth_scroll_delta.y;
                    i.smooth_scroll_delta = Vec2::ZERO;
                    d
                } else {
                    0.0
                }
            });

            scroll_offset = (scroll_offset - delta).max(0.0);

            let painter = ui.layer_painter(layer);

            let content_rect = Rect::from_min_max(
                dropdown_rect.min + vec2(8.0, 0.0),
                pos2(
                    dropdown_rect.max.x - 8.0,
                    (dropdown_rect.max.y - 8.0).max(dropdown_rect.min.y),
                ),
            );

            ui.move_to_top(layer);

            let mut dd_ui = ui.new_child(
                UiBuilder::new()
                    .layer_id(layer)
                    .max_rect(content_rect)
                    .layout(Layout::top_down(Align::Min)),
            );

            dd_ui.shrink_clip_rect(content_rect);

            // bloqueador de interacción para aislar clicks dentro de la lista
            dd_ui.interact(
                dropdown_rect,
                self.id.with("blocker"),
                Sense::click_and_drag(),
            );

            // marcador de forma previa para dibujar el fondo redondeado debajo del contenido procesado
            let bg_idx = painter.add(Shape::Noop);

            let out = ScrollArea::vertical()
                .id_salt(self.id.with("dropdown_scroll"))
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .max_height(animated_height.max(0.0))
                .vertical_scroll_offset(scroll_offset)
                .show(&mut dd_ui, |ui| {
                    ui.shrink_clip_rect(content_rect);
                    ui.set_opacity(linear_t);
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    menu_contents(ui)
                });

            let wanted = (out.content_size.y + 8.0).clamp(40.0, max_dropdown_height);
            let bg_h = animated_height.min(wanted);

            // sustituye el marcador "Noop" por el rect dibujado con el alto exacto del contenido
            painter.set(
                bg_idx,
                Shape::rect_filled(
                    Rect::from_min_size(pos, vec2(rect.width(), bg_h)),
                    CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: 20,
                        se: 20,
                    },
                    fill,
                ),
            );

            let max = (out.content_size.y - out.inner_rect.height()).max(0.0);
            scroll_offset = scroll_offset.min(max);

            // cierra el menú desplegable si se presiona "Esc" o se hace clck fuera
            let clicked_outside = ui.input(|i| {
                let any = i.pointer.primary_clicked() || i.pointer.secondary_clicked();
                any && i
                    .pointer
                    .latest_pos()
                    .map(|p| !content_rect.contains(p) && !rect.contains(p))
                    .unwrap_or(true)
            });

            if open && (clicked_outside || ui.input(|i| i.key_pressed(Key::Escape))) {
                open = false;
            }
        }

        // guarda el estado del componente para el próximo frame
        ui.data_mut(|data| {
            data.insert_temp(self.id.with("button_rect"), resp.rect);
            data.insert_temp(self.id, open);
            data.insert_temp(self.id.with("scroll_y"), scroll_offset);
        });

        resp
    }
}
