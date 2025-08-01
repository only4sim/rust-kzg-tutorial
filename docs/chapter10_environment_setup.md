# 第10章：环境搭建与基础使用

> **学习目标**: 掌握 Rust KZG 项目的使用方法，完成从零开始的环境搭建，编写第一个 KZG 程序

---

## 10.1 开发环境配置

### 🛠️ 系统要求

在开始之前，确保你的系统满足以下要求：

#### 基础环境
- **操作系统**: Linux, macOS, 或 Windows (推荐 Linux/macOS)
- **Rust 版本**: 1.70.0 或更高版本
- **内存**: 至少 4GB RAM (推荐 8GB+)
- **存储**: 至少 2GB 可用空间

#### 软件依赖
```bash
# 检查 Rust 版本
rustc --version

# 如果 Rust 未安装或版本过低，请安装/更新
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安装必要的工具链
rustup component add rustfmt clippy
```

### 📦 项目获取与编译

#### 1. 克隆项目仓库

```bash
# 克隆官方仓库
git clone https://github.com/grandinetech/rust-kzg.git
cd rust-kzg

# 查看项目结构
ls -la
```

**项目结构解析**：
```
rust-kzg/
├── Cargo.toml          # 工作区配置文件
├── Cargo.lock          # 依赖锁定文件
├── README.md           # 项目说明
├── kzg/               # 核心 Trait 定义
├── blst/              # BLST 后端实现（推荐）
├── arkworks3/         # Arkworks v0.3 后端
├── arkworks4/         # Arkworks v0.4 后端  
├── ckzg/              # C-KZG 兼容层
├── examples/          # 示例代码
└── tutorial/          # 教程文件（新增）
```

#### 2. 依赖安装与编译

```bash
# 编译所有后端（首次编译需要较长时间）
cargo build

# 仅编译 BLST 后端（推荐用于学习）
cargo build -p rust-kzg-blst

# 编译并运行基础示例
cargo run --example basic_example

# 运行测试确保环境正确
cargo test -p rust-kzg-blst
```

**编译选项说明**：
- `--release`: 优化编译，性能更高但编译时间更长
- `--features parallel`: 启用并行化支持
- `--features c_bindings`: 启用 C 语言绑定

#### 3. 受信任设置文件

KZG 方案需要受信任设置文件才能工作：

```bash
# 下载测试用的受信任设置文件
mkdir -p assets
cd assets

# 下载小型测试文件 (约 1MB)
wget https://github.com/ethereum/c-kzg-4844/raw/main/src/trusted_setup.txt

# 或者使用 curl
curl -L -o trusted_setup.txt \
  https://github.com/ethereum/c-kzg-4844/raw/main/src/trusted_setup.txt

cd ..
```

### 🔧 IDE 配置 (可选但推荐)

#### VS Code 配置

如果使用 VS Code，推荐安装以下扩展：

```json
// .vscode/extensions.json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "serayuzgur.crates"
    ]
}
```

已配置的任务文件：
```json
// .vscode/tasks.json (已存在)
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "cargo build",
            "type": "shell",
            "command": "cargo",
            "args": ["build"],
            "group": "build"
        },
        {
            "label": "cargo test",
            "type": "shell", 
            "command": "cargo",
            "args": ["test"],
            "group": "test"
        }
    ]
}
```

---

## 10.2 第一个 KZG 程序

### 🚀 Hello KZG World

让我们从最简单的示例开始：

```rust
// examples/hello_kzg.rs
use rust_kzg_blst::{
    eip_4844::{
        blob_to_kzg_commitment_rust, 
        compute_blob_kzg_proof_rust,
        verify_blob_kzg_proof_rust,
        load_trusted_setup_filename_rust
    },
    types::kzg_settings::FsKZGSettings,
    types::fr::FsFr,
    Fr,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Hello KZG World!");
    println!("=".repeat(30));

    // 1. 加载受信任设置
    let kzg_settings = load_trusted_setup_from_file()?;
    println!("✅ 受信任设置加载成功");

    // 2. 创建测试数据 (Blob)
    let blob = create_test_blob()?;
    println!("✅ 测试 Blob 创建成功");

    // 3. 生成承诺
    let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
    println!("✅ KZG 承诺生成成功");

    // 4. 生成证明
    let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings)?;
    println!("✅ KZG 证明生成成功");

    // 5. 验证证明
    let is_valid = verify_blob_kzg_proof_rust(
        &blob, &commitment, &proof, &kzg_settings
    )?;
    
    if is_valid {
        println!("🎉 证明验证成功！");
        println!("你已经成功完成了第一个 KZG 操作！");
    } else {
        println!("❌ 证明验证失败");
    }

    Ok(())
}

/// 加载受信任设置文件
fn load_trusted_setup_from_file() -> Result<FsKZGSettings, Box<dyn std::error::Error>> {
    // 尝试多个可能的路径
    let possible_paths = [
        "./assets/trusted_setup.txt",
        "../assets/trusted_setup.txt", 
        "../../assets/trusted_setup.txt",
        "./trusted_setup.txt",
    ];

    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            println!("🔍 找到受信任设置文件: {}", path);
            return Ok(load_trusted_setup_filename_rust(path)?);
        }
    }

    Err("未找到受信任设置文件，请确保 trusted_setup.txt 存在".into())
}

/// 创建有效的测试 Blob 数据
fn create_test_blob() -> Result<Vec<FsFr>, String> {
    const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
    let mut blob = Vec::with_capacity(FIELD_ELEMENTS_PER_BLOB);

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        // 创建有效的域元素：使用小值确保在域内
        let mut bytes = [0u8; 32];
        let value = (i % 255) as u8; // 确保值在有效范围内
        bytes[31] = value;
        
        let element = FsFr::from_bytes(&bytes)
            .map_err(|e| format!("创建域元素失败: {}", e))?;
        blob.push(element);
    }

    Ok(blob)
}
```

