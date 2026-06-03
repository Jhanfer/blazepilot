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

use crate::core::system::disk_reader::disk::Disk;
use crate::core::system::disk_reader::disk_manager::DiskManager;
use libc::statvfs64;
use std::collections::HashMap;
use std::sync::Arc;
use std::{ffi::CString, path::PathBuf};
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::StreamExt;
use tracing::{info, warn};
use udisks2::{block::BlockProxy, filesystem::FilesystemProxy, Client};
use users::{get_current_gid, get_current_uid};
use zbus::zvariant::{ObjectPath, Value};
use zbus::{Connection, MatchRule, MessageStream};

pub struct LinuxDisks {
    partitions: Vec<Disk>,
    client: Client,
}

impl LinuxDisks {
    pub async fn init() -> Self {
        info!("Iniciando LinuxDisks");
        let client = Client::new()
            .await
            .expect("Fallo al conectar a Udisk2 via D-Bus");

        Self {
            partitions: Vec::new(),
            client,
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
                return;
            }
        };

        info!("UDisks2 devolvió {} block devices", block_paths.len());

        for path in block_paths {
            if let Some(disk) = self.process_block_device(path.into()).await {
                self.partitions.push(disk);
            }
        }

        self.partitions
            .sort_by_key(|d| d.device.clone().unwrap_or_default());
        info!("Procesadas {} entradas válidas", self.partitions.len());
    }

    async fn process_block_device(&self, path: ObjectPath<'_>) -> Option<Disk> {
        let object = self.client.object(path).ok()?;
        let block = object.block().await.ok()?;

        if block.hint_ignore().await.unwrap_or(false) {
            return None;
        }

        let dev_bytes = block.device().await.ok()?;

        let device = String::from_utf8_lossy(&dev_bytes)
            .trim_end_matches('\0')
            .to_string();

        if device.is_empty()
            || device == "/dev/"
            || device.starts_with("/dev/loop")
            || device.starts_with("/dev/zram")
        {
            return None;
        }

        let size = block.size().await.unwrap_or(0);
        if size < 1_048_576 {
            return None;
        }

        let idtype_opt = block.id_type().await.ok().filter(|s| !s.is_empty());
        if idtype_opt.as_deref() == Some("swap") {
            return None;
        }

        let label_opt = block.id_label().await.ok().filter(|s| !s.is_empty());
        let name = block.hint_name().await.ok().filter(|s| !s.is_empty());
        let uuid_opt = block.id_uuid().await.ok().filter(|s| !s.is_empty());
        let kernel_name = Self::extract_kernel_name(&block).await;

        let is_removable = Self::check_removable(&self.client, &block).await;

        let has_partition_table = object.partition_table().await.is_ok();

        if has_partition_table {
            return None;
        }

        let is_partition = object.partition().await.is_ok();

        if !is_partition && !is_removable {
            return None;
        }

        let (mountpoint, available, total, used_percent) = self.read_fs_info(&object, size).await;

        let is_system = Self::check_is_system(mountpoint.as_deref());

        let display_name = label_opt
            .clone()
            .or(name.clone())
            .or(kernel_name)
            .or_else(|| mountpoint.clone())
            .unwrap_or_else(|| device.clone());

        Some(Disk {
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
            is_system,
        })
    }

    async fn extract_kernel_name(block: &BlockProxy<'_>) -> Option<String> {
        block.device().await.ok().and_then(|bytes| {
            String::from_utf8(bytes)
                .ok()
                .map(|p| {
                    p.trim_matches('\0')
                        .split("/")
                        .last()
                        .unwrap_or("")
                        .to_string()
                })
                .filter(|s| !s.is_empty())
        })
    }

    async fn check_removable(client: &Client, block: &BlockProxy<'_>) -> bool {
        match client.drive_for_block(block).await {
            Ok(drive) => {
                let removable = drive.removable().await.unwrap_or(false);
                let bus = drive.connection_bus().await.unwrap_or_default();
                removable || bus == "usb" || bus == "ieee1394"
            }
            Err(_) => false,
        }
    }

    async fn read_fs_info(
        &self,
        object: &udisks2::Object,
        fallback_size: u64,
    ) -> (Option<String>, u64, u64, f32) {
        let Ok(fs) = object.filesystem().await else {
            return (None, 0, fallback_size, 0.0);
        };

        let mount_points = fs.mount_points().await.unwrap_or_default();
        let Some(mp_bytes) = mount_points.first() else {
            return (None, 0, fallback_size, 0.0);
        };

        let mp = String::from_utf8_lossy(mp_bytes)
            .trim_end_matches('\0')
            .to_string();
        if mp.is_empty() {
            return (None, 0, fallback_size, 0.0);
        }

        let (avail, total, perc) = self.get_fs_usage(&mp).unwrap_or((0, fallback_size, 0.0));

        (Some(mp), avail, total.max(fallback_size), perc)
    }

    fn check_is_system(mountpoint: Option<&str>) -> bool {
        const SYSTEM_PATHS: &[&str] = &[
            "/",
            "/home",
            "/boot",
            "/boot/efi",
            "/var",
            "/etc",
            "/tmp",
            "/usr",
        ];
        mountpoint.is_some_and(|mp| SYSTEM_PATHS.contains(&mp) || mp.starts_with("/snap/"))
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

    fn block_devices_path(&self, device: &str) -> anyhow::Result<ObjectPath<'static>> {
        let dev_name = device.trim_start_matches("/dev/");
        Ok(ObjectPath::try_from(format!(
            "/org/freedesktop/UDisks2/block_devices/{}",
            dev_name
        ))?)
    }

    pub async fn mount(&self, disk: &Disk) -> anyhow::Result<String> {
        let uid = get_current_uid();
        let gid = get_current_gid();

        let device = disk
            .device
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Sin device"))?;

        let block_path = self.block_devices_path(device)?;

        let object = self.client.object(block_path)?;
        let fs: FilesystemProxy = object.filesystem().await?;

        let mut options: HashMap<&str, Value<'_>> = HashMap::new();

        options.insert("fstype", Value::from("auto"));
        options.insert(
            "options",
            Value::Str(format!("uid={},gid={}", uid, gid).into()),
        );

        let mount_point = fs.mount(options).await?;

        Ok(mount_point)
    }

    pub async fn unmount(&self, disk: &Disk) -> anyhow::Result<String> {
        if disk.mountpoint.is_none() {
            return Ok("No está montado.".to_string());
        }

        let device = disk
            .device
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Sin device"))?;

        let block_path = self.block_devices_path(device)?;

        let object = self.client.object(block_path)?;
        let fs: FilesystemProxy = object.filesystem().await?;

        let mut unmount_options = HashMap::new();
        unmount_options.insert("force", Value::Bool(true));

        fs.unmount(unmount_options).await?;

        Ok("Desmontado.".to_string())
    }

    pub async fn eject(&self, disk: &Disk) -> anyhow::Result<String> {
        let device = disk
            .device
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("El disco no tiene un nodo de dispositivo asignado"))?;

        let block_path = self.block_devices_path(device)?;

        let block_object = self.client.object(block_path)?;
        let block = block_object.block().await?;

        let drive_path = block.drive().await?;
        if drive_path.as_str() == "/" {
            return Err(anyhow::anyhow!(
                "Este dispositivo no tiene un Drive físico asociado (puede ser virtual o loop)"
            ));
        }

        let drive_object = self.client.object(drive_path)?;
        let drive = drive_object.drive().await?;

        let mut options = HashMap::new();

        options.insert("auth.no_user_interaction", Value::Bool(true));

        drive.eject(options).await?;

        Ok(format!(
            "Dispositivo {} expulsado de forma segura.",
            disk.display_name
        ))
    }

    pub async fn start_disk_watcher(
        &self,
        manager_arc: Arc<TokioMutex<DiskManager>>,
    ) -> anyhow::Result<()> {
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

        let mut stream_removed =
            MessageStream::for_match_rule(rule_removed, &conn, Some(32)).await?;

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
