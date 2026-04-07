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





use std::{collections::{HashMap, HashSet}, ffi::OsString, fs, num::NonZeroUsize, path::{Path, PathBuf}, sync::Arc};
use image::imageops;
use lru::LruCache;
use once_cell::sync::Lazy;
use resvg::usvg;
use tokio::sync::{Mutex, RwLock};
use tracing::{warn, info};
use uuid::Uuid;
use crate::{core::system::fileopener_module::{AppAssociation, platform::linux::{appassociation::AssociationManager, mimeapps::MimeApps}}, utils::channel_pool::{NotifyingSender, UiEvent}};


#[derive(Debug, Clone)]
pub enum AppsIconData {
    Rgba { data: Vec<u8>, width: f32, height: f32 },
    Path(String),
    None,
}

pub static APPS_ICON_CACHE: Lazy<RwLock<LruCache<String, Arc<AppsIconData>>>> = Lazy::new(|| {
    RwLock::new(LruCache::new(NonZeroUsize::new(80).unwrap()))
});

pub static LINUX_FILE_OPENER: Lazy<Arc<tokio::sync::Mutex<LinuxOpener>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(LinuxOpener::init()))
});

pub struct LinuxOpener {
    mimeapps: MimeApps,
    desktop_index: HashMap<String, PathBuf>,
    pub assoc_manager: AssociationManager,
    pub pending_path: Option<PathBuf>, 
    pub pending_mime: Option<String>,
    pub pending_default_app_name: Option<String>
}

impl LinuxOpener {
    fn init() -> Self {
        let mimeapps = MimeApps::load();
        let desktop_index = Self::build_desktop_index();
        Self {
            desktop_index,
            mimeapps,
            assoc_manager: AssociationManager::new(),
            pending_path: None,
            pending_mime: None,
            pending_default_app_name: None,
        }
    }


