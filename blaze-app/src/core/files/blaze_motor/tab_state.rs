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

use crate::core::files::blaze_motor::error::{MotorError, MotorResult};
use crate::core::files::blaze_motor::motor_structs::{
    FileEntry, FileSource, MillerSnapshot, MillerSourceSnapshot,
};
use crate::core::runtime::event_bus::{Dispatcher, with_event_bus};
use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;
use crate::core::system::sizer_manager::manager::SizerManager;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::warn;
use uuid::Uuid;

static NEXT_TASK: AtomicU64 = AtomicU64::new(1);
pub fn new_task_id() -> u64 {
    NEXT_TASK.fetch_add(1, Ordering::Relaxed)
}

/// Constructor para crear y configurar una nueva pestaña
///
/// Permite establecer la ruta inicial y el identificador
/// de la pestaña antes de construir su [`BlazeTabState`]
#[must_use = "llama .build() para construir la pestaña"]
pub struct BlazeTabBuilder {
    start_path: Arc<Path>,
    tab_id: Uuid,
}

impl Default for BlazeTabBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BlazeTabBuilder {
    /// Crea un builder con el directorio home como ruta inicial
    /// y genera un identificador único por pestaña
    pub fn new() -> Self {
        Self {
            start_path: KnownDirsManager::get().home.clone(),
            tab_id: Uuid::new_v4(),
        }
    }

    /// Establece una ruta inicial custumizada para la pestaña
    pub fn with_start_path(mut self, path: Arc<Path>) -> Self {
        self.start_path = path;
        self
    }

    /// Establece el identificador de la pestaña
    pub fn with_uuid(mut self, id: Uuid) -> Self {
        self.tab_id = id;
        self
    }

    /// Construye el estado de la pestaña y registra
    /// su identificador en el event bus
    #[must_use = "el valor construido debe utilizarse"]
    pub fn build(self) -> BlazeTabState {
        // reistra la pestaña en el event bus antes de devolver su estado
        with_event_bus(|bus| {
            bus.create_tab(self.tab_id);
        });

        BlazeTabState {
            id: self.tab_id,
            focused: self.start_path,
            history: Vec::new(),
            future: Vec::new(),
            sources: Vec::new(),
            miller_future: Vec::new(),
            miller_history: Vec::new(),
        }
    }
}

/// Mantiene el estado de una pestaña y su navegación
///
/// Gestiona el historial de navegación de las vistas normales
/// y de las columnas miller
pub struct BlazeTabState {
    /// Identificador único de la pestaña
    pub id: Uuid,

    /// Ruta actualmente enfocada de la pestaña
    /// Importante para el request de entradas [`FileEntry`]
    pub focused: Arc<Path>,

    /// Historial de navegación utilizado para retroceder
    pub history: Vec<Arc<Path>>,

    /// Historial de navegación utilizado para avanzar
    pub future: Vec<Arc<Path>>,

    /// Fuente de [`FileSource`] asociadas a las columnas de las vistas miller
    pub sources: Vec<FileSource>,

    /// Historial de navegación utilizado por las vistas miller para retroceder
    pub(crate) miller_history: Vec<Arc<Path>>,

    /// Historial de navegación utilizado por las vistas miller para avanzar
    pub(crate) miller_future: Vec<MillerSnapshot>,
}

impl BlazeTabState {
    /// Obtiene los archivos [`FileEntry`] asociados a `path` desde su [`FileSource`]
    ///
    /// Devuelve [`MotorError::NotFileSourceFound`] si no existe una fuente
    /// asociada a la ruta
    pub fn get_files_for(
        &mut self,
        path: &Arc<Path>,
        search_filter: &str,
        needs_sort: bool,
        sizer_manager: &SizerManager,
    ) -> MotorResult<Vec<Arc<FileEntry>>> {
        let sources = self
            .sources
            .iter_mut()
            .find(|s| s.cwd == *path)
            .ok_or(MotorError::NotFileSourceFound(path.clone()))?;

        sources.get_active_files(search_filter, needs_sort, sizer_manager)
    }

    pub fn set_focus(&mut self, path: Arc<Path>) {
        self.focused = path;
    }

    /// Solicita la carga de una nueva ruta utilizando el primer [`FileSource`]
    ///
    /// Actualiza el directorio de trabajo (`cwd`) y delega la carga de los archivos
    /// al [`FileSource`]. Devuelve [`MotorError`] si no existe ninguna fuente activa
    pub fn request_load_path(&mut self, path: Arc<Path>, sender: Dispatcher) -> MotorResult<()> {
        let source = self
            .sources
            .first_mut()
            .ok_or_else(|| MotorError::from("No hay FileSource activo"))?;

        source.cwd = path.clone();
        source.load_path(sender)?;
        Ok(())
    }

