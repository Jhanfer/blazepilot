use super::BlazeCoreState;
use crate::core::runtime::event_bus::with_event_bus;
use tracing::warn;

impl BlazeCoreState {
    fn focused_col_index(&self) -> Option<usize> {
        let motor = self.motor.borrow();
        let focused = motor.active_tab().focused.clone();
        self.miller_view
            .columns
            .iter()
            .position(|p| *p.as_ref() == *focused.as_ref())
    }

    pub fn current_search_filter(&self) -> &str {
        if self.is_miller() {
            self.focused_col_index()
                .and_then(|i| self.miller_view.search_filter.get(&i))
                .map(|s| s.as_str())
                .unwrap_or("")
        } else {
            &self.search_filter
        }
    }

    pub fn is_filter_empty(&self) -> bool {
        self.current_search_filter().is_empty()
    }

    pub fn clean_search(&mut self) {
        let focused = {
            let motor = self.motor.borrow();
            motor.active_tab().focused.clone()
        };

        {
            let mut motor = self.motor.borrow_mut();
            let tab = motor.active_tab_mut();

            if let Some(source) = tab.sources.iter_mut().find(|s| s.cwd == focused) {
                if let Err(e) = source.clear_recursive_files() {
                    warn!("Ha ocirrido un error: {}", e);
                }

                source.is_recursive_active = false;
            }
        }

        if self.is_miller() {
            if let Some(i) = self.focused_col_index() {
                self.miller_view.search_filter.remove(&i);
            }
        } else {
            self.search_filter.clear();
        }
    }

    pub fn set_search(&mut self, query: String) {
        let focused = {
            let motor = self.motor();
            motor.active_tab().focused.clone()
        };

        let was_recursive = self.current_search_filter().starts_with("rec:");
        let is_recursive = query.starts_with("rec:");

        if was_recursive && !is_recursive {
            self.clean_search();
            self.refresh();
        }

        if is_recursive {
            let clean = query.replacen("rec:", "", 1);
            if clean.len() >= 2 && clean != self.current_search_filter().replacen("rec:", "", 1) {
                let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

                let mut motor = self.motor.borrow_mut();
                let tab = motor.active_tab_mut();

                for source in tab.sources.iter_mut() {
                    if source.is_recursive_active {
                        let _ = source.clear_recursive_files();
                        source.is_recursive_active = false;
                    }
                }

                if let Some(source) = tab.sources.iter_mut().find(|s| s.cwd == focused)
                    && let Err(e) = source.start_recursive_search(clean, 30, dispatcher)
                {
                    warn!("Ha ocirrido un error: {}", e);
                }
            }
        }

        if self.is_miller() {
            if let Some(i) = self.focused_col_index() {
                self.miller_view.search_filter.insert(i, query);
            }
        } else {
            self.search_filter = query;
        }
    }
}
