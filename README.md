# 🔥 BlazePilot

**Explorador de archivos ultrarrápido** hecho con **egui** en Rust ⚡

Un gestor de archivos gráfico moderno, ligero y altamente personalizable. Navega por tus archivos con fluidez, colores por carpeta, caché inteligente y atajos de teclado potentes.

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![egui](https://img.shields.io/badge/egui-FF9900?logo=egui&logoColor=white)
![License](https://img.shields.io/badge/License-Apache%202.0-blue)
[![Latest Release](https://img.shields.io/github/v/release/Jhanfer/blazepilot)](https://github.com/Jhanfer/blazepilot/releases/latest)

## ✨ Características

- ⚡ **Extremadamente rápido** gracias a Rust + egui
- 🎨 **Colores personalizados por carpeta** (nuevo en v0.6.0)
- 🗃️ Caché genérico e inteligente (archivos, tamaños e iconos)
- 📏 Cálculo de tamaño de carpetas con “isla flotante”
- ⌨️ **Atajos de teclado** completos (F5, Delete, Ctrl+V, Cmd+R, etc.)
- 📋 Soporte total de **Cut / Copy / Paste**
- 🖱️ Menú contextual + selector de color con **preview en vivo**
- 🧩 Identificador único `FileId` (estable aunque renombres o muevas archivos)

## 🚀 Instalación

Solo descarga el binario (no requiere instalación ni dependencias):

    1. Ve a la página de **[Releases](https://github.com/Jhanfer/blazepilot/releases/latest)**
    2. Descarga el binario para tu sistema operativo (actualmente solo linux soportado)
    3. Abre una terminal en la carpeta descargada y dale permisos (Linux):

```bash
chmod +x blazepilot
```

4. ¡Ejecútalo!
```bash
./blazepilot
```

## 🎮 Uso básico

+ Flechas para moverte
+ Enter o clic para abrir carpetas
+ Clic derecho → menú contextual
+ F5 o Cmd/Ctrl + R → recargar caché completo
+ Delete → eliminar
+ Ctrl + C / X / V → copiar, cortar y pegar


## 📋 Próximas mejoras (Roadmap)

+ Soporte completo y nativo para Windows y MacOS
+ Temas completos y configurables
+ Vista previa de archivos (imágenes, texto, etc.)
+ Plugins / extensiones

## 🛠️ Compilar desde fuente
```bash
git clone https://github.com/Jhanfer/blazepilot.git
cd blazepilot
cargo run --bin blazepilot
```

## 📄 Licencia
Este proyecto está bajo la licencia Apache License 2.0 — ver el archivo LICENSE para más detalles.

## 💜 ¿Te gusta BlazePilot? 
¡Dale una ⭐ al repositorio y ayúdame a crecer! 🚀
Hecho con ❤️ por **[Jhanfer](https://github.com/Jhanfer/)**
