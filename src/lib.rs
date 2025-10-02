//! ai00-run 库
//!
//! 提供多语言运行时管理和脚本执行功能。

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod init;
pub mod node;
pub mod py;
pub mod run;

// 预导入模块
pub use error::{Error, Result};
pub use init::{get_library_info, init, InitManager, LibraryInfo};
pub use node::NodeManager;
pub use py::PyManager;
pub use run::ScriptRunner;

/// 库版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = "ai00-run";

/// 版本宏
#[macro_export]
macro_rules! version {
    () => {
        $crate::VERSION
    };
}

/// 名称宏
#[macro_export]
macro_rules! name {
    () => {
        $crate::NAME
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_macro() {
        assert_eq!(version!(), VERSION);
    }

    #[test]
    fn test_name_macro() {
        assert_eq!(name!(), NAME);
    }

    #[tokio::test]
    async fn test_init() {
        assert!(init().await.is_ok());
    }
}
