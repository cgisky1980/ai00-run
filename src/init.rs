//! 库初始化模块
//!
//! 负责库初始化时的版本检查和自动下载功能。

use crate::error::{Error, Result};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

/// 初始化管理器
#[derive(Debug)]
pub struct InitManager {
    /// 库安装目录
    install_dir: PathBuf,
    /// uv缓存目录
    uv_cache_dir: PathBuf,
}

impl Default for InitManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InitManager {
    /// 创建新的初始化管理器
    pub fn new() -> Self {
        let install_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from(".ai00-run"))
            .join("ai00-run");

        let uv_cache_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("uv-cache");

        Self {
            install_dir,
            uv_cache_dir,
        }
    }

    /// 使用自定义配置创建初始化管理器
    pub fn with_config(install_dir: PathBuf, uv_cache_dir: PathBuf) -> Self {
        Self {
            install_dir,
            uv_cache_dir,
        }
    }

    /// 获取安装目录
    pub fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }

    /// 获取uv缓存目录
    pub fn uv_cache_dir(&self) -> &PathBuf {
        &self.uv_cache_dir
    }

    /// 初始化库
    pub async fn init(&self) -> Result<()> {
        println!("Initializing ai00-run library...");

        // 1. 创建安装目录
        self.create_install_dirs().await?;

        // 2. 检查并下载uv
        self.check_and_download_uv().await?;

        // 3. 检查Python版本管理状态
        self.check_python_management_status().await?;

        // 4. 检查Node.js版本管理状态
        self.check_node_management_status().await?;

        println!("ai00-run library initialized successfully!");
        Ok(())
    }

    /// 创建安装目录
    async fn create_install_dirs(&self) -> Result<()> {
        // 创建主安装目录
        if !self.install_dir.exists() {
            fs::create_dir_all(&self.install_dir).await?;
            println!("Created installation directory: {:?}", self.install_dir);
        }

        // 创建uv缓存目录
        if !self.uv_cache_dir.exists() {
            fs::create_dir_all(&self.uv_cache_dir).await?;
            println!("Created uv cache directory: {:?}", self.uv_cache_dir);
        }

        // 创建Python安装目录
        let python_dir = self.install_dir.join("python");
        if !python_dir.exists() {
            fs::create_dir_all(&python_dir).await?;
            println!("Created Python installation directory: {:?}", python_dir);
        }

        // 创建Node.js安装目录
        let node_dir = self.install_dir.join("node");
        if !node_dir.exists() {
            fs::create_dir_all(&node_dir).await?;
            println!("Created Node.js installation directory: {:?}", node_dir);
        }

        Ok(())
    }

    /// 检查并下载uv
    async fn check_and_download_uv(&self) -> Result<()> {
        // 首先检查系统是否已安装uv
        match self.check_system_uv().await {
            Ok(version) => {
                println!("Using system UV: {}", version);
                return Ok(());
            }
            Err(e) => {
                println!("System UV not available: {}", e);
                println!("Will try to install UV...");
            }
        }

        // 如果系统没有uv，则使用安装脚本安装
        self.install_uv_with_script().await?;

        // 验证安装的uv
        let uv_exe_path = self.uv_cache_dir.join("uv.exe");
        match self.get_uv_version(&uv_exe_path).await {
            Ok(version) => {
                println!("Successfully installed UV: {}", version);
                Ok(())
            }
            Err(e) => Err(Error::InitializationFailed(format!(
                "Failed to verify installed UV: {}",
                e
            ))),
        }
    }

    /// 检查系统是否已安装uv
    async fn check_system_uv(&self) -> Result<String> {
        let output = Command::new("uv").arg("--version").output().map_err(|e| {
            Error::CommandExecutionFailed {
                command: "uv --version".to_string(),
                source: e,
            }
        })?;

        if output.status.success() {
            let version = String::from_utf8(output.stdout)?.trim().to_string();
            Ok(version)
        } else {
            Err(Error::CommandExecutionFailed {
                command: "uv --version".to_string(),
                source: std::io::Error::other(format!(
                    "System UV version check failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
            })
        }
    }

    /// 获取uv版本
    async fn get_uv_version(&self, uv_path: &PathBuf) -> Result<String> {
        let output = Command::new(uv_path)
            .arg("--version")
            .output()
            .map_err(|e| Error::CommandExecutionFailed {
                command: "uv --version".to_string(),
                source: e,
            })?;

        if output.status.success() {
            let version = String::from_utf8(output.stdout)?.trim().to_string();
            Ok(version)
        } else {
            Err(Error::CommandExecutionFailed {
                command: "uv --version".to_string(),
                source: std::io::Error::other(format!(
                    "UV version check failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
            })
        }
    }

    /// 使用安装脚本安装uv
    async fn install_uv_with_script(&self) -> Result<()> {
        let uv_exe_path = self.uv_cache_dir.join("uv.exe");

        // 检查是否已经有uv.exe文件
        if uv_exe_path.exists() {
            println!("UV executable already exists, skipping installation");
            return Ok(());
        }

        println!("Installing UV using installation script...");

        // 根据操作系统选择不同的安装方式
        if cfg!(windows) {
            self.install_uv_windows().await
        } else if cfg!(unix) {
            self.install_uv_unix().await
        } else {
            Err(Error::InitializationFailed(
                "Unsupported operating system for UV installation".to_string(),
            ))
        }
    }

    /// Windows系统安装uv
    async fn install_uv_windows(&self) -> Result<()> {
        let uv_exe_path = self.uv_cache_dir.join("uv.exe");

        // 方法1: 尝试从项目目录复制uv.exe
        let project_uv_path = std::env::current_dir()?.join("uv.exe");
        if project_uv_path.exists() {
            fs::copy(&project_uv_path, &uv_exe_path).await?;
            println!("Copied UV executable from project directory");
            return Ok(());
        }

        // 方法2: 尝试从uv目录复制
        let uv_dir_path = std::env::current_dir()?.join("uv").join("uv.exe");
        if uv_dir_path.exists() {
            fs::copy(&uv_dir_path, &uv_exe_path).await?;
            println!("Copied UV executable from uv directory");
            return Ok(());
        }

        // 方法3: 使用PowerShell安装脚本
        println!("Downloading UV using PowerShell script...");
        let script = r#"
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            $url = "https://astral.sh/uv/install.ps1"
            $installer = Invoke-WebRequest -Uri $url -UseBasicParsing
            Invoke-Expression $installer.Content
        "#;

        let status = Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .status()
            .map_err(|e| Error::CommandExecutionFailed {
                command: "PowerShell UV installation".to_string(),
                source: e,
            })?;

        if status.success() {
            // 检查是否安装成功
            let system_uv_check = Command::new("uv").arg("--version").output().map_err(|e| {
                Error::CommandExecutionFailed {
                    command: "uv --version".to_string(),
                    source: e,
                }
            })?;

            if system_uv_check.status.success() {
                println!("UV installed successfully via PowerShell script");
                return Ok(());
            }
        }

        Err(Error::InitializationFailed(
            "Failed to install UV using PowerShell script".to_string(),
        ))
    }

    /// Unix系统安装uv
    async fn install_uv_unix(&self) -> Result<()> {
        let _uv_exe_path = self.uv_cache_dir.join("uv");

        // 使用curl安装脚本
        println!("Downloading UV using curl script...");
        let status = Command::new("curl")
            .arg("-LsSf")
            .arg("https://astral.sh/uv/install.sh")
            .arg("|")
            .arg("sh")
            .status()
            .map_err(|e| Error::CommandExecutionFailed {
                command: "curl UV installation".to_string(),
                source: e,
            })?;

        if status.success() {
            // 检查是否安装成功
            let system_uv_check = Command::new("uv").arg("--version").output().map_err(|e| {
                Error::CommandExecutionFailed {
                    command: "uv --version".to_string(),
                    source: e,
                }
            })?;

            if system_uv_check.status.success() {
                println!("UV installed successfully via curl script");
                return Ok(());
            }
        }

        Err(Error::InitializationFailed(
            "Failed to install UV using curl script".to_string(),
        ))
    }

    /// 下载最新版uv（保留方法但标记为已弃用）
    async fn download_latest_uv(&self) -> Result<()> {
        // 这个方法现在被install_uv_with_script替代
        println!("Using new UV installation method...");
        self.install_uv_with_script().await
    }

    /// 检查Python版本管理状态
    async fn check_python_management_status(&self) -> Result<()> {
        let python_dir = self.install_dir.join("python");

        if python_dir.exists() {
            // 扫描已安装的Python版本
            let mut entries = fs::read_dir(&python_dir).await?;
            let mut installed_versions = Vec::new();

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if dir_name.starts_with("python-") {
                            let version = dir_name.trim_start_matches("python-");
                            installed_versions.push(version.to_string());
                        }
                    }
                }
            }

            if !installed_versions.is_empty() {
                println!(
                    "Found {} installed Python versions: {:?}",
                    installed_versions.len(),
                    installed_versions
                );
            } else {
                println!("No Python versions installed yet");
            }
        } else {
            println!("Python installation directory not found");
        }

        Ok(())
    }

    /// 检查Node.js版本管理状态
    async fn check_node_management_status(&self) -> Result<()> {
        let node_dir = self.install_dir.join("node");

        if node_dir.exists() {
            // 扫描已安装的Node.js版本
            let mut entries = fs::read_dir(&node_dir).await?;
            let mut installed_versions = Vec::new();

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if dir_name.starts_with("node-") {
                            let version = dir_name.trim_start_matches("node-");
                            installed_versions.push(version.to_string());
                        }
                    }
                }
            }

            if !installed_versions.is_empty() {
                println!(
                    "Found {} installed Node.js versions: {:?}",
                    installed_versions.len(),
                    installed_versions
                );
            } else {
                println!("No Node.js versions installed yet");
            }
        } else {
            println!("Node.js installation directory not found");
        }

        Ok(())
    }

    /// 获取库版本信息
    pub fn get_library_info(&self) -> LibraryInfo {
        LibraryInfo {
            name: crate::NAME.to_string(),
            version: crate::VERSION.to_string(),
            install_dir: self.install_dir.clone(),
            uv_cache_dir: self.uv_cache_dir.clone(),
        }
    }
}

