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





use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct MimeAppsFile {
    pub default_apps: HashMap<String, Vec<String>>,
    pub added: HashMap<String, Vec<String>>,
    pub removed: HashMap<String, Vec<String>>,
}

impl MimeAppsFile {
    pub fn parse(content: &str) -> Self {
        let mut current_group: Option<String> = None;
        let mut file = MimeAppsFile::default();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with("#") || line.starts_with(";") {
                continue;
            }

            if line.starts_with("[") && line.ends_with("]") {
                current_group = Some(line[1..line.len() - 1].to_string());
                continue;
            }

            let group = match current_group.as_deref() {
                Some("Default Applications") |
                Some("Added Associations") | 
                Some("Removed Associations") => current_group.as_deref(),
                _ => None,
            };

            if group.is_none() {continue;}

            let mut parts = line.splitn(2, "=");
            let key = match parts.next() {
                Some(k) if !k.is_empty() => k.trim(),
                _ => continue,
            };
            let value = match parts.next() {
                Some(v) => v.trim(),
                _ => continue,
            };

            let apps: Vec<String> = value
                .split(";")
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            if apps.is_empty() {continue;}

            match group.unwrap() {
                "Default Applications" => {
                    file.default_apps.insert(key.to_string(), apps);
                },
                "Added Associations" => {
                    file.added.insert(key.to_string(), apps);
                },
                "Removed Associations" => {
                    file.removed.insert(key.to_string(), apps);
                },
                _ => {}
            }
        }

        file
    }
}