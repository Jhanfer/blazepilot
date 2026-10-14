use egui::{Response, Ui, WidgetText};

use crate::ui::{custom_components::button::BlazeButton, icons_cache::icon_cache::IconCache};

pub mod button;
pub mod label;
pub mod text_edit;

pub trait UiExt {
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>;

    fn blaze_button<'a, T: Into<WidgetText>>(
        &mut self,
        text: T,
        icon_cache: &'a mut IconCache,
    ) -> BlazeButton<'a>;
}

impl UiExt for Ui {
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>,
    {
        label::label_ns(self, text)
    }

    fn blaze_button<'a, T: Into<WidgetText>>(
        &mut self,
        text: T,
        icon_cache: &'a mut IconCache,
    ) -> BlazeButton<'a> {
        BlazeButton::new(self, text, icon_cache)
    }
}
