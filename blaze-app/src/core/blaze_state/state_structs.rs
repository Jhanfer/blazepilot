use egui::{Pos2, pos2};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::Path,
    sync::Arc,
};

#[derive(Clone, PartialEq)]
pub enum NewItemType {
    Folder,
    File,
}

/// Define los modos disponibles para mostrar los elementos del explorador
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug, Default)]
pub enum LayoutMode {
    /// Vista en filas con la información básica.
    #[default]
    Row,

    /// Vista en filas con información adicional.
    RowDetailed,

    /// Vista en fila compacta.
    Compact,

    /// Vista organizada en cuadrícula.
    Grid,

    /// Vista por columnas siguiendo el modelo Miller.
    Miller,
}

/// Define el contexto de visualización del explorador y el layout utilizado
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum ViewMode {
    /// Vista normal del directorio de archivos (ej: Row, Grid, Etc.).
    Normal(LayoutMode),

    // Vista de las etiquetas y archivos etiquetados.
    Tags(LayoutMode),
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Normal(LayoutMode::Row)
    }
}

/// Define el filtro de las etiquetas para mostrar el
/// número de elementos que hay en una etiqueta seleccionada
pub enum TagViewFilter {
    /// Muestra todos los elementos de todas las etiquetas.
    All { all_items_len: usize },

    /// Muestra el nombre de la etiqueta y el número de elementos de esa etiqueta.
    Tag { name: String, items_len: usize },
}

/// Mantiene el estado de la selección mediante área de arrastre
pub struct RubberBand {
    pub rubber_band_start: Option<Pos2>,
    pub rubber_band_current: Option<Pos2>,
    pub is_rubber_banding: bool,
    pub rubber_band_start_content_y: f32,
}

/// Contiene el estado común de los distintos modos de vistas
///
/// Incluye la vista en row (compact, detailed), grid y miller
#[derive(Clone)]
pub struct ViewState {
    pub is_dragging_files: bool,
    pub drag_ghost_pos: Option<Pos2>,
    pub drop_target: Option<Arc<Path>>,
    pub drop_invalid_target: Option<Arc<Path>>,
    pub scroll_area_origin_y: f32,
    pub first_visible: usize,
    pub last_visible: usize,
    pub viewport_height: f32,
    pub icon_size: f32,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            is_dragging_files: false,
            drag_ghost_pos: None,
            drop_target: None,
            drop_invalid_target: None,
            scroll_area_origin_y: 0.0,
            first_visible: 0,
            last_visible: 0,
            viewport_height: 0.0,
            icon_size: 14.0,
        }
    }
}

pub struct GridView {
    pub base: ViewState,
    pub cols: usize,
    pub cell_size: f32,
    pub actual_origin: Pos2,
    pub row_height: f32,
}

impl GridView {
    pub fn default(base_view: ViewState) -> Self {
        Self {
            base: base_view,
            cols: 0,
            cell_size: 0.0,
            actual_origin: pos2(0.0, 0.0),
            row_height: 0.0,
        }
    }
}

/// gestiona los estados de las columnas miller
///
/// las columnas deben modificarse mediante [`Self::push_column`] y
/// [`Self::truncate_to`] para mantener sincronizado [`Self::opened`]
pub struct MillerView {
    pub base: ViewState,
    pub columns: Vec<Arc<Path>>,
    pub opened: HashMap<usize, usize>,
    pub scroll_offsets: HashMap<usize, f32>,
    pub visible_ranges: HashMap<usize, (usize, usize)>,
    pub search_filter: HashMap<usize, String>,
    pub row_height: f32,
}

impl MillerView {
    pub fn default(base_view: ViewState) -> Self {
        Self {
            base: base_view,
            columns: Vec::new(),
            opened: HashMap::new(),
            scroll_offsets: HashMap::new(),
            visible_ranges: HashMap::new(),
            search_filter: HashMap::new(),
            row_height: 28.0,
        }
    }
}

impl MillerView {
    /// Añade un path a las columnas
    pub fn push_column(&mut self, path: Arc<Path>) {
        self.columns.push(path);
    }

    /// Acorta las columnas, los directorios abiertos y el offset del scroll
    pub fn truncate_to(&mut self, depth: usize) {
        self.columns.truncate(depth);
        self.opened.retain(|&col, _| col < depth);
        self.scroll_offsets.retain(|&col, _| col < depth);
        self.search_filter.retain(|&col, _| col < depth);
    }

    /// Establece que item está abierto en una columna específica
    pub fn set_opened(&mut self, col: usize, row: usize) {
        self.opened.insert(col, row);
    }

    /// Cierra cualquier selección abierta en una columna y
    /// limpia el filtro de búsqueda
    pub fn clear_opened(&mut self, col: usize) {
        self.opened.remove(&col);
        self.search_filter.remove(&col);
    }

    /// Verifica si un item específico está abierto en su columna
    pub fn is_opened(&self, col: usize, row: usize) -> bool {
        self.opened.get(&col) == Some(&row)
    }
}

/// Rastea elementos que están siendo calculados
/// y los que han terminado de calcularse
pub struct CalcSet<K> {
    calculating: HashSet<K>,
    calculated: HashSet<K>,
}

impl<K> CalcSet<K> {
    pub fn new() -> Self {
        Self {
            calculating: HashSet::new(),
            calculated: HashSet::new(),
        }
    }
}

impl<K> Default for CalcSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash> CalcSet<K> {
    /// Marca un elemento como pendiente a calcularse
    pub fn start(&mut self, k: K) {
        self.calculating.insert(k);
    }

    pub fn is_calculating(&self, k: &K) -> bool {
        self.calculating.contains(k)
    }

    /// Marca un elemento como terminado de calcularse
    pub fn finish(&mut self, k: K) {
        self.calculating.remove(&k);
        self.calculated.insert(k);
    }

    pub fn is_calculated(&self, k: &K) -> bool {
        self.calculated.contains(k)
    }

    pub fn is_done(&self, k: &K) -> bool {
        self.calculated.contains(k)
    }

    /// Elimina el elemento de cualquier estado de cálculo
    pub fn remove_one(&mut self, k: &K) {
        self.calculating.remove(k);
        self.calculated.remove(k);
    }

    pub fn clear(&mut self) {
        self.calculating.clear();
        self.calculated.clear();
    }

    pub fn shrink(&mut self) {
        self.calculating.shrink_to_fit();
        self.calculated.shrink_to_fit();
    }

    /// Elimina todos los elementos y libera la capacidad no utilizada
    pub fn reset(&mut self) {
        self.clear();
        self.shrink();
    }
}

/// Rastea las fuentes solicitadas
pub struct RequestSet<K> {
    requested: HashSet<K>,
}

impl<K> RequestSet<K> {
    pub fn new() -> Self {
        Self {
            requested: HashSet::new(),
        }
    }
}

impl<K> Default for RequestSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash> RequestSet<K> {
    /// Marca una fuente como solicitada
    pub fn start(&mut self, k: K) {
        self.requested.insert(k);
    }

    pub fn is_requested(&self, k: &K) -> bool {
        self.requested.contains(k)
    }

    pub fn clear(&mut self) {
        self.requested.clear();
    }

    pub fn shrink(&mut self) {
        self.requested.shrink_to_fit();
    }

    /// Limpia las fuentes y libera la capacidad solicitada
    pub fn reset(&mut self) {
        self.clear();
        self.shrink();
    }
}
