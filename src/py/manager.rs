//! Python 管理器模块
//!
//! 负责Python虚拟环境的管理、激活和包管理，基于uv和uvx实现。

use crate::error::{Error, Result};
use std::path::PathBuf;
use std::process::Command;

/// Python 管理器配置
#[derive(Debug, Clone)]
pub struct PyManagerConfig {
    /// Python 安装目录
    pub python_dir: PathBuf,
    /// 虚拟环境目录
    pub venv_dir: PathBuf,
    /// 是否使用系统Python
    pub use_system_python: bool,
    /// 是否优先使用虚拟环境
    pub prefer_venv: bool,
}

/// Python 管理器
#[derive(Debug)]
pub struct PyManager {
    config: PyManagerConfig,
    current_venv: Option<PathBuf>,
}

impl PyManager {
    /// 创建Python管理器实例
    pub fn new(python_dir: PathBuf, venv_dir: PathBuf) -> Self {
        let config = PyManagerConfig {
            python_dir,
            venv_dir,
            use_system_python: true,
            prefer_venv: true,
        };

        Self {
            config,
            current_venv: None,
        }
    }

    /// 使用指定配置创建Python管理器实例
    pub fn with_config(config: PyManagerConfig) -> Self {
        Self {
            config,
            current_venv: None,
        }
    }

    /// 获取管理器配置
    pub fn config(&self) -> &PyManagerConfig {
        &self.config
    }

    /// 设置管理器配置
    pub fn set_config(&mut self, config: PyManagerConfig) {
        self.config = config;
    }

    /// 获取当前激活的虚拟环境路径
    pub fn current_venv(&self) -> Option<&PathBuf> {
        self.current_venv.as_ref()
    }

