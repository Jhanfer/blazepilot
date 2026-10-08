use std::{path::Path, sync::Arc};

use egui::{
    Align2, Area, Color32, CursorIcon, FontId, Frame, Id, Key, Order, Pos2, Rect, Response, Sense,
    Stroke, StrokeKind, Ui, pos2, vec2,
};

use crate::{
    core::{
        files::blaze_motor::motor_structs::FileEntry, runtime::event_bus::Dispatcher,
        system::disk_reader::disk::Disk,
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        modules::utilities::ensure_min_lightness,
        themes::{platform::structs::ToColor, theme_manager::with_theme},
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

    pub(crate) just_opened: bool,
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
            just_opened: false,
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
            self.just_opened = true;
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

        let screen = ui.content_rect();
        let pos = self.position;

        let area_response = Area::new(Id::new(menu_id))
            .order(Order::Foreground)
            .fixed_pos(pos)
            .constrain_to(screen)
            .show(ui, |ui| {
                Frame::new()
                    .fill(current_theme.components.panel.bg.to_color())
                    .corner_radius(12.0)
                    .stroke(Stroke::new(
                        0.5,
                        current_theme.components.panel.border.to_color(),
                    ))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_min_width(300.0);
                        ui.set_max_width(300.0);
                        callback(ui);
                    });
            });

        let menu_rect = area_response.response.rect;
        let clicked_outside = ui.input(|i| {
            i.pointer.primary_clicked()
                || i.pointer.secondary_clicked()
                    && i.pointer
                        .latest_pos()
                        .map(|p| !menu_rect.contains(p))
                        .unwrap_or(true)
        });

        if clicked_outside || ui.input(|i| i.key_pressed(Key::Escape)) {
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
        state: &mut BlazeUiState,
        rect: Rect,
        hint_pos: Pos2,
        popup_id: Id,
        mut callback: I,
    ) where
        I: FnMut(&mut Ui, &mut BlazeUiState),
    {
        let current_theme = with_theme(|t| t.current());
        let popup_size = vec2(120.0 + rect.size().x / 3.0, 80.0);
        let screen_rect = ui.content_rect();

        let anchor_x = hint_pos.x;
        let goes_right = anchor_x + popup_size.x + 15.0 < screen_rect.right();

        let popup_pos = if goes_right {
            pos2(anchor_x + 15.0, rect.top())
        } else {
            pos2(anchor_x - popup_size.x - 15.0, rect.top())
        };

        let popup_rect = Rect::from_min_size(popup_pos, popup_size);

        Area::new(popup_id.with("area"))
            .order(Order::TOP)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                Frame::popup(ui.style())
                    .fill(current_theme.components.panel.bg.to_color())
                    .stroke(Stroke::new(
                        0.5,
                        current_theme.components.panel.border.to_color(),
                    ))
                    .show(ui, |ui| {
                        ui.set_min_size(popup_size);
                        ui.set_max_size(popup_size);
                        callback(ui, state);
                    });
            });

        if ui.input(|i| i.pointer.any_click()) {
            let mouse_pos = ui.input(|i| i.pointer.interact_pos());
            if let Some(pos) = mouse_pos
                && !popup_rect.contains(pos)
                && !rect.contains(pos)
            {
                ui.memory_mut(|m| m.data.insert_temp(popup_id, false));
            }
        }
    }

    /// Renderiza los botones del menú contextual
    ///
    /// Gestiona su apariencia, estado e interacción, ejecutando
    /// acciones principales y, opcionalmente, mostrando un submenú
    /// contextual para acciones adicionales
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_context_button<J, I>(
        ui: &mut Ui,
        ui_state: &mut BlazeUiState,
        label: &str,
        hint: &str,
        icon: (&str, &[u8]),
        enabled: bool,
        mut callback_one: J,
        callback_two: Option<I>,
    ) where
        J: FnMut(),
        I: FnMut(&mut Ui, &mut BlazeUiState),
    {
        let current_theme = with_theme(|t| t.current());

        let (rect, response) = ui.allocate_exact_size(
            vec2(ui.available_width() - 2.0, 30.0),
            Sense::click_and_drag(),
        );

        let h_padding = 4.0;
        let paint_rect = rect.shrink2(vec2(h_padding, 0.0));

        let text_color = if enabled {
            current_theme.semantic.text_primary.to_color()
        } else {
            current_theme.semantic.text_secondary.to_color()
        };

        let popup_id = response.id.with("popup");
        let mut is_open = ui.memory_mut(|m| m.data.get_temp::<bool>(popup_id).unwrap_or(false));

        let (bg_color, icon_color) = if response.hovered() && enabled {
            ui.set_cursor_icon(CursorIcon::PointingHand);
            (
                current_theme.components.button.bg.to_color(),
                current_theme.components.button.label_hover.to_color(),
            )
        } else {
            (
                Color32::TRANSPARENT,
                current_theme.components.button.label_active.to_color(),
            )
        };

        ui.painter().rect_filled(paint_rect, 12.0, bg_color);

        let icon_size = vec2(16.0, 16.0);
        let padding = 8.0;

        let icon_pos = rect.left_top() + vec2(padding, (rect.height() - icon_size.x) / 2.0);
        let icon_rect = Rect::from_min_size(icon_pos, icon_size);

        let rounded_rect = Rect::from_min_max(
            pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
            pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
        );

        let icon = ui_state
            .icon_cache
            .get_or_load(ui, icon.0, icon.1, icon_color, icon_size);

        let painter = ui.painter();

        painter.image(
            icon.id(),
            rounded_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let text_pos = pos2(rounded_rect.right() + 6.0, rect.center().y);

        ui.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(14.0),
            text_color,
        );

        let mut hint_pos = pos2(rect.right() - padding, rect.center().y / 2.0);

        if !hint.is_empty() {
            let c = current_theme.semantic.accent_glow.to_color();

            let hint_stroke_color = Color32::from_rgba_unmultiplied(
                c.r().saturating_add(20),
                c.g().saturating_add(20),
                c.b().saturating_add(20),
                120,
            );

            let hint_color = Color32::from_rgba_unmultiplied(
                c.r().saturating_add(60),
                c.g().saturating_add(60),
                c.b().saturating_add(60),
                160,
            );

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
                current_theme.semantic.bg_container.to_color(),
                Stroke::new(1.0, hint_stroke_color),
                StrokeKind::Outside,
            );

            ui.painter().galley(hint_pos, hint_galley, hint_color);
        }

        if enabled {
            if response.clicked() {
                callback_one();
                is_open = false;
            }

            if response.secondary_clicked() {
                is_open = !is_open;
            }
        }

        ui.memory_mut(|m| m.data.insert_temp(popup_id, is_open));

        if let Some(mut callback) = callback_two
            && is_open
        {
            Self::render_small_context(
                ui,
                ui_state,
                rect,
                hint_pos,
                popup_id,
                |ui: &mut Ui, state: &mut BlazeUiState| callback(ui, state),
            )
        }
    }
}
