//! 错误处理模块
//!
//! 定义库中使用的错误类型和错误处理机制

use std::fmt;
use std::string::FromUtf8Error;
use thiserror::Error;

/// 库的主要结果类型
pub type Result<T> = std::result::Result<T, Error>;

/// AI00-Run 库的主要错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),

    /// 运行时错误
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// 版本管理错误
    #[error("Version management error: {0}")]
    Version(String),

    /// 虚拟环境错误
    #[error("Virtual environment error: {0}")]
    VirtualEnv(String),

    /// 包管理错误
    #[error("Package management error: {0}")]
    Package(String),

    /// 脚本执行错误
    #[error("Script execution error: {0}")]
    Script(String),

    /// 网络错误
    #[error("Network error: {0}")]
    Network(String),

    /// 平台不支持错误
    #[error("Platform not supported: {0}")]
    Platform(String),

    /// 架构不支持错误
    #[error("Unsupported architecture")]
    UnsupportedArchitecture,

    /// 初始化失败错误
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),

    /// 命令执行失败错误
    #[error("Command execution failed: {command}")]
    CommandExecutionFailed {
        /// 执行的命令
        command: String,
        /// 错误来源
        #[source]
        source: std::io::Error,
    },

    /// Python版本未找到错误
    #[error("Python version not found: {0}")]
    PythonVersionNotFound(String),

    /// 虚拟环境未找到错误
    #[error("Virtual environment not found: {0}")]
    VirtualEnvironmentNotFound(String),

    /// Python可执行文件未找到错误
    #[error("Python executable not found: {0}")]
    PythonExecutableNotFound(String),

    /// Python可执行文件无效错误
    #[error("Python executable is not valid: {0}")]
    PythonExecutableNotValid(String),

    /// Reqwest HTTP错误
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Zip归档错误
    #[error("Zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Tar归档错误
    #[error("Tar archive error: {0}")]
    Tar(String),
}

/// 为 Error 类型实现 From 转换，方便错误处理
impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Other(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error::Other(format!("UTF-8 conversion error: {}", err))
    }
}

/// 错误代码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// IO 错误
    IoError = 1001,
    /// 配置错误
    ConfigError = 2001,
    /// 运行时错误
    RuntimeError = 3001,
    /// 版本管理错误
    VersionError = 4001,
    /// 虚拟环境错误
    VirtualEnvError = 5001,
    /// 包管理错误
    PackageError = 6001,
    /// 脚本执行错误
    ScriptError = 7001,
    /// 网络错误
    NetworkError = 8001,
    /// 平台不支持错误
    PlatformError = 9001,
    /// 虚拟环境未找到错误
    VirtualEnvironmentNotFound = 5002,
    /// Python可执行文件未找到错误
    PythonExecutableNotFound = 5003,
    /// Python可执行文件无效错误
    PythonExecutableNotValid = 5004,
}

impl ErrorCode {
    /// 获取错误代码的数字值
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// 获取错误代码的描述
    pub fn description(self) -> &'static str {
        match self {
            ErrorCode::IoError => "IO operation failed",
            ErrorCode::ConfigError => "Configuration is invalid",
            ErrorCode::RuntimeError => "Runtime operation failed",
            ErrorCode::VersionError => "Version management operation failed",
            ErrorCode::VirtualEnvError => "Virtual environment operation failed",
            ErrorCode::VirtualEnvironmentNotFound => "Virtual environment not found",
            ErrorCode::PythonExecutableNotFound => "Python executable not found",
            ErrorCode::PythonExecutableNotValid => "Python executable is not valid",
            ErrorCode::PackageError => "Package management operation failed",
            ErrorCode::ScriptError => "Script execution failed",
            ErrorCode::NetworkError => "Network operation failed",
            ErrorCode::PlatformError => "Platform not supported",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.as_u32(), self.description())
    }
}

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 错误代码
    pub code: ErrorCode,
    /// 错误消息
    pub message: String,
    /// 错误详情
    pub details: Option<String>,
    /// 错误来源
    pub source: Option<String>,
}

impl ErrorContext {
    /// 创建新的错误上下文
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            source: None,
        }
    }

    /// 设置错误详情
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// 设置错误来源
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;

        if let Some(details) = &self.details {
            write!(f, " - {}", details)?;
        }

        if let Some(source) = &self.source {
            write!(f, " (source: {})", source)?;
        }

        Ok(())
    }
}

/// 错误处理工具函数
pub mod utils {
    use super::*;

    /// 创建配置错误
    pub fn config_error(message: impl Into<String>) -> Error {
        Error::Config(message.into())
    }

    /// 创建运行时错误
    pub fn runtime_error(message: impl Into<String>) -> Error {
        Error::Runtime(message.into())
    }

    /// 创建版本管理错误
    pub fn version_error(message: impl Into<String>) -> Error {
        Error::Version(message.into())
    }

    /// 创建虚拟环境错误
    pub fn virtual_env_error(message: impl Into<String>) -> Error {
        Error::VirtualEnv(message.into())
    }

    /// 创建包管理错误
    pub fn package_error(message: impl Into<String>) -> Error {
        Error::Package(message.into())
    }

    /// 创建脚本执行错误
    pub fn script_error(message: impl Into<String>) -> Error {
        Error::Script(message.into())
    }

    /// 创建网络错误
    pub fn network_error(message: impl Into<String>) -> Error {
        Error::Network(message.into())
    }

    /// 创建平台不支持错误
    pub fn platform_error(message: impl Into<String>) -> Error {
        Error::Platform(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Config("Invalid configuration".to_string());
        assert_eq!(
            err.to_string(),
            "Configuration error: Invalid configuration"
        );
    }

    #[test]
    fn test_error_code() {
        assert_eq!(ErrorCode::IoError.as_u32(), 1001);
        assert_eq!(
            ErrorCode::ConfigError.description(),
            "Configuration is invalid"
        );
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new(ErrorCode::ConfigError, "Invalid config")
            .with_details("Missing required field")
            .with_source("config.rs");

        assert!(context.to_string().contains("Invalid config"));
        assert!(context.to_string().contains("Missing required field"));
        assert!(context.to_string().contains("config.rs"));
    }

    #[test]
    fn test_error_utils() {
        let err = utils::config_error("test");
        assert!(matches!(err, Error::Config(_)));
    }
}
