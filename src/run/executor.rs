//! 脚本执行器模块
//!
//! 负责实际执行脚本命令，处理进程管理和输出捕获。

use super::ScriptResult;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

/// 流式输出消息类型
#[derive(Debug, Clone)]
pub enum StreamMessage {
    /// 标准输出消息
    Stdout(String),
    /// 标准错误消息
    Stderr(String),
    /// 进程退出
    Exit(i32),
    /// 错误消息
    Error(String),
}

/// 流式执行器句柄
pub struct StreamExecutorHandle {
    /// 消息接收器
    pub receiver: mpsc::UnboundedReceiver<StreamMessage>,
    /// 进程句柄
    pub child_handle: Option<tokio::process::Child>,
    /// 进程ID
    pub child_id: Option<u32>,
}

impl StreamExecutorHandle {
    /// 等待进程结束
    pub async fn wait(&mut self) -> Result<ScriptResult> {
        // 在流式执行中，进程监控由异步任务处理
        // 这里只需要等待Exit消息
        let mut exit_code = None;

        while let Some(message) = self.receiver.recv().await {
            match message {
                StreamMessage::Exit(code) => {
                    exit_code = Some(code);
                    break;
                }
                StreamMessage::Error(err) => {
                    return Err(Error::Script(format!("Process error: {}", err)));
                }
                _ => {
                    // 忽略其他消息，继续等待Exit
                }
            }
        }

        if let Some(code) = exit_code {
            Ok(ScriptResult::new(code, String::new(), String::new(), 0))
        } else {
            Err(Error::Script("Process did not exit properly".to_string()))
        }
    }

    /// 终止进程
    pub async fn kill(&mut self) -> Result<()> {
        // 在流式执行中，我们无法直接终止进程
        // 因为进程句柄已经被移动到异步任务中
        // 这里只能发送一个错误消息
        Err(Error::Script(
            "Kill operation not supported in streaming mode".to_string(),
        ))
    }
}

/// 脚本执行器
pub struct ScriptExecutor;

impl ScriptExecutor {
    /// 创建新的脚本执行器实例
    pub fn new() -> Self {
        Self
    }

    /// 异步执行命令
    ///
    /// # 参数
    /// - `command`: 命令字符串
    /// - `env_vars`: 环境变量映射
    /// - `working_dir`: 工作目录
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn execute_command_async(
        &self,
        command: &str,
        env_vars: Option<HashMap<String, String>>,
        working_dir: Option<&str>,
    ) -> Result<ScriptResult> {
        let start_time = std::time::Instant::now();

        // 解析命令
        let (program, args) = Self::parse_command(command);

        // 创建异步命令
        let mut cmd = AsyncCommand::new(program);

        // 设置参数
        for arg in args {
            cmd.arg(arg);
        }

        // 设置工作目录
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // 设置环境变量
        if let Some(envs) = env_vars {
            for (key, value) in envs {
                cmd.env(key, value);
            }
        }

        // 设置标准输入输出
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 执行命令
        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Script(format!("Failed to execute command: {}", e)))?;

        let execution_time = start_time.elapsed().as_millis();

        // 构建执行结果
        let result = ScriptResult::new(
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time.try_into().unwrap(),
        );

