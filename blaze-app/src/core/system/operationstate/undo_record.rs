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

use std::{path::Path, sync::Arc};

use crate::{
    core::{
        files::blaze_motor::{motor_structs::TaskType, tab_state::new_task_id},
        runtime::{
            bus_structs::{FileOperation, UiEvent},
            event_bus::Dispatcher,
        },
        system::clipboard::clipboard::TOKIO_RUNTIME,
    },
    ui::task_manager::task_manager::TaskMessage,
};

#[derive(Debug, Clone)]
pub enum UndoRecord {
    //reverso de mover, pegar cut
    MoveBack {
        from: Vec<Arc<Path>>,
        to: Vec<Arc<Path>>,
    },

    //reverso de copy, pegar copy
    DeleteCopied {
        paths: Vec<Arc<Path>>,
    },

    //reverso de rename
    RenameBack {
        current: Arc<Path>,
        original: Arc<Path>,
    },

    //reverso de papelera
    RestoreFromTrash {
        file_names: Vec<String>,
        trash_paths: Vec<Arc<Path>>,
    },

    //reverso de crear carpeta
    DeleteCreatedDir {
        path: Arc<Path>,
    },

    //reverso de crear archivo
    DeleteCreatedFile {
        path: Arc<Path>,
    },
}

impl UndoRecord {
    pub fn from_completed(op: &FileOperation) -> Option<Self> {
        match op {
            FileOperation::Move { .. } => None,

            FileOperation::PasteCut {
                sources,
                final_targets,
                ..
            } => Some(Self::MoveBack {
                from: final_targets.clone(),
                to: sources.clone(),
            }),

            FileOperation::PasteCopy { final_targets, .. } => Some(Self::DeleteCopied {
                paths: final_targets.clone(),
            }),

            FileOperation::Rename {
                original_path,
                new_path,
            } => Some(Self::RenameBack {
                current: new_path.clone(),
                original: original_path.clone(),
            }),

            FileOperation::CreateDir { path } => {
                Some(Self::DeleteCreatedDir { path: path.clone() })
            }
            FileOperation::CreateFile { path } => {
                Some(Self::DeleteCreatedFile { path: path.clone() })
            }

            FileOperation::Trash { .. } => None,

            _ => None,
        }
    }
}

impl UndoRecord {
    pub fn execute_undo(self, sender: &Dispatcher) {
        let sender = sender.clone();
        let task_id = new_task_id();

        sender
            .send(TaskMessage::Started {
                task_id,
                text: "Deshaciendo operación...".to_string(),
                task_type: TaskType::CopyPaste,
            })
            .ok();

        TOKIO_RUNTIME.spawn(async move {
            let result = match self {
                UndoRecord::MoveBack { from, to } => {
                    for (src, original_target) in from.iter().zip(to.iter()) {
                        if let Some(parent) = original_target.parent() {
                            let sources = vec![src.clone()];
                            let dest = Arc::from(parent);
                            sender.send(FileOperation::Move { sources, dest }).ok();
                        }
                    }
                    Ok(())
                }

                UndoRecord::DeleteCopied { paths } => {
                    let mut errors = Vec::new();

                    for path in paths {
                        let result = if path.is_dir() {
                            tokio::fs::remove_dir_all(path.as_ref()).await
                        } else {
                            tokio::fs::remove_file(path.as_ref()).await
                        };

                        if let Err(e) = result {
                            errors.push(format!("{:?}: {}", path.file_name(), e));
                        }
                    }
                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(errors.join(", "))
                    }
                }

                UndoRecord::RenameBack { current, original } => {
                    tokio::fs::rename(current.as_ref(), original.as_ref())
                        .await
                        .map_err(|e| e.to_string())
                }

                UndoRecord::RestoreFromTrash {
                    file_names,
                    trash_paths,
                } => {
                    let mut files_to_restore = vec![];

                    for (path, name) in trash_paths.iter().zip(file_names.iter()) {
                        if path.exists() {
                            files_to_restore.push(name.to_string());
                        }
                    }

                    if files_to_restore.is_empty() {
                        Ok(())
                    } else {
                        sender
                            .send(FileOperation::RestoreDeletedFiles {
                                file_names: files_to_restore,
                            })
                            .map_err(|e| e.to_string())
                    }
                }

                UndoRecord::DeleteCreatedDir { path } => tokio::fs::remove_dir(path.as_ref())
                    .await
                    .map_err(|e| e.to_string()),

                UndoRecord::DeleteCreatedFile { path } => tokio::fs::remove_file(path.as_ref())
                    .await
                    .map_err(|e| e.to_string()),
            };

            let success = result.is_ok();

            if let Err(msg) = result {
                sender.send(UiEvent::ShowError(msg.into())).ok();
            }

            sender
                .send(TaskMessage::Finished {
                    task_id,
                    success,
                    task_type: TaskType::CopyPaste,
                    text: if success {
                        "Operación deshecha".to_string()
                    } else {
                        "Error al deshacer".to_string()
                    },
                })
                .ok();
        });
    }
}
