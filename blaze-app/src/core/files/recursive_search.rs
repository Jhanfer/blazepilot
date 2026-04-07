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






use std::fs::Metadata;
use std::path::PathBuf;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use fuzzy_matcher::FuzzyMatcher;
use tokio::sync::Semaphore;
use tracing::{debug, warn};
use fuzzy_matcher::skim::SkimMatcherV2;
use tokio::fs;
use crate::core::files::motor::{FileEntry, FileKind};


#[derive(Debug, Clone)]
pub enum RecursiveMessages {
    Started {
        task_id: u64,
        text: String,
    },
    Progress {
        task_id: u64,       
        files_found: usize,  
        current_dir: PathBuf, 
        text: String, 
    },
    Finished {
        task_id: u64,
        success: bool,
        text: String,
    }
}

pub struct RecursiveSearchIterator {
    queue: VecDeque<PathBuf>,
    visited: HashSet<PathBuf>,
    current_batch: Vec<Arc<FileEntry>>,
    batch_size: usize,
    query: String,
    show_hidden: bool,
    max_depth: usize,
    pub current_dir: PathBuf,
    semaphore: Arc<Semaphore>,

    max_results: usize,
    results_found: usize,
}


impl RecursiveSearchIterator {
    pub fn new(root: PathBuf, query: String, show_hidden: bool, max_depth: usize) -> Self
    {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back(root.clone());
        visited.insert(root.clone());
        let core_count = std::thread::available_parallelism()
            .map(|c| c.get())
            .unwrap_or(4);

        Self {
            queue,
            visited,
            current_batch: Vec::with_capacity(100),
            batch_size: 100,
            query: query.to_lowercase(),
            show_hidden,
            max_depth,
            current_dir: root,
            semaphore: Arc::new(Semaphore::new(core_count)),

            max_results: 10000,
            results_found: 0,
        }
    }


    pub async fn next_batch(&mut self) -> Option<Vec<Arc<FileEntry>>> {
        if self.results_found >= self.max_results {
            return None;
        }

        let mut pending_dirs: Vec<PathBuf> = Vec::new();

        while let Some(dir) = self.queue.pop_front() {
            let depth = dir.components().count();
            if depth <= self.max_depth {
                pending_dirs.push(dir);
            }
            if pending_dirs.len() >= 8 {
                break;
            }
        }

        if pending_dirs.is_empty() {
            if !self.current_batch.is_empty() {
                return Some(std::mem::take(&mut self.current_batch));
            }
            return None;
        }

        let tasks: Vec<_> = pending_dirs
            .iter()
            .cloned()
            .map(|dir| {
                let sem = self.semaphore.clone();
                let query = self.query.clone();
                let show_hidden = self.show_hidden;
                let max_results = self.max_results;
                let results_so_far = self.results_found;

                tokio::spawn(async move {
                    let _permit = sem.acquire().await.ok();
                    Self::read_dir_entries(dir, query, show_hidden, max_results, results_so_far).await
                })

            })
            .collect();

        for (task, dir) in tasks.into_iter().zip(pending_dirs.iter()) {
            if let Ok(Some((files, subdirs))) = task.await  {
                self.current_dir = dir.clone();
                self.current_batch.extend(files);

                for subdir in subdirs {
                    if !self.visited.contains(&subdir) {
                        self.visited.insert(subdir.clone());
                        self.queue.push_back(subdir);
                    }
                }
            }
        }


        if self.results_found >= self.max_results {
            let final_batch = if !self.current_batch.is_empty() {
                Some(std::mem::take(&mut self.current_batch))
            } else {
                None
            };
            self.queue.clear();
            return final_batch;
        }

        if self.current_batch.len() >= self.batch_size {
            return Some(std::mem::take(&mut self.current_batch));
        }


        if !self.current_batch.is_empty() {
            return Some(std::mem::take(&mut self.current_batch));
        }

        None
    }


    async fn read_dir_entries(dir: PathBuf, query: String, show_hidden: bool, max_results: usize, results_so_far: usize) -> Option<(Vec<Arc<FileEntry>>, Vec<PathBuf>)> {
        let matcher = SkimMatcherV2::default();
        let mut entries = fs::read_dir(&dir).await.ok()?;

        let mut files = Vec::new();
        let mut subdirs = Vec::new();

        let remaining = max_results - results_so_far;
        if remaining == 0 {
            return Some((files, subdirs));
        }


        while let Ok(Some(entry)) = entries.next_entry().await {
            if files.len() >= remaining {
                break;
            }

            let name = entry.file_name().to_string_lossy().to_string();

            if !show_hidden && name.starts_with('.') {
                continue;
            }

            let metadata = match entry.metadata().await {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                subdirs.push(entry.path());
            } else if matcher.fuzzy_match(&name.to_lowercase(), &query).is_some() || name.to_lowercase().contains(&query) {
                files.push(Self::create_file_entry(name, metadata, entry.path()));
            }
        }

        Some((files, subdirs))
        
    }

    fn create_file_entry(name: String, metadata: Metadata, path: PathBuf) -> Arc<FileEntry> {
        let is_dir = metadata.is_dir();
        let kind = if metadata.file_type().is_symlink() {
            FileKind::Symlink
        } else if is_dir {
            FileKind::Dir
        } else {
            FileKind::File
        };

        let modified = metadata.modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

        let created = metadata.created()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

        Arc::new(FileEntry {
            name: name.clone().into_boxed_str(),
            is_dir,
            kind,
            size: metadata.len(),
            modified,
            created,
            readonly: metadata.permissions().readonly(),
            is_hidden: name.starts_with("."),
            full_path: path,
        })
    }

}