        Ok(result)
    }

    /// 异步执行命令（支持超时）
    ///
    /// # 参数
    /// - `command`: 命令字符串
    /// - `env_vars`: 环境变量映射
    /// - `working_dir`: 工作目录
    /// - `timeout_ms`: 超时时间（毫秒），None表示无超时
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub async fn execute_command_with_timeout(
        &self,
        command: &str,
        env_vars: Option<HashMap<String, String>>,
        working_dir: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<ScriptResult> {
        let start_time = std::time::Instant::now();

        // 解析命令
        let (program, args) = Self::parse_command(command);

        // 创建异步命令
        let mut cmd = AsyncCommand::new(program);

        // 设置参数
        for arg in args {
            cmd.arg(arg);
        }

        // 设置工作目录
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // 设置环境变量
        if let Some(envs) = env_vars {
            for (key, value) in envs {
                cmd.env(key, value);
            }
        }

        // 设置标准输入输出
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 执行命令（带超时）
        let output = if let Some(timeout_ms) = timeout_ms {
            let duration = Duration::from_millis(timeout_ms);
            timeout(duration, cmd.output())
                .await
                .map_err(|_| Error::Script("Command execution timeout".to_string()))?
                .map_err(|e| Error::Script(format!("Failed to execute command: {}", e)))?
        } else {
            // 无超时限制
            cmd.output()
                .await
                .map_err(|e| Error::Script(format!("Failed to execute command: {}", e)))?
        };

        let execution_time = start_time.elapsed().as_millis();

        // 构建执行结果
        let result = ScriptResult::new(
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time.try_into().unwrap(),
        );

        Ok(result)
    }

    /// 流式执行命令（支持长期运行和实时输出）
    ///
    /// # 参数
    /// - `command`: 命令字符串
    /// - `env_vars`: 环境变量映射
    /// - `working_dir`: 工作目录
    ///
    /// # 返回值
    /// 返回流式执行器句柄
    pub async fn execute_command_stream(
        &self,
        command: &str,
        env_vars: Option<HashMap<String, String>>,
        working_dir: Option<&str>,
    ) -> Result<StreamExecutorHandle> {
        // 解析命令
        let (program, args) = Self::parse_command(command);

        // 创建异步命令
        let mut cmd = AsyncCommand::new(program);

        // 设置参数
        for arg in args {
            cmd.arg(arg);
        }

        // 设置工作目录
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // 设置环境变量
        if let Some(envs) = env_vars {
            for (key, value) in envs {
                cmd.env(key, value);
            }
        }

        // 设置标准输入输出
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 启动进程
        let mut child = cmd
            .spawn()
            .map_err(|e| Error::Script(format!("Failed to spawn process: {}", e)))?;

        // 创建消息通道
        let (sender, receiver) = mpsc::unbounded_channel();

        // 获取stdout和stderr句柄
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // 启动stdout读取任务
        if let Some(stdout) = stdout {
            let sender = sender.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if sender.send(StreamMessage::Stdout(line)).is_err() {
                        break;
                    }
                }
            });
        }

        // 启动stderr读取任务
        if let Some(stderr) = stderr {
            let sender = sender.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if sender.send(StreamMessage::Stderr(line)).is_err() {
                        break;
                    }
                }
            });
        }

        // 获取子进程的ID
        let child_id = child.id();

        // 启动进程监控任务
        let sender_clone = sender.clone();

        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    let _ = sender_clone.send(StreamMessage::Exit(status.code().unwrap_or(-1)));
                }
                Err(e) => {
                    let _ = sender_clone
                        .send(StreamMessage::Error(format!("Process wait error: {}", e)));
                }
            }
        });

        Ok(StreamExecutorHandle {
            receiver,
            child_handle: None, // 由于child被移动到异步任务中，这里设为None
            child_id,
        })
    }

    /// 同步执行命令
    ///
    /// # 参数
    /// - `command`: 命令字符串
    /// - `env_vars`: 环境变量映射
    /// - `working_dir`: 工作目录
    ///
    /// # 返回值
    /// 返回脚本执行结果
    pub fn execute_command_sync(
        &self,
        command: &str,
        env_vars: Option<HashMap<String, String>>,
        working_dir: Option<&str>,
    ) -> Result<ScriptResult> {
        let start_time = std::time::Instant::now();

        // 解析命令
        let (program, args) = Self::parse_command(command);

        // 创建同步命令
        let mut cmd = Command::new(program);

        // 设置参数
        for arg in args {
            cmd.arg(arg);
        }

        // 设置工作目录
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // 设置环境变量
        if let Some(envs) = env_vars {
            for (key, value) in envs {
                cmd.env(key, value);
            }
        }

        // 设置标准输入输出
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 执行命令
        let output = cmd
            .output()
            .map_err(|e| Error::Script(format!("Failed to execute command: {}", e)))?;

        let execution_time = start_time.elapsed().as_millis();

        // 构建执行结果
        let result = ScriptResult::new(
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time.try_into().unwrap(),
        );

        Ok(result)
    }

    /// 解析命令字符串
    ///
    /// # 参数
    /// - `command`: 命令字符串
    ///
    /// # 返回值
    /// 返回(程序路径, 参数列表)
    fn parse_command(command: &str) -> (String, Vec<String>) {
        // 简单的命令解析，支持带引号的参数
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;
        let mut quote_char = None;

        for ch in command.chars() {
            match ch {
                '"' | '\'' => {
                    if in_quotes {
                        if Some(ch) == quote_char {
                            in_quotes = false;
                            quote_char = None;
                            if !current_part.is_empty() {
                                parts.push(current_part.clone());
                                current_part.clear();
                            }
                        } else {
                            current_part.push(ch);
                        }
                    } else {
                        in_quotes = true;
                        quote_char = Some(ch);
                        if !current_part.is_empty() {
                            parts.push(current_part.clone());
                            current_part.clear();
                        }
                    }
                }
                ' ' if !in_quotes => {
                    if !current_part.is_empty() {
                        parts.push(current_part.clone());
                        current_part.clear();
                    }
                }
                _ => {
                    current_part.push(ch);
                }
            }
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        if parts.is_empty() {
            (String::new(), Vec::new())
        } else {
            let program = parts[0].clone();
            let args = parts[1..].to_vec();
            (program, args)
        }
    }

    /// 检查命令是否存在
    ///
    /// # 参数
    /// - `command`: 命令名称
    ///
    /// # 返回值
    /// 返回布尔值表示命令是否存在
    pub async fn command_exists(&self, command: &str) -> bool {
        let (program, _) = Self::parse_command(command);

        if cfg!(windows) {
            // Windows下使用where命令检查
            let output = Command::new("where").arg(&program).output().ok();

            output.map(|o| o.status.success()).unwrap_or(false)
        } else {
            // Unix下使用which命令检查
            let output = Command::new("which").arg(&program).output().ok();

            output.map(|o| o.status.success()).unwrap_or(false)
        }
    }

    /// 获取命令的完整路径
    ///
    /// # 参数
    /// - `command`: 命令名称
    ///
    /// # 返回值
    /// 返回命令的完整路径
    pub async fn get_command_path(&self, command: &str) -> Option<String> {
        let (program, _) = Self::parse_command(command);

        if cfg!(windows) {
            // Windows下使用where命令
            let output = Command::new("where").arg(&program).output().ok()?;

            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        } else {
            // Unix下使用which命令
            let output = Command::new("which").arg(&program).output().ok()?;

            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        }
    }
}

