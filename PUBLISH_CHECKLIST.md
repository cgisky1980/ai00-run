# AI00-Run 发布到 crates.io 检查清单

## 发布前检查
- [x] 项目编译通过 (`cargo check`)
- [x] 代码格式化完成 (`cargo fmt`)
- [x] 代码质量检查通过 (`cargo clippy`)
- [x] Cargo.toml 配置正确
- [x] README.md 文档完整
- [x] LICENSE 文件存在
- [x] 所有测试通过

## Cargo.toml 配置检查
- [x] `name` = "ai00-run"
- [x] `version` = "0.1.0"
- [x] `edition` = "2021"
- [x] `authors` = ["AI00 Team"]
- [x] `license` = "MIT OR Apache-2.0"
- [x] `description` = "A Rust library for unified runtime management of Node.js, Python, and Rust environments"
- [x] `repository` = "https://github.com/cgisky1980/ai00-run"
- [x] `readme` = "README.md"
- [x] `keywords` = ["runtime", "nodejs", "python", "script", "management", "virtual-environment"]
- [x] `categories` = ["command-line-utilities", "development-tools", "web-programming"]

## 发布步骤

### 1. 注册 crates.io 账户（如果还没有）
```bash
cargo login
```

### 2. 验证包信息
```bash
cargo publish --dry-run
```

### 3. 实际发布
```bash
cargo publish
```

### 4. 发布后验证
```bash
cargo search ai00-run
```

## 注意事项
- 确保版本号符合语义化版本规范
- 发布后版本号不能修改，只能发布新版本
- 确保所有依赖都是稳定的版本
- 检查是否有敏感信息（如API密钥）在代码中

## 版本管理
- 每次发布前更新版本号
- 遵循语义化版本规范：MAJOR.MINOR.PATCH
- 重大变更：MAJOR+1
- 新功能：MINOR+1
- Bug修复：PATCH+1