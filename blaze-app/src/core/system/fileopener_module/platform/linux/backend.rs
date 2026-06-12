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

use crate::core::system::{
    fileopener_module::{
        error::{OpenerError, OpenerResult},
        platform::{
            linux::structs::OpenerFileKind,
            opener_trait::{AppIconSource, AppInfo, FileOpener},
        },
    },
    knowndirs::knowndirs_manager::KnownDirsManager,
};
use linicon::IconPath;
use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct DesktopApp {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub mimes: HashSet<String>,
}

impl From<&DesktopApp> for AppInfo {
    fn from(app: &DesktopApp) -> Self {
        AppInfo {
            id: app.id.clone(),
            name: app.name.clone(),
            icon: AppIconSource::None,
            is_default: false,
            is_recommended: false,
        }
    }
}

impl DesktopApp {
    fn to_app_info(&self, is_default: bool) -> AppInfo {
        AppInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            icon: AppIconSource::Unresolved(self.icon.clone().unwrap_or_default()),
            is_default,
            is_recommended: false,
        }
    }

    pub fn resolve_icon_path(icon_name_or_path: &str) -> AppIconSource {
        let p = Path::new(icon_name_or_path);

        if p.is_absolute() && p.exists() {
            return AppIconSource::Path(p.to_path_buf());
        }

        let icon_name = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(icon_name_or_path);

        if let Some(resolved) = Self::lookup_freedesktop(icon_name) {
            return AppIconSource::Path(resolved);
        }

        if let Some(resolved) = Self::search_common_locations(icon_name) {
            return AppIconSource::Path(resolved);
        }

        warn!("No se han encontrado iconos para: {icon_name}");
        AppIconSource::None
    }

    fn lookup_freedesktop(icon_name: &str) -> Option<PathBuf> {
        use linicon::lookup_icon;

        lookup_icon(icon_name)
            .next()
            .and_then(|result| result.ok())
            .map(|icon: IconPath| icon.path)
    }

    fn search_common_locations(icon_name: &str) -> Option<PathBuf> {
        let mut icon_map = HashMap::new();
        let home = KnownDirsManager::get().home.clone();

        let locations = [
            "/usr/share/pixmaps",
            "/usr/share/icons/hicolor/scalable/apps",
            "/usr/share/icons/hicolor/48x48/apps",
            "/usr/share/icons/hicolor/64x64/apps",
            "/usr/share/icons/hicolor/128x128/apps",
            "/usr/share/icons/hicolor/32x32/apps",
            "/usr/local/share/icons",
            &format!("{}/.local/share/icons", home.display()),
        ];

        let extensions = ["svg", "png", "xmp"];

        for location in &locations {
            let dir = Path::new(location);
            if !dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if !extensions.contains(&ext.to_lowercase().as_str()) {
                            continue;
                        }
                    }

                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let key = stem.to_lowercase().replace(['-', '_', '.'], "");
                        icon_map.entry(key).or_insert(path);
                    }
                }
            }
        }

        let search_target = icon_name.to_lowercase().replace(['-', '_', '.'], "");

        icon_map.iter().find_map(|(candidate, path)| {
            if candidate.contains(&search_target) || search_target.contains(candidate) {
                Some(path.clone())
            } else {
                None
            }
        })
    }
}

pub struct LinuxOpener {
    apps: HashMap<String, Arc<DesktopApp>>,
    mime_defaults: HashMap<String, String>, // Mime - AppID
    mime_associations: HashMap<String, Vec<String>>, // Mime - [apps recomendadas]
    mime_removed: HashMap<String, Vec<String>>,
    mime_cache: RwLock<HashMap<PathBuf, String>>,
}

impl FileOpener for LinuxOpener {
    fn get_mime(&self, path: Arc<Path>) -> String {
        self.resolve_mime(path)
    }

    fn open_file(&self, path: Arc<Path>) -> OpenerResult<()> {
        let file_kind = OpenerFileKind::detect(&path);

        if file_kind.is_directly_executable() {
            return self.launch_executable_direct(path);
        }

        let mime = self.resolve_mime(path.clone());

        if let Some(app) = self.resolve_default_app(&mime) {
            return self.launch_app(&app, path);
        }

        Command::new("xdg-open")
            .arg(path.as_ref())
            .spawn()
            .map_err(|e| OpenerError::Io {
                path,
                msg: "'xdg-open' ha fallado".into(),
                source: e,
            })?;

        Ok(())
    }

    fn open_with(&self, app_id: &str, path: Arc<Path>) -> OpenerResult<()> {
        let app = self.apps.get(app_id).ok_or(OpenerError::NoAppFound)?;

        self.launch_app(app, path)
    }

