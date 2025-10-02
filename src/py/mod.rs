//! Python 运行时管理模块
//!
//! 提供Python虚拟环境管理、依赖安装和脚本执行功能，通过复用uv实现。

pub mod installer;
pub mod manager;

use crate::error::Result;

/// Python 虚拟环境管理API
pub struct PyManager {
    installer: installer::PyInstaller,
}

impl Default for PyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PyManager {
    /// 创建新的Python管理器实例
    pub fn new() -> Self {
        Self {
            installer: installer::PyInstaller::new(),
        }
    }

    /// 使用自定义配置创建Python管理器实例
    pub fn with_config(config: installer::PyInstallerConfig) -> Self {
        Self {
            installer: installer::PyInstaller::with_config(config),
        }
    }

    /// 获取Python安装器实例
    pub fn installer(&self) -> &installer::PyInstaller {
        &self.installer
    }

    /// 创建Python虚拟环境
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径，默认为".venv"
    /// - `python_version`: Python版本号，如"3.11"
    ///
    /// # 示例
    /// ```no_run
    /// use ai00_run::py::PyManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> ai00_run::Result<()> {
    ///     let manager = PyManager::new();
    ///     manager.create_venv(None, Some("3.11")).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_venv(
        &self,
        path: Option<&str>,
        python_version: Option<&str>,
    ) -> Result<()> {
        let venv_path = path.unwrap_or(".venv");
        let version = python_version.unwrap_or("3.11");

        // 确保uv已安装
        installer::PyInstaller::ensure_uv_installed().await?;

        // 检查Python版本是否已安装，如果没有则安装
        if !self.installer.is_version_installed(version).await? {
            println!("Python version {} not found, installing...", version);
            self.installer.install_python(version).await?;
            // 安装成功后重新检查版本状态
            if !self.installer.is_version_installed(version).await? {
                return Err(crate::error::Error::PythonVersionNotFound(
                    version.to_string(),
                ));
            }
        }

        // 获取Python可执行文件路径
        // 注意：由于我们使用uv管理Python，路径由uv自动管理
        let python_path = format!("python{}", version);

        // 使用uv创建虚拟环境
        let status = tokio::process::Command::new("uv")
            .arg("venv")
            .arg(venv_path)
            .arg("--python")
            .arg(&python_path)
            .status()
            .await
            .map_err(|e| crate::error::Error::CommandExecutionFailed {
                command: "uv venv".to_string(),
                source: e,
            })?;

        if !status.success() {
            return Err(crate::error::Error::CommandExecutionFailed {
                command: "uv venv".to_string(),
                source: std::io::Error::other("Failed to create virtual environment with uv"),
            });
        }

        println!(
            "Successfully created Python virtual environment at {} with Python {}",
            venv_path, version
        );
        Ok(())
    }

    /// 检查虚拟环境是否存在
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径
    ///
    /// # 返回值
    /// 返回布尔值表示虚拟环境是否存在
    pub async fn venv_exists(&self, path: &str) -> Result<bool> {
        let venv_path = std::path::Path::new(path);

        // 检查虚拟环境目录是否存在
        if !venv_path.exists() {
            return Ok(false);
        }

        // 检查是否为有效的虚拟环境
        let pyvenv_cfg = venv_path.join("pyvenv.cfg");
        if !pyvenv_cfg.exists() {
            return Ok(false);
        }

        // 检查Python可执行文件是否存在
        let python_exe = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python3")
        };

        if !python_exe.exists() {
            return Ok(false);
        }

        // 额外检查：验证Python可执行文件是否可运行
        let check_result = std::process::Command::new(&python_exe)
            .arg("--version")
            .output();

        match check_result {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false), // 如果无法运行，则认为虚拟环境无效
        }
    }

    /// 激活虚拟环境
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径
    pub async fn activate_venv(&self, path: &str) -> Result<()> {
        // 检查虚拟环境是否存在
        if !self.venv_exists(path).await? {
            return Err(crate::error::Error::VirtualEnvironmentNotFound(
                path.to_string(),
            ));
        }

        println!("Activating virtual environment at: {}", path);

        // 获取虚拟环境中的Python可执行文件路径（虽然当前未使用，但保留以备将来需要）
        let _python_path = self.get_python_path_in_venv(path).await?;

        // 设置环境变量以激活虚拟环境
        let venv_path = std::path::Path::new(path);

        if cfg!(windows) {
            // Windows: 设置PATH环境变量
            let scripts_path = venv_path.join("Scripts");
            if let Some(current_path) = std::env::var_os("PATH") {
                let mut paths: Vec<_> = std::env::split_paths(&current_path).collect();
                paths.insert(0, scripts_path.clone());
                let new_path = std::env::join_paths(paths).map_err(|e| {
                    crate::error::Error::Runtime(format!("Failed to join PATH: {}", e))
                })?;
                std::env::set_var("PATH", new_path);
            }

            // 设置VIRTUAL_ENV环境变量
            std::env::set_var("VIRTUAL_ENV", venv_path);
        } else {
            // Unix-like: 设置PATH环境变量
            let bin_path = venv_path.join("bin");
            if let Some(current_path) = std::env::var_os("PATH") {
                let mut paths: Vec<_> = std::env::split_paths(&current_path).collect();
                paths.insert(0, bin_path.clone());
                let new_path = std::env::join_paths(paths).map_err(|e| {
                    crate::error::Error::Runtime(format!("Failed to join PATH: {}", e))
                })?;
                std::env::set_var("PATH", new_path);
            }

            // 设置VIRTUAL_ENV环境变量
            std::env::set_var("VIRTUAL_ENV", venv_path);
        }

        // 设置Python可执行文件路径
        std::env::set_var("PYTHONHOME", "");
        std::env::set_var("PYTHONPATH", "");

        println!("Successfully activated virtual environment at {}", path);
        Ok(())
    }

    /// 停用虚拟环境
    pub async fn deactivate_venv(&self) -> Result<()> {
        println!("Deactivating virtual environment");

        // 清除虚拟环境相关的环境变量
        std::env::remove_var("VIRTUAL_ENV");
        std::env::remove_var("PYTHONHOME");
        std::env::remove_var("PYTHONPATH");

        // 恢复原始的PATH环境变量（移除虚拟环境的路径）
        if let Some(current_path) = std::env::var_os("PATH") {
            let paths: Vec<_> = std::env::split_paths(&current_path).collect();

            // 过滤掉虚拟环境的路径
            let filtered_paths: Vec<_> = paths
                .into_iter()
                .filter(|path| {
                    // 检查路径是否包含虚拟环境的典型目录结构
                    let path_str = path.to_string_lossy().to_lowercase();
                    !(path_str.contains("scripts") && path_str.contains("venv")
                        || path_str.contains("bin") && path_str.contains("venv"))
                })
                .collect();

            if !filtered_paths.is_empty() {
                let new_path = std::env::join_paths(filtered_paths).map_err(|e| {
                    crate::error::Error::Runtime(format!("Failed to join PATH: {}", e))
                })?;
                std::env::set_var("PATH", new_path);
            }
        }

        println!("Successfully deactivated virtual environment");
        Ok(())
    }

    /// 安装Python包到虚拟环境
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径
    /// - `packages`: 要安装的包列表
    pub async fn install_packages(&self, path: &str, packages: &[&str]) -> Result<()> {
        // 确保uv已安装
        installer::PyInstaller::ensure_uv_installed().await?;

        // 检查虚拟环境是否存在
        if !self.venv_exists(path).await? {
            return Err(crate::error::Error::VirtualEnvironmentNotFound(
                path.to_string(),
            ));
        }

        if packages.is_empty() {
            return Ok(());
        }

        println!(
            "Installing packages {:?} to virtual environment at: {}",
            packages, path
        );

        // 使用uv pip install命令安装包
        let mut command = tokio::process::Command::new("uv");
        command.arg("pip");
        command.arg("install");

        // 添加包名
        for package in packages {
            command.arg(package);
        }

        // 指定虚拟环境路径
        command.arg("--python");
        command.arg(path);

        let status =
            command
                .status()
                .await
                .map_err(|e| crate::error::Error::CommandExecutionFailed {
                    command: format!("uv pip install {}", packages.join(" ")),
                    source: e,
                })?;

        if !status.success() {
            return Err(crate::error::Error::Runtime(format!(
                "Failed to install packages {:?} to virtual environment at {}",
                packages, path
            )));
        }

        println!(
            "Successfully installed packages {:?} to virtual environment at {}",
            packages, path
        );
        Ok(())
    }

    /// 卸载Python包
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径
    /// - `packages`: 要卸载的包列表
    pub async fn uninstall_packages(&self, path: &str, packages: &[&str]) -> Result<()> {
        // 确保uv已安装
        installer::PyInstaller::ensure_uv_installed().await?;

        // 检查虚拟环境是否存在
        if !self.venv_exists(path).await? {
            return Err(crate::error::Error::VirtualEnvironmentNotFound(
                path.to_string(),
            ));
        }

        if packages.is_empty() {
            return Ok(());
        }

        println!(
            "Uninstalling packages {:?} from virtual environment at: {}",
            packages, path
        );

        // 使用uv pip uninstall命令卸载包
        let mut command = tokio::process::Command::new("uv");
        command.arg("pip");
        command.arg("uninstall");

        // 添加包名
        for package in packages {
            command.arg(package);
        }

        // 指定虚拟环境路径
        command.arg("--python");
        command.arg(path);

        let status =
            command
                .status()
                .await
                .map_err(|e| crate::error::Error::CommandExecutionFailed {
                    command: format!("uv pip uninstall {}", packages.join(" ")),
                    source: e,
                })?;

        if !status.success() {
            return Err(crate::error::Error::Runtime(format!(
                "Failed to uninstall packages {:?} from virtual environment at {}",
                packages, path
            )));
        }

        println!(
            "Successfully uninstalled packages {:?} from virtual environment at {}",
            packages, path
        );
        Ok(())
    }

    /// 列出已安装的包
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径
    ///
    /// # 返回值
    /// 返回已安装包的字符串列表
    pub async fn list_packages(&self, path: &str) -> Result<Vec<String>> {
        // 确保uv已安装
        installer::PyInstaller::ensure_uv_installed().await?;

        // 检查虚拟环境是否存在
        if !self.venv_exists(path).await? {
            return Err(crate::error::Error::VirtualEnvironmentNotFound(
                path.to_string(),
            ));
        }

        println!("Listing packages in virtual environment at: {}", path);

        // 使用uv pip list命令获取包列表
        let output = tokio::process::Command::new("uv")
            .arg("pip")
            .arg("list")
            .arg("--format")
            .arg("freeze")
            .arg("--python")
            .arg(path)
            .output()
            .await
            .map_err(|e| crate::error::Error::CommandExecutionFailed {
                command: "uv pip list".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            return Err(crate::error::Error::Runtime(format!(
                "Failed to list packages in virtual environment at {}",
                path
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<String> = output_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        println!(
            "Found {} packages in virtual environment at {}",
            packages.len(),
            path
        );
        Ok(packages)
    }

    /// 在虚拟环境中运行Python脚本
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径
    /// - `script`: Python脚本内容或路径
    pub async fn run_script(&self, path: &str, script: &str) -> Result<()> {
        // 检查虚拟环境是否存在
        if !self.venv_exists(path).await? {
            return Err(crate::error::Error::VirtualEnvironmentNotFound(
                path.to_string(),
            ));
        }

        // 获取虚拟环境中的Python可执行文件路径
        let python_path = self.get_python_path_in_venv(path).await?;

        println!(
            "Running script in virtual environment at {}: {}",
            path, script
        );

        // 检查script是文件路径还是脚本内容
        let script_path = std::path::Path::new(script);

        if script_path.exists() && script_path.is_file() {
            // 如果是文件路径，直接运行文件
            let status = tokio::process::Command::new(&python_path)
                .arg(script)
                .status()
                .await
                .map_err(|e| crate::error::Error::CommandExecutionFailed {
                    command: format!("{} {}", python_path, script),
                    source: e,
                })?;

            if !status.success() {
                return Err(crate::error::Error::Runtime(format!(
                    "Failed to run script {} in virtual environment at {}",
                    script, path
                )));
            }
        } else {
            // 如果是脚本内容，创建临时文件并运行
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join("ai00_run_script.py");

            // 写入脚本内容到临时文件
            tokio::fs::write(&temp_file, script).await.map_err(|e| {
                crate::error::Error::Runtime(format!(
                    "Failed to create temporary script file: {}",
                    e
                ))
            })?;

            // 运行临时文件
            let status = tokio::process::Command::new(&python_path)
                .arg(&temp_file)
                .status()
                .await
                .map_err(|e| crate::error::Error::CommandExecutionFailed {
                    command: format!("{} {}", python_path, temp_file.display()),
                    source: e,
                })?;

            // 清理临时文件
            let _ = tokio::fs::remove_file(&temp_file).await;

            if !status.success() {
                return Err(crate::error::Error::Runtime(format!(
                    "Failed to run script in virtual environment at {}",
                    path
                )));
            }
        }

        println!("Successfully ran script in virtual environment at {}", path);
        Ok(())
    }

    /// 在虚拟环境中运行Python命令
    ///
    /// # 参数
    /// - `path`: 虚拟环境路径
    /// - `command`: Python命令
    pub async fn run_command(&self, path: &str, command: &str) -> Result<()> {
        // 检查虚拟环境是否存在
        if !self.venv_exists(path).await? {
            return Err(crate::error::Error::VirtualEnvironmentNotFound(
                path.to_string(),
            ));
        }

        // 获取虚拟环境中的Python可执行文件路径
        let python_path = self.get_python_path_in_venv(path).await?;

        println!(
            "Running command in virtual environment at {}: {}",
            path, command
        );

        // 使用-c参数运行Python命令
        let status = tokio::process::Command::new(&python_path)
            .arg("-c")
            .arg(command)
            .status()
            .await
            .map_err(|e| crate::error::Error::CommandExecutionFailed {
                command: format!("{} -c {}", python_path, command),
                source: e,
            })?;

        if !status.success() {
            return Err(crate::error::Error::Runtime(format!(
                "Failed to run command '{}' in virtual environment at {}",
                command, path
            )));
        }

        println!(
            "Successfully ran command in virtual environment at {}",
            path
        );
        Ok(())
    }

    /// 获取虚拟环境中Python可执行文件的路径
    ///
    /// # 参数
    /// - `venv_path`: 虚拟环境路径
    ///
    /// # 返回值
    /// 返回Python可执行文件的完整路径
    ///
    /// # 示例
    /// ```no_run
    /// use ai00_run::py::PyManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> ai00_run::Result<()> {
    ///     let manager = PyManager::new();
    ///     let python_path = manager.get_python_path_in_venv(".venv").await?;
    ///     println!("Python executable path: {}", python_path);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_python_path_in_venv(&self, venv_path: &str) -> Result<String> {
        let venv_path = std::path::Path::new(venv_path);

        // 检查虚拟环境是否存在
        if !self.venv_exists(venv_path.to_str().unwrap()).await? {
            return Err(crate::error::Error::VirtualEnvironmentNotFound(
                venv_path.to_string_lossy().to_string(),
            ));
        }

        // 根据操作系统确定Python可执行文件路径
        let python_exe = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python3")
        };

        // 检查Python可执行文件是否存在
        if !python_exe.exists() {
            return Err(crate::error::Error::PythonExecutableNotFound(
                python_exe.to_string_lossy().to_string(),
            ));
        }

        // 验证Python可执行文件是否可运行
        let check_result = std::process::Command::new(&python_exe)
            .arg("--version")
            .output()
            .map_err(|e| crate::error::Error::CommandExecutionFailed {
                command: format!("{} --version", python_exe.to_string_lossy()),
                source: e,
            })?;

        if !check_result.status.success() {
            return Err(crate::error::Error::PythonExecutableNotValid(
                python_exe.to_string_lossy().to_string(),
            ));
        }

        Ok(python_exe.to_string_lossy().to_string())
    }

    /// 获取指定版本的Python可执行文件路径
    ///
    /// # 参数
    /// - `version`: Python版本号，如"3.11"
    ///
    /// # 返回值
    /// 返回Python可执行文件的完整路径
    ///
    /// # 示例
    /// ```no_run
    /// use ai00_run::py::PyManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> ai00_run::Result<()> {
    ///     let manager = PyManager::new();
    ///     let python_path = manager.get_python_path("3.11").await?;
    ///     println!("Python executable path: {}", python_path);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_python_path(&self, version: &str) -> Result<String> {
        // 确保uv已安装
        installer::PyInstaller::ensure_uv_installed().await?;

        // 使用uv python which命令获取Python路径
        let output = std::process::Command::new("uv")
            .arg("python")
            .arg("which")
            .arg(version)
            .output()
            .map_err(|e| crate::error::Error::CommandExecutionFailed {
                command: format!("uv python which {}", version),
                source: e,
            })?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let path = output_str.trim();

            // 验证路径是否存在
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // 备用方案：检查系统Python
        let python_cmd = if cfg!(windows) { "python" } else { "python3" };

        // 检查系统Python版本
        let version_check = std::process::Command::new(python_cmd)
            .arg("--version")
            .output();

        if let Ok(output) = version_check {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains(version) {
                    // 返回系统Python路径
                    let which_output = std::process::Command::new("which")
                        .arg(python_cmd)
                        .output()
                        .map_err(|e| crate::error::Error::CommandExecutionFailed {
                            command: format!("which {}", python_cmd),
                            source: e,
                        })?;

                    if which_output.status.success() {
                        let path = String::from_utf8_lossy(&which_output.stdout)
                            .trim()
                            .to_string();
                        return Ok(path);
                    }
                }
            }
        }

        // 如果都找不到，返回错误
        Err(crate::error::Error::Runtime(format!(
            "Python version {} not found or not installed",
            version
        )))
    }
}

