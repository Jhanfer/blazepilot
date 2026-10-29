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

use crate::ui::{custom_components::UiExt, themes::platform::structs::AsColor};
use egui::{
    Color32, Id, Layout, Mesh, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, lerp, pos2,
    vec2,
};
use std::fmt::Debug;

/// Convierte valores RGBA normalizados `[0.0, 1.0]` a la estructura [`BlazeHsva`]
trait RgbToHsv {
    fn rgb_to_hsv(&self) -> BlazeHsva;
}

impl RgbToHsv for Color32 {
    fn rgb_to_hsv(&self) -> BlazeHsva {
        let [r, g, b, a] = self.to_srgba_unmultiplied();
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        let a = a as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let value = max;

        let mut saturation = 0.0;
        if max != 0.0 {
            saturation = delta / max;
        }

        let mut hue = 0.0;

        if delta != 0.0 {
            if max == r {
                hue = 60.0 * (((g - b) / delta) % 6.0);
            } else if max == g {
                hue = 60.0 * (((b - r) / delta) + 2.0);
            } else if max == b {
                hue = 60.0 * (((r - g) / delta) + 4.0);
            }
        }

        if hue < 0.0 {
            hue += 360.0;
        }

        let h = hue / 360.0;
        let s = saturation;
        let v = value;

        BlazeHsva { h, s, v, a }
    }
}

/// Representación interna de color en espacio HSVA (tono, saturación, valor, alfa)
///
/// Se utiliza para mantener el estado contínuo en el selector de color y evitar
/// la pérdida de tono o saturacíon al convertir desde o hacia espacios de color RGB
#[derive(Clone, Debug, Copy, Default)]
struct BlazeHsva {
    h: f32,
    s: f32,
    v: f32,
    a: f32,
}

impl BlazeHsva {
    /// Convierte el color actual HSVA a RGBA normalizados `[0.0, 1.0]`
    pub fn to_rgba(self) -> [f32; 4] {
        let Self { h, s, v, a } = self;
        let [r, g, b] = Self::rgb_from_hsv((h, s, v));
        [r, g, b, a]
    }

    /// Convierte una tupla HSV a RGB normalizado entre `[0.0, 1.0]`
    ///
    /// Maneja el empaquetado gradual del tono dentro del límite estándar
    pub fn rgb_from_hsv((h, s, v): (f32, f32, f32)) -> [f32; 3] {
        // asegura que el tono permanezca estrictamente en el rango [0.0, 1.0],
        // evitando los valores negativos
        let h = (h.fract() + 1.0).fract();
        let s = s.clamp(0.0, 1.0);

        let f = h * 6.0 - (h * 6.0).floor();
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        match (h * 6.0).floor() as i32 % 6 {
            0 => [v, t, p],
            1 => [q, v, p],
            2 => [p, v, t],
            3 => [p, q, v],
            4 => [t, p, v],
            5 => [v, p, q],
            _ => unreachable!(),
        }
    }

    /// Preserva las propiedades del estado anterior (como el tono) cuando el nuevo
    /// color es acromático (negro, blanco o transparente) y no es posible
    /// recuperar información del matiz válida
    fn update_from_color(&mut self, color: Color32) {
        let new = color.rgb_to_hsv();

        self.a = new.a;
        if self.a <= 0.05 {
            return;
        }

        self.v = new.v;
        if self.v <= f32::EPSILON {
            return;
        }

        self.s = new.s;
        if self.s <= f32::EPSILON {
            return;
        }

        self.h = new.h;
    }
}

/// Estado persistente para el widget [`BlazeColorSelector`]
///
/// Almacena el valor de color en espacio HSVA, el texto en hexadecimal de dicho color
/// sincronizado y el estado de foco de la entrada para evitar sobreescritura de texto
#[derive(Debug, Clone)]
struct PickerState {
    hsva: BlazeHsva,
    hex: String,
    hex_focused: bool,
}

/// Selector de color desplegable con controles para el tono,
/// área SV y entrada HEX
pub(crate) struct BlazeColorSelector<'a> {
    id: Id,
    rgba: &'a mut Color32,
    bg_color: Color32,
}

