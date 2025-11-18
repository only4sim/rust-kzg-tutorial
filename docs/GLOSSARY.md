# Rust KZG 术语表
# Glossary

> 本术语表提供教程中使用的密码学、数学和区块链术语的中英文对照和解释。

---

## 使用说明 / How to Use

- **按字母顺序排列** (中文拼音)
- **标记**: 密码学 | 数学 | 区块链 | 编程
- **难度**: Basic | Intermediate | Advanced

---

## A

### Arkworks - Intermediate
**英文**: Arkworks
**全称**: Arkworks Cryptography Library
**解释**: 用 Rust 编写的零知识证明密码学库，是 rust-kzg 支持的后端之一。

**相关术语**: BLST, Backend

---

## B

### Backend (后端) - Intermediate
**英文**: Backend
**中文**: 密码学后端
**解释**: rust-kzg 库使用的底层密码学实现。不同的后端提供相同的接口但使用不同的优化策略。

**支持的后端**:
- BLST (最常用，性能优化)
- Arkworks (zkSNARK 友好)
- Constantine (Nim 实现)
- MCL (C++ 实现)
- ZKCrypto (纯 Rust)

**示例**:
```rust
use rust_kzg_blst::{types::fr::FsFr, types::g1::FsG1};  // BLST 后端
```

**相关章节**: 第4章、第8章

---

### BLS12-381 - Advanced
**英文**: BLS12-381 Curve
**全称**: Barreto-Lynn-Scott curve with embedding degree 12, 381-bit prime
**解释**: 专为配对密码学设计的椭圆曲线，提供 128 位安全强度。

**关键参数**:
- 基域大小: 381 位素数
- 嵌入度: k = 12
- 曲线方程: y² = x³ + 4

**为什么使用它**:
- 配对计算效率高
- 安全性经过充分验证
- 被以太坊、Zcash、Filecoin 等广泛采用

**相关章节**: 第1章

---

### Blob - Basic
**英文**: Blob
**中文**: 数据块
**全称**: Binary Large Object
**解释**: 在 EIP-4844 中，blob 是一个包含 4096 个域元素的大型数据块，用于存储 rollup 交易数据。

**规格**:
- 大小: 4096 个 Fr 元素
- 字节大小: 约 128 KB
- 用途: Layer 2 数据可用性

**示例**:
```rust
let blob: [FsFr; 4096] = [FsFr::zero(); 4096];
```

**相关术语**: EIP-4844, Commitment, Data Availability
**相关章节**: 第3章、第7章

---

### BLST - Intermediate
**英文**: BLST
**全称**: BLS Signatures Library
**解释**: 高性能的 BLS12-381 曲线实现，使用汇编优化。是 rust-kzg 的默认后端。

**特点**:
- C 语言实现，Rust 绑定
- 汇编级别优化
- 通过 Supranational 开发

**相关章节**: 第8章

---

## C

### Cell (单元) - Intermediate
**英文**: Cell
**中文**: 单元
**解释**: 在 EIP-7594 PeerDAS 中，blob 被分成 128 个 cell，每个包含 64 个域元素。

**规格**:
- 每个 blob: 128 个 cell
- 每个 cell: 64 个 Fr 元素
- 用途: 数据可用性采样

**相关术语**: Blob, PeerDAS, EIP-7594
**相关章节**: 第7章

---

### Commitment (承诺) - Advanced
**英文**: Commitment
**中文**: 承诺
**解释**: 密码学承诺是对数据的简洁绑定，具有隐藏性和绑定性。KZG 承诺将多项式承诺为单个群元素。

**性质**:
- **绑定性**: 无法找到两个不同的输入产生相同承诺
- **隐藏性**: 承诺不泄露原始数据
- **简洁性**: KZG 承诺固定为 48 字节（G1 点）

**数学表示**:
```
C = [p(τ)]₁ = p(τ) · G₁
```

**示例**:
```rust
let commitment = blob_to_kzg_commitment_rust(&blob, &settings)?;
```

**相关术语**: Proof, Polynomial
**相关章节**: 第2章

---

### Constantine - Intermediate
**英文**: Constantine
**中文**: -
**解释**: 用 Nim 语言编写的密码学库，rust-kzg 的支持后端之一。

**相关术语**: Backend, BLST

---

## D

### DAS (数据可用性采样) - Intermediate
**英文**: Data Availability Sampling
**中文**: 数据可用性采样
**解释**: 一种技术，允许轻客户端通过随机采样小部分数据来验证整个数据集的可用性。

