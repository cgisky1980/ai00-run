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
- **Streaming Execution**: Real-time output streaming for long-running processes
- **Long-Running Process Support**: Indefinite execution with process monitoring and automatic restart
- **Configuration Management**: JSON/YAML configuration file support for script execution
- **Timeout Control**: Configurable execution timeouts for scripts and commands
- **Process Monitoring**: Real-time process status tracking and health checks
- **Automatic Restart**: Configurable restart policies for failed processes

## Quick Start

### Adding Dependencies

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
ai00-run = { git = "https://github.com/cgisky1980/ai00-run.git" }
# Or use local path
# ai00-run = { path = "./ai00-run" }
```

### 🚀 Primary Usage: Configuration-Based Script Execution

**Strongly recommended to use configuration files for running scripts** - this approach provides the most complete and flexible functionality support, including complex configuration options, environment variable management, dependency installation, and more.

#### Running Scripts from Configuration Files

ai00-run supports running scripts based on configuration files (JSON or YAML format), which allows for more complex and reusable script configurations.

```rust
use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let runner = run::ScriptRunner::new();
    
    // Run script from JSON configuration
    let result = runner.run_from_config("config/script_config.json", None).await?;
    println!("Script result: {}", result.stdout);
    
    // Run script from YAML configuration
    let result = runner.run_from_config("config/script_config.yaml", None).await?;
    println!("Script result: {}", result.stdout);
    
    // Run with custom options
    let options = run::ExecuteOptions {
        timeout_ms: Some(60000),
        working_dir: Some("examples/test-project".to_string()),
        env_vars: Some(vec![("NODE_ENV".to_string(), "development".to_string())]),
    };
    
    let result = runner.run_from_config_with_options("config.json", Some(options)).await?;
    println!("Script result: {}", result.stdout);
    
    Ok(())
}
```

#### Configuration File Format Examples

**JSON Configuration Example:**
```json
{
  "name": "example_python_app",
  "description": "An example Python application configuration",
  "script_type": "python",
  "script_path": "src/main.py",
  "runtime_version": "3.11",
  "venv_path": ".venv",
  "args": ["--verbose", "--debug"],
  "env_vars": {
    "PYTHONPATH": ".",
    "DEBUG": "true"
  },
  "working_dir": "examples/test-project",
  "timeout": null,
  "async_execution": true,
  "dependencies": ["requests", "flask"],
  "streaming_execution": true,
  "stream_buffer_size": 16384,
  "restart_policy": "on-failure",
  "max_restarts": 5,
  "restart_delay": 10000,
  "monitor_interval": 2000,
  "daemon_mode": false
}
```

**YAML Configuration Example:**
```yaml
name: example_node_app
description: An example Node.js application configuration
script_type: node
script_path: src/app.js
runtime_version: 18.0.0
args:
  - --port
  - "3000"
env_vars:
  NODE_ENV: development
  DEBUG: true
working_dir: examples/test-project
timeout: null
async_execution: true
dependencies:
  - express
  - cors
streaming_execution: true
stream_buffer_size: 16384
restart_policy: on-failure
max_restarts: 5
restart_delay: 10000
monitor_interval: 2000
daemon_mode: false
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

### Streaming Execution

ai00-run supports real-time streaming execution for long-running processes, allowing you to receive output as it's generated. The library automatically manages threads for streaming execution, providing built-in thread management capabilities.

#### Use Cases

- **Streaming script execution with automatic thread management**: Execute scripts in new threads with real-time output streaming
- **Multi-script parallel execution**: Run multiple scripts concurrently with independent thread management
- **Thread management and control**: Monitor, terminate, and manage script execution threads
- **Configuration-based streaming execution**: Use JSON/YAML configuration files for streaming execution

#### Streaming Script Execution with Automatic Thread Management

ai00-run automatically creates new threads for streaming execution and provides comprehensive thread management:

```rust
use ai00_run::run::{run_node_script_stream, StreamMessage, StreamExecutorHandle};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Execute script in a new thread with streaming output
    let mut handle: StreamExecutorHandle = run_node_script_stream(
        "server.js",
        &["--port", "8080"],
        Some("18.0.0")
    ).await?;
    
    println!("Script started in new thread with ID: {}", handle.child_id);
    
    // Monitor real-time output from the new thread
    let mut output_count = 0;
    while let Some(message) = handle.receiver.recv().await {
        match message {
            StreamMessage::Stdout(data) => {
                output_count += 1;
                println!("[Thread {}] STDOUT: {}", handle.child_id, data);
                
                // Example: Stop after receiving 10 outputs
                if output_count >= 10 {
                    println!("Received 10 outputs, terminating thread...");
                    handle.kill().await?;
                    break;
                }
            }
            StreamMessage::Stderr(data) => {
                eprintln!("[Thread {}] STDERR: {}", handle.child_id, data);
            }
            StreamMessage::Exit(code) => {
                println!("[Thread {}] Process exited with code: {}", handle.child_id, code);
                break;
            }
            StreamMessage::Error(err) => {
                eprintln!("[Thread {}] Error: {}", handle.child_id, err);
                break;
            }
        }
    }
    
    // Wait for thread completion
    let exit_code = handle.wait().await?;
    println!("Thread {} completed with exit code: {}", handle.child_id, exit_code);
    
    Ok(())
}
```

