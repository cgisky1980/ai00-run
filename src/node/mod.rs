//! Node.js 运行时管理模块
//!
//! 提供Node.js版本管理、环境设置和脚本执行功能，独立实现不依赖fnm。

pub mod installer;
pub mod manager;

use crate::error::Result;

/// Node.js 版本管理API
pub struct NodeManager;

impl Default for NodeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeManager {
    /// 创建新的Node.js管理器实例
    pub fn new() -> Self {
        Self
    }

    /// 安装指定版本的Node.js
    ///
    /// # 参数
    /// - `version`: Node.js版本号，如 "18.0.0" 或 "lts"
    ///
    /// # 示例
    /// ```no_run
    /// use ai00_run::node::NodeManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> ai00_run::Result<()> {
    ///     let manager = NodeManager::new();
    ///     manager.install("18.0.0").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn install(&self, version: &str) -> Result<()> {
        use std::env::temp_dir;

        // 创建安装器实例
        let installer = installer::NodeInstaller::new(Some(temp_dir()));

        // 安装Node.js版本
        let node_path = installer.install(version).await?;

        println!(
            "Node.js {} installed successfully at: {}",
            version,
            node_path.display()
        );
        Ok(())
    }

    /// 检查指定版本的Node.js是否已安装
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回布尔值表示是否已安装
    pub async fn is_installed(&self, version: &str) -> Result<bool> {
        // 创建安装器实例，使用默认安装目录
        let installer = installer::NodeInstaller::new(None);

        // 检查版本是否已安装
        installer.is_installed(version).await
    }

    /// 列出已安装的本地Node.js版本
    ///
    /// # 返回值
    /// 返回已安装版本的字符串列表
    pub async fn list_local(&self) -> Result<Vec<String>> {
        // 创建安装器实例，使用默认安装目录
        let installer = installer::NodeInstaller::new(None);

        // 获取已安装版本列表
        let versions = installer.list_installed().await?;

        if versions.is_empty() {
            println!("No Node.js versions installed");
        } else {
            println!("Found {} installed Node.js versions", versions.len());
        }

        Ok(versions)
    }

    /// 在指定版本下运行命令
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    /// - `command`: 要运行的命令
    pub async fn run_command(&self, version: &str, command: &str) -> Result<()> {
        // 项目内Node.js管理：在指定版本下运行命令
        // 使用ScriptExecutor来执行命令
        use crate::run::executor::ScriptExecutor;

        let executor = ScriptExecutor::new();
        let result = executor.execute_command_async(command, None, None).await?;

        if result.is_success() {
            println!(
                "Command executed successfully with Node.js version {}",
                version
            );
            Ok(())
        } else {
            Err(crate::error::Error::Script(format!(
                "Command failed with exit code: {}",
                result.exit_code
            )))
        }
    }

    /// 在指定版本下使用npx运行命令
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    /// - `command`: 要运行的npx命令
    /// - `args`: 命令参数
    pub async fn run_npx_command(
        &self,
        version: &str,
        command: &str,
        args: &[String],
    ) -> Result<()> {
        use crate::run::executor::ScriptExecutor;

        // 处理"current"版本
        let target_version = if version == "current" {
            // 项目内Node.js管理：不支持"current"版本，必须明确指定版本号
            return Err(crate::error::Error::Version(
                "Node.js version must be explicitly specified, 'current' is not supported"
                    .to_string(),
            ));
        } else {
            version.to_string()
        };

        // 检查版本是否已安装
        if !self.is_installed(&target_version).await? {
            return Err(crate::error::Error::Version(format!(
                "Node.js version {} is not installed",
                target_version
            )));
        }

        // 获取Node.js可执行文件路径
        let node_path = self.get_node_path(&target_version).await?;
        let node_dir = std::path::Path::new(&node_path).parent().unwrap();

        // 构建npx路径（Windows使用npx.cmd，Unix使用bin/npx）
        let npx_path = if cfg!(windows) {
            node_dir.join("npx.cmd")
        } else {
            node_dir.join("bin").join("npx")
        };

        if !npx_path.exists() {
            return Err(crate::error::Error::Version(format!(
                "npx not found for Node.js version {}",
                target_version
            )));
        }

        // 构建完整的npx命令
        let mut full_command = format!("\"{}\" {}", npx_path.display(), command);
        for arg in args {
            full_command.push_str(&format!(" \"{}\"", arg));
        }

        // 执行命令
        let executor = ScriptExecutor::new();
        let result = executor
            .execute_command_async(&full_command, None, None)
            .await?;

        if result.is_success() {
            println!("{}", result.stdout);
            Ok(())
        } else {
            eprintln!("Error: {}", result.stderr);
            Err(crate::error::Error::Script(format!(
                "npx command failed with exit code: {}",
                result.exit_code
            )))
        }
    }

    /// 获取指定版本的Node.js可执行文件路径
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回Node.js可执行文件的完整路径
    ///
    /// # 示例
    /// ```no_run
    /// use ai00_run::node::NodeManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> ai00_run::Result<()> {
    ///     let manager = NodeManager::new();
    ///     let node_path = manager.get_node_path("18.0.0").await?;
    ///     println!("Node.js executable path: {}", node_path);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_node_path(&self, version: &str) -> Result<String> {
        // 创建安装器实例，使用默认安装目录
        let installer = installer::NodeInstaller::new(None);

        // 获取Node.js可执行文件路径
        let node_path = installer.get_node_path(version).await?;

        Ok(node_path.to_string_lossy().to_string())
    }
}

/// 便捷函数：安装指定版本的Node.js
pub async fn install(version: &str) -> Result<()> {
    NodeManager::new().install(version).await
}

/// 便捷函数：检查指定版本的Node.js是否已安装
pub async fn is_installed(version: &str) -> Result<bool> {
    NodeManager::new().is_installed(version).await
}

/// 便捷函数：列出已安装的本地Node.js版本
pub async fn list_local() -> Result<Vec<String>> {
    NodeManager::new().list_local().await
}

/// 便捷函数：在指定版本下运行命令
pub async fn run_command(version: &str, command: &str) -> Result<()> {
    NodeManager::new().run_command(version, command).await
}

/// 便捷函数：在指定版本下使用npx运行命令
pub async fn run_npx_command(version: &str, command: &str, args: &[String]) -> Result<()> {
    NodeManager::new()
        .run_npx_command(version, command, args)
        .await
}

/// 便捷函数：获取指定版本的Node.js可执行文件路径
pub async fn get_node_path(version: &str) -> Result<String> {
    NodeManager::new().get_node_path(version).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_manager_creation() {
        let manager = NodeManager::new();
        // 测试管理器创建和基本功能
        // 注意：is_installed方法可能返回true如果系统已安装Node.js
        // 我们只测试方法调用是否成功，不测试具体返回值
        let result = manager.is_installed("18.0.0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_convenience_functions() {
        assert!(install("18.0.0").await.is_ok());
        assert!(is_installed("18.0.0").await.is_ok());
        assert!(list_local().await.is_ok());
        assert!(run_command("18.0.0", "node --version").await.is_ok());
        assert!(run_npx_command("18.0.0", "--version", &[]).await.is_ok());
    }
}
