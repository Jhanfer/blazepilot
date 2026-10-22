use egui::{Color32, Response, Ui, WidgetText};
use std::{fmt::Debug, hash::Hash};

use crate::ui::{
    custom_components::{
        button::BlazeButton, color_picker::BlazeColorPicker, combobox::BlazeComboBox,
        input::BlazeInput,
    },
    icons_cache::icon_cache::IconCache,
};

pub mod button;
pub mod color_picker;
pub mod combobox;
pub mod input;
pub mod label;
pub mod text_edit;

pub trait UiExt<'a> {
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>;

    fn blaze_button<T: Into<WidgetText>>(
        &mut self,
        text: T,
        icon_cache: Option<&'a mut IconCache>,
    ) -> BlazeButton<'a>;

    fn color_picker<T: Into<&'a mut Color32>, U: Hash + Debug>(
        &mut self,
        rgba: T,
        id_source: U,
    ) -> BlazeColorPicker<'a>;

    fn blaze_input<T: Into<&'a mut String>>(
        &mut self,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        input: T,
        icon_cache: &'a mut IconCache,
    ) -> BlazeInput<'a>;

    fn blaze_combobox(&mut self, icon_cache: Option<&'a mut IconCache>) -> BlazeComboBox<'a>;
}

impl<'a> UiExt<'a> for Ui {
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>,
    {
        label::label_ns(self, text)
    }

    fn blaze_button<T: Into<WidgetText>>(
        &mut self,
        text: T,
        icon_cache: Option<&'a mut IconCache>,
    ) -> BlazeButton<'a> {
        BlazeButton::new(self, text, icon_cache)
    }

    fn color_picker<T: Into<&'a mut Color32>, U: Hash + Debug>(
        &mut self,
        rgba: T,
        id_source: U,
    ) -> BlazeColorPicker<'a> {
        BlazeColorPicker::new(self, rgba, id_source)
    }

    fn blaze_input<T: Into<&'a mut String>>(
        &mut self,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        input: T,
        icon_cache: &'a mut IconCache,
    ) -> BlazeInput<'a> {
        BlazeInput::new(self, input, input_hint_text, hint_text, icon_cache)
    }

    fn blaze_combobox(&mut self, icon_cache: Option<&'a mut IconCache>) -> BlazeComboBox<'a> {
        BlazeComboBox::new(self, icon_cache)
    }
}
