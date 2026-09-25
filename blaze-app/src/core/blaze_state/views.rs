use std::{path::Path, sync::Arc, time::Instant};

use tracing::debug;

use crate::{
    core::{
        blaze_state::state_structs::{LayoutMode, ViewMode},
        bootstrap::configs::config_manager::with_configs,
        runtime::event_bus::with_event_bus,
    },
    ui::fonts::fonts_manager::with_fonts,
};

use super::BlazeCoreState;

impl BlazeCoreState {
    pub fn is_miller(&self) -> bool {
        matches!(self.view_mode, ViewMode::Normal(LayoutMode::Miller))
    }

    pub fn miller_reset_to(&mut self, path: Arc<Path>) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

        {
            let mut motor = self.motor_mut();
            let tab = motor.active_tab_mut();
            tab.sources.clear();
            tab.miller_future.clear();
            tab.focused = path.clone();
        }

        self.miller_view.columns.clear();
        self.miller_view.opened.clear();

        {
            let mut motor = self.motor_mut();
            let tab = motor.active_tab_mut();
            tab.ensure_source(path.clone());
            tab.request_load_from_path(path.clone(), dispatcher).ok();
        }

        self.save_caches(false);
        self.last_navigation_time = Some(Instant::now());
        with_fonts(|f| f.enter_dir(&path));
    }

    pub fn sync_columns_from_sources(&mut self) {
        let source_paths: Vec<Arc<Path>> = self
            .motor_mut()
            .active_tab()
            .sources
            .iter()
            .map(|s| Arc::clone(&s.cwd))
            .collect();

        let columns_match = self.miller_view.columns.len() == source_paths.len()
            && self
                .miller_view
                .columns
                .iter()
                .zip(source_paths.iter())
                .all(|(col, src)| **col == **src);

        if columns_match {
            return;
        }

        for (col_index, new_path) in source_paths.iter().enumerate() {
            let old_path = self.miller_view.columns.get(col_index);
            let changed = old_path.is_none_or(|old| **old != **new_path);
            if changed {
                self.miller_view.clear_opened(col_index);
            }
        }

        self.miller_view.columns.clear();
        for path in &source_paths {
            self.miller_view.push_column(Arc::clone(path));
        }

        debug!(
            "Columnas sincronizadas: {:?}",
            self.miller_view
                .columns
                .iter()
                .map(|c| c.display())
                .collect::<Vec<_>>()
        );
    }

    pub fn set_view_mode(&mut self, new_mode: ViewMode) {
        let old_mode = self.view_mode.clone();

        if matches!(old_mode, ViewMode::Normal(LayoutMode::Miller))
            && !matches!(new_mode, ViewMode::Normal(LayoutMode::Miller))
        {
            self.miller_view.columns.clear();
            self.miller_view.opened.clear();

            let focused = self.cwd();
            let mut motor = self.motor_mut();
            let tab = motor.active_tab_mut();
            tab.sources.retain(|s| s.cwd == focused);
            tab.miller_future.clear();
            tab.set_focus(focused);
        }

        if !matches!(old_mode, ViewMode::Normal(LayoutMode::Miller))
            && matches!(new_mode, ViewMode::Normal(LayoutMode::Miller))
        {
            self.miller_view.columns.clear();
            self.miller_view.opened.clear();
            self.sync_columns_from_sources();
        }

        // Se guarda en las configuraciones
        with_configs(|c| c.set_view_mode(new_mode.clone()));
        self.view_mode = new_mode;
    }
}
