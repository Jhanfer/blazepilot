<p align="center">
  <img src="blazeresources/LogoBlazepilot.png" width="180" alt="BlazePilot Logo">
</p>

<h1 align="center">BlazePilot</h1>

<p align="center">
  🌐 <a href="README.md"><strong>English</strong></a> • 🇪🇸 <strong>Español</strong>
</p>

<p align="center">
  Explorador de archivos hecho con <b>Egui</b> en <b> Rust</b>.
</p>

*BlazePilot ha nacido como un proyecto personal. Estaba cansado de las limitaciones de los exploradores que utilizaba a diario, así que empecé a desarrollarlo como una forma de practicar Rust mientras iba aprendiendo, adaptándolo a mis propias necesidades.*

BlazePilot es un gestor de archivos moderno y personalizable. Navega por tus archivos de manera fluida, incorpora un sistema de etiquetas para organizar los archivos, soporte a varios idiomas, miniaturas, soporte parcial a Git, gestión de discos y más.

> [!IMPORTANT]
> Actualmente BlazePilot es compatible con Linux. El soporte para Windows y macOS se encuentra en desarrollo.

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/egui-FF9900?logo=egui&logoColor=white" alt="egui">
  <img src="https://img.shields.io/badge/License-Apache%202.0-blue" alt="License">
  <a href="https://github.com/Jhanfer/blazepilot/releases/latest">
    <img src="https://img.shields.io/github/v/release/Jhanfer/blazepilot" alt="Latest Release">
  </a>
  <a href="https://deepwiki.com/Jhanfer/blazepilot">
    <img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki">
  </a>
  <a href="https://ko-fi.com/jhanfer">
    <img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Ko-fi">
  </a>
</p>

---

## Características

### Rendimiento
- Carga rápida asíncrona de archivos 
- Miniaturas y cálculo del tamaño de los directorios en segundo plano
- Runtime asíncrono **Tokio** para ejecutar operaciones de archivos sin bloquear la interfaz
- Asignador de memoria **mimalloc**

<p align="center">
	<img src="blazeresources/fileload.gif" width="280" alt="BlazePilot Logo">
</p>

### Operaciones de archivos
- Copiar, pegar, cortar gestionadas por un portapapeles global propio
- Renombrado mantiene el casing original
- Eliminar con soporte a la papelera del sistema
- Crear archivos y carpetas
- Mover con drag & drop dentro de la app
- Deshacer operaciones de archivos con **Ctrl + Z**
- Soporte básico a extracción de ZIP y otros formatos directamente desde el explorador

<p align="center">
	<img src="blazeresources/fileops.gif" width="280" alt="BlazePilot Logo">
</p>

### Drag & Drop (Wayland)
- Soporte nativo para drag & drop en Wayland
- Detección del tipo de contenido mediante MIME y magic bytes
- Acepta archivos, texto, imágenes y URLs
- Los archivos, imágenes y texto arrastrados se guardan directamente en el directorio actual
- Las URLs de imágenes ofrecen descargarlas
- Las URLs de páginas web pueden abrirse en el navegador

 >[!NOTE]
Cuando se arrastran datos desde otra aplicación, Blaze no depende únicamente del tipo MIME anunciado. Inspecciona los _magic bytes_ del contenido para identificar correctamente imágenes, vídeos, texto, URLs y otros formatos antes de decidir cómo procesarlos.

<p align="center">
	<img src="blazeresources/dnd.gif" width="280" alt="BlazePilot Logo">
</p>

### Navegación y búsqueda
- Navegación por pestañas **Ctrl + <- / Ctrl + -> / Ctrl + Nums**
- Búsqueda recursiva con el prefijo **rec:** en el buscador
- Búsqueda instantánea al escribir para filtrado en el directorio actual

<p align="center">
	<img src="blazeresources/search.gif" width="280" alt="BlazePilot Logo">
</p>

### Sistema de etiquetas / acceso rápido
- Etiquetas que permiten organización por tipos
- Toggle de vista etiquetas/normal con **Ctrl+T**
- Crear etiqueta con **Ctrl + Shift + T**

<p align="center">
	<img src="blazeresources/tags.gif" width="280" alt="BlazePilot Logo">
</p>

### Interfaz y personalización
- Colores de carpeta personalizables
- Miniaturas con caché persistente en disco
- Iconos con rasterizado SVG y semáforo de concurrencia
- Paleta de colores centralizada y bordes redondeados
- Vista previa de imágenes en diálogo dedicado

<p align="center">
	<img src="blazeresources/theming.gif" width="280" alt="BlazePilot Logo">
</p>

### Internacionalización
- **6 idiomas**: inglés, español, francés, alemán, italiano, ruso
- Cambio de idioma en runtime sin reiniciar

<p align="center">
	<img src="blazeresources/lang.gif" width="280" alt="BlazePilot Logo">
</p>

### Gestión e Integración con el sistema
- *Abrir con...* inicializa un selector de aplicaciones basado en tipo MIME
- Abrir la terminal desde cualquier carpeta
- Gestión de discos con montaje y desmontaje
- Integración Git que lee estados de los archivos de un repositorio local
- Actualizaciones automáticas con notificación de nueva versión
- Identificador de archivos con File ID persistente
- Ofrece instalar en caso de no estar instalado

