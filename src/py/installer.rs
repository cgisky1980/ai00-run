//! Python 安装器模块
//!
//! 负责Python版本的下载、安装和管理，基于uv和uvx实现。

use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, Result};

/// Python 安装器配置
#[derive(Debug, Clone)]
pub struct PyInstallerConfig {
    /// 安装目录
    pub install_dir: PathBuf,
    /// 默认Python版本
    pub default_version: String,
    /// 是否启用缓存
    pub enable_cache: bool,
}

/// Python 安装器
#[derive(Debug)]
pub struct PyInstaller {
    config: PyInstallerConfig,
}

impl Default for PyInstaller {
    fn default() -> Self {
        Self::new()
    }
}

impl PyInstaller {
    /// 创建新的Python安装器
    pub fn new() -> Self {
        let config = PyInstallerConfig {
            install_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ai00-run")
                .join("py"),
            default_version: "3.11".to_string(),
            enable_cache: true,
        };
        Self { config }
    }

    /// 使用自定义配置创建Python安装器
    pub fn with_config(config: PyInstallerConfig) -> Self {
        Self { config }
    }

    /// 获取安装器配置
    pub fn config(&self) -> &PyInstallerConfig {
        &self.config
    }

    /// 检查uv是否已安装
    async fn check_uv_installed() -> bool {
        match Command::new("uv").arg("--version").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// 安装uv
    async fn install_uv() -> Result<()> {
        println!("Installing uv...");

        if cfg!(windows) {
            // Windows安装命令
            let output = Command::new("powershell")
                .args([
                    "-ExecutionPolicy",
                    "ByPass",
                    "-c",
                    "irm https://astral.sh/uv/install.ps1 | iex",
                ])
                .output()
                .map_err(|e| {
                    Error::Runtime(format!("Failed to execute uv installation command: {}", e))
                })?;

            if output.status.success() {
                println!("uv installed successfully");
                Ok(())
            } else {
                let error_output = String::from_utf8_lossy(&output.stderr);
                Err(Error::Runtime(format!(
                    "Failed to install uv: {}",
                    error_output
                )))
            }
        } else {
            // macOS/Linux安装命令
            let output = Command::new("sh")
                .args(["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"])
                .output()
                .map_err(|e| {
                    Error::Runtime(format!("Failed to execute uv installation command: {}", e))
                })?;

            if output.status.success() {
                println!("uv installed successfully");
                Ok(())
            } else {
                let error_output = String::from_utf8_lossy(&output.stderr);
                Err(Error::Runtime(format!(
                    "Failed to install uv: {}",
                    error_output
                )))
            }
        }
    }

    /// 确保uv已安装
    pub async fn ensure_uv_installed() -> Result<()> {
        if !Self::check_uv_installed().await {
            Self::install_uv().await?;
        }
        Ok(())
    }

    /// 安装指定版本的Python
    pub async fn install_python(&self, version: &str) -> Result<()> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        // 验证Python版本号
        validate_python_version(version)?;

        println!("Installing Python {}...", version);

        // 使用uv python install命令安装Python
        let output = Command::new("uv")
            .arg("python")
            .arg("install")
            .arg(version)
            .output()
            .map_err(|e| {
                Error::Runtime(format!(
                    "Failed to execute uv python install command: {}",
                    e
                ))
            })?;

        if output.status.success() {
            println!("Python {} installed successfully", version);
            Ok(())
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr);
            Err(Error::Runtime(format!(
                "Failed to install Python {}: {}",
                version, error_output
            )))
        }
    }

    /// 列出所有已安装的Python版本
    pub async fn list_installed_versions(&self) -> Result<Vec<String>> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        // 使用uv python list命令获取已安装版本
        let output = Command::new("uv")
            .arg("python")
            .arg("list")
            .output()
            .map_err(|e| {
                Error::Runtime(format!("Failed to execute uv python list command: {}", e))
            })?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut versions = Vec::new();

            // 解析uv的输出，提取已安装的Python版本
            for line in output_str.lines() {
                // 已安装的版本会有具体的可执行文件路径，而不是"<download available>"
                if line.contains(".exe") && line.contains("cpython") {
                    // 提取版本号，例如从"cpython-3.11.13-windows-x86_64-none"中提取"3.11.13"
                    if let Some(version_start) = line.find("cpython-") {
                        let version_part = &line[version_start + 8..]; // 跳过"cpython-"
                        if let Some(version_end) = version_part.find('-') {
                            let version = &version_part[..version_end];
                            versions.push(version.to_string());
                        }
                    }
                }
            }

            Ok(versions)
        } else {
            Err(Error::Runtime(
                "Failed to list installed Python versions".to_string(),
            ))
        }
    }

    /// 列出所有可用的Python版本
    pub async fn list_available_versions(&self) -> Result<Vec<String>> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        // 使用uv python list命令获取可用版本
        let output = Command::new("uv")
            .arg("python")
            .arg("list")
            .output()
            .map_err(|e| {
                Error::Runtime(format!("Failed to execute uv python list command: {}", e))
            })?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut versions = Vec::new();

            // 解析uv的输出，提取所有Python版本
            for line in output_str.lines() {
                if line.contains("cpython-") {
                    // 提取版本号，例如从"cpython-3.11.13-windows-x86_64-none"中提取"3.11.13"
                    if let Some(version_start) = line.find("cpython-") {
                        let version_part = &line[version_start + 8..]; // 跳过"cpython-"
                        if let Some(version_end) = version_part.find('-') {
                            let version = &version_part[..version_end];
                            versions.push(version.to_string());
                        }
                    }
                }
            }

            Ok(versions)
        } else {
            Err(Error::Runtime(
                "Failed to list available Python versions".to_string(),
            ))
        }
    }

    /// 检查指定版本的Python是否已安装
    pub async fn is_version_installed(&self, version: &str) -> Result<bool> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        // 使用uv python list命令检查版本是否已安装
        let output = Command::new("uv")
            .arg("python")
            .arg("list")
            .output()
            .map_err(|e| {
                Error::Runtime(format!("Failed to execute uv python list command: {}", e))
            })?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // 检查版本是否在已安装列表中（有具体的可执行文件路径）
            // 支持模糊匹配：例如"3.11"应该匹配"3.11.13"
            for line in output_str.lines() {
                if line.contains(".exe") && line.contains("cpython-") {
                    // 提取版本号，例如从"cpython-3.11.13-windows-x86_64-none"中提取"3.11.13"
                    if let Some(version_start) = line.find("cpython-") {
                        let version_part = &line[version_start + 8..]; // 跳过"cpython-"
                        if let Some(version_end) = version_part.find('-') {
                            let installed_version = &version_part[..version_end];
                            
                            // 检查是否匹配（支持模糊匹配）
                            if installed_version.starts_with(version) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            Ok(false)
        } else {
            Err(Error::Runtime(
                "Failed to check Python version installation".to_string(),
            ))
        }
    }

    /// 卸载指定版本的Python
    pub async fn uninstall_python(&self, version: &str) -> Result<()> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        println!("Uninstalling Python {}...", version);

        // 使用uv python uninstall命令卸载Python
        let output = Command::new("uv")
            .arg("python")
            .arg("uninstall")
            .arg(version)
            .output()
            .map_err(|e| {
                Error::Runtime(format!(
                    "Failed to execute uv python uninstall command: {}",
                    e
                ))
            })?;

        if output.status.success() {
            println!("Python {} uninstalled successfully", version);
            Ok(())
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr);
            Err(Error::Runtime(format!(
                "Failed to uninstall Python {}: {}",
                version, error_output
            )))
        }
    }
}

