use super::BlazeCoreState;
use crate::core::files::blaze_motor::{
    motor_structs::FileEntry, selection_state::SelectionManager,
};
use std::{path::Path, sync::Arc};

impl BlazeCoreState {
    fn with_selection_manager<J>(
        &mut self,
        j: impl FnOnce(&mut SelectionManager) -> J,
    ) -> Option<J> {
        let cwd = self.cwd();
        let mut motor = self.motor_mut();
        let source = motor
            .active_tab_mut()
            .sources
            .iter_mut()
            .find(|s| s.cwd == cwd)?;
        Some(j(&mut source.selection_manager))
    }

    fn with_selection_manager_ref<I>(&self, i: impl FnOnce(&SelectionManager) -> I) -> Option<I> {
        let cwd = self.cwd();
        let motor = self.motor();

        let source = motor.active_tab().sources.iter().find(|s| s.cwd == cwd)?;
        Some(i(&source.selection_manager))
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.with_selection_manager_ref(|m| m.is_selected(index))
            .unwrap_or(false)
    }

    pub fn is_selected_for(&self, path: &Arc<Path>, index: usize) -> bool {
        let motor = self.motor();
        let source = motor.active_tab().sources.iter().find(|s| *s.cwd == **path);

        match source {
            Some(s) => s.selection_manager.is_selected(index),
            None => false,
        }
    }

    pub fn deselect_all(&mut self) {
        self.with_selection_manager(|m| m.deselect_all());
    }

    pub fn toggle_select_all(&mut self, files_len: usize) {
        self.with_selection_manager(|m| m.toggle_select_all(files_len));
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        self.with_selection_manager(|m| m.select_range(start, end));
    }

    pub fn selected_count(&self, files_len: usize) -> usize {
        self.with_selection_manager_ref(|m| m.selected_count(files_len))
            .unwrap_or(0)
    }

    pub fn get_selected_paths(&self, files: &[Arc<FileEntry>]) -> Vec<(Box<str>, Arc<Path>)> {
        self.with_selection_manager_ref(|m| m.get_selected_paths(files))
            .unwrap_or(vec![])
    }

    pub fn selected_as_entries(&self, files: &[Arc<FileEntry>]) -> Vec<Arc<FileEntry>> {
        self.with_selection_manager_ref(|m| m.selected_as_entries(files))
            .unwrap_or(vec![])
    }

    pub fn resize_selection(&mut self, new_len: usize) {
        self.with_selection_manager(|m| m.resize_selection(new_len));
    }

    pub fn set_last_selected_index(&mut self, index: Option<usize>) {
        self.with_selection_manager(|m| m.set_last_selected_index(index));
    }

    pub fn last_selected_index(&self) -> Option<usize> {
        self.with_selection_manager_ref(|m| m.last_selected_index())?
    }

    pub fn select_all_mode(&self) -> bool {
        self.with_selection_manager_ref(|m| m.select_all_mode())
            .unwrap_or(false)
    }

    pub fn selection_anchor(&self) -> Option<usize> {
        self.with_selection_manager_ref(|m| m.selection_anchor())?
    }

    pub fn set_selection_anchor(&mut self, anchor: Option<usize>) {
        self.with_selection_manager(|m| m.set_selection_anchor(anchor));
    }

    pub fn set_selection(&mut self, index: usize, value: bool) {
        self.with_selection_manager(|m| m.set_selection(index, value));
    }
}