    fn get_default_app(&self, path: Arc<Path>) -> OpenerResult<Option<AppInfo>> {
        let mime = self.resolve_mime(path);
        Ok(self.resolve_default_app(&mime).map(|app| {
            let mut info: AppInfo = (app.as_ref()).into();
            info.is_default = true;
            info
        }))
    }

    fn get_available_apps(&self, path: Arc<Path>) -> OpenerResult<Vec<AppInfo>> {
        let mime = self.resolve_mime(path);
        let default_app = self.resolve_default_app(&mime);

        let mut result: Vec<AppInfo> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        if let Some(app) = default_app {
            let mut info = app.to_app_info(true);
            info.is_default = true;
            info.is_recommended = true;
            seen.insert(app.id.clone());
            result.push(info);
        }

        if let Some(ids) = self.mime_associations.get(&mime) {
            for id in ids {
                if seen.contains(id) {
                    continue;
                }
                if let Some(app) = self.apps.get(id) {
                    let mut info = AppInfo::from(app.as_ref());
                    info.is_recommended = true;
                    seen.insert(id.clone());
                    result.push(info);
                }
            }
        }

        let mut others: Vec<AppInfo> = self
            .apps
            .values()
            .filter(|app| {
                !seen.contains(&app.id)
                    && app.mimes.contains(&mime)
                    && !self.is_removed(&mime, &app.id)
            })
            .map(|app| {
                let mut info = AppInfo::from(app.as_ref());
                info.is_recommended = true;
                info
            })
            .collect();

        others.sort_by_key(|a| a.name.to_lowercase());
        result.extend(others);

        Ok(result)
    }

    fn get_all_apps(&self, path: Arc<Path>) -> OpenerResult<Vec<AppInfo>> {
        // Primero las recomendadas
        let recommended_apps = self.get_available_apps(path)?;
        let recommended_ids: HashSet<_> = recommended_apps.iter().map(|r| &r.id).collect();

        let others: Vec<AppInfo> = self
            .apps
            .values()
            .filter(|app| !recommended_ids.contains(&app.id))
            .map(|app| app.to_app_info(false))
            .collect();

        let mut all_apps = recommended_apps;
        all_apps.extend(others);

        Ok(all_apps)
    }

    fn set_system_default(&self, path: Arc<Path>, app_id: &str) -> OpenerResult<()> {
        let mime = self.resolve_mime(path);

        let status = Command::new("xdg-mime")
            .args(["default", app_id, &mime])
            .status()
            .map_err(|_| OpenerError::XDGMimeError)?;

        if !status.success() {
            return Err(OpenerError::XDGMimeError);
        }

        Ok(())
    }
}

type HmSVS = HashMap<String, Vec<String>>;
type HmSS = HashMap<String, String>;

impl LinuxOpener {
    pub fn init() -> Self {
        let apps = Self::load_all_apps();
        let (mime_defaults, mime_associations, mime_removed) = Self::load_mimeapps(&apps);
        Self {
            apps,
            mime_defaults,
            mime_associations,
            mime_removed,
            mime_cache: RwLock::new(HashMap::new()),
        }
    }

    fn is_removed(&self, mime: &str, app_id: &str) -> bool {
        self.mime_removed
            .get(mime)
            .is_some_and(|ids| ids.iter().any(|id| id == app_id))
    }

    fn load_mimeapps(apps: &HashMap<String, Arc<DesktopApp>>) -> (HmSS, HmSVS, HmSVS) {
        let mut defaults: HmSS = HashMap::new();
        let mut associations: HmSVS = HashMap::new();
        let mut removed: HmSVS = HashMap::new();

        for path in Self::mimeapps_list_paths() {
            Self::parse_mimeapps_file(&path, &mut defaults, &mut associations, &mut removed, apps);
        }

        (defaults, associations, removed)
    }

    fn mimeapps_list_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        let xdg_data_dirs =
            std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
        for dir in xdg_data_dirs.split(':') {
            paths.push(PathBuf::from(dir).join("applications/mimeapps.list"));
        }

        let xdg_config_dirs =
            std::env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".into());
        for dir in xdg_config_dirs.split(':') {
            paths.push(PathBuf::from(dir).join("mimeapps.list"));
        }

