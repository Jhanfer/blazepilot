// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use egui::{FontData, FontDefinitions, FontFamily};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, LazyLock};
use tokio_util::sync::CancellationToken;

pub static GLOBAL_FONT_MANAGER: LazyLock<Mutex<FontManager>> =
    LazyLock::new(|| Mutex::new(FontManager::new()));

pub fn with_fonts<F>(f: impl FnOnce(&mut FontManager) -> F) -> F {
    f(&mut GLOBAL_FONT_MANAGER.lock())
}

macro_rules! add_font_if_needed {
    ($mask:expr, $script:expr, $cha:expr, $key:expr, $fonts:expr) => {
        if ($mask & $script as u32) != 0 {
            $fonts.push(($cha, $key));
        }
    };
}

//enum de scripts con desplazamiento
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum FontScripts {
    Lat = 1 << 0,
    Cy = 1 << 1,
    Cjksc = 1 << 2,
    Cjkjp = 1 << 3,
    Cjkkr = 1 << 4,
    Emoji = 1 << 5,
    Ar = 1 << 6,
    He = 1 << 7,
    Th = 1 << 8,
    Lo = 1 << 9,
    Dev = 1 << 10,
    Bn = 1 << 11,
    Ta = 1 << 12,
    Te = 1 << 13,
    Gu = 1 << 14,
    Ml = 1 << 15,
    Kn = 1 << 16,
    My = 1 << 17,
    Km = 1 << 18,
    Eth = 1 << 19,
    Geo = 1 << 20,
    Arm = 1 << 21,
    Tib = 1 << 22,
    Sin = 1 << 23,
    Mng = 1 << 24,
    Chr = 1 << 25,
    Gr = 1 << 26,
    Pa = 1 << 27,
}

pub struct FontManager {
    pub fonts_cache: HashMap<Box<str>, Arc<FontData>>,
    pub active_fonts: Vec<Box<str>>,
    current_dir: Option<Arc<Path>>,
    fonts_dir: HashMap<Arc<Path>, HashSet<Box<str>>>,
    pub dirty: bool,
    pub needs_rebuild: bool,
    pub pending_tasks: Arc<AtomicUsize>,
    pub cancellation_token: CancellationToken,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts_cache: HashMap::new(),
            active_fonts: Vec::new(),
            fonts_dir: HashMap::new(),
            current_dir: None,
            dirty: false,
            needs_rebuild: false,
            pending_tasks: Arc::new(AtomicUsize::new(0)),
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn enter_dir(&mut self, path: &Arc<Path>) {
        let new_dir = path.clone();

        if let Some(ref old_dir) = self.current_dir {
            if *old_dir == new_dir {
                return;
            }
            self.cancellation_token.cancel();
            self.cancellation_token = CancellationToken::new();
            self.leave_directory(&old_dir.clone());
        }

        self.current_dir = Some(new_dir);
        self.dirty = true;
    }

