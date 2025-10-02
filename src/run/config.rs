//! 配置管理模块
//!
//! 负责脚本运行配置的加载、解析和管理。

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// 脚本运行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    /// 脚本名称
    pub name: String,
    /// 脚本描述
    pub description: Option<String>,
    /// 脚本类型（"node", "python", "shell"）
    pub script_type: String,
    /// 脚本文件路径
    pub script_path: PathBuf,
    /// 运行时版本（Node.js版本或Python版本）
    pub runtime_version: Option<String>,
    /// 虚拟环境路径（仅Python）
    pub venv_path: Option<PathBuf>,
    /// 命令行参数
    pub args: Vec<String>,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 工作目录
    pub working_dir: Option<PathBuf>,
    /// 超时时间（毫秒），设置为None表示无超时限制（长时间运行）
    pub timeout: Option<u64>,
    /// 是否异步执行
    pub async_execution: Option<bool>,
    /// 依赖项
    pub dependencies: Option<Vec<String>>,
    /// 是否启用流式执行（实时输出）
    pub streaming_execution: Option<bool>,
    /// 流式输出缓冲区大小（字节）
    pub stream_buffer_size: Option<usize>,
    /// 长时间运行进程的重启策略
    pub restart_policy: Option<String>,
    /// 最大重启次数
    pub max_restarts: Option<u32>,
    /// 重启间隔（毫秒）
    pub restart_delay: Option<u64>,
    /// 进程监控间隔（毫秒）
    pub monitor_interval: Option<u64>,
    /// 是否启用进程守护模式
    pub daemon_mode: Option<bool>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            name: "unnamed_script".to_string(),
            description: None,
            script_type: "shell".to_string(),
            script_path: PathBuf::from("script.sh"),
            runtime_version: None,
            venv_path: None,
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
            timeout: None, // None表示无超时限制，支持长时间运行
            async_execution: Some(true),
            dependencies: None,
            streaming_execution: Some(false),
            stream_buffer_size: Some(8192),
            restart_policy: Some("never".to_string()),
            max_restarts: Some(3),
            restart_delay: Some(5000),
            monitor_interval: Some(1000),
            daemon_mode: Some(false),
        }
    }
}

impl RunConfig {
    /// 创建新的运行配置
    pub fn new(name: &str, script_type: &str, script_path: &str) -> Self {
        Self {
            name: name.to_string(),
            script_type: script_type.to_string(),
            script_path: PathBuf::from(script_path),
            ..Default::default()
        }
    }

    /// 设置脚本描述
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// 设置运行时版本
    pub fn with_runtime_version(mut self, version: &str) -> Self {
        self.runtime_version = Some(version.to_string());
        self
    }

    /// 设置虚拟环境路径
    pub fn with_venv_path(mut self, venv_path: &str) -> Self {
        self.venv_path = Some(PathBuf::from(venv_path));
        self
    }

    /// 添加命令行参数
    pub fn add_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// 添加多个命令行参数
    pub fn add_args(mut self, args: &[&str]) -> Self {
        for arg in args {
            self.args.push(arg.to_string());
        }
        self
    }

    /// 设置环境变量
    pub fn with_env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置多个环境变量
    pub fn with_env_vars(mut self, env_vars: &[(&str, &str)]) -> Self {
        for (key, value) in env_vars {
            self.env_vars.insert(key.to_string(), value.to_string());
        }
        self
    }

