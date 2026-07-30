use egui::{Label, Response, Ui, WidgetText};

pub trait UiExt {
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>;
}

impl UiExt for Ui {
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>,
    {
        self.add(Label::new(text).selectable(false))
    }
}
