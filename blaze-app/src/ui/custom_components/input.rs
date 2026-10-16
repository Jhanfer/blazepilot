#![allow(unused)]

use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Id, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2, pos2, vec2,
};

use crate::ui::{
    custom_components::text_edit::BlazeTextEdit,
    icons_cache::{icon_cache::IconCache, icons},
    modules::utilities::ensure_min_lightness,
};

pub(crate) struct BlazeInput<'a> {
    id: Id,
    input: &'a mut String,
    input_hint_text: &'a str,
    hint_text: Option<&'a str>,
    hint_text_color: Option<Color32>,
    hint_background_color: Option<Color32>,
    hint_stroke_color: Option<Color32>,
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
    icon_cache: &'a mut IconCache,
}

impl<'a> BlazeInput<'a> {
    pub(crate) fn new<T: Into<&'a mut String>>(
        ui: &mut Ui,
        input: T,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        icon_cache: &'a mut IconCache,
    ) -> Self {
        Self {
            id: ui.id().with("input"),
            input: input.into(),
            input_hint_text,
            hint_text,
            hint_text_color: None,
            hint_background_color: None,
            hint_stroke_color: None,
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

    pub fn show<F: FnMut()>(&mut self, ui: &mut Ui, mut callback: F) -> Response {
        let icon_size = self.icon_size;
        let gap = 6.0;
        let padding_x = 10_f32;

        let list_id = Id::new("inputs");
        let input_id = self.id;

        ui.data_mut(|data| {
            let mut ids: Vec<Id> = data.get_temp(list_id).unwrap_or_default();
            if !ids.contains(&input_id) {
                ids.push(input_id);
            }
            data.insert_temp(list_id, ids)
        });

        let size = vec2(self.width, self.height);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

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
            Stroke::new(0.5, st_c),
            StrokeKind::Middle,
        );

        let mut cursor_x = rect.min.x + padding_x;

        if let Some((name, bytes)) = self.icon {
            let icon_rect = Rect::from_min_size(
                pos2(cursor_x, rect.center().y - icon_size / 2.0),
                Vec2::splat(icon_size),
            );

            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );

            let icon = self
                .icon_cache
                .get_or_load(ui, name, bytes, ic_c, Vec2::splat(icon_size));

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

        let show_clear = !self.input.is_empty();

        let text_right = if show_clear {
            right_edge - clear_w - gap_edit
        } else {
            right_edge
        };

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

        // Renderizado de la sugerencia
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

            let hint_size = hint_galley.size();
            let pad = vec2(8.0, 2.0);
            let badge_size = hint_size + pad * 2.0;

            let right_edge = rect.max.x - padding_x;

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

            let hint_pos = hint_rect.min + pad;

            ui.painter()
                .galley(hint_pos, hint_galley, Color32::DARK_GREEN);
        }

        // Icono de X para eliminar el input
        if show_clear {
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

            let icon =
                self.icon_cache
                    .get_or_load(ui, "x", icons::ICON_X, ic_c, Vec2::splat(icon_size));

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
