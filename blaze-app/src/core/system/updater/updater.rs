use std::{error::Error, thread::current};
use self_update::cargo_crate_version;
use tracing::{error, info};
use uuid::Uuid;

use crate::utils::channel_pool::{NotifyingSender, UiEvent};

#[derive(Debug)]
pub enum UpdateMessages {
    NewVersionAvailable {
        current_version: String,
        new_version: String,
        tab_id: Uuid,
    },

    UpToDate,

    ProcedToUpdate,
}

pub struct Updater {
    pub version: String,
    pub owner: String,
    pub repo: String,
    pub is_updating: bool,
}

impl Updater {
    pub fn init() -> Self {
        Self {
            version: cargo_crate_version!().to_string(),
            owner: "Jhanfer".to_string(),
            repo:"blazepilot".to_string(),
            is_updating: false,
        }
    }

    fn is_newer_version(current: &str, new: &str) -> bool {
        let parse = |v: &str| -> Vec<u32> {
            v.split(".")
            .map(|s| s.parse::<u32>().unwrap_or(0))
            .collect()
        };

        let curr = parse(current);
        let newv = parse(new);

        newv > curr
    }

    pub fn check_for_update(&mut self, sender: NotifyingSender) {
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

            match result {
                Ok(new_ver) => {
                    if let Some(new_ver) = new_ver {
                        let tab_id = sender.tab_id;
                        if Self::is_newer_version(&version, &new_ver) {
                            sender.send_ui_event(
                                    UiEvent::UpdateMessages(
                                        UpdateMessages::NewVersionAvailable { 
                                        current_version: version, 
                                        new_version: new_ver,
                                        tab_id,
                                    }
                                )
                            ).ok();
                        } else {
                            sender.send_ui_event(
                                    UiEvent::UpdateMessages(
                                        UpdateMessages::UpToDate
                                )
                            ).ok();
                        }
                    } else {
                        info!("App actualizada a la última versión.");
                    }
                },
                Err(e) => {
                    info!("Error al buscar actualización: {}.", e);
                }
            }
        });
    }


    pub fn start_update_process(&mut self) {
        self.is_updating = true;
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
                    .no_confirm(true)
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

        self.is_updating = false;
    }
}