    fn build_desktop_index() -> HashMap<String, PathBuf> {
        let mut index: HashMap<String, PathBuf> = HashMap::new();
        let dirs = Self::xdg_app_dirs();

        for dir in dirs.iter().rev() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "desktop") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            index.insert(stem.to_string(), path);
                        }
                    }
                }
            }
        }
        
        index
    }


    fn xdg_app_dirs() -> Vec<PathBuf> {
        let mut dirs = vec![];

        if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(xdg_data_home).join("applications"));
        } else if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/applications"));
            dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
        }

        let xdg_data_dirs = std::env::var("XDG_DATA_DIRS") 
            .unwrap_or_else(|_| "usr/local/share:/usr/share".into());

        for dir in xdg_data_dirs.split(":") {
            dirs.push(PathBuf::from(dir).join("applications"));
        }

        dirs.into_iter().filter(|p| p.is_dir()).collect()
    }

    fn get_mime(&self, path: &PathBuf) -> Option<String> {
        mime_guess::from_path(path).first_raw().map(String::from)
    }


    fn parse_desktop_content(&self, content: &str) -> Option<(String, String, Option<String>, Vec<String>, Option<bool>)> {
        let mut name = None;
        let mut exec = None;
        let mut icon_raw = None;
        let mut mimes = Vec::new();
        let mut is_private= None;

        for line in content.lines() {
            if line.starts_with("Name=") && !line.starts_with("Name[") {
                name = Some(line[5..].to_string());
            } else if line.starts_with("Exec=") {
                exec = Some(line[5..].to_string());
                is_private = Some(exec.as_ref().map_or(false, |e| {
                    let e_lower = e.to_lowercase();
                    e_lower.contains("--incognito") 
                    || e_lower.contains("--private") 
                    || e_lower.contains("-private")
                }));
            } else if line.starts_with("Icon=") {
                icon_raw = Some(line[5..].to_string());
            } else if line.starts_with("MimeType=") {
                mimes = line[9..].split(';')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect();
            }
        }

        Some((name?, exec?, icon_raw, mimes, is_private))
    }


    fn parse_desktop_file(&self, path: &PathBuf) -> Option<AppAssociation> {
        let content = fs::read_to_string(path).ok()?;
        let (name, exec, icon, _mimes, is_private) = self.parse_desktop_content(&content)?;

        if name.trim().is_empty() || exec.trim().is_empty() {
            return None;
        }
        
        Some(
            AppAssociation { 
                id: path.file_stem()?.to_string_lossy().to_string(), 
                name: name, 
                exec: exec, 
                icon,
                is_private: is_private?,
                is_recommended: false,
            }
        )
    }



    fn app_from_desktop_id(&self, id: &str) -> Option<AppAssociation> {
        let stem = id.trim_end_matches(".desktop");
        let path = self.desktop_index.get(stem)?;
        
        let content = fs::read_to_string(path).ok()?;
        if let Some((name, exec, icon, _, is_private)) = self.parse_desktop_content(&content) {
            if name.trim().is_empty() || exec.trim().is_empty() {
                return None;
            }

            return Some(
                AppAssociation { 
                    id: id.to_string(), 
                    name, 
                    exec, 
                    icon, 
                    is_private: is_private?,
                    is_recommended: false,
                }
            );
        }
        None
    }


    fn parse_desktop_file_with_mime(&self, path: &PathBuf, target_mime: &str) -> Option<AppAssociation> {
        let content = fs::read_to_string(path).ok()?;
        let (name, exec, icon, mimes, is_private) = self.parse_desktop_content(&content)?;

        if name.trim().is_empty() || exec.trim().is_empty() {
            return None;
        }

        if mimes.iter().any(|m| m == target_mime) {
            Some(AppAssociation {
                id: path.file_stem()?.to_string_lossy().to_string(),
                name,
                exec,
                icon,
                is_private: is_private.unwrap_or(false),
                is_recommended: true,
            })
        } else {
            None
        }
    }

    async fn get_recommended_apps_for_mime(&self, mime: &str) -> Vec<AppAssociation>  {
        let mut apps: Vec<AppAssociation> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        for desktop_id in self.mimeapps.apps_for_mime(mime) {
            if let Some(mut app) = self.app_from_desktop_id(&desktop_id) {
                if seen_ids.insert(app.id.clone()) {
                    app.is_recommended = true;
                    apps.push(app);
                }
            }
        }

        if let Ok(output) = tokio::process::Command::new("xdg-mime")
            .args(["query", "default", mime])
            .output()
            .await {
                let desktop_file = String::from_utf8_lossy(&output.stdout).trim().to_string();

                if !desktop_file.is_empty() {
                    if let Some(mut app) = self.app_from_desktop_id(&desktop_file) {
                        info!("Default según xdg-mime: {} → is_private: {:?}", app.name, app.is_private);
                        if seen_ids.insert(app.id.clone()) {
                            app.is_recommended = true;
                            apps.push(app);
                        }
                    }
                }
            }

        for (stem, path) in &self.desktop_index {
            if seen_ids.contains(stem) || self.mimeapps.is_removed(mime, stem) {
                continue;
            }

            if let Some(app) = self.parse_desktop_file_with_mime(path, mime) {
                seen_ids.insert(stem.clone());

                if !app.is_private {
                    apps.push(app);
                } else {
                    apps.push(app);
                }

            }
        }
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        apps
    }



    async fn get_all_apps_for_open_with(&self, mime: &str) -> Vec<AppAssociation>  {
        let mut final_apps: Vec<AppAssociation> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        let recommended = self.get_recommended_apps_for_mime(mime).await;

        for mut app in recommended {
            app.is_recommended = true;
            if seen_ids.insert(app.id.clone()) {
                final_apps.push(app);
            }
        }

        for (stem, path) in &self.desktop_index {
            if seen_ids.contains(stem) {
                continue;
            }

            if let Some(mut app) = self.parse_desktop_file(path) {
                if seen_ids.insert(app.id.clone()) {
                    app.is_recommended = false;
                    final_apps.push(app);
                }
            }
        }

        let recommended_count = final_apps.iter().filter(|a| a.is_recommended).count();
        let (_, rest) = final_apps.split_at_mut(recommended_count);
        rest.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        final_apps
    }

    const BLOCKED_EXECUTABLES: &[&str] = & [
        //shells
        "bash", "zsh", "sh", "fish", "dash", "ksh", "csh", "tcsh",
        //Intérpretes
        "python", "python3", "python2", "ruby", "perl", "lua", "node",
        "php", "deno", "bun", 
        //Herramientas
        "sudo", "su", "pkexec", "doas", "xterm", "xdg-open", "env", "nohup",
        "env", "setsid",
    ];

    fn is_blocked(cmd: &str) -> bool {
        let bin_name = Path::new(cmd)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(cmd);
        Self::BLOCKED_EXECUTABLES.contains(&bin_name)
    }

    fn is_safe_executable(cmd: &str) -> bool {
        let path = Path::new(cmd);
        if path.is_absolute() {
            return path.exists() && Self::is_executable(&path);
        }
        if cmd.contains("/") {
            //rechazar rutas relativas, solo se permiten absolutas
            return  false;
        }
        which::which(cmd).is_ok()
    }

    #[cfg(unix)]
    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    fn build_arguments(exec: &str, file_path: &Path, app_name: &str) -> Result<Vec<OsString>, Box<dyn std::error::Error>> {
        let pre = exec.replace("%%", "\x00PERCENT\x00");
        let tokens = shell_words::split(&pre)?;
        let mut args: Vec<OsString> = Vec::new();

        for token in tokens {
            let token = token.replace("\x00PERCENT\x00", "%");

            match token.as_str() {
                "%f" | "%F" => args.push(file_path.as_os_str().to_owned()),
                "%u" | "%U" => args.push(format!("file://{}", file_path.display()).into()),
                "%c" | "%C" => args.push(OsString::from(app_name)),
                "%i" => {},
                other => {
                    if other.starts_with("%") {
                        warn!("Token desconoicido en exec: {}", other);
                        continue;
                    }
                    args.push(OsString::from(other))
                }
            }
        }

        Ok(args)
    }

    pub fn launch_app_linux(&self, app: &AppAssociation, path: &PathBuf) -> std::io::Result<()> {
        let args = Self::build_arguments(&app.exec, path, &app.name)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;

        if args.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Exec vacío"));
        }

        let cmd_str = args[0].to_string_lossy();

        if Self::is_blocked(&cmd_str) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Ejecutable bloqueado: {}", cmd_str)
            ));
        }

        if !Self::is_safe_executable(&cmd_str) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Ejecutable no encontrado o no válido: {}", cmd_str)
            ));
        }

        std::process::Command::new(&args[0])
            .args(&args[1..])
            .spawn()?;

        Ok(())
    }


    fn fallback_open(&self, path: &PathBuf) {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }


    pub async fn open_file_with_linux(&mut self, path: PathBuf, sender: NotifyingSender) {
        let mime = self.get_mime(&path).unwrap_or_else(|| "application/octet-stream".to_string());

        let available_apps = self.get_all_apps_for_open_with(&mime).await;
        info!("Apps disponibles: {}", available_apps.len());
        
        let show_all_apps = true;

        if available_apps.is_empty() {
            self.fallback_open(&path);
        } else {
            self.show_selector_linux(path, mime, available_apps, show_all_apps, sender).await;
        }
    }



    pub fn detect_appimage(&self, path: &PathBuf) -> Option<u8> {
        let mut f = fs::File::open(path).ok()?;
        let mut buf = [0u8; 16];

        use std::io::Read;
        f.read_exact(&mut buf).ok()?;

        if &buf[0..4] != b"\x7fELF" {
            return None;
        }

        match &buf[8..11] {
            [0x41, 0x49, 0x02] => Some(2),
            [0x41, 0x49, 0x01] => Some(1),
            _ => None,
        }
    }

    #[cfg(unix)]
    fn ensure_executable(&self, path: &PathBuf) -> std::io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let meta = fs::metadata(path)?;
        let mode = meta.permissions().mode();
        if mode & 0o111 == 0 {
            let mut perms = meta.permissions();
            perms.set_mode(mode | 0o111);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }


    fn minimal_env(&self) -> Vec<(&'static str, String)> {
        let mut env = vec![];
        for key in &["HOME", "USER", "LOGNAME", "SHELL", "PATH",
                    "XDG_RUNTIME_DIR", "DISPLAY", "WAYLAND_DISPLAY",
                    "DBUS_SESSION_BUS_ADDRESS", "LANG", "LC_ALL"] {
            if let Ok(val) = std::env::var(key) {
                env.push((*key, val));
            }
        }
        env
    }

    pub async fn open_file_linux(&mut self, path: PathBuf, sender: NotifyingSender) {
        info!("Abriendo con open file linux");

        if let Some(_app_image_type) = self.detect_appimage(&path) {
            if let Err(e) = self.ensure_executable(&path) {
                warn!("No se pudieron dar permisos de ejecución al AppImage: {}", e);
            }

            if let Ok(_) = std::process::Command::new(&path)
                .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
                .env_clear()
                .envs(self.minimal_env())
                .spawn() {
                    info!("AppImage lanzado correctamente con ejecución directa");
                    return;
                }

            warn!("No se pudo lanzar el AppImage con ningún método: {:?}", path);

            return ;
        }

        let mime = self.get_mime(&path).unwrap_or_else(|| "application/octet-stream".to_string());

        let app = self.assoc_manager.get_associations(&mime).cloned();
        if let Some(app) = app {
            if self.launch_app_linux(&app, &path).is_ok() {
                return;
            }
        }

        if let Some(default_id) = self.mimeapps.default_for_mime(&mime) {
            if let Some(app) = self.app_from_desktop_id(&default_id) {
                if self.launch_app_linux(&app, &path).is_ok() { return; }
            }
        }

        let available_apps = self.get_recommended_apps_for_mime(&mime).await;

        info!("Apps disponibles: {}", available_apps.len());

        let show_all_apps = false;

        if available_apps.is_empty() {
            self.fallback_open(&path);
        } else {
            self.show_selector_linux(path, mime, available_apps, show_all_apps, sender).await;
        }
    }








    async fn load_icons_async(&self, apps: &[AppAssociation]) -> Vec<AppsIconData> {
        let mut results = Vec::with_capacity(apps.len());
        
        for chunk in apps.chunks(25) {
            let mut handles = Vec::with_capacity(chunk.len());
            
            for app in chunk {
                let icon_name = app.icon
                    .clone()
                    .unwrap_or_else(|| "application-x-executable".to_string());

                let handle = tokio::spawn(async move {
                    Self::get_or_load_icon(&icon_name).await
                });
            
                handles.push(handle);
            }

            for handle in handles {
                match handle.await {
                    Ok(icon_data) => {
                        results.push(icon_data);
                    }
                    Err(e) => {
                        warn!("Error cargando icono: {}", e);
                        results.push(AppsIconData::None);
                    }
                }
            }
        }

        results
    }

    async fn get_or_load_icon(icon_name: &str) -> AppsIconData {
        {
            let cache = APPS_ICON_CACHE.read().await;
            if let Some(cached) = cache.peek(icon_name) {
                return (**cached).clone();
            }
        }

        let icon_name_clone = icon_name.to_string();

        let icon_data = tokio::task::spawn(async move {
            Self::load_single_icon_blocking(&icon_name_clone)
        })
        .await
        .unwrap_or_else(|e| {
            warn!("Error en spawn_blocking para icono: {}", e);
            AppsIconData::None
        });

        {
            let mut cache = APPS_ICON_CACHE.write().await;
            cache.put(icon_name.to_string(), Arc::new(icon_data.clone()));
        }

        icon_data
    } 


    #[cfg(target_os = "linux")]
    fn resolve_icon_path_static(icon_name_or_path: &str) -> String {
        use freedesktop_icons::lookup;
        let path = PathBuf::from(icon_name_or_path);
        if path.is_absolute() && path.exists() {
            return icon_name_or_path.to_string();
        }
        lookup(icon_name_or_path).with_size(48).find()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| icon_name_or_path.to_string())
    }

    fn rasterize_svg(path: &PathBuf) -> Option<AppsIconData> {
        let svg_data = fs::read(path).ok()?;
        let rtree = usvg::Tree::from_data(&svg_data, &usvg::Options::default()).ok()?;

        let original_size = rtree.size();
        let scale = (15.0 / original_size.width())
            .min(15.0 / original_size.height())
            .min(1.0);
        let target_w = (original_size.width() * scale).ceil() as u32;
        let target_h = (original_size.height() * scale).ceil() as u32;

        if target_w == 0 || target_h == 0 {
            return None;
        }

        let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)?;
        let transform =  resvg::tiny_skia::Transform::from_scale(scale, scale);
        resvg::render(&rtree, transform, &mut pixmap.as_mut());

        let data = pixmap.data().to_vec();

        Some(AppsIconData::Rgba {
            data, 
            width:target_w as f32, 
            height: target_h as f32, 
        })
    }

    fn load_single_icon_blocking(icon_name_or_path: &str) -> AppsIconData {
        let resolved = Self::resolve_icon_path_static(icon_name_or_path);

        let path = PathBuf::from(&resolved);
        if !path.is_absolute() || !path.exists() {
            return AppsIconData::Path(resolved);
        }

        if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("svg")) {
            if let Some(rgba) = Self::rasterize_svg(&path) {
                return rgba;
            }
        }

        match image::open(&path) {
            Ok(img) => {
                let thumb = imageops::thumbnail(&img, 15, 15);
                let (w, h) = (thumb.width(), thumb.height());

                return AppsIconData::Rgba {
                    data: thumb.into_raw(),
                    width: w as f32,
                    height: h as f32,
                };
            },
            Err(_) => AppsIconData::None,
        }
    }
    


    async fn show_selector_linux(&mut self, path: PathBuf, mime: String, apps: Vec<AppAssociation>, show_all_apps: bool, sender: NotifyingSender)  {
        self.pending_path = Some(path.clone());
        self.pending_mime = Some(mime.clone());

        let icon_data = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            self.load_icons_async(&apps)
        ).await.unwrap_or_else(|_| vec![AppsIconData::None; apps.len()]);

        sender.send_ui_event(
            UiEvent::OpenWithSelector { path, mime, apps, icon_data, show_all_apps }
        ).ok();
    }

}