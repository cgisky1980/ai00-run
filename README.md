# AI00 Run

A multi-language runtime management and script execution library written in Rust, providing version management for Node.js and Python, virtual environment management, and script execution capabilities.

## Project Overview

AI00 Run is a modern multi-language runtime management library designed to provide developers with a unified API interface for managing Node.js and Python runtime environments. The project draws inspiration from fnm and uv design concepts, combining Rust's high-performance characteristics to deliver a fast and reliable runtime management experience.

**Note: This is a Rust library project, not a command-line tool. You need to call its API in other Rust programs to use its functionality.**

### Core Features

- **Multi-language Support**: Unified management of Node.js and Python runtimes
- **High Performance**: Built on Rust with async operation support
- **Virtual Environment Management**: Complete Python virtual environment support (based on uv)
- **Package Management**: Python package installation, uninstallation, and listing
- **Version Management**: Node.js and Python version installation and checking
- **Script Execution**: Support for executing scripts in specified runtime environments

## Quick Start

### Adding Dependencies

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
ai00-run = { git = "https://github.com/cgisky1980/ai00-run.git" }
# Or use local path
# ai00-run = { path = "./ai00-run" }
```

### Basic Usage Example

```rust
use ai00_run::{node, py, run};

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Install Node.js version
    node::install("18.0.0").await?;
    
    // Check if Node.js is installed
    let is_installed = node::is_installed("18.0.0").await?;
    println!("Node.js 18.0.0 installed: {}", is_installed);
    
    // Create Python virtual environment
    py::create_venv(None, Some("3.11")).await?;
    
    // Install Python packages
    py::install_packages(".venv", &["requests", "flask"]).await?;
    
    // Run Python script
    let runner = run::ScriptRunner::new();
    let result = runner.run_python_script("script.py", &[], Some("3.11"), Some(".venv")).await?;
    
    println!("Script executed successfully: {}", result.stdout);
    
    Ok(())
}
```

## Node.js Management

### Installing Node.js Versions

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Install specified Node.js version
    node::install("18.0.0").await?;
    
    Ok(())
}
```

### Version Management

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Check if Node.js is installed
    let is_installed = node::is_installed("18.0.0").await?;
    println!("Node.js 18.0.0 installed: {}", is_installed);
    
    // List locally installed Node.js versions
    let local_versions = node::list_local().await?;
    println!("Installed versions: {:?}", local_versions);
    
    // Get Node.js executable path for specified version
    let node_path = node::get_node_path("18.0.0").await?;
    println!("Node.js executable path: {}", node_path);
    
    Ok(())
}
```

### Using npx Commands

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Run npx command with specified version
    node::run_npx_command("18.0.0", "--version", &[]).await?;
    
    // Use "current" version (will use first installed version)
    node::run_npx_command("current", "create-react-app", &["my-app".to_string()]).await?;
    
    Ok(())
}
```

**Note:** In the current version, functions like `list_remote()`, `use_version()`, `current()` are marked as TODO and not fully implemented.

## Python Management

### Virtual Environment Management

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Create virtual environment (default Python 3.11)
    py::create_venv(None, Some("3.11")).await?;
    
    // Check if virtual environment exists
    let exists = py::venv_exists(".venv").await?;
    println!("Virtual environment exists: {}", exists);
    
    // Activate virtual environment
    py::activate_venv(".venv").await?;
    
    // Deactivate virtual environment
    py::deactivate_venv().await?;
    
    Ok(())
}
```

### Package Management

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Install Python packages to virtual environment
    py::install_packages(".venv", &["requests", "flask"]).await?;
    
    // Uninstall Python packages
    py::uninstall_packages(".venv", &["requests"]).await?;
    
    // List packages installed in virtual environment
    let packages = py::list_packages(".venv").await?;
    println!("Installed packages: {:?}", packages);
    
    Ok(())
}
```

### Script and Command Execution

```rust
use ai00_run::py;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Run Python script in virtual environment
    py::run_script(".venv", "print('Hello from Python!')").await?;
    
    // Run Python command in virtual environment
    py::run_command(".venv", "import sys; print(sys.version)").await?;
    
    // Get Python executable path in virtual environment
    let python_path = py::get_python_path_in_venv(".venv").await?;
    println!("Python executable path: {}", python_path);
    
    // Get Python executable path for specified version
    let python_path = py::get_python_path("3.11").await?;
    println!("System Python path: {}", python_path);
    
    Ok(())
}
```

**Note:** Python management is based on the uv tool, ensure uv is installed on your system.

## Script Execution

### Running Scripts

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // Run Node.js script
    let result = runner.run_node_script("app.js", &[], Some("18.0.0")).await?;
    println!("Node.js script result: {}", result.stdout);
    
    // Run Python script
    let result = runner.run_python_script("script.py", &[], Some("3.11"), Some(".venv")).await?;
    println!("Python script result: {}", result.stdout);
    
    // Run shell script
    let result = runner.run_shell_script("script.sh", &[]).await?;
    println!("Shell script result: {}", result.stdout);
    
    Ok(())
}
```

### Direct Command Execution

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let executor = run::ScriptExecutor::new();
    
    // Execute command asynchronously
    let result = executor.execute_command_async("echo hello world", None, None).await?;
    println!("Command result: {}", result.stdout);
    
    // Execute command synchronously
    let result = executor.execute_command_sync("ls -la", None, None)?;
    println!("Command result: {}", result.stdout);
    
    // Check if command exists
    let exists = executor.command_exists("python").await;
    println!("Python command exists: {}", exists);
    
    // Get full path of command
    if let Some(path) = executor.get_command_path("python").await {
        println!("Python command path: {}", path);
    }
    
    Ok(())
}
```

