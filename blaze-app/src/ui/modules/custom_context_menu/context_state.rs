use std::{path::Path, sync::Arc};

use egui::{Color32, FontId, Id, Order, Pos2, Rect, Response, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::{
    core::{
        files::blaze_motor::motor_structs::FileEntry, runtime::event_bus::Dispatcher,
        system::disk_reader::disk::Disk,
    },
    ui::{
        custom_components::UiExt,
        modules::utilities::ensure_min_lightness,
        themes::{platform::color_ops, theme_manager::with_theme},
    },
};

/// Representa el contexto desde el que se abrió el menú contextual
///
/// Permite distinguir entre archivos, fondos, papelera y el panel
/// de discos para determinar las acciones disponibles
#[derive(Default, PartialEq)]
pub enum ContextMenuKind {
    #[default]
    None,

    /// Modo de archivo
    FileNormal,

    /// Modo de archivo en papelera
    FileTrash,

    /// Modo de fondo
    BackgroundNormal,

    /// Modo de fondo en papelera
    BackgroundTrash,

    /// Panel de discos
    DrivesPanel,
}

#[derive(Default)]
pub struct ContextMenuState {
    pub open: bool,
    pub position: Pos2,
    pub kind: ContextMenuKind,
    pub source_path: Option<Arc<Path>>,
    pub target_file: Option<Arc<FileEntry>>,
    pub target_sender: Option<Dispatcher>,
    pub target_drive: Option<Disk>,

    /// Unicamnente para  modo miller
    pub target_col: Option<usize>,
    pub target_row_col: Option<usize>,
}

impl ContextMenuState {
    pub fn new() -> Self {
        Self {
            open: false,
            position: pos2(0.0, 0.0),
            kind: ContextMenuKind::None,
            source_path: None,
            target_file: None,
            target_sender: None,
            target_drive: None,
            target_col: None,
            target_row_col: None,
        }
    }

    /// Detecta las interacciones del ratón para abrir o cerrar
    /// el menú contextual
    ///
    /// Evita cerrarlo mientras un campo de texto mantiene el foco
    pub fn handle_response(&mut self, response: &Response) {
        // abre el menú
        if response.secondary_clicked() {
            self.open = true;
            self.position = response
                .ctx
                .input(|i| i.pointer.latest_pos())
                .unwrap_or_default();
            self.clear_targets();
        }

        // en caso de abierto, se cierra al detectar click en otra parte de la pantalla
        // o al escribir en un input
        if self.open {
            let text_focused = response.ctx.memory(|m| m.focused().is_some());
            if !text_focused {
                let clicked_elsewhere =
                    response.ctx.input(|i| i.pointer.primary_clicked()) || response.drag_started();
                if clicked_elsewhere {
                    self.open = false;
                }
            }
        }
    }

    /// Limpia el contexto asociado al elemento seleccionado
    fn clear_targets(&mut self) {
        self.kind = ContextMenuKind::None;
        self.target_file = None;
        self.target_sender = None;
    }

    /// Cierra el menú contextual y limpia su contexto
    pub fn close(&mut self) {
        self.open = false;
        self.clear_targets();
    }

    /// Renderiza el menú contextual en la posición donde fue abierto
    ///
    /// Ejecuta el callback encargado de construir sus acciones y
    /// cierra el menú al hacer clic fuera de él o pulsar `Esc`
    pub(crate) fn show_menu<F>(&mut self, ui: &mut Ui, menu_id: &str, mut callback: F)
    where
        F: FnMut(&mut Ui),
    {
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.context_menu.palette(&current_theme.global);

        let id = Id::new(menu_id);
        let state_id = id.with("picker_state");

        ui.data_mut(|d| d.insert_temp(state_id, self.open));

        ui.blaze_popup()
            .id(id)
            .background_color(palette.bg_panel)
            .radius(20.0)
            .stroke_color(palette.border_panel)
            .stroke_width(current_theme.global.panel_stroke_width)
            .pos(self.position)
            .inner_margin(8.0)
            .auto_size(true)
            .show(|ui| {
                ui.set_min_width(300.0);
                ui.set_max_width(300.0);
                callback(ui);
            });

        if !ui.data(|d| d.get_temp::<bool>(state_id).unwrap_or(false)) {
            self.open = false;
        }
    }

    /// Renderiza un pequeño submenú contextual asociado al botón
    ///
    /// Posiciona el submenú junto al botón, adaptando su ubicación
    /// para evitar que se salga de los límites de la ventana, y
    /// gestiona su cierre al hacer clic fuera de él
    pub(crate) fn render_small_context<I>(
        ui: &mut Ui,
        rect: Rect,
        hint_pos: Pos2,
        popup_id: Id,
        mut callback: I,
    ) where
        I: FnMut(&mut Ui),
    {
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.context_menu.palette(&current_theme.global);
        let popup_size = vec2(120.0 + rect.size().x / 3.0, 80.0);
        let screen_rect = ui.content_rect();

        let anchor_x = hint_pos.x;
        let goes_right = anchor_x + popup_size.x + 15.0 < screen_rect.right();

        let popup_pos = if goes_right {
            pos2(anchor_x + 15.0, rect.top())
        } else {
            pos2(anchor_x - popup_size.x - 15.0, rect.top())
        };

        let id = popup_id.with("area");
        let state_id = id.with("picker_state");
        let menu_margin = ui.style().spacing.menu_margin.left as f32;

        ui.data_mut(|d| d.insert_temp(state_id, true));

        ui.blaze_popup()
            .id(id)
            .order(Order::TOP)
            .background_color(palette.bg_panel)
            .radius(20.0)
            .stroke_color(palette.border_panel)
            .stroke_width(current_theme.global.panel_stroke_width)
            .pos(popup_pos)
            .inner_margin(menu_margin)
            .auto_size(true)
            .show(|ui| {
                ui.set_min_size(popup_size);
                ui.set_max_size(popup_size);
                callback(ui);
            });

        // sincroniza el cierre del submenu con el botón
        if !ui.data(|d| d.get_temp::<bool>(state_id).unwrap_or(false)) {
            ui.memory_mut(|m| m.data.insert_temp(popup_id, false));
        }
    }

    /// Renderiza los botones del menú contextual
    ///
    /// Gestiona su apariencia, estado e interacción, ejecutando
    /// acciones principales y, opcionalmente, mostrando un submenú
    /// contextual para acciones adicionales
    pub(crate) fn render_context_button<J, I>(
        ui: &mut Ui,
        label: &str,
        hint: &str,
        icon: (&str, &[u8]),
        enabled: bool,
        mut callback_one: J,
        callback_two: Option<I>,
    ) where
        J: FnMut(),
        I: FnMut(&mut Ui),
    {
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.context_menu.palette(&current_theme.global);

        let text_color = if enabled {
            palette.fg_primary
        } else {
            palette.fg_secondary
        };

        let popup_id = ui.id().with("popup");
        let mut is_open = ui.memory_mut(|m| m.data.get_temp::<bool>(popup_id).unwrap_or(false));

        let padding = 8.0;

        let icon_color = if enabled {
            palette.icon_primary
        } else {
            palette.icon_secondary
        };

        let av_width = ui.available_width();
        let resp = ui
            .blaze_button(label)
            .set_is_active(enabled)
            .size((av_width - 2.0, 30.0))
            .radius(20.0)
            .text_color(text_color)
            .background_color(Color32::TRANSPARENT)
            .hover_background_color(palette.bg_hover)
            .stroke_color(Color32::TRANSPARENT)
            .icon(icon.0, icon.1)
            .icon_size(16_f32)
            .icon_color(icon_color)
            .hover_icon_color(palette.icon_hover)
            .show();

        if resp.clicked() {
            callback_one();
            is_open = false;
        }

        if resp.secondary_clicked() {
            is_open = !is_open;
        }

        let rect = resp.rect;

        let mut hint_pos = pos2(rect.right() - padding, rect.center().y / 2.0);

        if !hint.is_empty() {
            let c = palette.accent;
            let hint_bg = palette.bg_elevated;

            let hint_stroke_color = color_ops::mix(hint_bg, c, 0.45);
            let hint_color = color_ops::ensure_contrast(c, hint_bg, 3.0);

            let hint_galley = ui.painter().layout_no_wrap(
                hint.to_string(),
                FontId::proportional(10.0),
                ensure_min_lightness(hint_color),
            );

            let hint_size = hint_galley.size();
            let hint_width = hint_galley.size().x;

            hint_pos = pos2(
                rect.right() - padding - hint_width - 10.0,
                rect.center().y - hint_galley.size().y / 2.0,
            );

            let pad = vec2(4.0, 2.0);
            let hint_rect = Rect::from_min_size(hint_pos, hint_size).expand2(pad);

            ui.painter().rect(
                hint_rect,
                3.0,
                current_theme.global.bg_main(),
                Stroke::new(0.8, hint_stroke_color),
                StrokeKind::Middle,
            );

            ui.painter().galley(hint_pos, hint_galley, hint_color);
        }

        ui.memory_mut(|m| m.data.insert_temp(popup_id, is_open));

        if let Some(mut callback) = callback_two
            && is_open
        {
            Self::render_small_context(ui, rect, hint_pos, popup_id, |ui: &mut Ui| callback(ui))
        }
    }
}
