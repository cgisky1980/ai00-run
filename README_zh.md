# AI00 Run

一个用Rust编写的多语言运行时管理和脚本执行库，提供Node.js和Python的版本管理、虚拟环境管理以及脚本执行功能。

## 项目概述

AI00 Run 是一个现代化的多语言运行时管理库，旨在为开发者提供统一的API接口来管理Node.js和Python运行时环境。项目借鉴了fnm和uv的设计理念，结合了Rust语言的高性能特性，提供了快速、可靠的运行时管理体验。

**注意：这是一个Rust库项目，不是命令行工具。您需要在其他Rust程序中调用其API来使用功能。**

### 核心特性

- **多语言支持**：统一管理Node.js和Python运行时
- **高性能**：基于Rust构建，异步操作支持
- **虚拟环境管理**：完整的Python虚拟环境支持（基于uv）
- **包管理**：Python包安装、卸载和列表功能
- **版本管理**：Node.js和Python版本安装和检查
- **脚本执行**：支持在指定运行时环境中执行脚本

## 快速开始

### 添加依赖

在您的`Cargo.toml`中添加依赖：

```toml
[dependencies]
ai00-run = { git = "https://github.com/cgisky1980/ai00-run.git" }
# 或者使用本地路径
# ai00-run = { path = "./ai00-run" }
```

### 基本使用示例

```rust
use ai00_run::{node, py, run};

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 安装Node.js版本
    node::install("18.0.0").await?;
    
    // 检查Node.js是否已安装
    let is_installed = node::is_installed("18.0.0").await?;
    println!("Node.js 18.0.0 installed: {}", is_installed);
    
    // 创建Python虚拟环境
    py::create_venv(None, Some("3.11")).await?;
    
    // 安装Python包
    py::install_packages(".venv", &["requests", "flask"]).await?;
    
    // 运行Python脚本
    let runner = run::ScriptRunner::new();
    let result = runner.run_python_script("script.py", &[], Some("3.11"), Some(".venv")).await?;
    
    println!("脚本执行成功: {}", result.stdout);
    
    Ok(())
}
```

## Node.js 管理

### 安装Node.js版本

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 安装指定版本的Node.js
    node::install("18.0.0").await?;
    
    Ok(())
}
```

### 版本管理

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 检查Node.js是否已安装
    let is_installed = node::is_installed("18.0.0").await?;
    println!("Node.js 18.0.0 已安装: {}", is_installed);
    
    // 列出已安装的本地Node.js版本
    let local_versions = node::list_local().await?;
    println!("已安装版本: {:?}", local_versions);
    
    // 获取指定版本的Node.js可执行文件路径
    let node_path = node::get_node_path("18.0.0").await?;
    println!("Node.js 可执行文件路径: {}", node_path);
    
    Ok(())
}
```

### 使用npx执行命令

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 在指定版本下使用npx运行命令
    node::run_npx_command("18.0.0", "--version", &[]).await?;
    
    // 使用"current"版本（将使用第一个已安装的版本）
    node::run_npx_command("current", "create-react-app", &["my-app".to_string()]).await?;
    
    Ok(())
}
```

**注意：** 当前版本中，`list_remote()`、`use_version()`、`current()` 等功能标记为TODO，尚未完全实现。

## Python 管理

### 虚拟环境管理

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 创建虚拟环境（默认使用Python 3.11）
    py::create_venv(None, Some("3.11")).await?;
    
    // 检查虚拟环境是否存在
    let exists = py::venv_exists(".venv").await?;
    println!("虚拟环境存在: {}", exists);
    
    // 激活虚拟环境
    py::activate_venv(".venv").await?;
    
    // 停用虚拟环境
    py::deactivate_venv().await?;
    
    Ok(())
}
```

### 包管理

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 安装Python包到虚拟环境
    py::install_packages(".venv", &["requests", "flask"]).await?;
    
    // 卸载Python包
    py::uninstall_packages(".venv", &["requests"]).await?;
    
    // 列出虚拟环境中已安装的包
    let packages = py::list_packages(".venv").await?;
    println!("已安装包: {:?}", packages);
    
    Ok(())
}
```

### 脚本和命令执行

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 在虚拟环境中运行Python脚本
    py::run_script(".venv", "print('Hello from Python!')").await?;
    
    // 在虚拟环境中运行Python命令
    py::run_command(".venv", "import sys; print(sys.version)").await?;
    
    // 获取虚拟环境中Python可执行文件的路径
    let python_path = py::get_python_path_in_venv(".venv").await?;
    println!("Python 可执行文件路径: {}", python_path);
    
    // 获取指定版本的Python可执行文件路径
    let python_path = py::get_python_path("3.11").await?;
    println!("系统Python路径: {}", python_path);
    
    Ok(())
}
```

**注意：** Python管理基于uv工具实现，需要确保系统已安装uv。

## 脚本执行

### 运行脚本

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // 运行Node.js脚本
    let result = runner.run_node_script("app.js", &[], Some("18.0.0")).await?;
    println!("Node.js脚本结果: {}", result.stdout);
    
    // 运行Python脚本
    let result = runner.run_python_script("script.py", &[], Some("3.11"), Some(".venv")).await?;
    println!("Python脚本结果: {}", result.stdout);
    
    // 运行Shell脚本
    let result = runner.run_shell_script("script.sh", &[]).await?;
    println!("Shell脚本结果: {}", result.stdout);
    
    Ok(())
}
```

### 直接执行命令

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let executor = run::ScriptExecutor::new();
    
    // 异步执行命令
    let result = executor.execute_command_async("echo hello world", None, None).await?;
    println!("命令结果: {}", result.stdout);
    
    // 同步执行命令
    let result = executor.execute_command_sync("ls -la", None, None)?;
    println!("命令结果: {}", result.stdout);
    
    // 检查命令是否存在
    let exists = executor.command_exists("python").await;
    println!("Python命令存在: {}", exists);
    
    // 获取命令的完整路径
    if let Some(path) = executor.get_command_path("python").await {
        println!("Python命令路径: {}", path);
    }
    
    Ok(())
}
```

