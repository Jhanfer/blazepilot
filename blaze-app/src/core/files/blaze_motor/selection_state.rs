use crate::core::files::blaze_motor::motor_structs::FileEntry;
use bitvec::vec::BitVec;
use std::{path::Path, sync::Arc};

/// Gestiona la selección de elementos de una lista de [`FileEntry`]
///
/// Mantiene las selecciones mediante un [`BitVec`]
pub struct SelectionManager {
    last_selected_index: Option<usize>,
    select_all_mode: bool,
    selection_anchor: Option<usize>,
    selection: BitVec,
}

impl SelectionManager {
    /// Crea el gestor de selecciones
    pub fn new() -> Self {
        Self {
            last_selected_index: None,
            select_all_mode: false,
            selection_anchor: None,
            selection: BitVec::new(),
        }
    }

    /// Devuelve el indice de la última selección
    pub fn last_selected_index(&self) -> Option<usize> {
        self.last_selected_index
    }

    /// Establece el indice de la última selección
    pub fn set_last_selected_index(&mut self, index: Option<usize>) {
        self.last_selected_index = index;
    }

    /// Indica si está activo el modo de selección total
    pub fn select_all_mode(&self) -> bool {
        self.select_all_mode
    }

    ///Devuelve el indice utilizado como ancla para la selección por rango
    pub fn selection_anchor(&self) -> Option<usize> {
        self.selection_anchor
    }

    /// Establece el ancla para la selección por rango
    pub fn set_selection_anchor(&mut self, anchor: Option<usize>) {
        self.selection_anchor = anchor;
    }

    /// Establece el estado de selección de un elemento
    ///
    /// No modifica la selección si `index` está fuera del tamaño actual
    pub fn set_selection(&mut self, index: usize, value: bool) {
        if index < self.selection.len() {
            self.selection.set(index, value);
        }
    }

    /// Ajusta el tamaño de la selección al número actual de elementos
    pub fn resize_selection(&mut self, new_len: usize) {
        if self.selection.len() != new_len {
            self.selection.resize(new_len, false);
        }
    }

    /// Indica si el elemento en `index` está seleccionado
    ///
    /// En el modo de selección total, los bits se invierten,
    /// representando así que el elemento está excluído de la selección
    pub fn is_selected(&self, index: usize) -> bool {
        if index >= self.selection.len() {
            return false;
        }

        if self.select_all_mode {
            !self.selection[index]
        } else {
            self.selection[index]
        }
    }

    /// Selecciona todos los elementos de la lista
    ///
    /// Activa el modo de selección invertida, en el que los elementos marcados
    /// en [`Self::selection`] representan excepciones a la selección global
    pub fn select_all(&mut self, files_len: usize) {
        self.select_all_mode = true;
        self.selection.clear();
        self.resize_selection(files_len);
        self.last_selected_index = if files_len > 0 {
            Some(files_len - 1)
        } else {
            None
        };
    }

    /// Limpia todas las selecciones y desactiva el modo de selección total
    pub fn deselect_all(&mut self) {
        self.select_all_mode = false;
        self.selection.clear();
        self.last_selected_index = None;
        self.selection_anchor = None;
    }

    /// Alterna entre la selección total y la no selección
    ///
    /// Si el modo de selección total está activo y no existen excepciones dentro
    /// de [`Self::selection`], deselecciona todos los elementos. En caso contrario,
    /// activa la selección total y elimina las excepciones existentes
    pub fn toggle_select_all(&mut self, files_len: usize) {
        if self.select_all_mode && self.selection.not_any() {
            self.deselect_all();
        } else {
            self.select_all(files_len);
        }
    }

    /// Selecciona todos los elementos que están entre los índices indicados
    /// incluyendo ambos extremos
    pub fn select_range(&mut self, start: usize, end: usize) {
        let start = start.min(end);
        let end = start.max(end);

        if start >= self.selection.len() {
            return;
        }
        let end = end.min(self.selection.len() - 1);

        if self.select_all_mode {
            for i in start..=end {
                self.selection.set(i, false);
            }
        } else {
            for i in start..=end {
                self.selection.set(i, true);
            }
        }
    }

    /// Devuelve el número de elementos seleccionados
    ///
    /// `files_len` corresponde al número total de entradas disponibles
    pub fn selected_count(&self, files_len: usize) -> usize {
        if self.select_all_mode {
            files_len - self.selection.count_ones()
        } else {
            self.selection.count_ones()
        }
    }

    /// Devuelve las rutas y nombres de los elementos seleccionados
    pub fn get_selected_paths(&self, files: &[Arc<FileEntry>]) -> Vec<(Box<str>, Arc<Path>)> {
        files
            .iter()
            .enumerate()
            .filter(|(i, _)| self.is_selected(*i))
            .map(|(_, f)| (f.name.clone(), f.full_path.clone()))
            .collect()
    }

    /// Devuelve los elementos seleccionados de la lista
    pub fn selected_as_entries(&self, files: &[Arc<FileEntry>]) -> Vec<Arc<FileEntry>> {
        if files.is_empty() {
            return vec![];
        }

        let mut result = Vec::with_capacity(files.len() / 2);

        if self.select_all_mode {
            for (i, file) in files.iter().enumerate() {
                if !self.selection.get(i).map(|b| *b).unwrap_or(false) {
                    result.push(file.clone());
                }
            }
        } else {
            for (i, file) in files.iter().enumerate() {
                if self.selection.get(i).map(|b| *b).unwrap_or(false) {
                    result.push(file.clone());
                }
            }
        }
        result
    }
}
