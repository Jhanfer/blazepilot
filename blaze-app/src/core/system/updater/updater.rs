use std::error::Error;

use self_update::cargo_crate_version;
use tracing::{error, info};

use crate::core::system::clipboard::TOKIO_RUNTIME;

pub struct Updater {
    pub version: String,
    pub owner: String,
    pub repo: String,
}

impl Updater {
    pub fn init() -> Self {
        Self {
            version: cargo_crate_version!().to_string(),
            owner: "Jhanfer".to_string(),
            repo:"blazepilot".to_string(),
        }
    }

    pub fn check_for_update(&mut self) {
        let version = self.version.clone();
        let owner = self.owner.clone();
        let repo = self.repo.clone();

        std::thread::spawn(move || {
            let result = (|| -> Result<Option<String>, Box<dyn Error>> {
                let release = self_update::backends::github::ReleaseList::configure()
                    .repo_owner(&owner)
                    .repo_name(&repo)
                    .build()?
                    .fetch()?;

                if let Some(latest) = release.first() {
                    if latest.version != version {
                        return Ok(Some(latest.version.clone()));
                    }
                }
                Ok(None)
            })();

            info!("{:?}", result);
        });
    }


    pub fn start_update_process(&mut self) {
        let version = self.version.clone();
        let owner = self.owner.clone();
        let repo = self.repo.clone();

        std::thread::spawn(move || {
            let status = self_update::backends::github::Update::configure()
                    .repo_owner(&owner)
                    .repo_name(&repo)
                    .bin_name("blazepilot")
                    .show_download_progress(true)
                    .current_version(&version)
                    .build()
                    .and_then(|u| u.update());

            match status {
                Ok(s) if s.updated() => {
                    info!("Actualizado a {}. Reinicia el programa.", s.version());
                    std::process::exit(0)
                },

                Ok(_) => info!("Ya estás actualizado"),

                Err(e) => error!("Error al actualizar: {}", e),
            }
        });
    }
}