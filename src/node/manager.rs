//! Node.js 管理器模块
//!
//! 负责Node.js版本的管理、切换和环境配置。
//! 借鉴fnm的设计模式，但保持独立的API接口。

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Node.js 管理器配置
#[derive(Debug, Clone)]
pub struct NodeManagerConfig {
    /// Node.js安装目录
    pub install_dir: PathBuf,
    /// 默认Node.js版本
    pub default_version: Option<String>,
    /// 是否自动切换版本
    pub auto_switch: bool,
    /// 版本别名映射
    pub aliases: HashMap<String, String>,
    /// 远程镜像源（借鉴fnm的镜像配置）
    pub mirror_url: Option<String>,
    /// 是否使用LTS版本作为默认
    pub use_lts_as_default: bool,
}

impl Default for NodeManagerConfig {
    fn default() -> Self {
        Self {
            install_dir: PathBuf::from("./ai00-run/runtimes/node"),
            default_version: None,
            auto_switch: true,
            aliases: HashMap::new(),
            mirror_url: Some("https://nodejs.org/dist".to_string()),
            use_lts_as_default: false,
        }
    }
}

/// Node.js 版本信息（借鉴fnm的版本解析）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeVersion {
    /// 语义化版本（如：18.17.1）
    Semver(String),
    /// LTS版本别名（如：lts/hydrogen）
    Lts(String),
    /// 版本别名（如：latest, stable）
    Alias(String),
    /// 精确版本号
    Exact(String),
}

impl NodeVersion {
    /// 解析版本字符串
    pub fn parse(version_str: &str) -> Result<Self> {
        if let Some(stripped) = version_str.strip_prefix("lts/") {
            Ok(NodeVersion::Lts(stripped.to_string()))
        } else if version_str == "latest" || version_str == "stable" {
            Ok(NodeVersion::Alias(version_str.to_string()))
        } else if version_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
            Ok(NodeVersion::Exact(version_str.to_string()))
        } else {
            // 尝试语义化版本解析
            Ok(NodeVersion::Semver(version_str.to_string()))
        }
    }
}

/// Node.js 管理器
pub struct NodeManager {
    /// 管理器配置
    config: NodeManagerConfig,
    /// 当前使用的版本
    current_version: Option<String>,
    /// 已安装的版本列表（缓存）
    installed_versions: Vec<String>,
    /// 远程版本列表（缓存）
    remote_versions: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_manager_creation() {
        let config = NodeManagerConfig::default();
        let manager = NodeManager::with_config(config);

        assert!(manager
            .config()
            .mirror_url
            .as_ref()
            .unwrap()
            .starts_with("https://"));
        println!("NodeManager创建测试通过");
    }

    #[tokio::test]
    async fn test_version_parsing() {
        // 测试Exact版本解析（纯数字和点号）
        let version = NodeVersion::parse("18.17.1").unwrap();
        assert!(matches!(version, NodeVersion::Exact(_)));

        // 测试LTS版本解析
        let lts_version = NodeVersion::parse("lts/hydrogen").unwrap();
        assert!(matches!(lts_version, NodeVersion::Lts(_)));

        // 测试别名解析
        let alias_version = NodeVersion::parse("latest").unwrap();
        assert!(matches!(alias_version, NodeVersion::Alias(_)));

        // 测试Semver版本解析（包含其他字符的版本）
        let semver_version = NodeVersion::parse("18.17.1-beta").unwrap();
        assert!(matches!(semver_version, NodeVersion::Semver(_)));

        println!("版本解析测试通过");
    }

    #[tokio::test]
    async fn test_list_installed_versions() {
        let mut manager = NodeManager::new();

        // 测试获取已安装版本（应该为空列表）
        let installed = manager.list_installed_versions().await.unwrap();
        assert!(installed.is_empty());

        println!("已安装版本列表测试通过");
    }

    #[tokio::test]
    async fn test_is_version_installed() {
        let manager = NodeManager::new();

        // 测试检查版本是否已安装（应该返回false）
        let is_installed = manager.is_version_installed("18.17.1").await.unwrap();
        assert!(!is_installed);

        println!("版本安装检查测试通过");
    }
}

impl NodeManager {
    /// 创建新的Node.js管理器实例
    pub fn new() -> Self {
        Self {
            config: NodeManagerConfig::default(),
            current_version: None,
            installed_versions: Vec::new(),
            remote_versions: None,
        }
    }

    /// 使用指定配置创建Node.js管理器实例
    pub fn with_config(config: NodeManagerConfig) -> Self {
        Self {
            config,
            current_version: None,
            installed_versions: Vec::new(),
            remote_versions: None,
        }
    }

    /// 获取管理器配置
    pub fn config(&self) -> &NodeManagerConfig {
        &self.config
    }

    /// 设置管理器配置
    pub fn set_config(&mut self, config: NodeManagerConfig) {
        self.config = config;
    }

    /// 获取当前使用的Node.js版本
    pub fn current_version(&self) -> Option<&str> {
        self.current_version.as_deref()
    }

