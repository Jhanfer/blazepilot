use egui::{Color32, Response, Ui, WidgetText, emath::Numeric};
use parking_lot::Mutex;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use crate::ui::{
    custom_components::{
        button::BlazeButton, color_picker::BlazeColorSelector, combobox::BlazeComboBox,
        icon::BlazeIcon, input::BlazeInput, popup::BlazePopup, slider::BlazeSlider,
    },
    icons_cache::icon_cache::IconCache,
};

pub mod button;
pub mod color_picker;
pub mod combobox;
pub mod icon;
pub mod input;
pub mod label;
pub mod popup;
pub mod slider;
pub mod text_edit;

/// Extensión de [`Ui`] de egui que provee los componentes personalizados
/// de BlazePilot
///
/// Proporciona constructores para añadir botones, selector de color, campo de entrada,
/// lista desplegable, ventana emergente y deslizador con el estilo y comportamiento
/// adaptados a la aplicación
pub trait UiExt<'a> {
    /// Renderiza una etiqueta no seleccionable
    ///
    /// # Parámetros:
    /// - `text`: Es el texto o contenido convertible en [`WidgetText`] que se mostrará en la interfaz
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>;

    /// Crea y configura un botón personalizado [`BlazeButton`]
    ///
    /// # Parámetros:
    /// -`text`: Etiqueta del botón convertible en [`WidgetText`]
    fn blaze_button<T: Into<WidgetText>>(&'a mut self, text: T) -> BlazeButton<'a>;

    /// Despliega un selector de color personalizado [`BlazeColorSelector`]
    ///
    /// # Parámetros:
    /// - `rgba`: Referencia mutable al valor del color [`Color32`] a modificar
    /// - `id_source`: Identificador único (que implementa [`Hash`] y [`Debug`]) para mantener el estado interno del widget
    fn color_picker<T: Into<&'a mut Color32>, U: Hash + Debug>(
        &'a mut self,
        rgba: T,
        id_source: U,
    ) -> BlazeColorSelector<'a>;

    /// Construye un campo de entrada de texto [`BlazeInput`]
    ///
    /// # Parámetros:
    /// - `input_hint_text`: texto de sugerencia de hot_keys
    /// - `hint_text`: Texto explicativo que se renderiza dentro del input cuando está vacío
    /// - `input`: referencia mutable a la cadena donde se almacenará la entrada escrita por el usuario
    fn blaze_input<T: Into<&'a mut String>>(
        &'a mut self,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        input: T,
    ) -> BlazeInput<'a>;

    /// Instancia un menú desplegable  [`BlazeComboBox`]
    ///
    /// # Parámetros:
    fn blaze_combobox(&'a mut self) -> BlazeComboBox<'a>;

    /// Inicia la construcción de la base para ventanas o menús emergentes [`BlazePopup`]
    fn blaze_popup(&'a mut self) -> BlazePopup<'a>;

    /// Crea un control deslizante para valores numéricos [`BlazeSlider`]
    ///
    /// # Parámetros:
    /// - `value`: Referencia mutable al valor numérico que se actualizará con el deslizador
    /// - `range`: Rango inclusivo que delimita los valores mínimos y máximos permitidos por el deslizador
    fn blaze_slider<Num: Numeric>(
        &'a mut self,
        value: &'a mut Num,
        range: impl Into<std::ops::RangeInclusive<Num>>,
    ) -> BlazeSlider<'a, Num>;

    fn draw_icon(&'a mut self, icon_data: (&'a str, impl Into<&'a [u8]>)) -> BlazeIcon<'a>;
}

impl<'a> UiExt<'a> for Ui {
    fn label_ns<T>(&mut self, text: T) -> Response
    where
        T: Into<WidgetText>,
    {
        label::label_ns(self, text)
    }

    fn blaze_button<T: Into<WidgetText>>(&'a mut self, text: T) -> BlazeButton<'a> {
        let cache_arc =
            self.data(|d| d.get_temp::<Arc<Mutex<IconCache>>>(egui::Id::new("icon_cache")));

        BlazeButton::new(self, text, cache_arc)
    }

    fn color_picker<T: Into<&'a mut Color32>, U: Hash + Debug>(
        &'a mut self,
        rgba: T,
        id_source: U,
    ) -> BlazeColorSelector<'a> {
        BlazeColorSelector::new(self, rgba, id_source)
    }

    fn blaze_input<T: Into<&'a mut String>>(
        &'a mut self,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        input: T,
    ) -> BlazeInput<'a> {
        let cache_arc =
            self.data(|d| d.get_temp::<Arc<Mutex<IconCache>>>(egui::Id::new("icon_cache")));
        BlazeInput::new(self, input, input_hint_text, hint_text, cache_arc)
    }

    fn blaze_combobox(&'a mut self) -> BlazeComboBox<'a> {
        let cache_arc =
            self.data(|d| d.get_temp::<Arc<Mutex<IconCache>>>(egui::Id::new("icon_cache")));
        BlazeComboBox::new(self, cache_arc)
    }

    fn blaze_popup(&'a mut self) -> BlazePopup<'a> {
        BlazePopup::new(self)
    }

    fn blaze_slider<Num: Numeric>(
        &'a mut self,
        value: &'a mut Num,
        range: impl Into<std::ops::RangeInclusive<Num>>,
    ) -> BlazeSlider<'a, Num> {
        BlazeSlider::new(self, value, range)
    }

    fn draw_icon(&'a mut self, icon_data: (&'a str, impl Into<&'a [u8]>)) -> BlazeIcon<'a> {
        let cache_arc = self.data(|d| {
            d.get_temp::<Arc<Mutex<IconCache>>>(egui::Id::new("icon_cache"))
                .expect("IconCache no inyectado en el frame")
        });
        BlazeIcon::new(self, icon_data, cache_arc)
    }
}
