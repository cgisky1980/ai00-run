# AI00 Run

[English Version](README.md) | [中文版本](README_zh.md)

一个用于多语言运行时管理和脚本执行的Rust库。

## 概述

AI00 Run 提供了统一的API来管理Node.js和Python运行时、虚拟环境和脚本执行。使用Rust构建，具有高性能和异步支持。

**注意：这是一个Rust库，不是CLI工具。在Rust程序中使用其API。**

### 核心特性

- **多语言支持**：Node.js和Python运行时管理
- **虚拟环境**：Python虚拟环境支持（基于uv）
- **包管理**：Python包安装和管理
- **脚本执行**：在指定运行时环境中运行脚本
- **流式执行**：长运行进程的实时输出
- **配置文件**：JSON/YAML配置支持
- **进程监控**：健康检查和自动重启

## 快速开始

### 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
ai00-run = { git = "https://github.com/cgisky1980/ai00-run.git" }
```

### 基于配置的脚本执行（推荐）

使用JSON/YAML配置文件进行脚本执行：

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // 从配置运行
    let result = runner.run_from_config("config.json", None).await?;
    println!("结果: {}", result.stdout);
    
    Ok(())
}
```

#### 配置示例

**JSON配置** (`config.json`):

```json
{
  "name": "test-script",
  "script_type": "node",
  "node_version": "18.17.0",
  "script_path": "src/main.js",
  "working_dir": "examples/test-project",
  "env_vars": {
    "NODE_ENV": "development"
  }
}
```

**YAML配置** (`config.yaml`):

```yaml
name: test-script
script_type: python
python_version: 3.9
script_path: src/main.py
working_dir: examples/test-project
env_vars:
  PYTHONPATH: ./src
```

### 基本用法

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // 运行Node.js脚本
    let result = runner.run_node_script("18.17.0", "src/main.js").await?;
    println!("结果: {}", result.stdout);
    
    // 运行Python脚本
    let result = runner.run_python_script("3.9", "src/main.py").await?;
    println!("结果: {}", result.stdout);
    
    Ok(())
}
```

## Node.js管理

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 安装Node.js版本
    node::install("18.17.0").await?;
    
    // 检查安装
    let installed = node::is_installed("18.17.0").await?;
    println!("已安装: {}", installed);
    
    // 运行命令
    let result = node::run_command("18.17.0", &["--version"]).await?;
    println!("版本: {}", result.stdout);
    
    Ok(())
}
```

## Python管理

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // 创建虚拟环境
    py::create_venv(None, Some("3.9")).await?;
    
    // 安装包
    py::install_packages(None, &["requests", "pandas"]).await?;
    
    // 运行Python脚本
    let result = py::run_script(None, "script.py", &[]).await?;
    println!("输出: {}", result.stdout);
    
    Ok(())
}
```

**注意：** Python管理基于uv工具，请确保系统已安装uv。

## 脚本执行

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // 运行脚本
    let result = runner.run_node_script("18.17.0", "script.js").await?;
    println!("结果: {}", result.stdout);
    
    // 执行命令
    let result = runner.execute_command("echo", &["Hello"]).await?;
    println!("命令: {}", result.stdout);
    
    Ok(())
}
```

## 流式执行

```rust
use ai00_run::run;
use futures::StreamExt;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // 流式命令输出
    let mut stream = runner.execute_command_stream("ping", &["google.com"], None).await?;
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(output) => println!("输出: {}", output.stdout),
            Err(e) => eprintln!("错误: {}", e),
        }
    }
    
    Ok(())
}
```

## 项目初始化

```rust
use ai00_run::init;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    init::initialize_project(None, None, None).await?;
    Ok(())
}
```

## 错误处理

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    match node::install("18.0.0").await {
        Ok(()) => println!("成功"),
        Err(e) => eprintln!("错误: {}", e),
    }
    Ok(())
}
```

## 架构

- **Node.js管理**：版本安装和脚本执行
- **Python管理**：虚拟环境和包管理（基于uv）
- **脚本执行**：多语言脚本运行，支持流式
- **配置**：JSON/YAML配置文件支持

## 开发

```bash
# 构建
cargo build --release

# 测试
cargo test

# 格式化
cargo fmt

# 检查
cargo clippy
```

## 功能

- Node.js版本安装和脚本执行
- Python虚拟环境和包管理（基于uv）
- 多语言脚本执行，支持流式
- JSON/YAML配置文件

## 要求

- **uv**：Python管理必需
- **Rust 1.70+**：构建项目
- **Windows/Linux/macOS**：全平台支持

## 贡献

欢迎贡献！Fork仓库并提交Pull Request。

## 许可证

MIT OR Apache-2.0 双重许可证。

## 致谢

灵感来源：
- [fnm](https://github.com/Schniz/fnm)
- [uv](https://github.com/astral-sh/uv)
- [nvm](https://github.com/nvm-sh/nvm)