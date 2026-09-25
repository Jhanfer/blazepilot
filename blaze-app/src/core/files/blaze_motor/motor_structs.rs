use crate::core::files::{
    blaze_motor::{
        blaze_loader::BlazeLoader, selection_state::SelectionManager, watcher::FileWatcher,
    },
    file_extension::FileExtension,
};
use file_id::FileId;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};
use uuid::Uuid;

// Solo para la documentación
#[allow(unused)]
use crate::core::runtime::event_bus::with_event_bus;

/// Tipo de operación ejecutada como tarea en segundo plano
#[derive(Debug, Clone)]
pub enum TaskType {
    #[allow(unused)]
    FileLoading,
    CopyPaste,
    #[allow(unused)]
    CutPaste,
    MoveTrash,
    #[allow(unused)]
    Delete,
    RestoreTrash,
}

/// Mensajes enviados durante la carga y actualización de archivos
#[derive(Debug, Clone)]
pub enum FileLoadingMessage {
    /// Entrega el lote de archivos asociados a una generación de carga
    Batch(FileSourceId, u64, Vec<Arc<FileEntry>>),

    /// Indica que una generación de carga está terminada
    Finished(FileSourceId, u64),

    /// Indica el progreso de una tarea. Debug
    #[allow(unused)]
    ProgressUpdate {
        total: usize,
        done: usize,
        text: String,
    },

    /// Indica cuando se añade un archivo y su nombre. Debug
    FileAdded {
        name: String,
    },

    /// Indica cuando se elimina un archivo y su nombre. Debug
    FileRemoved {
        name: String,
    },

    /// Indica cuando se modifica un archivo y su nombre. Debug
    FileModified {
        name: String,
    },

    /// Pide un refresco completo
    FullRefresh,

    /// Entrega el lote de archivos
    /// de la búsqueda recursiva asociados a
    /// una generación de carga
    RecursiveBatch {
        generation: u64,
        batch: Vec<Arc<FileEntry>>,
        #[allow(unused)]
        source_dir: Arc<Path>,
    },

    GitStatusChanged,
}

/// Mensajes enviados durante la búsqueda recursiva.
/// Por ahora no usado
#[derive(Debug, Clone)]
pub enum RecursiveMessages {
    #[allow(unused)]
    Started { task_id: u64, text: String },
    #[allow(unused)]
    Progress {
        task_id: u64,
        files_found: usize,
        current_dir: Arc<Path>,
        text: String,
    },
    #[allow(unused)]
    Finished {
        task_id: u64,
        success: bool,
        text: String,
    },
}

/// Representa una entreada del sistema de archivos
///
/// Contiene la información básica necesaria para identificar,
/// clasificar y mostrar un archivo, directorio o enlace simbólico
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: Box<str>,
    pub full_path: Arc<Path>,
    pub extension: FileExtension,
    pub kind: FileKind,
    pub size: u64,
    pub modified: u64,
    pub is_hidden: bool,

    /// Identificador único de la entrada,
    /// si el sistema de archivos proporciona uno
    pub unique_id: Option<FileId>,

    /// Marca el tiempo del último acceso
    pub accessed: u64,

    /// Marca la fecha de creación
    pub created: u64,
    /// Permisos del archivo
    pub permissions: u32,

    /// Identificador único de la entrada
    /// que el sistema asigna dentro de una
    /// misma partición
    #[allow(unused)]
    #[cfg(unix)]
    pub inode: u64,

    /// Representa el conteo de entradas de directorio
    /// que apuntan a un mismo inodo
    #[allow(unused)]
    #[cfg(unix)]
    pub nlink: u64,

    /// Identificador único del dispositivo
    /// de bloque o partición (física o virtual)
    /// donde reside físicamente el inodo
    #[allow(unused)]
    #[cfg(unix)]
    pub device: u64,

    /// Máscara de bits específica del subsistema
    /// de archivos `NTFS` en windows
    ///
    /// Cada bit de los `NTFS` actuan como banderas booleanas y
    /// describen el comportamiento del archivo
    ///
    /// `FILE_ATTRIBUTE_READONLY (0x1)`: Impide modificaciones o borrado directo sin cambiar permisos
    ///
    /// `FILE_ATTRIBUTE_HIDDEN (0x2)`: Oculta el archivo en visiones estándar
    ///
    /// `FILE_ATTRIBUTE_DIRECTORY (0x10)`: Indica que la ruta corresponde a una carpeta
    ///
    /// `FILE_ATTRIBUTE_ARCHIVE (0x20)`: Utilizado por software de respaldo para saber si el archivo ha sido modificado desde la última copia
    #[cfg(windows)]
    pub attributes: u32,
}