**工作原理**:
1. 数据编码为 Reed-Solomon 纠错码
2. 节点随机下载少量样本
3. 高概率检测数据是否可用

**应用**: EIP-7594 PeerDAS

**相关章节**: 第7章

---

### Danksharding - Intermediate
**英文**: Danksharding
**中文**: Danksharding（以 Dankrad Feist 命名）
**解释**: 以太坊的分片设计方案，使用 KZG 承诺和数据可用性采样。

**阶段**:
- **Proto-Danksharding** (EIP-4844): 引入 blob 交易
- **Full Danksharding**: 完整的分片系统（未来）

**相关术语**: EIP-4844, Blob, Sharding
**相关章节**: 第3章

---

## E

### EIP-4844 - Basic
**英文**: Ethereum Improvement Proposal 4844
**中文**: 以太坊改进提案 4844
**别名**: Proto-Danksharding
**解释**: 引入 blob-carrying 交易的以太坊升级，为 rollup 提供低成本数据可用性。

**关键特性**:
- 新交易类型: Blob 交易
- 临时存储: Blob 数据仅保留约 18 天
- 降低成本: Rollup 数据成本降低 10-100 倍

**规范**:
- Blob 大小: 4096 字段元素
- 每个区块: 最多 6 个 blob (目标 3 个)

**相关章节**: 第3章

---

### EIP-7594 - Intermediate
**英文**: Ethereum Improvement Proposal 7594
**中文**: 以太坊改进提案 7594
**别名**: PeerDAS
**解释**: 通过对等数据可用性采样扩展 EIP-4844 的能力。

**改进**:
- 每个 blob 分成 128 个 cell
- 节点只需存储部分 cell
- 支持更多 blob (目标 16-32 个/区块)

**相关章节**: 第7章

---

### Elliptic Curve (椭圆曲线) - Intermediate
**英文**: Elliptic Curve
**中文**: 椭圆曲线
**解释**: 满足特定方程的点的集合，具有群结构，用于现代密码学。

**标准形式**:
```
y² = x³ + ax + b (mod p)
```

**优势**:
- 相同安全强度下密钥更短
- 计算效率高

**相关术语**: BLS12-381, Group
**相关章节**: 第1章

---

## F

### FFT (快速傅里叶变换) - Advanced
**英文**: Fast Fourier Transform
**中文**: 快速傅里叶变换
**解释**: 高效计算多项式求值和插值的算法。KZG 中用于多点求值。

**复杂度**:
- 朴素算法: O(n²)
- FFT: O(n log n)

**应用**:
- 多项式乘法
- Reed-Solomon 编码
- 批量证明生成

**相关章节**: 第13章（性能优化）

---

### Field Element (域元素) - Intermediate
**英文**: Field Element
**中文**: 域元素
**类型**: `Fr` (Field of r elements)
**解释**: 有限域中的元素，支持加法、减法、乘法、除法（除0外）。

**示例**:
```rust
let a = FsFr::from_u64(5);
let b = FsFr::from_u64(3);
let c = a.add(&b);  // 8
```

**相关术语**: Fr, Scalar
**相关章节**: 第1章

---

### Fiat-Shamir Transform - Advanced
**英文**: Fiat-Shamir Transform
**中文**: Fiat-Shamir 变换
**解释**: 将交互式证明转换为非交互式证明的技术，使用哈希函数模拟随机挑战。

**原理**:
- 交互式: 验证者发送随机挑战
- 非交互式: 用哈希函数 H(承诺) 生成挑战

**应用**: KZG 证明中的评估点生成

**相关章节**: 第2章

---

### Fr - Intermediate
**英文**: Fr (Field of r elements)
**中文**: 标量域
**解释**: BLS12-381 曲线的标量域，包含大约 2^255 个元素。

**用途**:
- 多项式系数
- 标量乘法
- 私钥

**Rust 类型**:
```rust
use rust_kzg_blst::types::fr::FsFr;
let scalar: FsFr = FsFr::from_u64(42);
```

**相关术语**: Field Element, Scalar
**相关章节**: 第1章

---

## G

### G1, G2 - Advanced
**英文**: G1, G2 Groups
**中文**: G1 群、G2 群
**解释**: BLS12-381 曲线上的两个椭圆曲线群，用于配对密码学。

**区别**:
- **G1**: 定义在基域 Fp 上的曲线点
  - 点大小: 48 字节（压缩）
  - 用途: KZG 承诺、证明

