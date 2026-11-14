use crate::ui::icons_cache::icon_cache::IconCache;
use egui::{Color32, Id, Rect, Sense, TextureHandle, TextureId, Ui, Vec2, pos2, vec2};
use parking_lot::Mutex;
use std::sync::Arc;

/// Widget builder para renderizar iconos
///
/// Encapsula la configuración de renderizado (tamaño, tinte, id)
/// y delega la carga, reutilización de la textura a [`IconCache`] compartido
pub(crate) struct BlazeIcon<'a> {
    id: Id,
    ui: &'a mut Ui,
    name: &'a str,
    bytes: &'a [u8],
    size: Vec2,
    icon_cache: Arc<Mutex<IconCache>>,
    tint: Color32,
}

impl<'a> BlazeIcon<'a> {
    /// Crea una nueva instancia para un icono
    ///
    /// Genera un [`Id`] por defecto basado en los datos del icono (bytes y nombre)
    ///
    /// # Parámetros:
    /// - `ui`: referencia mutable al contexto de renderizado de egui
    /// - `icon_data`: tupla con el nombre y los bytes del archivo de imagen
    /// - `icon_cache`: referencia compartida `thread-safe` a la caché de iconos
    pub(crate) fn new(
        ui: &'a mut Ui,
        icon_data: (&'a str, impl Into<&'a [u8]>),
        icon_cache: Arc<Mutex<IconCache>>,
    ) -> Self {
        let (name, bytes) = icon_data;
        let bytes = bytes.into();
        Self {
            id: ui.id().with(format!("{:?}_{}", bytes, name)),
            ui,
            name,
            bytes,
            size: vec2(20.0, 20.0),
            icon_cache,
            tint: Color32::WHITE,
        }
    }

    /// Sobreescribe el [`Id`] único del icono en egui
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = id.into();
        self
    }

    /// Establece las dimensiones con las que se renderizará el icono
    pub fn size(mut self, size: impl Into<Vec2>) -> Self {
        self.size = size.into();
        self
    }

    /// Aplica un tinte de color sobre la textura del icono
    pub fn tint(mut self, tint: impl Into<Color32>) -> Self {
        self.tint = tint.into();
        self
    }

    /// Reserva el espacio correspondiente en lel layout [`Ui`] y pinta el icono
    pub fn show(&mut self) {
        let (rect, _) = self.ui.allocate_exact_size(self.size, Sense::hover());
        self.paint_rect_internal(rect);
    }

    /// Dibuja el icono en una región determinada sin reservar espacio en el layout [`Ui`]
    pub fn show_at(&mut self, rect: Rect) {
        self.paint_rect_internal(rect);
    }

    /// Consume el builder y dibuja el icono en un [`Rect`] específico
    #[allow(unused)]
    pub fn paint_at(mut self, rect: Rect) {
        self.paint_rect_internal(rect);
    }

    /// Consume el builder, carga la textura en caché y devuelve su [`TextureId`]
    #[allow(unused)]
    pub fn into_texture(mut self) -> TextureId {
        self.load_texture().id()
    }

    /// Obtiene el handle de la tectura desde la caché compartida, cargándola si no existe
    fn load_texture(&mut self) -> TextureHandle {
        self.icon_cache
            .lock()
            .get_or_load(self.ui, self.name, self.bytes, self.size)
    }

    /// Pinta la imagen ajustando los límites del [`Rect`] a pixeles enteros
    fn paint_rect_internal(&mut self, rect: Rect) {
        let tex_id = self.load_texture().id();

        // redondea las esquinas para evitar borrosidad por subpixel
        let rounded_rect = Rect::from_min_max(
            pos2(rect.min.x.round(), rect.min.y.round()),
            pos2(rect.max.x.round(), rect.max.y.round()),
        );

        // mapea las textura completa dentro del rect de destino
        self.ui.painter().image(
            tex_id,
            rounded_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            self.tint,
        );
    }
}
