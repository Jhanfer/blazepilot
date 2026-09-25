use super::BlazeCoreState;
use std::path::Path;

impl BlazeCoreState {
    pub fn active_id(&self) -> uuid::Uuid {
        self.motor.borrow().active_tab().id
    }

    pub fn switch_to_tab(&mut self, index: usize) {
        {
            let mut motor = self.motor_mut();
            motor.switch_to_tab(index);
        }
        if self.is_miller() {
            self.sync_columns_from_sources();
        }
    }

    pub fn next_tab(&mut self) {
        {
            let mut motor = self.motor_mut();
            motor.next_tab();
        }
        if self.is_miller() {
            self.sync_columns_from_sources();
        }
    }

    pub fn prev_tab(&mut self) {
        {
            let mut motor = self.motor_mut();
            motor.prev_tab();
        }
        if self.is_miller() {
            self.sync_columns_from_sources();
        }
    }

    pub fn close_tab(&mut self, index: usize) -> bool {
        let closed = {
            let mut motor = self.motor_mut();
            motor.close_tab(index).is_ok()
        };
        self.refresh();
        if self.is_miller() {
            self.sync_columns_from_sources();
        }
        closed
    }

    pub fn add_tab_from_file(&mut self, tab_path: &Path) {
        {
            let mut motor = self.motor_mut();
            motor.add_tab(tab_path);
        }
        self.refresh();
        if self.is_miller() {
            self.sync_columns_from_sources();
        }
    }

    pub fn create_tab(&mut self) {
        {
            let mut motor = self.motor_mut();
            motor.create_tab();
        }
        if self.is_miller() {
            self.sync_columns_from_sources();
        }
    }

    pub fn tab_title(&mut self, index: usize) -> String {
        self.motor_mut().tab_title(index)
    }
}
