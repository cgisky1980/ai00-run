# AI00 Run

[English Version](README.md) | [中文版本](README_zh.md)

A Rust library for multi-language runtime management and script execution.

## Overview

AI00 Run provides unified APIs for managing Node.js and Python runtimes, virtual environments, and script execution. Built with Rust for high performance and async support.

**Note: This is a Rust library, not a CLI tool. Use its API in Rust programs.**

### Key Features

- **Multi-language Support**: Node.js and Python runtime management
- **Virtual Environments**: Python virtual environment support (uv-based)
- **Package Management**: Python package installation and management
- **Script Execution**: Run scripts in specified runtime environments
- **Streaming Execution**: Real-time output for long-running processes
- **Configuration Files**: JSON/YAML configuration support
- **Process Monitoring**: Health checks and automatic restart

## Quick Start

### Add Dependency

Add to `Cargo.toml`:

```toml
[dependencies]
ai00-run = { git = "https://github.com/cgisky1980/ai00-run.git" }
```

### Configuration-Based Script Execution (Recommended)

Use JSON/YAML configuration files for script execution:

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // Run from configuration
    let result = runner.run_from_config("config.json", None).await?;
    println!("Result: {}", result.stdout);
    
    Ok(())
}
```

#### Configuration Examples

**JSON Configuration** (`config.json`):

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

**YAML Configuration** (`config.yaml`):

```yaml
name: test-script
script_type: python
python_version: 3.9
script_path: src/main.py
working_dir: examples/test-project
env_vars:
  PYTHONPATH: ./src
```

### Basic Usage

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // Run Node.js script
    let result = runner.run_node_script("18.17.0", "src/main.js").await?;
    println!("Result: {}", result.stdout);
    
    // Run Python script
    let result = runner.run_python_script("3.9", "src/main.py").await?;
    println!("Result: {}", result.stdout);
    
    Ok(())
}
```

## Node.js Management

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Install Node.js version
    node::install("18.17.0").await?;
    
    // Check installation
    let installed = node::is_installed("18.17.0").await?;
    println!("Installed: {}", installed);
    
    // Run command
    let result = node::run_command("18.17.0", &["--version"]).await?;
    println!("Version: {}", result.stdout);
    
    Ok(())
}
```

## Python Management

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Create virtual environment
    py::create_venv(None, Some("3.9")).await?;
    
    // Install packages
    py::install_packages(None, &["requests", "pandas"]).await?;
    
    // Run Python script
    let result = py::run_script(None, "script.py", &[]).await?;
    println!("Output: {}", result.stdout);
    
    Ok(())
}
```

**Note:** Python management is based on the uv tool, ensure uv is installed on your system.

## Script Execution

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // Run scripts
    let result = runner.run_node_script("18.17.0", "script.js").await?;
    println!("Result: {}", result.stdout);
    
    // Execute commands
    let result = runner.execute_command("echo", &["Hello"]).await?;
    println!("Command: {}", result.stdout);
    
    Ok(())
}
```

## Streaming Execution

```rust
use ai00_run::run;
use futures::StreamExt;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // Stream command output
    let mut stream = runner.execute_command_stream("ping", &["google.com"], None).await?;
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(output) => println!("Output: {}", output.stdout),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

### Configuration-Based Script Execution
```

## Project Initialization

```rust
use ai00_run::init;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    init::initialize_project(None, None, None).await?;
    Ok(())
}
```

## Error Handling

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    match node::install("18.0.0").await {
        Ok(()) => println!("Success"),
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}
```

## Architecture

- **Node.js Management**: Version installation and script execution
- **Python Management**: Virtual environments and package management (uv-based)
- **Script Execution**: Multi-language script running with streaming support
- **Configuration**: JSON/YAML configuration file support

## Development

```bash
# Build
cargo build --release

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Features

- Node.js version installation and script execution
- Python virtual environments and package management (uv-based)
- Multi-language script execution with streaming support
- JSON/YAML configuration files

## Requirements

- **uv**: Required for Python management
- **Rust 1.70+**: For building the project
- **Windows/Linux/macOS**: Fully supported

## Contributing

Contributions welcome! Fork the repository and submit pull requests.

## License

MIT OR Apache-2.0 dual license.

## Acknowledgments

Inspired by:
- [fnm](https://github.com/Schniz/fnm)
- [uv](https://github.com/astral-sh/uv)
- [nvm](https://github.com/nvm-sh/nvm)