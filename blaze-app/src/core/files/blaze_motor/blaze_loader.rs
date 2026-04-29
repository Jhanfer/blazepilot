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




use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicBool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use file_id::{get_file_id};
use tokio::sync::Mutex;
use tracing::warn;
use crate::core::files::blaze_motor::error::{MotorError, MotorResult};
use crate::core::files::blaze_motor::motor_structs::{FileEntry, FileLoadingMessage};
use crate::core::system::clipboard::TOKIO_RUNTIME;
use crate::utils::channel_pool::NotifyingSender;
use crate::core::files::blaze_motor::utilities::build_entry;





#[derive(Clone)]
enum FileOp {
    Add(Arc<FileEntry>),
    #[allow(unused)]
    Remove(PathBuf),
    #[allow(unused)]
    Modify(PathBuf),
}


type PathKey = PathBuf;
pub struct EventBuffer {
    map: HashMap<PathKey, FileOp>,
}


impl EventBuffer {
    fn push(&mut self, path: PathBuf, op:FileOp) {
        use FileOp::*;

        match (self.map.remove(&path), op) {

            //Crear - anula todo
            (_, Add(entry)) => {
                self.map.insert(path, Add(entry));
            },

            //Borrar - anula todo
            (_, Remove(entry)) => {
                self.map.insert(path, Remove(entry));
            },

            //modificar y fusionar en existentes
            (Some(Add(e)), Modify(_)) => {
                self.map.insert(path, Add(e));
            },

            //modificar unico
            (_, Modify(p)) => {
                self.map.insert(path, Modify(p));
            },

        }
    }

    fn drain(&mut self) -> Vec<(PathBuf, FileOp)> {
        self.map.drain().collect()
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self { map: HashMap::new() }
    }
}



pub struct BlazeLoader {
    pub cancel: Arc<AtomicBool>,
    buffer: Arc<Mutex<EventBuffer>>,
    pub handles: Vec<tokio::task::JoinHandle<()>>, 
}

impl Default for BlazeLoader {
    fn default() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(EventBuffer::default())),
            handles: Vec::new(),
        }
    }
}


impl BlazeLoader {
    pub fn cancel(&mut self) {
        self.cancel.store(true, Ordering::Release);

        for handle in self.handles.drain(..) {
            handle.abort();
        }
    }


    pub fn load_path(&mut self, path: &PathBuf, sender: NotifyingSender, generation: u64) -> MotorResult<()> {
        //cancelar la carga anterior
        self.cancel();

        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = cancel.clone();

        let buffer = Arc::new(Mutex::new(EventBuffer::default()));
        self.buffer = buffer.clone();

        //canales para avisar al dispatcher
        let (done_tx, done_rx) = tokio::sync::oneshot::channel::<MotorResult<()>>();


        //Tarea A:
        let scan_cancel = cancel.clone();
        let scan_buffer = buffer.clone();
        let scan_path = path.clone();
        let handle1 = TOKIO_RUNTIME.spawn(async move {
            let res = Self::scan_dir(scan_path, scan_buffer, scan_cancel).await;
            //avisar al dispatcher que el scan ha terminado
            if done_tx.send(res).is_err() {
                warn!("El receptor done en scan se ha cerrado antes de tiempo.");
            }
        });

        self.handles.push(handle1);

        //Tarea B:
        let disp_cancel = cancel.clone();
        let handle2 = TOKIO_RUNTIME.spawn(async move {
            if Self::dispatcher(buffer, sender, generation, disp_cancel, done_rx).await.is_err() {
                warn!("Erro en el dispatcher.");
            };
        });

        self.handles.push(handle2);

        Ok(())
    }

    async fn scan_dir(path: PathBuf, buffer: Arc<Mutex<EventBuffer>>, cancel: Arc<AtomicBool>) -> MotorResult<()> {
        let mut entries = match tokio::fs::read_dir(&path).await {
            Ok(e) => e,
            Err(e) => {
                warn!("scan_dir: No se ha podido leer el directorio {:?}: {}", path, e);
                return Err(MotorError::Io(e));
            },
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            if cancel.load(Ordering::Acquire) {
                break;
            }

            let entry_path = entry.path();

            let meta = match entry.metadata().await {
                Ok(m) => m,
                Err(e) => return Err(MotorError::Io(e)),
            };

            let file_entry = tokio::task::spawn_blocking(move ||{
                let unique_id = get_file_id(&entry_path).ok();
                let entry = build_entry(entry_path.clone(), meta, unique_id);
                Arc::new(entry)
            })
            .await
            .map_err(|e| MotorError::ThreadError(e))?;

            buffer.lock().await.push(file_entry.full_path.clone(), FileOp::Add(file_entry));
        }

        Ok(())
    }



    async fn dispatcher(buffer: Arc<Mutex<EventBuffer>>, sender: NotifyingSender, generation: u64, cancel: Arc<AtomicBool>, mut done_rx: tokio::sync::oneshot::Receiver<MotorResult<()>>) -> MotorResult<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        let mut scan_done = false;

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if cancel.load(Ordering::Acquire) {
                        break;
                    }
                    Self::flush_buffer(&buffer, &sender, generation).await?;
                }

                _ = &mut done_rx, if !scan_done => {
                    scan_done = true;
                    Self::flush_buffer(&buffer, &sender, generation).await?;

                    sender.send_files_batch(FileLoadingMessage::Finished(generation))
                        .map_err(|e| MotorError::SendFileBatchError(e))?;
                    break;
                }
            }
        }

        Ok(())
    }

    async fn flush_buffer(buffer: &Arc<Mutex<EventBuffer>>, sender: &NotifyingSender, generation: u64) -> MotorResult<()> {
        let batch = {
            let mut buf = buffer.lock().await;
            if buf.is_empty() {
                return Ok(());
            }
            buf.drain()
        };

        let mut adds: Vec<Arc<FileEntry>> = Vec::with_capacity(batch.len());

        for (_, op) in batch {
            match op {
                FileOp::Add(entry) => {
                    adds.push(entry);
                },

                FileOp::Remove(path) => {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    if !name.is_empty() {
                        sender.send_files_batch(FileLoadingMessage::FileRemoved { name })
                            .map_err(|e| MotorError::SendFileBatchError(e))?;
                    }
                },

                FileOp::Modify(_) => {
                    sender.send_files_batch(FileLoadingMessage::FullRefresh)
                        .map_err(|e| MotorError::SendFileBatchError(e))?;
                },

            }
        }

        if !adds.is_empty() {
            sender.send_files_batch(FileLoadingMessage::Batch(generation, adds))
                .map_err(|e| MotorError::SendFileBatchError(e))?;
        }

        Ok(())
    }
}