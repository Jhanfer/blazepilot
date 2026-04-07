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





use std::sync::Arc;
use tracing::{debug, error, info, instrument::WithSubscriber, trace, warn};
use crate::core::system::disk_reader::disk::Disk;
use tokio::sync::{Mutex as TokioMutex};



#[cfg(target_os = "linux")]
use crate::core::system::disk_reader::platform::linux::LinuxDisks;

#[cfg(target_os = "linux")]
enum PlatformDisks {
    Linux(Arc<TokioMutex<LinuxDisks>>),
}

#[cfg(not(target_os = "linux"))]
pub enum PlatformDisks {}

pub struct DiskManager {
    #[cfg(target_os = "linux")]
    disks: PlatformDisks,
    watcher_started: bool,
}

impl DiskManager {
    pub async fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            info!("Iniciando DiskManager Linux");
            Self {
                disks: PlatformDisks::Linux(TokioMutex::new(LinuxDisks::init().await).into()),
                watcher_started: false,
            }

        }

        #[cfg(not(target_os = "linux"))]
        {
            debug!("DiskManager no soportado en esta plataforma");
            Self {}
        }
    }


    #[cfg(target_os = "linux")]
    pub async fn start_watcher_linux(&mut self, manager_arc: Arc<TokioMutex<DiskManager>>) -> anyhow::Result<()> {
        if self.watcher_started { info!("Disk Watcher inciado."); return Ok(()); }
        info!("Inciando watcher");
        
        let PlatformDisks::Linux(mutex_arc) = &self.disks;
        let linux_disks = mutex_arc.lock().await;
        linux_disks.start_disk_watcher(manager_arc).await?;

        self.watcher_started = true;

        Ok(())
    }

    /// Obtiene la lista actual de discos/particiones
    #[cfg(target_os = "linux")]
    pub async fn load_disks(&mut self) {
        match &mut self.disks {
            PlatformDisks::Linux(disks) => {
                disks.lock().await.load_disks().await;
            }
        }
    }

    /// Obtiene la lista de discos disponibles
    #[cfg(target_os = "linux")]
    pub async fn get_partitions(&self) -> Vec<Disk> {
        match &self.disks {
            PlatformDisks::Linux(disks) => {
                disks.lock().await.get_partitions()
            }
        }
    }

    /// Monta un disco específico
    #[cfg(target_os = "linux")]
    pub async fn mount_disk(&mut self, disk: &Disk, ) -> anyhow::Result<String> {
        match &mut self.disks {
            PlatformDisks::Linux(disks) => {
                let result = disks.lock().await.mount(disk).await;
                result
            }
        }
    }

    /// Desmonta un disco específico
    #[cfg(target_os = "linux")]
    pub async fn unmount_disk(&mut self, disk: &Disk) -> anyhow::Result<String> {
        match &mut self.disks {
            PlatformDisks::Linux(disks) => {
                let result = disks.lock().await.unmount(disk).await;
                result
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub async fn eject_disk(&mut self, disk: &Disk, ) -> anyhow::Result<String> {
        match &mut self.disks {
            PlatformDisks::Linux(disks) => {
                let result = disks.lock().await.eject(disk).await;
                result
            }
        }
    }

    /// Verifica si hay discos USB removibles
    #[cfg(target_os = "linux")]
    pub fn has_removable_disks(&self) -> bool {
        match &self.disks {
            PlatformDisks::Linux(disks) => {
                let partitions = disks.blocking_lock().get_partitions();
                partitions.iter().any(|d| d.is_removable)
            }
        }
    }

    /// Obtiene solo discos removibles (USB)
    #[cfg(target_os = "linux")]
    pub async fn get_removable_disks(&self) -> Vec<Disk> {
        match &self.disks {
            PlatformDisks::Linux(disks) => {
                let partitions = disks.lock().await.get_partitions();
                partitions.into_iter()
                    .filter(|d| d.is_removable)
                    .collect()
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn load_disks(&mut self) {
        warn!("Gestión de discos no soportada en esta plataforma");
    }

    #[cfg(not(target_os = "linux"))]
    pub fn get_partitions(&self) -> Vec<Disk> {
        vec![]
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn mount_disk(&mut self, _disk: &Disk, _ui: &slint::Weak<BlazeApp>) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("Gestión de discos no soportada en esta plataforma"))
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn unmount_disk(&mut self, _disk: &Disk, _ui: &slint::Weak<BlazeApp>) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("Gestión de discos no soportada en esta plataforma"))
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn refresh_disks(&mut self, _ui: &slint::Weak<BlazeApp>) {
        warn!("Gestión de discos no soportada en esta plataforma");
    }

    #[cfg(not(target_os = "linux"))]
    pub fn has_removable_disks(&self) -> bool {
        false
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn get_removable_disks(&self) -> Vec<Disk> {
        vec![]
    }
}