    /// 设置当前使用的Node.js版本
    pub fn set_current_version(&mut self, version: Option<String>) {
        self.current_version = version;
    }

    /// 切换到指定版本的Node.js
    ///
    /// # 参数
    /// - `version`: Node.js版本号或别名
    pub async fn use_version(&mut self, version: &str) -> Result<()> {
        // 解析版本别名
        let _actual_version = self.resolve_alias(version).unwrap_or(version.to_string());

        // 项目内Node.js管理：不需要版本切换功能
        // 每次执行时明确指定版本即可
        println!(
            "Node.js version management: specify version when running commands, no need to switch"
        );

        Ok(())
    }

    /// 解析版本别名
    ///
    /// # 参数
    /// - `alias`: 版本别名
    ///
    /// # 返回值
    /// 返回实际版本号，如果别名不存在则返回None
    pub fn resolve_alias(&self, alias: &str) -> Option<String> {
        self.config.aliases.get(alias).cloned()
    }

    /// 设置版本别名
    ///
    /// # 参数
    /// - `alias`: 版本别名
    /// - `version`: 实际版本号
    pub fn set_alias(&mut self, alias: &str, version: &str) {
        self.config
            .aliases
            .insert(alias.to_string(), version.to_string());
    }

    /// 删除版本别名
    ///
    /// # 参数
    /// - `alias`: 版本别名
    pub fn remove_alias(&mut self, alias: &str) -> Option<String> {
        self.config.aliases.remove(alias)
    }

    /// 获取所有版本别名
    pub fn aliases(&self) -> &HashMap<String, String> {
        &self.config.aliases
    }

    /// 获取已安装的版本列表（借鉴fnm的版本列表功能）
    pub async fn list_installed_versions(&mut self) -> Result<Vec<String>> {
        if self.installed_versions.is_empty() {
            self.refresh_installed_versions().await?;
        }
        Ok(self.installed_versions.clone())
    }