<p align="center">
	<img src="blazeresources/fileopen.gif" width="280" alt="BlazePilot Logo">
</p>

---

## Atajos de teclado

### Navegación

| Atajo              | Acción                                    |
| :----------------- | :---------------------------------------- |
| `↑` / `↓`          | Seleccionar elemento anterior o siguiente |
| `Enter`            | Abrir carpeta o archivo seleccionado      |
| `Cmd + A`          | Seleccionar todo                          |
| `F5` / `Cmd + R`   | Recargar / refrescar                      |
| Botón ratón Extra1 | Navegar atrás                             |
| Botón ratón Extra2 | Navegar adelante                          |

### Operaciones de archivos

| Atajo             | Acción                                                     |
| :---------------- | :--------------------------------------------------------- |
| `Delete`          | Mover a papelera (eliminar si ya se encuentra en papelera) |
| `Ctrl + Z`        | Deshacer última operación                                  |
| `Cmd + C`         | Copiar                                                     |
| `Cmd + X`         | Cortar                                                     |
| `Cmd + V`         | Pegar                                                      |
| `Cmd + Shift + N` | Crear nueva carpeta                                        |
| `Cmd + Shift + F` | Crear nuevo archivo                                        |

### Búsqueda y vista

| Atajo              | Acción                            |
| :----------------- | :-------------------------------- |
| `Alt + R`          | Activar búsqueda recursiva        |
| `Ctrl + T`         | Alternar vista etiquetas / normal |
| `Ctrl + Shift + T` | Crear nuevo tag                   |

### Terminal

| Atajo | Acción |
| :--- | :--- |
| `Alt + T` | Abrir terminal en el directorio actual |

### Pestañas

| Atajo                              | Acción                |
| :--------------------------------- | :-------------------- |
| `Cmd + N`                          | Nueva pestaña         |
| `Cmd + W`                          | Cerrar pestaña actual |
| `Ctrl + Tab` / `Ctrl + ->`         | Siguiente pestaña     |
| `Ctrl + Shift + Tab` / `Ctrl + <-` | Pestaña anterior      |
| `Ctrl + 1` … `Ctrl + 5`            | Ir a pestaña 1–5      |

### Renombrado y creación de archivos

| Atajo    | Acción                                        |
| :------- | :-------------------------------------------- |
| `Enter`  | Confirmar renombrar / crear carpeta o archivo |
| `Escape` | Cancelar renombrar / crear carpeta o archivo  |

---

## Instalación

BlazePilot se distribuye como un único binario. Basta con descargarlo y ejecutarlo:
> [!NOTE]
> BlazePilot utiliza `wgpu` como renderizador de eframe, por lo que requiere un sistema con soporte gráfico compatible (Vulkan en la mayoría de distribuciones Linux).

1. Ve a la página de **[Releases](https://github.com/Jhanfer/blazepilot/releases/latest)**
2. Descarga el binario para tu sistema (actualmente Blaze es sólo compatible con Linux)
> [!IMPORTANT]
> A partir de la versión **0.18.0**, BlazePilot requiere: 
> - ALSA para salida de audio:
>   - Arch Linux / Manjaro: `sudo pacman -S alsa-lib`
>   - Ubuntu / Debian: `sudo apt install libasound2`
>   - Fedora: `sudo dnf install alsa-lib`

3. Dale permisos de ejecución:

```bash
chmod +x blazepilot-x86_64-unknown-linux-gnu-vX.X.X
```

4. ¡Ejecútalo!

```bash
./blazepilot-x86_64-unknown-linux-gnu-vX.X.X
```

---

## Compilar desde fuente

```bash
git clone https://github.com/Jhanfer/blazepilot.git
cd blazepilot
cargo run --bin blazepilot
```

>[!NOTE]
> **Requisitos de compilación**
> - Rust nightly
> - Cargo
> - Meson
> - Ninja
> - NASM
> - YASM
> - pkg-config
> - OpenSSL (`libssl-dev`)
> - ALSA (`libasound2-dev`)
> - FFmpeg (libavutil, libavcodec, libavformat, libswscale, libavfilter y libswresample)
> - dav1d (`libdav1d-dev`)
> - Bibliotecas de desarrollo para:
>   - X11 (`libx11-dev`)
>   - XKB (`libxkbcommon-dev`, `libxkbcommon-x11-dev`)
>   - Wayland (`libwayland-dev`)
>   - OpenGL / EGL / GLES (`libgl1-mesa-dev`, `libegl1-mesa-dev`, `libgles2-mesa-dev`)
>   - Vulkan (`libvulkan-dev`)
>   - D-Bus (`libdbus-1-dev`)


---

## Estado del proyecto

BlazePilot está en desarrollo activo. Aunque ya es utilizable, algunas funcionalidades continúan evolucionando y la compatibilidad con Windows y macOS todavía está en desarrollo.

---

## Roadmap

- Soporte completo y nativo para Windows y macOS
- Temas completos y configurables (todavía WIP)
- Plugins o extensiones

---

## Licencia

Este proyecto está bajo la licencia **Apache License 2.0**. Ver el archivo `LICENSE` para más detalles.

---

## ¿Te gusta BlazePilot?

¡Dale una ⭐ al repositorio y ayúdame a crecer!

Hecho con ❤️ por **[Jhanfer](https://github.com/Jhanfer/)**