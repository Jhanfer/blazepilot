use super::BlazeCoreState;
use crate::core::{
    blaze_state::state_structs::NewItemType,
    bootstrap::configs::config_manager::with_configs,
    files::blaze_motor::{motor::BlazeMotor, motor_structs::FileEntry},
    runtime::{bus_structs::UiEvent, event_bus::with_event_bus},
    system::clipboard::global_clipboard::TOKIO_RUNTIME,
};
use std::{
    cell::{Ref, RefMut},
    path::Path,
    sync::{Arc, atomic::Ordering},
};
use tracing::{error, info, warn};

impl BlazeCoreState {
    pub fn motor_mut(&mut self) -> RefMut<'_, BlazeMotor> {
        self.motor.borrow_mut()
    }

    pub fn motor(&self) -> Ref<'_, BlazeMotor> {
        self.motor.borrow()
    }

    pub fn get_files_for(&mut self, path: &Arc<Path>) -> Vec<Arc<FileEntry>> {
        let search_filter = if self.is_miller() {
            let col_index = self.miller_view.columns.iter().position(|p| p == path);

            match col_index {
                Some(i) => self
                    .miller_view
                    .search_filter
                    .get(&i)
                    .map(|s| s.as_str())
                    .unwrap_or(""),
                None => "",
            }
        } else {
            &self.search_filter
        };

        let mut motor = self.motor.borrow_mut();
        let tab = motor.active_tab_mut();

        match tab.get_files_for(path, search_filter, self.needs_sort, &self.sizer_manager) {
            Ok(files) => {
                self.needs_sort = true;
                files
            }
            Err(e) => {
                warn!("Ha ocurrido un error obteniendo los archivos: {e}");
                vec![]
            }
        }
    }

    pub fn filesource_just_loaded(&mut self, path: &Arc<Path>) -> bool {
        let mut motor = self.motor.borrow_mut();
        let tab = motor.active_tab_mut();

        let Some(source) = tab.sources.iter_mut().find(|s| s.cwd == *path) else {
            return false;
        };

        source.files_just_loaded.swap(false, Ordering::Relaxed)
    }

    #[allow(unused)]
    pub fn clear_clipboard(&self) {
        match self.clipboard.clear() {
            Ok(_) => {
                info!("Se limpia el clipboard");
            }
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn copy(&self, files: &[Arc<FileEntry>]) {
        let cwd = self.cwd();
        let items = self.selected_as_entries(files);
        match self.clipboard.copy_items(items, cwd) {
            Ok(_) => {
                info!("Se copia");
            }
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn cut(&self, files: &[Arc<FileEntry>]) {
        let cwd = self.cwd();
        let items = self.selected_as_entries(files);
        match self.clipboard.cut_items(items, cwd) {
            Ok(_) => {
                info!("Se corta");
            }
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn rename(&self, file_name: &str) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
        if let Err(e) = self
            .clipboard
            .rename_file(file_name, &self.rename_buffer, &dispatcher)
        {
            error!("Error renombrando: {}", e);
        }
    }

    pub fn create_new(&self, nit: NewItemType) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

        let res = match nit {
            NewItemType::File => {
                self.clipboard
                    .create_new_file(&self.new_item_buffer, self.cwd(), &dispatcher)
            }
            NewItemType::Folder => {
                self.clipboard
                    .create_new_dir(&self.new_item_buffer, self.cwd(), &dispatcher)
            }
        };

        if let Err(e) = res {
            warn!("Ha ocurrido un error en el clipboard: {e}");
        }
    }

    pub fn move_to_trash(&mut self, items: Vec<(Box<str>, Arc<Path>)>) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
        match self.clipboard.move_to_trash(items, &dispatcher) {
            Ok(_) => {
                self.refresh();
            }
            Err(e) => warn!("Ha ocurridoun error al mover a papelera: {e}"),
        }
    }

    pub fn move_files(&mut self, sources: Vec<Arc<Path>>, dest: Arc<Path>) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

        match self.clipboard.move_files(sources, dest, &dispatcher) {
            Ok(_) => {}
            Err(e) => warn!("Ha ocurrido un error al mover: {e}"),
        }
    }

    pub fn paste(&mut self, path: Arc<Path>) {
        match self.clipboard.set_dest(path) {
            Ok(_) => {}
            Err(e) => {
                warn!("Eror en clipboard: {e}");
                return;
            }
        }

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

        match self.clipboard.paste(&dispatcher) {
            Ok(_) => {}
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn open_file_by_path(&mut self, path: Arc<Path>) {
        let manager_arc = self.file_opener_manager.clone();
        info!("intentando abrir");
        let mut manager = manager_arc.lock();
        match manager.open_file(path) {
            Ok(_) => {}
            Err(e) => {
                warn!("Ha fallado la apertura del archivo: {}", e);
            }
        }
    }

    pub fn open_file(&mut self, file: &Arc<FileEntry>) {
        let path = file.full_path.to_owned();
        let manager_arc = self.file_opener_manager.clone();
        info!("intentando abrir");
        let mut manager = manager_arc.lock();
        match manager.open_file(path) {
            Ok(_) => {}
            Err(e) => {
                warn!("Ha fallado la apertura del archivo: {}", e);
            }
        }
    }

    pub fn open_file_with(&mut self, file: &Arc<FileEntry>) {
        let path = file.full_path.to_owned();
        info!("intentando abrir");

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
        dispatcher.send(UiEvent::OpenWithSelector { path }).ok();
    }

    pub fn open_terminal_here(&self) {
        let cwd = self.cwd();

        let preferred_terminal = with_configs(|c| {
            if c.get_default_terminal().trim().is_empty() {
                None
            } else {
                Some(c.get_default_terminal())
            }
        });

        let tm_manager = self.terminal_manager.clone();
        TOKIO_RUNTIME.spawn(async move {
            let mut tm_manager = tm_manager.lock().await;
            if let Err(e) = tm_manager
                .request_open_terminal(&cwd, preferred_terminal)
                .await
            {
                error!("No se pudo abrir la terminal: {}", e);
            }
        });
    }
}
