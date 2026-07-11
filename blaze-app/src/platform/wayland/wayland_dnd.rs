use tracing::{debug, error, info, warn};
use wayland_client::protocol::wl_data_device_manager::DndAction;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_client::{
    EventQueue,
    protocol::{
        wl_data_device::{self, WlDataDevice},
        wl_data_device_manager::WlDataDeviceManager,
        wl_data_offer::{self, WlDataOffer},
        wl_registry::{self, WlRegistry},
        wl_seat::WlSeat,
    },
};

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
};

use crate::platform::wayland::mime_handler::choose_best_mime;
use crate::platform::wayland::reader::{DroppedData, parse_payload, receive_raw_bytes};

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
}

impl DndState {
    fn new(sender: Sender<DndEvent>) -> Self {
        Self {
            current_offer: None,
            current_mime_types: Vec::new(),
            sender,
            seat: None,
            data_device_manager: None,
            data_device: None,
            accepted_mime: None,
        }
    }
}

//wlseat para atraparlo en wlregistry
impl Dispatch<WlSeat, ()> for DndState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        event: <WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let _ = event;
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
        _proxy: &WlDataOffer,
        event: <WlDataOffer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wl_data_offer::Event;
        match event {
            Event::Offer { mime_type } => {
                // mimes que el compositor tira
                state.current_mime_types.push(mime_type);
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
                    let rx = receive_raw_bytes(&offer, &mime);
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let backend = unsafe {
            wayland_client::backend::Backend::from_foreign_display(display_ptr.0 as *mut _)
        };

        let conn = Connection::from_backend(backend);
        let display = conn.display();
        let mut event_queue: EventQueue<DndState> = conn.new_event_queue();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());

        let mut state = DndState::new(sender);
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
            let _data_device = manager.get_data_device(seat, &qh, ());
            debug!("wl_data_device creado correctamente");
        } else {
            error!("No se ha encontrado wl_seat o wl_data_device_manager");
            return Err("No se ha encotrado wl_seat o wl_data_device_manager".into());
        }

        loop {
            if shutdown.load(Ordering::Relaxed) {
                break Ok(());
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

        let (sender, receiver) = std::sync::mpsc::channel();

        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let ptr = SendPtr(display_ptr);

        std::thread::spawn(move || {
            if let Err(e) = Self::run_dnd_loop(sender, shutdown_clone, ptr) {
                error!("Error en BLazeDND: {e}");
            }
        });

        Some(Self {
            events: receiver,
            shutdown,
        })
    }
}