impl Default for ScriptExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 命令执行选项
#[derive(Debug, Clone)]
pub struct ExecuteOptions {
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 工作目录
    pub working_dir: Option<String>,
    /// 超时时间（毫秒）
    pub timeout: Option<u64>,
    /// 是否异步执行
    pub async_execution: bool,
}

impl Default for ExecuteOptions {
    fn default() -> Self {
        Self {
            env_vars: HashMap::new(),
            working_dir: None,
            timeout: None,
            async_execution: true,
        }
    }
}

impl ExecuteOptions {
    /// 创建新的执行选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置环境变量
    pub fn with_env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置工作目录
    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }

    /// 设置异步执行
    pub fn with_async_execution(mut self, async_exec: bool) -> Self {
        self.async_execution = async_exec;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_script_executor_creation() {
        let executor = ScriptExecutor::new();
        assert!(executor.command_exists("echo").await);
    }

    #[tokio::test]
    async fn test_parse_command() {
        let (program, args) = ScriptExecutor::parse_command("echo \"hello world\"");
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello world"]);

        let (program2, args2) = ScriptExecutor::parse_command("ls -la /tmp");
        assert_eq!(program2, "ls");
        assert_eq!(args2, vec!["-la", "/tmp"]);
    }

    #[tokio::test]
    async fn test_command_exists() {
        let executor = ScriptExecutor::new();

        // 检查常见命令是否存在
        assert!(executor.command_exists("echo").await);

        // 检查不存在的命令
        assert!(!executor.command_exists("nonexistent_command_12345").await);
    }

    #[tokio::test]
    async fn test_get_command_path() {
        let executor = ScriptExecutor::new();

        // 获取echo命令的路径
        let path = executor.get_command_path("echo").await;
        assert!(path.is_some());

        // 获取不存在的命令的路径
        let path = executor.get_command_path("nonexistent_command_12345").await;
        assert!(path.is_none());
    }

    #[tokio::test]
    async fn test_execute_command_async() {
        let executor = ScriptExecutor::new();

        // 执行简单的echo命令
        let result = executor
            .execute_command_async("echo hello", None, None)
            .await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.is_success());
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_execute_command_sync() {
        let executor = ScriptExecutor::new();

        // 执行简单的echo命令
        let result = executor.execute_command_sync("echo hello", None, None);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.is_success());
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_execute_options() {
        let options = ExecuteOptions::new()
            .with_env_var("TEST", "value")
            .with_working_dir("/tmp")
            .with_timeout(5000)
            .with_async_execution(false);

        assert_eq!(options.env_vars.get("TEST"), Some(&"value".to_string()));
        assert_eq!(options.working_dir, Some("/tmp".to_string()));
        assert_eq!(options.timeout, Some(5000));
        assert!(!options.async_execution);
    }
}
