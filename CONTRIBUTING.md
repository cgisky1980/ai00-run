# 贡献指南

感谢您对AI00 Run项目的关注！我们欢迎各种形式的贡献。

## 开发环境设置

1. **克隆仓库**
   ```bash
   git clone https://github.com/ai00-run/ai00-run.git
   cd ai00-run
   ```

2. **安装Rust工具链**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **构建项目**
   ```bash
   cargo build --release
   ```

4. **运行测试**
   ```bash
   cargo test
   ```

## 贡献流程

1. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **进行更改**
   - 遵循Rust编码规范
   - 添加适当的测试用例
   - 更新相关文档

3. **代码质量检查**
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```

4. **提交更改**
   ```bash
   git add .
   git commit -m "描述您的更改"
   git push origin feature/your-feature-name
   ```

5. **创建Pull Request**

## 代码规范

- 使用`cargo fmt`格式化代码
- 使用`cargo clippy`检查代码质量
- 为公共API添加文档注释
- 为新功能添加测试用例
- 遵循Rust的命名约定

## 报告问题

如果您发现bug或有功能建议，请通过GitHub Issues报告。

## 许可证

通过贡献代码，您同意您的贡献将根据项目的MIT许可证进行授权。