// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.




use uuid::Uuid;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;
use std::vec;
use tracing::error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::files::blaze_motor::error::MotorResult;
use crate::core::system::disk_reader::disk_manager::DiskManager;
use crate::core::runtime::event_bus::with_event_bus;
use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;
use crate::core::files::blaze_motor::tab_state::{BlazeTabState, BlazeTabBuilder};




#[must_use = "Llama .build() para construir motor"]
pub struct BlazeMotorBuilder {
    start_path: Option<Arc<Path>>,
}


impl BlazeMotorBuilder {
    pub fn new() -> Self {
        Self {
            start_path: None,
        }
    }

    pub fn with_start_path(mut self, path: Option<Arc<Path>>) -> Self {
        self.start_path = path;
        self
    }

    #[must_use]
    pub async fn build(self) -> BlazeMotor {
        let tab_id = Uuid::new_v4();

        let path = if let Some(init_path) = self.start_path {
            init_path
        } else {
            KnownDirsManager::get().home.clone()
        };

        let fist_tab = BlazeTabBuilder::default()
            .with_start_path(path)
            .with_uuid(tab_id)
            .build();

        let disk_manager = Arc::new(tokio::sync::Mutex::new(DiskManager::new().await));

        {
            let mut mgr_guard = disk_manager.lock().await;
            mgr_guard.load_disks().await;
            if let Err(e) = mgr_guard.start_watcher_linux(disk_manager.clone()).await {
                error!("Error inicializando watcher de discos: {}", e);
            }
        }

        BlazeMotor {
            tabs: vec![fist_tab],
            active_tab_index: 0,
            disk_manager,
            limit: 50,
        }
    }
}

impl Default for BlazeMotorBuilder {
    fn default() -> Self {
        Self::new()
    }
}



pub struct BlazeMotor {
    pub tabs: Vec<BlazeTabState>,
    pub active_tab_index: usize,
    pub disk_manager: Arc<Mutex<DiskManager>>, 
    pub limit: usize,
}


thread_local! {
    pub static MOTOR: RefCell<Option<Rc<RefCell<BlazeMotor>>>> = RefCell::new(None);
}
pub fn with_motor<F, R>(f: F) -> R 
    where F: FnOnce(&mut BlazeMotor) -> R {
        MOTOR.with(|m|{
            let motor_rc = m.borrow()
                .as_ref()
                .expect("Motor no inicializado")
                .clone();

            let mut motor = motor_rc.borrow_mut();
            f(&mut *motor)
        })
    }


impl BlazeMotor {
    fn set_active_index(&mut self, index: usize) {
        self.active_tab_index = index;
        crate::core::runtime::event_bus::set_active_tab(self.tabs[index].id);
    }


