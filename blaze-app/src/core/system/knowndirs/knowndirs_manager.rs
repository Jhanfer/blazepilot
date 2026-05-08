use directories::{BaseDirs, ProjectDirs, UserDirs};
use tracing::warn;
use std::fs;
use std::path::{PathBuf, Path};
use std::sync::OnceLock;

pub struct KnownDirsManager {
    pub home: PathBuf,
    pub app_config: PathBuf,
    pub app_cache: PathBuf,
    pub app_data: PathBuf,
    pub sys_config: PathBuf,
    pub sys_cache: PathBuf,
    pub data_local: PathBuf,

    pub desktop: Option<PathBuf>,
    pub downloads: Option<PathBuf>,
    pub documents: Option<PathBuf>,
    pub pictures: Option<PathBuf>,
    pub videos: Option<PathBuf>,
    pub music: Option<PathBuf>,
    pub public: Option<PathBuf>,
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

            let opt = |f: Option<&Path>| f.map(PathBuf::from);

            KnownDirsManager {
                home,
                app_config,
                app_cache,
                app_data,
                sys_config,
                sys_cache,
                data_local,

                desktop: user.as_ref().and_then(|u| opt(u.desktop_dir())),
                downloads: user.as_ref().and_then(|u| opt(u.download_dir())),
                documents: user.as_ref().and_then(|u| opt(u.document_dir())),
                pictures: user.as_ref().and_then(|u| opt(u.picture_dir())),
                videos: user.as_ref().and_then(|u| opt(u.video_dir())),
                music: user.as_ref().and_then(|u| opt(u.audio_dir())),
                public: user.as_ref().and_then(|u| opt(u.public_dir())),
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

    pub fn sidebar_dirs(&'static self) -> Vec<(&'static str, &'static PathBuf)> {
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