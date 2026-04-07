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





use std::{sync::Arc};
use once_cell::sync::Lazy;


pub static MACOS_FILE_OPENER: Lazy<Arc<tokio::sync::Mutex<MacosOpener>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(MacosOpener::init()))
});


pub struct MacosOpener {
}


impl MacosOpener {
    fn init() -> Self {
        Self {
        }
    }

    #[cfg(target_os = "macos")]
    fn get_apps_for_mime_macos(&self, mime: &str) -> Vec<AppAssociation> {

    }

    #[cfg(target_os = "macos")]
    fn launch_app_macos(&self, app: &AppAssociation, path: &Path) -> std::io::Result<()> {

    }
}