    pub fn active_tab_mut(&mut self) -> &mut BlazeTabState {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn active_tab(&self) -> &BlazeTabState {
        &self.tabs[self.active_tab_index]
    }

    pub fn switch_to_tab(&mut self, index:usize) {
        if index < self.tabs.len() {
            self.set_active_index(index);
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() || self.tabs.len() <= 1 {
            return;
        }
        self.set_active_index((self.active_tab_index + 1) % self.tabs.len());
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() || self.tabs.len() <= 1 {
            return;
        }
        let next= if self.active_tab_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab_index - 1
        };
        self.set_active_index(next);
    }
    
    fn remove_channels(&self, tab_id: Uuid) {
        with_event_bus(|pool| {
            pool.remove_tab(tab_id)
        });
    }

    pub fn close_tab(&mut self, index:usize) -> MotorResult<bool> {
        if self.tabs.len() <= 1 {
            return Ok(false);
        }

        let tab_id = self.tabs[index].id;

        {
            let tab = &mut self.tabs[index];
            tab.watcher.stop_watching();

            tab.reset_for_new_path()?;

            tab.history.clear();
            tab.future.clear();
        }

        self.remove_channels(tab_id);

        self.tabs.remove(index);
        if self.active_tab_index >= self.tabs.len() {
            self.set_active_index(self.tabs.len() - 1);
        }

        Ok(true)
    }

    fn start_tab_load(&mut self, index: usize) {
        let tab = &self.tabs[index];
        let tab_id = tab.id;
        let sender = with_event_bus(|pool| pool.dispatcher(tab_id));
        self.tabs[index].load_path(false, sender).ok();
    }

    pub fn add_tab(&mut self, tab_path: &Path) -> Option<Uuid> {
        if self.tabs.len() >= self.limit {
            return None;
        }
        let tab_id = Uuid::new_v4();
        let new_tab = BlazeTabBuilder::default()
            .with_start_path(tab_path.into())
            .with_uuid(tab_id)
            .build();

        let insert_index = self.active_tab_index + 1;
        self.tabs.insert(insert_index, new_tab);
        
        self.set_active_index(insert_index);

        self.start_tab_load(self.active_tab_index);

        Some(tab_id)
    }

    pub fn create_tab(&mut self) -> Option<Uuid> {
        if self.tabs.len() >= self.limit {
            return None;
        }
        let path = &KnownDirsManager::get().home;
        let tab_id = Uuid::new_v4();
        
        let new_tab = BlazeTabBuilder::default()
            .with_start_path(path.to_owned())
            .with_uuid(tab_id)
            .build();

        let insert_index = self.active_tab_index + 1;
        self.tabs.insert(insert_index, new_tab);

        self.set_active_index(insert_index);

        self.start_tab_load(self.active_tab_index);

        Some(tab_id)
    }

    pub fn tab_title(&self, index:usize) -> String {
        self.tabs.get(index)
        .and_then(|tab|tab.cwd.file_name())
        .and_then(|name|name.to_str())
        .unwrap_or("Home")
        .to_owned()
    }

}




#[cfg(test)]
mod tests {
    use crate::core::{files::{blaze_motor::{error::MotorError, motor_structs::{FileEntry, FileKind}}, file_extension::FileExtension}, system::{clipboard::clipboard::TOKIO_RUNTIME, trash_manager::trash_manager::init_trash_backend}};

    use super::*;
    use std::{sync::atomic::Ordering, time::Duration};

    fn init_dir_trash() -> Result<(), Box<dyn std::error::Error>> {
        KnownDirsManager::init();
        init_trash_backend()?;
        Ok(())
    }


    fn make_tab(path: Arc<Path>) -> BlazeTabState {
        if let Err(e) = init_dir_trash() {
            println!("El backend de la papelera ya se encuentra activo: {}", e);
        }

        let id = Uuid::new_v4();
        BlazeTabBuilder::default()
            .with_start_path(path)
            .with_uuid(id)
            .build()
    }

    #[test]
    fn test_stop_watching_does_not_leave_handle() {
        let mut tab = make_tab(std::env::temp_dir().into());
        // Simular que tenía un handle activo
        let handle = TOKIO_RUNTIME.spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
        tab.watcher.watching_handle = Some(handle);
        tab.watcher.watching.store(true, Ordering::Relaxed);

        tab.watcher.stop_watching();

        assert!(!tab.watcher.watching.load(Ordering::Relaxed), "watching debe ser false");
        assert!(tab.watcher.watching_handle.is_none(), "handle debe haberse consumido");
        assert!(tab.watcher.watcher.is_none(), "watcher debe ser None");
    }

    #[test]
    fn test_cancel_loading_drains_handles() {
        let mut tab = make_tab(std::env::temp_dir().into());
        let h1 = TOKIO_RUNTIME.spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
        let h2 = TOKIO_RUNTIME.spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
        tab.loader.handles.push(h1);
        tab.loader.handles.push(h2);
        tab.loading_flag.store(false, Ordering::Relaxed);

        tab.loader.cancel();

        assert!(tab.loader.handles.is_empty(), "handles deben haberse drenado");
        assert!(!tab.loading_flag.load(Ordering::Relaxed), "flag debe ser false");
    }

