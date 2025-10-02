//! 脚本运行模块
//!
//! 提供脚本执行功能，支持Node.js和Python脚本的运行。

use crate::error::{Error, Result};
use std::path::PathBuf;

/// 脚本执行结果
#[derive(Debug, Clone)]
pub struct ScriptResult {
    /// 退出代码
    pub exit_code: i32,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
}

impl ScriptResult {
    /// 创建新的脚本执行结果
    pub fn new(exit_code: i32, stdout: String, stderr: String, duration_ms: u64) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
            duration_ms,
        }
    }

    /// 检查是否执行成功
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// 获取执行状态描述
    pub fn status(&self) -> &'static str {
        if self.is_success() {
            "success"
        } else {
            "failed"
        }
    }
}

/// 脚本运行器
pub struct ScriptRunner;

impl ScriptRunner {
    /// 创建新的脚本运行器实例
    pub fn new() -> Self {
        Self
    }

    /// 运行Node.js脚本
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    /// - `node_version`: Node.js版本（可选）
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn run_node_script(
        &self,
        script_path: &str,
        args: &[&str],
        node_version: Option<&str>,
    ) -> Result<ScriptResult> {
        use crate::node::NodeManager;
        use crate::run::executor::ScriptExecutor;

        // 检查脚本文件
        self.check_script_permissions(script_path)?;

        // 获取Node.js路径
        let node_manager = NodeManager::new();
        let target_version = node_version.unwrap_or("latest");
        let node_path = node_manager.get_node_path(target_version).await?;

        // 构建命令
        let mut command = format!("\"{}\" \"{}\"", node_path, script_path);
        for arg in args {
            command.push_str(&format!(" \"{}\"", arg));
        }

        // 执行命令
        let executor = ScriptExecutor::new();
        executor.execute_command_async(&command, None, None).await
    }

    /// 运行Python脚本
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    /// - `python_version`: Python版本（可选）
    /// - `venv_path`: 虚拟环境路径（可选）
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn run_python_script(
        &self,
        script_path: &str,
        args: &[&str],
        python_version: Option<&str>,
        venv_path: Option<&str>,
    ) -> Result<ScriptResult> {
        use crate::py::PyManager;
        use crate::run::executor::ScriptExecutor;

        // 检查脚本文件
        self.check_script_permissions(script_path)?;

        // 获取Python路径
        let py_manager = PyManager::new();
        let target_version = python_version.unwrap_or("3.11");

        let python_path = if let Some(venv) = venv_path {
            // 使用虚拟环境中的Python
            py_manager.get_python_path_in_venv(venv).await?
        } else {
            // 使用系统Python
            py_manager.get_python_path(target_version).await?
        };

        // 构建命令
        let mut command = format!("\"{}\" \"{}\"", python_path, script_path);
        for arg in args {
            command.push_str(&format!(" \"{}\"", arg));
        }

        // 执行命令
        let executor = ScriptExecutor::new();
        executor.execute_command_async(&command, None, None).await
    }

    /// 运行Shell脚本
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn run_shell_script(&self, script_path: &str, args: &[&str]) -> Result<ScriptResult> {
        use crate::run::executor::ScriptExecutor;

        // 检查脚本文件
        self.check_script_permissions(script_path)?;

        // 构建命令
        let mut command = format!("\"{}\"", script_path);
        for arg in args {
            command.push_str(&format!(" \"{}\"", arg));
        }

        // 执行命令
        let executor = ScriptExecutor::new();
        executor.execute_command_async(&command, None, None).await
    }

    /// 检查脚本文件是否存在
    pub fn script_exists(&self, script_path: &str) -> bool {
        PathBuf::from(script_path).exists()
    }

    /// 验证脚本文件权限
    pub fn check_script_permissions(&self, script_path: &str) -> Result<()> {
        let path = PathBuf::from(script_path);

        if !path.exists() {
            return Err(Error::Runtime(format!(
                "Script file not found: {}",
                script_path
            )));
        }

        if !path.is_file() {
            return Err(Error::Runtime(format!("Not a file: {}", script_path)));
        }

        // TODO: 检查文件权限

        Ok(())
    }
}

impl Default for ScriptRunner {
    fn default() -> Self {
        Self::new()
    }
}

// 子模块声明
pub mod checker;
pub mod config;
pub mod executor;

// 预导入模块
pub use checker::SmartChecker;
pub use config::{ConfigManager, RunConfig};
pub use executor::ScriptExecutor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_result() {
        let result = ScriptResult::new(0, "output".to_string(), "".to_string(), 100);

        assert!(result.is_success());
        assert_eq!(result.status(), "success");
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "output");
        assert_eq!(result.stderr, "");
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_script_runner_creation() {
        let runner = ScriptRunner::new();
        assert!(runner.script_exists("Cargo.toml")); // 检查Cargo.toml是否存在
    }
}
