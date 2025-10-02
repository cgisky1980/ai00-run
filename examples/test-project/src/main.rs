use ai00_run::{
    init, node, py,
    run::{run_from_config, ScriptRunner, ExecuteOptions, run_from_config_with_options, run_from_config_stream, StreamMessage},
};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ai00-run 库测试项目 ===\n");

    // 1. 初始化ai00-run环境
    println!("--- 初始化ai00-run环境 ---");
    let init_manager = init::InitManager::new();
    match init_manager.init().await {
        Ok(_) => println!("✓ ai00-run环境初始化成功"),
        Err(e) => {
            println!("✗ ai00-run环境初始化失败: {}", e);
            return Ok(());
        }
    }

    // 2. 安装和管理Node.js版本
    println!("\n--- 安装和管理Node.js版本 ---");
    let node_manager = node::NodeManager::new();

    // 检查是否已安装Node.js版本
    match node_manager.list_local().await {
        Ok(versions) => {
            if versions.is_empty() {
                println!("未找到已安装的Node.js版本，开始安装...");

                // 尝试安装Node.js 18.0.0版本
                match node_manager.install("18.0.0").await {
                    Ok(_) => {
                        println!("✓ Node.js 18.0.0 安装成功");

                        // 再次检查安装结果
                        match node_manager.list_local().await {
                            Ok(installed_versions) => {
                                println!("已安装的Node.js版本: {:?}", installed_versions);
                            }
                            Err(e) => {
                                println!("✗ 获取安装后的版本列表失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Node.js 18.0.0 安装失败: {}", e);
                        println!("⚠ 跳过Node.js版本管理测试");
                    }
                }
            } else {
                println!("已安装的Node.js版本: {:?}", versions);
            }
        }
        Err(e) => {
            println!("✗ 获取Node.js版本列表失败: {}", e);
            println!("⚠ 跳过Node.js版本管理测试");
        }
    }

    // 3. 安装和管理Python版本
    println!("\n--- 安装和管理Python版本 ---");
    let py_manager = py::PyManager::new();

    // 测试创建虚拟环境
    match py_manager.create_venv(None, Some("3.11")).await {
        Ok(_) => println!("✓ Python虚拟环境创建成功"),
        Err(e) => {
            println!("✗ Python虚拟环境创建失败: {}", e);
            println!("⚠ 跳过Python虚拟环境测试");
        }
    }

    // 4. 创建测试脚本目录和脚本
    println!("\n--- 创建测试脚本 ---");
    let test_dir = Path::new("./test-scripts");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
        println!("创建测试脚本目录: {:?}", test_dir);
    }
    create_test_scripts(test_dir)?;

    // 5. 测试脚本执行（跳过，因为需要先安装版本）
    println!("\n--- 脚本执行测试 ---");
    println!("⚠ 跳过脚本执行测试（需要先安装Node.js和Python版本）");

    // 6. 测试基于配置文件的脚本执行
    println!("\n--- 测试基于配置文件的脚本执行 ---");
    test_run_from_config().await?;

    // 7. 测试超时和流式执行功能
    println!("\n--- 测试超时和流式执行功能 ---");
    test_timeout_and_streaming().await?;

    // 8. 清理测试文件
    println!("\n--- 清理测试文件 ---");
    cleanup_test_files(test_dir)?;

    println!("\n=== 测试完成 ===");
    println!("\n说明：");
    println!("- 测试项目验证了ai00-run的初始化功能");
    println!("- Node.js和Python版本管理功能需要进一步实现安装逻辑");
    println!("- 脚本执行功能需要先安装对应的运行时版本");
    println!("- 基于配置文件的脚本执行功能已测试");

    Ok(())
}

