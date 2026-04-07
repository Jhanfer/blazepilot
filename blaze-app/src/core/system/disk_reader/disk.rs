#[derive(Clone, Debug)]
pub struct Disk {
    pub display_name: String,
    pub device: Option<String>,
    pub idtype: Option<String>,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub mountpoint: Option<String>,
    pub available: u64,
    pub total: u64,
    pub used_percent: f32,
    
    pub is_removable: bool,
    pub is_partition: bool,
    pub size: u64,
}