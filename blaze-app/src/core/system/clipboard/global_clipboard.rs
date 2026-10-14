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

use crate::core::files::blaze_motor::motor_structs::FileEntry;
use crate::core::runtime::event_bus::Dispatcher;
use crate::core::system::clipboard::clipboard_state::{Clipboard, ClipboardMode, ConflictStrategy};
use crate::core::system::clipboard::error::ClipBoardResult;
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;
use tracing::debug;

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(16)
        .build()
        .expect("Failed to create Tokio runtime")
});

pub struct GlobalClipboard;

impl GlobalClipboard {
    pub fn new() -> Self {
        Self
    }

    fn inner() -> &'static Clipboard {
        static INSTANCE: OnceLock<Clipboard> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            debug!("Clipboard global creado");
            Clipboard::new()
        })
    }

    pub fn clipboard_has_files(&self) -> ClipBoardResult<bool> {
        Self::inner().clipboard_has_files()
    }

    pub fn get_items(&self) -> ClipBoardResult<Vec<Box<str>>> {
        Self::inner().get_items()
    }

    pub fn get_mode(&self) -> ClipBoardResult<ClipboardMode> {
        Self::inner().get_mode()
    }

    pub fn delete_one(&self, name: Box<str>) -> ClipBoardResult<Box<str>> {
        Self::inner().delete_one(name)
    }

    pub fn is_cutted(&self, requested_item: &Arc<Path>) -> ClipBoardResult<bool> {
        Self::inner().is_cutted(requested_item)
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
        items: Vec<(Box<str>, Arc<Path>)>,
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
