# Changelog

---
## [0.20.0](https://github.com/Jhanfer/blazepilot/compare/v0.19.1..v0.20.0) - 2026-08-25

### ✦ Bug fixes

- **(fonts)** Corrige condición de carrera en token de cancelación([`5028ad3`](https://github.com/Jhanfer/blazepilot/commit/5028ad363e0c7670ff97d6fc1052e87c928106ce)) - Jhanfer
- **(wayland_dnd)** Espera al hilo DnD antes de destruir wl_display([`f90a4d6`](https://github.com/Jhanfer/blazepilot/commit/f90a4d6abf974d8c1a9119bad8168fa9a37aadf3)) - Jhanfer

### ✦ Docs

- **(readme)** Rediseña documentación con gifs, añade logo y elimina recursos obsoletos([`18516b9`](https://github.com/Jhanfer/blazepilot/commit/18516b92e330283cdd2be38131e334a9717d9848)) - Jhanfer
- **(readme)** Actualiza README.md por limpieza([`d81e67b`](https://github.com/Jhanfer/blazepilot/commit/d81e67b3d7524e4d08705bfa660fe62f8b4722f6)) - Jhanfer

### ✦ Miscelánea

-  Cambia a un nivel más bajo de compresión en UPX([`eafdf8d`](https://github.com/Jhanfer/blazepilot/commit/eafdf8de1c6197cbdcb5b7d8c0d2d11c74db0868)) - Jhanfer
-  Arregla avisos clippy chunks_exact([`f1fec2c`](https://github.com/Jhanfer/blazepilot/commit/f1fec2c0e88a956277e9a05bcaf01a619d268c92)) - Jhanfer
-  Arregla aviso clippy chunks_exact en thumbnails/utils.rs([`62e3536`](https://github.com/Jhanfer/blazepilot/commit/62e3536dc3fbedc78a567a667e72b99bffb5e55f)) - Jhanfer

### ✦ Performance

- **(core)** Optimiza cachés con LruCache, refactoriza reproductor multimedia y mejora gestión de fuentes por directorio([`88f165e`](https://github.com/Jhanfer/blazepilot/commit/88f165e373d19f56e33fd11ff4f3937699344e6b)) - Jhanfer
- **(memory)** Configura mimalloc con purga agresiva, libera memoria por frame y mejora gestión de texturas([`9bb8c0a`](https://github.com/Jhanfer/blazepilot/commit/9bb8c0a9a1c2f7dc59a69e31345dfce1a6203de2)) - Jhanfer
- **(memory)** Corrige constantes mimalloc, mueve free_mi a limpieza periódica y optimiza concurrencia con Arc y peek([`bac47b8`](https://github.com/Jhanfer/blazepilot/commit/bac47b83612fdd620220ded4d876492a12f11c8b)) - Jhanfer
- **(sizer)** Optimiza uso de memoria y actualizaciones de sizes([`d2c03d4`](https://github.com/Jhanfer/blazepilot/commit/d2c03d4929759f0bb9c36b87db2efbbb23568994)) - Jhanfer
- **(thumbnails)** Añade límites de tamaño, optimiza memoria con BufReader y mejora seek en video([`40158a4`](https://github.com/Jhanfer/blazepilot/commit/40158a4b01960dfcf2981f05fc010a0c3155bd30)) - Jhanfer

### ✦ Refactor

- **(window)** Migra de eframe a winit y wgpu, añade backend propio y mejora control de renderizado([`c310f6c`](https://github.com/Jhanfer/blazepilot/commit/c310f6c4db6c75b474f06d79f59a10c4c87ffc45)) - Jhanfer

---
## [0.19.1](https://github.com/Jhanfer/blazepilot/compare/v0.19.0..v0.19.1) - 2026-08-03

### ✦ Bug fixes

- **(ffmpeg)** Corrige enlace de librerías ligadas al binario, añade compresión UPX y expande features de compilación([`5a491e3`](https://github.com/Jhanfer/blazepilot/commit/5a491e34faa70660af02b23e6bbfb51540d120ba)) - Jhanfer

---
## [0.19.0](https://github.com/Jhanfer/blazepilot/compare/v0.18.0..v0.19.0) - 2026-07-30

### ✦ Bug fixes

-  Carga de fuentes del sistema via fc-match para fallback unicode (#3)([`545d7c2`](https://github.com/Jhanfer/blazepilot/commit/545d7c2550a946fc1d69785dcfcac8c24bb5c350)) - Jhan

### ✦ Docs

-  Añade secure policy([`cbd03fb`](https://github.com/Jhanfer/blazepilot/commit/cbd03fb81c7e7add32558581e246d934c82e17bc)) - Jhanfer

### ✦ Refactor

- **(theme)** Migra a sistema de tokens semánticos, implementa migración automática y unifica uso en UI (#5)([`2079dc4`](https://github.com/Jhanfer/blazepilot/commit/2079dc424498f408ddfdb4514e90bcdb2cac0bab)) - Jhan

### ✦ Release

- **(release)** Simplifica mensajes de workflow, añade locale y enlaces y mejora formato de changelog([`ea7321f`](https://github.com/Jhanfer/blazepilot/commit/ea7321f6955686e79c3cef668fef86ec2598af7d)) - Jhanfer

## [0.18.0] - 2026-07-24

### 🚀 Features
- **(media)** Reproductor integrado de video y audio, previsualización contextual y soporte para GIFs([`1faaf6d`](https://github.com/Jhanfer/blazepilot/commit/1faaf6dc52b5ea487d8bb5672012f31907595b7a))
- **(clipboard)** Implementa portapapeles en Wayland, corrige atajos de teclado y reemplaza TextEdit por componente personalizado([`7697447`](https://github.com/Jhanfer/blazepilot/commit/769744782df191520d150c8026939d2831e0d7ba))


### 🐛 Bug fixes
- **(preview)** Evita vista vacía y estabiliza navegación([`1cebb2a`](https://github.com/Jhanfer/blazepilot/commit/1cebb2a12aae671fe408978127561b8444320229))


### 📚 Docs
- Añade ALSA como dependencia obligatoria para audio en 0.18.0([`895a80d`](https://github.com/Jhanfer/blazepilot/commit/895a80dfcca4e71fc30818b9ffa2cb0d4bbfd75e))


### ⚙️ Miscelánea
- **(ci)** Reorganiza dependencias, corrige sintaxis([`d7076b5`](https://github.com/Jhanfer/blazepilot/commit/d7076b5b70d5dfa5a174e2121d35ad91f6b438cf))
- **(ci)** Agrega dependencias de FFmpeg en workflows de CI y release([`593ca12`](https://github.com/Jhanfer/blazepilot/commit/593ca12c2180dcc89b0068520611acbf96ea8168))
- **(ci)** Amplía deps FFmpeg con libavfilter, libavdevice y libswresample([`02088cc`](https://github.com/Jhanfer/blazepilot/commit/02088cc0ca1a6760068d7a52d59a1eacef37b8d6))

## [0.17.0] - 2026-07-11

### 🚀 Features
- **(app)** Refactor de drag & drop, internacionalización completa y descarga desde URL([`d1c5176`](https://github.com/Jhanfer/blazepilot/commit/d1c5176de883bfc616faa2c95309e80ded14fcf7))


### 🚀 Release
- **(release)** V0.17.0([`7b5dff3`](https://github.com/Jhanfer/blazepilot/commit/7b5dff3c4b8bc39e24812ebf9c28f576ac6cdd9c))

## [0.16.0] - 2026-07-03

### ⚡ Performance
- **(blaze)** Migra caché a DashMap, rendimiento en miniaturas mejorado y estados atómicos en caché manager([`e2d2f0a`](https://github.com/Jhanfer/blazepilot/commit/e2d2f0a339cbee3e9da8c74222add781fa723c0f))


### 📚 Docs
- Actualiza readmes para incluir dependencias de compilación([`c739c52`](https://github.com/Jhanfer/blazepilot/commit/c739c524effc2e71f398822ba496285fc71bd798))

## [0.15.1] - 2026-06-29

### 🚀 Release
- **(release)** V0.15.1 – actualización de deps, Rust 2024 y refactorización([`8e54215`](https://github.com/Jhanfer/blazepilot/commit/8e5421594dc42926b4a7d5bab4c904fe3c00015a))

## [0.15.0] - 2026-06-26

### 🚀 Features
- **(ui)** Ordenamiento independiente, navegación con historial y sidebar derecha rediseñada([`b2cc3eb`](https://github.com/Jhanfer/blazepilot/commit/b2cc3eb8487b033c4459cb6d16d5409c5f548ea4))
- **(theming)** Introduce sistema completo de temas dinámicos para BlazePilot([`abaa3eb`](https://github.com/Jhanfer/blazepilot/commit/abaa3eb8ceafe8a35b699fe61228186053820d32))


### 🐛 Bug fixes
- **(ui)** Corrige cierre de Quick Dialogs retornando should_close([`2bb07df`](https://github.com/Jhanfer/blazepilot/commit/2bb07df4981591b9205cb221740cfedfcaac47b0))


### ⚡ Performance
- **(cache)** Migra de tokio::sync::RwLock a parking_lot::RwLock y simplifica accesos([`6f51508`](https://github.com/Jhanfer/blazepilot/commit/6f51508afe9308922d01c59276c38d21bec37bc1))


### 📚 Docs
- Corrige indentación checksum en release.yml([`16464cc`](https://github.com/Jhanfer/blazepilot/commit/16464cc881863962da77b5ad7ae3f9c34019ebbf))


### ⚙️ Miscelánea
- **(release)** Añade checksum SHA-256 y hace workflow reutilizable([`5b37226`](https://github.com/Jhanfer/blazepilot/commit/5b37226eef44bae1729d9d0a696e7f28fe62bd9a))


## [0.14.0] - 2026-06-19

### 🚀 Features
- **(preview)** Rediseño de vista previa con zoom por píxeles y fondo dinámico([`cf5fc54`](https://github.com/Jhanfer/blazepilot/commit/cf5fc54592853a6d48023b8232a839351a70117e))
- **(blaze)** Añade vista Grid, reorganiza módulos y config con debounce([`3add657`](https://github.com/Jhanfer/blazepilot/commit/3add6570e92640a1de51fa452d50feff670e6df1))


### ⚡ Performance
- **(images)** Reemplaza crate image por decodificadores ligeros([`e6ca639`](https://github.com/Jhanfer/blazepilot/commit/e6ca6390f6a44a831819cd8ae9317de927c6fa2a))


### ⚙️ Miscelánea
- Instala dependencias del sistema en workflow([`3e24fce`](https://github.com/Jhanfer/blazepilot/commit/3e24fcedf4a7a27d7c9b1e786b15e834ca0240f0))
- **(version)** Versiona a 0.14.0([`e74c6b7`](https://github.com/Jhanfer/blazepilot/commit/e74c6b76d8da78cacd59ef290d1aa50e33befc69))

## [0.13.0] - 2026-06-12

### 🐛 Bug fixes
- **(startup)** Reintentos en proceso hijo y panic=unwind([`bda7e59`](https://github.com/Jhanfer/blazepilot/commit/bda7e5955113d922f41d2cf66d5718b858c64cb2))


### 🚜 Refactor
- **(app)** Reemplaza reintentos por env por lista fija de configs([`7935c4c`](https://github.com/Jhanfer/blazepilot/commit/7935c4c56bb2ba3aaa9b1473820d9929bbd2db5f))
- **(core)** Reestructuración modular Linux y endurece CI a -D warnings([`4c73ea5`](https://github.com/Jhanfer/blazepilot/commit/4c73ea5a2304b0088dd48011f2387672b86bd60f))
- **(ui)** ModalDialog.render devuelve bool para control de cierre([`d55b670`](https://github.com/Jhanfer/blazepilot/commit/d55b670419ad9b205c83b46c30f303f34abd94b0))
- **(linux)** Simplifica fileopener, elimina async y channel_pool([`5057977`](https://github.com/Jhanfer/blazepilot/commit/505797743f3c95daeafda6588f8c8a3cdf106780))


### 📚 Docs
- **(readme)** Añade dos screenshots y actualiza README([`7005296`](https://github.com/Jhanfer/blazepilot/commit/7005296f4f991ee9951a93e79f4d5038e69aeafa))


### ⚙️ Miscelánea
- **(release)** Genera changelog automático en el cuerpo del release([`cf676ab`](https://github.com/Jhanfer/blazepilot/commit/cf676ab058962a80efc71fb30691c5e558e13dd7))
- **(github)** Añade FUNDING.yml para GitHub Sponsors([`10da90d`](https://github.com/Jhanfer/blazepilot/commit/10da90d716d3fdd9a30945581964f5c34234e0a7))
- Fuerza clippy a denegar warnings (-D warnings)([`eb5ed83`](https://github.com/Jhanfer/blazepilot/commit/eb5ed83e48118779b2ea7d6deade5d478de17c35))
- Mejora workflows con cache, concurrencia y release notes automáticas([`77926fa`](https://github.com/Jhanfer/blazepilot/commit/77926fa8fc0bac3105df25b5ea1a427c834e9b79))
- **(ci)** Añade nombre al job del CI([`1fa361c`](https://github.com/Jhanfer/blazepilot/commit/1fa361c4bdb71f57989cded81aea05d7ddd38da6))
- **(version)** V0.13.0([`ab0ed63`](https://github.com/Jhanfer/blazepilot/commit/ab0ed6363ecc82f81446bce2dbe7ebe100b0cef7))

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
