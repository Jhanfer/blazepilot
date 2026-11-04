use crate::ui::icons_cache::icon_cache::IconCache;
use egui::{Color32, Id, Rect, Response, Sense, Ui, Vec2, pos2, vec2};
use parking_lot::Mutex;
use std::sync::Arc;

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

    pub fn size(mut self, size: impl Into<Vec2>) -> Self {
        self.size = size.into();
        self
    }

    pub fn tint(mut self, tint: impl Into<Color32>) -> Self {
        self.tint = tint.into();
        self
    }

    pub fn show(&mut self) -> Response {
        let (icon, bytes) = (self.name, self.bytes);
        let (icon_rect, mut icon_resp) = self.ui.allocate_exact_size(self.size, Sense::click());

        icon_resp.id = self.id;

        let rounded_rect = Rect::from_min_max(
            pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
            pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
        );

        let icon = self
            .icon_cache
            .lock()
            .get_or_load(self.ui, icon, bytes, self.size);

        self.ui.painter().image(
            icon.id(),
            rounded_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            self.tint,
        );

        icon_resp
    }
}
