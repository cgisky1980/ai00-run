//! 智能检查机制模块
//!
//! 提供智能的运行时环境检查、依赖验证和配置验证功能。

use crate::error::{Error, Result};
use crate::node::NodeManager;
use crate::py::PyManager;
use std::collections::HashMap;
use std::path::PathBuf;

/// 检查结果
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// 检查项名称
    pub name: String,
    /// 检查是否通过
    pub passed: bool,
    /// 检查详情
    pub details: String,
    /// 建议操作
    pub suggestion: Option<String>,
}

impl CheckResult {
    /// 创建新的检查结果
    pub fn new(name: &str, passed: bool, details: &str) -> Self {
        Self {
            name: name.to_string(),
            passed,
            details: details.to_string(),
            suggestion: None,
        }
    }

    /// 设置建议操作
    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    /// 检查是否通过
    pub fn is_passed(&self) -> bool {
        self.passed
    }
}

/// 智能检查器
pub struct SmartChecker {
    /// Node.js管理器
    node_manager: NodeManager,
    /// Python管理器
    py_manager: PyManager,
}

impl SmartChecker {
    /// 创建新的智能检查器
    pub fn new() -> Self {
        Self {
            node_manager: NodeManager::new(),
            py_manager: PyManager::new(),
        }
    }

    /// 检查Node.js运行时环境
    pub async fn check_node_runtime(&self, version: Option<&str>) -> Result<CheckResult> {
        let target_version = version.unwrap_or("latest");

        // 检查Node.js是否已安装
        let is_installed = self.node_manager.is_installed(target_version).await?;

        if is_installed {
            Ok(CheckResult::new(
                "Node.js Runtime",
                true,
                &format!("Node.js {} is installed and ready", target_version),
            ))
        } else {
            Ok(CheckResult::new(
                "Node.js Runtime",
                false,
                &format!("Node.js {} is not installed", target_version),
            )
            .with_suggestion(&format!("Run: node install {}", target_version)))
        }
    }

    /// 检查Python运行时环境
    pub async fn check_python_runtime(&self, version: Option<&str>) -> Result<CheckResult> {
        let target_version = version.unwrap_or("3.11");

        // 检查Python是否已安装
        let is_installed = self
            .py_manager
            .installer()
            .is_version_installed(target_version)
            .await?;

        if is_installed {
            Ok(CheckResult::new(
                "Python Runtime",
                true,
                &format!("Python {} is installed and ready", target_version),
            ))
        } else {
            Ok(CheckResult::new(
                "Python Runtime",
                false,
                &format!("Python {} is not installed", target_version),
            )
            .with_suggestion(&format!("Run: python install {}", target_version)))
        }
    }

    /// 检查虚拟环境
    pub async fn check_virtual_environment(&self, venv_path: &str) -> Result<CheckResult> {
        let exists = self.py_manager.venv_exists(venv_path).await?;

        if exists {
            Ok(CheckResult::new(
                "Virtual Environment",
                true,
                &format!("Virtual environment at '{}' exists", venv_path),
            ))
        } else {
            Ok(CheckResult::new(
                "Virtual Environment",
                false,
                &format!("Virtual environment at '{}' does not exist", venv_path),
            )
            .with_suggestion(&format!("Run: python venv create {}", venv_path)))
        }
    }

    /// 检查脚本文件
    pub async fn check_script_file(&self, script_path: &str) -> Result<CheckResult> {
        let path = PathBuf::from(script_path);

        if !path.exists() {
            return Ok(CheckResult::new(
                "Script File",
                false,
                &format!("Script file '{}' does not exist", script_path),
            ));
        }

        if !path.is_file() {
            return Ok(CheckResult::new(
                "Script File",
                false,
                &format!("'{}' is not a file", script_path),
            ));
        }

        // 检查文件权限（在Windows上主要是检查文件是否可读）
        if let Ok(metadata) = std::fs::metadata(&path) {
            let permissions = metadata.permissions();

            // 在Windows上，主要检查文件是否可读
            if std::fs::read_to_string(&path).is_err() {
                return Ok(CheckResult::new(
                    "Script File",
                    false,
                    &format!("Script file '{}' is not readable", script_path),
                ));
            }
        }

        // 检查文件扩展名
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let valid_extensions = ["js", "py", "sh", "bat", "cmd", "ps1"];
        let is_valid_extension = valid_extensions.contains(&extension);

        if !is_valid_extension {
            return Ok(CheckResult::new(
                "Script File",
                false,
                &format!("File extension '{}' is not supported", extension),
            )
            .with_suggestion("Supported extensions: js, py, sh, bat, cmd, ps1"));
        }

        Ok(CheckResult::new(
            "Script File",
            true,
            &format!("Script file '{}' is valid", script_path),
        ))
    }

    /// 检查系统命令
    pub async fn check_system_command(&self, command: &str) -> Result<CheckResult> {
        use crate::run::executor::ScriptExecutor;

        let executor = ScriptExecutor::new();
        let exists = executor.command_exists(command).await;

        if exists {
            Ok(CheckResult::new(
                "System Command",
                true,
                &format!("Command '{}' is available", command),
            ))
        } else {
            Ok(CheckResult::new(
                "System Command",
                false,
                &format!("Command '{}' is not available", command),
            ))
        }
    }

