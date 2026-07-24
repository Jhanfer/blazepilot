use parking_lot::Mutex;
use tracing::{debug, error, info, warn};
use wayland_client::protocol::wl_data_device_manager::DndAction;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_client::{
    EventQueue,
    protocol::wl_keyboard::WlKeyboard,
    protocol::{
        wl_data_device::{self, WlDataDevice},
        wl_data_device_manager::WlDataDeviceManager,
        wl_data_offer::{self, WlDataOffer},
        wl_data_source::WlDataSource,
        wl_registry::{self, WlRegistry},
        wl_seat::WlSeat,
    },
};

use std::os::fd::FromRawFd;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crossbeam_channel::{Receiver, Sender};

use crate::platform::wayland::mime_handler::choose_best_mime;
use crate::platform::wayland::reader::{
    DroppedData, decode_text, parse_payload, receive_raw_bytes,
};

pub enum DndEvent {
    #[allow(unused)]
    Hovered(Vec<PathBuf>),
    Leaving,
    Dropped(DroppedData),
}

struct DndState {
    // Offer actual
    current_offer: Option<WlDataOffer>,
    // mimes que da el offer
    current_mime_types: Vec<String>,
    // canal para la ui
    sender: Sender<DndEvent>,
    // seat para el data device
    seat: Option<WlSeat>,
    // manager para crear datos en el data device
    data_device_manager: Option<WlDataDeviceManager>,
    // el data device
    #[allow(unused)]
    data_device: Option<WlDataDevice>,
    accepted_mime: Option<String>,

    // oferta del clipboard de wayland
    clipboard_offer: Option<WlDataOffer>,

    clipboard_mime_types: Vec<String>,
    clipboard_text_to_send: Option<String>,
    data_source: Option<WlDataSource>,
    last_serial: u32,
    clipboard_text: Arc<Mutex<Option<String>>>,
}

impl DndState {
    fn new(sender: Sender<DndEvent>, clipboard_text: Arc<Mutex<Option<String>>>) -> Self {
        Self {
            current_offer: None,
            current_mime_types: Vec::new(),
            sender,
            seat: None,
            data_device_manager: None,
            data_device: None,
            accepted_mime: None,
            clipboard_offer: None,
            clipboard_mime_types: Vec::new(),
            clipboard_text_to_send: None,
            data_source: None,
            last_serial: 0,
            clipboard_text,
        }
    }
}

//wldatasource para poder exponer datos al compositor
impl Dispatch<WlDataSource, ()> for DndState {
    fn event(
        state: &mut Self,
        _proxy: &WlDataSource,
        event: wayland_client::protocol::wl_data_source::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_data_source::Event;
        match event {
            Event::Send { mime_type: _, fd } => {
                if let Some(text) = state.clipboard_text_to_send.as_ref() {
                    use std::io::Write;
                    use std::os::fd::IntoRawFd;

                    let raw_fd = fd.into_raw_fd();
                    let mut file = unsafe { std::fs::File::from_raw_fd(raw_fd) };

                    if let Err(e) = file.write_all(text.as_bytes()) {
                        warn!("Error escribiendo: {e}");
                    }
                    if let Err(e) = file.flush() {
                        warn!("Error flush: {e}");
                    }
                }
            }
            Event::Cancelled => {
                state.clipboard_text_to_send = None;
                if let Some(source) = state.data_source.take() {
                    source.destroy();
                }
            }
            _ => {}
        }
    }
}

//wlseat para atraparlo en wlregistry
impl Dispatch<WlSeat, ()> for DndState {
    fn event(
        _state: &mut Self,
        proxy: &WlSeat,
        event: <WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_seat::Event;
        if let Event::Capabilities { capabilities } = event
            && let wayland_client::WEnum::Value(
                wayland_client::protocol::wl_seat::Capability::Keyboard,
            ) = capabilities
        {
            let _ = proxy.get_keyboard(qh, ());
        }
    }
}

