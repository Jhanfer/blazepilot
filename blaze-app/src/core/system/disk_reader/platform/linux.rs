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
use std::{ffi::CString, path::PathBuf};
use std::collections::HashMap;
use tokio::sync::Mutex as TokioMutex;
use tracing::{warn, info};
use uuid::Uuid;
use zbus::{Connection, MatchRule, MessageStream};
use crate::core::system::disk_reader::disk::Disk;
use crate::core::system::disk_reader::disk_manager::{DiskManager};
use udisks2::{Client, filesystem::FilesystemProxy, block::BlockProxy};
use zbus::zvariant::{ObjectPath, Value};
use libc::statvfs64;
use tokio_stream::StreamExt;


pub struct LinuxDisks {
    partitions: Vec<Disk>,
    client: Client,
}


impl LinuxDisks {
    pub async fn init() -> Self {
        info!("Iniciando LinuxDisks");
        let client = Client::new().await.expect("Fallo al conectar a Udisk2 via D-Bus");

        Self {
            partitions: Vec::new(),
            client
        }
    }

    pub fn get_partitions(&self) -> Vec<Disk> {
        self.partitions.clone()
    }

    pub async fn load_disks(&mut self) {
        self.partitions.clear();
        let manager = self.client.manager();

        let block_paths = match manager.get_block_devices(HashMap::new()).await {
            Ok(paths) => paths,
            Err(e) => {
                warn!("Error al obtener block devices: {}", e);
                return ;
            }
        };

        info!("UDisks2 devolvió {} block devices", block_paths.len());

        for path in block_paths {
            let object_path_str = path.to_string();
            let object = match self.client.object(path) {
                Ok(o) => o,
                Err(e) => {
                    warn!("No se pudo obtener object para {}: {}", object_path_str, e);
                    continue;
                }
            };

            let block: BlockProxy = match object.block().await {
                Ok(b) => b,
                Err(e) => {
                    warn!("No BlockProxy en {}: {}", object_path_str, e);
                    continue;
                }
            };

            if block.hint_ignore().await.unwrap_or(false) { continue; }

            let dev_bytes = match block.device().await {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };

            let device = String::from_utf8_lossy(&dev_bytes)
                .trim_end_matches('\0')
                .to_string();

            if device.is_empty() || device == "/dev/" {
                continue;
            }

            let size = block.size().await.unwrap_or(0);

            if size < 1_048_576 || device.starts_with("/dev/loop") || device.starts_with("/dev/zram") {
                continue;
            }
            
            let label_opt = block.id_label().await.ok().filter(|s| !s.is_empty());
            
            let name = block.hint_name().await.ok().filter(|s| !s.is_empty());

            let kernel_name = block.device().await.ok().and_then(|bytes|{
                String::from_utf8(bytes).ok()
                    .map(|p| p.trim_matches(char::from(0)).split("/").last().unwrap_or(&p).to_string())
            });

            let uuid_opt = block.id_uuid().await.ok().filter(|s| !s.is_empty());

            let (is_removable, _) = match self.client.drive_for_block(&block).await {
                Ok(drive) => {
                    let removable = drive.removable().await.unwrap_or(false);
                    let bus = drive.connection_bus().await.unwrap_or_default();
                    (removable || bus == "usb" || bus == "ieee1394", bus)
                }
                Err(_) => (false, String::new()),
            };

            let idtype_opt = block.id_type().await.ok().filter(|s| !s.is_empty());

            let ends_with_number = device.chars().rev().next().map_or(false, |c| c.is_numeric());
            let has_partition_suffix = device.contains('p') || ends_with_number && device.matches(|c: char| c.is_numeric()).count() > 1;
            let is_partition = idtype_opt.is_some() || has_partition_suffix;

            if !is_partition && !is_removable {
                continue;
            }

            let mut mountpoint: Option<String> = None;
            let mut available: u64 = 0;
            let mut total: u64 = size;
            let mut used_percent: f32 = 0.0;

            if let Ok(fs) = object.filesystem().await {
                let mount_points = fs.mount_points().await.unwrap_or_default();
                if let Some(mp_bytes) = mount_points.first() {
                    let mp = String::from_utf8_lossy(mp_bytes).trim_end_matches('\0').to_string();
                    if !mp.is_empty() {
                        mountpoint = Some(mp.clone());
                        if let Some((avail, tot, perc)) = self.get_fs_usage(&mp) {
                            available = avail;
                            total = tot.max(size);
                            used_percent = perc;
                        }
                    }
                }
            }

            if let Some(mp) = &mountpoint {
                let system_paths = [ "/boot", "/boot/efi", "/var", "/etc"];
                if system_paths.contains(&mp.as_str()) {
                    continue;
                }
            }

            if idtype_opt.as_deref() == Some("swap") || device.starts_with("/dev/loop") || device.starts_with("/dev/zram") || size < 1_048_576 || (idtype_opt.is_none() && mountpoint.is_none()) {
                continue;
            }

            let display_name = label_opt.clone()
                .or(name.clone())
                .or(kernel_name)
                .or_else(|| mountpoint.clone())
                .unwrap_or_else(|| device.clone());

            self.partitions.push(Disk {
                display_name,
                device: Some(device),
                idtype: idtype_opt,
                label: label_opt,
                uuid: uuid_opt,
                mountpoint,
                available,
                total,
                used_percent,
                is_removable,
                is_partition,
                size,
            });
        }

        self.partitions.sort_by_key(|d| d.device.clone().unwrap_or_default());

        info!("Procesadas {} entradas válidas (particiones con filesystem)", self.partitions.len());
    }