    /// 检查依赖包
    pub async fn check_dependencies(
        &self,
        venv_path: &str,
        packages: &[&str],
    ) -> Result<Vec<CheckResult>> {
        let mut results = Vec::new();

        // 首先检查虚拟环境是否存在
        let venv_check = self.check_virtual_environment(venv_path).await?;
        if !venv_check.passed {
            results.push(venv_check);
            return Ok(results);
        }

        // 获取已安装的包列表
        let installed_packages = self.py_manager.list_packages(venv_path).await?;

        for package in packages {
            let is_installed = installed_packages.iter().any(|p| p == package);

            if is_installed {
                results.push(CheckResult::new(
                    &format!("Dependency: {}", package),
                    true,
                    &format!("Package '{}' is installed", package),
                ));
            } else {
                results.push(
                    CheckResult::new(
                        &format!("Dependency: {}", package),
                        false,
                        &format!("Package '{}' is not installed", package),
                    )
                    .with_suggestion(&format!("Run: python install {} {}", venv_path, package)),
                );
            }
        }

        Ok(results)
    }

    /// 综合检查脚本运行环境
    pub async fn check_script_environment(
        &self,
        script_type: &str,
        script_path: &str,
        runtime_version: Option<&str>,
        venv_path: Option<&str>,
        dependencies: Option<&[&str]>,
    ) -> Result<Vec<CheckResult>> {
        let mut results = Vec::new();

        // 检查脚本文件
        let script_check = self.check_script_file(script_path).await?;
        results.push(script_check);

        // 根据脚本类型检查运行时环境
        match script_type {
            "node" => {
                let runtime_check = self.check_node_runtime(runtime_version).await?;
                results.push(runtime_check);
            }
            "python" => {
                let runtime_check = self.check_python_runtime(runtime_version).await?;
                results.push(runtime_check);

                if let Some(venv) = venv_path {
                    let venv_check = self.check_virtual_environment(venv).await?;
                    results.push(venv_check);

                    // 检查依赖包
                    if let Some(deps) = dependencies {
                        let dep_checks = self.check_dependencies(venv, deps).await?;
                        results.extend(dep_checks);
                    }
                }
            }
            "shell" => {
                // 对于shell脚本，主要检查系统命令可用性
                let shell_check = self.check_system_command("sh").await?;
                results.push(shell_check);
            }
            _ => {
                results.push(CheckResult::new(
                    "Script Type",
                    false,
                    &format!("Unsupported script type: {}", script_type),
                ));
            }
        }

        Ok(results)
    }

    /// 生成检查报告
    pub fn generate_report(&self, results: &[CheckResult]) -> String {
        let mut report = String::new();

        report.push_str("=== Environment Check Report ===\n\n");

        let passed_count = results.iter().filter(|r| r.passed).count();
        let total_count = results.len();

        report.push_str(&format!(
            "Summary: {}/{} checks passed\n\n",
            passed_count, total_count
        ));

        for result in results {
            let status = if result.passed { "✓" } else { "✗" };
            report.push_str(&format!("{} {}: {}\n", status, result.name, result.details));

            if let Some(suggestion) = &result.suggestion {
                report.push_str(&format!("    Suggestion: {}\n", suggestion));
            }
        }

        if passed_count == total_count {
            report.push_str("\n✅ All checks passed! Environment is ready.\n");
        } else {
            report.push_str("\n⚠️  Some checks failed. Please fix the issues above.\n");
        }

        report
    }
}

impl Default for SmartChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_creation() {
        let result = CheckResult::new("Test Check", true, "Everything is fine")
            .with_suggestion("No action needed");

        assert_eq!(result.name, "Test Check");
        assert!(result.passed);
        assert_eq!(result.details, "Everything is fine");
        assert_eq!(result.suggestion, Some("No action needed".to_string()));
    }

    #[tokio::test]
    async fn test_smart_checker_creation() {
        let checker = SmartChecker::new();

        // 基本检查器创建测试
        assert!(checker.check_script_file("Cargo.toml").await.is_ok());
    }

    #[tokio::test]
    async fn test_script_file_check() {
        let checker = SmartChecker::new();

        // 检查存在的文件（使用有效的脚本扩展名）
        let result = checker.check_script_file("test_script.py").await.unwrap();
        assert!(result.passed);

        // 检查不存在的文件
        let result = checker
            .check_script_file("nonexistent_file.py")
            .await
            .unwrap();
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_system_command_check() {
        let checker = SmartChecker::new();

        // 检查存在的命令
        let result = checker.check_system_command("echo").await.unwrap();
        assert!(result.passed);

        // 检查不存在的命令
        let result = checker
            .check_system_command("nonexistent_command_12345")
            .await
            .unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_report_generation() {
        let checker = SmartChecker::new();

        let results = vec![
            CheckResult::new("Check 1", true, "Passed"),
            CheckResult::new("Check 2", false, "Failed"),
        ];

        let report = checker.generate_report(&results);

        assert!(report.contains("Summary: 1/2 checks passed"));
        assert!(report.contains("✓ Check 1: Passed"));
        assert!(report.contains("✗ Check 2: Failed"));
    }
}