/// 便捷函数：创建Python虚拟环境
pub async fn create_venv(path: Option<&str>, python_version: Option<&str>) -> Result<()> {
    PyManager::new().create_venv(path, python_version).await
}

/// 便捷函数：检查虚拟环境是否存在
pub async fn venv_exists(path: &str) -> Result<bool> {
    PyManager::new().venv_exists(path).await
}

/// 便捷函数：激活虚拟环境
pub async fn activate_venv(path: &str) -> Result<()> {
    PyManager::new().activate_venv(path).await
}

/// 便捷函数：停用虚拟环境
pub async fn deactivate_venv() -> Result<()> {
    PyManager::new().deactivate_venv().await
}

/// 便捷函数：安装Python包
pub async fn install_packages(path: &str, packages: &[&str]) -> Result<()> {
    PyManager::new().install_packages(path, packages).await
}

/// 便捷函数：卸载Python包
pub async fn uninstall_packages(path: &str, packages: &[&str]) -> Result<()> {
    PyManager::new().uninstall_packages(path, packages).await
}

/// 便捷函数：列出已安装的包
pub async fn list_packages(path: &str) -> Result<Vec<String>> {
    PyManager::new().list_packages(path).await
}

/// 便捷函数：在虚拟环境中运行Python脚本
pub async fn run_script(path: &str, script: &str) -> Result<()> {
    PyManager::new().run_script(path, script).await
}

