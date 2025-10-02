//! Node.js 安装器模块
//!
//! 负责Node.js版本的下载、安装和管理，独立实现不依赖fnm。

use crate::error::{Error, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Node.js 版本信息结构体
#[derive(Debug, Deserialize, Clone)]
pub struct NodeVersionInfo {
    pub version: String,
    pub date: String,
    pub files: Vec<String>,
    pub npm: Option<String>,
    pub v8: Option<String>,
    pub uv: Option<String>,
    pub zlib: Option<String>,
    pub openssl: Option<String>,
    pub modules: Option<String>,
    #[serde(alias = "lts")]
    pub lts: Option<serde_json::Value>,
}

impl NodeVersionInfo {
    /// 检查是否为LTS版本
    pub fn is_lts(&self) -> bool {
        match &self.lts {
            Some(serde_json::Value::Bool(b)) => *b,
            Some(serde_json::Value::String(s)) => !s.is_empty() && s.to_lowercase() != "false",
            _ => false,
        }
    }
}

/// Node.js 安装器配置
pub struct NodeInstallerConfig {
    /// 镜像源URL
    pub mirror_url: String,
    /// 安装目录
    pub install_dir: PathBuf,
    /// 是否使用LTS版本
    pub prefer_lts: bool,
}

/// Node.js 安装器
pub struct NodeInstaller {
    /// 配置信息
    config: NodeInstallerConfig,
    /// HTTP客户端
    client: Client,
}

impl NodeInstaller {
    /// 创建新的Node.js安装器实例
    ///
    /// # 参数
    /// - `install_dir`: Node.js安装目录，如果为None则使用默认目录
    pub fn new(install_dir: Option<PathBuf>) -> Self {
        let install_dir = install_dir.unwrap_or_else(|| {
            // 默认安装目录：统一使用用户目录下的.ai00-run/node
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ai00-run")
                .join("node")
        });

        Self {
            config: NodeInstallerConfig {
                mirror_url: "https://nodejs.org/dist".to_string(),
                install_dir,
                prefer_lts: true,
            },
            client: Client::new(),
        }
    }

    /// 安装指定版本的Node.js
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回安装的Node.js可执行文件路径
    pub async fn install(&self, version: &str) -> Result<PathBuf> {
        println!("安装Node.js版本: {}", version);

        // 1. 验证版本号格式
        println!("验证版本号格式...");
        match validate_version(version) {
            Ok(_) => println!("版本号验证通过"),
            Err(e) => {
                println!("版本号验证失败: {}", e);
                return Err(e);
            }
        }

        // 2. 检查是否已安装
        println!("检查是否已安装...");
        if self.is_installed(version).await? {
            println!("Node.js {} 已经安装", version);
            return self.get_node_path(version).await;
        }
        println!("版本未安装，开始安装过程");

        println!("Installing Node.js {}...", version);

        // 3. 获取版本信息
        println!("获取版本信息...");
        let version_info = self.get_version_info(version).await?;
        println!("版本信息获取成功: {}", version_info.version);

        // 4. 下载Node.js二进制文件
        println!("获取下载URL...");
        let download_url = self.get_download_url(&version_info).await?;
        println!("下载URL: {}", download_url);

        println!("开始下载文件...");
        let download_path = self.download_file(&download_url, version).await?;
        println!("文件下载完成: {}", download_path.display());

        // 5. 提取文件到安装目录
        println!("开始提取文件...");
        let install_path = self.extract_archive(&download_path, version).await?;
        println!("文件提取完成: {}", install_path.display());

        // 6. 清理临时文件
        println!("清理临时文件...");
        fs::remove_file(&download_path).await?;

        println!(
            "Node.js {} installed successfully at {}",
            version,
            install_path.display()
        );

        Ok(install_path)
    }

    /// 获取指定版本的Node.js版本信息
    async fn get_version_info(&self, version: &str) -> Result<NodeVersionInfo> {
        let url = format!("{}/index.json", self.config.mirror_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to fetch version list: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        // 先获取文本内容，然后解析JSON
        let text = response
            .text()
            .await
            .map_err(|e| Error::Runtime(format!("Failed to read response text: {}", e)))?;

        // 使用serde_json解析JSON
        let versions: Vec<NodeVersionInfo> = serde_json::from_str(&text)
            .map_err(|e| Error::Runtime(format!("Failed to parse version list: {}", e)))?;

        // 查找匹配的版本
        let target_version = if version == "latest" {
            versions.first()
        } else if version == "lts" {
            versions.iter().find(|v| v.is_lts())
        } else {
            versions
                .iter()
                .find(|v| v.version == format!("v{}", version) || v.version == version)
        };

        target_version
            .cloned()
            .ok_or_else(|| Error::Runtime(format!("Version {} not found", version)))
    }

    /// 获取下载URL
    async fn get_download_url(&self, version_info: &NodeVersionInfo) -> Result<String> {
        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else {
            "arm64"
        };
        let platform = if cfg!(windows) {
            "win"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else {
            "linux"
        };

        let extension = if cfg!(windows) { "zip" } else { "tar.gz" };
        let filename = format!(
            "node-{}-{}-{}.{}",
            version_info.version, platform, arch, extension
        );

        Ok(format!(
            "{}/{}/{}",
            self.config.mirror_url, version_info.version, filename
        ))
    }

    /// 下载文件
    async fn download_file(&self, url: &str, version: &str) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let filename = url.split('/').last().unwrap_or("node.zip");
        let download_path = temp_dir.join(filename);

        println!("Downloading from: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to download: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| Error::Network(format!("Failed to read response: {}", e)))?;

        fs::write(&download_path, &content)
            .await
            .map_err(|e| Error::Runtime(format!("Failed to write file: {}", e)))?;

        Ok(download_path)
    }

    /// 提取归档文件
    async fn extract_archive(&self, archive_path: &PathBuf, version: &str) -> Result<PathBuf> {
        let version_dir = self.config.install_dir.join(format!("v{}", version));

        if cfg!(windows) {
            // Windows: 使用zip库处理zip文件（借鉴fnm的实现）
            // 注意：zip库操作是同步的，所以需要在tokio::task::spawn_blocking中执行
            let archive_path = archive_path.clone();
            let version_dir = version_dir.clone();

            let node_exe = tokio::task::spawn_blocking(move || {
                let file = std::fs::File::open(&archive_path)
                    .map_err(|e| Error::Runtime(format!("Failed to open archive: {}", e)))?;

                let mut archive = zip::ZipArchive::new(file)
                    .map_err(|e| Error::Runtime(format!("Failed to read zip archive: {}", e)))?;

                // 检查zip文件中的第一个条目，确定根目录名称
                let root_dir_name = if !archive.is_empty() {
                    let first_entry = archive.by_index(0).map_err(|e| {
                        Error::Runtime(format!("Failed to read first zip entry: {}", e))
                    })?;
                    let name = first_entry.name();
                    if name.contains('/') {
                        name.split('/').next().unwrap_or("").to_string()
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                };

                // 确保目标目录存在
                std::fs::create_dir_all(&version_dir).map_err(|e| {
                    Error::Runtime(format!("Failed to create version directory: {}", e))
                })?;

                // 提取所有文件
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).map_err(|e| {
                        Error::Runtime(format!("Failed to read zip entry {}: {}", i, e))
                    })?;

                    let mut outpath = version_dir.join(file.name());

                    // 如果zip文件有根目录，需要去掉根目录部分
                    if !root_dir_name.is_empty() && file.name().starts_with(&root_dir_name) {
                        let relative_path = if file.name().len() > root_dir_name.len() + 1 {
                            &file.name()[root_dir_name.len() + 1..] // +1 是为了去掉斜杠
                        } else {
                            continue; // 跳过根目录本身
                        };
                        if relative_path.is_empty() {
                            continue; // 跳过根目录本身
                        }
                        // 将正斜杠转换为当前系统的路径分隔符
                        let relative_path =
                            relative_path.replace('/', std::path::MAIN_SEPARATOR_STR);
                        outpath = version_dir.join(relative_path);
                    }

                    if file.name().ends_with('/') {
                        // 创建目录
                        std::fs::create_dir_all(&outpath).map_err(|e| {
                            Error::Runtime(format!(
                                "Failed to create directory {}: {}",
                                outpath.display(),
                                e
                            ))
                        })?;
                    } else {
                        // 创建文件
                        if let Some(parent) = outpath.parent() {
                            if !parent.exists() {
                                std::fs::create_dir_all(parent).map_err(|e| {
                                    Error::Runtime(format!(
                                        "Failed to create parent directory: {}",
                                        e
                                    ))
                                })?;
                            }
                        }

                        let mut outfile = std::fs::File::create(&outpath).map_err(|e| {
                            Error::Runtime(format!(
                                "Failed to create file {}: {}",
                                outpath.display(),
                                e
                            ))
                        })?;

                        let mut content = Vec::new();
                        file.read_to_end(&mut content).map_err(|e| {
                            Error::Runtime(format!("Failed to read zip file content: {}", e))
                        })?;

                        std::io::copy(&mut content.as_slice(), &mut outfile).map_err(|e| {
                            Error::Runtime(format!(
                                "Failed to write file {}: {}",
                                outpath.display(),
                                e
                            ))
                        })?;
                    }
                }

                // 查找node可执行文件
                let root_node = version_dir.join("node.exe");
                let bin_node = version_dir.join("bin").join("node.exe");

                if root_node.exists() {
                    Ok(root_node)
                } else if bin_node.exists() {
                    Ok(bin_node)
                } else {
                    Err(Error::Runtime(format!(
                        "Node executable not found in {}",
                        version_dir.display()
                    )))
                }
            })
            .await
            .map_err(|e| Error::Runtime(format!("Failed to extract archive: {}", e)))??;

            Ok(node_exe)
        } else {
            // Unix: 使用tar库处理tar.gz文件
            let file = std::fs::File::open(archive_path)
                .map_err(|e| Error::Runtime(format!("Failed to open archive: {}", e)))?;

            let tar = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(tar);

            archive
                .unpack(&version_dir)
                .map_err(|e| Error::Runtime(format!("Failed to extract tar archive: {}", e)))?;

            // 查找node可执行文件
            let node_exe = version_dir.join("bin").join("node");
            if node_exe.exists() {
                Ok(node_exe)
            } else {
                Err(Error::Runtime(format!(
                    "Node executable not found in {}",
                    version_dir.display()
                )))
            }
        }
    }

    /// 检查指定版本的Node.js是否已安装
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回布尔值表示是否已安装
    pub async fn is_installed(&self, version: &str) -> Result<bool> {
        let version_dir = self.config.install_dir.join(format!("v{}", version));

        if !version_dir.exists() {
            return Ok(false);
        }

        // 检查node可执行文件是否存在
        let node_exe = if cfg!(windows) {
            version_dir.join("node.exe")
        } else {
            version_dir.join("bin").join("node")
        };

        Ok(node_exe.exists())
    }

    /// 卸载指定版本的Node.js
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    pub async fn uninstall(&self, version: &str) -> Result<()> {
        // TODO: 实现Node.js版本卸载逻辑
        // 删除指定版本的安装目录

        println!("Uninstalling Node.js version: {}", version);
        Ok(())
    }

    /// 获取已安装的Node.js版本列表
    ///
    /// # 返回值
    /// 返回已安装版本的字符串列表
    pub async fn list_installed(&self) -> Result<Vec<String>> {
        if !self.config.install_dir.exists() {
            return Ok(vec![]);
        }

        let mut versions = Vec::new();

        let mut entries = fs::read_dir(&self.config.install_dir)
            .await
            .map_err(|e| Error::Runtime(format!("Failed to read install directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::Runtime(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('v') {
                        // 检查node可执行文件是否存在
                        let node_exe = if cfg!(windows) {
                            path.join("node.exe")
                        } else {
                            path.join("bin").join("node")
                        };

                        if node_exe.exists() {
                            versions.push(dir_name.to_string());
                        }
                    }
                }
            }
        }

        Ok(versions)
    }

    /// 获取指定版本的Node.js可执行文件路径
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回Node.js可执行文件的完整路径
    pub async fn get_node_path(&self, version: &str) -> Result<PathBuf> {
        let version_dir = self.config.install_dir.join(format!("v{}", version));

        if !version_dir.exists() {
            return Err(Error::Runtime(format!(
                "Node.js version {} is not installed",
                version
            )));
        }

        let node_exe = if cfg!(windows) {
            version_dir.join("node.exe")
        } else {
            version_dir.join("bin").join("node")
        };

        if !node_exe.exists() {
            return Err(Error::Runtime(format!(
                "Node.js executable not found at: {}",
                node_exe.display()
            )));
        }

        Ok(node_exe)
    }

    /// 获取安装目录
    pub fn install_dir(&self) -> &PathBuf {
        &self.config.install_dir
    }

    /// 设置安装目录
    pub fn set_install_dir(&mut self, install_dir: PathBuf) {
        self.config.install_dir = install_dir;
    }

    /// 设置镜像源URL
    pub fn set_mirror_url(&mut self, mirror_url: String) {
        self.config.mirror_url = mirror_url;
    }
}

/// 版本号验证函数
pub fn validate_version(version: &str) -> Result<()> {
    // 简单的版本号格式验证
    if version.is_empty() {
        return Err(Error::Version("Version cannot be empty".to_string()));
    }

    // 检查是否为特殊版本标识（如lts、latest等）
    let special_versions = ["lts", "latest", "current", "stable"];
    if special_versions.contains(&version.to_lowercase().as_str()) {
        return Ok(());
    }

    // 检查是否为有效的语义化版本号
    let version_parts: Vec<&str> = version.split('.').collect();
    if version_parts.len() < 3 {
        return Err(Error::Version(format!(
            "Invalid version format: {}",
            version
        )));
    }

    for part in version_parts {
        if part.is_empty() {
            return Err(Error::Version(format!(
                "Invalid version format: {}",
                version
            )));
        }

        // 检查是否为数字或包含预发布标识
        if !part
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c.is_ascii_alphabetic())
        {
            return Err(Error::Version(format!(
                "Invalid version format: {}",
                version
            )));
        }
    }

    Ok(())
}