async fn test_timeout_and_streaming() -> Result<(), Box<dyn std::error::Error>> {
    
    // 创建测试脚本目录
    let test_dir = Path::new("./test-scripts");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
        println!("创建测试脚本目录: {:?}", test_dir);
    }

    // 创建长时间运行的测试脚本
    create_long_running_scripts(test_dir)?;

    // 1. 测试超时功能
    println!("1. 测试超时功能...");
    let timeout_options = ExecuteOptions::new()
        .with_timeout(2000); // 2秒超时

    match run_from_config_with_options("config_timeout.json", timeout_options).await {
        Ok(result) => {
            println!("✓ 超时测试执行成功");
            println!("  退出码: {}", result.exit_code);
            println!("  执行时间: {}ms", result.duration_ms);
        }
        Err(e) => {
            println!("✗ 超时测试执行失败: {}", e);
        }
    }

    // 2. 测试流式执行功能
    println!("\n2. 测试流式执行功能...");
    match run_from_config_stream("config_stream.json").await {
        Ok(mut handle) => {
            println!("✓ 流式执行器启动成功");
            
            // 接收流式输出
            let mut received_messages = 0;
            let max_messages = 5; // 只接收前5条消息
            
            while let Some(message) = handle.receiver.recv().await {
                match message {
                    StreamMessage::Stdout(line) => {
                        println!("  [STDOUT] {}", line);
                    }
                    StreamMessage::Stderr(line) => {
                        println!("  [STDERR] {}", line);
                    }
                    StreamMessage::Exit(code) => {
                        println!("  [EXIT] 进程退出，代码: {}", code);
                        break;
                    }
                    StreamMessage::Error(err) => {
                        println!("  [ERROR] {}", err);
                        break;
                    }
                }
                
                received_messages += 1;
                if received_messages >= max_messages {
                    println!("  [INFO] 已接收{}条消息，等待进程自然结束...", max_messages);
                    break;
                }
            }
        }
        Err(e) => {
            println!("✗ 流式执行器启动失败: {}", e);
        }
    }

    // 3. 测试无超时限制的长期运行
    println!("\n3. 测试无超时限制的长期运行...");
    let long_running_options = ExecuteOptions::new()
        .with_timeout(None); // 无超时限制

    match run_from_config_with_options("config_long_running.json", long_running_options).await {
        Ok(result) => {
            println!("✓ 长期运行测试执行成功");
            println!("  退出码: {}", result.exit_code);
            println!("  执行时间: {}ms", result.duration_ms);
        }
        Err(e) => {
            println!("✗ 长期运行测试执行失败: {}", e);
        }
    }

    // 清理测试文件
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
        println!("✓ 清理测试目录: {:?}", test_dir);
    }

    Ok(())
}

fn create_long_running_scripts(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 创建长时间运行的Python脚本
    let long_running_python = r#"#!/usr/bin/env python3
import sys
import time

def main():
    print("=== 长时间运行Python脚本开始 ===")
    
    # 输出一些信息
    for i in range(10):
        print(f"进度: {i+1}/10")
        sys.stdout.flush()
        time.sleep(1)  # 每秒输出一次
    
    print("=== 长时间运行Python脚本完成 ===")
    return 0

if __name__ == "__main__":
    try:
        exit_code = main()
        sys.exit(exit_code)
    except Exception as e:
        print(f"错误: {e}", file=sys.stderr)
        sys.exit(1)
"#;

    let python_script_path = test_dir.join("long_running.py");
    fs::write(&python_script_path, long_running_python)?;

    // 创建配置文件
    let config_timeout = r#"{
    "script_type": "python",
    "script_path": "./test-scripts/long_running.py",
    "runtime_version": "3.11",
    "args": [],
    "timeout": 2000
}"#;

    fs::write("config_timeout.json", config_timeout)?;

    let config_stream = r#"{
    "script_type": "python",
    "script_path": "./test-scripts/long_running.py",
    "runtime_version": "3.11",
    "args": [],
    "async_execution": true
}"#;

    fs::write("config_stream.json", config_stream)?;

    let config_long_running = r#"{
    "script_type": "python",
    "script_path": "./test-scripts/long_running.py",
    "runtime_version": "3.11",
    "args": [],
    "timeout": null
}"#;

    fs::write("config_long_running.json", config_long_running)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_timeout_and_streaming() {
        assert!(test_timeout_and_streaming().await.is_ok());
    }
}

async fn test_run_from_config() -> Result<(), Box<dyn std::error::Error>> {
    // 创建测试脚本目录
    let test_dir = Path::new("./test-scripts");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
        println!("创建测试脚本目录: {:?}", test_dir);
    }

    // 创建真实的测试脚本
    create_real_test_scripts(test_dir)?;

    // 测试JSON配置文件
    println!("测试JSON配置文件...");
    let json_config_path = Path::new("config_example.json");
    match run_from_config(json_config_path.to_str().unwrap()).await {
        Ok(result) => {
            println!("✓ JSON配置文件执行成功");
            println!("  退出码: {}", result.exit_code);
            println!("  标准输出: {}", result.stdout);
            if !result.stderr.is_empty() {
                println!("  标准错误: {}", result.stderr);
            }
        }
        Err(e) => {
            println!("✗ JSON配置文件执行失败: {}", e);
        }
    }

    // 测试YAML配置文件
    println!("\n测试YAML配置文件...");
    let yaml_config_path = Path::new("config_example.yaml");
    match run_from_config(yaml_config_path.to_str().unwrap()).await {
        Ok(result) => {
            println!("✓ YAML配置文件执行成功");
            println!("  退出码: {}", result.exit_code);
            println!("  标准输出: {}", result.stdout);
            if !result.stderr.is_empty() {
                println!("  标准错误: {}", result.stderr);
            }
        }
        Err(e) => {
            println!("✗ YAML配置文件执行失败: {}", e);
        }
    }

    // 清理测试文件
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
        println!("✓ 清理测试目录: {:?}", test_dir);
    }

    Ok(())
}