    #[test]
    fn test_close_tab_clears_all_memory() -> MotorResult<()> {
        if let Err(e) = init_dir_trash() {
            println!("El backend de la papelera ya se encuentra activo: {}", e);
        }

        let mut motor = TOKIO_RUNTIME.block_on(
            BlazeMotorBuilder::default().build()
        );
        // Añadir un segundo tab para poder cerrar el primero
        motor.add_tab(&std::env::temp_dir());
        assert_eq!(motor.tabs.len(), 2);

        // Llenar datos en tab 0

        {
            let mut file_guard = motor.tabs[0].files.write()
                .map_err(|_| MotorError::PoisonedLock)?;
            // simular con vec vacío, basta para el test
            file_guard.push(
                Arc::new(
                FileEntry {
                        name: "".into(),
                        full_path: Path::new("").into(),
                        extension: FileExtension::Unknown,
                        kind: FileKind::File,
                        size: 0,
                        modified: 0,
                        created: 0,
                        is_hidden: false,
                        unique_id: None,
                        accessed: 0,
                        permissions: 0,
                        inode: 0,
                        nlink: 0,
                        device: 0,
                    }
                )
            );
        }

        let closed = motor.close_tab(0)?;

        assert!(closed);
        assert_eq!(motor.tabs.len(), 1);

        Ok(())
    }

    #[test]
    fn test_close_tab_refuses_last_tab() -> MotorResult<()> {
        if let Err(e) = init_dir_trash() {
            println!("El backend de la papelera ya se encuentra activo: {}", e);
        }

        let mut motor = TOKIO_RUNTIME.block_on(
            BlazeMotorBuilder::default().build()
        );
        assert_eq!(motor.tabs.len(), 1);

        let result = motor.close_tab(0)?;
        assert!(!result, "no debe permitir cerrar el último tab");
        assert_eq!(motor.tabs.len(), 1);

        Ok(())
    }

    #[test]
    fn test_active_tab_index_clamps_after_close() -> MotorResult<()> {
        if let Err(e) = init_dir_trash() {
            println!("El backend de la papelera ya se encuentra activo: {}", e);
        }

        let mut motor = TOKIO_RUNTIME.block_on(
            BlazeMotorBuilder::default().build()
        );
        motor.add_tab(&std::env::temp_dir());
        motor.add_tab(&std::env::temp_dir());
        // tabs: [0, 1, 2], active = 2
        motor.active_tab_index = 2;

        motor.close_tab(2)?; // cierra el activo (el último)

        assert_eq!(motor.active_tab_index, 1, "debe apuntar al nuevo último tab");

        Ok(())
    }

    #[test]
    fn test_watcher_task_exits_on_watcher_drop() {
        // Verificar que la task del watcher termina sola cuando se dropea el watcher
        let mut tab = make_tab(std::env::temp_dir().into());
        let cwd = tab.cwd;
        let sender = with_event_bus(|pool| pool.dispatcher(tab.id));

        tab.watcher.start_watching(cwd, sender).ok();
        assert!(tab.watcher.watching_handle.is_some());

        // stop_watching dropea el watcher → fs_tx se cierra → task termina
        tab.watcher.stop_watching();

        // Dar tiempo a la task para terminar (Disconnected break)
        std::thread::sleep(Duration::from_millis(200));

        // El handle fue abortado/tomado por stop_watching
        assert!(tab.watcher.watching_handle.is_none());
    }


    fn two_distinct_dirs() -> (Arc<Path>, Arc<Path>) {
        let base = std::env::temp_dir();
        let a = base.join("blaze_test_a");
        let b = base.join("blaze_test_b");
        std::fs::create_dir_all(&a).ok();
        std::fs::create_dir_all(&b).ok();
        (a.into(), b.into())
    }


    #[test]
    fn test_navigate_to_updates_history() {
        let (start, other) = two_distinct_dirs();
        let mut tab = make_tab(start.to_owned());

        tab.navigate_to(other.to_owned());

        assert_eq!(tab.cwd, other);
        assert!(tab.history.contains(&start));
        assert!(tab.future.is_empty());

        if start.exists() {
            let _ = std::fs::remove_dir(start);
        }
        if other.exists() {
            let _ = std::fs::remove_dir(other);
        }
    }

    #[test]
    fn test_back_and_forward() {
        let (start, other) = two_distinct_dirs();
        let mut tab = make_tab(start.to_owned());

        tab.navigate_to(other.to_owned());
        tab.back();

        assert_eq!(tab.cwd, start);
        assert!(!tab.future.is_empty());

        tab.forward();
        assert_eq!(tab.cwd, other);

        if start.exists() {
            let _ = std::fs::remove_dir(start);
        }
        if other.exists() {
            let _ = std::fs::remove_dir(other);
        }
    }
}