/// 库信息结构
#[derive(Debug, Clone)]
pub struct LibraryInfo {
    /// 库名称
    pub name: String,
    /// 库版本
    pub version: String,
    /// 安装目录
    pub install_dir: PathBuf,
    /// uv缓存目录
    pub uv_cache_dir: PathBuf,
}

impl LibraryInfo {
    /// 格式化显示库信息
    pub fn display(&self) -> String {
        format!(
            "{} v{}\nInstallation Directory: {}\nUV Cache Directory: {}",
            self.name,
            self.version,
            self.install_dir.display(),
            self.uv_cache_dir.display()
        )
    }
}

/// 便捷函数：初始化库
pub async fn init() -> Result<()> {
    InitManager::new().init().await
}

/// 便捷函数：获取库信息
pub fn get_library_info() -> LibraryInfo {
    InitManager::new().get_library_info()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_init_manager_creation() {
        let manager = InitManager::new();
        assert!(manager.install_dir().to_string_lossy().contains("ai00-run"));
        assert!(manager
            .uv_cache_dir()
            .to_string_lossy()
            .contains("uv-cache"));
    }

    #[tokio::test]
    async fn test_init_manager_with_config() {
        let install_dir = temp_dir().join("test-init-manager");
        let uv_cache_dir = temp_dir().join("test-uv-cache");

        let manager = InitManager::with_config(install_dir.clone(), uv_cache_dir.clone());

        assert_eq!(manager.install_dir(), &install_dir);
        assert_eq!(manager.uv_cache_dir(), &uv_cache_dir);
    }

    #[tokio::test]
    async fn test_create_install_dirs() {
        let install_dir = temp_dir().join("test-create-dirs");
        let uv_cache_dir = temp_dir().join("test-uv-cache-dirs");

        let manager = InitManager::with_config(install_dir.clone(), uv_cache_dir.clone());

        // 确保目录不存在
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).await.unwrap();
        }
        if uv_cache_dir.exists() {
            fs::remove_dir_all(&uv_cache_dir).await.unwrap();
        }

        // 创建目录
        let result = manager.create_install_dirs().await;
        assert!(result.is_ok());

        // 检查目录是否创建成功
        assert!(install_dir.exists());
        assert!(uv_cache_dir.exists());
        assert!(install_dir.join("python").exists());
        assert!(install_dir.join("node").exists());
    }

    #[tokio::test]
    async fn test_get_library_info() {
        let info = get_library_info();

        assert_eq!(info.name, "ai00-run");
        assert_eq!(info.version, crate::VERSION);
        assert!(info.install_dir.to_string_lossy().contains("ai00-run"));
        assert!(info.uv_cache_dir.to_string_lossy().contains("uv-cache"));
    }

    #[test]
    fn test_library_info_display() {
        let info = LibraryInfo {
            name: "test-lib".to_string(),
            version: "1.0.0".to_string(),
            install_dir: PathBuf::from("/test/install"),
            uv_cache_dir: PathBuf::from("/test/uv-cache"),
        };

        let display = info.display();
        assert!(display.contains("test-lib v1.0.0"));
        assert!(display.contains("/test/install"));
        assert!(display.contains("/test/uv-cache"));
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        // 测试便捷函数（不实际执行初始化，因为需要uv.exe）
        let info = get_library_info();
        assert_eq!(info.name, "ai00-run");
    }
}
