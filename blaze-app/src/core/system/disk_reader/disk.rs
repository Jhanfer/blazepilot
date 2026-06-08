#![allow(unused)]

#[derive(Clone, Debug)]
pub struct Disk {
    pub display_name: String,
    pub device: Option<String>,
    pub is_removable: bool,
    pub mountpoint: Option<String>,
    pub used_percent: f32,
    pub is_system: bool,
    pub available: u64,
    pub total: u64,
    pub idtype: Option<String>,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub is_partition: bool,
    pub size: u64,
}