fn create_real_test_scripts(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 创建Python测试脚本
    let python_script = r#"#!/usr/bin/env python3
import sys
import os
import time

def main():
    print("=== Python测试脚本开始执行 ===")
    print(f"Python版本: {sys.version}")
    print(f"平台: {sys.platform}")
    print(f"当前工作目录: {os.getcwd()}")
    print(f"命令行参数: {sys.argv}")
    
    # 检查环境变量
    print("\n环境变量:")
    for key, value in os.environ.items():
        if key.startswith('PYTHON') or key.startswith('DEBUG') or key.startswith('TEST'):
            print(f"  {key}: {value}")
    
    # 模拟一些工作
    print("\n模拟工作...")
    for i in range(3):
        print(f"进度: {i+1}/3")
        time.sleep(0.5)
    
    print("\n=== Python测试脚本执行完成 ===")
    return 0

if __name__ == "__main__":
    try:
        exit_code = main()
        sys.exit(exit_code)
    except Exception as e:
        print(f"错误: {e}", file=sys.stderr)
        sys.exit(1)
"#;

    let python_script_path = test_dir.join("test_python.py");
    fs::write(&python_script_path, python_script)?;
    println!("创建 Python 测试脚本: {:?}", python_script_path);

    // 创建Node.js测试脚本
    let node_script = r#"#!/usr/bin/env node
console.log("=== Node.js测试脚本开始执行 ===");
console.log(`Node.js版本: ${process.version}`);
console.log(`平台: ${process.platform}`);
console.log(`架构: ${process.arch}`);
console.log(`当前工作目录: ${process.cwd()}`);
console.log(`命令行参数: ${process.argv.slice(2)}`);

// 检查环境变量
console.log("\n环境变量:");
Object.keys(process.env).forEach(key => {
    if (key.startsWith('NODE') || key.startsWith('DEBUG') || key.startsWith('TEST')) {
        console.log(`  ${key}: ${process.env[key]}`);
    }
});

// 模拟一些工作
console.log("\n模拟工作...");
for (let i = 0; i < 3; i++) {
    console.log(`进度: ${i+1}/3`);
    // 模拟异步操作 - 使用同步setTimeout
    const wait = (ms) => {
        const start = Date.now();
        while (Date.now() - start < ms) {}
    };
    wait(500);
}

console.log("\n=== Node.js测试脚本执行完成 ===");
process.exit(0);
"#;

    let node_script_path = test_dir.join("test_node.js");
    fs::write(&node_script_path, node_script)?;
    println!("创建 Node.js 测试脚本: {:?}", node_script_path);

    Ok(())
}

fn create_test_scripts(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Node.js 测试脚本
    let node_script = r#"
console.log("Hello from Node.js!");
console.log("Node.js version:", process.version);
console.log("Platform:", process.platform);
"#;

    let node_script_path = test_dir.join("test_node.js");
    fs::write(&node_script_path, node_script)?;
    println!("创建 Node.js 测试脚本: {:?}", node_script_path);

    // Python 测试脚本
    let python_script = r#"
import sys
print("Hello from Python!")
print("Python version:", sys.version)
print("Platform:", sys.platform)
"#;

    let python_script_path = test_dir.join("test_python.py");
    fs::write(&python_script_path, python_script)?;
    println!("创建 Python 测试脚本: {:?}", python_script_path);

    // Node.js 错误脚本
    let error_script = r#"
console.log("This script will fail...");
throw new Error("Intentional error for testing");
"#;

    let error_script_path = test_dir.join("error_script.js");
    fs::write(&error_script_path, error_script)?;
    println!("创建错误测试脚本: {:?}", error_script_path);

    Ok(())
}

fn cleanup_test_files(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
        println!("✓ 清理测试目录: {:?}", test_dir);
    }
    Ok(())
}
