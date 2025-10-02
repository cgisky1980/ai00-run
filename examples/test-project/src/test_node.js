#!/usr/bin/env node
/**
 * 测试Node.js脚本
 */

console.log("=== Node.js测试脚本开始执行 ===");
console.log(`Node.js版本: ${process.version}`);
console.log(`平台: ${process.platform}`);
console.log(`架构: ${process.arch}`);
console.log(`当前工作目录: ${process.cwd()}`);
console.log(`命令行参数: ${process.argv.slice(2)}`);

// 检查环境变量
console.log("\n环境变量:");
Object.keys(process.env).forEach(key => {
    if (key.startsWith('NODE') || key.startsWith('DEBUG')) {
        console.log(`  ${key}: ${process.env[key]}`);
    }
});

// 模拟一些工作
console.log("\n模拟工作...");
for (let i = 0; i < 3; i++) {
    console.log(`进度: ${i+1}/3`);
    // 模拟异步操作 - 使用同步等待
    const wait = (ms) => {
        const start = Date.now();
        while (Date.now() - start < ms) {}
    };
    wait(500);
}

// 测试错误处理
if (process.argv.includes('--error')) {
    console.error("模拟错误发生");
    process.exit(1);
}

console.log("\n=== Node.js测试脚本执行完成 ===");
process.exit(0);