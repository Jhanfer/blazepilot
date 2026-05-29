use crate::core::bootstrap::quick_access_manager::error::QuickAccResult;

pub trait QuickAccessTrait {
    fn load(&mut self) -> QuickAccResult<()>;
    fn save(&mut self) -> QuickAccResult<()>;
}