use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;
use std::{fs, path::PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;
use uuid::Uuid;
use zbus::zvariant::Str;

use crate::core::files::motor::{FileEntry, with_motor, new_task_id, TaskType};
use crate::ui::task_manager::task_manager::TaskMessage;
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;
use crate::utils::channel_pool::{NotifyingSender, UiEvent, with_channel_pool};
use tracing::{info, warn, error, debug};

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});



pub enum ConflictStrategy {
    Ask,        // preguntar al usuario (por defecto)
    Overwrite,  // sobreescribir siempre
    Rename,     // renombrar automáticamente (copia 2, copia 3...)
    Skip,       // saltar el archivo
}




#[derive(Clone, Debug)]
pub enum ClipboardMode {
    Copy, 
    Cut,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClipboardItem {
    pub src_path: PathBuf,
    pub name: String
}


#[derive(Clone, Debug)]
pub struct Clipboard {
    inner: Arc<Mutex<ClipboardInner>>,
}


#[derive(Clone, Debug)]
struct ClipboardInner {
    pub mode: Option<ClipboardMode>,
    pub items: Vec<ClipboardItem>,
    pub dest_dir: Option<PathBuf>,
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

    pub fn clipboard_has_files(&self) -> bool {
        Self::inner().clipboard_has_files()
    }

    pub fn is_in_trash(&self, file_name: String) -> Option<bool> {
        Self::inner().is_in_trash_ui(file_name)
    }

    pub fn clear(&self) {
        Self::inner().clear();
    }

    pub fn set_dest(&self, dest: PathBuf) {
        Self::inner().set_dest(dest);
    }

    pub fn is_clipboard_empty(&self) -> bool {
        Self::inner().is_clipboard_empty()
    }

    pub fn copy_items(&self, items: Vec<Arc<FileEntry>>, current_cwd: PathBuf) {
        Self::inner().copy_items(items, current_cwd);
    }

    pub fn cut_items(&self, items: Vec<Arc<FileEntry>>, current_cwd: PathBuf) {
        Self::inner().cut_items(items, current_cwd);
    }

    pub fn paste(&self, sender: NotifyingSender) -> Result<(), String> {
        Self::inner().pastex(sender)
    }

    pub fn move_files(&self, items: Vec<PathBuf>, dest: PathBuf, sender: NotifyingSender) -> Result<(), String> {
        Self::inner().move_files(items, dest, ConflictStrategy::Ask, sender)
    }

    pub fn move_to_trash(&self, items: Vec<Arc<FileEntry>>, current_cwd: PathBuf, sender: NotifyingSender) -> Result<(), String> {
        Self::inner().move_to_trash(items, current_cwd, sender)
    }

    pub fn restore_from_trash(&self, items: Vec<String>, trash_root: PathBuf, sender: NotifyingSender) -> Result<(), String>  {
        Self::inner().restore_items(items, trash_root, sender)
    }

    pub fn rename_file(&self, file_name: &str, new_file_name: &str) -> Result<(), String> {
        Self::inner().rename_file(file_name, new_file_name)
    }

    pub fn create_new_dir(&self, file_name: &str, current_cwd: PathBuf) -> Result<(), String> {
        Self::inner().create_new_dir(file_name, current_cwd)
    }

