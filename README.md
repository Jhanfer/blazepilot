# 🔥 BlazePilot
🌐 **[English]** | 🇪🇸 **[Español](README.es.md)**

**Blazing-fast file explorer** built with **egui** in Rust ⚡

A modern, lightweight, and highly customizable graphical file manager. Navigate your files smoothly with multi-language support, a tag system, thumbnails, Git integration, disk management, and much more.

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![egui](https://img.shields.io/badge/egui-FF9900?logo=egui&logoColor=white)
![License](https://img.shields.io/badge/License-Apache%202.0-blue)
[![Latest Release](https://img.shields.io/github/v/release/Jhanfer/blazepilot)](https://github.com/Jhanfer/blazepilot/releases/latest)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/Jhanfer/blazepilot)
[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/jhanfer)

<img src="screenshots/blaze_example1.webp" width="1914" alt="BlazePilot screenshot 1" style="max-width:100%;" />
<img src="screenshots/blaze_example2.webp" width="1914" alt="BlazePilot screenshot 2" style="max-width:100%;" />

---

## ✨ Features

### ⚡ Performance
- Blazing fast thanks to Rust + egui with a **wgpu** backend (hardware acceleration)
- **MiMalloc** memory allocator to reduce fragmentation
- **Tokio** asynchronous runtime — file operations never block the UI
- LRU cache of 50 directories with debounced saving (3s)

### 📁 File operations
- **Copy / Cut / Paste** with global clipboard and conflict resolution
- **Rename** preserving original casing
- **Delete** with trash support (XDG Trash compliant)
- **Create** files and folders
- **Move** with drag & drop
- **Undo** file operations (Ctrl+Z)
- **Extract ZIP** directly from the explorer

### 🔍 Navigation and search
- **Tabbed navigation** with history (Ctrl+← / Ctrl+→)
- **Recursive search** with the `rec:` prefix — powered by jwalk
- **Type-to-search** for instant filtering in the current directory
- **Auto-scroll** when selecting search results

### 🏷️ Tag System / Quick Access *(v0.11.0)*
- Flexible tags replacing hardcoded favorites
- Toggle Tag/Normal view with **Ctrl+T**
- Create tags with **Ctrl+Shift+T**
- Bottom floating island for tag management

### 🎨 Interface and customization
- **Custom folder colors** with color picker and live preview
- **Thumbnails** with persistent disk cache
- **Icons** with SVG rasterization and concurrency semaphore
- Centralized color palette and rounded corners
- **Image preview** in dedicated dialog
- Full context menu with all operations

### 🌍 Internationalization *(v0.12.0)*
- **6 languages**: English, Spanish, French, German, Italian, Russian
- Language switching **at runtime** without restarting

### 🖥️ System integration
- **Open with** — application selector based on MIME type
- **Open in terminal** from any folder
- **Disk management** — mount/unmount with drive sidebar (UDisks2 / D-Bus)
- **Real MIME detection** using `xdg-mime` + magic byte signature
- **Git integration** — file status with state-specific colors
- **Automatic updates** with new version notification
- **Stable FileId** — the identifier persists even when renaming or moving files

---

## ⌨️ Keyboard shortcuts

### Navigation

| Shortcut | Action |
| :--- | :--- |
| `↑` / `↓` | Select previous / next item |
| `Enter` | Open selected folder or file |
| `Cmd + A` | Select all |
| `F5` / `Cmd + R` | Reload / refresh |
| Extra Mouse Button 1 | Navigate backward |
| Extra Mouse Button 2 | Navigate forward |

### File operations

| Shortcut | Action |
| :--- | :--- |
| `Delete` | Delete (move to trash) |
| `Ctrl + Z` | Undo last operation |
| `Cmd + C` | Copy |
| `Cmd + X` | Cut |
| `Cmd + V` | Paste |
| `Cmd + Shift + N` | Create new folder |
| `Cmd + Shift + F` | Create new file |

### Search and view

| Shortcut | Action |
| :--- | :--- |
| `Alt + R` | Toggle recursive search |
| `Ctrl + T` | Toggle Tag / Normal view |
| `Ctrl + Shift + T` | Create new tag |

### Terminal

| Shortcut | Action |
| :--- | :--- |
| `Alt + T` | Open terminal in the current directory |

### Tabs

| Shortcut | Action |
| :--- | :--- |
| `Cmd + N` | New tab |
| `Cmd + W` | Close current tab |
| `Ctrl + Tab` / `Ctrl + →` | Next tab |
| `Ctrl + Shift + Tab` / `Ctrl + ←` | Previous tab |
| `Ctrl + 1` … `Ctrl + 5` | Go to tab 1–5 |

### Dialogs

| Shortcut | Action |
| :--- | :--- |
| `Enter` | Confirm (rename / create folder or file) |
| `Escape` | Cancel (rename / create folder or file) |

---

## 🚀 Installation

Just download the binary — no installation or external dependencies required:

1. Go to the **[Releases](https://github.com/Jhanfer/blazepilot/releases/latest)** page
2. Download the binary for your system (currently Linux only)
3. Give it execution permissions:

```bash
chmod +x blazepilot
```

4. Run it!

```bash
./blazepilot
```

---

## 🛠️ Build from source

```bash
git clone https://github.com/Jhanfer/blazepilot.git
cd blazepilot
cargo run --bin blazepilot
```

Compilation requirements: rust nightly, cargo, make, ninja, nasm, libdav1d, pkg-config y development headers x11, wayland and dbus.


---

## 📋 Roadmap

- Full native support for Windows and macOS
- Complete and configurable themes
- Plugins / extensions

---

## 📄 License

This project is licensed under the **Apache License 2.0** — see the `LICENSE` file for details.

---

## 💜 Do you like BlazePilot?

Give the repository a ⭐ and help me grow! 🚀

Made with ❤️ by **[Jhanfer](https://github.com/Jhanfer/)**