/// Python版本号验证函数
pub fn validate_python_version(version: &str) -> Result<()> {
    // 简单的Python版本号格式验证
    if version.is_empty() {
        return Err(Error::Runtime("Version cannot be empty".to_string()));
    }

    // 检查是否为有效的Python版本号格式 (如 3.11.4)
    let version_parts: Vec<&str> = version.split('.').collect();
    if version_parts.len() < 2 || version_parts.len() > 3 {
        return Err(Error::Runtime(format!(
            "Invalid Python version format: {}",
            version
        )));
    }

    // 验证主要版本号
    if version_parts[0] != "3" {
        return Err(Error::Runtime(format!(
            "Only Python 3.x versions are supported: {}",
            version
        )));
    }

    // 验证次要版本号
    if let Ok(minor) = version_parts[1].parse::<u32>() {
        if minor < 8 {
            return Err(Error::Runtime(format!(
                "Python version must be at least 3.8: {}",
                version
            )));
        }
    } else {
        return Err(Error::Runtime(format!(
            "Invalid minor version number: {}",
            version
        )));
    }

    Ok(())
}

/// 比较两个Python版本号
pub fn compare_python_versions(version1: &str, version2: &str) -> Result<std::cmp::Ordering> {
    // 验证两个版本号
    validate_python_version(version1)?;
    validate_python_version(version2)?;

    // 简单的版本比较逻辑
    let v1_parts: Vec<&str> = version1.split('.').collect();
    let v2_parts: Vec<&str> = version2.split('.').collect();

    // 比较主要版本
    if v1_parts[0] != v2_parts[0] {
        return Ok(v1_parts[0].cmp(v2_parts[0]));
    }

    // 比较次要版本
    let minor1 = v1_parts[1].parse::<u32>().unwrap();
    let minor2 = v2_parts[1].parse::<u32>().unwrap();
    if minor1 != minor2 {
        return Ok(minor1.cmp(&minor2));
    }

    // 比较修订版本（如果有）
    if v1_parts.len() > 2 && v2_parts.len() > 2 {
        let patch1 = v1_parts[2].parse::<u32>().unwrap();
        let patch2 = v2_parts[2].parse::<u32>().unwrap();
        return Ok(patch1.cmp(&patch2));
    }

    // 如果版本号长度不同，认为较长的版本号更大
    Ok(v1_parts.len().cmp(&v2_parts.len()))
}

/// 获取推荐的Python版本
pub fn get_recommended_python_version() -> String {
    // 推荐使用最新的稳定版本
    "3.11".to_string()
}

/// 检查Python版本是否受支持
pub fn is_python_version_supported(version: &str) -> Result<bool> {
    // 验证版本号格式
    validate_python_version(version)?;

    // 解析版本号
    let version_parts: Vec<&str> = version.split('.').collect();
    let minor = version_parts[1].parse::<u32>().unwrap();

    // 支持Python 3.8及以上版本
    Ok(minor >= 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_py_installer_creation() {
        let installer = PyInstaller::new();
        assert!(std::mem::size_of_val(&installer) > 0);
    }

    #[test]
    fn test_validate_python_version() {
        assert!(validate_python_version("3.8").is_ok());
        assert!(validate_python_version("3.11.4").is_ok());
        assert!(validate_python_version("2.7").is_err());
        assert!(validate_python_version("3.7").is_err());
        assert!(validate_python_version("invalid").is_err());
    }
}