    /// 设置当前激活的虚拟环境路径
    pub fn set_current_venv(&mut self, venv_path: Option<PathBuf>) {
        self.current_venv = venv_path;
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
    async fn ensure_uv_installed() -> Result<()> {
        if !Self::check_uv_installed().await {
            Self::install_uv().await?;
        }
        Ok(())
    }

    /// 查找可用的Python版本
    pub async fn find_python_versions(&self, version_request: Option<&str>) -> Result<Vec<String>> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        // 使用uv python list命令查找Python版本
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

            // 解析uv的输出，提取Python版本
            for line in output_str.lines() {
                if line.contains("Python") && line.contains("version") {
                    if let Some(version) = line.split_whitespace().find(|s| s.starts_with("3.")) {
                        versions.push(version.to_string());
                    }
                }
            }

            Ok(versions)
        } else {
            Err(Error::Runtime("Failed to list Python versions".to_string()))
        }
    }

    /// 安装指定版本的Python
    pub async fn install_python(&self, version: &str) -> Result<()> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

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

    /// 创建虚拟环境
    pub async fn create_venv(
        &mut self,
        venv_path: &str,
        python_version: Option<&str>,
    ) -> Result<()> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        let venv_path = PathBuf::from(venv_path);

        // 确保虚拟环境目录存在
        if let Some(parent) = venv_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(Error::Io)?;
        }

        // 构建uv venv命令
        let mut command = Command::new("uv");
        command.arg("venv");

        if let Some(version) = python_version {
            command.arg("--python").arg(format!("python{}", version));
        }

        command.arg(&venv_path);

        // 创建虚拟环境
        let output = command
            .output()
            .map_err(|e| Error::Runtime(format!("Failed to create virtual environment: {}", e)))?;

        if output.status.success() {
            println!("Created virtual environment at: {}", venv_path.display());
            Ok(())
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr);
            Err(Error::Runtime(format!(
                "Failed to create virtual environment: {}",
                error_output
            )))
        }
    }

    /// 激活虚拟环境
    pub async fn activate_venv(&mut self, venv_path: &str) -> Result<()> {
        let venv_path = PathBuf::from(venv_path);

        // 验证虚拟环境是否存在
        if !venv_path.exists() {
            return Err(Error::Runtime(format!(
                "Virtual environment does not exist: {}",
                venv_path.display()
            )));
        }

        println!("Activating virtual environment at: {}", venv_path.display());
        self.current_venv = Some(venv_path);

        Ok(())
    }

    /// 停用虚拟环境
    pub async fn deactivate_venv(&mut self) -> Result<()> {
        println!("Deactivating virtual environment");
        self.current_venv = None;
        Ok(())
    }

    /// 检查虚拟环境是否存在
    pub async fn venv_exists(&self, venv_path: &str) -> Result<bool> {
        let venv_path = PathBuf::from(venv_path);
        Ok(venv_path.exists())
    }

    /// 安装Python包
    pub async fn install_packages(&mut self, venv_path: &str, packages: &[&str]) -> Result<()> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        let venv_path = PathBuf::from(venv_path);

        // 验证虚拟环境是否存在
        if !venv_path.exists() {
            return Err(Error::Runtime(format!(
                "Virtual environment does not exist: {}",
                venv_path.display()
            )));
        }

        println!(
            "Installing packages {:?} to virtual environment at: {}",
            packages,
            venv_path.display()
        );

        // 使用uvx安装包到指定虚拟环境
        let output = Command::new("uvx")
            .arg("--python")
            .arg(&venv_path)
            .arg("pip")
            .arg("install")
            .args(packages)
            .output()
            .map_err(|e| Error::Runtime(format!("Failed to install packages: {}", e)))?;

        if output.status.success() {
            println!("Successfully installed packages");
            Ok(())
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr);
            Err(Error::Runtime(format!(
                "Failed to install packages: {}",
                error_output
            )))
        }
    }

    /// 卸载Python包
    pub async fn uninstall_packages(&mut self, venv_path: &str, packages: &[&str]) -> Result<()> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        let venv_path = PathBuf::from(venv_path);

        // 验证虚拟环境是否存在
        if !venv_path.exists() {
            return Err(Error::Runtime(format!(
                "Virtual environment does not exist: {}",
                venv_path.display()
            )));
        }

        println!(
            "Uninstalling packages {:?} from virtual environment at: {}",
            packages,
            venv_path.display()
        );

        // 使用uvx卸载包
        let output = Command::new("uvx")
            .arg("--python")
            .arg(&venv_path)
            .arg("pip")
            .arg("uninstall")
            .arg("-y") // 自动确认卸载
            .args(packages)
            .output()
            .map_err(|e| Error::Runtime(format!("Failed to uninstall packages: {}", e)))?;

        if output.status.success() {
            println!("Successfully uninstalled packages");
            Ok(())
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr);
            Err(Error::Runtime(format!(
                "Failed to uninstall packages: {}",
                error_output
            )))
        }
    }

    /// 列出已安装的包
    pub async fn list_packages(&self, venv_path: &str) -> Result<Vec<String>> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        let venv_path = PathBuf::from(venv_path);

        // 验证虚拟环境是否存在
        if !venv_path.exists() {
            return Err(Error::Runtime(format!(
                "Virtual environment does not exist: {}",
                venv_path.display()
            )));
        }

        // 使用uvx列出包
        let output = Command::new("uvx")
            .arg("--python")
            .arg(&venv_path)
            .arg("pip")
            .arg("list")
            .output()
            .map_err(|e| Error::Runtime(format!("Failed to list packages: {}", e)))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut packages = Vec::new();

            // 解析pip list的输出，跳过标题行
            for line in output_str.lines().skip(2) {
                // 跳过前两行标题
                if let Some(package_name) = line.split_whitespace().next() {
                    packages.push(package_name.to_string());
                }
            }

            Ok(packages)
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr);
            Err(Error::Runtime(format!(
                "Failed to list packages: {}",
                error_output
            )))
        }
    }

    /// 获取Python版本
    pub async fn get_python_version(&self, venv_path: &str) -> Result<String> {
        // 确保uv已安装
        Self::ensure_uv_installed().await?;

        let venv_path = PathBuf::from(venv_path);

        // 验证虚拟环境是否存在
        if !venv_path.exists() {
            return Err(Error::Runtime(format!(
                "Virtual environment does not exist: {}",
                venv_path.display()
            )));
        }

        // 使用uvx获取Python版本
        let output = Command::new("uvx")
            .arg("--python")
            .arg(&venv_path)
            .arg("python")
            .arg("--version")
            .output()
            .map_err(|e| Error::Runtime(format!("Failed to get Python version: {}", e)))?;

        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);

            // 解析版本输出，例如 "Python 3.11.7" -> "3.11.7"
            if let Some(version) = version_output.trim().strip_prefix("Python ") {
                Ok(version.to_string())
            } else {
                Ok(version_output.trim().to_string())
            }
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr);
            Err(Error::Runtime(format!(
                "Failed to get Python version: {}",
                error_output
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_py_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let python_dir = temp_dir.path().join("python");
        let venv_dir = temp_dir.path().join("venv");

        let manager = PyManager::new(python_dir, venv_dir);
        assert_eq!(manager.config().use_system_python, true);
        assert_eq!(manager.config().prefer_venv, true);
    }

    #[tokio::test]
    async fn test_py_manager_with_config() {
        let temp_dir = tempdir().unwrap();
        let config = PyManagerConfig {
            python_dir: temp_dir.path().join("python"),
            venv_dir: temp_dir.path().join("venv"),
            use_system_python: false,
            prefer_venv: false,
        };

        let manager = PyManager::with_config(config);
        assert_eq!(manager.config().use_system_python, false);
        assert_eq!(manager.config().prefer_venv, false);
    }
}
