use std::path::{PathBuf, Path};
use std::sync::{Arc, OnceLock};
use crate::core::system::trash_manager::error::{TrashError, TrashResult};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrashDestination {
    Home,
    External { mount_point: Arc<Path> },
}


pub trait TrashBackend: Send + Sync + std::fmt::Debug {

    fn is_in_trash(&self, path: &Path) -> bool;

    fn etched_in_trash_path(&self, path: &Path) -> bool;

    //En linux deteca si está en Home o en externo, en Win y Mac por volumenes
    fn resolve_destination(&self, file_path: &Path) -> TrashResult<TrashDestination>;

    fn move_to_trash(&self, source: &Path) -> TrashResult<PathBuf>;

    fn restore_from_trash(&self, trash_path: &Path) -> TrashResult<PathBuf>;

    fn permanently_delete(&self, trash_path: &Path) -> TrashResult<()>;

    //limpiar papelera
    fn empty_trash(&self) -> TrashResult<()>;

    //ruta base de la papelera para este destino
    fn get_trash_root(&self, destination: &TrashDestination) -> TrashResult<Arc<Path>>;

    fn get_trash_files(&self, destination: &TrashDestination) -> TrashResult<Arc<Path>>;
}



static TRASH_BACKEND: OnceLock<Box<dyn TrashBackend>> = OnceLock::new();


pub fn get_backend() -> &'static dyn TrashBackend {
    let boxed_backend = TRASH_BACKEND.get().expect("TrashBackend no inicializado.");
    &**boxed_backend
}


pub fn init_trash_backend() -> TrashResult<()> {
    let backend: Box<dyn TrashBackend> = if cfg!(target_os = "linux") {
        Box::new(
            crate::core::system::trash_manager::platform::linux::LinuxTrashBackend::new()?
        )
    // } else if cfg!(target_os = "macos") {
    //     Box::new(macos::MacOsTrashBackend::new()?)
    // } else if cfg!(target_os = "windows") {
    //     Box::new(windows::WindowsTrashBackend::new()?)
    } else {
        return Err(TrashError::PlatformNotSupported);
    };
    
    TRASH_BACKEND.set(backend)
        .map_err(|_| TrashError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Trash backend already initialized"
        )))?;
    
    Ok(())
}