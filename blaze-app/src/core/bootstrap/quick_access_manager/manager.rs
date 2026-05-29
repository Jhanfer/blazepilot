use std::{collections::HashSet, sync::LazyLock};
use egui::Color32;
use parking_lot::Mutex;
use tracing::warn;
use uuid::Uuid;


use crate::core::bootstrap::quick_access_manager::{
    error::QuickAccResult,
    platform::{
        PlatformQuickAccess, 
        QuickAccessTrait, 
        QuickTag, 
        structs::QuickLinks,
    }
};


pub static GLOBAL_QUICK_ACCESS: LazyLock<Mutex<QuickAccessManager>> = LazyLock::new(|| {
    Mutex::new(QuickAccessManager::new())
});

pub fn with_quick_tags<R>(f: impl FnOnce(&mut QuickAccessManager) -> R) -> R {
    f(&mut GLOBAL_QUICK_ACCESS.lock())
}


pub struct QuickAccessManager {
    platform: PlatformQuickAccess
}

impl QuickAccessManager {
    pub fn new() -> Self {
        let mut manager = Self {
            platform: PlatformQuickAccess::default(),
        };
        
        if let Err(e) = manager.platform.load() {
            warn!("Ha fallado la carga de los accesos rápidos: {e}");
        }

        manager
    }


    fn find_tag<F>(&mut self, tag_id: Uuid, mut callback: F) -> bool
        where F: FnMut(&mut QuickTag) -> bool {
            let found_tag = self.platform.tags.iter_mut().find(|t| t.id == tag_id);

            if let Some(tag) = found_tag {
                return callback(tag);
            }

            false
    }

    pub fn update_tag_callback<F>(&mut self, id: Uuid, mut updater: F) -> bool 
        where F: FnMut(&mut QuickTag) {
            let found =  self.find_tag(id, |tag|{
                updater(tag);
                true
            });

            if found {
                self.platform.save().ok();
            }

            found
    }


    fn find_quick<F>(&mut self, tag_id: Uuid, quick_id: Uuid, mut callback: F) -> bool
        where F: FnMut(&mut QuickLinks) -> bool {
            let Some(tag) = self.platform.tags.iter_mut().find(|t| t.id == tag_id) else {
                return false;
            };

            let Some(quick) = tag.items.iter_mut().find(|q| q.id == quick_id) else {
                return false;
            };

            callback(quick)
        }


    pub fn update_quick_callback<F>(&mut self, tag_id: Uuid, quick_id: Uuid, mut updater: F) -> bool
        where F: FnMut(&mut QuickLinks) {
            let found = self.find_quick(tag_id, quick_id, |quick| {
                updater(quick);
                true
            });
            
            if found {
                self.platform.save().ok();
            }

            found
        }

    
    pub fn remove_tag(&mut self, tag_id: Uuid) {
        self.platform.tags.retain(|rtag| rtag.id != tag_id);
        self.platform.save().ok();
    }


    pub fn get_tags(&self) -> Vec<QuickTag> {
        self.platform.tags.to_owned()
    }


    pub fn create_tag(&mut self, title: &str, color: Color32) -> bool {
        if self.platform.tags.iter().any(|t| t.title.eq_ignore_ascii_case(title)) {
            return false;
        }

        let tag = QuickTag::new(title, color);

        self.platform.tags.push(tag);
        self.platform.save().ok();
        true
    }


    pub fn add_quicks_to_tag(&mut self, tag_id: Uuid, quicks: &Vec<QuickLinks>) -> bool {
        let found = self.find_tag(tag_id, |tag| {
            let quick_ids: HashSet<_> = quicks.iter().map(|q| q.id).collect();

            if tag.items.iter().any(|ql| quick_ids.contains(&ql.id)) {
                return false;
            } else {
                tag.items.extend(quicks.to_owned());
                return true;
            }
        });

        if found {
            self.platform.save().ok();
        }

        found
    }


    pub fn remove_quick_to_tag(&mut self, tag_id: Uuid, quick_id: Uuid) {
        let found = self.find_tag(tag_id, |tag| {
            tag.items.retain(|q| q.id != quick_id);
            return true;
        });

        if found {
            self.platform.save().ok();
        }
    }

    #[must_use]
    pub fn save(&mut self) -> QuickAccResult<()> {
        self.platform.save()
    }
}