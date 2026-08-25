//! dsh PoC: 用 ai00-run 安装 node 并运行 dsh web
//!
//! Phase 0.1 验证：不依赖系统 node，由 ai00-run 维护运行时。
//! 用法: cargo run --release --example dsh_poc -- [install|run]

use ai00_run::node::installer::NodeInstaller;
use std::path::PathBuf;

const NODE_VERSION: &str = "22.23.2";

fn node_root() -> PathBuf {
    // 统一安装根目录（与 NodeInstaller 默认一致）：~/.ai00-run/node
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ai00-run")
        .join("node")
}

fn npx_path(installer: &NodeInstaller) -> PathBuf {
    if cfg!(windows) {
        installer
            .install_dir()
            .join(format!("v{NODE_VERSION}"))
            .join("npx.cmd")
    } else {
        installer
            .install_dir()
            .join(format!("v{NODE_VERSION}"))
            .join("bin")
            .join("npx")
    }
}

#[tokio::main]
async fn main() -> ai00_run::Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "install".into());
    let installer = NodeInstaller::new(Some(node_root()));

    match mode.as_str() {
        // 第一步：安装 node 并验证可执行
        "install" => {
            if installer.is_installed(NODE_VERSION).await? {
                println!("node {} already installed", NODE_VERSION);
            } else {
                println!("installing node {} via ai00-run ...", NODE_VERSION);
                installer.install(NODE_VERSION).await?;
            }
            let node_path = installer.get_node_path(NODE_VERSION).await?;
            println!("node path: {}", node_path.display());

            // 验证: node --version
            let out = tokio::process::Command::new(&node_path)
                .arg("--version")
                .output()
                .await
                .map_err(ai00_run::Error::Io)?;
            println!(
                "node --version => {}",
                String::from_utf8_lossy(&out.stdout).trim()
            );
            println!("npx  path: {}", npx_path(&installer).display());
        }
        // 第二步：用托管 node 启动 dsh web（前台运行）
        "run" => {
            let npx = npx_path(&installer);
            println!("starting dsh web via {} ...", npx.display());
            let status = tokio::process::Command::new(&npx)
                .args(["-y", "@deepseek-ai/dsh", "web"])
                .status()
                .await
                .map_err(ai00_run::Error::Io)?;
            println!("dsh exited with {:?}", status.code());
        }
        _ => eprintln!("unknown mode: {mode} (use install|run)"),
    }
    Ok(())
}