    /// 设置工作目录
    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(PathBuf::from(dir));
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }

    /// 设置异步执行
    pub fn with_async_execution(mut self, async_exec: bool) -> Self {
        self.async_execution = Some(async_exec);
        self
    }

    /// 添加依赖项
    pub fn add_dependency(mut self, dependency: &str) -> Self {
        if self.dependencies.is_none() {
            self.dependencies = Some(Vec::new());
        }
        if let Some(ref mut deps) = self.dependencies {
            deps.push(dependency.to_string());
        }
        self
    }

    /// 添加多个依赖项
    pub fn add_dependencies(mut self, dependencies: &[&str]) -> Self {
        if self.dependencies.is_none() {
            self.dependencies = Some(Vec::new());
        }
        if let Some(ref mut deps) = self.dependencies {
            for dependency in dependencies {
                deps.push(dependency.to_string());
            }
        }
        self
    }

    /// 启用流式执行（实时输出）
    pub fn with_streaming_execution(mut self, streaming: bool) -> Self {
        self.streaming_execution = Some(streaming);
        self
    }

    /// 设置流式输出缓冲区大小
    pub fn with_stream_buffer_size(mut self, buffer_size: usize) -> Self {
        self.stream_buffer_size = Some(buffer_size);
        self
    }

    /// 设置重启策略
    pub fn with_restart_policy(mut self, policy: &str) -> Self {
        self.restart_policy = Some(policy.to_string());
        self
    }

    /// 设置最大重启次数
    pub fn with_max_restarts(mut self, max_restarts: u32) -> Self {
        self.max_restarts = Some(max_restarts);
        self
    }

    /// 设置重启间隔
    pub fn with_restart_delay(mut self, delay_ms: u64) -> Self {
        self.restart_delay = Some(delay_ms);
        self
    }

    /// 设置进程监控间隔
    pub fn with_monitor_interval(mut self, interval_ms: u64) -> Self {
        self.monitor_interval = Some(interval_ms);
        self
    }

    /// 启用进程守护模式
    pub fn with_daemon_mode(mut self, daemon: bool) -> Self {
        self.daemon_mode = Some(daemon);
        self
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        // 检查脚本名称
        if self.name.is_empty() {
            return Err(Error::Config("Script name cannot be empty".to_string()));
        }

        // 检查脚本类型
        let valid_types = ["node", "python", "shell"];
        if !valid_types.contains(&self.script_type.as_str()) {
            return Err(Error::Config(format!(
                "Invalid script type: {}",
                self.script_type
            )));
        }

        // 检查脚本路径
        if self.script_path.to_string_lossy().is_empty() {
            return Err(Error::Config("Script path cannot be empty".to_string()));
        }

        // 检查Node.js配置
        if self.script_type == "node" && self.runtime_version.is_none() {
            return Err(Error::Config(
                "Node.js script requires runtime version".to_string(),
            ));
        }

        // 检查Python配置
        if self.script_type == "python" && self.venv_path.is_none() {
            return Err(Error::Config(
                "Python script requires virtual environment path".to_string(),
            ));
        }

        // 检查流式执行配置
        if let Some(buffer_size) = self.stream_buffer_size {
            if !(1024..=65536).contains(&buffer_size) {
                return Err(Error::Config(
                    "Stream buffer size must be between 1024 and 65536 bytes".to_string(),
                ));
            }
        }

        // 检查重启策略
        if let Some(ref policy) = self.restart_policy {
            let valid_policies = ["never", "on-failure", "always"];
            if !valid_policies.contains(&policy.as_str()) {
                return Err(Error::Config(format!("Invalid restart policy: {}", policy)));
            }
        }

        // 检查监控间隔
        if let Some(interval) = self.monitor_interval {
            if !(100..=60000).contains(&interval) {
                return Err(Error::Config(
                    "Monitor interval must be between 100 and 60000 milliseconds".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// 获取脚本文件扩展名
    pub fn get_script_extension(&self) -> Option<String> {
        self.script_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())
    }

    /// 获取完整的命令行参数
    pub fn get_full_command_args(&self) -> Vec<String> {
        let mut full_args = Vec::new();

        // 添加脚本路径
        full_args.push(self.script_path.to_string_lossy().to_string());

        // 添加额外参数
        full_args.extend(self.args.clone());

        full_args
    }
}

/// 配置管理器
pub struct ConfigManager;

impl ConfigManager {
    /// 创建新的配置管理器实例
    pub fn new() -> Self {
        Self
    }

    /// 从JSON文件加载配置
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径
    ///
    /// # 返回值
    /// 返回解析后的运行配置
    pub async fn load_from_json(&self, config_path: &str) -> Result<RunConfig> {
        let config_content = tokio::fs::read_to_string(config_path)
            .await
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        let config: RunConfig = serde_json::from_str(&config_content)
            .map_err(|e| Error::Config(format!("Failed to parse config JSON: {}", e)))?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 从YAML文件加载配置
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径
    ///
    /// # 返回值
    /// 返回解析后的运行配置
    pub async fn load_from_yaml(&self, config_path: &str) -> Result<RunConfig> {
        let config_content = tokio::fs::read_to_string(config_path)
            .await
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        let config: RunConfig = serde_yaml::from_str(&config_content)
            .map_err(|e| Error::Config(format!("Failed to parse config YAML: {}", e)))?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 保存配置到JSON文件
    ///
    /// # 参数
    /// - `config`: 运行配置
    /// - `config_path`: 配置文件路径
    pub async fn save_to_json(&self, config: &RunConfig, config_path: &str) -> Result<()> {
        // 验证配置
        config.validate()?;

        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        tokio::fs::write(config_path, config_json)
            .await
            .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// 保存配置到YAML文件
    ///
    /// # 参数
    /// - `config`: 运行配置
    /// - `config_path`: 配置文件路径
    pub async fn save_to_yaml(&self, config: &RunConfig, config_path: &str) -> Result<()> {
        // 验证配置
        config.validate()?;

        let config_yaml = serde_yaml::to_string(config)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        tokio::fs::write(config_path, config_yaml)
            .await
            .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// 从环境变量创建配置
    ///
    /// # 参数
    /// - `prefix`: 环境变量前缀
    ///
    /// # 返回值
    /// 返回从环境变量创建的运行配置
    pub fn from_env_vars(&self, prefix: &str) -> RunConfig {
        let mut config = RunConfig::default();

        // 设置环境变量前缀
        let env_prefix = if prefix.is_empty() {
            "AI00_RUN_".to_string()
        } else {
            format!("{}_", prefix)
        };

        // 从环境变量读取配置
        if let Ok(name) = std::env::var(format!("{}NAME", env_prefix)) {
            config.name = name;
        }

        if let Ok(description) = std::env::var(format!("{}DESCRIPTION", env_prefix)) {
            config.description = Some(description);
        }

        if let Ok(script_type) = std::env::var(format!("{}SCRIPT_TYPE", env_prefix)) {
            config.script_type = script_type;
        }

        if let Ok(script_path) = std::env::var(format!("{}SCRIPT_PATH", env_prefix)) {
            config.script_path = PathBuf::from(script_path);
        }

        if let Ok(runtime_version) = std::env::var(format!("{}RUNTIME_VERSION", env_prefix)) {
            config.runtime_version = Some(runtime_version);
        }

        if let Ok(venv_path) = std::env::var(format!("{}VENV_PATH", env_prefix)) {
            config.venv_path = Some(PathBuf::from(venv_path));
        }

        if let Ok(working_dir) = std::env::var(format!("{}WORKING_DIR", env_prefix)) {
            config.working_dir = Some(PathBuf::from(working_dir));
        }

        if let Ok(timeout) = std::env::var(format!("{}TIMEOUT", env_prefix)) {
            if let Ok(timeout_ms) = timeout.parse::<u64>() {
                config.timeout = Some(timeout_ms);
            }
        }

        if let Ok(async_execution) = std::env::var(format!("{}ASYNC_EXECUTION", env_prefix)) {
            if let Ok(async_exec) = async_execution.parse::<bool>() {
                config.async_execution = Some(async_exec);
            }
        }

        // 流式执行配置
        if let Ok(streaming_execution) = std::env::var(format!("{}STREAMING_EXECUTION", env_prefix))
        {
            if let Ok(streaming) = streaming_execution.parse::<bool>() {
                config.streaming_execution = Some(streaming);
            }
        }

        if let Ok(stream_buffer_size) = std::env::var(format!("{}STREAM_BUFFER_SIZE", env_prefix)) {
            if let Ok(buffer_size) = stream_buffer_size.parse::<usize>() {
                config.stream_buffer_size = Some(buffer_size);
            }
        }

        // 长时间运行配置
        if let Ok(restart_policy) = std::env::var(format!("{}RESTART_POLICY", env_prefix)) {
            config.restart_policy = Some(restart_policy);
        }

        if let Ok(max_restarts) = std::env::var(format!("{}MAX_RESTARTS", env_prefix)) {
            if let Ok(max_restarts_val) = max_restarts.parse::<u32>() {
                config.max_restarts = Some(max_restarts_val);
            }
        }

        if let Ok(restart_delay) = std::env::var(format!("{}RESTART_DELAY", env_prefix)) {
            if let Ok(delay_ms) = restart_delay.parse::<u64>() {
                config.restart_delay = Some(delay_ms);
            }
        }

        if let Ok(monitor_interval) = std::env::var(format!("{}MONITOR_INTERVAL", env_prefix)) {
            if let Ok(interval_ms) = monitor_interval.parse::<u64>() {
                config.monitor_interval = Some(interval_ms);
            }
        }

        if let Ok(daemon_mode) = std::env::var(format!("{}DAEMON_MODE", env_prefix)) {
            if let Ok(daemon) = daemon_mode.parse::<bool>() {
                config.daemon_mode = Some(daemon);
            }
        }

        config
    }

    /// 生成配置模板
    ///
    /// # 参数
    /// - `config_type`: 配置类型（"node", "python", "shell"）
    /// - `template_path`: 模板文件路径
    ///
    /// # 返回值
    /// 返回生成的配置模板
    pub fn generate_template(&self, config_type: &str, template_path: &str) -> Result<RunConfig> {
        let template = match config_type {
            "node" => RunConfig::new("node_app", "node", "app.js")
                .with_description("Node.js application")
                .with_runtime_version("18.0.0")
                .add_arg("--port")
                .add_arg("3000")
                .with_env_var("NODE_ENV", "development")
                .with_timeout(30000),
            "python" => RunConfig::new("python_app", "python", "main.py")
                .with_description("Python application")
                .with_runtime_version("3.11")
                .with_venv_path(".venv")
                .add_dependency("requests")
                .add_dependency("flask")
                .with_env_var("PYTHONPATH", ".")
                .with_timeout(60000),
            "shell" => RunConfig::new("shell_script", "shell", "script.sh")
                .with_description("Shell script")
                .add_arg("--verbose")
                .with_env_var("DEBUG", "true")
                .with_timeout(10000),
            _ => {
                return Err(Error::Config(format!(
                    "Unknown config type: {}",
                    config_type
                )))
            }
        };

        // 保存模板到文件
        let template_json = serde_json::to_string_pretty(&template)
            .map_err(|e| Error::Config(format!("Failed to serialize template: {}", e)))?;

        let mut file = fs::File::create(template_path)
            .map_err(|e| Error::Config(format!("Failed to create template file: {}", e)))?;

        file.write_all(template_json.as_bytes())
            .map_err(|e| Error::Config(format!("Failed to write template file: {}", e)))?;

        Ok(template)
    }

    /// 合并多个配置
    ///
    /// # 参数
    /// - `base_config`: 基础配置
    /// - `override_config`: 覆盖配置
    ///
    /// # 返回值
    /// 返回合并后的配置
    pub fn merge_configs(&self, base_config: RunConfig, override_config: RunConfig) -> RunConfig {
        let mut merged = base_config;

        // 合并名称（如果覆盖配置有名称）
        if !override_config.name.is_empty() && override_config.name != "unnamed_script" {
            merged.name = override_config.name;
        }

        // 合并描述
        if override_config.description.is_some() {
            merged.description = override_config.description;
        }

        // 合并脚本类型
        if override_config.script_type != "shell" {
            merged.script_type = override_config.script_type;
        }

        // 合并脚本路径
        if override_config.script_path != PathBuf::from("script.sh") {
            merged.script_path = override_config.script_path;
        }

        // 合并运行时版本
        if override_config.runtime_version.is_some() {
            merged.runtime_version = override_config.runtime_version;
        }

        // 合并虚拟环境路径
        if override_config.venv_path.is_some() {
            merged.venv_path = override_config.venv_path;
        }

        // 合并命令行参数
        if !override_config.args.is_empty() {
            merged.args = override_config.args;
        }

        // 合并环境变量
        if !override_config.env_vars.is_empty() {
            merged.env_vars = override_config.env_vars;
        }

        // 合并工作目录
        if override_config.working_dir.is_some() {
            merged.working_dir = override_config.working_dir;
        }

        // 合并超时时间
        if override_config.timeout.is_some() {
            merged.timeout = override_config.timeout;
        }

        // 合并异步执行设置
        if override_config.async_execution.is_some() {
            merged.async_execution = override_config.async_execution;
        }

        // 合并依赖项
        if override_config.dependencies.is_some() {
            merged.dependencies = override_config.dependencies;
        }

        // 合并流式执行配置
        if override_config.streaming_execution.is_some() {
            merged.streaming_execution = override_config.streaming_execution;
        }

        if override_config.stream_buffer_size.is_some() {
            merged.stream_buffer_size = override_config.stream_buffer_size;
        }

        // 合并长时间运行配置
        if override_config.restart_policy.is_some() {
            merged.restart_policy = override_config.restart_policy;
        }

        if override_config.max_restarts.is_some() {
            merged.max_restarts = override_config.max_restarts;
        }

        if override_config.restart_delay.is_some() {
            merged.restart_delay = override_config.restart_delay;
        }

        if override_config.monitor_interval.is_some() {
            merged.monitor_interval = override_config.monitor_interval;
        }

        if override_config.daemon_mode.is_some() {
            merged.daemon_mode = override_config.daemon_mode;
        }

        merged
    }

    /// 验证配置文件的完整性
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径
    /// - `config_type`: 配置类型（"json", "yaml"）
    ///
    /// # 返回值
    /// 返回验证结果和错误信息
    pub async fn validate_config_file(
        &self,
        config_path: &str,
        config_type: &str,
    ) -> Result<(bool, Vec<String>)> {
        let mut errors = Vec::new();

        // 检查文件是否存在
        if !std::path::Path::new(config_path).exists() {
            errors.push(format!("Config file does not exist: {}", config_path));
            return Ok((false, errors));
        }

        // 根据类型加载配置
        let config = match config_type {
            "json" => self.load_from_json(config_path).await,
            "yaml" => self.load_from_yaml(config_path).await,
            _ => {
                errors.push(format!("Unsupported config type: {}", config_type));
                return Ok((false, errors));
            }
        };

        match config {
            Ok(config) => {
                // 验证配置
                if let Err(e) = config.validate() {
                    errors.push(format!("Config validation failed: {}", e));
                    Ok((false, errors))
                } else {
                    Ok((true, errors))
                }
            }
            Err(e) => {
                errors.push(format!("Failed to load config: {}", e));
                Ok((false, errors))
            }
        }
    }

    /// 获取配置摘要信息
    ///
    /// # 参数
    /// - `config`: 运行配置
    ///
    /// # 返回值
    /// 返回配置摘要字符串
    pub fn get_config_summary(&self, config: &RunConfig) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("Name: {}\n", config.name));
        summary.push_str(&format!("Type: {}\n", config.script_type));
        summary.push_str(&format!("Script: {}\n", config.script_path.display()));

        if let Some(version) = &config.runtime_version {
            summary.push_str(&format!("Runtime: {}\n", version));
        }

        if let Some(venv) = &config.venv_path {
            summary.push_str(&format!("Venv: {}\n", venv.display()));
        }

        if !config.args.is_empty() {
            summary.push_str(&format!("Args: {}\n", config.args.join(" ")));
        }

        if let Some(timeout) = config.timeout {
            summary.push_str(&format!("Timeout: {}ms\n", timeout));
        }

        if let Some(async_exec) = config.async_execution {
            summary.push_str(&format!("Async: {}\n", async_exec));
        }

        // 流式执行配置
        if let Some(streaming) = config.streaming_execution {
            summary.push_str(&format!("Streaming: {}\n", streaming));
        }

        if let Some(buffer_size) = config.stream_buffer_size {
            summary.push_str(&format!("Stream Buffer: {} bytes\n", buffer_size));
        }

        // 长时间运行配置
        if let Some(ref policy) = config.restart_policy {
            summary.push_str(&format!("Restart Policy: {}\n", policy));
        }

        if let Some(max_restarts) = config.max_restarts {
            summary.push_str(&format!("Max Restarts: {}\n", max_restarts));
        }

        if let Some(restart_delay) = config.restart_delay {
            summary.push_str(&format!("Restart Delay: {}ms\n", restart_delay));
        }

        if let Some(monitor_interval) = config.monitor_interval {
            summary.push_str(&format!("Monitor Interval: {}ms\n", monitor_interval));
        }

        if let Some(daemon_mode) = config.daemon_mode {
            summary.push_str(&format!("Daemon Mode: {}\n", daemon_mode));
        }

        summary
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_run_config_creation() {
        let config = RunConfig::new("test_script", "node", "app.js");

        assert_eq!(config.name, "test_script");
        assert_eq!(config.script_type, "node");
        assert_eq!(config.script_path, PathBuf::from("app.js"));
    }

    #[test]
    fn test_run_config_builder_pattern() {
        let config = RunConfig::new("test_script", "python", "script.py")
            .with_description("A test script")
            .with_runtime_version("3.11")
            .with_venv_path(".venv")
            .add_arg("--verbose")
            .add_args(&["--debug", "--test"])
            .with_env_var("DEBUG", "true")
            .with_env_vars(&[("KEY1", "value1"), ("KEY2", "value2")])
            .with_working_dir("/tmp")
            .with_timeout(5000)
            .with_async_execution(false)
            .add_dependency("requests")
            .add_dependencies(&["numpy", "pandas"]);

        assert_eq!(config.description, Some("A test script".to_string()));
        assert_eq!(config.runtime_version, Some("3.11".to_string()));
        assert_eq!(config.venv_path, Some(PathBuf::from(".venv")));
        assert_eq!(config.args, vec!["--verbose", "--debug", "--test"]);
        assert_eq!(config.env_vars.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.working_dir, Some(PathBuf::from("/tmp")));
        assert_eq!(config.timeout, Some(5000));
        assert_eq!(config.async_execution, Some(false));
        assert_eq!(
            config.dependencies,
            Some(vec![
                "requests".to_string(),
                "numpy".to_string(),
                "pandas".to_string()
            ])
        );
    }

    #[test]
    fn test_run_config_validation() {
        // 测试有效配置
        let valid_config = RunConfig::new("test", "node", "app.js").with_runtime_version("18.0.0");
        assert!(valid_config.validate().is_ok());

        // 测试无效配置（缺少运行时版本）
        let invalid_config = RunConfig::new("test", "node", "app.js");
        assert!(invalid_config.validate().is_err());

        // 测试无效配置（无效脚本类型）
        let invalid_config2 = RunConfig::new("test", "invalid", "app.js");
        assert!(invalid_config2.validate().is_err());
    }

    #[test]
    fn test_run_config_utility_methods() {
        let config = RunConfig::new("test", "python", "script.py").add_args(&["--arg1", "value1"]);

        assert_eq!(config.get_script_extension(), Some("py".to_string()));
        assert_eq!(
            config.get_full_command_args(),
            vec!["script.py", "--arg1", "value1"]
        );
    }

    #[tokio::test]
    async fn test_config_manager_json() {
        let manager = ConfigManager::new();

        // 创建临时配置文件
        let config = RunConfig::new("test", "node", "app.js").with_runtime_version("18.0.0");

        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_str().unwrap();

        // 保存配置
        let result = manager.save_to_json(&config, config_path).await;
        assert!(result.is_ok());

        // 加载配置
        let loaded_config = manager.load_from_json(config_path).await;
        assert!(loaded_config.is_ok());

        let loaded_config = loaded_config.unwrap();
        assert_eq!(loaded_config.name, "test");
        assert_eq!(loaded_config.script_type, "node");
        assert_eq!(loaded_config.runtime_version, Some("18.0.0".to_string()));
    }

    #[test]
    fn test_config_manager_env_vars() {
        let manager = ConfigManager::new();

        // 设置环境变量
        env::set_var("AI00_RUN_NAME", "env_test");
        env::set_var("AI00_RUN_SCRIPT_TYPE", "python");
        env::set_var("AI00_RUN_SCRIPT_PATH", "env_script.py");
        env::set_var("AI00_RUN_VENV_PATH", ".venv");
        env::set_var("AI00_RUN_TIMEOUT", "3000");
        env::set_var("AI00_RUN_ASYNC_EXECUTION", "false");

        // 从环境变量创建配置
        let config = manager.from_env_vars("AI00_RUN");

        assert_eq!(config.name, "env_test");
        assert_eq!(config.script_type, "python");
        assert_eq!(config.script_path, PathBuf::from("env_script.py"));
        assert_eq!(config.venv_path, Some(PathBuf::from(".venv")));
        assert_eq!(config.timeout, Some(3000));
        assert_eq!(config.async_execution, Some(false));

        // 清理环境变量
        env::remove_var("AI00_RUN_NAME");
        env::remove_var("AI00_RUN_SCRIPT_TYPE");
        env::remove_var("AI00_RUN_SCRIPT_PATH");
        env::remove_var("AI00_RUN_VENV_PATH");
        env::remove_var("AI00_RUN_TIMEOUT");
        env::remove_var("AI00_RUN_ASYNC_EXECUTION");
    }
}
