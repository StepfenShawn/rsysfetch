use crate::AppResult;
use crate::system_info::SystemInfo;

/// 应用程序状态
#[derive(Debug)]
pub struct App {
    pub system_info: SystemInfo,
    pub should_quit: bool,
}

impl App {
    /// 创建新的应用程序实例
    pub fn new() -> AppResult<Self> {
        let system_info = SystemInfo::collect()?;

        Ok(Self {
            system_info,
            should_quit: false,
        })
    }
}
