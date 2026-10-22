use core::ops::RangeInclusive;
use egui::{
    Color32, Id, Layout, Mesh, Popup, PopupCloseBehavior, Rect, Response, Sense, Shape, Stroke,
    StrokeKind, Ui, emath, lerp, pos2, remap_clamp, vec2,
};
use std::fmt::Debug;

#[derive(Clone, Debug, Copy, Default)]
struct BlazeHsva {
    h: f32,
    s: f32,
    v: f32,
    a: f32,
}

impl BlazeHsva {
    pub fn to_rgba(self) -> [f32; 4] {
        let Self { h, s, v, a } = self;
        let [r, g, b] = Self::rgb_from_hsv((h, s, v));
        [r, g, b, a]
    }

    pub fn rgb_from_hsv((h, s, v): (f32, f32, f32)) -> [f32; 3] {
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
}

#[derive(Debug, Clone, Copy)]
struct PickerState {
    hsvs: BlazeHsva,
    target_hue: f32,
    target_alpha: f32,
    last_color: Color32,
}

pub(crate) struct BlazeColorPicker<'a> {
    id: Id,
    rgba: &'a mut Color32,
}

impl<'a> BlazeColorPicker<'a> {
    pub(crate) fn new<T: Into<&'a mut Color32>>(
        ui: &mut Ui,
        rgba: T,
        id_source: impl std::hash::Hash + Debug,
    ) -> Self {
        let rgba = rgba.into();
        Self {
            id: ui.id().with(id_source),
            rgba,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let memory_id = self.id.with("picker_state");
        let enter: Color32 = *self.rgba;

        let from_color = |color: Color32| {
            let [r, g, b, a] = color.to_srgba_unmultiplied();
            let hsvs = rgb_to_hsv(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            );
            PickerState {
                hsvs,
                target_hue: hsvs.h,
                target_alpha: hsvs.a,
                last_color: color,
            }
        };

        let mut ps: PickerState = ui
            .data(|d| d.get_temp::<PickerState>(memory_id))
            .unwrap_or_else(|| from_color(enter));

        if ps.last_color != enter {
            let new = from_color(enter);
            ps.hsvs.s = new.hsvs.s;
            ps.hsvs.v = new.hsvs.v;
            ps.hsvs.a = new.hsvs.a;

            if new.hsvs.s > 0.05 && new.hsvs.v > 0.05 {
                ps.hsvs.h = new.hsvs.h;
                ps.target_hue = new.hsvs.h;
            }
            ps.last_color = enter;
        }

        let size = vec2(40.0, 20.0);
        let (rect, btn_resp) = ui.allocate_exact_size(size, Sense::click());

        ui.painter().rect_filled(rect, 20.0, enter);

        let mut sv_response: Response = btn_resp.clone();

        let popup_id = ui.auto_id_with("popup");
        Popup::menu(&btn_resp)
            .id(popup_id)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.allocate_ui_with_layout(
                    vec2(200.0, 200.0),
                    Layout::bottom_up(egui::Align::Center),
                    |ui| {
                        self.paint_hue_alpha_slider(
                            ui,
                            0.0..=1.0,
                            &mut ps.target_hue,
                            ps.hsvs.h,
                            ps.hsvs.s,
                            ps.hsvs.v,
                            ps.hsvs.a,
                            true,
                        );

                        self.paint_hue_alpha_slider(
                            ui,
                            0.0..=1.0,
                            &mut ps.target_alpha,
                            ps.hsvs.h,
                            ps.hsvs.s,
                            ps.hsvs.v,
                            ps.hsvs.a,
                            false,
                        );

                        let diff_hue = ps.target_hue - ps.hsvs.h;
                        let diff_alpha = ps.target_alpha - ps.hsvs.a;

                        if diff_hue.abs() > 0.001 {
                            ps.hsvs.h += diff_hue;
                        } else {
                            ps.hsvs.h = ps.target_hue;
                        }

                        if diff_alpha.abs() > 0.001 {
                            ps.hsvs.a += diff_alpha;
                        } else {
                            ps.hsvs.a = ps.target_alpha;
                        }

                        let size = vec2(240.0, 180.0);
                        let (rect, mut response) =
                            ui.allocate_exact_size(size, Sense::click_and_drag());

                        if (response.dragged() || response.clicked())
                            && let Some(pointer_pos) = response.interact_pointer_pos()
                        {
                            ps.hsvs.s =
                                ((pointer_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                            ps.hsvs.v = (1.0 - (pointer_pos.y - rect.top()) / rect.height())
                                .clamp(0.0, 1.0);
                        }

                        let [r, g, b, a] = ps.hsvs.to_rgba();
                        let q = |x: f32| (x * 255.0).round().clamp(0.0, 255.0) as u8;
                        let new_color = Color32::from_rgba_unmultiplied(q(r), q(g), q(b), q(a));

                        if new_color != *self.rgba {
                            *self.rgba = new_color;
                            response.mark_changed();
                        }

                        ps.last_color = new_color;
                        ui.data_mut(|data| {
                            data.insert_temp(memory_id, ps);
                        });

                        if ui.is_rect_visible(rect) {
                            self.paint_sv_square(
                                ui, rect, ps.hsvs.h, ps.hsvs.s, ps.hsvs.v, ps.hsvs.a,
                            );
                        }

                        sv_response = response;
                    },
                );
            });

        sv_response
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_hue_alpha_slider<Num: emath::Numeric>(
        &mut self,
        ui: &mut Ui,
        range: impl Into<RangeInclusive<Num>>,
        value: &mut Num,
        h: f32,
        s: f32,
        v: f32,
        a: f32,
        is_hue: bool,
    ) {
        let range = range.into();
        let range_f64 = range.start().to_f64()..=range.end().to_f64();
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(240.0, 16.0), Sense::click_and_drag());

        if (response.dragged() || response.clicked())
            && let Some(pos) = response.interact_pointer_pos()
        {
            let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let v = lerp(range_f64.clone(), t as f64);
            *value = Num::from_f64(v);
            response.mark_changed();
        }

        let t = remap_clamp(value.to_f64(), range_f64, 0.0..=1.0) as f32;

        if ui.is_rect_visible(rect) {
            const STEPS: usize = 64;
            let mut mesh = Mesh::default();

            for i in 0..=STEPS {
                let t = i as f32 / STEPS as f32;
                let x = lerp(rect.left()..=rect.right(), t);

                let color = if is_hue {
                    hsva_to_color32(t, s, v, a)
                } else {
                    hsva_to_color32(h, s, t, a)
                };

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
            let bg = ui.visuals().panel_fill;
            ui.painter()
                .rect_stroke(rect, r, Stroke::new(w, bg), StrokeKind::Outside);

            let x = lerp(rect.left()..=rect.right(), t);
            let center = pos2(x, rect.center().y);

            ui.painter().circle_filled(center, 6.0, Color32::WHITE);
            ui.painter()
                .circle_stroke(center, 6.0, Stroke::new(1.5, Color32::BLACK));
        }
    }

    fn paint_sv_square(
        &self,
        ui: &mut Ui,
        rect: Rect,
        hue: f32,
        selected_s: f32,
        selected_v: f32,
        a: f32,
    ) {
        const STEPS: usize = 36;
        let mut mesh = Mesh::default();

        for y in 0..=STEPS {
            for x in 0..=STEPS {
                let s = x as f32 / STEPS as f32;
                let v = 1.0 - y as f32 / STEPS as f32;

                let color = hsva_to_color32(hue, s, v, a);

                let px = lerp(rect.left()..=rect.right(), s);

                let py = lerp(rect.top()..=rect.bottom(), y as f32 / STEPS as f32);

                mesh.colored_vertex(pos2(px, py), color);
            }
        }

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

        // Pintar el circulo selector
        let selector_pos = pos2(
            lerp(rect.left()..=rect.right(), selected_s),
            lerp(rect.bottom()..=rect.top(), selected_v),
        );

        let color = hsva_to_color32(hue, selected_s, selected_v, a);

        ui.painter().circle_filled(selector_pos, 8.0, color);

        ui.painter()
            .circle_stroke(selector_pos, 8.0, Stroke::new(2.0, Color32::WHITE));

        ui.painter()
            .circle_stroke(selector_pos, 9.0, Stroke::new(1.0, Color32::BLACK));
    }
}

fn hsva_to_color32(h: f32, s: f32, v: f32, a: f32) -> Color32 {
    let [r, g, b, a] = BlazeHsva { h, s, v, a }.to_rgba();
    let q = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(q(r), q(g), q(b), q(a))
}

fn rgb_to_hsv(r: f32, g: f32, b: f32, a: f32) -> BlazeHsva {
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
