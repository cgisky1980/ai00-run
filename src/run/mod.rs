//! 脚本运行模块
//!
//! 提供脚本执行功能，支持Node.js和Python脚本的运行。

use crate::error::{Error, Result};
use crate::run::executor::{ExecuteOptions, StreamExecutorHandle};
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

    /// 根据配置文件运行脚本
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径（JSON或YAML格式）
    /// - `options`: 执行选项
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn run_from_config(
        &self,
        config_path: &str,
        options: Option<ExecuteOptions>,
    ) -> Result<ScriptResult> {
        use crate::run::config::ConfigManager;

        // 加载配置
        let config_manager = ConfigManager::new();
        let config = if config_path.ends_with(".json") {
            config_manager.load_from_json(config_path).await?
        } else if config_path.ends_with(".yaml") || config_path.ends_with(".yml") {
            config_manager.load_from_yaml(config_path).await?
        } else {
            return Err(Error::Config(format!(
                "Unsupported config file format: {}",
                config_path
            )));
        };

        // 根据配置类型运行脚本
        match config.script_type.as_str() {
            "node" => {
                let node_version = config.runtime_version.as_deref();
                let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
                self.run_node_script_with_options(
                    config.script_path.to_str().unwrap(),
                    &args,
                    node_version,
                    options,
                )
                .await
            }
            "python" => {
                let python_version = config.runtime_version.as_deref();
                let venv_path = config.venv_path.as_ref().map(|p| p.to_str().unwrap());
                let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
                self.run_python_script_with_options(
                    config.script_path.to_str().unwrap(),
                    &args,
                    python_version,
                    venv_path,
                    options,
                )
                .await
            }
            "shell" => {
                let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
                self.run_shell_script_with_options(
                    config.script_path.to_str().unwrap(),
                    &args,
                    options,
                )
                .await
            }
            _ => Err(Error::Config(format!(
                "Unsupported script type: {}",
                config.script_type
            ))),
        }
    }

    /// 流式运行脚本（支持长期运行和实时输出）
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径（JSON或YAML格式）
    ///
    /// # 返回值
    /// 返回流式执行器句柄
    pub async fn run_from_config_stream(&self, config_path: &str) -> Result<StreamExecutorHandle> {
        use crate::run::config::ConfigManager;

        // 加载配置
        let config_manager = ConfigManager::new();
        let config = if config_path.ends_with(".json") {
            config_manager.load_from_json(config_path).await?
        } else if config_path.ends_with(".yaml") || config_path.ends_with(".yml") {
            config_manager.load_from_yaml(config_path).await?
        } else {
            return Err(Error::Config(format!(
                "Unsupported config file format: {}",
                config_path
            )));
        };

        // 根据配置类型运行脚本
        match config.script_type.as_str() {
            "node" => {
                let node_version = config.runtime_version.as_deref();
                let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
                self.run_node_script_stream(
                    config.script_path.to_str().unwrap(),
                    &args,
                    node_version,
                )
                .await
            }
            "python" => {
                let python_version = config.runtime_version.as_deref();
                let venv_path = config.venv_path.as_ref().map(|p| p.to_str().unwrap());
                let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
                self.run_python_script_stream(
                    config.script_path.to_str().unwrap(),
                    &args,
                    python_version,
                    venv_path,
                )
                .await
            }
            "shell" => {
                let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
                self.run_shell_script_stream(config.script_path.to_str().unwrap(), &args)
                    .await
            }
            _ => Err(Error::Config(format!(
                "Unsupported script type: {}",
                config.script_type
            ))),
        }
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

    /// 运行Node.js脚本（支持执行选项）
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    /// - `node_version`: Node.js版本（可选）
    /// - `options`: 执行选项
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn run_node_script_with_options(
        &self,
        script_path: &str,
        args: &[&str],
        node_version: Option<&str>,
        options: Option<ExecuteOptions>,
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

        // 执行命令（支持超时）
        let executor = ScriptExecutor::new();
        let options = options.unwrap_or_default();

        executor
            .execute_command_with_timeout(
                &command,
                Some(options.env_vars),
                options.working_dir.as_deref(),
                options.timeout,
            )
            .await
    }

    /// 运行Python脚本（支持执行选项）
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    /// - `python_version`: Python版本（可选）
    /// - `venv_path`: 虚拟环境路径（可选）
    /// - `options`: 执行选项
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn run_python_script_with_options(
        &self,
        script_path: &str,
        args: &[&str],
        python_version: Option<&str>,
        venv_path: Option<&str>,
        options: Option<ExecuteOptions>,
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

        // 执行命令（支持超时）
        let executor = ScriptExecutor::new();
        let options = options.unwrap_or_default();

        executor
            .execute_command_with_timeout(
                &command,
                Some(options.env_vars),
                options.working_dir.as_deref(),
                options.timeout,
            )
            .await
    }

    /// 运行Shell脚本（支持执行选项）
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    /// - `options`: 执行选项
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn run_shell_script_with_options(
        &self,
        script_path: &str,
        args: &[&str],
        options: Option<ExecuteOptions>,
    ) -> Result<ScriptResult> {
        use crate::run::executor::ScriptExecutor;

        // 检查脚本文件
        self.check_script_permissions(script_path)?;

        // 构建命令
        let mut command = format!("\"{}\"", script_path);
        for arg in args {
            command.push_str(&format!(" \"{}\"", arg));
        }

        // 执行命令（支持超时）
        let executor = ScriptExecutor::new();
        let options = options.unwrap_or_default();

        executor
            .execute_command_with_timeout(
                &command,
                Some(options.env_vars),
                options.working_dir.as_deref(),
                options.timeout,
            )
            .await
    }

    /// 流式运行Node.js脚本（支持长期运行和实时输出）
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    /// - `node_version`: Node.js版本（可选）
    ///
    /// # 返回值
    /// 返回流式执行器句柄
    pub async fn run_node_script_stream(
        &self,
        script_path: &str,
        args: &[&str],
        node_version: Option<&str>,
    ) -> Result<StreamExecutorHandle> {
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

        // 流式执行命令
        let executor = ScriptExecutor::new();
        executor.execute_command_stream(&command, None, None).await
    }

    /// 流式运行Python脚本（支持长期运行和实时输出）
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    /// - `python_version`: Python版本（可选）
    /// - `venv_path`: 虚拟环境路径（可选）
    ///
    /// # 返回值
    /// 返回流式执行器句柄
    pub async fn run_python_script_stream(
        &self,
        script_path: &str,
        args: &[&str],
        python_version: Option<&str>,
        venv_path: Option<&str>,
    ) -> Result<StreamExecutorHandle> {
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

        // 流式执行命令
        let executor = ScriptExecutor::new();
        executor.execute_command_stream(&command, None, None).await
    }

    /// 流式运行Shell脚本（支持长期运行和实时输出）
    ///
    /// # 参数
    /// - `script_path`: 脚本文件路径
    /// - `args`: 命令行参数
    ///
    /// # 返回值
    /// 返回流式执行器句柄
    pub async fn run_shell_script_stream(
        &self,
        script_path: &str,
        args: &[&str],
    ) -> Result<StreamExecutorHandle> {
        use crate::run::executor::ScriptExecutor;

        // 检查脚本文件
        self.check_script_permissions(script_path)?;

        // 构建命令
        let mut command = format!("\"{}\"", script_path);
        for arg in args {
            command.push_str(&format!(" \"{}\"", arg));
        }

        // 流式执行命令
        let executor = ScriptExecutor::new();
        executor.execute_command_stream(&command, None, None).await
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

        // 项目内脚本管理：简化权限检查
        // 主要检查文件是否存在和是否为文件
        // 项目内操作不需要复杂的系统权限检查

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

/// 便捷函数：根据配置文件运行脚本
///
/// # 参数
/// - `config_path`: 配置文件路径（JSON或YAML格式）
///
/// # 返回值
/// 返回脚本执行结果
pub async fn run_from_config(config_path: &str) -> Result<ScriptResult> {
    ScriptRunner::new().run_from_config(config_path, None).await
}

/// 便捷函数：根据配置文件运行脚本（支持执行选项）
///
/// # 参数
/// - `config_path`: 配置文件路径（JSON或YAML格式）
/// - `options`: 执行选项
///
/// # 返回值
/// 返回脚本执行结果
pub async fn run_from_config_with_options(
    config_path: &str,
    options: ExecuteOptions,
) -> Result<ScriptResult> {
    ScriptRunner::new()
        .run_from_config(config_path, Some(options))
        .await
}

/// 便捷函数：流式运行脚本（支持长期运行和实时输出）
///
/// # 参数
/// - `config_path`: 配置文件路径（JSON或YAML格式）
///
/// # 返回值
/// 返回流式执行器句柄
pub async fn run_from_config_stream(config_path: &str) -> Result<StreamExecutorHandle> {
    ScriptRunner::new()
        .run_from_config_stream(config_path)
        .await
}

/// 便捷函数：流式运行Node.js脚本
///
/// # 参数
/// - `script_path`: 脚本文件路径
/// - `args`: 命令行参数
/// - `node_version`: Node.js版本（可选）
///
/// # 返回值
/// 返回流式执行器句柄
pub async fn run_node_script_stream(
    script_path: &str,
    args: &[&str],
    node_version: Option<&str>,
) -> Result<StreamExecutorHandle> {
    ScriptRunner::new()
        .run_node_script_stream(script_path, args, node_version)
        .await
}

/// 便捷函数：流式运行Python脚本
///
/// # 参数
/// - `script_path`: 脚本文件路径
/// - `args`: 命令行参数
/// - `python_version`: Python版本（可选）
/// - `venv_path`: 虚拟环境路径（可选）
///
/// # 返回值
/// 返回流式执行器句柄
pub async fn run_python_script_stream(
    script_path: &str,
    args: &[&str],
    python_version: Option<&str>,
    venv_path: Option<&str>,
) -> Result<StreamExecutorHandle> {
    ScriptRunner::new()
        .run_python_script_stream(script_path, args, python_version, venv_path)
        .await
}

/// 便捷函数：流式运行Shell脚本
///
/// # 参数
/// - `script_path`: 脚本文件路径
/// - `args`: 命令行参数
///
/// # 返回值
/// 返回流式执行器句柄
pub async fn run_shell_script_stream(
    script_path: &str,
    args: &[&str],
) -> Result<StreamExecutorHandle> {
    ScriptRunner::new()
        .run_shell_script_stream(script_path, args)
        .await
}

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
