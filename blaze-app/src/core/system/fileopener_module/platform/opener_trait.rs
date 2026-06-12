use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::core::system::fileopener_module::error::OpenerResult;

#[derive(Debug, Clone)]
pub enum AppIconSource {
    Unresolved(String),
    Path(PathBuf),
    None,
}

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub icon: AppIconSource,
    pub is_default: bool,
    pub is_recommended: bool,
}

pub trait FileOpener: Send + Sync {
    /// Implementación específica para cada plataforma
    fn get_mime(&self, path: Arc<Path>) -> String;
    /// Abre con app predeterminada del sistema o la guardada por Blaze
    fn open_file(&self, path: Arc<Path>) -> OpenerResult<()>;
    /// Abre el archivo con una app concreta, identificada por su ID
    fn open_with(&self, app_id: &str, path: Arc<Path>) -> OpenerResult<()>;
    /// Devuelve la app predeterminada para este archivo, si existe.
    fn get_default_app(&self, path: Arc<Path>) -> OpenerResult<Option<AppInfo>>;
    ///  Lista de apps recomendadas para abrir este archivo.
    fn get_available_apps(&self, path: Arc<Path>) -> OpenerResult<Vec<AppInfo>>;
    /// Tiene todas las appas.
    fn get_all_apps(&self, path: Arc<Path>) -> OpenerResult<Vec<AppInfo>>;
    ///  Registra app_id como predeterminada en el sistema
    fn set_system_default(&self, path: Arc<Path>, app_id: &str) -> OpenerResult<()>;
}