#### Multi-Script Parallel Execution with Thread Management

Run multiple scripts concurrently with independent thread management:

```rust
use ai00_run::run::{run_python_script_stream, run_shell_script_stream, StreamMessage};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let mut tasks = JoinSet::new();
    
    // Start multiple scripts in separate threads
    tasks.spawn(async move {
        let mut handle = run_python_script_stream(
            "data_processor.py",
            &["--mode", "batch"],
            Some("3.11"),
            Some(".venv")
        ).await?;
        
        println!("Started data processor in thread: {}", handle.child_id);
        
        while let Some(message) = handle.receiver.recv().await {
            match message {
                StreamMessage::Stdout(data) => println!("[Processor {}] {}", handle.child_id, data),
                StreamMessage::Exit(code) => {
                    println!("Processor {} exited with code: {}", handle.child_id, code);
                    break;
                }
                _ => {}
            }
        }
        
        handle.wait().await
    });
    
    tasks.spawn(async move {
        let mut handle = run_shell_script_stream(
            "monitor.sh",
            &["--interval", "5"]
        ).await?;
        
        println!("Started monitor in thread: {}", handle.child_id);
        
        while let Some(message) = handle.receiver.recv().await {
            match message {
                StreamMessage::Stdout(data) => println!("[Monitor {}] {}", handle.child_id, data),
                StreamMessage::Exit(code) => {
                    println!("Monitor {} exited with code: {}", handle.child_id, code);
                    break;
                }
                _ => {}
            }
        }
        
        handle.wait().await
    });
    
    // Wait for all threads to complete
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(exit_code) => println!("Thread completed with exit code: {}", exit_code),
            Err(e) => eprintln!("Thread failed: {}", e),
        }
    }
    
    Ok(())
}
```

#### Thread Management and Control

ai00-run provides comprehensive thread management capabilities:

```rust
use ai00_run::run::{run_from_config_stream, StreamMessage};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    // Start script with configuration
    let mut handle = run_from_config_stream("config/streaming_server.json").await?;
    
    let handle_arc = Arc::new(Mutex::new(handle));
    let thread_id = handle_arc.lock().await.child_id;
    
    println!("Script started in thread: {}", thread_id);
    
    // Monitor thread for 30 seconds, then terminate
    let monitor_handle = Arc::clone(&handle_arc);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        println!("30 seconds elapsed, terminating thread {}...", thread_id);
        
        if let Ok(mut handle) = monitor_handle.lock().await {
            let _ = handle.kill().await;
        }
    });
    
    // Process real-time output
    let mut handle_guard = handle_arc.lock().await;
    while let Some(message) = handle_guard.receiver.recv().await {
        match message {
            StreamMessage::Stdout(data) => {
                println!("[Thread {}] {}", thread_id, data);
            }
            StreamMessage::Exit(code) => {
                println!("Thread {} exited with code: {}", thread_id, code);
                break;
            }
            StreamMessage::Error(err) => {
                eprintln!("Thread {} error: {}", thread_id, err);
                break;
            }
            _ => {}
        }
    }
    
    // Clean up
    let exit_code = handle_guard.wait().await?;
    println!("Thread {} cleanup completed with code: {}", thread_id, exit_code);
    
    Ok(())
}
```

#### Thread Management Features

- **Automatic thread creation**: Scripts are automatically executed in new threads
- **Thread ID tracking**: Each execution thread has a unique identifier
- **Graceful termination**: Use `kill()` method for controlled thread termination
- **Thread monitoring**: Monitor thread status and receive completion notifications
- **Resource cleanup**: Automatic cleanup of thread resources upon completion
- **Error handling**: Comprehensive error handling for thread-related issues

#### Extended Use Cases

- **Microservices orchestration**: Manage multiple service threads
- **Background task processing**: Execute long-running tasks in background threads
- **Real-time data processing**: Stream process data with thread isolation
- **Development servers**: Run development servers with thread management
- **CI/CD pipelines**: Execute build and test scripts in managed threads

#### Configuration-Based Streaming Execution

Use JSON configuration for streaming execution with thread management:

```json
{
  "script_type": "node",
  "script_path": "server.js",
  "args": ["--port", "8080"],
  "node_version": "18.0.0",
  "streaming_execution": true,
  "timeout": 300000,
  "working_dir": "./app",
  "env": {
    "NODE_ENV": "development",
    "DEBUG": "true"
  }
}
```

Execute with automatic thread management:

```rust
use ai00_run::run::run_from_config_stream;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let mut handle = run_from_config_stream("config/streaming_server.json").await?;
    
    println!("Script started in thread: {}", handle.child_id);
    
    // Thread management and output processing...
    // (Same as previous examples)
    
    Ok(())
}
```

### Configuration-Based Script Execution
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
- Python package installation, uninstallation, and listing
- Node.js version installation and checking
- Script execution and command running
- Async and sync command execution
- **Streaming execution with real-time output**
- **Configuration-based script execution (JSON/YAML)**
- **Timeout control for scripts and commands**
- **Advanced configuration management**

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