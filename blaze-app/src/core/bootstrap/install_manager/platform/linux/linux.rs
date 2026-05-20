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




use std::{os::unix::fs::PermissionsExt, path::{Path, PathBuf}, process::Command, sync::Arc};
use tracing::{info, warn};
use crate::core::bootstrap::install_manager::{installation_manager::InstallResult, platform::installation_trait::InstallationTrait};


const INSTALL_PATH: &str = "/usr/local/bin/blazepilot";
const FALLBACK_PATH: &str = ".local/bin/blazepilot";


pub struct LinuxInstallation {
    pub installation_path: PathBuf,
    pub fallback_path: PathBuf,
}


impl Default for LinuxInstallation {
    fn default() -> Self {
        let fallback = std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join(FALLBACK_PATH))
            .unwrap_or_else(|| PathBuf::from(FALLBACK_PATH));

        Self { 
            installation_path: PathBuf::from(INSTALL_PATH),
            fallback_path: fallback,
        }
    }
}


impl InstallationTrait for LinuxInstallation {
    fn installation_path(&self) -> &Path {
        &self.installation_path
    }

    fn fallback_path(&self) -> &Path {
        &self.fallback_path
    }

    fn install(&self) -> InstallResult {
        let current = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => return InstallResult::Failed(e.to_string().into()),
        };

        if current == self.installation_path {
            return InstallResult::AlreadyInstalled;
        }

        if self.try_direct_copy(&current, &self.installation_path) {
            return InstallResult::InstalledSystem(current.into());
        }

        if self.try_pkexec_install(&current, &self.installation_path) {
            return InstallResult::InstalledSystem(current.into());
        }

        if let Some(local) = self.try_local_install(&current) {
            return InstallResult::InstalledLocal(local.into());
        }

        InstallResult::Failed("No se pudo instalar en ninguna ruta".into())
    }

    fn post_install(&self) -> Result<(), String> {
        self.generate_desktop_file()?;
        self.register_mime_default();
        Ok(())
    }
}


impl LinuxInstallation {
    fn try_direct_copy(&self, src: &PathBuf, dst: &PathBuf) -> bool {
        match std::fs::copy(src, dst) {
            Ok(_) => {
                match std::fs::set_permissions(dst, std::fs::Permissions::from_mode(0o755)) {
                    Ok(_) => {
                        return true;
                    },

                    Err(e) => {
                        warn!("Ha ocurrido un error poniendo los permisos al binario: {e}");
                        return false;
                    }
                }
            },

            Err(e) => {
                warn!("Ha ocurrido un error copiando el binario: {e}");
                return false;
            }
        }
    }

    fn try_pkexec_install(&self, src: &PathBuf, dst: &PathBuf) -> bool {
        let mut cmd = Command::new("pkexec");

        cmd.envs(std::env::vars());

        cmd.args([
            "install", "-m", "755", "-o", "root",
            src.to_str().unwrap_or(""),
            dst.to_str().unwrap_or(""),
        ]);

        cmd.stdin(std::process::Stdio::null());

        match cmd.status() {
            Ok(s) if s.success() => true,
            Ok(s) => {
                warn!("pkexec terminó con código: {:?}", s.code());
                false
            },
            Err(e) => {
                warn!("Ha ocurrido un error ejecutando pkexec: {e}");
                return false;
            }
        }
    }

    fn try_local_install(&self, src: &PathBuf) -> Option<PathBuf> {
        if let Some(parent) = self.fallback_path.parent() {
            match std::fs::create_dir_all(parent) {
                Ok(_) => {},
                Err(e) => {
                    warn!("No se ha podido crear carpeta en '{:?}': {e}", parent);
                    return None;
                }
            }
        }

        if std::fs::copy(src, &self.fallback_path).is_ok() {
            match std::fs::set_permissions(&self.fallback_path, std::fs::Permissions::from_mode(0o755)) {
                Ok(_) => {},
                Err(e) => {
                    warn!("No se han podido dar permisos a '{:?}': {e}", self.fallback_path);
                    return None;
                },
            }
            Some(self.fallback_path.clone())
        } else {
            None
        }
    }


    fn get_data_home(&self) -> Arc<Path> {
        if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
            if !path.is_empty() {
                return Path::new(&path).into();
            }
        }

        if let Some(home) = std::env::var_os("HOME") {
            return Path::new(&home).join(".local/share").into();
        }

        return Path::new(&"/tmp").into();
    }


    fn write_desktop_file(&self, desktop_path: Arc<Path>) -> Result<(), String> {
        std::fs::File::create_new(&desktop_path)
            .map_err(|e|format!("No se ha podido crear '.desktop': {e}"))?;

        let fallback_path = Path::new(&std::env::var_os("HOME").unwrap_or_default())
            .join(self.fallback_path.clone());

        let exec = if self.installation_path.exists() {
            self.installation_path.clone()
        } else if fallback_path.exists() {
            fallback_path
        } else {
            std::fs::remove_file(&desktop_path)
                .map_err(|e|format!("No se ha podido borrar el '.desktop': {e}"))?;
            return Err("No existe instalación".to_string());
        };

        if desktop_path.exists() {
            let content = format!(
                "[Desktop Entry]\n\
                Name=BlazePilot\n\
                Comment=Explorador con estilo\n\
                Exec={} %u\n\
                Icon=system-file-manager\n\
                Terminal=false\n\
                Type=Application\n\
                MimeType=inode/directory;\n\
                Categories=System;FileManager;\n",
                exec.display()
            );
            std::fs::write(desktop_path, content)
                .map_err(|e| format!("No se ha podido escribir '.desktop': {e}"))?;
        }

        Ok(())
    }
    

    pub fn generate_desktop_file(&self) -> Result<(), String> {
        let data_home = self.get_data_home();
        let apps_dir = data_home.join("applications");
        let blz_desktop_file = apps_dir.join("blazepilot.desktop");

        std::fs::create_dir_all(&apps_dir)
            .map_err(|e|format!("No se ha podido crear directorio: {e}"))?;

        if blz_desktop_file.exists() {
            info!("Hola Mundo! Existe, usar")
        } else {
            info!("No existe {:?}, creando...", blz_desktop_file);
            self.write_desktop_file(blz_desktop_file.into())?;
        }

        Ok(())
    }


    pub fn register_mime_default(&self) {
        let data_home = self.get_data_home();
        let apps_dir = data_home.join("applications");

        match Command::new("update-desktop-database")
            .arg(&apps_dir)
            .status() {
                Ok(s) if s.success() => {info!("Actualizado database: {}",apps_dir.display())},
                Ok(s) => {
                    info!("Salidio con código: {s}")
                }
                Err(e) => {warn!("Ha ocurrido un error actualizando database: {e}")}
            }
        
        match Command::new("xdg-mime")
            .args(["default", "blazepilot.desktop", "inode/directory"])
            .status() {
                Ok(s) if s.success() => {info!("'xdg-mime default' ha sido un éxito: {s}")},
                Ok(s) => {
                    info!("Salidio con código: {s}")
                }
                Err(e) => {warn!("Ha ocurrido un error actualizando database: {e}")}
            }
    }

}