### 🏃‍♂️ 运行第一个程序

```bash
# 创建示例文件
cat > examples/hello_kzg.rs << 'EOF'
[上面的代码内容]
EOF

# 编译并运行
cargo run --example hello_kzg

# 预期输出:
# 🎯 Hello KZG World!
# ==============================
# 🔍 找到受信任设置文件: ./assets/trusted_setup.txt
# ✅ 受信任设置加载成功
# ✅ 测试 Blob 创建成功
# ✅ KZG 承诺生成成功
# ✅ KZG 证明生成成功
# 🎉 证明验证成功！
# 你已经成功完成了第一个 KZG 操作！
```

### 📖 代码详解

#### 1. 受信任设置加载
```rust
let kzg_settings = load_trusted_setup_filename_rust("path/to/trusted_setup.txt")?;
```
- **作用**: 加载预计算的椭圆曲线点
- **内容**: 包含 $[G_1, \tau G_1, \tau^2 G_1, \ldots]$ 和 $[G_2, \tau G_2]$
- **重要性**: 这是 KZG 方案的核心，没有它无法进行任何操作

#### 2. Blob 数据创建
```rust
let blob = create_test_blob()?;
```
- **Blob**: 4096 个域元素的数组，代表多项式的求值
- **域元素**: BLS12-381 标量域 $F_r$ 中的元素
- **注意**: 必须确保所有字节都表示有效的域元素

#### 3. 承诺生成
```rust
let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
```
- **数学原理**: $C = \sum_{i=0}^{n-1} f_i \cdot \tau^i G_1$
- **输入**: Blob 数据 + 受信任设置
- **输出**: 48 字节的 G1 群元素

#### 4. 证明生成
```rust
let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings)?;
```
- **目的**: 证明承诺确实对应给定的 blob
- **挑战**: 使用 Fiat-Shamir 变换生成随机挑战点
- **输出**: 48 字节的 G1 群元素

#### 5. 证明验证
```rust
let is_valid = verify_blob_kzg_proof_rust(&blob, &commitment, &proof, &kzg_settings)?;
```
- **验证等式**: 使用双线性配对进行验证
- **效率**: 常数时间验证，与 blob 大小无关

---

## 10.3 常见问题与解决方案

### ❌ 编译错误排查

#### 问题 1: "Invalid scalar" 错误
```
Error: Invalid scalar
```

**原因**: 字节数组不表示有效的域元素
**解决方案**:
```rust
// ❌ 错误的做法
let invalid_bytes = [255u8; 32]; // 可能超出域大小
let scalar = FsFr::from_bytes(&invalid_bytes)?; // 可能失败

// ✅ 正确的做法  
let mut valid_bytes = [0u8; 32];
valid_bytes[31] = 42; // 使用小值
let scalar = FsFr::from_bytes(&valid_bytes)?; // 安全
```

#### 问题 2: 找不到受信任设置文件
```
Error: 未找到受信任设置文件
```

**解决方案**:
```bash
# 确保文件存在
ls -la assets/trusted_setup.txt

# 如果不存在，重新下载
mkdir -p assets
cd assets
wget https://github.com/ethereum/c-kzg-4844/raw/main/src/trusted_setup.txt
```

#### 问题 3: 链接错误
```
error: linking with `cc` failed
```

**解决方案** (Linux):
```bash
# 安装必要的构建工具
sudo apt update
sudo apt install build-essential

# Ubuntu/Debian
sudo apt install gcc g++ libc6-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
```

**解决方案** (macOS):
```bash
# 安装 Xcode 命令行工具
xcode-select --install

# 或安装完整的 Xcode
```

### 🐛 运行时错误处理

