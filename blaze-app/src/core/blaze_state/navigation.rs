use std::{path::Path, sync::Arc, time::Instant};

use tracing::warn;

use crate::{
    core::{
        blaze_state::state_structs::{LayoutMode, ViewMode},
        files::blaze_motor::tab_state::BlazeTabState,
        runtime::event_bus::{Dispatcher, with_event_bus},
        system::sizer_manager::manager::SizerMessages,
    },
    ui::fonts::fonts_manager::with_fonts,
};

use super::BlazeCoreState;

impl BlazeCoreState {
    pub fn cwd(&self) -> Arc<Path> {
        self.motor.borrow().active_tab().focused.clone()
    }

    pub fn im_navigating(&self) -> bool {
        if let Some(time) = self.last_navigation_time {
            time.elapsed() < self.navigation_cooldown
        } else {
            false
        }
    }

    pub fn refresh(&mut self) {
        self.clean_search();
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
        dispatcher.send(SizerMessages::CancelAll).ok();

        {
            let mut motor = self.motor.borrow_mut();
            let tab = motor.active_tab_mut();

            if !matches!(&self.view_mode, ViewMode::Normal(LayoutMode::Miller))
                && let Err(e) = tab.request_load_from_path(tab.focused.clone(), dispatcher.clone())
            {
                warn!("Ha ocurrido un error al cargar los archivos: {}", e);
            }
        }

        self.dir_sizes.reset();
        self.extended_info.reset();
        self.fonts.reset();
        self.deselect_all();
    }

    pub fn reload_all(&mut self) {
        self.clean_search();
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
        dispatcher.send(SizerMessages::CancelAll).ok();

        {
            let mut motor = self.motor.borrow_mut();
            let tab = motor.active_tab_mut();
            for source in &mut tab.sources {
                match source.load_path(dispatcher.clone()) {
                    Ok(()) => {}
                    Err(e) => warn!(
                        "Ha ocurrido un error al cargar path: {} : {e}",
                        source.cwd.display()
                    ),
                }
            }
        };

        self.dir_sizes.reset();
        self.extended_info.reset();
        self.fonts.reset();
        self.deselect_all();
    }

    pub fn navigate_to(&mut self, path: Arc<Path>) {
        let prev_dir = self.cwd();

        with_fonts(|f| f.leave_directory(&prev_dir));

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
        dispatcher.send(SizerMessages::CancelAll).ok();
        self.dir_sizes.clear();
        self.extended_info_manager.clear_directory(&prev_dir);

        self.motor
            .borrow_mut()
            .active_tab_mut()
            .navigate_to(path.clone(), dispatcher)
            .ok();
        self.save_caches(false);
        self.last_navigation_time = Some(Instant::now());

        with_fonts(|f| f.enter_dir(&path));
    }

    pub fn navigate_to_from_col(&mut self, path: Arc<Path>, col_index: usize) {
        let prev_dir = self.cwd();
        with_fonts(|f| f.leave_directory(&prev_dir));

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
        dispatcher.send(SizerMessages::CancelAll).ok();
        self.dir_sizes.reset();
        self.extended_info_manager.clear_directory(&prev_dir);

        self.motor
            .borrow_mut()
            .active_tab_mut()
            .miller_navigate_to(path.clone(), col_index, dispatcher)
            .ok();

        self.save_caches(false);
        self.last_navigation_time = Some(Instant::now());
        with_fonts(|f| f.enter_dir(&path));

        self.extended_info.reset();
        self.fonts.reset();
        self.deselect_all();
    }

    fn navigate_with<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut BlazeTabState, Dispatcher) -> Result<(), ()>,
    {
        let prev_dir = self.cwd();
        with_fonts(|f| f.leave_directory(&prev_dir));

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

        {
            let mut binding = self.motor.borrow_mut();
            let tab = binding.active_tab_mut();
            callback(tab, dispatcher.clone()).ok();
        }

        if !self.is_miller() {
            let new_cwd = self.cwd();

            if let Some(pos) = self
                .miller_view
                .columns
                .iter()
                .position(|p| **p == *new_cwd)
            {
                self.miller_view.truncate_to(pos + 1);
            } else {
                self.miller_view.columns.clear();
                self.miller_view.opened.clear();
                self.miller_view.push_column(new_cwd.clone());
                self.miller_view.set_opened(0, 0);
            }
        }

        self.extended_info_manager.clear_directory(&prev_dir);
        dispatcher.send(SizerMessages::CancelAll).ok();
        with_fonts(|f| f.enter_dir(&self.cwd()));
        self.save_caches(false);
    }

    pub fn up(&mut self) {
        if self.is_miller() {
            self.navigate_with(|tab, dispatcher| {
                tab.miller_up(dispatcher);
                Ok(())
            });
        } else {
            self.navigate_with(|tab, dispatcher| {
                tab.up(dispatcher);
                Ok(())
            });
        }
    }

    pub fn back(&mut self) {
        if self.is_miller() {
            self.navigate_with(|tab, _dispatcher| {
                tab.miller_back();
                Ok(())
            });
            self.sync_columns_from_sources();
        } else {
            self.navigate_with(|tab, dispatcher| {
                tab.back(dispatcher);
                Ok(())
            });
        }
    }

    pub fn forward(&mut self) {
        if self.is_miller() {
            self.navigate_with(|tab, dispatcher| {
                tab.miller_forward(dispatcher);
                Ok(())
            });
            self.sync_columns_from_sources();
        } else {
            self.navigate_with(|tab, dispatcher| {
                tab.forward(dispatcher);
                Ok(())
            });
        }
    }

    pub fn can_go_up(&self) -> bool {
        if self.is_miller() {
            self.motor.borrow().active_tab().miller_can_go_up()
        } else {
            self.motor.borrow().active_tab().can_go_up()
        }
    }

    pub fn can_go_back(&self) -> bool {
        if self.is_miller() {
            self.motor.borrow().active_tab().miller_can_go_back()
        } else {
            self.motor.borrow().active_tab().can_go_back()
        }
    }

    pub fn can_go_forward(&self) -> bool {
        if self.is_miller() {
            self.motor.borrow().active_tab().miller_can_go_forward()
        } else {
            self.motor.borrow().active_tab().can_go_forward()
        }
    }
}
