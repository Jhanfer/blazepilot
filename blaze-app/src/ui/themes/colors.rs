use egui::Color32;

// Tonos principales de fondo y paneles
pub const COLOR_BG_MAIN: Color32 = Color32::from_rgb(13, 6, 20); // Fondo general ultra oscuro
pub const COLOR_BG_PANEL: Color32 = Color32::from_rgb(27, 17, 36);   // Fondo de los paneles laterales y principal
pub const COLOR_BG_CONTAINER: Color32 = Color32::from_rgb(37, 23, 49);   // Elementos contenedores (ej. cuadro de búsqueda)
pub const COLOR_MAIN_BUTTONS: Color32 = Color32::from_rgb(40, 30, 48);

// Colores de acento y estados activos
pub const COLOR_ACCENT_PURPLE: Color32 = Color32::from_rgb(140, 75, 247);    // El morado brillante de selección (ej. barra de volumen, botón activo)
pub const COLOR_ACCENT_GLOW: Color32 = Color32::from_rgb(186, 110, 255);      // El tono neón más claro del resplandor superior
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(122, 106, 133);       // Color de los iconos y texto secundario desvanecido

// Colores específicos de las carpetas
pub const COLOR_FOLDER_PURPLE: Color32 = Color32::from_rgb(163, 97,255);    // El morado de los iconos de carpeta estándar
pub const COLOR_FOLDER_RED: Color32 = Color32::from_rgb(214,69,69);       // El color rojo/coral de la carpeta "searxng"

// Tonos de texto principales
pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(255,255,255);// Blanco puro para textos principales
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(226,213,237);   // Blanco violáceo para nombres de archivos y carpetas