#### 内存不足
```rust
// 监控内存使用
fn monitor_memory_usage() {
    use std::alloc::{GlobalAlloc, Layout, System};
    
    // 在生产环境中实现内存监控
    println!("内存使用监控 - 实现中...");
}
```

#### 性能优化建议
```rust
// 使用 rayon 进行并行处理
#[cfg(feature = "parallel")]
use rayon::prelude::*;

// 并行化 blob 处理
#[cfg(feature = "parallel")]
fn process_blobs_parallel(blobs: &[Vec<FsFr>]) -> Vec<Result<G1, String>> {
    blobs.par_iter()
        .map(|blob| blob_to_kzg_commitment_rust(blob, &settings))
        .collect()
}
```

---

## 10.4 调试技巧与工具

### 🔍 调试器使用

#### LLDB 调试器 (推荐)
```bash
# 编译带调试信息的版本
cargo build --example hello_kzg

# 使用 LLDB 调试
lldb target/debug/examples/hello_kzg

# 在 LLDB 中设置断点
(lldb) b hello_kzg.rs:25
(lldb) run
```

#### GDB 调试器 (Linux)
```bash
# 使用 GDB
gdb target/debug/examples/hello_kzg

# 设置断点并运行
(gdb) break main
(gdb) run
```

### 📝 日志输出最佳实践

```rust
// 添加到 Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.10"

// 在代码中使用
use log::{info, debug, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    info!("🚀 开始 KZG 操作");
    
    let kzg_settings = load_trusted_setup_from_file()?;
    debug!("受信任设置包含 {} 个 G1 点", kzg_settings.g1_count());
    
    // ... 其他代码
    
    Ok(())
}

// 运行时设置日志级别
// RUST_LOG=debug cargo run --example hello_kzg
```

### 🧪 单元测试编写

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_creation() -> Result<(), String> {
        let blob = create_test_blob()?;
        
        // 验证 blob 大小
        assert_eq!(blob.len(), 4096);
        
        // 验证所有元素都是有效的域元素
        for (i, element) in blob.iter().enumerate() {
            assert!(!element.is_zero() || i == 0, "第 {} 个元素不应为零", i);
        }
        
        Ok(())
    }

    #[test]
    fn test_kzg_commitment_consistency() -> Result<(), Box<dyn std::error::Error>> {
        let settings = load_trusted_setup_from_file()?;
        let blob = create_test_blob()?;
        
        // 多次生成承诺应该得到相同结果
        let commitment1 = blob_to_kzg_commitment_rust(&blob, &settings)?;
        let commitment2 = blob_to_kzg_commitment_rust(&blob, &settings)?;
        
        assert!(commitment1.equals(&commitment2));
        
        Ok(())
    }
}

// 运行测试
// cargo test --example hello_kzg
```

### 📊 性能分析

```rust
use std::time::Instant;

fn benchmark_kzg_operations() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_trusted_setup_from_file()?;
    let blob = create_test_blob()?;
    
    // 测量承诺生成时间
    let start = Instant::now();
    let commitment = blob_to_kzg_commitment_rust(&blob, &settings)?;
    let commitment_time = start.elapsed();
    
    // 测量证明生成时间  
    let start = Instant::now();
    let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &settings)?;
    let proof_time = start.elapsed();
    
    // 测量验证时间
    let start = Instant::now();
    let _is_valid = verify_blob_kzg_proof_rust(&blob, &commitment, &proof, &settings)?;
    let verify_time = start.elapsed();
    
    println!("⏱️  性能统计:");
    println!("   承诺生成: {:?}", commitment_time);
    println!("   证明生成: {:?}", proof_time);
    println!("   证明验证: {:?}", verify_time);
    
    Ok(())
}
```

---

## 📚 本章总结

通过本章学习，你已经：

### ✅ 完成的任务
1. **环境搭建**: 安装 Rust、克隆项目、编译代码
2. **第一个程序**: 编写并运行完整的 KZG 示例
3. **错误处理**: 学会诊断和解决常见问题
4. **调试技能**: 掌握调试器、日志、测试的使用

### 🎯 核心概念
- **受信任设置**: KZG 方案的基础设施
- **Blob 数据**: 多项式求值的载体
- **承诺-证明-验证**: KZG 的三个核心步骤

### 🚀 下章预告

第11章将深入探讨 **高级 API 使用指南**，包括：
- 受信任设置的深度管理
- 多种后端的性能对比
- 批量操作的优化技巧
- 内存管理和性能调优

### 💡 练习建议

1. **修改示例**: 尝试改变 blob 的大小和内容
2. **性能测试**: 比较不同数据大小的性能差异
3. **错误注入**: 故意引入错误，观察错误处理机制
4. **功能扩展**: 添加更多的统计信息和可视化输出

**下一章**: [第11章：高级 API 使用指南](chapter11_advanced_api.md)