- **G2**: 定义在扩展域 Fp² 上的曲线点
  - 点大小: 96 字节（压缩）
  - 用途: 可信设置（SRS）

**配对**:
```
e: G1 × G2 → GT
```

**示例**:
```rust
use rust_kzg_blst::types::g1::FsG1;
use rust_kzg_blst::types::g2::FsG2;

let g1_point: FsG1 = FsG1::generator();
let g2_point: FsG2 = FsG2::generator();
```

**相关术语**: Pairing, BLS12-381
**相关章节**: 第1章

---

### Group (群) - Intermediate
**英文**: Group
**中文**: 群
**解释**: 一个集合配上一个运算，满足四个性质：封闭性、结合律、单位元、逆元。

**性质**:
1. **封闭性**: a, b ∈ G ⇒ a · b ∈ G
2. **结合律**: (a · b) · c = a · (b · c)
3. **单位元**: ∃ e, e · a = a
4. **逆元**: ∀ a, ∃ a⁻¹, a · a⁻¹ = e

**示例**: 椭圆曲线上的点加法构成群

**相关术语**: Elliptic Curve, G1, G2
**相关章节**: 第1章

---

### GPU Acceleration (GPU 加速) - Intermediate
**英文**: GPU Acceleration
**中文**: GPU 加速
**解释**: 使用图形处理器加速密码学计算，特别是多标量乘法 (MSM)。

**优势**:
- MSM 性能提升 10-100 倍
- 并行处理大量计算

**库**: SPPARK (Supranational 的 GPU 库)

**相关章节**: 第9章

---

## K

### KZG - Advanced
**英文**: KZG (Kate-Zaverucha-Goldberg)
**中文**: KZG 承诺方案
**解释**: 一种多项式承诺方案，允许对多项式进行简洁的承诺和高效的证明。

**关键特性**:
- **简洁性**: 承诺和证明都是单个群元素
- **效率**: 验证时间为 O(1)
- **可批处理**: 支持批量验证

**应用**:
- 以太坊 Proto-Danksharding
- zkSNARK
- 可验证计算

**相关术语**: Commitment, Polynomial, Proof
**相关章节**: 第2章

---

## M

### MCL - Intermediate
**英文**: MCL (Multi-precision and Cryptography Library)
**中文**: -
**解释**: C++ 实现的密码学库，rust-kzg 支持的后端之一。

**相关术语**: Backend, BLST

---

### MSM (多标量乘法) - Advanced
**英文**: Multi-Scalar Multiplication
**中文**: 多标量乘法
**解释**: 计算多个标量与对应群元素的乘法之和，KZG 中的核心计算。

**数学形式**:
```
MSM = s₁·P₁ + s₂·P₂ + ... + sₙ·Pₙ
```

**优化**:
- Pippenger 算法: O(n / log n) 群运算
- GPU 加速: 并行计算
- 预计算: 存储常用点的倍数

**性能关键点**: MSM 占 KZG 计算时间的 90%+

**相关章节**: 第9章、第13章

---

## P

### Pairing (配对) - Advanced
**英文**: Pairing (Bilinear Pairing)
**中文**: 双线性配对
**解释**: 一个满足双线性性质的映射，用于 KZG 验证。

**定义**:
```
e: G1 × G2 → GT
```

**双线性性质**:
```
e(aP, bQ) = e(P, Q)^(ab)
```

**应用**: KZG 证明验证公式
```
e([p(τ)]₁, [1]₂) = e([π]₁, [τ - z]₂) · e([y]₁, [1]₂)
```

**相关章节**: 第1章、第2章

---

### PeerDAS - Intermediate
**英文**: Peer Data Availability Sampling
**中文**: 对等数据可用性采样
**解释**: EIP-7594 的别名，通过 P2P 网络进行数据可用性采样。

**相关术语**: EIP-7594, DAS

---

### Polynomial (多项式) - Intermediate
**英文**: Polynomial
**中文**: 多项式
**解释**: 形如 p(x) = aₙxⁿ + ... + a₁x + a₀ 的数学表达式。KZG 承诺的是多项式。

**在 KZG 中的角色**:
- Blob 数据被编码为多项式的系数
- 承诺是对多项式的承诺
- 证明证明多项式在某点的值

**示例**:
```rust
let coefficients: Vec<FsFr> = vec![a0, a1, a2, a3];  // p(x) = a3x³ + a2x² + a1x + a0
```

