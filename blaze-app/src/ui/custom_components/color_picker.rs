use egui::{
    Color32, Id, Mesh, Popup, PopupCloseBehavior, Rect, Response, Sense, Shape, Slider,
    SliderClamping, Stroke, Ui, lerp, pos2, vec2,
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
                ui.add(
                    Slider::new(&mut ps.target_hue, 0.0..=1.0)
                        .clamping(SliderClamping::Always)
                        .text("Hue"),
                );

                ui.add(
                    Slider::new(&mut ps.hsvs.a, 0.0..=1.0)
                        .clamping(SliderClamping::Always)
                        .text("Alpha"),
                );

                let diff = ps.target_hue - ps.hsvs.h;

                if diff.abs() > 0.001 {
                    ps.hsvs.h += diff;
                } else {
                    ps.hsvs.h = ps.target_hue;
                }

                let size = vec2(240.0, 180.0);
                let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());

                if (response.dragged() || response.clicked())
                    && let Some(pointer_pos) = response.interact_pointer_pos()
                {
                    ps.hsvs.s = ((pointer_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                    ps.hsvs.v =
                        (1.0 - (pointer_pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
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
                    self.paint_sv_square(ui, rect, ps.hsvs.h, ps.hsvs.s, ps.hsvs.v);
                }

                sv_response = response;
            });

        sv_response
    }

    fn paint_sv_square(&self, ui: &mut Ui, rect: Rect, hue: f32, selected_s: f32, selected_v: f32) {
        const STEPS: usize = 36;
        let mut mesh = Mesh::default();

        for y in 0..=STEPS {
            for x in 0..=STEPS {
                let s = x as f32 / STEPS as f32;
                let v = 1.0 - y as f32 / STEPS as f32;

                let color = hsv_to_rgb(hue, s, v);

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

        let color = hsv_to_rgb(hue, selected_s, selected_v);

        ui.painter().circle_filled(selector_pos, 8.0, color);

        ui.painter()
            .circle_stroke(selector_pos, 8.0, Stroke::new(2.0, Color32::WHITE));

        ui.painter()
            .circle_stroke(selector_pos, 9.0, Stroke::new(1.0, Color32::BLACK));
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let h = h.rem_euclid(1.0) * 6.0;
    let c = v * s;
    let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color32::from_rgb(
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
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
