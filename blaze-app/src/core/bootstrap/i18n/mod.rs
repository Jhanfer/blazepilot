use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use toml::Value;
use tracing::info;

macro_rules! load_locale_str {
    ($locale:expr_2021) => {
        match $locale {
            "es" => include_str!("../../../ui/assets/i18n/es.toml"),
            "en" => include_str!("../../../ui/assets/i18n/en.toml"),
            "ru" => include_str!("../../../ui/assets/i18n/ru.toml"),
            "fr" => include_str!("../../../ui/assets/i18n/fr.toml"),
            "de" => include_str!("../../../ui/assets/i18n/de.toml"),
            "it" => include_str!("../../../ui/assets/i18n/it.toml"),
            _ => include_str!("../../../ui/assets/i18n/en.toml"),
        }
    };
}

#[derive(Clone, Debug, Default)]
pub struct I18n {
    translations: Arc<RwLock<HashMap<String, String>>>,
    current_locale: Arc<RwLock<String>>,
}

impl I18n {
    pub fn load(locale: &str) -> Self {
        let translations = Self::load_locale(locale);
        info!("Cargando idioma: {locale}");
        Self {
            translations: Arc::new(RwLock::new(translations)),
            current_locale: Arc::new(RwLock::new(locale.to_string())),
        }
    }

    pub fn t(&self, key: &str) -> Box<str> {
        self.translations
            .read()
            .get(key)
            .cloned()
            .unwrap_or(format!("?{key}?"))
            .into()
    }

    pub fn t_args(&self, key: &str, args: &[(&str, &str)]) -> Box<str> {
        let mut result = self.t(key);

        for (k, v) in args {
            result = result.replace(&format!("{{{k}}}"), v).into();
        }

        result
    }

    pub fn switch_locale(&self, locale: &str) {
        let new_translation = Self::load_locale(locale);
        *self.translations.write() = new_translation;
        *self.current_locale.write() = locale.to_string();
    }

    #[allow(unused)]
    pub fn current_locale(&self) -> String {
        self.current_locale.read().clone()
    }

    fn load_locale(locale: &str) -> HashMap<String, String> {
        let content = load_locale_str!(locale);
        flatten_toml(content)
    }
}

fn flatten_toml(content: &str) -> HashMap<String, String> {
    let value: Value = toml::from_str(content).unwrap_or(Value::Table(Default::default()));
    let mut map = HashMap::new();
    flatten_value("", &value, &mut map);
    map
}

fn flatten_value(prefix: &str, value: &Value, map: &mut HashMap<String, String>) {
    match value {
        Value::Table(table) => {
            for (k, v) in table {
                let new_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_value(&new_key, v, map);
            }
        }
        Value::String(s) => {
            map.insert(prefix.to_string(), s.clone());
        }
        other => {
            map.insert(prefix.to_string(), other.to_string());
        }
    }
}
