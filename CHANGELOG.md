## [0.12.0] - 2026-06-06

### 🚀 Features
- **(i18n)** Implementa internacionalización con 6 idiomas ([`2546bf8`](https://github.com/Jhanfer/blazepilot/commit/2546bf87161103edcc6a7f486a5813393269046a))
- **(main)** Añade sistema de reintentos automático con backends X11/Wayland ([`0e1cd0d`](https://github.com/Jhanfer/blazepilot/commit/0e1cd0d4ce2b2644b7cc4863a5e396fe10317c48))

### 🐛 Bug fixes
- **(ui)** Reintentos y timeout en carga de terminales en configs_dialog ([`075a6af`](https://github.com/Jhanfer/blazepilot/commit/075a6af2c4aba83430b0f626143f9f81dc8f34e5))
- Prevenir inserción accidental de 'rec:' y simplificar pre-commit ([`b9e2a6b`](https://github.com/Jhanfer/blazepilot/commit/b9e2a6b9022a0be6f6ff4d3bbc595e4045328278))

### ⚡ Performance
- **(ui)** Optimiza renderizado de iconos y mejora visual ([`cf7a91c`](https://github.com/Jhanfer/blazepilot/commit/cf7a91cda03e066254426f88de8ecf1c28320b9b))
- **(watcher+git)** Debounce 500ms, caché Git con propagación y rutas absolutas ([`c47c23a`](https://github.com/Jhanfer/blazepilot/commit/c47c23adbcbac3044e163498f56195dfc541b752))
- **(quick-access)** Optimiza watcher y evita cálculos redundantes ([`7824e55`](https://github.com/Jhanfer/blazepilot/commit/7824e55129b4e4b4c6341fa67de6b4bd542be3af))

### ⚙️ Miscelánea
- **(repo)** Añade pre-commit, CI y estandarización masiva de código ([`322b3f4`](https://github.com/Jhanfer/blazepilot/commit/322b3f4df852c5a602878d8b0aebb42fc345d817))

## [0.11.0] - 2026-05-29
 
### 🚀 Features
- **(tags)** Sistema de Tags/Quick Access — reemplaza favoritos hardcoded; `ViewMode {Normal, Tags}`, `TagViewFilter`, `QuickTagEvent` en `bus_structs` ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(tags)** Toggle en toolbar (Ctrl+T), crear tag (Ctrl+Shift+T), isla inferior `render_tags_island_bubble` ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(ui)** `pending_scroll_to` — scroll automático al seleccionar resultado de búsqueda o navegación ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
### 🐛 Bug fixes
- **(undo)** Corrige el deshacer de mover y notifica conflictos en pegar ([`c752547`](https://github.com/Jhanfer/blazepilot/commit/c752547578b1561f219993ee678448067b209ed6))
- **(rename)** Renombrado ya no convierte a minúsculas — se mantiene el casing original al iniciar edición desde el menú contextual ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
### ⚡ Performance
- **(sizer)** Cancelación de cálculos de tamaño con `AbortHandle` y `AtomicBool` — `navigate_to/up/back/forward` cancelan tareas pendientes automáticamente ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(sizer)** Migra de Mutex/Semaphore complejos a `spawn_blocking` con chequeo atómico; timeout 300s, nuevo `CancelledError` ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(wgpu)** `PresentMode` Immediate → Fifo para mejor VSync y reducir tearing ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
### 🚜 Refactor
- **(config)** Elimina `config_state.rs` (393 líneas) y lógica de favoritos — nuevo módulo `quick_access_manager` ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(ui)** Paleta centralizada — `COLOR_BG_MAIN`, `COLOR_BG_PANEL`, `COLOR_ACCENT_GLOW`, `COLOR_TEXT_PRIMARY`; bordes redondeados 20px ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(row_view)** `row_view_callback.rs`: 245 → 92 líneas, lógica extraída a `background_response_logic` ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(utilities)** `resolve_icon`, `git_dot_color`, `text_color_for_git` centralizados en `utilities.rs` ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
### ⚙️ Miscelánea
- **(i18n)** Carpetas del sidebar en español — Escritorio, Descargas, Imágenes, etc. ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))
- **(icons)** `ICON_TAG` añadido para la nueva funcionalidad de tags ([`9f53025`](https://github.com/Jhanfer/blazepilot/commit/9f53025dd11928471dd39847e43ae129f5e0937e))

## [0.10.0] - 2026-05-15

### 🚀 Features
- **(core)** Añade sistema de Deshacer (Undo) y refactoriza clipboard([`3d56cce`](https://github.com/Jhanfer/blazepilot/commit/3d56cce750bc73f2d200de012be5bc4871a3ee56))

### 🐛 Bug fixes
- **(ui)** Se evita selección automática tras doble click en carpeta([`8d602fa`](https://github.com/Jhanfer/blazepilot/commit/8d602fabebbadfb21c4e5e938d4180a641b58cc6))- **(recursive)** Se aplica filtro show_hidden y migra a Arc<FileEntry>([`1ce13ef`](https://github.com/Jhanfer/blazepilot/commit/1ce13efcc0f5b55ea7ba77d4bf2a4d43a67f6ae8))

### 🚜 Refactor
- **(clipboard)** Errores tipados, Arc<Path> y manejo seguro de locks([`3071b66`](https://github.com/Jhanfer/blazepilot/commit/3071b6647467d8bf2fb86ebe481a7333fb4a5811))

## [0.9.0] - 2026-05-22

### 🚀 Features
- **Nuevo** sistema de papelera (`trash_backend.rs`) con trait `TrashBackend`
- **KnownDirsManager**: centraliza directorios estándar (Home, Desktop, Documents, etc.)
- **MIME detection** real usando `xdg-mime` + firma mágica
- **analyze_file()** mejorado para detectar ELF, AppImage, imágenes, PDF, ZIP y shebangs
- **Sidebar** con barra de progreso en drives y tooltip
- **Hints** flotantes inferiores mostrando atajos de teclado

### 🚜 Refactor
- **Refactor completo** de clipboard usando backend
- **Opener** refactorizado con `OpenerResult<T>` y `OpenStrategy`
- **Iconos** con semáforo de concurrencia y rasterizado SVG
- **Detección de discos** mejorada y más robusta
- Limpieza general de unwraps y fallbacks seguros

### 🐛 Bug fixes
- **Fix** en reutilización de IDs de pestañas
- **Fix** en detección de estado de selección
- **Fix** en activación de hotkeys
- **Fix** en navegación de pestañas (Ctrl+← / Ctrl+→)

### ⚙️ Changes
- Eliminadas dependencias `dirs` y `trash`
- Añadida dependencia `urlencoding`
- Mejoras en inicialización de directorios críticos

## [0.8.0] - 2026-05-01

### 🚀 Features
- **(ui)** Añadido ThumbnailManager con caché en disco y generación async([`8b4170d`](https://github.com/Jhanfer/blazepilot/commit/8b4170d7256cc18fe746bec086973a11fcfea218))

### 🐛 Bug fixes
- Evitar crash al redimensionar columnas. Evita un Rect inválido añadiendo .max() a date_w y size_w y eliminando declaración duplicada de variables.([`8aeb6f1`](https://github.com/Jhanfer/blazepilot/commit/8aeb6f16bd02cda6231f0440f332d1a4841166cf))- Typo en nombre de función load_or_init_cofigs -> configs([`d460981`](https://github.com/Jhanfer/blazepilot/commit/d46098192344b287ec403bb9146837256ab0f01b))- Typo en método 'foward' -> 'forward' en 'TabState'([`05ea588`](https://github.com/Jhanfer/blazepilot/commit/05ea5884f1328a9c2f502f95ca27d26ed5926ee4))- **(cache)** Se arregla el guardado de caché de colores. Se elimina 'update_color_cache()' para ser reemplazado por 'save_color_cache()' en el diálogo de selección de colores.([`59dea4b`](https://github.com/Jhanfer/blazepilot/commit/59dea4b1e60b79746705f3024afab51d6789e4ea))

### ⚡ Performance
- Aumenta caché LRU de directorios de 2 a 50 entradas([`d47a858`](https://github.com/Jhanfer/blazepilot/commit/d47a85863723f7bb81466743aa6f7004d67372a1))

### 🚜 Refactor
- -CacheManager async y guardado debounced: cambiado RwLock a tokio::sync::RwLock, eliminando unwraps en acceso a caché y añadiendo save_caches con debounce de 3s que se llama en navigate/up/back/forward. Fuerza guardado en el método 'on_exit' en 'main.rs'.([`a3e1c20`](https://github.com/Jhanfer/blazepilot/commit/a3e1c204611948502372c2dc905bbf48a76150c5))

### Ui
- Se traduce labels de GitStatus a español (hardcoded)([`b0d4eed`](https://github.com/Jhanfer/blazepilot/commit/b0d4eedc79130d0a9904b0ae8aa71ef882f0b810))
## [0.7.0] - 2026-04-24

### 🚀 Features
- Migrar a egui 0.34 y mejorar el sistema de configuración([`7c5cb2b`](https://github.com/Jhanfer/blazepilot/commit/7c5cb2b06cad3cafd14624a0dce42433d5c8b6b8))
## [0.6.1] - 2026-04-18

### 🐛 Bug fixes
- Estabilidad, compatibilidad y build de 0.6.1([`0a48001`](https://github.com/Jhanfer/blazepilot/commit/0a480013e2fb04e4f33a145393f0ef785007a5e3))
## [0.6.0] - 2026-04-18

### 🚀 Features
- **(cache)** Colores por carpeta con FileId e IconCache con tint([`1b34f64`](https://github.com/Jhanfer/blazepilot/commit/1b34f64ff65636b1b745ed54dec070eba8d98861))

### 🐛 Bug fixes
- Carga de peso en isla y validación de hotkeys/clipboard([`ec517c1`](https://github.com/Jhanfer/blazepilot/commit/ec517c11999679ce522ddd322b195fc9356c307b))

### ⚙️ Miscelánea
- Versionado 0.6.0 y actualizar dependencias([`ddc6f30`](https://github.com/Jhanfer/blazepilot/commit/ddc6f30ac32c5e4c935f3ce0bf44ac340a4e016a))

### Change
- Usar unidades decimales SI para tamaños de archivo. Cambiado base de cálculo de 1024 a 1000 para mostrar tamaños.([`5846dcb`](https://github.com/Jhanfer/blazepilot/commit/5846dcb478fc1694e9650be9f01fdc225850285a))
## [0.5.0] - 2026-04-17

### 🚀 Features
- **(ui)** Añadir 'Abrir en terminal' y hotkeys en menús contextuales([`e4528e1`](https://github.com/Jhanfer/blazepilot/commit/e4528e1b5a4d9b9acf8c1b1ec153f08591d1e548))- **(tabs)** Añadir sistema de pestañas con isla flotante y atajos([`e63ba9f`](https://github.com/Jhanfer/blazepilot/commit/e63ba9f0498bcd97f00bb8f7add422a9efeeb92f))- **(search)** Reescrita la búsqueda recursiva con jwalk y type-to-search([`d631f52`](https://github.com/Jhanfer/blazepilot/commit/d631f52c53862b615f4149eb49a18a0c5192c43e))

### 🐛 Bug fixes
- **(selección)** Evitar el panic al navegar con flechas en lista vacía o sin selección. Se producía 'index out of range' en BitVec al presionar ArrowUp/ArrowDown cuando no existían selecciones.([`b9b3c74`](https://github.com/Jhanfer/blazepilot/commit/b9b3c74be3358b3de635b9a79e3e61258cdc676d))- Quitar parámetros no usados y corregir E0432([`5e9136e`](https://github.com/Jhanfer/blazepilot/commit/5e9136ec244db072dcb590fddda91022cf6fb2f3))

### Fix
- Corrección en la activación del icono de pegado. Ahora funciona cuando detecte que tiene documentos que pegar.([`c2df1c4`](https://github.com/Jhanfer/blazepilot/commit/c2df1c4aed71109e76a4a8f5ae7a19a3ce90127f))
## [0.4.0] - 2026-04-15

### 🚀 Features
- Añadir atajos de teclado y ratón para gestión de archivos([`0c79dab`](https://github.com/Jhanfer/blazepilot/commit/0c79dabab098befdb46ce87ff7b83315fde40386))

### 🐛 Bug fixes
- Corregir bugs críticos de papelera, UI y ordenamiento([`640aa13`](https://github.com/Jhanfer/blazepilot/commit/640aa13c771a39f75f71b7a42b002c3419743b2a))

### Cambios
- -Detección de tamaño de directorios recursivos: se ha retirado 'get_recursive_size' de 'TabState' y creado un manager de tamaños para mejor manejo (SizerManager). Realizadas mejoras en la detección y creado canales para disparar los eventos.([`1fbbf69`](https://github.com/Jhanfer/blazepilot/commit/1fbbf6964d42c4f030818233a0a73bc08d81649a))

### Chore
- Versionando a '0.4.0'([`059c20b`](https://github.com/Jhanfer/blazepilot/commit/059c20be37818b230e78734b59186d3425144707))

### Fix
- Corrección de 'release.yml'([`7b06ea6`](https://github.com/Jhanfer/blazepilot/commit/7b06ea6cc93851504d935b04143f2e5caf209a48))- Corrección de 'release.yml'([`9eb04e0`](https://github.com/Jhanfer/blazepilot/commit/9eb04e0936a2f0a95a4a31642f560d580c2e413e))
## [0.3.0] - 2026-04-10

### 🚀 Features
- V0.3.0 - soporte de notificaciones, gestión de tamaños, papelera freedesktop y mejoras de seguridad([`f950f39`](https://github.com/Jhanfer/blazepilot/commit/f950f3953ab09b57261d8cc2027a266a6df02b5a))
## [0.2.0] - 2026-04-07

### Add
- Implementado sistema de actualizaciones automáticas en ui.([`41caf2e`](https://github.com/Jhanfer/blazepilot/commit/41caf2eb21fbe4a09c69e22c50efa23e3502ddc7))
## [0.1.1] - 2026-04-07

### Add
- Añadiendo dependencia openssl para compilación remota.([`e8f01d5`](https://github.com/Jhanfer/blazepilot/commit/e8f01d52cfe88460a367e99e4e808db782ceb8fe))

### Changes
- Cambios en release.yml([`78b466a`](https://github.com/Jhanfer/blazepilot/commit/78b466a1c036ebab34598149ec93818994014850))

### Test
- Reversionando cargo([`71acd9d`](https://github.com/Jhanfer/blazepilot/commit/71acd9d943df5b1ae09bd9ee2ba576e659c136fa))

### Tests
- Cambiando yml.([`d64aec8`](https://github.com/Jhanfer/blazepilot/commit/d64aec82d792b89d83c4b25817df78ea68969b8a))- Cambiando yml.([`1434cd6`](https://github.com/Jhanfer/blazepilot/commit/1434cd6fdc333f6fa22914450533e24b327d21cb))- Cambiando yml.([`d0127b0`](https://github.com/Jhanfer/blazepilot/commit/d0127b0389f70ad21366cef35acf681c48b6986a))- Cambiando yml.([`b3ccc24`](https://github.com/Jhanfer/blazepilot/commit/b3ccc2459d314d92503a361935bce29f31db1b0a))