/// Tipos de entrada del sistema
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    /// Archivo normal
    #[default]
    File,

    /// Directorio
    Dir,

    /// Enlace simbólico
    Symlink,
}

impl Default for FileEntry {
    fn default() -> Self {
        Self {
            name: Default::default(),
            full_path: Arc::from(Path::new("")),
            extension: Default::default(),
            kind: Default::default(),
            size: Default::default(),
            modified: Default::default(),
            created: Default::default(),
            is_hidden: Default::default(),
            unique_id: Default::default(),
            accessed: Default::default(),
            permissions: Default::default(),
            inode: Default::default(),
            nlink: Default::default(),
            device: Default::default(),

            #[cfg(windows)]
            attributes: 0,
        }
    }
}

impl FileEntry {
    pub fn is_dir(&self) -> bool {
        matches!(self.kind, FileKind::Dir)
    }
}

/// Identificador único de una entrada [`FileSource`]
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct FileSourceId(pub Uuid);

impl FileSourceId {
    /// Genera un nuevo identificador aleatorio usando `UUID v4`
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FileSourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Representa una fuente de archivos y mantiene su estado de carga,
/// ordenamiento, observación de cambios y resultados de carga recursiva
pub struct FileSource {
    pub id: FileSourceId,
    pub cwd: Arc<Path>,
    pub files: Arc<RwLock<Vec<Arc<FileEntry>>>>,
    pub sorted_indices: Arc<RwLock<Vec<usize>>>,

    pub selection_manager: SelectionManager,

    /// Nombres normalizados de las entradas
    /// junto a sus índices para buscar sin
    /// distinguir entre mayúsculas y minúsculas
    pub lower_names: Vec<(usize, Box<str>)>,

    /// Entradas obtenidas mediante la búsqueda recursiva
    pub recursive_entries: Arc<RwLock<Vec<Arc<FileEntry>>>>,
    pub is_recursive_active: bool,

    /// Generación asociada a la carga actual en curso
    pub loading_generation: u64,

    /// Última generación de carga considerada activa
    pub active_generation: u64,
    pub loading_flag: Arc<AtomicBool>,

    /// Monitor de entradas
    ///
    /// Detecta cambios, modificaciones,
    /// creaciones o eliminaciones de las entradas
    pub watcher: FileWatcher,

    /// Cargador de entradas
    ///
    /// Se encarga de analizar los directorios y enviar
    /// las entradas delegando a [`with_event_bus()`] que
    /// envía [`FileLoadingMessage`]
    pub loader: BlazeLoader,

    pub files_just_loaded: Arc<AtomicBool>,
}

impl Drop for FileSource {
    fn drop(&mut self) {
        self.loader.cancel();
        self.watcher.stop_watching();
    }
}

/// Estado mínimo de un [`FileSource`] necesario
/// para restaurar una navegación de la vista miller
pub struct MillerSourceSnapshot {
    pub id: FileSourceId,
    pub cwd: Arc<Path>,
    pub files: Arc<RwLock<Vec<Arc<FileEntry>>>>,
    pub sorted_indices: Arc<RwLock<Vec<usize>>>,
}

/// Estado de navegación necesario para restaurar una
/// acción en la vista miller
pub enum MillerSnapshot {
    /// Guarda la ruta de una columna eliminada al retroceder
    Back { path: Arc<Path> },

    /// Guarda las fuentes anteriores a una navegación hacia el directorio padre
    Up {
        previous_sources: Vec<MillerSourceSnapshot>,
    },
}
