use egui::{
    Color32, CornerRadius, CursorIcon, Id, Rect, Response, Sense, Stroke, StrokeKind, TextStyle,
    TextWrapMode, Ui, Vec2, WidgetText, pos2, vec2,
};

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
    text: WidgetText,
    width: f32,
    height: f32,
    content_offset_y: f32,
    corner_radius: CornerRadius,
    text_color: Color32,
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

impl<'a> BlazeButton<'a> {
    /// Crea un botón con valores predeterminados e identidad basada en texto
    pub fn new<T: Into<WidgetText>>(ui: &mut Ui, text: T, icon_cache: &'a mut IconCache) -> Self {
        let widget = text.into();
        let salt = widget.text();
        Self {
            id: ui.id().with(salt),
            text: widget,
            width: 20.0,
            height: 15.0,
            content_offset_y: 0.0,
            corner_radius: CornerRadius::same(5),
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

    /// Establece el tamaño del botón
    pub fn size(mut self, size: impl Into<(f32, f32)>) -> Self {
        let (w, h) = size.into();
        self.width = w;
        self.height = h;
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
    pub fn show(self, ui: &mut Ui) -> Response {
        let icon_size = self.icon_size;
        let gap = 6.0;
        let padding_x = 10_f32;

        let has_text = !self.text.is_empty();
        let has_icon = self.icon.is_some();

        let gap_eff = if has_icon && has_text { gap } else { 0.0 };
        let icon_w = if has_icon { icon_size + gap_eff } else { 0.0 };

        let available_w = ui.available_width();
        let button_w = self.width.max(0.0).min(available_w);
        let text_max_w = (button_w - icon_w - padding_x * 2.0).max(0.0);

        let galley = self.text.into_galley(
            ui,
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

        let rect = ui.allocate_space(desired_size).1;
        let resp = ui.interact(rect, self.id, Sense::click());

        let (bg, st_c, ic_c) = if resp.hovered() {
            ui.set_cursor_icon(CursorIcon::PointingHand);
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

        let mut cursor_x = rect.min.x + if has_text { padding_x } else { 0.0 };

        if let Some((name, bytes)) = self.icon {
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

        if show_text && has_text {
            let text_pos = pos2(
                cursor_x,
                rect.center().y - galley.size().y / 2.0 + self.content_offset_y,
            );

            ui.painter().galley(text_pos, galley, self.text_color);
        }

        resp
    }
}