    /// Solicita la carga de una nueva ruta
    ///
    /// Asegura que exista [`FileSource`]
    /// para esa ruta específica y delega la carga
    pub fn request_load_from_path(
        &mut self,
        path: Arc<Path>,
        sender: Dispatcher,
    ) -> MotorResult<()> {
        let source = self.ensure_source(path);
        source.load_path(sender)?;
        Ok(())
    }

    /// Devuelve el [`FileSource`] asociado a una ruta, creandolo si no existe
    pub fn ensure_source(&mut self, path: Arc<Path>) -> &mut FileSource {
        if let Some(idx) = self.sources.iter().position(|s| s.cwd == path) {
            return &mut self.sources[idx];
        }

        self.sources.push(FileSource::new(path));
        let idx = self.sources.len() - 1;
        &mut self.sources[idx]
    }

    /// Cambia el directorio enfocado y delega las cargas a [`Self::request_load_path`]
    ///
    /// Este método se utiliza para la navegación del modo de vista normal
    ///
    /// Actualiza el [`Self::history`] y [`Self::future`] para reflejar
    /// la nueva navegación y limita el historial a 100 rutas
    pub fn navigate_to(&mut self, new_path: Arc<Path>, sender: Dispatcher) -> MotorResult<()> {
        if new_path.is_dir() && new_path != self.focused {
            let old_path = self.focused.clone();
            if self.history.last() != Some(&old_path) {
                self.history.push(old_path);
            }

            self.future.clear();

            if self.history.len() > 100 {
                self.history.remove(0);
            }

            self.request_load_path(new_path.clone(), sender)?;
            self.set_focus(new_path);
        }

        Ok(())
    }

    /// Navega a una ruta dentro de la vista de columnas Miller
    /// Trunca los [`FileSource`] posteriores a `col_index`,
    /// actualiza el directorio enfocado y delega las cargas a [`FileSource::load_path`]
    ///
    /// Actualiza el [`Self::miller_history`] y [`Self::miller_future`] para reflejar
    /// la nueva navegación y limita el historial a 100 rutas
    ///
    /// Obtiene o crea el [`FileSource`] correspondiente
    /// mediante [`Self::ensure_source`]
    pub fn miller_navigate_to(
        &mut self,
        new_path: Arc<Path>,
        col_index: usize,
        sender: Dispatcher,
    ) -> MotorResult<()> {
        self.sources.truncate(col_index + 1);

        self.miller_future.clear();
        if self.miller_history.last() != Some(&self.focused) {
            self.miller_history.push(self.focused.clone());
        }

        if self.miller_history.len() > 100 {
            self.miller_history.remove(0);
        }

        let source = self.ensure_source(new_path);
        source.load_path(sender)?;

        Ok(())
    }

    /// Navega al directorio padre de la ruta enfocada
    ///
    /// Añade la ruta actual al historial, limpia el [`Self::future`]
    /// y carga el directorio padre mediante [`Self::request_load_path`]
    pub fn up(&mut self, sender: Dispatcher) {
        if let Some(new_path) = self.focused.parent() {
            let old_path = self.focused.clone();

            if *new_path == *old_path {
                return;
            }

            self.history.push(old_path.clone());
            self.future.clear();
            self.future.push(old_path);

            if self.history.len() > 100 {
                self.history.remove(0);
            }

            let new_path_arc: Arc<Path> = new_path.into();
            match self.request_load_path(new_path_arc.clone(), sender) {
                Ok(()) => self.set_focus(new_path_arc),
                Err(e) => warn!("Ha ocurrido un error al cargar el directorio: {e}"),
            }
        }
    }

    /// Navega a la ruta anterior del historial [`Self::history`]
    ///
    /// Añade la ruta actual al historial y carga
    /// la siguente ruta mediante [`Self::request_load_path`]
    pub fn back(&mut self, sender: Dispatcher) {
        if let Some(prev) = self.history.pop() {
            self.future.push(self.focused.clone());
            match self.request_load_path(prev.clone(), sender) {
                Ok(()) => self.set_focus(prev),
                Err(e) => warn!("Ha ocurrido un error al cargar el directorio: {e}"),
            }
        }
    }

