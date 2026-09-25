use file_id::get_file_id;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use jwalk::{Parallelism, WalkDir};
use parking_lot::RwLock;
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tracing::{debug, warn};

use crate::core::{
    bootstrap::configs::{
        config_manager::with_configs,
        platform::linux::conf_structs::{OrderingDirection, OrderingKind},
    },
    files::blaze_motor::{
        blaze_loader::BlazeLoader,
        error::{MotorError, MotorResult},
        motor_structs::{
            FileEntry, FileLoadingMessage, FileSource, FileSourceId, RecursiveMessages,
        },
        selection_state::SelectionManager,
        utilities::build_entry,
        watcher::FileWatcher,
    },
    runtime::{bus_structs::UiEvent, event_bus::Dispatcher},
    system::{clipboard::global_clipboard::TOKIO_RUNTIME, sizer_manager::manager::SizerManager},
};

// Solo para la documentación
#[allow(unused)]
use crate::core::runtime::bus_structs::FileOperation;

impl FileSource {
    /// Crea una nueva fuente de archivos para `path`
    ///
    /// Inicializa su estado de carga y ordenamiento,
    /// genera un identificador único y comienza
    /// a observar cambios en el sistema de archivos
    pub fn new(path: Arc<Path>) -> Self {
        // Genera identificador único
        let id = FileSourceId::new();
        Self {
            id: id.clone(),
            cwd: path,
            files: Arc::new(RwLock::new(Vec::new())),
            sorted_indices: Arc::new(RwLock::new(Vec::new())),
            selection_manager: SelectionManager::new(),
            lower_names: Vec::new(),
            recursive_entries: Arc::new(RwLock::new(Vec::new())),
            loading_generation: 0,
            active_generation: 0,
            loading_flag: Arc::new(AtomicBool::new(false)),
            watcher: FileWatcher::start(),
            loader: BlazeLoader::default(),
            is_recursive_active: false,
            files_just_loaded: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Obtiene la lista de entradas teniendo en cuenta
    /// el filtro de búsqueda y ordenamiento
    ///
    /// Filtra las entradas según `search_filter` y
    /// las organiza los índices delegando a [`Self::ensure_sorted()`]
    ///
    /// Respeta si la entrada está oculta
    ///
    /// Devuelve las entradas recursivas si el modo de búsqueda
    /// recursiva está activo
    pub(super) fn get_active_files(
        &self,
        search_filter: &str,
        needs_sort: bool,
        sizer_manager: &SizerManager,
    ) -> MotorResult<Vec<Arc<FileEntry>>> {
        let show_hidden = with_configs(|c| c.get_show_hidden_files());
        let query_lower = search_filter.to_lowercase();
        let matcher = SkimMatcherV2::default();

        if self.is_recursive_active {
            let recursive_guard = self.recursive_entries.read();

            let result = recursive_guard
                .iter()
                .filter(|f| {
                    if !show_hidden && f.is_hidden {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();

            return Ok(result);
        }

        self.ensure_sorted(needs_sort, sizer_manager)?;

        let file_guard = self.files.read();

        let indices_guard = self.sorted_indices.read();

        let sorted = indices_guard
            .iter()
            .map(|&i| file_guard[i].clone())
            .filter(|f| {
                if !show_hidden && f.is_hidden {
                    return false;
                }
                if search_filter.is_empty() {
                    return true;
                }
                matcher
                    .fuzzy_match(&f.name.to_lowercase(), &query_lower)
                    .is_some()
            })
            .collect();

        Ok(sorted)
    }

    /// Ordena las entradas según [`OrderingKind`]
    ///
    /// Recupera el modo de ordenamiento desde las configuraciones con [`with_configs()`]
    /// organiza los índices, obtiene los tamaños de las entradas usando [`Self::get_effective_size`]
    ///
    /// Muestra siempre los directorios primero
    pub(super) fn ensure_sorted(
        &self,
        needs_sort: bool,
        sizer_manager: &SizerManager,
    ) -> MotorResult<()> {
        if !needs_sort {
            return Ok(());
        }

        let mode = with_configs(|c| c.get_ordering_mode());

        let file_guard = self.files.write();
        let mut indices_guard = self.sorted_indices.write();

        let mut indices: Vec<usize> = (0..file_guard.len()).collect();

        indices.sort_by(|&a, &b| {
            let (ea, eb) = (&file_guard[a], &file_guard[b]);

            // Carpetas primero
            match (ea.is_dir(), eb.is_dir()) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            let ord = match mode.kind {
                OrderingKind::Size => {
                    let (sa, sb) = (
                        self.get_effective_size(ea, sizer_manager),
                        self.get_effective_size(eb, sizer_manager),
                    );
                    sa.cmp(&sb)
                }
                OrderingKind::Name => ea.name.to_lowercase().cmp(&eb.name.to_lowercase()),
                OrderingKind::Date => ea.modified.cmp(&eb.modified),
            };

            // Invertir si es descendente
            if mode.direction == OrderingDirection::Desc {
                ord.reverse()
            } else {
                ord
            }
        });

        *indices_guard = indices;
        Ok(())
    }

    /// Obtiene el tamaño de las entradas, evitando los directorios
    ///
    /// Recupera los tamaños de las entradas desde el
    /// caché del `SizerManager` con la ruta de la entrada como clave
    fn get_effective_size(&self, entry: &FileEntry, sizer_manager: &SizerManager) -> u64 {
        if !entry.is_dir() {
            return entry.size;
        }
        let key = entry.full_path.to_string_lossy();
        sizer_manager
            .cache_manager
            .size_cache
            .lock()
            .get(key.as_ref())
            .map(|c| c.size)
            .unwrap_or(0)
    }

    /// Actualiza el tamaño de los directorios
    ///
    /// Recupera los tamaños de los directorios desde el `SizerManager`
    /// enviado desde los canales usando [`FileOperation`]
    pub fn update_dir_size(&self, full_path: Arc<Path>, new_size: u64) -> MotorResult<bool> {
        let mut guard = self.files.write();

        if let Some(entry) = guard
            .iter_mut()
            .find(|f| *f.full_path.as_ref() == *full_path)
        {
            let mut new_entry = (**entry).clone();
            new_entry.size = new_size;
            *entry = Arc::new(new_entry);
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    /// Limpia las entradas [`FileEntry`] de la búsqueda recursiva
    pub fn clear_recursive_files(&self) -> MotorResult<()> {
        {
            let mut recursive_entries_guard = self.recursive_entries.write();
            recursive_entries_guard.clear();
            recursive_entries_guard.shrink_to_fit();
        }
        Ok(())
    }

    /// Limpia las entradas [`FileEntry`]
    pub fn clear_files(&self) -> MotorResult<()> {
        {
            let mut file_guard = self.files.write();
            file_guard.clear();
            file_guard.shrink_to_fit();
        }
        Ok(())
    }

    /// Limpia los índices del ordenamiento
    pub fn clear_sorted_indices(&self) -> MotorResult<()> {
        {
            let mut sorted_indices_guard = self.sorted_indices.write();
            sorted_indices_guard.clear();
            sorted_indices_guard.shrink_to_fit();
        }
        Ok(())
    }

    /// Llama a la limpieza de las entradas [`FileEntry`],
    /// los índices de ordenamiento, entradas [`FileEntry`]
    /// recursivas y nombres de entradas normalizados
    pub fn reset_for_new_path(&mut self) -> MotorResult<()> {
        self.clear_files()?;
        self.clear_sorted_indices()?;
        self.clear_recursive_files()?;
        self.lower_names.clear();
        self.lower_names.shrink_to_fit();
        Ok(())
    }

    /// Carga las entradas de [`Self::cwd`] y comienza a observar sus cambios
    ///
    /// Valida que la ruta sea un directorio, reinicia el estado de la fuente
    /// para la nueva ruta y actualiza la generación de carga
    pub fn load_path(&mut self, sender: Dispatcher) -> MotorResult<()> {
        let path = self.cwd.clone();

        if !path.exists() || !path.is_dir() {
            return Err(MotorError::InvalidPath(path));
        }

        self.loading_generation += 1;

        self.reset_for_new_path()?;

        self.active_generation = 0;
        self.loader.load_path(
            path.clone(),
            sender.clone(),
            self.id.clone(),
            self.loading_generation,
        )?;

        self.watcher.start_watching(path, sender)
    }

    /// Ejecuta una búsqueda recursiva en segundo plano
    ///
    /// Recorre [`Self::cwd`] hasta `max_depth`, filtra las entrada según `query`
    /// y `show_hidden` y envía los resultados por lotes mediante el `sender`
    ///
    /// La búsqueda utiliza `loading_generation` para asociar sus resultados con
    /// la generación de carga correspondiente y `flag` para permitir su cancelación
    fn recursive_search(
        cwd: Arc<Path>,
        query: String,
        max_depth: usize,
        sender: Dispatcher,
        show_hidden: bool,
        loading_generation: u64,
        flag: Arc<AtomicBool>,
    ) {
        TOKIO_RUNTIME.spawn(async move {
            let query_lower = query.to_lowercase().trim().to_string();
            let mut total_files = 0usize;
            let mut batch: Vec<Arc<FileEntry>> = Vec::with_capacity(150);

            sender
                .send(RecursiveMessages::Started {
                    task_id: loading_generation,
                    text: format!("Buscando \"{}\"...", query),
                })
                .ok();

            let cwd_clone = cwd.clone();
            let flag_clone = flag.clone();
            let sender_clone = sender.clone();

            let walk_result = tokio::task::spawn_blocking(move || {
                let walker = WalkDir::new(&cwd_clone)
                    .max_depth(max_depth)
                    .follow_links(false)
                    .skip_hidden(!show_hidden)
                    .parallelism(Parallelism::RayonNewPool(0));

                for entry in walker {
                    if !flag_clone.load(Ordering::Relaxed) {
                        return (vec![], total_files);
                    }

                    let entry = match entry {
                        Ok(e) => e,
                        Err(e) => {
                            warn!("Error caminando: {}", e);
                            continue;
                        }
                    };

                    let path = entry.path();

                    if entry.file_type().is_dir() {
                        continue;
                    }

                    if !show_hidden
                        && let Some(name) = path.file_name()
                        && name.to_string_lossy().starts_with('.')
                    {
                        continue;
                    }

                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();

                    let is_match = query_lower.is_empty() || name_lower.contains(&query_lower) || {
                        let name_norm = name_lower.replace(['-', '_', ' ', '.'], "");
                        let query_norm = query_lower.replace(['-', '_', ' ', '.'], "");
                        name_norm.contains(&query_norm)
                    };

                    if is_match && let Ok(metadata) = entry.metadata() {
                        let entry_path = path.to_path_buf();
                        let unique_id = get_file_id(&entry_path).ok();

                        let file_entry = build_entry(&entry_path, metadata, unique_id);

                        let arc_entry = Arc::from(file_entry);

                        batch.push(arc_entry);
                        total_files += 1;

                        if batch.len() >= 150 {
                            let send_batch = std::mem::take(&mut batch);
                            sender_clone
                                .send(FileLoadingMessage::RecursiveBatch {
                                    generation: loading_generation,
                                    batch: send_batch,
                                    source_dir: cwd_clone.clone(),
                                })
                                .ok();
                        }
                    }
                }
                (batch, total_files)
            })
            .await;

            match walk_result {
                Ok((remaining_batch, found_total)) => {
                    total_files = found_total;

                    if !remaining_batch.is_empty() {
                        sender
                            .send(FileLoadingMessage::RecursiveBatch {
                                generation: loading_generation,
                                batch: remaining_batch,
                                source_dir: cwd,
                            })
                            .ok();
                    }

                    sender
                        .send(RecursiveMessages::Finished {
                            task_id: loading_generation,
                            success: true,
                            text: format!("Completado: {} archivos encontrados", total_files),
                        })
                        .ok();

                    debug!("Búsqueda recursiva completada: {} archivos", total_files);
                }
                Err(e) => {
                    sender
                        .send(UiEvent::ShowError(
                            format!("Error buscando archivos: {}", e).into(),
                        ))
                        .ok();
                }
            }

            flag.store(false, std::sync::atomic::Ordering::Relaxed);
            debug!("Búsqueda recursiva completada: {} archivos", total_files);
        });
    }

    /// Inicia una búsqueda recursiva desde el directorio [`Self::cwd`] actual
    ///
    /// Limpia los resultados de búsqueda anteriores, activa el modo búsqueda
    /// recursiva y actualiza la generación de carga antes de iniciar la búsqueda
    /// en segundo plano mediante [`Self::recursive_search`]
    pub fn start_recursive_search(
        &mut self,
        query: String,
        max_depth: usize,
        sender: Dispatcher,
    ) -> MotorResult<()> {
        {
            let mut recursive_entries_guard = self.recursive_entries.write();
            recursive_entries_guard.clear();
            recursive_entries_guard.shrink_to_fit();
        }

        self.is_recursive_active = true;

        self.loading_generation += 1;
        let current_generation = self.loading_generation;
        self.loading_flag.store(true, Ordering::Relaxed);

        let path = self.cwd.clone();
        let flag = self.loading_flag.clone();

        let show_hidden = with_configs(|c| c.get_show_hidden_files());

        Self::recursive_search(
            path,
            query,
            max_depth,
            sender,
            show_hidden,
            current_generation,
            flag,
        );

        Ok(())
    }
}
