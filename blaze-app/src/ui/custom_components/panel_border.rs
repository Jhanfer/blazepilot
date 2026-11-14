use egui::{CornerRadius, Pos2, Rect, Stroke, StrokeKind, Ui, epaint::PathShape, pos2};

/// Número de subdivisiones lineales usadas para aproximar cada esquina redondeada
/// cuando se dibuja un contorno abierto mediante [`PathShape`]
const ARC_SEGMENTS: usize = 12;

/// Lado del contorno que se omite al dibujar un [`PanelBorder`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenSide {
    /// No se dibuja el lado superior
    Top,
    /// No se dibuja el lado inferior
    Bottom,
}

/// Constructor para dibujar el contorno de un [`Rect`] con esquinas redondeadas
/// y un lado abierto opcional
pub(crate) struct PanelBorder<'a> {
    ui: &'a mut Ui,
    rect: Rect,
    corner_radius: CornerRadius,
    open_side: Option<OpenSide>,
    stroke: Stroke,
}

impl<'a> PanelBorder<'a> {
    /// Instancia un contorno cerrado sobre un [`Rect`] con radio de esquinas de 20px
    /// y un [`Stroke`] por defecto
    pub(crate) fn new(ui: &'a mut Ui, rect: Rect) -> Self {
        Self {
            ui,
            rect,
            corner_radius: CornerRadius::same(20),
            open_side: None,
            stroke: Stroke::default(),
        }
    }

    /// Establece el stroke sobreescriboendo el default
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Establece el radio del borde
    pub fn radius(mut self, radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = radius.into();
        self
    }

    /// Define qué lado del rectángulo queda abierto o `None` para dibujar el perímetro completo
    pub fn open_side(mut self, open_side: impl Into<Option<OpenSide>>) -> Self {
        self.open_side = open_side.into();
        self
    }

    /// Genera los puntos de un arco circular de `from_deg` a `to_deg`
    /// al rededor de `center`
    fn arc(&self, center: Pos2, radius: f32, from_deg: f32, to_deg: f32) -> Vec<Pos2> {
        (0..=ARC_SEGMENTS)
            .map(|i| {
                let t = i as f32 / ARC_SEGMENTS as f32;
                let angle = (from_deg + (to_deg - from_deg) * t).to_radians();
                pos2(
                    center.x + radius * angle.cos(),
                    center.y + radius * angle.sin(),
                )
            })
            .collect()
    }

    /// Dibuja el contorno de un `rect`, omitiendo el lado indicado en [Self::open_side]
    ///
    /// El trazo se desplaza hacia el interior de la mitad de su grosor (`stroke.width() * 0.5`) para que el borde exterior coincida exactamente con los límites del `rect`. En el extremo abierto, los tramos vertcales llegan hasta el límite sin ampliarse, permitiendo acoplar paneles contiguos sin solaparlos
    pub fn paint_border(&mut self) {
        if self.stroke.width <= 0.0 || self.rect.width() <= 0.0 || self.rect.height() <= 0.0 {
            return;
        }

        // si no hay lado abierto, se le delega a `rect_stroke` para aprovechar el trazado nativo de egui
        let Some(open_side) = self.open_side else {
            self.ui.painter().rect_stroke(
                self.rect,
                self.corner_radius,
                self.stroke,
                StrokeKind::Inside,
            );
            return;
        };

        // el trazo de PathShape::line se centra sobre la línea, se desplazan
        // los ejes hacia adentro respecto a la mitad del grosor para simular StrokeKind::Inside
        let inset = self.stroke.width * 0.5;
        let max_radius = (self.rect.width().min(self.rect.height()) * 0.5 - inset).max(0.0);
        let radius = |value: u8| (value as f32 - inset).clamp(0.0, max_radius);

        let left = self.rect.left() + inset;
        let right = self.rect.right() - inset;
        let top = self.rect.top() + inset;
        let bottom = self.rect.bottom() - inset;

        let nw = radius(self.corner_radius.nw);
        let ne = radius(self.corner_radius.ne);
        let sw = radius(self.corner_radius.sw);
        let se = radius(self.corner_radius.se);

        // los 4 puntos de uniòn más 2 esquinas redondeadas de (ARC_SEGMENTS + 1) puntos cada una
        let mut points: Vec<Pos2> = Vec::with_capacity(4 * (ARC_SEGMENTS + 1) + 4);

        match open_side {
            // Recorrido antihorario desde arriba-izquierda hasta arriba-derecha dejando abierto el techo
            OpenSide::Top => {
                points.push(pos2(left, self.rect.top()));
                points.push(pos2(left, bottom - sw));
                points.extend(self.arc(pos2(left + sw, bottom - sw), sw, 180.0, 90.0));
                points.push(pos2(right - se, bottom));
                points.extend(self.arc(pos2(right - se, bottom - se), se, 90.0, 0.0));
                points.push(pos2(right, self.rect.top()));
            }
            // Recorrido horario desde abajo-izquierda hasta abajo-derecha dejando abierto el suelo
            OpenSide::Bottom => {
                points.push(pos2(left, self.rect.bottom()));
                points.push(pos2(left, top + nw));
                points.extend(self.arc(pos2(left + nw, top + nw), nw, 180.0, 270.0));
                points.push(pos2(right - ne, top));
                points.extend(self.arc(pos2(right - ne, top + ne), ne, 270.0, 360.0));
                points.push(pos2(right, self.rect.bottom()));
            }
        }

        self.ui.painter().add(PathShape::line(points, self.stroke));
    }
}