## Project Initialization

### Initializing New Projects

```rust
use ai00_run::init;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Initialize project
    init::initialize_project(None, None, None).await?;
    
    Ok(())
}
```

## Configuration

### Error Handling

The library uses a unified error type `ai00_run::Result<T>` containing various runtime errors:

```rust
use ai00_run::{node, py, run};

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    match node::install("18.0.0").await {
        Ok(()) => println!("Node.js installed successfully"),
        Err(e) => eprintln!("Failed to install Node.js: {}", e),
    }
    
    Ok(())
}
```

### Async Support

All APIs support async operations and require the `#[tokio::main]` macro:

```rust
use ai00_run::node;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Install Node.js asynchronously
    node::install("18.0.0").await?;
    
    Ok(())
}
```

## Technical Architecture

### Module Structure

```
src/
├── lib.rs               # Library entry (provides public API)
├── error.rs             # Error handling
├── init.rs              # Project initialization
├── node/                # Node.js management module
│   ├── mod.rs           # Module entry (provides NodeManager and convenience functions)
│   ├── installer.rs     # Node.js installer
│   └── manager.rs       # Node.js manager
├── py/                  # Python management module
│   ├── mod.rs           # Module entry (provides PyManager and convenience functions)
│   ├── installer.rs     # Python installer (based on uv)
│   └── manager.rs       # Python manager
└── run/                 # Script execution module
    ├── mod.rs           # Module entry (provides ScriptRunner and ScriptExecutor)
    ├── checker.rs       # Script checker
    ├── config.rs        # Runtime configuration
    └── executor.rs      # Script executor
```

### Core Dependencies

- **tokio**: Async runtime
- **reqwest**: HTTP client (for downloading Node.js)
- **serde**: Serialization/deserialization
- **zip/tar**: Archive handling (for extracting Node.js)

### Design Features

1. **uv-based Python Management**: Python functionality fully implemented based on uv tool
2. **Async-First**: All APIs support async operations
3. **Error Handling**: Unified error type with detailed error information
4. **Cross-Platform Support**: Support for Windows and Unix systems
5. **Modular Design**: Clear module separation for easy maintenance and extension

## Development Guide

### Building the Project

```bash
# Development build
cargo build

# Release build (recommended)
cargo build --release

# Run tests
cargo test

# Code formatting
cargo fmt

# Code linting
cargo clippy
```

### Running Example Projects

The project includes example code located in the `examples/` directory:

```bash
# Enter example project directory
cd examples/test-project

# Build and run example
cargo run --release
```

### Feature Status

**Implemented Features:**
- Python virtual environment creation and management (based on uv)
- Python package installation, uninstallation, listing
- Node.js version installation and checking
- Script execution and command running
- Async and sync command execution

**Partially Implemented Features:**
- Node.js npx command execution
- Command existence checking

**Not Implemented Features (marked as TODO):**
- Node.js remote version listing
- Node.js version switching
- Node.js current version retrieval
- Complete permission checking

## System Requirements

### Required Tools

- **uv**: Python management features require uv tool installation
- **Node.js**: Node.js management features require system Node.js installation or installation through this library
- **Rust 1.70+**: Rust toolchain required for building the project

### Platform Support

- **Windows**: Fully supported
- **Linux**: Fully supported
- **macOS**: Fully supported

## Troubleshooting

### Common Issues

1. **uv Tool Not Installed**
   ```bash
   # Install uv tool
   curl -LsSf https://astral.sh/uv/install.sh | sh
   ```

2. **Node.js Installation Failed**
   - Check network connection
   - Ensure sufficient disk space
   - Check system permissions

3. **Python Virtual Environment Creation Failed**
   - Ensure uv tool is correctly installed
   - Check if Python version is available
   - Ensure write permissions in target directory

### Error Handling

The library provides detailed error information to help diagnose issues:

```rust
use ai00_run::{node, py};

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    match node::install("18.0.0").await {
        Ok(()) => println!("Installation successful"),
        Err(e) => {
            eprintln!("Installation failed: {}", e);
            // Handle specific error types
        }
    }
    
    Ok(())
}
```

## Contributing

We welcome community contributions! Please refer to the following guidelines:

1. Fork the project repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

### Development Standards

- Follow Rust coding standards
- Add appropriate test cases
- Update relevant documentation
- Ensure all tests pass
- Run `cargo fmt` and `cargo clippy` to ensure code quality

## License

This project is licensed under MIT OR Apache-2.0 dual license.

## Acknowledgments

AI00 Run is inspired by the following excellent projects:

- [fnm](https://github.com/Schniz/fnm) - Fast Node Manager
- [uv](https://github.com/astral-sh/uv) - Extremely fast Python package manager
- [nvm](https://github.com/nvm-sh/nvm) - Node Version Manager

Thanks to these projects for their contributions to the runtime management field!

## Support and Feedback

If you encounter issues or have improvement suggestions, please contact us through:

- [GitHub Issues](https://github.com/cgisky1980/ai00-run/issues)
- [Project Documentation](https://ai00-run.github.io/ai00-run/)

---

**AI00 Run** - Making multi-language runtime management simple and efficient!