    pub fn create_new_file(&self, file_name: &str, current_cwd: PathBuf) -> Result<(), String> {
        Self::inner().create_new_file(file_name, current_cwd)
    }

}


impl Clipboard {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(
                ClipboardInner {
                    mode: None, 
                    items: Vec::new(), 
                    dest_dir: None,
                }
            ))
        }
    }

    pub fn clipboard_has_files(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        !inner.items.is_empty()
    }

    pub fn is_in_trash_ui(&self, file_name: String) -> Option<bool> {
        let (trash_dir, cwd) = with_motor(|m| (m.get_trash_dir(None).unwrap(), m.active_tab().cwd.clone()));

        let file_path = cwd.join(file_name);
        if !file_path.exists() { return Some(false); }

        file_path.canonicalize()
        .ok()
        .map(|canonical_path| canonical_path.starts_with(&*trash_dir))
    }


    fn is_in_trash(path: &Path) -> bool {
        path.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s == ".Trash" || 
            s.starts_with(".Trash-") ||
            s == "Trash"
        })
    }


    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.mode = None;
        inner.items.clear();
        inner.dest_dir = None;
    }

    pub fn is_clipboard_empty(&self) -> bool { 
        let inner = self.inner.lock().unwrap();
        inner.items.is_empty()
    }

    pub fn copy_items(&self, items: Vec<Arc<FileEntry>>, current_cwd: PathBuf) {
        let mut inner = self.inner.lock().unwrap();
        inner.mode = Some(ClipboardMode::Copy);
        self.prepare_items(items, current_cwd, &mut inner);
    }

    pub fn cut_items(&self, items: Vec<Arc<FileEntry>>, current_cwd: PathBuf) {
        let mut inner = self.inner.lock().unwrap();
        inner.mode = Some(ClipboardMode::Cut);
        self.prepare_items(items, current_cwd, &mut inner);
    }

    fn prepare_items(&self, items: Vec<Arc<FileEntry>>, current_cwd: PathBuf, inner: &mut std::sync::MutexGuard<'_, ClipboardInner>) {
        inner.items = items.iter().map(|e| ClipboardItem {
            src_path: current_cwd.join(e.name.as_ref()),
            name: e.name.to_string()
        }).collect();
        inner.dest_dir = None;
    }

    pub fn set_dest(&self, dest: PathBuf) {
        let mut inner = self.inner.lock().unwrap();
        if !dest.is_dir() || !dest.exists() { return; }
        inner.dest_dir = Some(dest);
    }


    pub fn pastex(&self, sender: NotifyingSender) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap();
        let Some(dest) = inner.dest_dir.take() else {
            warn!("No hay destio donde pegar.");
            return  Err("No hay destino".to_string());
        };

        let items = inner.items.clone();
        let mode = inner.mode.clone();
        let task_id = new_task_id();

        sender.send_tasks(
            TaskMessage::Started {
                task_id,
                text: "Pegando archivos...".to_string(),
                task_type: TaskType::CopyPaste,
            }
        ).ok();

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

            for item in items {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let dest = dest.clone();
                let mode = mode.clone();
                let sender = sender.clone();
                let errors = errors_count.clone();
                let copied_global = copied_bytes_global.clone();

                let handle = tokio::spawn(async move {
                    let _permit = permit;

                    let mut dest_path = dest.join(&item.name);

                    if dest_path.exists() {
                        match mode {
                            Some(ClipboardMode::Cut) => {
                                warn!("Destino ya existe para Cut, saltando");
                                errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                                return ;
                            },
                            _ => {
                                dest_path = Self::generate_unique_path(&dest_path);
                                info!("Renombrado: {} -> {}", item.name, dest_path.display());
                            }
                        }
                    }

                    let is_cut = matches!(mode, Some(ClipboardMode::Cut));

                    let result = Self::paste_item_with_progress(
                        &item.src_path,
                        &dest_path,
                        task_id,
                        &sender,
                        &copied_global,
                        total_bytes_global,
                        is_cut
                    ).await;


                    if let Err(err_msg) = result {
                        errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        let needs_root = err_msg.to_lowercase().contains("permission denied") || err_msg.to_lowercase().contains("permiso denegado");

                        sender.send_ui_event(
                            UiEvent::ShowError(err_msg)
                        ).ok();

                        if needs_root {
                            info!("Error de permisos detectado. Abortando tarea.");
                            return;
                        }
                    }

                });

                handles.push(handle);
            }

            for handle in handles { let _ = handle.await; }

            let has_errors = errors_count.load(std::sync::atomic::Ordering::Relaxed) > 0;

            info!("Todos los items procesados, mandando Finished");
            sender.send_tasks(
                TaskMessage::Finished {
                    task_id,
                    success: !has_errors,
                    task_type: TaskType::CopyPaste,
                    text: "Listo!".to_string(),
                }
            ).ok();
            info!("TaskMessage::Finished enviado, success={}", !has_errors);

        });
        
        Ok(())
    }



    async fn paste_item_with_progress(src: &PathBuf, dest: &PathBuf, task_id: u64, sender: &NotifyingSender, copied_global: &Arc<AtomicU64>, total_bytes_global: u64, is_cut: bool) -> Result<(), String> {

        if is_cut {
            match tokio::fs::rename(src, dest).await {
                Ok(_) => {
                    info!("Move rápido exitoso {:?}", dest.file_name());
                    return Ok(());
                },
                Err(e) => {
                    if e.raw_os_error() != Some(18) && e.kind() == std::io::ErrorKind::CrossesDevices {
                        warn!("rename falló (no es cross-device): {}", e);
                    }
                },
            }
        }
        
        let copy_result = if src.is_dir() {
            Self::copy_dir_recursive_async(src, dest, task_id, sender, copied_global, total_bytes_global).await
        } else {
            Self::copy_file_with_progress(src, dest, task_id, sender, copied_global, total_bytes_global).await
        };
        

        if let Err(e) = copy_result {
            return Err(e);
        }


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


    async fn copy_file_with_progress(src: &PathBuf, dest: &PathBuf, task_id: u64, sender: &NotifyingSender, copied_global: &Arc<AtomicU64>, total_bytes_global: u64) -> Result<(), String>{
        let mut reader = tokio::fs::File::open(src)
            .await
            .map_err(|e| format!("Error abriendo origen: {}", e))?;
        let mut writer = tokio::fs::File::create(dest)
            .await
            .map_err(|e| format!("Error creando destino: {}", e))?;

        if let Ok(metadata) = tokio::fs::metadata(src).await {
            tokio::fs::set_permissions(dest, metadata.permissions()).await.ok();
        }

        //future conservar timestamps

        let mut buffer = vec![0u8; 64 * 1024];
        let mut last_update = std::time::Instant::now();

        loop {
            let bytes_read = reader.read(&mut buffer).await.unwrap();
            if bytes_read == 0 { break; }
            
            writer.write(&buffer[..bytes_read]).await.unwrap();

            copied_global.fetch_add(bytes_read as u64, Ordering::Relaxed);

            tokio::task::yield_now().await;
            
            if last_update.elapsed().as_millis() > 100 {
                let copied = copied_global.load(Ordering::Relaxed);
                let progress = if total_bytes_global > 0 { copied as f32 / total_bytes_global as f32 } else { 0.0 };
                sender.send_tasks(
                    TaskMessage::Progress { 
                        task_id, 
                        progress, 
                        text: format!("{} MB copiados", copied as f64 / 1_000_000.0),
                        task_type: TaskType::CopyPaste
                    }
                ).ok();
                last_update = std::time::Instant::now();
                debug!("Progress durante copia: {}%", progress * 100.0);
            }
        }

        info!("Archivo copiado exitosamente: {:?}", dest.file_name());
        Ok(())
    }



    async fn copy_dir_recursive_async(src: &PathBuf, dest: &PathBuf, task_id: u64, sender: &NotifyingSender, copied_global: &Arc<AtomicU64>, total_bytes_global: u64) -> Result<(), String> {
        tokio::fs::create_dir_all(dest)
            .await
            .map_err(|e| format!("No se pudo crear carpeta: {}", e))?;


        if let Ok(metadata) = tokio::fs::metadata(src).await {
            let permissions = metadata.permissions();
            if let Err(e) = tokio::fs::set_permissions(dest, permissions).await {
                warn!("No se pudieron aplicar permisos a la carpeta '{}': {}", dest.display(), e);
            }
        }

        //future conservar timestamps

        let mut read_dir = match tokio::fs::read_dir(src).await {
            Ok(rd) => rd,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    tokio::fs::remove_dir_all(dest).await.ok();
                    return Err(format!("Permiso denegado al leer carpeta: {}", src.display()));
                }

                tokio::fs::remove_dir_all(dest).await.ok();
                return Err(format!("No se pudo leer la carpeta '{}': {}", src.display(), e));
            },
        };

        while let Some(entry) = read_dir.next_entry().await.map_err(|e|format!("Error en next_entry: {}", e))? {
            let entry_path = entry.path();
            let dst_path = dest.join(entry.file_name());

            if entry_path.is_dir() {
                Box::pin(Self::copy_dir_recursive_async(&entry_path, &dst_path, task_id, sender, copied_global, total_bytes_global)).await?;
            } else {
                if let Err(e) = Self::copy_file_with_progress(&entry_path, &dst_path, task_id, sender, copied_global, total_bytes_global).await {
                    return  Err(e);
                }
            }
        }

        Ok(())
    }



    async fn calculate_size(path: &PathBuf) -> u64 {
        if path.is_file() {
            return tokio::fs::metadata(path).await
            .map(|m|m.len())
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
                    total += tokio::fs::metadata(&entry_path).await
                        .map(|m|m.len())
                        .unwrap_or(0);
                } else {
                    stack.push(entry_path);
                }
            };
        }

        total
    }

    pub fn generate_unique_path(path: &Path) -> PathBuf {
        if !path.exists() {
            return path.to_path_buf();
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str());

        let mut counter = 1u32;

        loop {
            let new_name = match ext {
                Some(e) => format!("{} ({}) .{}", stem, counter, e),
                None => format!("{} ({})", stem, counter),
            };

            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path;
            }
            counter += 1;

            if counter > 10_00 {
                return parent.join(format!("{}_{}", stem, uuid::Uuid::new_v4()));
            }
        }
    }
    

    ///Mover archivos
    fn resolve_dest_path(target: &PathBuf) -> PathBuf {
        if !target.exists() {
            return target.clone();
        }

        let stem = target.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("archivo");
        let ext = target.extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        let parent = target.parent().unwrap_or(std::path::Path::new(""));

        let mut counter = 2;
        loop {
            let new_name = format!("{} ({}){}", stem, counter, ext);
            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path;
            }
            counter += 1;
        }
    }


    pub fn move_files(&self, items: Vec<PathBuf>, dest: PathBuf, conflict: ConflictStrategy, sender: NotifyingSender) -> Result<(), String> {
        let task_id = new_task_id();

        sender.send_tasks(TaskMessage::Started { 
            task_id, 
            text: "Moviendo archivos...".to_string(), 
            task_type: TaskType::CopyPaste,
        }).ok();

        TOKIO_RUNTIME.spawn(async move {
            let total = items.len();
            let mut errors = Vec::new();

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
                        ConflictStrategy::Overwrite => target,
                        ConflictStrategy::Rename => Self::resolve_dest_path(&target),
                        ConflictStrategy::Skip => {
                            sender.send_tasks(TaskMessage::Progress {
                                task_id,
                                progress: (done + 1) as f32 / total as f32,
                                text: format!("Omitido: {}", file_name.to_string_lossy()),
                                task_type: TaskType::CopyPaste,
                            }).ok();
                            continue;
                        },
                        ConflictStrategy::Ask => {
                            
                            // sender.send_ui_event(UiEvent::FileConflict {
                            //     task_id,
                            //     source: source.clone(),
                            //     target: target.clone(),
                            // }).ok();

                            info!("ya existe archivo con este nombre!");
                            continue;
                        }
                    }
                } else {
                    target
                };



                let result = fs::rename(source, &final_target).or_else(|_|{
                    if source.is_dir() {
                        Self::copy_dir_recursive(source, &final_target)
                            .and_then(|_| fs::remove_dir_all(source))
                    } else {
                        fs::copy(source, &final_target)
                            .map(|_| ())
                            .and_then(|_| fs::remove_file(source))
                    }
                });
                
                if let Err(e) = result {
                    errors.push(format!("Error moviendo '{:?}': {}", file_name, e));
                }

                sender.send_tasks(TaskMessage::Progress {
                    task_id,
                    progress: (done + 1) as f32 / total as f32,
                    text: format!("Moviendo {}...", file_name.to_string_lossy()),
                    task_type: TaskType::CopyPaste,
                }).ok();
            }

            sender.send_tasks(TaskMessage::Finished {
                task_id,
                success: errors.is_empty(),
                task_type: TaskType::CopyPaste,
                text: if errors.is_empty() {
                    "Archivos movidos".to_string()
                } else {
                    format!("{} errores al mover", errors.len())
                },
            }).ok();
        });

        Ok(())
    }

    fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), std::io::Error> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }


    fn generate_unique_trash_path(original_path: &Path, trash_files_dir: &Path) -> PathBuf {
        let stem = original_path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = original_path.extension().and_then(|s| s.to_str());

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
    
        let base = format!("{}_{}{}", stem, now.as_secs(), now.subsec_nanos() / 1_000_000);
        
        let candidate = match ext {
            Some(e) => format!("{}.{}", base, e),
            None => base.clone(),
        };

        let mut new_path = trash_files_dir.join(candidate);
        if !new_path.exists() {
            return new_path;
        }

        let mut counter = 1u32;

        loop {
            let candidate = match ext {
                Some(e) => format!("{}_{}.{}", base, counter, e),
                None => format!("{}_{}", base, counter),
            };

            new_path = trash_files_dir.join(candidate);
            if !new_path.exists() {
                return new_path;
            }
            counter += 1;

            if counter > 1000 {
                return trash_files_dir.join(format!("{}_{}", stem, uuid::Uuid::new_v4().simple()));
            }
        }
    }

    pub fn move_to_trash(&self, items: Vec<Arc<FileEntry>>, current_cwd: PathBuf, sender: NotifyingSender) -> Result<(), String> {
        let task_id = new_task_id();

        sender.send_tasks(
            TaskMessage::Started { 
                task_id, 
                text: "Moviendo a la papelera...".to_string(), 
                task_type: TaskType::MoveTrash,
            }
        ).ok();


        let mut items_with_trash: Vec<(Arc<FileEntry>, PathBuf, PathBuf)> = Vec::new();

        for item in &items {
            let source_path = current_cwd.join(&*item.name);
            
            let trash_root = match with_motor(|m| m.get_trash_dir(Some(&source_path))) {
                Some(dir) => dir,
                None => {
                    with_motor(|m| m.get_trash_dir(None))
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".local/share/Trash"))
                }
            };

            let trash_files_dir = trash_root.join("files");
            let trash_info_dir = trash_root.join("info");

            std::fs::create_dir_all(&trash_files_dir).ok();
            std::fs::create_dir_all(&trash_info_dir).ok();

            items_with_trash.push((item.clone(), source_path, trash_files_dir));
        }


        TOKIO_RUNTIME.spawn(async move {
            let total = items.len();
            let mut errors = Vec::new();

            for (done, (item, source_path, trash_files_dir)) in items_with_trash.into_iter().enumerate() {
                if Self::is_in_trash(&source_path) {
                    let r = if source_path.is_dir() {
                        std::fs::remove_dir_all(&source_path)
                    } else {
                        std::fs::remove_file(&source_path)
                    };

                    if let Err(e) = r {
                        errors.push(format!("Error borrando '{}': {}", item.name, e));
                    }

                    let info_dir = trash_files_dir.parent().unwrap_or(&trash_files_dir).join("info");
                    if let Some(name) = source_path.file_name() {
                        let info_path = info_dir.join(format!("{}.trashinfo", name.to_string_lossy()));
                        std::fs::remove_file(info_path).ok();
                    }
                } else {

                    if let Err(e) = trash::delete(&source_path) {
                        if item.size > 256_000_000 {
                            let dest_path = Self::generate_unique_trash_path(&source_path, &trash_files_dir);
                            let info_dir = trash_files_dir.parent().unwrap_or(&trash_files_dir).join("trashinfo");

                            if let Err(e) = std::fs::create_dir_all(&info_dir) {
                                errors.push(format!("Error creando directorio info: {}", e));
                                continue;
                            }

                            let deletion_time = chrono::Local::now().to_rfc3339();
                            let trash_info_content = format!(
                                "[Trash Info]\nPath={}\nDeletionDate={}\n",
                                source_path.to_string_lossy(),
                                deletion_time
                            );

                            if let Err(e) = std::fs::rename(&source_path, &dest_path) {
                                if e.kind() == std::io::ErrorKind::NotFound {
                                    errors.push(format!("Archivo no encontrado: {:?}", source_path));
                                } else {
                                    std::fs::copy(&source_path, &dest_path)
                                        .and_then(|_| std::fs::remove_file(&source_path)).ok();
                                }
                            }

                            let info_filename = dest_path.file_name()
                                .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                                .to_string_lossy();
                            let info_path = info_dir.join(format!("{}.trashinfo", info_filename));

                            if let Err(e) = std::fs::write(&info_path, trash_info_content) {
                                errors.push(format!("Error escribiendo .trashinfo: {}", e));
                            }
                        } else {
                            errors.push(format!("Error moviendo '{}' a la papelera: {}", item.name, e));
                        }
                    }
                }

                sender.send_tasks(TaskMessage::Progress { 
                    task_id, 
                    progress: (done + 1) as f32 / total as f32, 
                    text: "Procesando...".to_string(),
                    task_type: TaskType::MoveTrash,
                }).ok();
            }

            sender.send_tasks(
                TaskMessage::Finished { 
                    task_id, 
                    success: errors.is_empty(),
                    task_type: TaskType::MoveTrash,
                    text: if errors.is_empty() {
                        "Listo".to_string()
                    } else {
                        format!("Completado con {} errores", errors.len())
                    },
                }
            ).ok();
        });
        
        Ok(())
    }

    pub fn restore_from_trash(trashed_name: &str, trash_files_dir: &Path, trash_info_dir: &Path) -> Result<PathBuf, String> {
        let trashed_path = trash_files_dir.join(trashed_name);
        if !trashed_path.exists() {
            return Err(format!("Archivo no encontrado en la papelera: {}", trashed_name));
        }

        let info_path = trash_info_dir.join(format!("{}.trashinfo", trashed_name));
        if !info_path.exists() {
            return Err(format!("No se encontró el archivo .trashinfo para {}", trashed_name));
        }

        let content = std::fs::read_to_string(&info_path)
            .map_err(|e| format!("Error leyendo .trashinfo: {}", e))?;

        let original_path_str = content
            .lines()
            .find(|line| line.starts_with("Path="))
            .and_then(|line| line.strip_prefix("Path="))
            .map(|s| s.trim())
            .ok_or("No se encontró la línea 'Path=' en el .trashinfo")?;

        let mut original_path = PathBuf::from(original_path_str);

        if let Some(parent) = original_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("No se pudo crear la carpeta original: {}", e))?;
            }
        }

        if original_path.exists() {
            original_path = Self::generate_unique_path(&original_path);
        }

        std::fs::rename(&trashed_path, &original_path)
            .or_else(|_| {
                std::fs::copy(&trashed_path, &original_path)
                    .and_then(|_| std::fs::remove_file(&trashed_path))
            })
            .map_err(|e| format!("Error restaurando '{}': {}", trashed_name, e))?;

        std::fs::remove_file(&info_path).ok();

        Ok(original_path)
    }


    pub fn restore_items(&self, items_to_restore: Vec<String>, trash_root: PathBuf, sender: NotifyingSender) -> Result<(), String> {
        let task_id = new_task_id();

        sender.send_tasks(TaskMessage::Started {
            task_id,
            text: "Restaurando desde la papelera...".to_string(),
            task_type: TaskType::RestoreTrash,
        }).ok();

        let trash_files_dir = trash_root.join("files");
        let trash_info_dir = trash_root.join("info");

        TOKIO_RUNTIME.spawn(async move {
            let total = items_to_restore.len();
            let mut errors = Vec::new();

            for (done, name) in items_to_restore.iter().enumerate() {
                match Self::restore_from_trash(name, &trash_files_dir, &trash_info_dir) {
                    Ok(final_path) => {
                        println!("Restaurado: {} → {:?}", name, final_path);
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", name, e));
                    }
                }

                sender.send_tasks(TaskMessage::Progress {
                    task_id,
                    progress: (done + 1) as f32 / total as f32,
                    text: "Restaurando...".to_string(),
                    task_type: TaskType::RestoreTrash,
                }).ok();
            }

            sender.send_tasks(TaskMessage::Finished {
                task_id,
                success: errors.is_empty(),
                task_type: TaskType::RestoreTrash,
                text: if errors.is_empty() {
                    "Elementos restaurados correctamente".to_string()
                } else {
                    format!("Restauración completada con {} error(es)", errors.len())
                },
            }).ok();
        });

        Ok(())
    }


    pub fn rename_file(&self, file_name: &str, new_file_name: &str) -> Result<(), String> {
        info!("Renombrando");
        let new_name_trimmed = new_file_name.trim();
        if new_name_trimmed.is_empty() {
            return Err("El nombre no puede estar vacío".to_string());
        }
        if new_name_trimmed.contains("/") || new_name_trimmed.contains("\\") {
            return Err("El nombre no puede contener barras".to_string());
        }

        let cwd = with_motor(|m|m.active_tab_mut().cwd.clone());
        let file_path = cwd.join(file_name);
        let new_file_path = cwd.join(new_file_name);


        if new_file_path.exists() && file_name != new_file_name   {
            let same_file = file_path.canonicalize().ok() == new_file_path.canonicalize().ok();
            // canonicalize() ayuda a comparar si son físicamente el mismo lugar en el disco
            if !same_file {
                return Err(format!("Ya existe un archivo llamado {}", new_file_name));
            }  
        };

        match fs::rename(file_path, &new_file_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error al renombrar: {}", e)),
        }
    }

    pub fn create_new_dir(&self, file_name: &str, current_cwd: PathBuf) -> Result<(), String> {
        let new_name_trimmed = file_name.trim();
        if new_name_trimmed.is_empty() {
            return Err("El nombre no puede estar vacío.".to_string());
        }
        if new_name_trimmed.contains("/") || new_name_trimmed.contains("\\") {
            return Err("El nombre no puede contener barras.".to_string());
        }

        if let Ok(meta) = current_cwd.metadata() {
            if meta.permissions().readonly() {
                return Err("No tienes los permisos para crear ficheros.".to_string());
            }
        }

        let final_path_name = current_cwd.join(file_name);
        if final_path_name.exists() {
            return Err("Ya existe un fichero con ese nombre.".to_string());
        }
        
        match fs::create_dir(final_path_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error al crear fichero. {}", e))
        } 
    }


    pub fn create_new_file(&self, file_name: &str, current_cwd: PathBuf) -> Result<(), String> {
        let new_name_trimmed = file_name.trim();
        if new_name_trimmed.is_empty() {
            return Err("El nombre no puede estar vacío.".to_string());
        }
        if new_name_trimmed.contains("/") || new_name_trimmed.contains("\\") {
            return Err("El nombre no puede contener barras.".to_string());
        }

        if let Ok(meta) = current_cwd.metadata() {
            if meta.permissions().readonly() {
                return Err("No tienes los permisos para crear ficheros.".to_string());
            }
        }

        let final_path_name = current_cwd.join(file_name);
        if final_path_name.exists() {
            return Err("Ya existe un fichero con ese nombre.".to_string());
        }
        
        match fs::File::create_new(final_path_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error al crear fichero. {}", e))
        } 
    }

}