/// 版本比较函数
pub fn compare_versions(version1: &str, version2: &str) -> Result<std::cmp::Ordering> {
    // TODO: 实现版本比较逻辑
    // 使用语义化版本比较算法

    Ok(version1.cmp(version2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_node_installer_creation() {
        let temp_dir = temp_dir();
        let installer = NodeInstaller::new(Some(temp_dir));

        assert!(installer.install_dir().exists());
    }

    #[tokio::test]
    async fn test_install_node() {
        let temp_dir = temp_dir();
        let installer = NodeInstaller::new(Some(temp_dir));

        let result = installer.install("18.0.0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_installed() {
        let temp_dir = temp_dir();
        let installer = NodeInstaller::new(Some(temp_dir));

        let result = installer.is_installed("18.0.0").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_version() {
        // 测试有效版本号
        assert!(validate_version("18.0.0").is_ok());
        assert!(validate_version("16.20.2").is_ok());
        assert!(validate_version("lts").is_ok());
        assert!(validate_version("latest").is_ok());

        // 测试无效版本号
        assert!(validate_version("").is_err());
        assert!(validate_version("18.0").is_err());
        assert!(validate_version("18..0").is_err());
    }

    #[test]
    fn test_compare_versions() {
        let result = compare_versions("18.0.0", "16.0.0");
        assert!(result.is_ok());

        // 注意：当前实现只是简单的字符串比较
        // 后续需要实现语义化版本比较
    }
}
