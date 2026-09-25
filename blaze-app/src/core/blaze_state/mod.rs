use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, warn};

use crate::{
    core::{
        blaze_state::state_structs::{
            CalcSet, GridView, MillerView, NewItemType, RequestSet, RubberBand, TagViewFilter,
            ViewMode, ViewState,
        },
        bootstrap::{
            configs::config_manager::with_configs,
            install_manager::installation_manager::with_installation_manager,
        },
        files::blaze_motor::motor::{BlazeMotor, BlazeMotorBuilder, MOTOR},
        runtime::{bus_structs::UiEvent, event_bus::with_event_bus},
        system::{
            clipboard::global_clipboard::GlobalClipboard,
            extended_info::extended_info_manager::ExtendedInfoManager,
            fileopener_module::{FileOpenerManager, GLOBAL_FILE_OPENER},
            sizer_manager::manager::SizerManager,
            terminal_opener::terminal_manager::{GLOBAL_TERMINAL_MANAGER, TerminalManager},
            updater::updater_manager::Updater,
            zip_manager::manager::ZipManager,
        },
    },
    ui::task_manager::tasks::TaskManager,
};

mod cache;
mod file_ops;
mod messages;
mod navigation;
mod search;
mod selections;
pub mod state_structs;
mod tab;
mod views;

pub struct BlazeCoreState {
    pub is_loading: bool,
    pub search_filter: String,
    pub clipboard: GlobalClipboard,
    pub active_tasks: usize,
    pub motor: Rc<RefCell<BlazeMotor>>,
    pub file_opener_manager: Arc<Mutex<FileOpenerManager>>,
    pub pending_scroll_to: Option<usize>,
    pub scroll_offset: f32,
    pub rubber_band: RubberBand,
    pub row_view: ViewState,
    pub grid_view: GridView,
    pub miller_view: MillerView,
    pub view_mode: ViewMode,
    pub renaming_file: Option<PathBuf>,
    pub rename_buffer: String,
    pub creating_new: Option<NewItemType>,
    pub new_item_buffer: String,
    pub focus_requested: bool,
    pub updater: Updater,
    pub last_fs_event: Option<Instant>,
    pub task_manager: &'static TaskManager,
    pub sizer_manager: SizerManager,
    pub needs_sort: bool,
    pub _cwd_input: String,
    pub terminal_manager: Arc<TokioMutex<TerminalManager>>,
    pub extended_info_manager: ExtendedInfoManager,
    pub extended_info: CalcSet<Arc<Path>>,
    pub zip_manager: ZipManager,
    last_navigation_time: Option<Instant>,
    navigation_cooldown: Duration,
    pub tag_filter: TagViewFilter,
    pub dir_sizes: CalcSet<Arc<Path>>,
    pub fonts: RequestSet<Box<str>>,
}

#[must_use = "llama .build() para crear el state"]
pub struct BlazeCoreBuilder {
    start_path: Option<Arc<Path>>,
}

impl BlazeCoreBuilder {
    fn new() -> Self {
        Self { start_path: None }
    }

    pub fn with_start_path(mut self, path: Option<Arc<Path>>) -> Self {
        self.start_path = path;
        self
    }

    #[must_use]
    pub async fn build(self) -> BlazeCoreState {
        let motor = Rc::new(RefCell::new(
            BlazeMotorBuilder::default()
                .with_start_path(self.start_path)
                .build()
                .await,
        ));
        let active_id = motor.borrow().active_tab().id;

        //Asignar el id de la ventana inicial al active_id
        crate::core::runtime::event_bus::set_active_tab(active_id);

        MOTOR.with(|m| {
            *m.borrow_mut() = Some(motor.clone());
        });

        let rubber_band = RubberBand {
            rubber_band_start: None,
            rubber_band_current: None,
            is_rubber_banding: false,
            rubber_band_start_content_y: 0.0,
        };

        // Traer los tamaños desde las configs
        let (row_icon_size, grid_icon_size, view_mode) = with_configs(|c| {
            (
                c.get_row_icon_size(),
                c.get_grid_icon_size(),
                c.get_view_mode(),
            )
        });

        let row_view = ViewState {
            icon_size: row_icon_size,
            ..Default::default()
        };

        let mut grid_base = row_view.clone();
        grid_base.icon_size = grid_icon_size;

        let grid_view = GridView::default(grid_base);

        let miller_view = MillerView::default(row_view.clone());

        let file_opener_manager = GLOBAL_FILE_OPENER.clone();

        let task_manager = TaskManager::global();

        let sizer_manager = SizerManager::new();

        let terminal_manager = GLOBAL_TERMINAL_MANAGER.clone();

        let mut state = BlazeCoreState {
            motor,
            is_loading: false,
            search_filter: String::new(),
            clipboard: GlobalClipboard::new(),
            active_tasks: 0,
            pending_scroll_to: None,
            scroll_offset: 0.0,
            rubber_band,
            row_view,
            grid_view,
            miller_view,
            view_mode,
            renaming_file: None,
            rename_buffer: String::new(),
            creating_new: None,
            new_item_buffer: String::new(),
            focus_requested: false,
            updater: Updater::init(),
            file_opener_manager,
            last_fs_event: None,
            task_manager,
            sizer_manager,
            needs_sort: false,
            _cwd_input: String::new(),
            terminal_manager,
            extended_info_manager: ExtendedInfoManager::new(),
            zip_manager: ZipManager::new(),
            last_navigation_time: None,
            navigation_cooldown: Duration::from_millis(100),
            tag_filter: TagViewFilter::All { all_items_len: 0 },
            extended_info: CalcSet::default(),
            dir_sizes: CalcSet::default(),
            fonts: RequestSet::default(),
        };

        let dispatcher = with_event_bus(|e| e.dispatcher(active_id));

        {
            let mut motor = state.motor.borrow_mut();
            let tab = motor.active_tab_mut();
            let path = tab.focused.clone();

            if let Err(e) = tab.request_load_from_path(path, dispatcher.clone()) {
                warn!("Ha ocurrido un error al cargar los archivos: {}", e);
            }
            state.updater.check_for_update(dispatcher.clone());
        };

        let is_installed = with_installation_manager(|im| !im.is_installed());

        with_configs(|c| {
            let day_elapsed = match c.get_last_time_asked_install() {
                None => true,
                Some(time) => time.elapsed().unwrap_or_default() >= Duration::from_hours(24),
            };

            if is_installed && (c.get_should_ask_install() || day_elapsed) {
                dispatcher.send(UiEvent::ShowWantToInstall).ok();
            } else {
                debug!(
                    "Instalado {} {} {}",
                    !is_installed,
                    c.get_should_ask_install(),
                    day_elapsed
                )
            }
        });

        state
    }
}

impl Default for BlazeCoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}