    pub fn leave_directory(&mut self, path: &Arc<Path>) {
        let keys_to_remove = self.fonts_dir.remove(path).unwrap_or_default();

        let still_need: HashSet<Box<str>> = self
            .fonts_dir
            .values()
            .flat_map(|set| set.iter().cloned())
            .collect();

        for k in &keys_to_remove {
            if !still_need.contains(k) {
                self.fonts_cache.remove(k);
            }
        }

        self.active_fonts
            .retain(|key| self.fonts_cache.contains_key(key));

        if !keys_to_remove.is_empty() {
            self.needs_rebuild = true;
        }

        if self.current_dir.as_ref() == Some(path) {
            self.current_dir = None;
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

    pub fn process_file(&mut self, text: &str, dir: Arc<Path>) {
        //procesa cada nobre de archivo buscando la fuentes correspondioentes por caracteres
        let required = self.detect_scripts(text);
        let candidates = self.get_required_fonts(required);

        let missing: Vec<_> = candidates
            .into_iter()
            .filter(|(_, key)| !self.fonts_cache.contains_key(*key))
            .collect();

        if missing.is_empty() {
            return;
        }

        if let Some(sys_fonts) = self.load_system_fonts(&missing) {
            for (key, bytes) in sys_fonts {
                let key: Box<str> = key.into();

                if self.fonts_cache.contains_key(&key) {
                    continue;
                }

                let font_data = Arc::new(FontData::from_owned(bytes));

                self.fonts_cache.insert(key.clone(), font_data);

                self.fonts_dir
                    .entry(dir.clone())
                    .or_default()
                    .insert(key.clone());

                if !self.active_fonts.iter().any(|k| k == &key) {
                    self.active_fonts.push(key);
                }
            }
        }

        self.dirty = true;
    }

    pub fn build_font_definitions(&mut self) -> FontDefinitions {
        //uso de default para las fuentes de emojis
        let mut fonts = FontDefinitions::default();

        //se elimina todas las fuentes default que no se vayan a usar
        fonts.font_data.retain(|name, _| {
            let lower = name.to_lowercase();
            lower.contains("emoji")
        });

        for family_vec in fonts.families.values_mut() {
            family_vec.retain(|name| {
                let lower = name.to_lowercase();
                lower.contains("emoji")
            });
        }

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

        for keys in &self.active_fonts {
            let key_str: String = keys.to_string();
            if key_str == "NotoSans" {
                continue;
            };

            let Some(font_data_arc) = self.fonts_cache.get(keys) else {
                continue;
            };

            fonts
                .font_data
                .insert(key_str.clone(), font_data_arc.clone());

            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push(key_str);
        }

        self.dirty = false;

        fonts
    }

    pub fn detect_scripts(&self, text: &str) -> u32 {
        //rompe el texto en caracteres y detecta la unidata para clasificar las fuentes
        let mut mask = 0u32;

        for ch in text.chars() {
            let code = ch as u32;

            mask |= match code {
                0x0000..=0x007F
                | 0x0080..=0x00FF
                | 0x0100..=0x017F
                | 0x0180..=0x024F
                | 0x0250..=0x02AF
                | 0x02B0..=0x02FF
                | 0x0300..=0x036F
                | 0x1E00..=0x1EFF
                | 0x2C60..=0x2C7F
                | 0xA720..=0xA7FF
                | 0xAB30..=0xAB6F
                | 0x10780..=0x107BF
                | 0x1DF00..=0x1DFFF => FontScripts::Lat as u32,

                0x0400..=0x04FF
                | 0x0500..=0x052F
                | 0x2DE0..=0x2DFF
                | 0xA640..=0xA69F
                | 0x1C80..=0x1C8F => FontScripts::Cy as u32,

                0x2E80..=0x2EFF
                | 0x2F00..=0x2FDF
                | 0x2FF0..=0x2FFF
                | 0x3000..=0x303F
                | 0x3400..=0x4DBF
                | 0x4E00..=0x9FFF
                | 0xF900..=0xFAFF
                | 0xFE30..=0xFE4F
                | 0x2F800..=0x2FA1F
                | 0x20000..=0x2A6DF
                | 0x2A700..=0x2B73F
                | 0x2B740..=0x2B81F
                | 0x2B820..=0x2CEAF
                | 0x2CEB0..=0x2EBEF
                | 0x2EBF0..=0x2EE5F
                | 0x30000..=0x3134F
                | 0x31350..=0x323AF
                | 0x323B0..=0x3347F => FontScripts::Cjksc as u32,

                0x3040..=0x309F
                | 0x30A0..=0x30FF
                | 0x31F0..=0x31FF
                | 0xFF66..=0xFF9D
                | 0xFF9E..=0xFF9F => FontScripts::Cjkjp as u32,

                0x1100..=0x11FF
                | 0x3130..=0x318F
                | 0xAC00..=0xD7AF
                | 0xA960..=0xA97F
                | 0xD7B0..=0xD7FF => FontScripts::Cjkkr as u32,

                0x1F000..=0x1F02F
                | 0x1F030..=0x1F09F
                | 0x1F0A0..=0x1F0FF
                | 0x1F100..=0x1F1FF
                | 0x1F200..=0x1F2FF
                | 0x1F300..=0x1F5FF
                | 0x1F600..=0x1F64F
                | 0x1F650..=0x1F67F
                | 0x1F680..=0x1F6FF
                | 0x1F700..=0x1F77F
                | 0x1F780..=0x1F7FF
                | 0x1F800..=0x1F8FF
                | 0x1F900..=0x1F9FF
                | 0x1FA00..=0x1FA6F
                | 0x1FA70..=0x1FAFF
                | 0x1FB00..=0x1FBFF
                | 0xE0000..=0xE007F
                | 0xE0100..=0xE01EF => FontScripts::Emoji as u32,

                0x0600..=0x06FF
                | 0x0750..=0x077F
                | 0x0870..=0x089F
                | 0x08A0..=0x08FF
                | 0xFB50..=0xFDFF
                | 0xFE70..=0xFEFF
                | 0x10E60..=0x10E7F
                | 0x10EC0..=0x10EFF
                | 0x1EE00..=0x1EEFF => FontScripts::Ar as u32,

                0x0590..=0x05FF | 0xFB1D..=0xFB4F => FontScripts::He as u32,

                0x0E00..=0x0E7F => FontScripts::Th as u32,

                0x0E80..=0x0EFF => FontScripts::Lo as u32,

                0x0900..=0x097F | 0xA8E0..=0xA8FF | 0x11B00..=0x11B5F => FontScripts::Dev as u32,

                0x0980..=0x09FF => FontScripts::Bn as u32,

                0x0B80..=0x0BFF | 0x11FC0..=0x11FFF => FontScripts::Ta as u32,

                0x0C00..=0x0C7F => FontScripts::Te as u32,

                0x0A00..=0x0A7F => FontScripts::Pa as u32,

                0x0A80..=0x0AFF => FontScripts::Gu as u32,

                0x0D00..=0x0D7F => FontScripts::Ml as u32,

                0x0C80..=0x0CFF => FontScripts::Kn as u32,

                0x1000..=0x109F | 0xA9E0..=0xA9FF | 0xAA60..=0xAA7F | 0x116D0..=0x116FF => {
                    FontScripts::My as u32
                }

                0x1780..=0x17FF | 0x19E0..=0x19FF => FontScripts::Km as u32,

                0x1200..=0x137F
                | 0x1380..=0x139F
                | 0x2D80..=0x2DDF
                | 0xAB00..=0xAB2F
                | 0x1E7E0..=0x1E7FF => FontScripts::Eth as u32,

                0x10A0..=0x10FF | 0x1C90..=0x1CBF | 0x2D00..=0x2D2F => FontScripts::Geo as u32,

                0x0530..=0x058F => FontScripts::Arm as u32,
                0x0F00..=0x0FFF => FontScripts::Tib as u32,

                0x0D80..=0x0DFF | 0x111E0..=0x111FF => FontScripts::Sin as u32,

                0x1800..=0x18AF | 0x11660..=0x1167F => FontScripts::Mng as u32,

                0x13A0..=0x13FF | 0xAB70..=0xABBF => FontScripts::Chr as u32,

                0x0370..=0x03FF | 0x1F00..=0x1FFF => FontScripts::Gr as u32,

                _ => 0,
            }
        }

        mask
    }

    pub fn get_required_fonts(&self, mask: u32) -> Vec<(char, &'static str)> {
        //usa la máscara de scripts para buscar una posible fuente en el sistema
        let mut fonts = Vec::new();

        add_font_if_needed!(mask, FontScripts::Lat, 'A', "FontLAT", fonts);
        add_font_if_needed!(mask, FontScripts::Cjksc, '汉', "FontCJKSC", fonts);
        add_font_if_needed!(mask, FontScripts::Cjkjp, 'こ', "FontJP", fonts);
        add_font_if_needed!(mask, FontScripts::Cjkkr, '한', "FontKR", fonts);
        add_font_if_needed!(mask, FontScripts::Emoji, '😀', "FontEmoji", fonts);
        add_font_if_needed!(mask, FontScripts::Ar, 'ع', "FontAR", fonts);
        add_font_if_needed!(mask, FontScripts::He, 'א', "FontHE", fonts);
        add_font_if_needed!(mask, FontScripts::Th, 'อ', "FontTH", fonts);
        add_font_if_needed!(mask, FontScripts::Lo, 'ກ', "FontLO", fonts);
        add_font_if_needed!(mask, FontScripts::Dev, 'अ', "FontDEV", fonts);
        add_font_if_needed!(mask, FontScripts::Bn, 'অ', "FontBN", fonts);
        add_font_if_needed!(mask, FontScripts::Ta, 'அ', "FontTA", fonts);
        add_font_if_needed!(mask, FontScripts::Te, 'అ', "FontTE", fonts);
        add_font_if_needed!(mask, FontScripts::Gu, 'અ', "FontGU", fonts);
        add_font_if_needed!(mask, FontScripts::Ml, 'അ', "FontML", fonts);
        add_font_if_needed!(mask, FontScripts::Kn, 'ಅ', "FontKN", fonts);
        add_font_if_needed!(mask, FontScripts::My, 'က', "FontMY", fonts);
        add_font_if_needed!(mask, FontScripts::Km, 'ក', "FontKM", fonts);
        add_font_if_needed!(mask, FontScripts::Eth, 'ሀ', "FontETH", fonts);
        add_font_if_needed!(mask, FontScripts::Geo, 'ა', "FontGEO", fonts);
        add_font_if_needed!(mask, FontScripts::Arm, 'Ա', "FontARM", fonts);
        add_font_if_needed!(mask, FontScripts::Tib, 'ཀ', "FontTIB", fonts);
        add_font_if_needed!(mask, FontScripts::Sin, 'අ', "FontSIN", fonts);
        add_font_if_needed!(mask, FontScripts::Mng, 'ᠠ', "FontMNG", fonts);
        add_font_if_needed!(mask, FontScripts::Chr, 'Ꭰ', "FontCHR", fonts);
        add_font_if_needed!(mask, FontScripts::Cy, 'П', "FontCY", fonts);
        add_font_if_needed!(mask, FontScripts::Gr, 'α', "FontGR", fonts);
        add_font_if_needed!(mask, FontScripts::Pa, 'ਸ', "FontPA", fonts);

        fonts
    }
}

#[cfg(test)]
mod test {
    use crate::ui::fonts::fonts_manager::FontManager;
    use std::path::Path;

    fn test_text(text: &str) {
        let mut fm = FontManager::new();

        let counter = fm.pending_tasks.clone();
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        fm.process_file(text, Path::new("").into());
        counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        let font_defs = fm.build_font_definitions();

        assert!(
            !font_defs.families.is_empty(),
            "No se generaron familias después de procesar texto"
        );

        for (i, (family, names)) in font_defs.families.iter().enumerate() {
            eprintln!("{}. {:?}: {:?}", i, family, names);
        }

        let has_valid_names = font_defs
            .families
            .iter()
            .any(|(_, names)| !names.is_empty());

        assert!(
            has_valid_names,
            "Todas las familias tienen nombres vacíos para: '{text}'"
        );
    }

    #[test]
    fn test_jp() {
        let text = "こんにちは";
        test_text(text);
    }

    #[test]
    fn test_ko() {
        let text = "안녕하세요";
        test_text(text);
    }

    #[test]
    fn test_zh() {
        let text = "你好";
        test_text(text);
    }

    #[test]
    fn test_hi() {
        let text = "नमस्ते";
        test_text(text);
    }

    #[test]
    fn test_lao() {
        let text = "ສະບາຍດີ";
        test_text(text);
    }

    #[test]
    fn test_km() {
        let text = "សួស្តី";
        test_text(text);
    }

    #[test]
    fn test_my() {
        let text = "မင်္ဂလာပါ";
        test_text(text);
    }

    #[test]
    fn test_pa() {
        let text = "ਸਤ ਸ੍ਰੀ ਅਕਾਲ";
        test_text(text);
    }

    #[test]
    fn test_te() {
        let text = "హలో";
        test_text(text);
    }

    #[test]
    fn test_kn() {
        let text = "ಹಲೋ";
        test_text(text);
    }

    #[test]
    fn test_ml() {
        let text = "ഹലോ";
        test_text(text);
    }

    #[test]
    fn test_ta() {
        let text = "வணக்கம்";
        test_text(text);
    }

    #[test]
    fn test_kan() {
        let text = "ನಮಸ್ಕಾರ";
        test_text(text);
    }

    #[test]
    fn test_ar() {
        let text = "مرحبًا";
        test_text(text);
    }

    #[test]
    fn test_he() {
        let text = "שלום";
        test_text(text);
    }

    #[test]
    fn test_ru() {
        let text = "Привет";
        test_text(text);
    }

    #[test]
    fn test_el() {
        let text = "γειά";
        test_text(text);
    }

    #[test]
    fn test_th() {
        let text = "สวัสดี";
        test_text(text);
    }

    #[test]
    fn test_am() {
        let text = "ሰላም";
        test_text(text);
    }

    #[test]
    fn test_hy() {
        let text = "Բարեւ";
        test_text(text);
    }
}
