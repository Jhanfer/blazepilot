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





use std::{collections::{HashMap, HashSet}, ffi::OsString, fs, path::{Path, PathBuf}, sync::Arc};
use image::{DynamicImage, imageops};
use once_cell::sync::Lazy;
use resvg::usvg;
use tokio::sync::Semaphore;
use tracing::{warn, info};
use crate::core::{runtime::{bus_structs::UiEvent, event_bus::Dispatcher}, system::{fileopener_module::{AppAssociation, error::{OpenerError, OpenerResult}, platform::linux::{appassociation::AssociationManager, mimeapps::MimeApps, structs::{APPS_ICON_CACHE, AppsIconData, DesktopEntry, OpenStrategy, OpenerFileKind}}}, knowndirs::knowndirs_manager::KnownDirsManager}};




pub static LINUX_FILE_OPENER: Lazy<Arc<tokio::sync::Mutex<LinuxOpener>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(LinuxOpener::init()))
});

pub struct LinuxOpener {
    mimeapps: MimeApps,
    desktop_index: HashMap<String, Arc<Path>>,
    pub assoc_manager: AssociationManager,
    pub pending_default_app_name: Option<String>
}

impl LinuxOpener {
    fn init() -> Self {
        let mimeapps = MimeApps::load().unwrap_or_else(|e| {
            eprintln!("Error loading mimeapps: {}, usando configuración vacía", e);
            MimeApps::empty()
        });
        let desktop_index = Self::build_desktop_index();

        Self {
            desktop_index,
            mimeapps,
            assoc_manager: AssociationManager::new(),
            pending_default_app_name: None,
        }
    }


