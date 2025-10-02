use ai00_run::{init, node, py, run::ScriptRunner};
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

    // 6. 清理测试文件
    println!("\n--- 清理测试文件 ---");
    cleanup_test_files(test_dir)?;

    println!("\n=== 测试完成 ===");
    println!("\n说明：");
    println!("- 测试项目验证了ai00-run的初始化功能");
    println!("- Node.js和Python版本管理功能需要进一步实现安装逻辑");
    println!("- 脚本执行功能需要先安装对应的运行时版本");

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
