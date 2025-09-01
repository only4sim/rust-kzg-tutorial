# Rust KZG 教程

## 📖 项目概述

这是一个独立的 **KZG (Kate-Zaverucha-Goldberg) 承诺方案** 学习教程，专门为想要深入理解零知识证明和多项式承诺方案的开发者设计。

本教程使用 [rust-kzg](https://github.com/grandinetech/rust-kzg) 库作为依赖，提供了完整的实践示例和理论解释。

## 🚀 快速开始

### 环境要求
- Rust 1.75.0+
- 支持的操作系统：macOS, Linux, Windows

### 安装和运行

```bash
# 克隆项目（如果还没有）
git clone https://github.com/yourusername/rust-kzg-tutorial
cd rust-kzg-tutorial

# 运行第一个示例
cargo run --example hello_kzg

# 运行密码学基础示例
cargo run --example chapter01_basics

# 运行所有测试
cargo test
```

## 📚 教程内容

### 🏃‍♂️ 快速入门示例

1. **`hello_kzg.rs`** - 完整的 KZG 工作流程演示
   - 受信任设置加载
   - Blob 数据创建
   - KZG 承诺生成
   - 证明生成和验证
   - 性能统计和数据分析

2. **`chapter01_basics.rs`** - 椭圆曲线密码学基础
   - 标量运算
   - 群元素操作
   - 椭圆曲线点运算
   - 多项式基础操作

### 📖 理论文档

- [`docs/TUTORIAL_OUTLINE.md`](docs/TUTORIAL_OUTLINE.md) - 完整教程大纲
- [`docs/chapter01_cryptography_basics.md`](docs/chapter01_cryptography_basics.md) - 密码学基础理论
- [`docs/chapter10_environment_setup.md`](docs/chapter10_environment_setup.md) - 环境配置指南

## 🎯 学习路径

### 🔰 初学者路径
1. 先运行 `hello_kzg` 示例，感受完整的 KZG 工作流程
2. 学习 `chapter01_basics` 了解数学基础
3. 阅读理论文档，深入理解原理

### 🚀 实践路径
1. 运行所有示例代码
2. 修改参数，观察性能变化
3. 尝试添加自己的测试用例
4. 探索不同的 Blob 数据模式

## ⚡ 性能基准

基于 M 系列 MacBook 的典型性能：

| 操作 | 耗时 | 数据大小 |
|------|------|----------|
| KZG 承诺生成 | ~14ms | 48 字节 |
| KZG 证明生成 | ~89ms | 48 字节 |
| 证明验证 | ~9ms | - |
| Blob 数据 | - | 4096 元素 (128KB) |

## 🔧 技术架构

### 依赖库
- **rust-kzg-blst**: 基于 BLST 的 KZG 实现
- **kzg**: KZG 核心接口和算法

### 项目结构
```
rust-kzg-tutorial/
├── Cargo.toml          # 项目配置和依赖
├── assets/             # 受信任设置文件
│   └── trusted_setup.txt
├── examples/           # 示例代码
│   ├── hello_kzg.rs   # 完整 KZG 流程演示
│   └── chapter01_basics.rs # 密码学基础
├── docs/              # 教程文档
└── README.md          # 项目说明
```

## 🧪 测试验证

项目包含完整的测试套件：

```bash
# 运行所有测试
cargo test

# 运行特定示例的测试
cargo test --example hello_kzg -- --nocapture
cargo test --example chapter01_basics -- --nocapture
```

### 测试覆盖
- ✅ Blob 数据创建和验证
- ✅ KZG 承诺一致性测试
- ✅ 完整工作流程验证
- ✅ 椭圆曲线数学属性验证

## 🔍 深入学习

### 相关资源
- [KZG 原始论文](https://www.iacr.org/archive/asiacrypt2010/6477178/6477178.pdf)
- [以太坊 EIP-4844](https://eips.ethereum.org/EIPS/eip-4844)
- [多项式承诺方案概述](https://dankradfeist.de/ethereum/2020/06/16/kate-polynomial-commitments.html)

### 扩展实验
1. 尝试不同大小的 Blob 数据
2. 测试批量验证性能
3. 比较不同椭圆曲线的性能
4. 实现自定义的多项式操作

## 💡 常见问题

### Q: 为什么需要受信任设置？
A: KZG 承诺方案需要一个通用的参考字符串 (CRS)，这个设置必须在可信环境中生成，确保没有人知道设置过程中的随机数。

### Q: Blob 数据有什么限制？
A: Blob 必须包含恰好 4096 个有效的域元素，每个元素必须在 BLS12-381 的标量域内。

### Q: 如何优化性能？
A: 可以考虑：
- 使用并行计算
- 批量操作
- 硬件加速（GPU）
- 预计算优化

## 🤝 贡献指南

我们欢迎所有形式的贡献：

1. **报告问题**: 发现 bug 或有改进建议
2. **改进文档**: 让教程更容易理解
3. **添加示例**: 展示更多应用场景
4. **性能优化**: 提升代码效率

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [rust-kzg](https://github.com/grandinetech/rust-kzg) 项目提供了优秀的 KZG 实现
- 以太坊基金会对 KZG 研究的支持
- EPF Cohort 6 对本项目的指导和支持

---

**开始你的 KZG 学习之旅吧！** 🎯

```bash
cargo run --example hello_kzg
```