    /// 刷新已安装版本列表
    async fn refresh_installed_versions(&mut self) -> Result<()> {
        let mut versions = Vec::new();

        if let Ok(entries) = tokio::fs::read_dir(&self.config.install_dir).await {
            let mut entries = entries;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    if dir_name.starts_with("node-") {
                        let version = dir_name.trim_start_matches("node-").to_string();
                        versions.push(version);
                    }
                }
            }
        }

        self.installed_versions = versions;
        Ok(())
    }

    /// 获取远程可用的版本列表（借鉴fnm的远程版本查询）
    pub async fn list_remote_versions(&mut self) -> Result<Vec<String>> {
        if self.remote_versions.is_none() {
            self.fetch_remote_versions().await?;
        }
        Ok(self.remote_versions.as_ref().unwrap().clone())
    }

    /// 从远程获取版本列表
    async fn fetch_remote_versions(&mut self) -> Result<()> {
        let mirror_url = self
            .config
            .mirror_url
            .as_deref()
            .unwrap_or("https://nodejs.org/dist");
        let url = format!("{}/index.json", mirror_url);

        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Failed to fetch versions from {}",
                url
            )));
        }

        let json: serde_json::Value = response.json().await?;
        let mut versions = Vec::new();

        if let Some(array) = json.as_array() {
            for item in array {
                if let Some(version) = item.get("version") {
                    if let Some(version_str) = version.as_str() {
                        // 过滤掉v前缀
                        let clean_version = version_str.trim_start_matches('v').to_string();
                        versions.push(clean_version);
                    }
                }
            }
        }

        self.remote_versions = Some(versions);
        Ok(())
    }

    /// 获取Node.js可执行文件路径
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回Node.js可执行文件的完整路径
    pub fn get_node_path(&self, version: &str) -> PathBuf {
        let version_dir = self.config.install_dir.join(format!("node-{}", version));

        if cfg!(windows) {
            version_dir.join("node.exe")
        } else {
            version_dir.join("bin").join("node")
        }
    }

    /// 获取npm可执行文件路径
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回npm可执行文件的完整路径
    pub fn get_npm_path(&self, version: &str) -> PathBuf {
        let version_dir = self.config.install_dir.join(format!("node-{}", version));

        if cfg!(windows) {
            version_dir.join("npm.cmd")
        } else {
            version_dir.join("bin").join("npm")
        }
    }

    /// 获取npx可执行文件路径
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回npx可执行文件的完整路径
    pub fn get_npx_path(&self, version: &str) -> PathBuf {
        let version_dir = self.config.install_dir.join(format!("node-{}", version));

        if cfg!(windows) {
            version_dir.join("npx.cmd")
        } else {
            version_dir.join("bin").join("npx")
        }
    }

    /// 检查版本是否可用
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回布尔值表示版本是否可用
    pub async fn is_version_available(&self, version: &str) -> Result<bool> {
        // 首先检查本地是否已安装
        if self.is_version_installed(version).await? {
            return Ok(true);
        }

        // 然后检查远程是否可用
        if let Some(remote_versions) = &self.remote_versions {
            return Ok(remote_versions.contains(&version.to_string()));
        }

        // 如果没有缓存，直接检查远程
        let mirror_url = self
            .config
            .mirror_url
            .as_deref()
            .unwrap_or("https://nodejs.org/dist");
        let url = format!("{}/v{}/", mirror_url, version);

        let response = reqwest::get(&url).await?;
        Ok(response.status().is_success())
    }

    /// 检查版本是否已安装
    pub async fn is_version_installed(&self, version: &str) -> Result<bool> {
        let version_dir = self.config.install_dir.join(format!("node-{}", version));
        let node_path = self.get_node_path(version);

        // 检查目录和可执行文件是否存在
        if version_dir.exists() && node_path.exists() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 安装指定版本的Node.js（借鉴fnm的安装逻辑）
    pub async fn install_version(&mut self, version: &str) -> Result<()> {
        // 检查是否已安装
        if self.is_version_installed(version).await? {
            println!("Node.js version {} is already installed", version);
            return Ok(());
        }

        println!("Installing Node.js version: {}", version);

        // 确定下载URL和文件名（借鉴fnm的架构适配逻辑）
        let (download_url, filename) = self.get_download_info(version).await?;

        // 创建临时目录
        let temp_dir = std::env::temp_dir();
        let archive_path = temp_dir.join(&filename);

        // 下载Node.js发行版
        self.download_file(&download_url, &archive_path).await?;

        // 提取文件到安装目录
        self.extract_archive(&archive_path, version).await?;

        // 刷新已安装版本列表
        self.refresh_installed_versions().await?;

        println!("Successfully installed Node.js version: {}", version);
        Ok(())
    }

    /// 获取下载信息（架构和平台适配）
    pub async fn get_download_info(&self, version: &str) -> Result<(String, String)> {
        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(Error::UnsupportedArchitecture);
        };

        let platform = if cfg!(windows) {
            "win"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(unix) {
            "linux"
        } else {
            return Err(Error::Platform("Unsupported platform".to_string()));
        };

        let extension = if cfg!(windows) { "zip" } else { "tar.gz" };

        let filename = format!("node-v{}-{}-{}.{}", version, platform, arch, extension);
        let mirror_url = self
            .config
            .mirror_url
            .as_deref()
            .unwrap_or("https://nodejs.org/dist");
        let download_url = format!("{}/v{}/{}", mirror_url, version, filename);

        Ok((download_url, filename))
    }

    /// 下载文件
    pub async fn download_file(&self, url: &str, path: &PathBuf) -> Result<()> {
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            return Err(Error::Network(format!("Failed to download from {}", url)));
        }

        let content = response.bytes().await?;
        tokio::fs::write(path, content).await?;

        Ok(())
    }

    /// 提取归档文件
    pub async fn extract_archive(&self, archive_path: &PathBuf, version: &str) -> Result<()> {
        let extract_dir = self.config.install_dir.join(format!("node-{}", version));

        if cfg!(windows) {
            // 处理zip文件
            let file = std::fs::File::open(archive_path)?;
            let mut archive = zip::ZipArchive::new(file)?;

            // 检查zip文件中的第一个条目，确定根目录名称
            let root_dir_name = if !archive.is_empty() {
                let first_entry = archive.by_index(0)?;
                let name = first_entry.name();
                if name.contains('/') {
                    name.split('/').next().unwrap_or("").to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let mut outpath = extract_dir.join(file.name());

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
                    outpath = extract_dir.join(relative_path);
                }

                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)?;
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
            }
        } else {
            // 处理tar.gz文件
            let file = std::fs::File::open(archive_path)?;
            let tar = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(tar);

            archive
                .unpack(&extract_dir)
                .map_err(|e| Error::Tar(e.to_string()))?;
        }

        Ok(())
    }

    /// 获取版本环境变量
    ///
    /// # 参数
    /// - `version`: Node.js版本号
    ///
    /// # 返回值
    /// 返回环境变量映射
    pub fn get_version_env(&self, version: &str) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        let node_path = self.get_node_path(version);
        let npm_path = self.get_npm_path(version);
        let npx_path = self.get_npx_path(version);

        // 设置PATH环境变量
        if let Some(bin_dir) = node_path.parent() {
            let mut path = bin_dir.to_string_lossy().to_string();
            if cfg!(windows) {
                path.push(';');
            } else {
                path.push(':');
            }
            path.push_str(&std::env::var("PATH").unwrap_or_default());

            env_vars.insert("PATH".to_string(), path);
        }

        // 设置Node.js相关环境变量
        env_vars.insert("NODE_VERSION".to_string(), version.to_string());
        env_vars.insert(
            "NODE_PATH".to_string(),
            node_path.to_string_lossy().to_string(),
        );
        env_vars.insert(
            "NPM_PATH".to_string(),
            npm_path.to_string_lossy().to_string(),
        );
        env_vars.insert(
            "NPX_PATH".to_string(),
            npx_path.to_string_lossy().to_string(),
        );

        env_vars
    }
}

impl Default for NodeManager {
    fn default() -> Self {
        Self::new()
    }
}
