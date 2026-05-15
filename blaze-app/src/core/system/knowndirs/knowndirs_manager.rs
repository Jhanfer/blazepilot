use directories::{BaseDirs, ProjectDirs, UserDirs};
use tracing::warn;
use std::fs;
use std::path::{PathBuf, Path};
use std::sync::{Arc, OnceLock};

pub struct KnownDirsManager {
    pub home: Arc<Path>,
    pub app_config: Arc<Path>,
    pub app_cache: Arc<Path>,
    pub app_data: Arc<Path>,
    pub sys_config: Arc<Path>,
    pub sys_cache: Arc<Path>,
    pub data_local: Arc<Path>,

    pub desktop: Option<Arc<Path>>,
    pub downloads: Option<Arc<Path>>,
    pub documents: Option<Arc<Path>>,
    pub pictures: Option<Arc<Path>>,
    pub videos: Option<Arc<Path>>,
    pub music: Option<Arc<Path>>,
    pub public: Option<Arc<Path>>,
}

static INSTANCE: OnceLock<KnownDirsManager> = OnceLock::new();

impl KnownDirsManager {
    pub fn init() {
        let instance = INSTANCE.get_or_init(|| {

            let user = UserDirs::new();

            let home = user.as_ref()
                .map(|u| u.home_dir().to_path_buf())
                .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
                .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
                .unwrap_or_else(|| {
                    if cfg!(target_os = "windows") {
                        PathBuf::from("C:\\")
                    } else {
                        PathBuf::from("/")
                    }
                });

            let (app_config, app_cache, app_data, data_local) = match ProjectDirs::from("com", "blazepilot", "blazepilotapp") {
                Some(proj) => (
                    proj.config_dir().to_path_buf(),
                    proj.cache_dir().to_path_buf(),
                    proj.data_dir().to_path_buf(),
                    proj.data_local_dir().to_path_buf(),
                ),
                None => {
                    if cfg!(target_os = "windows") {
                        let roaming = home.join("AppData").join("Roaming").join("blazepilot");

                        let local = home.join("AppData").join("Local").join("blazepilot");
                        (
                            roaming.clone(),
                            local.join("cache"),
                            roaming.clone(),
                            local,
                        )
                    } else if cfg!(target_os = "macos") {
                        let app_support = home.join("Library/Application Support/blazepilot");
                        (
                            app_support.clone(),
                            home.join("Library/Caches/blazepilot"),
                            app_support.clone(),
                            app_support,
                        )
                    } else {
                        (
                            home.join(".config/blazepilot"),
                            home.join(".cache/blazepilot"),
                            home.join(".local/share/blazepilot"),
                            home.join(".local/share/blazepilot"),
                        )
                    }
                }
            };

            let (sys_config, sys_cache) = match BaseDirs::new() {
                Some(base) => (
                    base.config_dir().to_path_buf(),
                    base.cache_dir().to_path_buf(),
                ),
                None => {
                    if cfg!(target_os = "windows") {
                        (
                            PathBuf::from("C:\\ProgramData"),
                            std::env::temp_dir(),
                        )
                    } else if cfg!(target_os = "macos") {
                        (
                            PathBuf::from("/Library/Preferences"),
                            PathBuf::from("/Library/Caches"),
                        )
                    } else {
                        (
                            home.join(".config"),
                            home.join(".cache"),
                        )
                    }
                },
            };

            let opt = |f: Option<&Path>| f.map(|p| p.into());

            let user_ref = user.as_ref();

            KnownDirsManager {
                home: home.into(),
                app_config: app_config.into(),
                app_cache: app_cache.into(),
                app_data: app_data.into(),
                sys_config: sys_config.into(),
                sys_cache: sys_cache.into(),
                data_local: data_local.into(),

                desktop: user_ref.and_then(|u| opt(u.desktop_dir())),
                downloads: user_ref.and_then(|u| opt(u.download_dir())),
                documents: user_ref.and_then(|u| opt(u.document_dir())),
                pictures: user_ref.and_then(|u| opt(u.picture_dir())),
                videos: user_ref.and_then(|u| opt(u.video_dir())),
                music: user_ref.and_then(|u| opt(u.audio_dir())),
                public: user_ref.and_then(|u| opt(u.public_dir())),
            }
        });

        //validar directorios críticos
        instance.validate_critical_dirs();
    }

    fn validate_critical_dirs(&self) {
        for dir in [&self.app_config, &self.app_cache, &self.app_data] {
            if !dir.exists() {
                #[cfg(debug_assertions)]
                warn!("Directorio crítico no existe, creando: {:?}", dir);

                if let Err(e) = fs::create_dir_all(dir) {
                    warn!("No se ha podido crear directorio crítico {:?}: {}", dir, e);
                }
            }
        }
    }

    #[inline]
    pub fn get() -> &'static KnownDirsManager {
        INSTANCE.get().expect("KnownDirsManager::init() no fue llamado")
    }

    pub fn sidebar_dirs(&'static self) -> Vec<(&'static str, &'static Arc<Path>)> {
        let mut dirs = vec![("Home", &self.home)];

        macro_rules! push_opt {
            ($label:expr, $field:expr) => {
                if let Some(ref p) = $field {
                    dirs.push(($label, p));
                }
            };
        }

        push_opt!("Desktop",   self.desktop);
        push_opt!("Downloads", self.downloads);
        push_opt!("Documents", self.documents);
        push_opt!("Pictures",  self.pictures);
        push_opt!("Videos",    self.videos);
        push_opt!("Music",     self.music);
        push_opt!("Public",    self.public);

        dirs
    } 
}