/// 便捷函数：在虚拟环境中运行Python命令
pub async fn run_command(path: &str, command: &str) -> Result<()> {
    PyManager::new().run_command(path, command).await
}

/// 便捷函数：获取虚拟环境中Python可执行文件的路径
pub async fn get_python_path_in_venv(venv_path: &str) -> Result<String> {
    PyManager::new().get_python_path_in_venv(venv_path).await
}

/// 便捷函数：获取指定版本的Python可执行文件路径
pub async fn get_python_path(version: &str) -> Result<String> {
    PyManager::new().get_python_path(version).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_py_manager_creation() {
        let manager = PyManager::new();
        // 检查配置是否正确设置
        assert_eq!(manager.installer().config().default_version, "3.11");
        assert!(manager.installer().config().enable_cache);
    }

    #[tokio::test]
    async fn test_py_manager_with_config() {
        let config = installer::PyInstallerConfig {
            install_dir: temp_dir().join("test-python-manager"),
            default_version: "3.11".to_string(),
            enable_cache: true,
        };
        let manager = PyManager::with_config(config);
        assert_eq!(manager.installer().config().default_version, "3.11");
        assert!(manager.installer().config().enable_cache);
    }

    #[tokio::test]
    async fn test_venv_exists() {
        let manager = PyManager::new();

        // 测试不存在的虚拟环境
        let result = manager.venv_exists("/nonexistent/path").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // 测试临时目录（应该不是虚拟环境）
        let temp_path = temp_dir().to_string_lossy().to_string();
        let result = manager.venv_exists(&temp_path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        // 测试便捷函数创建
        let manager = PyManager::new();
        assert!(manager.venv_exists(".venv").await.is_ok());

        // 其他便捷函数测试（这些目前还是TODO实现）
        assert!(activate_venv(".venv").await.is_ok());
        assert!(deactivate_venv().await.is_ok());
        assert!(install_packages(".venv", &["requests"]).await.is_ok());
        assert!(uninstall_packages(".venv", &["requests"]).await.is_ok());
        assert!(list_packages(".venv").await.is_ok());
        assert!(run_script(".venv", "print('hello')").await.is_ok());
        assert!(run_command(".venv", "import sys; print(sys.version)")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_create_venv_convenience_function() {
        // 测试便捷函数创建虚拟环境
        // 注意：这个测试可能需要实际安装Python，所以可能会失败
        let result = create_venv(None, Some("3.11")).await;
        // 由于需要实际安装Python，我们只检查函数调用是否成功（不检查实际结果）
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_python_path_in_venv() {
        let manager = PyManager::new();

        // 测试不存在的虚拟环境
        let result = manager.get_python_path_in_venv("/nonexistent/path").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::VirtualEnvironmentNotFound(_)
        ));

        // 测试临时目录（应该不是虚拟环境）
        let temp_path = temp_dir().to_string_lossy().to_string();
        let result = manager.get_python_path_in_venv(&temp_path).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::VirtualEnvironmentNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_get_python_path_in_venv_convenience_function() {
        // 测试便捷函数
        let result = get_python_path_in_venv("/nonexistent/path").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::VirtualEnvironmentNotFound(_)
        ));
    }
}