impl Dispatch<WlKeyboard, ()> for DndState {
    fn event(
        state: &mut Self,
        _proxy: &WlKeyboard,
        event: wayland_client::protocol::wl_keyboard::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_keyboard::Event;
        match event {
            Event::Key { serial, .. } | Event::Modifiers { serial, .. } => {
                state.last_serial = serial;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlDataDeviceManager, ()> for DndState {
    fn event(
        _state: &mut Self,
        _proxy: &WlDataDeviceManager,
        event: <WlDataDeviceManager as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let _ = event;
    }
}

//guarda el wlseat y el wldatadevice cuando aparezcan
impl Dispatch<WlRegistry, ()> for DndState {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wl_registry::Event;
        match event {
            Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == WlSeat::interface().name {
                    let seat = proxy.bind::<WlSeat, _, _>(name, version, qh, ());
                    state.seat = Some(seat);
                } else if interface == WlDataDeviceManager::interface().name {
                    let manager = proxy.bind::<WlDataDeviceManager, _, _>(name, version, qh, ());
                    state.data_device_manager = Some(manager);
                }
            }
            //no manejo desconexiones todavía
            Event::GlobalRemove { .. } => {}
            _ => {}
        }
    }
}

//guardado de los mimes para poder procesarlos abajo
impl Dispatch<WlDataOffer, ()> for DndState {
    fn event(
        state: &mut Self,
        proxy: &WlDataOffer,
        event: <WlDataOffer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wl_data_offer::Event;
        match event {
            // mimes que el compositor tira
            Event::Offer { mime_type } => {
                if state
                    .current_offer
                    .as_ref()
                    .map(|o| o == proxy)
                    .unwrap_or(false)
                {
                    state.current_mime_types.push(mime_type);
                } else if state
                    .clipboard_offer
                    .as_ref()
                    .map(|o| o == proxy)
                    .unwrap_or(false)
                {
                    state.clipboard_mime_types.push(mime_type);
                }
            }
            Event::Action { dnd_action } => {
                debug!("Acción de DnD acordada por el compositor: {:?}", dnd_action);
            }
            _ => {}
        }
    }
}

//eventos principales donde obtengo los datos con los que deseo trabajar
impl Dispatch<WlDataDevice, ()> for DndState {
    fn event(
        state: &mut Self,
        _proxy: &WlDataDevice,
        event: <WlDataDevice as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wl_data_device::Event;
        match event {
            Event::DataOffer { id } => {
                //guardamos el offer y limpiamos los mimes antiguos
                state.current_mime_types.clear();
                state.current_offer = Some(id);
            }
            Event::Enter {
                surface: _,
                x: _,
                y: _,
                id: Some(offer),
                serial,
                ..
            } => {
                //cursor entra a la ventana con datos (usamos el mismo offer que sacamos de DataOffer)

                //seleccionamos como queremos trabjar los mimes
                if let Some(best_mime) = choose_best_mime(&state.current_mime_types) {
                    info!("Aceptando MIME: {}", best_mime);
                    // se acepta para avisar al compositor que queremos el mime
                    offer.accept(serial, Some(best_mime.clone()));
                    offer.set_actions(DndAction::Copy, DndAction::Copy);
                    state.accepted_mime = Some(best_mime);
                } else {
                    //fallback por si no hay mimes con los que deseamos trabajar
                    offer.accept(serial, None);
                }
            }
            Event::Leave => {
                //limpieza
                state.current_offer = None;
                state.current_mime_types.clear();
                let _ = state.sender.send(DndEvent::Leaving);
            }
            Event::Drop => {
                // aquí es donde leemos el pipe
                if let Some(offer) = state.current_offer.take()
                    && let Some(mime) = state.accepted_mime.take()
                {
                    //se leen los datos del pipe dependiendo del mime
                    match receive_raw_bytes(&offer, &mime) {
                        Ok(rx) => {
                            let sender = state.sender.clone();

                            //siempre es recomendable usar spawn para no saturar al hilo principal mientras hacemos recv de los datos que envia "receive_raw_bytes"
                            std::thread::spawn(move || {
                                if let Ok(raw) = rx.recv() {
                                    //parseo de datos: path si es uri y devuelve bytes si es media o texto plano y los envía a ui por el sender¡
                                    let data = parse_payload(&mime, raw);
                                    sender.send(DndEvent::Dropped(data)).ok();
                                }

                                //finalizar y destruir el offer independiente de si hay o no datos, asi se evitan comportamientos raros o congelamiento
                                offer.finish();
                                offer.destroy();
                            });
                        }
                        Err(s) => {
                            warn!("{}", s);
                        }
                    }
                }
            }

            Event::Selection { id } => {
                state.clipboard_offer = id;
                state.clipboard_mime_types = state.current_mime_types.clone();

                if let Some(offer) = &state.clipboard_offer
                    && let Some(mime) =
                        choose_best_mime(&state.clipboard_mime_types).filter(|m| m.contains("text"))
                {
                    match receive_raw_bytes(offer, &mime) {
                        Ok(rx) => {
                            let clipboard_text = Arc::clone(&state.clipboard_text);

                            std::thread::spawn(move || {
                                if let Ok(raw) = rx.recv() {
                                    let text = decode_text(raw);
                                    *clipboard_text.lock() = Some(text);
                                }
                            });
                        }
                        Err(s) => {
                            warn!("{}", s);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    //instacia wldataoffer vacío usando DndState como estado
    wayland_client::event_created_child!(
        DndState,
        WlDataDevice, [
            0 => (WlDataOffer, ())
        ]
    );
}

//receiver para ui
pub struct WaylandDndReceiver {
    pub events: Receiver<DndEvent>,
    shutdown: Arc<AtomicBool>,
    pub copy_tx: Sender<String>,
    pub clipboard_text: Arc<Mutex<Option<String>>>,
}

impl Drop for WaylandDndReceiver {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

struct SendPtr(*mut std::ffi::c_void);
unsafe impl Send for SendPtr {}

impl WaylandDndReceiver {
    fn run_dnd_loop(
        sender: Sender<DndEvent>,
        shutdown: Arc<AtomicBool>,
        display_ptr: SendPtr,
        copy_rx: Receiver<String>,
        clipboard_text: Arc<Mutex<Option<String>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let backend = unsafe {
            wayland_client::backend::Backend::from_foreign_display(display_ptr.0 as *mut _)
        };

        let conn = Connection::from_backend(backend);
        let display = conn.display();
        let mut event_queue: EventQueue<DndState> = conn.new_event_queue();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());

        let mut state = DndState::new(sender, clipboard_text);
        let mut trys = 0;

        loop {
            debug!("Intento: {trys}");
            event_queue.roundtrip(&mut state)?;

            if state.seat.is_some() && state.data_device_manager.is_some() {
                break;
            }

            trys += 1;
            if trys >= 10 {
                error!(
                    "Error tras {trys} intentos: roundtrip: seat={}, manager={}",
                    state.seat.is_some(),
                    state.data_device_manager.is_some(),
                );

                return Err(
                    "No se han encontrado wl_seat/wl_data_device_manager tras varios intentos"
                        .into(),
                );
            }
        }

        if let (Some(seat), Some(manager)) = (&state.seat, &state.data_device_manager) {
            state.data_device = Some(manager.get_data_device(seat, &qh, ()));
            debug!("wl_data_device creado correctamente");
        } else {
            error!("No se ha encontrado wl_seat o wl_data_device_manager");
            return Err("No se ha encotrado wl_seat o wl_data_device_manager".into());
        }

        loop {
            if shutdown.load(Ordering::Relaxed) {
                break Ok(());
            }

            while let Ok(text) = copy_rx.try_recv() {
                if let (Some(manager), Some(data_device)) =
                    (&state.data_device_manager, &state.data_device)
                {
                    debug!("Se recibe el text: {text}");

                    if let Some(old) = state.data_source.take() {
                        old.destroy();
                    }

                    state.clipboard_text_to_send = None;

                    let source = manager.create_data_source(&qh, ());
                    source.offer("text/plain;charset=utf-8".into());
                    source.offer("text/plain".into());
                    data_device.set_selection(Some(&source), state.last_serial);
                    state.clipboard_text_to_send = Some(text);
                    state.data_source = Some(source);
                }
            }

            event_queue.dispatch_pending(&mut state)?;

            if let Err(e) = conn.flush() {
                warn!("El flush ha fallado: {e}");
            }

            match event_queue.prepare_read() {
                Some(guard) => match guard.read() {
                    Ok(_) => {}
                    Err(e) => warn!("Error al leer eventos Wayland (no fatal, se reintenta): {e}"),
                },
                None => info!("prepare_read devolvió None, otro hilo tiene el guard"),
            }

            std::thread::yield_now();
        }
    }

    pub fn spawn(display_ptr: *mut std::ffi::c_void) -> Option<Self> {
        //si no es wayland, esto no se va a disparar
        std::env::var_os("WAYLAND_DISPLAY")?;

        let (sender, receiver) = crossbeam_channel::bounded(4);
        let (copy_tx, copy_rx) = crossbeam_channel::bounded::<String>(4);
        let clipboard_text = Arc::new(Mutex::new(None::<String>));
        let clipboard_text_clone = Arc::clone(&clipboard_text);

        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let ptr = SendPtr(display_ptr);

        std::thread::spawn(move || {
            if let Err(e) =
                Self::run_dnd_loop(sender, shutdown_clone, ptr, copy_rx, clipboard_text_clone)
            {
                error!("Error en BLazeDND: {e}");
            }
        });

        Some(Self {
            events: receiver,
            shutdown,
            copy_tx,
            clipboard_text,
        })
    }
}
