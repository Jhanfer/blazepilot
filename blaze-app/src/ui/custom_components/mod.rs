use egui::{Color32, Response, Ui, WidgetText, emath::Numeric};
use std::{fmt::Debug, hash::Hash};

use crate::ui::{
    custom_components::{
        button::BlazeButton, color_picker::BlazeColorSelector, combobox::BlazeComboBox,
        input::BlazeInput, popup::BlazePopup, slider::BlazeSlider,
    },
    icons_cache::icon_cache::IconCache,
};

pub mod button;
pub mod color_picker;
pub mod combobox;
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
    /// - `icon_cache`: Pool de caché opcional para registrar o recuperar texturas de iconos y evitar rasterizar imágenes repetidas
    fn blaze_button<T: Into<WidgetText>>(
        &mut self,
        text: T,
        icon_cache: Option<&'a mut IconCache>,
    ) -> BlazeButton<'a>;

    /// Despliega un selector de color personalizado [`BlazeColorSelector`]
    ///
    /// # Parámetros:
    /// - `rgba`: Referencia mutable al valor del color [`Color32`] a modificar
    /// - `id_source`: Identificador único (que implementa [`Hash`] y [`Debug`]) para mantener el estado interno del widget
    fn color_picker<T: Into<&'a mut Color32>, U: Hash + Debug>(
        &mut self,
        rgba: T,
        id_source: U,
    ) -> BlazeColorSelector<'a>;

    /// Construye un campo de entrada de texto [`BlazeInput`]
    ///
    /// # Parámetros:
    /// - `input_hint_text`: texto de sugerencia de hot_keys
    /// - `hint_text`: Texto explicativo que se renderiza dentro del input cuando está vacío
    /// - `input`: referencia mutable a la cadena donde se almacenará la entrada escrita por el usuario
    /// - `icon_cache`: Pool de caché opcional para registrar o recuperar texturas de iconos y evitar rasterizar imágenes repetidas
    fn blaze_input<T: Into<&'a mut String>>(
        &mut self,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        input: T,
        icon_cache: Option<&'a mut IconCache>,
    ) -> BlazeInput<'a>;

    /// Instancia un menú desplegable  [`BlazeComboBox`]
    ///
    /// # Parámetros:
    /// - `icon_cache`: Pool de caché opcional para registrar o recuperar texturas de iconos y evitar rasterizar imágenes repetidas
    fn blaze_combobox(&mut self, icon_cache: Option<&'a mut IconCache>) -> BlazeComboBox<'a>;

    /// Inicia la construcción de la base para ventanas o menús emergentes [`BlazePopup`]
    fn blaze_popup(&mut self) -> BlazePopup;

    /// Crea un control deslizante para valores numéricos [`BlazeSlider`]
    ///
    /// # Parámetros:
    /// - `value`: Referencia mutable al valor numérico que se actualizará con el deslizador
    /// - `range`: Rango inclusivo que delimita los valores mínimos y máximos permitidos por el deslizador
    fn blaze_slider<Num: Numeric>(
        &mut self,
        value: &'a mut Num,
        range: impl Into<std::ops::RangeInclusive<Num>>,
    ) -> BlazeSlider<'a, Num>;
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
    ) -> BlazeColorSelector<'a> {
        BlazeColorSelector::new(self, rgba, id_source)
    }

    fn blaze_input<T: Into<&'a mut String>>(
        &mut self,
        input_hint_text: &'a str,
        hint_text: Option<&'a str>,
        input: T,
        icon_cache: Option<&'a mut IconCache>,
    ) -> BlazeInput<'a> {
        BlazeInput::new(self, input, input_hint_text, hint_text, icon_cache)
    }

    fn blaze_combobox(&mut self, icon_cache: Option<&'a mut IconCache>) -> BlazeComboBox<'a> {
        BlazeComboBox::new(self, icon_cache)
    }

    fn blaze_popup(&mut self) -> BlazePopup {
        BlazePopup::new(self)
    }

    fn blaze_slider<Num: Numeric>(
        &mut self,
        value: &'a mut Num,
        range: impl Into<std::ops::RangeInclusive<Num>>,
    ) -> BlazeSlider<'a, Num> {
        BlazeSlider::new(self, value, range)
    }
}
