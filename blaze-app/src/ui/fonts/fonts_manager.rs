use egui::{FontData, FontDefinitions, FontFamily};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock};

pub static GLOBAL_FONT_MANAGER: LazyLock<Mutex<FontManager>> =
    LazyLock::new(|| Mutex::new(FontManager::new()));

pub fn with_fonts<F>(f: impl FnOnce(&mut FontManager) -> F) -> F {
    f(&mut GLOBAL_FONT_MANAGER.lock())
}

//enum de scripts con desplazamiento
//todavía wip
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum FontScripts {
    Lat = 1 << 0,
    Cjksc = 1 << 1,
    Cjktc = 1 << 2,
    Cjkjp = 1 << 3,
    Cjkkr = 1 << 4,
    Cjkhk = 1 << 5,
    Emoji = 1 << 6,
    Ar = 1 << 7,
    He = 1 << 8,
    Th = 1 << 9,
    Lo = 1 << 10,
    Dev = 1 << 11,
    Bn = 1 << 12,
    Ta = 1 << 13,
    Te = 1 << 14,
    Gu = 1 << 15,
    Ml = 1 << 16,
    Kn = 1 << 17,
    My = 1 << 18,
    Km = 1 << 19,
    Eth = 1 << 20,
    Geo = 1 << 21,
    Arm = 1 << 22,
    Tib = 1 << 23,
    Sin = 1 << 24,
    Mng = 1 << 25,
    Chr = 1 << 26,
}

pub struct CustomFontData {
    data: Vec<u8>,
}

pub struct FontManager {
    cache: Mutex<LruCache<String, Arc<CustomFontData>>>,
    pub loaded_scripts: u32,
    pub dirty: bool,
    pub fonts: FontDefinitions,
}

impl FontManager {
    pub fn new() -> Self {
        let def_cap: NonZeroUsize = match NonZeroUsize::new(10) {
            Some(n) => n,
            None => unreachable!(),
        };

        let cap = NonZeroUsize::new(10).unwrap_or(def_cap);

        let mut fonts = FontDefinitions::default();

        //cargado de la fuente estática
        fonts.font_data.insert(
            "NotoSans".to_owned(),
            FontData::from_static(include_bytes!(
                "../.././ui/assets/noto/NotoSans-Regular.ttf"
            ))
            .into(),
        );

        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "NotoSans".to_owned());

        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "NotoSans".to_owned());

        Self {
            cache: Mutex::new(LruCache::new(cap)),
            loaded_scripts: 0,
            dirty: true,
            fonts,
        }
    }

    fn load_system_fonts<'a>(
        &self,
        candidates: &'a [(char, &'a str)],
    ) -> Option<Vec<(&'a str, Vec<u8>)>> {
        //buscar las fuentes con fontconfig
        let mut result = vec![];
        for &(ch, key) in candidates {
            let output = std::process::Command::new("fc-match")
                .args(["--format=%{file}", &format!(":charset={:04X}", ch as u32)])
                .output()
                .ok()?;

            let path = String::from_utf8(output.stdout).ok()?;
            //luego poner esta funciṕn dentro de un task para evitar congelar ui
            if let Ok(bytes) = std::fs::read(path.trim()) {
                result.push((key, bytes));
            }
        }

        Some(result)
    }

    pub fn process_file(&mut self, text: &str) {
        //procesa cada nobre de archivo buscando la fuentes correspondioentes por caracteres
        let required = self.detect_scripts(text);

        if (required & !self.loaded_scripts) == 0 {
            return;
        }

        let missing = required & !self.loaded_scripts;

        self.loaded_scripts |= missing;

        let candidates = self.get_required_fonts(missing);

        match self.load_system_fonts(&candidates) {
            Some(sys_fonts) => {
                let mut cache = self.cache.lock();

                for (key, bytes) in sys_fonts {
                    //asignación de bytes al caché
                    cache.put(key.to_string(), Arc::new(CustomFontData { data: bytes }));
                }
            }
            None => return,
        }

        self.dirty = true;
    }

    pub fn detect_scripts(&self, text: &str) -> u32 {
        //rompe el texto en caracteres y detecta la unidata para clasificar las fuentes
        let mut mask = 0u32;

        for ch in text.chars() {
            let code = ch as u32;

            mask |= match code {
                0x0000..=0x007F => FontScripts::Lat as u32,
                0x0590..=0x05FF => FontScripts::He as u32,
                0x3040..=0x30FF => FontScripts::Cjkjp as u32,
                0xAC00..=0xD7AF | 0x1100..=0x11FF => FontScripts::Cjkkr as u32,
                0x0600..=0x06FF | 0x0750..=0x077F => FontScripts::Ar as u32,
                0x0E00..=0x0E7F => FontScripts::Th as u32,
                _ => 0,
            }
        }

        mask
    }

    pub fn get_required_fonts(&self, mask: u32) -> Vec<(char, &'static str)> {
        //usa la máscara de scripts para buscar una posible fuente en el sistema
        let mut fonts = Vec::new();

        if (mask & FontScripts::Lat as u32) != 0 {
            fonts.push(('A', "FontLAT"));
        }
        if (mask & FontScripts::He as u32) != 0 {
            fonts.push(('א', "FontHE"));
        }
        if (mask & FontScripts::Ar as u32) != 0 {
            fonts.push(('ع', "FontAR"));
        }
        if (mask & FontScripts::Cjkjp as u32) != 0 {
            fonts.push(('こ', "FontJP"));
        }
        if (mask & FontScripts::Cjkkr as u32) != 0 {
            fonts.push(('한', "FontKR"));
        }
        if (mask & FontScripts::Th as u32) != 0 {
            fonts.push(('อ', "FontTH"));
        }

        fonts
    }

    pub fn build_font_definitions(&mut self) -> FontDefinitions {
        //construye las fuentes para egui usando la caché generada
        let cache = self.cache.lock();

        for (key, bytes) in cache.iter() {
            self.fonts.font_data.insert(
                key.to_owned(),
                FontData::from_owned(bytes.data.clone()).into(),
            );

            self.fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push(key.clone());
        }

        self.fonts.clone()
    }
}

#[cfg(test)]
mod test {
    use crate::ui::fonts::fonts_manager::FontManager;

    #[test]
    fn testeo() {
        let mut fm = FontManager::new();

        fm.process_file("こんにちは, 안녕하세요");
    }
}
