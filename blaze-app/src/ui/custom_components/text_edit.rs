use crate::core::system::clipboard_text::{
    keyboard_state::{KeyboardAction, with_keyboard_state},
    text_clipboard::with_text_clipboard,
};
use egui::{FontSelection, Frame, Id, Margin, Response, TextEdit, Widget};

pub struct BlazeTextEdit<'a> {
    text: &'a mut String,
    desired_width: f32,
    hint: String,
    id: Id,
    margin: Margin,
    font_selection: FontSelection,
}

impl<'a> BlazeTextEdit<'a> {
    pub fn singleline(text: &'a mut String) -> Self {
        Self {
            text,
            desired_width: 100.0,
            margin: Margin::symmetric(4, 2),
            id: Id::new(""),
            hint: String::new(),
            font_selection: FontSelection::Default,
        }
    }

    #[inline]
    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = width;
        self
    }

    #[inline]
    pub fn hint_text(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }

    #[inline]
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    #[inline]
    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    pub fn font(mut self, font_selection: impl Into<FontSelection>) -> Self {
        self.font_selection = font_selection.into();
        self
    }
}

impl Widget for BlazeTextEdit<'_> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let output = TextEdit::singleline(self.text)
            .id(self.id)
            .frame(Frame::NONE)
            .desired_width(self.desired_width)
            .hint_text(self.hint)
            .margin(self.margin)
            .font(self.font_selection)
            .show(ui);

        if let Some(range) = output.cursor_range {
            let selected = range.slice_str(self.text).to_string();
            if !selected.is_empty() {
                with_keyboard_state(|k| k.update_selection(selected));
            }
        }

        with_keyboard_state(|k| {
            if let Some(KeyboardAction::Copy | KeyboardAction::Cut) =
                k.get(ui.cumulative_frame_nr())
                && let Some(ref selected) = k.take_selection()
            {
                with_text_clipboard(|c| c.copy(selected.clone()));
            }
        });

        output.response.response
    }
}