    fn build_desktop_index() -> HashMap<String, Arc<Path>> {
        let mut index: HashMap<String, Arc<Path>> = HashMap::new();
        let dirs = Self::xdg_app_dirs();

        for dir in dirs.iter().rev() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "desktop") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            index.insert(stem.to_string(), path.into());
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
        } else {
            let home = &KnownDirsManager::get().home;
            dirs.push(home.join(".local/share/applications"));
            dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
        }

        let xdg_data_dirs = std::env::var("XDG_DATA_DIRS") 
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".into());

        for dir in xdg_data_dirs.split(":") {
            dirs.push(PathBuf::from(dir).join("applications"));
        }

        dirs.into_iter().filter(|p| p.is_dir()).collect()
    }


    async fn detect_mime(path: &Path) -> String {
        if let Ok(output) = tokio::process::Command::new("xdg-mime")
            .args(["query", "filetype", &path.to_string_lossy()])
            .output()
            .await
        {
            let mime = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !mime.is_empty() && mime.contains('/') {
                return mime;
            }
        }

        Self::analyze_file(path).mime().to_string()
    }


    fn analyze_file(path: &Path) -> OpenerFileKind {
        use std::io::Read;

        let mut f = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return OpenerFileKind::Unknown,
        };

        let mut buf = [0u8; 16];
        if f.read_exact(&mut buf).is_err() {
            return OpenerFileKind::Unknown;
        }

        match &buf {
            // AppImage
            [0x7f, b'E', b'L', b'F', ..] if &buf[8..11] == [0x41, 0x49, 0x02] => OpenerFileKind::AppImage(2),
            [0x7f, b'E', b'L', b'F', ..] if &buf[8..11] == [0x41, 0x49, 0x01] => OpenerFileKind::AppImage(1),
            // ELF genérico
            [0x7f, b'E', b'L', b'F', ..] => OpenerFileKind::ElfExecutable,
            // Shebangs conocidos
            [b'#', b'!', b'/', b'b', b'i', b'n', b'/', b's', b'h', ..]   => OpenerFileKind::ShellScript,
            [b'#', b'!', b'/', b'b', b'i', b'n', b'/', b'b', b'a', b's', b'h', ..] => OpenerFileKind::ShellScript,
            // Cualquier otro shebang, se lee la línea completa
            [b'#', b'!', ..] => Self::classify_shebang(path),
            [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => OpenerFileKind::Png,
            [0xFF, 0xD8, 0xFF, ..] => OpenerFileKind::Jpeg,
            [b'%', b'P', b'D', b'F', ..] => OpenerFileKind::Pdf,
            [0x50, 0x4B, 0x03, 0x04, ..] => OpenerFileKind::Zip,
            _ => OpenerFileKind::Unknown,
        }
    }

    fn classify_shebang(path: &Path) -> OpenerFileKind {
        let Ok(content) = fs::read_to_string(path) else { return OpenerFileKind::OtherScript };
        let line = content.lines().next().unwrap_or("");

        if line.contains("python")      { OpenerFileKind::PythonScript }
        else if line.contains("ruby")   { OpenerFileKind::RubyScript }
        else if line.contains("perl")   { OpenerFileKind::PerlScript }
        else if line.contains("node") || line.contains("deno") { OpenerFileKind::NodeScript }
        else                            { OpenerFileKind::OtherScript }
    }


    fn parse_desktop_content(&self, content: &str) -> Option<DesktopEntry> {
        let mut name = None;
        let mut exec = None;
        let mut icon = None;
        let mut mimes = Vec::new();
        let mut is_private = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("Name=") && !line.starts_with("Name[") {
                name = Some(line[5..].trim().to_string());
            } else if line.starts_with("Exec=") {
                let e = line[5..].trim().to_string();
                is_private = {
                    let e_lower = e.to_lowercase();
                    e_lower.contains("--incognito")
                        || e_lower.contains("--private")
                        || e_lower.contains("-private")
                };
                exec = Some(e);
            } else if line.starts_with("Icon=") {
                icon = Some(line[5..].trim().to_string());
            } else if line.starts_with("MimeType=") {
                mimes = line[9..]
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect();
            }
        }

        Some(DesktopEntry { name: name?, exec: exec?, icon, mimes, is_private })
    }


    fn entry_to_association(&self, entry: DesktopEntry, id: &str) -> AppAssociation {
        AppAssociation { 
            id: id.to_owned(), 
            name: entry.name, 
            exec: entry.exec, 
            icon: entry.icon,
            is_private: entry.is_private,
            is_recommended: false,
        }
    }


    fn parse_desktop_file(&self, path: Arc<Path>) -> OpenerResult<AppAssociation> {
        let content = fs::read_to_string(&path)
            .map_err(|e| OpenerError::Io { path: path.clone(), source: e })?;

        let entry = self.parse_desktop_content(&content)
            .ok_or_else(|| OpenerError::DesktopParsedFaild { path: path.clone() })?;

        if entry.name.trim().is_empty() || entry.exec.trim().is_empty() {
            return Err(OpenerError::DesktopParsedFaild { path: path.to_owned() });
        }

        let id = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(self.entry_to_association(entry, &id))
    }

    fn app_from_desktop_id(&self, id: &str) -> OpenerResult<AppAssociation> {
        let stem = id.trim_end_matches(".desktop");
        let path = match self.desktop_index.get(stem) {
            Some(path) => path,
            None => return Err(OpenerError::DesktopParsedFaild { 
                path: PathBuf::from("").into()
            }),
        };


        let content = fs::read_to_string(path)
            .map_err(|e| OpenerError::Io { path: PathBuf::from("").into() , source: e })?;

        let entry = self.parse_desktop_content(&content)
            .ok_or_else(|| OpenerError::DesktopParsedFaild { path: path.clone() })?;
        
        if entry.name.trim().is_empty() || entry.exec.trim().is_empty() {
            return Err(OpenerError::DesktopParsedFaild { path: path.to_owned() });
        }

        Ok(self.entry_to_association(entry, id))
    }

    fn parse_desktop_file_with_mime(&self, path: Arc<Path>, target_mime: &str) -> OpenerResult<AppAssociation> {
        let content = fs::read_to_string(&path)
            .map_err(|e| OpenerError::Io { path: PathBuf::from("").into() , source: e })?;

        let entry = self.parse_desktop_content(&content)
            .ok_or_else(|| OpenerError::DesktopParsedFaild { path: path.to_owned() })?;

        if entry.name.trim().is_empty() || entry.exec.trim().is_empty() {
            return Err(OpenerError::DesktopParsedFaild { path: path.to_owned() });
        }

        if !entry.mimes.iter().any(|m| m == target_mime) {
            return Err(OpenerError::DesktopParsedFaild { path: path.to_owned() });
        }

        let id = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(self.entry_to_association(entry, &id))
    }




    async fn get_recommended_apps_for_mime(&self, mime: &str) -> Vec<AppAssociation> {
        let mut apps: Vec<AppAssociation> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        //mimeapps.list del usuario
        for desktop_id in self.mimeapps.apps_for_mime(mime) {
            match self.app_from_desktop_id(&desktop_id) {
                Ok(mut app) => {
                    if seen_ids.insert(app.id.clone()) {
                        app.is_recommended = true;
                        apps.push(app);
                    }
                },
                Err(e) => {
                    warn!("Ha ocurrido un error al parsear 'desktop_id': {}", e)
                },
            }
        }

        //default según xdg-mime
        if let Ok(output) = tokio::process::Command::new("xdg-mime")
            .args(["query", "default", mime])
            .output()
            .await
        {
            let desktop_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !desktop_id.is_empty() {
                match self.app_from_desktop_id(&desktop_id) {
                    Ok(mut app) => {
                        if seen_ids.insert(app.id.clone()) {
                            app.is_recommended = true;
                            apps.push(app);
                        }
                    },
                    Err(e) => {
                        warn!("Ha ocurrido un error al parsear 'desktop_id': {}", e)
                    },
                }
            }
        }


        //apps que declaran soporte para este MIME pero no están en mimeapps.list
        for (stem, path) in &self.desktop_index {
            if  seen_ids.contains(stem) || self.mimeapps.is_removed(mime, &stem) {
                continue;
            }

            match self.parse_desktop_file_with_mime(path.to_owned(), mime) {
                Ok(mut app) => {
                    if seen_ids.insert(app.id.clone()) {
                        app.is_recommended = false;
                        apps.push(app);
                    }
                },
                Err(e) => {
                    warn!("Ha ocurrido un error al parsear 'desktop_id': {}", e)
                },
            }
        }

        //Ordenar solo las no-recomendadas
        let first_non_recommended = apps.partition_point(|a| a.is_recommended);
        apps[first_non_recommended..].sort_by(|a, b| {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        });

        apps
    }



    async fn get_all_apps_for_open_with(&self, mime: &str) -> Vec<AppAssociation>  {
        let mut apps = self.get_recommended_apps_for_mime(mime).await;
        let mut seen_ids: HashSet<String> = apps.iter().map(|a| a.id.clone()).collect();

        let filtered: Vec<_> = self.desktop_index
            .iter()
            .filter(|(stem, _)| !seen_ids.contains(*stem))
            .collect();

        let mut rest: Vec<AppAssociation> = filtered
            .into_iter()
            .filter_map(|(stem, path)| {

                let app = match self.parse_desktop_file(path.to_owned()) {
                    Ok(mut app) => {
                        seen_ids.insert(stem.clone());
                        app.is_recommended = false;
                        app
                    },
                    Err(e) => {
                        warn!("Ha ocurrido un error al parsear 'desktop_id': {}", e);
                        return None;
                    },
                };

                Some(app)
            })
            .collect();

        rest.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_ascii_lowercase()));
        apps.extend(rest);
        
        apps
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

    fn build_arguments(exec: &str, file_path: Arc<Path>, app_name: &str) -> OpenerResult<Vec<OsString>> {
        let pre = exec.replace("%%", "\x00PERCENT\x00");
        let tokens = shell_words::split(&pre)
            .map_err(|_| OpenerError::ExecParseFailed { raw: exec.to_string() })?;
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

    pub fn launch_app_linux(&self, app: &AppAssociation, path: Arc<Path>) -> OpenerResult<()> {
        let args = Self::build_arguments(&app.exec, path.clone(), &app.name)?;

        if args.is_empty() {
            return Err(OpenerError::InvalidExec { desktop_id: app.id.clone() });
        }

        let cmd_str = args[0].to_string_lossy();

        if Self::is_blocked(&cmd_str) {
            return Err(OpenerError::BlockedExecutable { name: cmd_str.to_string() });
        }

        if !Self::is_safe_executable(&cmd_str) {
            return Err(OpenerError::ExecutableNotFound { name: cmd_str.to_string() });
        }

        std::process::Command::new(&args[0])
            .args(&args[1..])
            .spawn()
            .map_err(|e| OpenerError::Io { path: path, source: e })?;

        Ok(())
    }


    fn fallback_open(&self, path: Arc<Path>) {
        let _ = std::process::Command::new("xdg-open")
            .arg(path.to_path_buf())
            .spawn();
    }



    #[cfg(unix)]
    fn ensure_executable(&self, path: &Path) -> std::io::Result<()> {
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




    pub async fn open_file_with_linux(&mut self, path: Arc<Path>, sender: Dispatcher) {
        let mime = Self::detect_mime(&path).await;

        let available_apps = self.get_all_apps_for_open_with(&mime).await;
        info!("Apps disponibles: {}", available_apps.len());
        
        let show_all_apps = true;

        if available_apps.is_empty() {
            self.fallback_open(path);
        } else {
            self.show_selector_linux(path, mime, available_apps, show_all_apps, sender).await;
        }
    }





    fn launch_direct(&self, path: &Path) {
        if let Err(e) = self.ensure_executable(path) {
            warn!("No se han podido aplicar los permisos de ejecución: {}", e);
        }

        let res = std::process::Command::new(&path)
            .current_dir(path.parent().unwrap_or(Path::new("")))
            .env_clear()
            .envs(self.minimal_env())
            .spawn();

        match res {
            Ok(_) => info!("El ejecutable ha sido lanzado: {:?}", path),
            Err(e) => warn!("No se ha podido lanzar: {:?}: {}", path, e),
        }
    }
    

    fn launch_with_app_logged(&self, app: &AppAssociation, path: Arc<Path>) {
        match self.launch_app_linux(app, path.to_owned()) {
            Ok(_) => info!("Abierto '{}' con '{}'", path.display(), app.name),
            Err(e) => warn!("Fallo lanzando '{}': {e}", app.name),
        }
    }


    pub async fn open_file_linux(&mut self, path: Arc<Path>, sender: Dispatcher) {
        let mime = Self::detect_mime(&path).await;

        match self.resolve_open_strategy(&path, &mime).await {
            OpenStrategy::LaunchDirect => self.launch_direct(&path),
            OpenStrategy::LaunchWithApp(app) => self.launch_with_app_logged(&app, path),
            OpenStrategy::ShowSelector(apps) => self.show_selector_linux(path, mime, apps, false, sender).await,
            OpenStrategy::Fallback => self.fallback_open(path),
        }
    }



    async fn resolve_open_strategy(&self, path: &Path, mime: &str) -> OpenStrategy {
        let kind = Self::analyze_file(path);

        if kind.is_directly_executable() && Self::is_executable(path) {
            return OpenStrategy::LaunchDirect;
        }

        if let Some(app) = self.assoc_manager.get_associations(mime).cloned() {
            return OpenStrategy::LaunchWithApp(app.clone());
        }

        if let Some(id) = self.mimeapps.default_for_mime(mime) {
            if let Ok(app) = self.app_from_desktop_id(&id) {
                return OpenStrategy::LaunchWithApp(app);
            }
        }

        let apps = self.get_recommended_apps_for_mime(mime).await;
        if !apps.is_empty() {
            return OpenStrategy::ShowSelector(apps);
        }

        OpenStrategy::Fallback
    }



    async fn load_icons_async(&self, apps: &[AppAssociation]) -> OpenerResult<Vec<AppsIconData>> {
        const MAX_CONCURRENT: usize = 25;
        
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
        let mut handles = Vec::with_capacity(apps.len());


        for app in apps {
            let icon_name = app.icon
                .clone()
                .unwrap_or_else(|| "application-x-executable".to_string());

            let sem = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("Semaphore closed");
                Self::get_or_load_icon(&icon_name).await
            });
            handles.push(handle);
        }

        let mut results = Vec::with_capacity(apps.len());
        
        for handle in handles {
            match handle.await {
                Ok(icon_data) => {
                    match icon_data {
                        Ok(icon) => results.push(icon),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                    warn!("Error cargando icono: {}", e);
                    results.push(AppsIconData::None);
                }
            }
        }

        Ok(results)
    }

    async fn get_or_load_icon(icon_name: &str) -> OpenerResult<AppsIconData> {
        {
            let cache = APPS_ICON_CACHE.read().await;
            if let Some(cached) = cache.peek(icon_name) {
                return Ok((**cached).clone());
            }
        }

        let icon_name_clone = icon_name.to_string();

        let icon_data = tokio::task::spawn_blocking(move || {
            Self::load_single_icon_blocking(&icon_name_clone)
        })
        .await
        .map_err(|e| OpenerError::ThreadError(e))?;

        {
            let mut cache = APPS_ICON_CACHE.write().await;
            cache.put(icon_name.to_string(), Arc::new(icon_data.clone()));
        }

        Ok(icon_data)
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

    fn rasterize_svg(path: &PathBuf) -> OpenerResult<AppsIconData> {
        let svg_data = fs::read(path)
            .map_err(|e| OpenerError::Io { path: path.to_owned().into(), source: e })?;

        let rtree = usvg::Tree::from_data(&svg_data, &usvg::Options::default())
            .map_err(|e| OpenerError::SvgError(e))?;

        let original_size = rtree.size();
        let scale = (48.0 / original_size.width())
            .min(48.0 / original_size.height())
            .min(1.0);
        let target_w = (original_size.width() * scale).ceil() as u32;
        let target_h = (original_size.height() * scale).ceil() as u32;

        if target_w == 0 || target_h == 0 {
            return Err(OpenerError::TargetDimensionError);
        }

        let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)
            .ok_or_else(|| OpenerError::TargetDimensionError)?;

        let transform =  resvg::tiny_skia::Transform::from_scale(scale, scale);
        resvg::render(&rtree, transform, &mut pixmap.as_mut());

        let data = pixmap.data().to_vec();

        Ok(AppsIconData::Rgba {
            data, 
            width:target_w as f32, 
            height: target_h as f32, 
        })
    }



    fn img_handler(path: &Path) -> OpenerResult<DynamicImage> {
        use std::io::Read;
        use std::io::BufReader;

        let mut file = std::fs::File::open(path)
            .map_err(|e| OpenerError::Io { path: path.to_owned().into(), source: e })?;

        let mut header = [0u8; 16];

        let n = file.read(&mut header)
            .map_err(|e| OpenerError::Io { path: path.to_owned().into(), source: e })?;

        let real_format = image::guess_format(&header[..n])
            .map_err(|_| OpenerError::UnsuportedFormat)?;

        let file = std::fs::File::open(path)
            .map_err(|e| OpenerError::Io { path: path.to_owned().into(), source: e })?;

        let dynamic_image = image::ImageReader::with_format(BufReader::new(file), real_format)
            .decode()
            .map_err(|e| OpenerError::ImageError(e))?;

        Ok(dynamic_image)
    }



    fn load_single_icon_blocking(icon_name_or_path: &str) -> AppsIconData {
        let resolved = Self::resolve_icon_path_static(icon_name_or_path);
        let path = PathBuf::from(&resolved);

        if !path.is_absolute() || !path.exists() {
            return AppsIconData::Path(resolved);
        }

        if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("svg")) {
            if let Ok(rgba) = Self::rasterize_svg(&path) {
                return rgba;
            }
        }

        match Self::img_handler(&path) {
            Ok(dynamic_image) => {
                let thumb = imageops::thumbnail(&dynamic_image, 48, 48);

                AppsIconData::Rgba {
                    data: thumb.into_raw(),
                    width: 48.0,
                    height: 48.0,
                }
            },
            Err(_) => AppsIconData::None,
        }
    }



    async fn show_selector_linux(&mut self, path: Arc<Path>, mime: String, apps: Vec<AppAssociation>, show_all_apps: bool, sender: Dispatcher)  {
        let icon_data = match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            self.load_icons_async(&apps)
        )
        .await
        {
            Ok(Ok(img)) => img,

            Ok(Err(e)) => {
                eprintln!("Error cargando iconos: {}", e);
                vec![]
            },

            Err(e) => {
                eprintln!("TimeOput cargando los iconos: {}", e);
                vec![]
            },
        };

        sender.send(
            UiEvent::OpenWithSelector { 
                path: path.into(), 
                mime, 
                apps, 
                icon_data, 
                show_all_apps 
            }
        ).ok();
    }

}