**相关章节**: 第2章

---

### Proof (证明) - Advanced
**英文**: Proof
**中文**: 证明
**解释**: 在 KZG 中，证明是一个 G1 点，用于证明多项式在某点的求值。

**数学**:
```
π = [q(τ)]₁  其中 q(x) = (p(x) - y) / (x - z)
```

**大小**: 48 字节（G1 点）

**验证**: 使用配对检查
```
e([p(τ)]₁, [1]₂) ?= e([π]₁, [τ-z]₂) · e([y]₁, [1]₂)
```

**示例**:
```rust
let proof = compute_blob_kzg_proof_rust(&blob, &commitment_bytes, &settings)?;
```

**相关章节**: 第2章

---

## R

### Reed-Solomon 编码 - Intermediate
**英文**: Reed-Solomon Encoding
**中文**: Reed-Solomon 编码
**解释**: 一种纠错码，用于数据可用性采样。可以从部分数据恢复完整数据。

**性质**:
- k 个数据块 → n 个编码块
- 可从任意 k 个块恢复全部数据
- 擦除码（Erasure Code）

**在 EIP-7594 中**:
- 4096 个数据 → 8192 个编码值
- 可从 50% 的数据恢复

**相关章节**: 第7章

---

### Rollup - Basic
**英文**: Rollup
**中文**: Rollup（卷叠）
**解释**: Layer 2 扩展方案，将多个交易"卷起"成一个，在 Layer 1 发布简洁证明。

**类型**:
- **Optimistic Rollup**: 乐观假设交易有效，欺诈证明
- **ZK Rollup**: 零知识证明交易有效性

**与 KZG 的关系**: EIP-4844 提供低成本数据发布

**相关章节**: 第3章、第20章

---

## S

### Scalar (标量) - Intermediate
**英文**: Scalar
**中文**: 标量
**解释**: 域元素，用于标量乘法。在 BLS12-381 中，标量来自 Fr 域。

**示例**:
```rust
let scalar: FsFr = FsFr::from_u64(7);
let point: FsG1 = FsG1::generator();
let result = point.mul(&scalar);  // 7 · G₁
```

**相关术语**: Fr, Field Element
**相关章节**: 第1章

---

### SRS (结构化参考串) - Advanced
**英文**: Structured Reference String
**中文**: 结构化参考串
**别名**: CRS (Common Reference String), Trusted Setup
**解释**: KZG 方案需要的公共参数，包含秘密值 τ 的幂次。

**形式**:
```
SRS = ([1]₁, [τ]₁, [τ²]₁, ..., [τⁿ]₁, [1]₂, [τ]₂)
```

**生成**: 通过可信设置仪式（Trusted Setup Ceremony）

**安全假设**: 如果至少一个参与者诚实销毁秘密 τ，系统就安全

**文件**: `assets/trusted_setup.txt` (807 KB)

**相关章节**: 第2章、第14章

---

### SPPARK - Intermediate
**英文**: SPPARK
**中文**: -
**解释**: Supranational 开发的 GPU 加速库，用于椭圆曲线运算。

**功能**:
- GPU 上的 MSM
- CUDA 内核优化

**相关章节**: 第9章

---

## T

### Trait - Intermediate
**英文**: Trait
**中文**: Trait（特征）
**解释**: Rust 中定义共享行为的语言特性。rust-kzg 使用 trait 抽象不同后端。

**核心 Trait**:
- `Fr`: 标量域操作
- `G1`: G1 群操作
- `G2`: G2 群操作
- `Pairing`: 配对计算
- `KZGSettings`: KZG 参数

**示例**:
```rust
pub trait Fr: Clone + Copy + ... {
    fn zero() -> Self;
    fn one() -> Self;
    fn add(&self, other: &Self) -> Self;
    // ...
}
```

**相关章节**: 第5章

---

### Trusted Setup (可信设置) - Advanced
**英文**: Trusted Setup
**中文**: 可信设置
**解释**: 生成 SRS 的过程，需要秘密值 τ，必须在生成后销毁。

**过程**:
1. 选择随机秘密 τ
2. 计算 [τⁱ]₁ 和 [τʲ]₂
3. 销毁 τ（关键！）

**以太坊仪式**:
- 超过 140,000 人参与
- 2023 年完成
- 生成用于 EIP-4844 的 SRS

**相关术语**: SRS, τ (Tau)
**相关章节**: 第2章