## 项目初始化

### 初始化新项目

```rust
use ai00_run::init;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 初始化项目
    init::initialize_project(None, None, None).await?;
    
    Ok(())
}
```

## 配置说明

### 错误处理

库使用统一的错误类型 `ai00_run::Result<T>`，包含各种运行时错误：

```rust
use ai00_run::{node, py, run};

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    match node::install("18.0.0").await {
        Ok(()) => println!("Node.js安装成功"),
        Err(e) => eprintln!("Node.js安装失败: {}", e),
    }
    
    Ok(())
}
```

### 异步支持

所有API都支持异步操作，需要使用 `#[tokio::main]` 宏：

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 异步安装Node.js
    node::install("18.0.0").await?;
    
    Ok(())
}
```

## 技术架构

### 模块结构

```
src/
├── lib.rs               # 库入口（提供公共API）
├── error.rs             # 错误处理
├── init.rs              # 项目初始化
├── node/                # Node.js管理模块
│   ├── mod.rs           # 模块入口（提供NodeManager和便捷函数）
│   ├── installer.rs     # Node.js安装器
│   └── manager.rs       # Node.js管理器
├── py/                  # Python管理模块
│   ├── mod.rs           # 模块入口（提供PyManager和便捷函数）
│   ├── installer.rs     # Python安装器（基于uv）
│   └── manager.rs       # Python管理器
└── run/                 # 脚本执行模块
    ├── mod.rs           # 模块入口（提供ScriptRunner和ScriptExecutor）
    ├── checker.rs       # 脚本检查器
    ├── config.rs        # 运行配置
    └── executor.rs      # 脚本执行器
```

### 核心依赖

- **tokio**：异步运行时
- **reqwest**：HTTP客户端（用于下载Node.js）
- **serde**：序列化/反序列化
- **zip/tar**：压缩包处理（用于解压Node.js）

### 设计特点

1. **基于uv的Python管理**：Python功能完全基于uv工具实现
2. **异步优先**：所有API都支持异步操作
3. **错误处理**：统一的错误类型和详细的错误信息
4. **跨平台支持**：支持Windows和Unix系统
5. **模块化设计**：清晰的模块分离，便于维护和扩展

## 开发指南

### 构建项目

```bash
# 开发模式构建
cargo build

# 发布模式构建（推荐）
cargo build --release

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 运行示例项目

项目包含示例代码，位于 `examples/` 目录：

```bash
# 进入示例项目目录
cd examples/test-project

# 构建并运行示例
cargo run --release
```

### 功能状态

**已实现的功能：**
- Python虚拟环境创建和管理（基于uv）
- Python包安装、卸载、列表
- Node.js版本安装和检查
- 脚本执行和命令运行
- 异步和同步命令执行

**部分实现的功能：**
- Node.js的npx命令执行
- 命令存在性检查

**未实现的功能（标记为TODO）：**
- Node.js远程版本列表
- Node.js版本切换
- Node.js当前版本获取
- 完整的权限检查

## 系统要求

### 依赖工具

- **uv**：Python管理功能需要安装uv工具
- **Node.js**：Node.js管理功能需要系统安装Node.js或通过本库安装
- **Rust 1.70+**：构建项目需要Rust工具链

### 平台支持

- **Windows**：完全支持
- **Linux**：完全支持
- **macOS**：完全支持

## 故障排除

### 常见问题

1. **uv工具未安装**
   ```bash
   # 安装uv工具
   curl -LsSf https://astral.sh/uv/install.sh | sh
   ```

2. **Node.js安装失败**
   - 检查网络连接
   - 确保有足够的磁盘空间
   - 检查系统权限

3. **Python虚拟环境创建失败**
   - 确保uv工具正确安装
   - 检查Python版本是否可用
   - 确保目标目录有写入权限

### 错误处理

库提供详细的错误信息，帮助诊断问题：

```rust
use ai00_run::{node, py};

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    match node::install("18.0.0").await {
        Ok(()) => println!("安装成功"),
        Err(e) => {
            eprintln!("安装失败: {}", e);
            // 根据错误类型进行特定处理
        }
    }
    
    Ok(())
}
```

## 贡献指南

我们欢迎社区贡献！请参考以下指南：

1. Fork 项目仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建Pull Request

### 开发规范

- 遵循Rust编码规范
- 添加适当的测试用例
- 更新相关文档
- 确保所有测试通过
- 运行 `cargo fmt` 和 `cargo clippy` 确保代码质量

## 许可证

本项目采用 MIT OR Apache-2.0 双许可证。

## 致谢

AI00 Run 的灵感来源于以下优秀项目：

- [fnm](https://github.com/Schniz/fnm) - Fast Node Manager
- [uv](https://github.com/astral-sh/uv) - 极速Python包管理器
- [nvm](https://github.com/nvm-sh/nvm) - Node Version Manager

感谢这些项目为运行时管理领域做出的贡献！

## 支持与反馈

如果您遇到问题或有改进建议，请通过以下方式联系我们：

- [GitHub Issues](https://github.com/cgisky1980/ai00-run/issues)
- [项目文档](https://ai00-run.github.io/ai00-run/)

---

**AI00 Run** - 让多语言运行时管理变得简单高效！