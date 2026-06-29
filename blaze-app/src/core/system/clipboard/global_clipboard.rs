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

use crate::core::files::blaze_motor::motor::with_motor;
use crate::core::files::blaze_motor::motor_structs::{FileEntry, TaskType};
use crate::core::files::blaze_motor::tab_state::new_task_id;
use crate::core::runtime::bus_structs::{FileConflict, FileOperation, UiEvent};
use crate::core::runtime::event_bus::Dispatcher;
use crate::core::system::clipboard::error::{ClipBoardError, ClipBoardResult};
use crate::core::system::operationstate::operation_manager::with_history;
use crate::core::system::operationstate::undo_record::UndoRecord;
use crate::core::system::trash_manager::manager::{TrashBackend, TrashDestination, get_backend};
use crate::ui::task_manager::tasks::TaskMessage;
use once_cell::sync::Lazy;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::vec;
use std::{fs, path::Path};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

pub enum ConflictStrategy {
    Ask, // preguntar al usuario (por defecto)
    #[allow(unused)]
    Overwrite, // sobreescribir siempre
    #[allow(unused)]
    Rename, // renombrar automáticamente (copia 2, copia 3...)
    #[allow(unused)]
    Skip, // saltar el archivo
}

#[derive(Clone, Debug)]
pub enum ClipboardMode {
    Copy,
    Cut,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClipboardItem {
    pub src_path: Arc<Path>,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Clipboard {
    inner: Arc<Mutex<ClipboardInner>>,
}

#[derive(Clone, Debug)]
struct ClipboardInner {
    pub mode: Option<ClipboardMode>,
    pub items: Vec<ClipboardItem>,
    pub dest_dir: Option<Arc<Path>>,
}

pub struct GlobalClipboard;

impl GlobalClipboard {
    pub fn new() -> Self {
        Self
    }

    fn inner() -> &'static Clipboard {
        static INSTANCE: OnceLock<Clipboard> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            info!("Clipboard global creado");
            Clipboard::new()
        })
    }

    pub fn clipboard_has_files(&self) -> ClipBoardResult<bool> {
        Self::inner().clipboard_has_files()
    }

    pub fn clear(&self) -> ClipBoardResult<()> {
        Self::inner().clear()
    }

    pub fn set_dest(&self, dest: Arc<Path>) -> ClipBoardResult<()> {
        Self::inner().set_dest(dest)
    }

    pub fn copy_items(
        &self,
        items: Vec<Arc<FileEntry>>,
        current_cwd: Arc<Path>,
    ) -> ClipBoardResult<()> {
        Self::inner().copy_items(items, current_cwd)
    }

    pub fn cut_items(
        &self,
        items: Vec<Arc<FileEntry>>,
        current_cwd: Arc<Path>,
    ) -> ClipBoardResult<()> {
        Self::inner().cut_items(items, current_cwd)
    }

    pub fn paste(&self, sender: &Dispatcher) -> ClipBoardResult<()> {
        Self::inner().pastex(sender)
    }

    pub fn move_files(
        &self,
        items: Vec<Arc<Path>>,
        dest: Arc<Path>,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        Self::inner().move_files(items, dest, ConflictStrategy::Ask, sender)
    }

    pub fn move_to_trash(
        &self,
        items: Vec<(Arc<str>, Arc<Path>)>,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        Self::inner().move_to_trash(items, sender)
    }

    pub fn restore_from_trash(
        &self,
        items: Vec<String>,
        trash_root: Arc<Path>,
        sender: Dispatcher,
    ) -> ClipBoardResult<()> {
        Self::inner().restore_items(items, trash_root, sender)
    }

    pub fn rename_file(
        &self,
        file_name: &str,
        new_file_name: &str,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        Self::inner().rename_file(file_name, new_file_name, sender)
    }

    pub fn create_new_dir(
        &self,
        file_name: &str,
        current_cwd: Arc<Path>,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        Self::inner().create_new_dir(file_name, current_cwd, sender)
    }

    pub fn create_new_file(
        &self,
        file_name: &str,
        current_cwd: Arc<Path>,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        Self::inner().create_new_file(file_name, current_cwd, sender)
    }
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClipboardInner {
                mode: None,
                items: Vec::new(),
                dest_dir: None,
            })),
        }
    }

    fn get_inner(&self) -> ClipBoardResult<MutexGuard<'_, ClipboardInner>> {
        self.inner.lock().map_err(|_| ClipBoardError::PoisonedLock)
    }

    pub fn clipboard_has_files(&self) -> ClipBoardResult<bool> {
        let inner = self.get_inner()?;
        Ok(!inner.items.is_empty())
    }

    pub fn clear(&self) -> ClipBoardResult<()> {
        let mut inner = self.get_inner()?;
        inner.mode = None;
        inner.items.clear();
        inner.dest_dir = None;

        Ok(())
    }

    pub fn copy_items(
        &self,
        items: Vec<Arc<FileEntry>>,
        current_cwd: Arc<Path>,
    ) -> ClipBoardResult<()> {
        let mut inner = self.get_inner()?;
        inner.mode = Some(ClipboardMode::Copy);
        self.prepare_items(items, current_cwd, &mut inner);

        Ok(())
    }

    pub fn cut_items(
        &self,
        items: Vec<Arc<FileEntry>>,
        current_cwd: Arc<Path>,
    ) -> ClipBoardResult<()> {
        let mut inner = self.get_inner()?;
        inner.mode = Some(ClipboardMode::Cut);
        self.prepare_items(items, current_cwd, &mut inner);

        Ok(())
    }

    fn prepare_items(
        &self,
        items: Vec<Arc<FileEntry>>,
        current_cwd: Arc<Path>,
        inner: &mut MutexGuard<'_, ClipboardInner>,
    ) {
        inner.items = items
            .iter()
            .map(|e| ClipboardItem {
                src_path: current_cwd.join(e.name.as_ref()).into(),
                name: e.name.to_string(),
            })
            .collect();
        inner.dest_dir = None;
    }

    pub fn set_dest(&self, dest: Arc<Path>) -> ClipBoardResult<()> {
        let mut inner = self.get_inner()?;
        if !dest.is_dir() || !dest.exists() {
            return Ok(());
        }
        inner.dest_dir = Some(dest);

        Ok(())
    }

    pub fn pastex(&self, sender: &Dispatcher) -> ClipBoardResult<()> {
        let mut inner = self.get_inner()?;
        let Some(dest) = inner.dest_dir.take() else {
            warn!("No hay destio donde pegar.");
            return Err(ClipBoardError::NoDestError);
        };

        let items = inner.items.clone();
        let mode = inner.mode.clone();
        let task_id = new_task_id();

        sender
            .send(TaskMessage::Started {
                task_id,
                text: "Pegando archivos...".to_string(),
                task_type: TaskType::CopyPaste,
            })
            .ok();

        let inner = Arc::clone(&self.inner);

        let sender = sender.clone();
        TOKIO_RUNTIME.spawn(async move {
            let mut total_bytes_global: u64 = 0;
            for item in &items {
                total_bytes_global += Self::calculate_size(&item.src_path).await;
            }

            info!(total_bytes = %total_bytes_global, "Peso total calculado");

            let copied_bytes_global = Arc::new(AtomicU64::new(0));

            let semaphore = Arc::new(Semaphore::new(3));
            let mut handles = Vec::default();

            let errors_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

            let actual_sources = Arc::new(Mutex::new(Vec::<Arc<Path>>::new()));
            let actual_finals = Arc::new(Mutex::new(Vec::<Arc<Path>>::new()));

            let is_cut = matches!(mode, Some(ClipboardMode::Cut));

            for item in items {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(e) => {
                        errors_count.fetch_add(1, Ordering::Relaxed);
                        warn!("Acquire error: {:?}", e);
                        continue;
                    }
                };

                let dest = dest.clone();
                let mode = mode.clone();
                let sender = sender.clone();
                let errors = errors_count.clone();
                let copied_global = copied_bytes_global.clone();

                let src_for_history = Arc::clone(&actual_sources);
                let final_for_history = Arc::clone(&actual_finals);

                let handle = tokio::spawn(async move {
                    let _permit = permit;

                    let dest_path_buf = dest.join(&item.name);
                    let dest_path = if dest_path_buf.exists() {
                        match mode {
                            Some(ClipboardMode::Cut) => {
                                warn!("Destino ya existe para Cut, saltando");
                                errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                sender
                                    .send(UiEvent::ShowError(
                                        format!("Ya existe '{}' en destino.", item.name).into(),
                                    ))
                                    .ok();
                                return;
                            }
                            _ => Self::generate_unique_path(&dest_path_buf),
                        }
                    } else {
                        dest_path_buf.into()
                    };

                    let src_path_clone = item.src_path.clone();
                    let dest_path_clone = dest_path.clone();

                    let result = Self::paste_item_with_progress(
                        item.src_path,
                        dest_path,
                        task_id,
                        &sender,
                        &copied_global,
                        total_bytes_global,
                        is_cut,
                    )
                    .await;

                    if let Err(err_msg) = result {
                        errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        let msg = err_msg.to_string();

                        let needs_root = matches!(err_msg, ClipBoardError::PermissionDenied(_));

                        sender.send(UiEvent::ShowError(msg.into())).ok();

                        if needs_root {
                            info!("Error de permisos detectado. Abortando tarea.");
                        }
                    } else {
                        match src_for_history.lock() {
                            Ok(mut src) => src.push(src_path_clone),
                            Err(e) => {
                                errors.fetch_add(1, Ordering::Relaxed);
                                warn!("Acquire error: {:?}", e);
                            }
                        };

                        match final_for_history.lock() {
                            Ok(mut final_for) => final_for.push(dest_path_clone),
                            Err(e) => {
                                errors.fetch_add(1, Ordering::Relaxed);
                                warn!("Acquire error: {:?}", e);
                            }
                        };
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.await;
            }

            let has_errors = errors_count.load(std::sync::atomic::Ordering::Relaxed) > 0;

            let mut inner = match inner.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    warn!("Error dentro de TOKIO_RUNTIME:Pastex: {}", e);
                    return;
                }
            };

            if is_cut {
                inner.mode = None;
                inner.items.clear();
            }
            inner.dest_dir = None;

            let (actual_sources, actual_finals) = {
                let sources = actual_sources.lock();
                let finals = actual_finals.lock();

                match (sources, finals) {
                    (Ok(s), Ok(f)) => (s.clone(), f.clone()),
                    _ => {
                        warn!("mutex envenenado en results de pastex");
                        return;
                    }
                }
            };

            if !actual_finals.is_empty() {
                let op = if is_cut {
                    FileOperation::PasteCut {
                        sources: actual_sources,
                        final_targets: actual_finals,
                    }
                } else {
                    FileOperation::PasteCopy {
                        final_targets: actual_finals,
                    }
                };

                sender.send(op).ok();
            }

            info!("Todos los items procesados, mandando Finished");
            sender
                .send(TaskMessage::Finished {
                    task_id,
                    success: !has_errors,
                    task_type: TaskType::CopyPaste,
                    text: "Listo!".to_string(),
                })
                .ok();
            info!("TaskMessage::Finished enviado, success={}", !has_errors);
        });

        Ok(())
    }

    async fn paste_item_with_progress(
        src: Arc<Path>,
        dest: Arc<Path>,
        task_id: u64,
        sender: &Dispatcher,
        copied_global: &Arc<AtomicU64>,
        total_bytes_global: u64,
        is_cut: bool,
    ) -> ClipBoardResult<()> {
        if is_cut {
            match tokio::fs::rename(src.clone(), dest.clone()).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    if e.raw_os_error() != Some(18) {
                        return Err(ClipBoardError::CrossDeviceError);
                    }
                }
            }
        }

        let copy_result = if src.is_dir() {
            Self::copy_dir_recursive_async(
                src.clone(),
                dest,
                task_id,
                sender,
                copied_global,
                total_bytes_global,
            )
            .await
        } else {
            Self::copy_file_with_progress(
                src.clone(),
                dest,
                task_id,
                sender,
                copied_global,
                total_bytes_global,
            )
            .await
        };

        copy_result?;

        if is_cut {
            let delete_result = if src.is_dir() {
                tokio::fs::remove_dir_all(src).await
            } else {
                tokio::fs::remove_file(src).await
            };

            if let Err(e) = delete_result {
                warn!("No se pudo eliminar origen después de copiar: {}", e);
            }
        }

        Ok(())
    }

    async fn copy_file_with_progress(
        src: Arc<Path>,
        dest: Arc<Path>,
        task_id: u64,
        sender: &Dispatcher,
        copied_global: &Arc<AtomicU64>,
        total_bytes_global: u64,
    ) -> ClipBoardResult<()> {
        let mut reader = tokio::fs::File::open(src.clone())
            .await
            .map_err(ClipBoardError::Io)?;
        let mut writer = tokio::fs::File::create(dest.clone())
            .await
            .map_err(ClipBoardError::Io)?;

        if let Ok(metadata) = tokio::fs::metadata(src.clone()).await {
            tokio::fs::set_permissions(dest.clone(), metadata.permissions())
                .await
                .map_err(|_| ClipBoardError::PermissionDenied(src))?;
        }

        //future conservar timestamps

        let mut buffer = vec![0u8; 64 * 1024];
        let mut last_update = std::time::Instant::now();

        loop {
            let bytes_read = reader.read(&mut buffer).await.map_err(ClipBoardError::Io)?;
            if bytes_read == 0 {
                break;
            }

            writer
                .write_all(&buffer[..bytes_read])
                .await
                .map_err(ClipBoardError::Io)?;

            copied_global.fetch_add(bytes_read as u64, Ordering::Relaxed);

            tokio::task::yield_now().await;

            if last_update.elapsed().as_millis() > 100 {
                let copied = copied_global.load(Ordering::Relaxed);
                let progress = if total_bytes_global > 0 {
                    copied as f32 / total_bytes_global as f32
                } else {
                    0.0
                };
                sender
                    .send(TaskMessage::Progress {
                        task_id,
                        progress,
                        text: format!("{} MB copiados", copied as f64 / 1_000_000.0),
                        task_type: TaskType::CopyPaste,
                    })
                    .ok();
                last_update = std::time::Instant::now();
                debug!("Progress durante copia: {}%", progress * 100.0);
            }
        }

        info!("Archivo copiado exitosamente: {:?}", dest.file_name());
        Ok(())
    }

    async fn copy_dir_recursive_async(
        src: Arc<Path>,
        dest: Arc<Path>,
        task_id: u64,
        sender: &Dispatcher,
        copied_global: &Arc<AtomicU64>,
        total_bytes_global: u64,
    ) -> ClipBoardResult<()> {
        tokio::fs::create_dir_all(dest.clone())
            .await
            .map_err(ClipBoardError::Io)?;

        if let Ok(metadata) = tokio::fs::metadata(src.clone()).await {
            let permissions = metadata.permissions();
            tokio::fs::set_permissions(dest.clone(), permissions)
                .await
                .map_err(|_| ClipBoardError::PermissionDenied(dest.clone()))?;
        }

        //future conservar timestamps

        let mut read_dir = match tokio::fs::read_dir(src).await {
            Ok(rd) => rd,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    tokio::fs::remove_dir_all(dest.clone()).await.ok();
                    return Err(ClipBoardError::PermissionDenied(dest));
                }

                tokio::fs::remove_dir_all(dest).await.ok();
                return Err(ClipBoardError::Io(e));
            }
        };

        while let Some(entry) = read_dir.next_entry().await.map_err(ClipBoardError::Io)? {
            let entry_path = entry.path();
            let entry_path = entry_path.as_path();
            let dst_path = dest.join(entry.file_name());

            if entry_path.is_dir() {
                Box::pin(Self::copy_dir_recursive_async(
                    entry_path.into(),
                    dst_path.into(),
                    task_id,
                    sender,
                    copied_global,
                    total_bytes_global,
                ))
                .await?;
            } else {
                Self::copy_file_with_progress(
                    entry_path.into(),
                    dst_path.into(),
                    task_id,
                    sender,
                    copied_global,
                    total_bytes_global,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn calculate_size(path: &Arc<Path>) -> u64 {
        if path.is_file() {
            return tokio::fs::metadata(path)
                .await
                .map(|m| m.len())
                .unwrap_or(0);
        }
        let mut total = 0u64;
        let mut stack = vec![path.clone()];

        while let Some(current) = stack.pop() {
            let mut read_dir = match tokio::fs::read_dir(&current).await {
                Ok(rd) => rd,
                Err(e) => {
                    warn!("No se pudo leer directorio {:?}: {}", current, e);
                    continue;
                }
            };

            while let Ok(Some(entry)) = read_dir.next_entry().await {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    total += tokio::fs::metadata(&entry_path)
                        .await
                        .map(|m| m.len())
                        .unwrap_or(0);
                } else {
                    stack.push(entry_path.into());
                }
            }
        }

        total
    }

    pub fn generate_unique_path(path: &Path) -> Arc<Path> {
        if !path.exists() {
            return path.into();
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str());

        let mut counter = 1u32;

        loop {
            let new_name = match ext {
                Some(e) => format!("{} ({}).{}", stem, counter, e),
                None => format!("{} ({})", stem, counter),
            };

            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path.into();
            }
            counter += 1;

            if counter > 10_00 {
                return parent
                    .join(format!("{}_{}", stem, uuid::Uuid::new_v4()))
                    .into();
            }
        }
    }

    ///Mover archivos
    fn resolve_dest_path(target: Arc<Path>) -> Arc<Path> {
        if !target.exists() {
            return target;
        }

        let stem = target
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("archivo");
        let ext = target
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        let parent = target.parent().unwrap_or(Path::new(""));

        let mut counter = 2;
        loop {
            let new_name = format!("{} ({}){}", stem, counter, ext);
            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path.into();
            }
            counter += 1;
        }
    }

    pub fn move_files(
        &self,
        items: Vec<Arc<Path>>,
        dest: Arc<Path>,
        conflict: ConflictStrategy,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        let task_id = new_task_id();

        sender
            .send(TaskMessage::Started {
                task_id,
                text: "Moviendo archivos...".to_string(),
                task_type: TaskType::CopyPaste,
            })
            .ok();

        let sender = sender.clone();

        TOKIO_RUNTIME.spawn(async move {
            let total = items.len();
            let mut errors = Vec::new();
            let mut actual_finals: Vec<Arc<Path>> = Vec::new();
            let mut actual_sources: Vec<Arc<Path>> = Vec::new();

            for (done, source) in items.iter().enumerate() {
                let file_name = match source.file_name() {
                    Some(n) => n,
                    None => {
                        errors.push(format!("Ruta inválida: {:?}", source));
                        continue;
                    }
                };

                let target = dest.join(file_name);

                let final_target = if target.exists() {
                    match conflict {
                        ConflictStrategy::Overwrite => target.into(),
                        ConflictStrategy::Rename => Self::resolve_dest_path(target.into()),
                        ConflictStrategy::Skip => {
                            sender
                                .send(TaskMessage::Progress {
                                    task_id,
                                    progress: (done + 1) as f32 / total as f32,
                                    text: format!("Omitido: {}", file_name.to_string_lossy()),
                                    task_type: TaskType::CopyPaste,
                                })
                                .ok();
                            continue;
                        }
                        ConflictStrategy::Ask => {
                            sender
                                .send(UiEvent::FileConflict(FileConflict::AlreadyExist {
                                    name: file_name.to_string_lossy().to_string(),
                                    path: target.into(),
                                }))
                                .ok();

                            continue;
                        }
                    }
                } else {
                    target.into()
                };

                let result = fs::rename(source, &final_target)
                    .map_err(ClipBoardError::Io)
                    .or_else(|_| {
                        if source.is_dir() {
                            Self::copy_dir_recursive(source.to_owned(), final_target.to_owned())
                                .and_then(|_| {
                                    fs::remove_dir_all(source).map_err(ClipBoardError::Io)
                                })
                        } else {
                            fs::copy(source, &final_target)
                                .map_err(ClipBoardError::Io)
                                .and_then(|_| fs::remove_file(source).map_err(ClipBoardError::Io))
                        }
                    });

                if result.is_ok() {
                    actual_sources.push(source.to_owned());
                    actual_finals.push(final_target.clone());
                } else if let Err(e) = result {
                    errors.push(format!("Error moviendo '{:?}': {}", file_name, e));
                }

                sender
                    .send(TaskMessage::Progress {
                        task_id,
                        progress: (done + 1) as f32 / total as f32,
                        text: format!("Moviendo {}...", file_name.to_string_lossy()),
                        task_type: TaskType::CopyPaste,
                    })
                    .ok();
            }

            if !actual_finals.is_empty() {
                with_history(|h| {
                    h.push(UndoRecord::MoveBack {
                        from: actual_finals,
                        to: actual_sources,
                    })
                });
            }

            sender
                .send(TaskMessage::Finished {
                    task_id,
                    success: errors.is_empty(),
                    task_type: TaskType::CopyPaste,
                    text: if errors.is_empty() {
                        "Archivos movidos".to_string()
                    } else {
                        format!("{} errores al mover", errors.len())
                    },
                })
                .ok();
        });

        Ok(())
    }

    fn copy_dir_recursive(src: Arc<Path>, dst: Arc<Path>) -> ClipBoardResult<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                Self::copy_dir_recursive(src_path.into(), dst_path.into())?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    fn move_to_trash(
        &self,
        items: Vec<(Arc<str>, Arc<Path>)>,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        let task_id = new_task_id();
        let backend = get_backend();

        let mut resolved: Vec<(Arc<str>, Arc<Path>, TrashDestination)> = Vec::new();

        for (name, full_path) in &items {
            let destination = backend
                .resolve_destination(full_path)
                .map_err(ClipBoardError::TrashError)?;

            resolved.push((name.clone(), full_path.to_owned(), destination));
        }

        sender
            .send(TaskMessage::Started {
                task_id,
                text: "Moviendo a la papelera...".to_string(),
                task_type: TaskType::MoveTrash,
            })
            .ok();

        let sender = sender.clone();
        TOKIO_RUNTIME.spawn(async move {
            Self::process_trash_operations(task_id, resolved, backend, &sender);
        });

        Ok(())
    }

    fn process_trash_operations(
        task_id: u64,
        resolved: Vec<(Arc<str>, Arc<Path>, TrashDestination)>,
        backend: &dyn TrashBackend,
        sender: &Dispatcher,
    ) {
        let total = resolved.len();
        let mut errors: Vec<ClipBoardError> = Vec::new();

        let mut trash_paths: Vec<Arc<Path>> = Vec::new();
        let mut names = Vec::new();
        let mut is_permanent = false;

        for (done, (name, source, destination)) in resolved.into_iter().enumerate() {
            let result = if backend.is_in_trash(&source) {
                is_permanent = true;
                backend
                    .permanently_delete(&source)
                    .map_err(ClipBoardError::TrashError)
            } else {
                backend
                    .move_to_trash(&source)
                    .map(|_| ())
                    .map_err(ClipBoardError::TrashError)
            };

            if result.is_ok() && !is_permanent {
                names.push(name.to_string());
                match backend.get_trash_root(&destination) {
                    Ok(trash_path) => {
                        let file_trash_path = trash_path.join("files").join(name.to_string());
                        trash_paths.push(file_trash_path.into());
                    }
                    Err(e) => warn!("Ha ocurrido un error obteniendo las rutas de trash: {}", e),
                }
            } else if let Err(e) = result {
                warn!("{}", e);
                errors.push(e);
            }

            sender
                .send(TaskMessage::Progress {
                    task_id,
                    progress: (done + 1) as f32 / total as f32,
                    text: format!("Procesando {}/{}...", done + 1, total),
                    task_type: TaskType::MoveTrash,
                })
                .ok();
        }

        if !trash_paths.is_empty() {
            with_history(|h| {
                h.push(UndoRecord::RestoreFromTrash {
                    file_names: names,
                    trash_paths,
                })
            });
        }

        sender
            .send(TaskMessage::Finished {
                task_id,
                success: errors.is_empty(),
                task_type: TaskType::MoveTrash,
                text: if errors.is_empty() {
                    "Listo".to_string()
                } else {
                    format!("Completado con {} error(es)", errors.len())
                },
            })
            .ok();
    }

    pub fn restore_items(
        &self,
        items_to_restore: Vec<String>,
        trash_root: Arc<Path>,
        sender: Dispatcher,
    ) -> ClipBoardResult<()> {
        let task_id = new_task_id();

        sender
            .send(TaskMessage::Started {
                task_id,
                text: "Restaurando desde la papelera...".to_string(),
                task_type: TaskType::RestoreTrash,
            })
            .ok();

        let trash_files_dir = trash_root.join("files");
        let backend = get_backend();

        TOKIO_RUNTIME.spawn(async move {
            let total = items_to_restore.len();
            let mut errors = Vec::new();

            for (done, name) in items_to_restore.iter().enumerate() {
                let tras_item_path = trash_files_dir.join(name);
                match backend.restore_from_trash(&tras_item_path) {
                    Ok(final_path) => {
                        println!("Restaurado: {} → {:?}", name, final_path);
                    }
                    Err(e) => {
                        warn!("Ha ocurrido error: {}", e);
                        errors.push(format!("{}: {}", name, e));
                    }
                }

                sender
                    .send(TaskMessage::Progress {
                        task_id,
                        progress: (done + 1) as f32 / total as f32,
                        text: "Restaurando...".to_string(),
                        task_type: TaskType::RestoreTrash,
                    })
                    .ok();
            }

            sender
                .send(TaskMessage::Finished {
                    task_id,
                    success: errors.is_empty(),
                    task_type: TaskType::RestoreTrash,
                    text: if errors.is_empty() {
                        "Elementos restaurados correctamente".to_string()
                    } else {
                        format!("Restauración completada con {} error(es)", errors.len())
                    },
                })
                .ok();
        });

        Ok(())
    }

    pub fn rename_file(
        &self,
        file_name: &str,
        new_file_name: &str,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        info!("Renombrando");
        let new_name_trimmed = new_file_name.trim();
        if new_name_trimmed.is_empty() {
            return Err(ClipBoardError::InvalidName("no puede estar vacío".into()));
        }
        if new_name_trimmed.contains("/") || new_name_trimmed.contains("\\") {
            return Err(ClipBoardError::InvalidName(
                "no puede contener barras: '\\' - '/' ".into(),
            ));
        }

        let cwd = with_motor(|m| m.active_tab_mut().cwd.clone());
        let file_path = cwd.join(file_name);
        let new_file_path = cwd.join(new_file_name);

        if new_file_path.exists() && file_name != new_file_name {
            let same_file = file_path.canonicalize().ok() == new_file_path.canonicalize().ok();
            // canonicalize() ayuda a comparar si son físicamente el mismo lugar en el disco
            if !same_file {
                return Err(ClipBoardError::AlreadyExist(new_file_name.into()));
            }
        };

        match fs::rename(&file_path, &new_file_path) {
            Ok(_) => {
                sender
                    .send(FileOperation::Rename {
                        original_path: file_path.into(),
                        new_path: new_file_path.into(),
                    })
                    .ok();
                Ok(())
            }
            Err(e) => Err(ClipBoardError::Io(e)),
        }
    }

    pub fn create_new_dir(
        &self,
        file_name: &str,
        current_cwd: Arc<Path>,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        let new_name_trimmed = file_name.trim();
        if new_name_trimmed.is_empty() {
            return Err(ClipBoardError::InvalidName("no puede estar vacío".into()));
        }
        if new_name_trimmed.contains("/") || new_name_trimmed.contains("\\") {
            return Err(ClipBoardError::InvalidName(
                "no puede contener barras: '\\' - '/' ".into(),
            ));
        }

        if let Ok(meta) = current_cwd.metadata()
            && meta.permissions().readonly()
        {
            return Err(ClipBoardError::PermissionDenied(current_cwd));
        }

        let final_path_name = current_cwd.join(file_name);
        if final_path_name.exists() {
            return Err(ClipBoardError::AlreadyExist(file_name.into()));
        }

        match fs::create_dir(&final_path_name) {
            Ok(_) => {
                sender
                    .send(FileOperation::CreateDir {
                        path: final_path_name.into(),
                    })
                    .ok();
                Ok(())
            }
            Err(e) => Err(ClipBoardError::Io(e)),
        }
    }

    pub fn create_new_file(
        &self,
        file_name: &str,
        current_cwd: Arc<Path>,
        sender: &Dispatcher,
    ) -> ClipBoardResult<()> {
        let new_name_trimmed = file_name.trim();
        if new_name_trimmed.is_empty() {
            return Err(ClipBoardError::InvalidName("no puede estar vacío".into()));
        }
        if new_name_trimmed.contains("/") || new_name_trimmed.contains("\\") {
            return Err(ClipBoardError::InvalidName(
                "no puede contener barras: '\\' - '/' ".into(),
            ));
        }

        if let Ok(meta) = current_cwd.metadata()
            && meta.permissions().readonly()
        {
            return Err(ClipBoardError::PermissionDenied(current_cwd));
        }

        let final_path_name = current_cwd.join(file_name);
        if final_path_name.exists() {
            return Err(ClipBoardError::AlreadyExist(file_name.into()));
        }

        match fs::File::create_new(&final_path_name) {
            Ok(_) => {
                sender
                    .send(FileOperation::CreateFile {
                        path: final_path_name.into(),
                    })
                    .ok();
                Ok(())
            }
            Err(e) => Err(ClipBoardError::Io(e)),
        }
    }
}