        let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config")
            });

        paths.push(xdg_config_home.join("mimeapps.list"));

        paths.into_iter().filter(|p| p.exists()).collect()
    }

    fn parse_mimeapps_file(
        path: &Path,
        defaults: &mut HmSS,
        associations: &mut HmSVS,
        removed: &mut HmSVS,
        apps: &HashMap<String, Arc<DesktopApp>>,
    ) {
        let Ok(content) = std::fs::read_to_string(path) else {
            return;
        };

        #[derive(PartialEq)]
        enum Section {
            Default,
            Added,
            Removed,
            Other,
        }
        let mut section = Section::Other;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            match line {
                "[Default Applications]" => {
                    section = Section::Default;
                    continue;
                }
                "[Added Associations]" => {
                    section = Section::Added;
                    continue;
                }
                "[Removed Associations]" => {
                    section = Section::Removed;
                    continue;
                }
                s if s.starts_with("[") => {
                    section = Section::Other;
                    continue;
                }
                _ => {}
            }

            if section == Section::Other {
                continue;
            }

            let Some((mime, value)) = line.split_once("=") else {
                continue;
            };
            let mime = mime.trim().to_string();

            let valid_ids: Vec<String> = value
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .filter(|id| apps.contains_key(*id))
                .map(|s| s.to_string())
                .collect();

            if valid_ids.is_empty() {
                continue;
            }

            match section {
                Section::Default => {
                    defaults.insert(mime, valid_ids[0].clone());
                }
                Section::Added => {
                    associations.insert(mime, valid_ids);
                }
                Section::Removed => {
                    removed.entry(mime).or_default().extend(valid_ids);
                }
                Section::Other => {}
            }
        }
    }

    fn load_all_apps() -> HashMap<String, Arc<DesktopApp>> {
        let mut apps = HashMap::new();

        for dir in Self::app_dirs() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "desktop") {
                    if let Some(app) = Self::parse_desktop_file(&path) {
                        apps.insert(app.id.clone(), Arc::new(app));
                    }
                }
            }
        }

        apps
    }

    fn app_dirs() -> Vec<PathBuf> {
        let mut dirs = vec![];

        let home = KnownDirsManager::get().home.to_owned();

        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));

        let xdf_data_dirs =
            std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());

        for d in xdf_data_dirs.split(":") {
            dirs.push(PathBuf::from(d).join("applications"));
            dirs.push(PathBuf::from(d).join("flatpak/exports/share/applications"));
        }

        dirs.into_iter().filter(|p| p.is_dir()).collect()
    }

    fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
        let content = std::fs::read_to_string(path).ok()?;
        let id = path.file_name()?.to_str()?.to_string();

        let (name, exec, icon, mimes) = Self::extract_desktop_fields(&content)?;

        Some(DesktopApp {
            id,
            name,
            exec,
            icon,
            mimes: mimes.into_iter().collect(),
        })
    }

    fn extract_desktop_fields(
        content: &str,
    ) -> Option<(String, String, Option<String>, Vec<String>)> {
        let mut in_desktop_entry = false;
        let mut name: Option<String> = None;
        let mut localized_names: HashMap<String, String> = HashMap::new();
        let mut exec: Option<String> = None;
        let mut icon: Option<String> = None;
        let mut mimes: Vec<String> = Vec::new();
        let mut no_display = false;

        let sys_lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .unwrap_or_default();
        let lang_code = sys_lang.split('.').next().unwrap_or("");
        let lang_prefix = lang_code.split('_').next().unwrap_or("");

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some(v) = line.strip_prefix("NoDisplay=") {
                no_display = v.trim().eq_ignore_ascii_case("true");
            }
            if let Some(v) = line.strip_prefix("Name=") {
                name = Some(v.trim().to_string());
            } else if line.starts_with("Name[") {
                if let Some(rest) = line.strip_prefix("Name[") {
                    if let Some((lang, val)) = rest.split_once(']') {
                        let val = val.strip_prefix('=').unwrap_or(val).trim();
                        localized_names.insert(lang.to_string(), val.trim().to_string());
                    }
                }
            } else if let Some(v) = line.strip_prefix("Exec=") {
                exec = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("Icon=") {
                icon = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("MimeType=") {
                mimes = v
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }
        }

        if no_display {
            return None;
        }

        let final_name = localized_names
            .get(lang_code)
            .or_else(|| localized_names.get(lang_prefix))
            .or(name.as_ref())
            .cloned()?;

        Some((final_name, exec?, icon, mimes))
    }

    pub fn resolve_mime(&self, path: Arc<Path>) -> String {
        {
            let cache = self.mime_cache.read();
            let path = path.to_path_buf();
            if let Some(mime) = cache.get(&path) {
                return mime.clone();
            }
        }

        let mime = Command::new("xdg-mime")
            .args(["query", "filetype", &path.to_string_lossy()])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "application/octet-stream".into());

        self.mime_cache
            .write()
            .insert(path.to_path_buf(), mime.clone());
        mime
    }

    fn resolve_default_app(&self, mime: &str) -> Option<Arc<DesktopApp>> {
        let id = self.mime_defaults.get(mime)?;
        self.apps.get(id).cloned()
    }

    fn minimal_env(&self) -> Vec<(&'static str, String)> {
        let mut env = vec![];
        for key in &[
            "HOME",
            "USER",
            "LOGNAME",
            "SHELL",
            "PATH",
            "XDG_RUNTIME_DIR",
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "DBUS_SESSION_BUS_ADDRESS",
            "LANG",
            "LC_ALL",
        ] {
            if let Ok(val) = std::env::var(key) {
                env.push((*key, val));
            }
        }
        env
    }

    fn launch_executable_direct(&self, path: Arc<Path>) -> OpenerResult<()> {
        debug!("Ingrasa a direct");
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&path).map_err(|e| OpenerError::Io {
            msg: "No se puede leer metadata".into(),
            path: path.clone(),
            source: e,
        })?;

        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            warn!("El archivo no tiene permisos de ejecución. Intentando dar permisos.");
        }

        if path.extension().and_then(|e| e.to_str()) == Some("AppImage")
            || path.extension().and_then(|e| e.to_str()) == Some("appimage")
        {
            match std::process::Command::new("chmod")
                .args(["+x", &path.to_string_lossy()])
                .status()
            {
                Ok(e) if e.success() => {
                    debug!("Se han dado permisos.");
                }
                Ok(e) => {
                    debug!("Ha salido con: {:?}", e.code());
                }
                Err(e) => {
                    return Err(OpenerError::Io {
                        path,
                        msg: "Ha ocurrido un error al dar permisos".into(),
                        source: e,
                    });
                }
            }
        }

        let res = Command::new(path.to_path_buf())
            .current_dir(path.parent().unwrap_or(Path::new("")))
            .env_clear()
            .envs(self.minimal_env())
            .spawn();

        match res {
            Ok(_) => {
                debug!("El ejecutable ha sido lanzado: {:?}", path);
                Ok(())
            }
            Err(e) => Err(OpenerError::Io {
                path,
                msg: "No se ha podido lanzar".into(),
                source: e,
            }),
        }
    }

    fn launch_app(&self, app: &DesktopApp, path: Arc<Path>) -> OpenerResult<()> {
        if which::which("gtk-launch").is_ok() {
            Command::new("gtk-launch")
                .args([&app.id, &path.to_string_lossy().to_string()])
                .spawn()
                .map_err(|e| OpenerError::Io {
                    path,
                    msg: "'gtk-launch' ha fallado".into(),
                    source: e,
                })?;
            return Ok(());
        }

        let args = Self::build_exec_args(&app.exec, path.clone())?;

        if args.is_empty() {
            return Err(OpenerError::ArgsEmpty);
        }

        let bin = Path::new(&args[0])
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        const BLOCKED: &[&str] = &[
            "bash", "sh", "zsh", "fish", "dash", "ksh", "csh", "tcsh", "sudo", "su", "pkexec",
            "doas", "python", "python3", "python2", "ruby", "perl", "lua", "node", "php", "deno",
            "bun", "xterm", "xdg-open", "nohup", "setsid", "rm", "dd",
        ];

        if BLOCKED.contains(&bin) {
            return Err(OpenerError::BlockedExecutable { name: bin.into() });
        }

        Command::new(&args[0])
            .args(&args[1..])
            .spawn()
            .map_err(|e| OpenerError::Io {
                path,
                msg: "No se ha podido lanzar la app".into(),
                source: e,
            })?;

        Ok(())
    }

    fn build_exec_args(exec: &str, path: Arc<Path>) -> OpenerResult<Vec<String>> {
        let path_str = path.to_string_lossy().to_string();
        let url_str = format!("file://{path_str}");
        let mut has_file_arg = false;

        let tokens =
            shell_words::split(exec).map_err(|e| OpenerError::ExecParseFailed { error: e })?;

        let mut args: Vec<String> = tokens
            .into_iter()
            .filter_map(|tok| match tok.as_str() {
                "%f" | "%F" => {
                    has_file_arg = true;
                    Some(path_str.clone())
                }
                "%u" | "%U" => {
                    has_file_arg = true;
                    Some(url_str.clone())
                }
                "%d" | "%D" | "%n" | "%N" | "%v" | "%m" | "%i" | "%c" | "%k" => None,
                other => Some(other.replace("%%", "%")),
            })
            .collect();

        if !has_file_arg && !args.is_empty() {
            args.push(path_str);
        }

        Ok(args)
    }
}