    fn get_fs_usage(&self, mountpoint: &str) -> Option<(u64, u64, f32)> {
        let path = PathBuf::from(mountpoint).canonicalize().ok()?;
        let path_cstring = CString::new(path.to_string_lossy().as_bytes()).ok()?;

        unsafe {
            let mut stat: statvfs64 = std::mem::zeroed();
            
            if statvfs64(path_cstring.as_ptr(), &mut stat) == 0 {
                let block_size = stat.f_frsize;
                let total = stat.f_blocks as u64 * block_size;
                let available = stat.f_bavail as u64 * block_size;

                let free = stat.f_bfree as u64 * block_size;
                let used = total.saturating_sub(free);

                let used_percent = if total > 0 {
                    (used as f32 / total as f32) * 100.0
                } else {
                    0.0
                };

                Some((available, total, used_percent))
            } else {
                None
            }
        }
    }


    pub async fn mount(&self, disk: &Disk) -> anyhow::Result<String> {
        use users::{get_current_uid, get_current_gid};
        let uid = get_current_uid();
        let gid = get_current_gid();

        let device = disk.device.as_ref().ok_or_else(|| anyhow::anyhow!("Sin device"))?;

        let block_path = ObjectPath::try_from(format!("/org/freedesktop/UDisks2/block_devices{}", device.trim_start_matches("/dev")))?;

        let object = self.client.object(block_path)?;
        let fs: FilesystemProxy = object.filesystem().await?;

        let mut options: HashMap<&str, Value<'_>> = HashMap::new();

        options.insert("fstype", Value::from("auto"));
        options.insert("options", Value::Str(format!("uid={},gid={}", uid, gid).into()));

        let mount_point = fs.mount(options).await?;

        Ok(mount_point)
    }


    pub async fn unmount(&self, disk: &Disk) -> anyhow::Result<String> {
        if disk.mountpoint.is_none() {
            return Ok("No está montado.".to_string());
        }
    
        let device = disk.device.as_ref().ok_or_else(|| anyhow::anyhow!("Sin device"))?;
        let block_path = ObjectPath::try_from(format!("/org/freedesktop/UDisks2/block_devices{}", device.trim_start_matches("/dev")))?;

        let object = self.client.object(block_path)?;
        let fs: FilesystemProxy = object.filesystem().await?;

        let mut unmount_options = HashMap::new();
        unmount_options.insert("force", Value::Bool(true));
        
        fs.unmount(unmount_options).await?;

        Ok("Desmontado.".to_string())
    }


    pub async fn eject(&self, disk: &Disk) -> anyhow::Result<String> {
        let device = disk.device.as_ref()
            .ok_or_else(|| anyhow::anyhow!("El disco no tiene un nodo de dispositivo asignado"))?;

        let dev_name = device.trim_start_matches("/dev/");
        let block_path = ObjectPath::try_from(format!("/org/freedesktop/UDisks2/block_devices/{}", dev_name))?;

        let block_object = self.client.object(block_path)?;
        let block = block_object.block().await?;

        let drive_path = block.drive().await?;
        if drive_path.as_str() == "/" {
            return Err(anyhow::anyhow!("Este dispositivo no tiene un Drive físico asociado (puede ser virtual o loop)"));
        }

        let drive_object = self.client.object(drive_path)?;
        let drive = drive_object.drive().await?;

        let mut options = HashMap::new();

        options.insert("auth.no_user_interaction", Value::Bool(true));

        drive.eject(options).await.ok();

        Ok(format!("Dispositivo {} expulsado de forma segura.", disk.display_name))
    }


    pub async fn start_disk_watcher(&self, manager_arc: Arc<TokioMutex<DiskManager>>) -> anyhow::Result<()> {
        let conn = Connection::system().await?;

        let rule_added = MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .sender("org.freedesktop.UDisks2")?
            .interface("org.freedesktop.DBus.ObjectManager")?
            .member("InterfacesAdded")?
            .build();

        let rule_removed = MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .sender("org.freedesktop.UDisks2")?
            .interface("org.freedesktop.DBus.ObjectManager")?
            .member("InterfacesRemoved")?
            .build();

        let mut stream_added = MessageStream::for_match_rule(rule_added, &conn, Some(32)).await?;

        let mut stream_removed = MessageStream::for_match_rule(rule_removed, &conn, Some(32)).await?;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(Ok(msg)) = stream_added.next() => {
                        info!("Disco conectado: {:?}", msg.header().path());
                        let mut mgr = manager_arc.lock().await;
                        mgr.load_disks().await;
                    },
                    Some(Ok(msg)) = stream_removed.next() => {
                        info!("Disco desconectado: {:?}", msg.header().path());
                        let mut mgr = manager_arc.lock().await;
                        mgr.load_disks().await;
                    },

                    else => {
                        warn!("Disk Watcher terminado.");
                        break;
                    }
                }
            }
        });

        Ok(())
    }
    
}