    /// Navega a la siguiente ruta del historial [`Self::history`]
    ///
    /// Mueve la ruta actual al historial y carga
    /// la siguente ruta mediante [`Self::request_load_path`]
    pub fn forward(&mut self, sender: Dispatcher) {
        if let Some(next) = self.future.pop() {
            self.history.push(self.focused.clone());
            match self.request_load_path(next.clone(), sender) {
                Ok(()) => self.set_focus(next),
                Err(e) => warn!("Ha ocurrido un error al cargar el directorio: {e}"),
            }
        }
    }

    /// Indica si existe una ruta disponible en el historial de retroceso
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// Indica si existe una ruta disponible en el historial de avance
    pub fn can_go_forward(&self) -> bool {
        !self.future.is_empty()
    }

    /// Indica si la ruta enfocada tiene un directorio padre para navegar
    pub fn can_go_up(&self) -> bool {
        match self.focused.parent() {
            Some(parent) => parent != self.focused.iter().as_path(),
            None => false,
        }
    }

    /// Retrocede una columna en la vista miller
    ///
    /// Elimina el [`FileSource`] actual y guarda su ruta en [`Self::miller_future`]
    /// para poder restaurarla mediante [`Self::miller_forward`]
    ///
    /// La columna anterior pasa a ser la enfocada
    pub fn miller_back(&mut self) {
        if self.sources.len() < 2 {
            return;
        }

        let removed = Arc::clone(&self.sources.last().unwrap().cwd);

        self.miller_future.clear();
        self.miller_future
            .push(MillerSnapshot::Back { path: removed });

        self.sources.pop();
    }

    /// Avanza a la siguente ruta en la vista miller
    ///
    /// Restaura el estado almacenado en [`Self::miller_future`], ya sea
    /// reconstruyendo una columna eliminada o restaurando las fuentes guardadas
    /// antes de una navegación hacia el directorio padre
    pub fn miller_forward(&mut self, sender: Dispatcher) {
        match self.miller_future.pop() {
            Some(MillerSnapshot::Back { path }) => {
                self.sources.push(FileSource::new(path.clone()));
                let _ = self.request_load_from_path(path, sender);
            }

            Some(MillerSnapshot::Up { previous_sources }) => {
                self.sources.clear();
                for snapshot in previous_sources {
                    let mut source = FileSource::new(snapshot.cwd.clone());
                    source.id = snapshot.id;
                    source.files = snapshot.files;
                    source.sorted_indices = snapshot.sorted_indices;

                    self.sources.push(source);
                }

                for s in &self.sources {
                    tracing::debug!("  - {}", s.cwd.display());
                }
            }

            None => {}
        }
    }

    /// Navega al directorio padre de la ruta enfocada en la vista miller
    ///
    /// Guarda el estado actual de [`Self::sources`] en [`Self::miller_future`]
    /// para poder restaurarlo mediante [`Self::miller_forward`]
    ///
    /// Desplaza las columnas existentes una posición,
    /// reutiliza sus files y carga el directorio padre en la primera columna
    pub fn miller_up(&mut self, sender: Dispatcher) {
        let first_cwd = self.sources.first().map(|s| s.cwd.clone());

        if let Some(first) = first_cwd
            && let Some(parent) = first.parent()
        {
            let parent_arc: Arc<Path> = parent.into();

            let previous_sources: Vec<MillerSourceSnapshot> = self
                .sources
                .iter()
                .map(|s| MillerSourceSnapshot {
                    id: s.id.clone(),
                    cwd: s.cwd.clone(),
                    files: Arc::clone(&s.files),
                    sorted_indices: Arc::clone(&s.sorted_indices),
                })
                .collect();

            self.miller_future
                .push(MillerSnapshot::Up { previous_sources });

            for i in (1..self.sources.len()).rev() {
                let files = self.sources[i - 1].files.clone();
                self.sources[i].cwd = self.sources[i - 1].cwd.clone();
                self.sources[i].files = files;
            }

            self.sources[0].cwd = parent_arc.clone();
            self.sources[0].files = Arc::new(vec![].into());
            let _ = self.request_load_from_path(parent_arc, sender);
        }
    }

    /// Indica si existe una columna anterior a la que retroceder en la vista miller
    pub fn miller_can_go_back(&self) -> bool {
        self.sources.len() > 1
    }

    /// Indica si existe un estado de navegación que pueda restaurarse mediante
    /// [`Self::miller_forward`]
    pub fn miller_can_go_forward(&self) -> bool {
        !self.miller_future.is_empty()
    }

    /// Indica si la ruta enfocada tiene un directorio padre para navegar en
    /// la vista miller
    pub fn miller_can_go_up(&self) -> bool {
        match self.sources.first() {
            Some(source) => source.cwd.parent().is_some(),
            None => false,
        }
    }
}
