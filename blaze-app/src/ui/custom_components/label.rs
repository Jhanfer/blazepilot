use egui::{Label, Response, Ui, WidgetText};

pub(crate) fn label_ns<T>(ui: &mut Ui, text: T) -> Response
where
    T: Into<WidgetText>,
{
    ui.add(Label::new(text).selectable(false))
}