impl<'a> BlazeColorSelector<'a> {
    /// Instancia un nuevo selector de color
    pub(crate) fn new<T: Into<&'a mut Color32>>(
        ui: &mut Ui,
        rgba: T,
        id_source: impl std::hash::Hash + Debug,
    ) -> Self {
        let rgba = rgba.into();
        Self {
            id: ui.id().with(id_source),
            rgba,
            bg_color: Color32::LIGHT_GRAY,
        }
    }

    /// Establece el color de fondo
    pub fn background_color(mut self, color: impl Into<Color32>) -> Self {
        self.bg_color = color.into();
        self
    }

    /// Garantiza que la cadena del HEX contenga el prefijo "#" para el parseo correcto
    fn normalize_hex(&self, hex: &str) -> String {
        let text = hex.trim();
        if text.starts_with("#") {
            text.to_string()
        } else {
            format!("#{text}")
        }
    }

    /// Renderiza el botón indicador de color y gestiona la ventana emergente
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let memory_id = self.id.with("picker_state");
        let enter: Color32 = *self.rgba;

        let from_color = |color: Color32| {
            let hsva = color.rgb_to_hsv();
            PickerState {
                hsva,
                hex: color.to_hex(),
                hex_focused: false,
            }
        };

        // carga o inicializa el estado del selector desde la memoria temporal
        let mut ps: PickerState = ui
            .data(|d| d.get_temp::<PickerState>(memory_id))
            .unwrap_or_else(|| from_color(enter));

        let own = hsva_to_color32(ps.hsva.h, ps.hsva.s, ps.hsva.v, ps.hsva.a);

        // tolerancia a errores de redondeo al convertir entre f32 y u8
        let colors_are_close = |c1: Color32, c2: Color32| {
            let [r1, g1, b1, a1] = c1.to_srgba_unmultiplied();
            let [r2, g2, b2, a2] = c2.to_srgba_unmultiplied();
            r1.abs_diff(r2) <= 2
                && g1.abs_diff(g2) <= 2
                && b1.abs_diff(b2) <= 2
                && a1.abs_diff(a2) <= 2
        };

        // si el valor externo rgba cambia de forma independiente, actualiza el estado
        // interno respetando el foco del usuario
        if !colors_are_close(enter, own) {
            ps.hsva.update_from_color(enter);

            if !ps.hex_focused {
                let hex_matches = self
                    .normalize_hex(&ps.hex)
                    .as_color()
                    .map(|c| c == enter)
                    .unwrap_or(false);

                if !hex_matches {
                    ps.hex = enter.to_hex();
                }
            }
        }

        // renderizado el botón de los colores
        let size = vec2(40.0, 20.0);
        let (rect, btn_resp) = ui.allocate_exact_size(size, Sense::click());

        ui.painter().rect_filled(rect, 20.0, enter);

        let mut sv_response: Response = btn_resp.clone();
        let mut hex_applied = false;

        let popup_id = ui.auto_id_with("popup");
        let popup_result = ui
            .blaze_popup()
            .id(popup_id)
            .toggle(&btn_resp)
            .pos(rect.left_bottom())
            .inner_margin(10.0)
            .size(vec2(255.0, 295.0))
            .show(ui, |ui| {
                let available = ui.available_size();

                ui.allocate_ui_with_layout(
                    available,
                    Layout::bottom_up(egui::Align::Center),
                    |ui| {
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            let hex_resp = ui
                                .blaze_input("hex", None, &mut ps.hex, None)
                                .background_color(Color32::BLACK)
                                .radius(20)
                                .size((100.0, 30.0))
                                .show(ui, || {});

                            if hex_resp.changed() {
                                let text = self.normalize_hex(&ps.hex);

                                if let Ok(parsed) = text.as_color() {
                                    let old_h = ps.hsva.h;
                                    let typed = std::mem::take(&mut ps.hex);

                                    let new_state = from_color(parsed);
                                    ps.hsva = new_state.hsva;

                                    // mantiene el tono previo cuando el color parseado es
                                    // acromático
                                    if ps.hsva.s <= f32::EPSILON
                                        || ps.hsva.v <= f32::EPSILON
                                        || ps.hsva.a < 0.05
                                    {
                                        ps.hsva.h = old_h;
                                    }
                                    ps.hex = typed;

                                    *self.rgba = parsed;

                                    hex_applied = true;
                                }
                            }

                            ps.hex_focused = hex_resp.has_focus();

                            let (rect, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 30.0),
                                Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 20, *self.rgba);
                        });

                        ui.add_space(10.0);

                        let hsva_b = (ps.hsva.h, ps.hsva.s, ps.hsva.v, ps.hsva.a);

                        ui.add_space(10.0);

                        self.paint_alpha_slider(
                            ui,
                            &mut ps.hsva.a,
                            ps.hsva.h,
                            ps.hsva.s,
                            ps.hsva.v,
                            self.bg_color,
                        );

                        ui.add_space(10.0);

                        self.paint_hue_slider(ui, &mut ps.hsva.h, self.bg_color);

                        // área interactiva de la selección 2D (saturación y valor)
                        let size = vec2(240.0, 180.0);
                        let (rect, mut response) =
                            ui.allocate_exact_size(size, Sense::click_and_drag());

                        if (response.dragged() || response.clicked())
                            && let Some(pointer_pos) = response.interact_pointer_pos()
                        {
                            ps.hsva.s =
                                ((pointer_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                            ps.hsva.v = (1.0 - (pointer_pos.y - rect.top()) / rect.height())
                                .clamp(0.0, 1.0);
                        }

                        let hsva_a = (ps.hsva.h, ps.hsva.s, ps.hsva.v, ps.hsva.a);

                        // si los controles modifican el estado HSVA, propaga los cambios a "*self.rgba" y "hex"
                        if hsva_a != hsva_b {
                            let [r, g, b, a] = ps.hsva.to_rgba();

                            let q = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;

                            let new_color = Color32::from_rgba_unmultiplied(q(r), q(g), q(b), q(a));

                            if new_color != *self.rgba {
                                *self.rgba = new_color;

                                let hex_matches = self
                                    .normalize_hex(&ps.hex)
                                    .as_color()
                                    .map(|c| c == new_color)
                                    .unwrap_or(false);

                                if !hex_matches {
                                    ps.hex = new_color.to_hex();
                                }

                                response.mark_changed();
                            }
                        }

                        if ui.is_rect_visible(rect) {
                            self.paint_sv_square(ui, rect, ps.hsva.h, ps.hsva.s, ps.hsva.v);
                        }

                        sv_response = response;
                    },
                );
            });
        if hex_applied {
            sv_response.mark_changed();
        }

        if popup_result.is_none() {
            ps.hex_focused = false;
        }

        ui.data_mut(|data| {
            data.insert_temp(memory_id, ps.clone());
        });

        sv_response
    }

    /// Dibuja una barra de opacidad mediante una malla con gradiente personalizado
    fn paint_alpha_slider(
        &self,
        ui: &mut Ui,
        alpha: &mut f32,
        h: f32,
        s: f32,
        v: f32,
        bg_color: Color32,
    ) -> Response {
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(240.0, 16.0), Sense::click_and_drag());

        if (response.dragged() || response.clicked())
            && let Some(pos) = response.interact_pointer_pos()
        {
            *alpha = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);

            response.mark_changed();
        }

        let t = alpha.clamp(0.0, 1.0);

        // construcción de los triángulos para el gradiente alfa
        if ui.is_rect_visible(rect) {
            const STEPS: usize = 64;
            let mut mesh = Mesh::default();

            for i in 0..=STEPS {
                let t = i as f32 / STEPS as f32;
                let x = lerp(rect.left()..=rect.right(), t);

                let color = hsva_to_color32(h, s, v, t);

                let base = mesh.vertices.len() as u32;

                mesh.colored_vertex(pos2(x, rect.top()), color);
                mesh.colored_vertex(pos2(x, rect.bottom()), color);

                if i > 0 {
                    let prev_top = base - 2;
                    let prev_bottom = base - 1;
                    let cur_top = base;
                    let cur_bottom = base + 1;

                    mesh.add_triangle(prev_top, cur_top, prev_bottom);
                    mesh.add_triangle(cur_top, cur_bottom, prev_bottom);
                }
            }

            ui.painter().add(Shape::mesh(mesh));

            let r = rect.height() / 2.0;
            let w = r * 0.42 + 1.0;
            ui.painter()
                .rect_stroke(rect, r, Stroke::new(w, bg_color), StrokeKind::Outside);

            let x = lerp(rect.left()..=rect.right(), t);
            let center = pos2(x, rect.center().y);

            ui.painter().circle_filled(center, 8.0, Color32::WHITE);
            ui.painter()
                .circle_stroke(center, 8.0, Stroke::new(1.5, Color32::BLACK));
        }

        response
    }

    /// Dibuja la barra de selección del espectro cromático (hue)
    fn paint_hue_slider(&self, ui: &mut Ui, hue: &mut f32, bg_color: Color32) -> Response {
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(240.0, 16.0), Sense::click_and_drag());

        if (response.dragged() || response.clicked())
            && let Some(pos) = response.interact_pointer_pos()
        {
            *hue = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);

            response.mark_changed();
        }

        let t = hue.clamp(0.0, 1.0);

        if ui.is_rect_visible(rect) {
            const STEPS: usize = 36;
            let mut mesh = Mesh::default();

            // interpola los colores del arcoiris a lo largo de la barra
            for i in 0..=STEPS {
                let t = i as f32 / STEPS as f32;
                let x = lerp(rect.left()..=rect.right(), t);

                let color = hsva_to_color32(t, 1.0, 1.0, 1.0);

                let base = mesh.vertices.len() as u32;

                mesh.colored_vertex(pos2(x, rect.top()), color);
                mesh.colored_vertex(pos2(x, rect.bottom()), color);

                if i > 0 {
                    let prev_top = base - 2;
                    let prev_bottom = base - 1;
                    let cur_top = base;
                    let cur_bottom = base + 1;

                    mesh.add_triangle(prev_top, cur_top, prev_bottom);
                    mesh.add_triangle(cur_top, cur_bottom, prev_bottom);
                }
            }

            ui.painter().add(Shape::mesh(mesh));

            let r = rect.height() / 2.0;
            let w = r * 0.42 + 1.0;
            ui.painter()
                .rect_stroke(rect, r, Stroke::new(w, bg_color), StrokeKind::Outside);

            let x = lerp(rect.left()..=rect.right(), t);
            let center = pos2(x, rect.center().y);

            ui.painter().circle_filled(center, 8.0, Color32::WHITE);
            ui.painter()
                .circle_stroke(center, 8.0, Stroke::new(1.5, Color32::BLACK));
        }

        response
    }

    /// Genera el plano 2D de saturación (eje X) y valor (eje Y)
    /// ajustando las esquinas para quedar redondeadas
    fn paint_sv_square(&self, ui: &mut Ui, rect: Rect, hue: f32, selected_s: f32, selected_v: f32) {
        const STEPS: usize = 36;
        let mut mesh = Mesh::default();
        let r = 10.0;

        let ctl = pos2(rect.left() + r, rect.top() + r);
        let ctr = pos2(rect.right() - r, rect.top() + r);
        let cbl = pos2(rect.left() + r, rect.bottom() - r);
        let cbr = pos2(rect.right() - r, rect.bottom() - r);

        // genera la red de vértices 2D exhibiendo el tono fijo y variando s y v
        for y in 0..=STEPS {
            for x in 0..=STEPS {
                let s = x as f32 / STEPS as f32;
                let v = 1.0 - y as f32 / STEPS as f32;

                let color = hsva_to_color32(hue, s, v, 1.0);

                let px = lerp(rect.left()..=rect.right(), s);
                let py = lerp(rect.top()..=rect.bottom(), y as f32 / STEPS as f32);
                let mut pos = pos2(px, py);

                // recorta los vértices que sobresalen del radio de las esquinas
                if pos.x < ctl.x && pos.y < ctl.y && pos.distance(ctl) > r {
                    let dir = (pos - ctl).normalized();
                    pos = ctl + (dir * r);
                }

                if pos.x > ctr.x && pos.y < ctr.y && pos.distance(ctr) > r {
                    let dir = (pos - ctr).normalized();
                    pos = ctr + (dir * r);
                }

                if pos.x < cbl.x && pos.y > cbl.y && pos.distance(cbl) > r {
                    let dir = (pos - cbl).normalized();
                    pos = cbl + (dir * r);
                }

                if pos.x > cbr.x && pos.y > cbr.y && pos.distance(cbr) > r {
                    let dir = (pos - cbr).normalized();
                    pos = cbr + (dir * r);
                }

                mesh.colored_vertex(pos, color);
            }
        }

        // conecta la grilla de vértices formando dos triangulos por celda
        for y in 0..STEPS {
            for x in 0..STEPS {
                let top_left = (y * (STEPS + 1) + x) as u32;
                let top_right = top_left + 1;
                let bottom_left = top_left + (STEPS + 1) as u32;
                let bottom_right = bottom_left + 1;

                mesh.add_triangle(top_left, top_right, bottom_left);
                mesh.add_triangle(top_right, bottom_right, bottom_left);
            }
        }

        ui.painter().add(Shape::mesh(mesh));

        ui.painter().rect_stroke(
            rect,
            r,
            Stroke::new(1.5, ui.visuals().window_fill),
            StrokeKind::Middle,
        );

        // Pintar el circulo selector
        let selector_pos = pos2(
            lerp(rect.left()..=rect.right(), selected_s),
            lerp(rect.bottom()..=rect.top(), selected_v),
        );

        let color = hsva_to_color32(hue, selected_s, selected_v, 1.0);

        ui.painter().circle_filled(selector_pos, 12.0, color);

        ui.painter()
            .circle_stroke(selector_pos, 12.0, Stroke::new(2.0, Color32::WHITE));

        ui.painter()
            .circle_stroke(selector_pos, 13.0, Stroke::new(1.0, Color32::BLACK));
    }
}

/// Helper para convertir parámetros HSVA directamente a Color32
fn hsva_to_color32(h: f32, s: f32, v: f32, a: f32) -> Color32 {
    let [r, g, b, a] = BlazeHsva { h, s, v, a }.to_rgba();
    let q = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(q(r), q(g), q(b), q(a))
}
