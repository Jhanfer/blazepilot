use crate::core::{
    blaze_state::state_structs::ViewMode,
    files::blaze_motor::motor_structs::{FileLoadingMessage, RecursiveMessages},
    runtime::{bus_structs::FileOperation, event_bus::with_event_bus},
    system::{
        extended_info::extended_info_manager::ExtendedInfoMessages,
        operationstate::operation_manager::with_history,
        trash_manager::manager::{TrashDestination, get_backend},
    },
};
use std::{
    path::Path,
    sync::{Arc, atomic::Ordering},
};
use tracing::{debug, info, warn};

use super::BlazeCoreState;

impl BlazeCoreState {
    fn consume_file_messages(&mut self) {
        let file_messages: Vec<FileLoadingMessage> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(self.active_id(), |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        for msg in file_messages {
            match msg {
                FileLoadingMessage::Batch(id, gene, batch) => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab_mut();

                    let Some(source) = tab.sources.iter_mut().find(|s| s.id == id) else {
                        warn!(
                            "Batch de source desconocido {:?} (removido o cancelado), descartado",
                            id
                        );
                        continue;
                    };

                    source.files_just_loaded.store(false, Ordering::Relaxed);

                    debug!(
                        "Batch recibido: generation={}, tamaño={}",
                        gene,
                        batch.len()
                    );

                    if gene != source.loading_generation {
                        warn!(
                            "Generation no coincide: esperado={}, recibido={}",
                            source.loading_generation, gene
                        );

                        continue;
                    }
                    debug!("Batch aplicado a tab");

                    {
                        let mut files_guard = source.files.write();

                        let mut indices_guard = source.sorted_indices.write();

                        let start = files_guard.len();

                        files_guard.extend(batch.iter().cloned());

                        indices_guard.extend(start..files_guard.len());
                    }

                    self.needs_sort = true;
                }
                FileLoadingMessage::ProgressUpdate { total, done, text } => {
                    debug!("Progress: {} - {}", done as f32 / total as f32, text);
                }

                FileLoadingMessage::RecursiveBatch {
                    generation,
                    batch,
                    source_dir,
                } => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab_mut();

                    let Some(source) = tab.sources.iter_mut().find(|s| s.cwd == source_dir) else {
                        warn!(
                            "RecursiveBatch de source desconocido {:?}, descartado",
                            source_dir.display()
                        );
                        continue;
                    };

                    if generation == source.loading_generation {
                        {
                            let mut recursive_entries_guard = source.recursive_entries.write();

                            recursive_entries_guard.extend(batch);
                        }
                    }
                }

                FileLoadingMessage::Finished(id, gene) => {
                    {
                        let mut motor = self.motor.borrow_mut();
                        let tab = motor.active_tab_mut();

                        debug!("LOS SOURCES DISPONIBLES SON: {}", tab.sources.len());

                        let Some(source) = tab.sources.iter_mut().find(|s| s.id == id) else {
                            warn!("Finished de source desconocido {:?}, descartado", id);
                            continue;
                        };

                        debug!("Finished recibido: generation={}", gene);
                        if gene == source.loading_generation {
                            debug!("Finished aplicado a tab");

                            source.active_generation = gene;
                            source.loading_flag.store(false, Ordering::Relaxed);
                            self.is_loading = false;

                            if source.is_recursive_active {
                            } else {
                                {
                                    let mut files_guard = source.files.write();

                                    files_guard.shrink_to_fit();

                                    source.lower_names.clear();
                                    source.lower_names.extend(
                                        files_guard.iter().enumerate().map(|(i, e)| {
                                            (i, e.name.to_lowercase().into_boxed_str())
                                        }),
                                    );
                                    source.lower_names.shrink_to_fit();
                                }
                                self.needs_sort = true;
                            }

                            source.files_just_loaded.store(true, Ordering::Relaxed);
                        }
                    }

                    if self.is_miller() {
                        self.sync_columns_from_sources();
                    }
                }

                FileLoadingMessage::FileRemoved { name } => {
                    debug!("- Archivo eliminado: {}", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                }
                FileLoadingMessage::FileAdded { name } => {
                    debug!("+ Archivo añadido: {}", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                }
                FileLoadingMessage::FileModified { name } => {
                    debug!("Archivo {} modificado", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                }
                FileLoadingMessage::FullRefresh => {
                    debug!("FullRefresh solicitado");
                    self.last_fs_event = Some(std::time::Instant::now());
                    self.reload_all();
                }

                FileLoadingMessage::GitStatusChanged => {
                    let motor = self.motor.borrow();
                    let tab = motor.active_tab();

                    for source in &tab.sources {
                        {
                            let files_guard = source.files.read();

                            let paths: Vec<Arc<Path>> =
                                files_guard.iter().map(|f| f.full_path.clone()).collect();

                            let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
                            for path in paths {
                                dispatcher.send(ExtendedInfoMessages::ForceScan(path)).ok();
                            }
                        }
                    }
                }
            }
        }
    }

    fn consume_recursive_messages(&mut self) {
        let recursive_messages: Vec<RecursiveMessages> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(self.active_id(), |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        for msg in recursive_messages {
            match &msg {
                RecursiveMessages::Started { .. } => {
                    debug!("Started: {:?}", msg);
                    self.is_loading = true;
                }
                RecursiveMessages::Progress { .. } => {
                    debug!("Progress: {:?}", msg);
                }
                RecursiveMessages::Finished { .. } => {
                    debug!("Finished: {:?}", msg);
                    self.is_loading = false;
                }
            }
        }
    }

    fn consume_fileops_messages(&mut self) {
        let fileops_events: Vec<FileOperation> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(self.active_id(), |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        for msg in fileops_events {
            match msg {
                // Operaciones de Archivos
                // __--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--
                FileOperation::PasteCut { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::PasteCopy { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::Rename { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::CreateDir { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::CreateFile { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::Move { sources, dest, .. } => {
                    self.move_files(sources, dest);
                }

                FileOperation::Trash { files } => {
                    self.move_to_trash(files);
                }

                FileOperation::RestoreDeletedFiles { file_names } => {
                    let Some(trash_path) =
                        get_backend().get_trash_files(&TrashDestination::Home).ok()
                    else {
                        return;
                    };

                    if let Some(trash_root) = trash_path.parent() {
                        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));
                        self.clipboard
                            .restore_from_trash(file_names, trash_root.into(), dispatcher)
                            .ok();
                    }
                }

                FileOperation::ExtractHere { entry, dest_dir } => {
                    info!(
                        "Solicitando extracción de: [{}] -> [{:?}]",
                        entry.name, dest_dir
                    );
                    let res = self.zip_manager.extract(&entry, &dest_dir);
                    res.map_err(|e| warn!("Error: {}", e)).ok();
                }

                // Operaciones Extra
                // __--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--
                FileOperation::Update => {
                    self.updater.start_update_process();
                }

                FileOperation::UpdateDirSize {
                    full_path,
                    size,
                    tab_id,
                } => {
                    let mut motor = self.motor.borrow_mut();

                    if let Some(tab) = motor.tabs.iter_mut().find(|t| t.id == tab_id)
                        && let Some(source) = tab.sources.iter_mut().find(|s| s.cwd == tab.focused)
                        && let Ok(e) = source.update_dir_size(full_path.to_owned(), size)
                        && e
                    {
                        self.dir_sizes.finish(full_path);
                        self.needs_sort = true;
                    }
                }

                FileOperation::ExtendedInfoReady { full_path } => {
                    self.extended_info.finish(full_path);
                }

                FileOperation::NavigateTo(path) => {
                    let current_layout = match &self.view_mode {
                        ViewMode::Normal(layout) => layout,
                        ViewMode::Tags(layout) => layout,
                    };

                    self.view_mode = ViewMode::Normal(current_layout.to_owned());
                    self.navigate_to(path);
                }

                FileOperation::OpenFileByPath(path) => {
                    self.open_file_by_path(path);
                }
            }
        }
    }

    pub fn process_messages(&mut self) {
        let active_id = {
            let motor = self.motor.borrow();
            let tab = motor.active_tab();
            tab.id
        };

        self.task_manager.process_message(active_id);

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

        self.sizer_manager
            .process_messages(active_id, dispatcher.clone());

        if let Err(e) = self
            .extended_info_manager
            .process_messages(active_id, dispatcher.clone())
        {
            warn!("Error procesando mensajes de ExtendedInfo: {}", e);
        }

        self.consume_file_messages();

        self.consume_recursive_messages();

        self.consume_fileops_messages();

        if let Some(last_event) = self.last_fs_event
            && last_event.elapsed() > std::time::Duration::from_millis(50)
        {
            self.last_fs_event = None;
            if self.active_tasks == 0 {
                let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id()));

                self.dir_sizes.reset();

                let paths: Vec<Arc<Path>> = {
                    let motor = self.motor.borrow();
                    let tab = motor.active_tab();
                    tab.sources.iter().map(|s| &s.cwd).cloned().collect()
                };

                {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab_mut();

                    for path in paths {
                        if let Err(e) = tab.request_load_from_path(path.clone(), dispatcher.clone())
                        {
                            warn!("Ha ocurrido un error al cargar los archivos: {}", e);
                        }
                    }
                }
            }
        }
    }
}
