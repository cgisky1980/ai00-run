//! 示例：根据配置文件运行脚本
//!
//! 演示如何使用 `run_from_config` 函数根据配置文件自动运行脚本。

use ai00_run::run;

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    println!("=== 根据配置文件运行脚本示例 ===");

    // 示例1：运行JSON配置的Python脚本
    println!("\n1. 运行JSON配置的Python脚本:");
    match run::run_from_config("../config_example.json").await {
        Ok(result) => {
            println!("执行成功!");
            println!("退出代码: {}", result.exit_code);
            println!("标准输出: {}", result.stdout);
            println!("标准错误: {}", result.stderr);
            println!("执行耗时: {}ms", result.duration_ms);
        }
        Err(e) => {
            println!("执行失败: {}", e);
        }
    }

    // 示例2：运行YAML配置的Node.js脚本
    println!("\n2. 运行YAML配置的Node.js脚本:");
    match run::run_from_config("../config_example.yaml").await {
        Ok(result) => {
            println!("执行成功!");
            println!("退出代码: {}", result.exit_code);
            println!("标准输出: {}", result.stdout);
            println!("标准错误: {}", result.stderr);
            println!("执行耗时: {}ms", result.duration_ms);
        }
        Err(e) => {
            println!("执行失败: {}", e);
        }
    }

    // 示例3：使用ScriptRunner实例运行配置
    println!("\n3. 使用ScriptRunner实例运行配置:");
    let runner = run::ScriptRunner::new();
    match runner.run_from_config("../config_example.json").await {
        Ok(result) => {
            println!("执行成功!");
            println!("状态: {}", result.status());
            println!("是否成功: {}", result.is_success());
        }
        Err(e) => {
            println!("执行失败: {}", e);
        }
    }

    // 示例4：生成配置模板
    println!("\n4. 生成配置模板:");
    let config_manager = run::ConfigManager::new();

    // 生成Python配置模板
    let python_config =
        config_manager.generate_template("python", "../python_template.json")?;
    println!("Python配置模板已生成");
    println!(
        "配置摘要:\n{}",
        config_manager.get_config_summary(&python_config)
    );

    // 生成Node.js配置模板
    let node_config = config_manager.generate_template("node", "../node_template.json")?;
    println!("Node.js配置模板已生成");
    println!(
        "配置摘要:\n{}",
        config_manager.get_config_summary(&node_config)
    );

    // 示例5：验证配置文件
    println!("\n5. 验证配置文件:");
    match config_manager
        .validate_config_file("../config_example.json", "json")
        .await
    {
        Ok((valid, errors)) => {
            if valid {
                println!("配置文件验证通过!");
            } else {
                println!("配置文件验证失败:");
                for error in errors {
                    println!("  - {}", error);
                }
            }
        }
        Err(e) => {
            println!("验证失败: {}", e);
        }
    }

    println!("\n=== 示例完成 ===");
    Ok(())
}