---

## Z

### Zero-Knowledge (零知识) - Advanced
**英文**: Zero-Knowledge
**中文**: 零知识
**解释**: 一种证明性质，证明者可以向验证者证明某个陈述为真，而不泄露任何其他信息。

**KZG 与零知识**:
- KZG 本身不是零知识的
- 可以通过添加随机性使其零知识
- 常用于 zkSNARK 系统

**相关章节**: 第2章

---

### zkSNARK - Advanced
**英文**: Zero-Knowledge Succinct Non-Interactive Argument of Knowledge
**中文**: 零知识简洁非交互式知识论证
**解释**: 一种证明系统，允许证明者简洁地证明计算的正确性。

**特性**:
- **Zero-Knowledge**: 不泄露额外信息
- **Succinct**: 证明大小小（常数级）
- **Non-Interactive**: 单向通信
- **Argument**: 计算安全性

**KZG 的角色**: 作为 zkSNARK 的多项式承诺方案

**应用**: ZK Rollup, Zcash, Filecoin

**相关章节**: 第2章

---

## 数学符号 / Mathematical Notation

| 符号 | 含义 | 说明 |
|------|------|------|
| `∈` | 属于 | a ∈ G: a 属于群 G |
| `⊕` | 群加法 | 椭圆曲线点加法 |
| `⊗` | 标量乘法 | s ⊗ P: 标量 s 乘点 P |
| `≡` | 同余 | a ≡ b (mod n) |
| `[a]₁` | G1 中的元素 | [a]₁ = a · G₁ |
| `[b]₂` | G2 中的元素 | [b]₂ = b · G₂ |
| `e()` | 配对函数 | e: G1 × G2 → GT |
| `τ` | Tau (希腊字母) | 可信设置中的秘密值 |
| `p(x)` | 多项式 | 以 x 为变量的多项式 |
| `Fr` | 标量域 | 有限域 |
| `Fp` | 基域 | 素数域 |

---

## 缩写对照表 / Abbreviations

| 缩写 | 全称 | 中文 |
|------|------|------|
| API | Application Programming Interface | 应用程序接口 |
| BLS | Barreto-Lynn-Scott | - |
| CRS | Common Reference String | 公共参考串 |
| DAS | Data Availability Sampling | 数据可用性采样 |
| ECC | Elliptic Curve Cryptography | 椭圆曲线密码学 |
| EIP | Ethereum Improvement Proposal | 以太坊改进提案 |
| FFT | Fast Fourier Transform | 快速傅里叶变换 |
| GPU | Graphics Processing Unit | 图形处理器 |
| KZG | Kate-Zaverucha-Goldberg | - |
| MSM | Multi-Scalar Multiplication | 多标量乘法 |
| SRS | Structured Reference String | 结构化参考串 |

---

## 按章节索引 / Index by Chapter

### 第1章：密码学基础
- Elliptic Curve (椭圆曲线)
- BLS12-381
- Group (群)
- Field Element (域元素)
- Pairing (配对)
- G1, G2
- Fr

### 第2章：KZG 方案
- KZG
- Commitment (承诺)
- Proof (证明)
- Polynomial (多项式)
- Trusted Setup (可信设置)
- SRS
- Fiat-Shamir Transform

### 第3章：EIP-4844
- EIP-4844
- Blob
- Danksharding
- Rollup

### 第7章：EIP-7594
- EIP-7594
- PeerDAS
- Cell
- DAS
- Reed-Solomon 编码

### 第8章：BLST 后端
- BLST
- Backend

### 第9章：GPU 加速
- GPU Acceleration
- SPPARK
- MSM

---

## 学习建议 / Study Advice

**初学者路径**:
1. 先理解基础术语（Basic）
2. 逐步学习中级概念（Intermediate）
3. 最后掌握高级内容（Advanced）

**使用方法**:
- 遇到不熟悉的术语时，快速查阅本术语表
- 点击"相关章节"链接深入学习
- 查看"相关术语"了解概念之间的联系

**扩展资源**:
- [Moonmath Manual](https://github.com/LeastAuthority/moonmath-manual)
- [以太坊官方文档](https://ethereum.org/zh/developers/docs/)
- [BLS12-381 For The Rest Of Us](https://hackmd.io/@benjaminion/bls12-381)

---

**术语表版本**: 1.0
**最后更新**: 2025-11-18
**贡献**: 欢迎通过 GitHub Issues 提交术语补充或修正
