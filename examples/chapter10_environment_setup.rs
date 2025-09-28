//! 第10章：环境搭建与基础使用示例
//! 
//! 本示例演示了 Rust KZG 库的基础使用方法，包括：
//! - 环境配置和受信任设置加载
//! - 创建有效的 Blob 数据
//! - KZG 承诺-证明-验证的完整流程
//! - 性能统计和调试技巧
//! - 错误处理最佳实践

use std::time::Instant;
use std::path::Path;

/// 主函数：演示完整的 KZG 工作流程
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 第10章：环境搭建与基础使用示例");
    println!("{}", "=".repeat(50));
    println!("这是你的第一个 KZG 程序，让我们开始吧！\n");

    // 1. 加载受信任设置
    println!("📁 步骤 1: 加载受信任设置...");
    let kzg_settings = load_trusted_setup_from_file()?;
    println!("✅ 受信任设置加载成功!\n");

    // 2. 创建测试数据 (Blob)
    println!("🔢 步骤 2: 创建测试 Blob 数据...");
    let blob = create_test_blob()?;
    println!("✅ 测试 Blob 创建成功! (包含 {} 个域元素)\n", blob.len());

    // 3. 生成承诺
    println!("🔐 步骤 3: 生成 KZG 承诺...");
    let start = Instant::now();
    let commitment = blob_to_kzg_commitment_mock(&blob, &kzg_settings)?;
    let commitment_time = start.elapsed();
    println!("✅ KZG 承诺生成成功! 耗时: {:?}\n", commitment_time);

    // 4. 生成证明
    println!("📝 步骤 4: 生成 KZG 证明...");
    let start = Instant::now();
    let proof = compute_blob_kzg_proof_mock(&blob, &commitment, &kzg_settings)?;
    let proof_time = start.elapsed();
    println!("✅ KZG 证明生成成功! 耗时: {:?}\n", proof_time);

    // 5. 验证证明
    println!("🔍 步骤 5: 验证 KZG 证明...");
    let start = Instant::now();
    let is_valid = verify_blob_kzg_proof_mock(&blob, &commitment, &proof, &kzg_settings)?;
    let verify_time = start.elapsed();
    
    if is_valid {
        println!("🎉 证明验证成功! 耗时: {:?}", verify_time);
        println!("\n{}", "=".repeat(50));
        println!("🏆 恭喜! 你已经成功完成了第一个 KZG 操作!");
        println!("   - 你学会了如何加载受信任设置");
        println!("   - 你学会了如何创建有效的 Blob 数据");
        println!("   - 你学会了 KZG 承诺-证明-验证的完整流程");
        println!("{}", "=".repeat(50));
    } else {
        println!("❌ 证明验证失败 - 这不应该发生!");
        return Err("验证失败".into());
    }

    // 6. 额外演示：性能统计
    println!("\n📊 性能统计:");
    println!("   🔐 承诺生成: {:?}", commitment_time);
    println!("   📝 证明生成: {:?}", proof_time);
    println!("   🔍 证明验证: {:?}", verify_time);
    println!("   ⏱️ 总耗时: {:?}", commitment_time + proof_time + verify_time);

    // 7. 演示数据大小
    println!("\n📏 数据大小统计:");
    println!("   📊 Blob 数据: {} 个域元素 (≈ 128KB)", blob.len());
    println!("   🔐 承诺大小: 48 字节 (G1 群元素)");
    println!("   📝 证明大小: 48 字节 (G1 群元素)");
    println!("   💾 压缩比: {:.2}%", (96.0 / (blob.len() * 32) as f64) * 100.0);

    // 8. 演示调试功能
    demo_debugging_features(&kzg_settings, &blob)?;

    // 9. 演示错误处理
    demo_error_handling(&kzg_settings)?;

    // 10. 演示性能测试
    demo_performance_testing(&kzg_settings)?;

    println!("\n🎯 第10章演示完成！");
    println!("   下一章将学习高级 API 使用技巧");

    Ok(())
}

// ============================================================================
// 模拟的 KZG 类型定义（与实际库接口保持一致）
// ============================================================================

/// 模拟的有限域元素
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Fr([u8; 32]);

impl Fr {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }
    
    pub fn one() -> Self {
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        Self(bytes)
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err("Invalid byte length".to_string());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }
    
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
    
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
    
    pub fn random() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&hash.to_le_bytes());
        Self(bytes)
    }
}

/// 模拟的 G1 群元素
#[derive(Debug, Clone, PartialEq)]
pub struct G1([u8; 48]);

impl G1 {
    pub fn zero() -> Self {
        Self([0u8; 48])
    }
    
    pub fn generator() -> Self {
        let mut bytes = [0u8; 48];
        bytes[47] = 1;
        Self(bytes)
    }
    
    pub fn random() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut bytes = [0u8; 48];
        bytes[..8].copy_from_slice(&hash.to_le_bytes());
        Self(bytes)
    }
    
    pub fn equals(&self, other: &G1) -> bool {
        self.0 == other.0
    }
}

/// 模拟的 KZG 设置
#[derive(Debug)]
pub struct KzgSettings {
    pub g1_count: usize,
    pub g2_count: usize,
}

impl KzgSettings {
    pub fn new(g1_count: usize, g2_count: usize) -> Self {
        Self { g1_count, g2_count }
    }
    
    pub fn g1_count(&self) -> usize {
        self.g1_count
    }
    
    pub fn g2_count(&self) -> usize {
        self.g2_count
    }
}

/// EIP-4844 标准常量
pub const FIELD_ELEMENTS_PER_BLOB: usize = 4096;

// ============================================================================
// 模拟的 KZG 操作函数
// ============================================================================

/// 模拟的承诺生成函数
fn blob_to_kzg_commitment_mock(blob: &[Fr], _settings: &KzgSettings) -> Result<G1, String> {
    if blob.is_empty() {
        return Err("Empty blob".to_string());
    }
    
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(format!("Invalid blob size: {}, expected: {}", blob.len(), FIELD_ELEMENTS_PER_BLOB));
    }
    
    // 模拟计算时间
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // 返回一个基于 blob 内容的"承诺"
    let mut commitment_bytes = [0u8; 48];
    for (i, element) in blob.iter().take(6).enumerate() {
        let element_bytes = element.to_bytes();
        commitment_bytes[i * 8..(i + 1) * 8].copy_from_slice(&element_bytes[24..32]);
    }
    
    Ok(G1(commitment_bytes))
}

/// 模拟的证明生成函数
fn compute_blob_kzg_proof_mock(blob: &[Fr], commitment: &G1, _settings: &KzgSettings) -> Result<G1, String> {
    if blob.is_empty() {
        return Err("Empty blob".to_string());
    }
    
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(format!("Invalid blob size: {}, expected: {}", blob.len(), FIELD_ELEMENTS_PER_BLOB));
    }
    
    // 模拟计算时间
    std::thread::sleep(std::time::Duration::from_millis(80));
    
    // 返回一个基于 blob 和承诺的"证明"
    let mut proof_bytes = [0u8; 48];
    let commitment_bytes = &commitment.0;
    
    for i in 0..6 {
        proof_bytes[i * 8] = commitment_bytes[i * 8] ^ (i as u8);
        proof_bytes[i * 8 + 1] = blob[i * 100].to_bytes()[31];
    }
    
    Ok(G1(proof_bytes))
}

/// 模拟的验证函数
fn verify_blob_kzg_proof_mock(blob: &[Fr], commitment: &G1, proof: &G1, _settings: &KzgSettings) -> Result<bool, String> {
    if blob.is_empty() {
        return Err("Empty blob".to_string());
    }
    
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(format!("Invalid blob size: {}, expected: {}", blob.len(), FIELD_ELEMENTS_PER_BLOB));
    }
    
    // 模拟验证时间
    std::thread::sleep(std::time::Duration::from_millis(5));
    
    // 模拟验证逻辑：检查证明是否与承诺和blob一致
    let expected_commitment = blob_to_kzg_commitment_mock(blob, _settings)?;
    let expected_proof = compute_blob_kzg_proof_mock(blob, commitment, _settings)?;
    
    Ok(commitment.equals(&expected_commitment) && proof.equals(&expected_proof))
}

// ============================================================================
// 核心功能函数
// ============================================================================

/// 智能加载受信任设置文件
/// 会尝试多个可能的路径，自动找到文件位置
fn load_trusted_setup_from_file() -> Result<KzgSettings, Box<dyn std::error::Error>> {
    let possible_paths = [
        "./assets/trusted_setup.txt",
        "../assets/trusted_setup.txt", 
        "../../assets/trusted_setup.txt",
        "./trusted_setup.txt",
        "./src/trusted_setup.txt",
        "../src/trusted_setup.txt",
    ];

    println!("🔍 搜索受信任设置文件...");
    for path in &possible_paths {
        if Path::new(path).exists() {
            println!("   ✅ 找到文件: {}", path);
            return load_trusted_setup_file(path);
        } else {
            println!("   ❌ 未找到: {}", path);
        }
    }

    // 如果没有找到文件，创建一个模拟的设置
    println!("   ⚠️  未找到受信任设置文件，使用模拟设置");
    println!("   💡 在生产环境中，请确保下载真实的受信任设置文件");
    
    Ok(KzgSettings::new(4096, 65))
}

/// 加载具体的受信任设置文件
fn load_trusted_setup_file(path: &str) -> Result<KzgSettings, Box<dyn std::error::Error>> {
    use std::fs;
    
    println!("   📖 读取文件: {}", path);
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.len() < 2 {
        return Err("受信任设置文件格式错误".into());
    }
    
    // 解析文件头部的 G1 和 G2 点数量
    let g1_count = lines[0].parse::<usize>()
        .map_err(|_| "无法解析 G1 点数量")?;
    let g2_count = lines[1].parse::<usize>()
        .map_err(|_| "无法解析 G2 点数量")?;
    
    println!("   📊 G1 点数量: {}", g1_count);
    println!("   📊 G2 点数量: {}", g2_count);
    
    Ok(KzgSettings::new(g1_count, g2_count))
}

/// 创建有效的测试 Blob 数据
/// Blob 必须包含 4096 个有效的域元素
fn create_test_blob() -> Result<Vec<Fr>, String> {
    let mut blob = Vec::with_capacity(FIELD_ELEMENTS_PER_BLOB);

    println!("   🔢 生成 {} 个域元素...", FIELD_ELEMENTS_PER_BLOB);
    
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        // 创建有效的域元素
        // 使用递增的小值，确保都在域内
        let mut bytes = [0u8; 32];
        
        // 创建一个有趣的模式，而不是单调递增
        let value = match i {
            0..=255 => i as u8,
            256..=511 => (i - 256) as u8,
            512..=767 => ((i - 512) * 2) as u8,
            768..=1023 => ((i - 768) / 2) as u8,
            _ => (i % 256) as u8,
        };
        
        bytes[31] = value;
        
        let element = Fr::from_bytes(&bytes)
            .map_err(|e| format!("❌ 创建第 {} 个域元素失败: {}", i, e))?;
        blob.push(element);
        
        // 每完成 1000 个元素就报告进度
        if (i + 1) % 1000 == 0 {
            println!("     进度: {}/{}", i + 1, FIELD_ELEMENTS_PER_BLOB);
        }
    }

    println!("   ✅ 所有域元素创建完成!");
    Ok(blob)
}

// ============================================================================
// 演示功能
// ============================================================================

/// 演示调试功能
fn demo_debugging_features(settings: &KzgSettings, blob: &[Fr]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 调试功能演示");
    println!("----------------------------------------");
    
    // 1. 设置信息调试
    println!("📊 受信任设置信息:");
    println!("   G1 点数量: {}", settings.g1_count());
    println!("   G2 点数量: {}", settings.g2_count());
    println!("   内存占用估算: {} MB", (settings.g1_count() * 48 + settings.g2_count() * 96) / 1024 / 1024);
    
    // 2. Blob 数据分析
    println!("\n📊 Blob 数据分析:");
    println!("   总元素数: {}", blob.len());
    let zero_count = blob.iter().filter(|&x| x.is_zero()).count();
    println!("   零元素数: {} ({:.2}%)", zero_count, (zero_count as f64 / blob.len() as f64) * 100.0);
    
    // 显示前几个和后几个元素
    println!("   前5个元素:");
    for (i, element) in blob.iter().take(5).enumerate() {
        println!("     [{}]: {:02x}...{:02x}", i, element.0[0], element.0[31]);
    }
    
    println!("   后5个元素:");
    for (i, element) in blob.iter().rev().take(5).enumerate() {
        let idx = blob.len() - 1 - i;
        println!("     [{}]: {:02x}...{:02x}", idx, element.0[0], element.0[31]);
    }
    
    // 3. 内存使用统计
    println!("\n💾 内存使用统计:");
    let blob_memory = blob.len() * 32;
    println!("   Blob 内存: {} KB", blob_memory / 1024);
    println!("   设置内存: {} KB", (settings.g1_count() * 48 + settings.g2_count() * 96) / 1024);
    
    Ok(())
}

/// 演示错误处理
fn demo_error_handling(settings: &KzgSettings) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚨 错误处理演示");
    println!("----------------------------------------");
    
    // 1. 空 blob 错误
    println!("🧪 测试空 blob 处理:");
    let empty_blob: Vec<Fr> = vec![];
    match blob_to_kzg_commitment_mock(&empty_blob, settings) {
        Ok(_) => println!("   ❌ 预期失败但成功了"),
        Err(e) => println!("   ✅ 正确处理空 blob: {}", e),
    }
    
    // 2. 错误大小的 blob
    println!("\n🧪 测试错误大小 blob 处理:");
    let wrong_size_blob: Vec<Fr> = vec![Fr::zero(); 100]; // 应该是 4096
    match blob_to_kzg_commitment_mock(&wrong_size_blob, settings) {
        Ok(_) => println!("   ❌ 预期失败但成功了"),
        Err(e) => println!("   ✅ 正确处理错误大小: {}", e),
    }
    
    // 3. 无效的域元素处理
    println!("\n🧪 测试无效域元素处理:");
    let invalid_bytes = [255u8; 32]; // 可能超出域大小
    match Fr::from_bytes(&invalid_bytes) {
        Ok(fr) => println!("   ✅ 域元素创建成功: {:?}", fr.is_zero()),
        Err(e) => println!("   ✅ 正确处理无效字节: {}", e),
    }
    
    // 4. 演示恢复策略
    println!("\n🔄 错误恢复策略演示:");
    let mut retry_count = 0;
    let max_retries = 3;
    
    loop {
        retry_count += 1;
        println!("   尝试 {} / {}...", retry_count, max_retries);
        
        // 模拟间歇性错误
        if retry_count < 3 {
            println!("   ❌ 模拟错误发生");
            if retry_count >= max_retries {
                println!("   🚨 达到最大重试次数，操作失败");
                break;
            }
            println!("   🔄 1秒后重试...");
            std::thread::sleep(std::time::Duration::from_millis(100)); // 模拟等待
        } else {
            println!("   ✅ 操作成功!");
            break;
        }
    }
    
    Ok(())
}

/// 演示性能测试
fn demo_performance_testing(settings: &KzgSettings) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ 性能测试演示");
    println!("----------------------------------------");
    
    // 创建不同大小的测试数据
    let test_sizes = vec![100, 500, 1000, 2000, 4096];
    let mut results = Vec::new();
    
    for &size in &test_sizes {
        println!("\n🧪 测试 {} 个元素的性能:", size);
        
        // 创建指定大小的 blob
        let mut test_blob = vec![Fr::zero(); size];
        for (i, element) in test_blob.iter_mut().enumerate() {
            let mut bytes = [0u8; 32];
            bytes[31] = (i % 256) as u8;
            *element = Fr::from_bytes(&bytes)?;
        }
        
        if size == 4096 {
            // 只对标准大小进行完整测试
            let start = Instant::now();
            let commitment = blob_to_kzg_commitment_mock(&test_blob, settings)?;
            let commitment_time = start.elapsed();
            
            let start = Instant::now();
            let proof = compute_blob_kzg_proof_mock(&test_blob, &commitment, settings)?;
            let proof_time = start.elapsed();
            
            let start = Instant::now();
            let _is_valid = verify_blob_kzg_proof_mock(&test_blob, &commitment, &proof, settings)?;
            let verify_time = start.elapsed();
            
            println!("   🔐 承诺生成: {:?}", commitment_time);
            println!("   📝 证明生成: {:?}", proof_time);
            println!("   🔍 证明验证: {:?}", verify_time);
            
            let total_time = commitment_time + proof_time + verify_time;
            results.push((size, total_time));
            
            // 计算吞吐量
            let throughput = size as f64 / total_time.as_secs_f64();
            println!("   📊 吞吐量: {:.2} 元素/秒", throughput);
        } else {
            // 对非标准大小只进行时间测量（会失败，但可以测量错误处理时间）
            let start = Instant::now();
            let _ = blob_to_kzg_commitment_mock(&test_blob, settings);
            let time = start.elapsed();
            println!("   ⚠️  非标准大小，错误处理时间: {:?}", time);
            results.push((size, time));
        }
    }
    
    // 性能统计总结
    println!("\n📊 性能统计总结:");
    println!("   {:>8} | {:>12} | {:>20}", "尺寸", "总时间", "备注");
    println!("   {}", "-".repeat(45));
    
    for (size, time) in results {
        let note = if size == 4096 { "完整测试" } else { "错误处理" };
        println!("   {:>8} | {:>12?} | {:>20}", size, time, note);
    }
    
    Ok(())
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fr_creation() {
        println!("🧪 测试 Fr 元素创建...");
        
        let zero = Fr::zero();
        assert!(zero.is_zero());
        
        let one = Fr::one();
        assert!(!one.is_zero());
        assert_eq!(one.0[31], 1);
        
        println!("✅ Fr 元素创建测试通过!");
    }

    #[test]
    fn test_fr_from_bytes() {
        println!("🧪 测试 Fr::from_bytes...");
        
        let bytes = [0u8; 32];
        let fr = Fr::from_bytes(&bytes).unwrap();
        assert!(fr.is_zero());
        
        let mut bytes = [0u8; 32];
        bytes[31] = 42;
        let fr = Fr::from_bytes(&bytes).unwrap();
        assert!(!fr.is_zero());
        assert_eq!(fr.0[31], 42);
        
        // 测试错误大小
        let wrong_bytes = [0u8; 16];
        assert!(Fr::from_bytes(&wrong_bytes).is_err());
        
        println!("✅ Fr::from_bytes 测试通过!");
    }

    #[test]
    fn test_g1_creation() {
        println!("🧪 测试 G1 元素创建...");
        
        let zero = G1::zero();
        let gen = G1::generator();
        
        assert_ne!(zero.0, gen.0);
        assert!(zero.equals(&G1::zero()));
        assert!(gen.equals(&G1::generator()));
        
        println!("✅ G1 元素创建测试通过!");
    }

    #[test]
    fn test_blob_creation() {
        println!("🧪 测试 Blob 创建...");
        
        let blob = create_test_blob().unwrap();
        
        assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);
        
        // 验证前几个元素
        for (i, element) in blob.iter().take(10).enumerate() {
            assert_eq!(element.0[31], i as u8);
        }
        
        println!("✅ Blob 创建测试通过!");
    }

    #[test]
    fn test_kzg_settings() {
        println!("🧪 测试 KZG 设置...");
        
        let settings = KzgSettings::new(4096, 65);
        
        assert_eq!(settings.g1_count(), 4096);
        assert_eq!(settings.g2_count(), 65);
        
        println!("✅ KZG 设置测试通过!");
    }

    #[test]
    fn test_kzg_commitment() {
        println!("🧪 测试 KZG 承诺生成...");
        
        let settings = KzgSettings::new(4096, 65);
        let blob = create_test_blob().unwrap();
        
        let commitment = blob_to_kzg_commitment_mock(&blob, &settings).unwrap();
        
        // 相同输入应产生相同输出
        let commitment2 = blob_to_kzg_commitment_mock(&blob, &settings).unwrap();
        assert!(commitment.equals(&commitment2));
        
        println!("✅ KZG 承诺生成测试通过!");
    }

    #[test]
    fn test_full_kzg_workflow() {
        println!("🧪 测试完整 KZG 工作流程...");
        
        let settings = KzgSettings::new(4096, 65);
        let blob = create_test_blob().unwrap();
        
        // 完整的承诺-证明-验证流程
        let commitment = blob_to_kzg_commitment_mock(&blob, &settings).unwrap();
        let proof = compute_blob_kzg_proof_mock(&blob, &commitment, &settings).unwrap();
        let is_valid = verify_blob_kzg_proof_mock(&blob, &commitment, &proof, &settings).unwrap();
        
        assert!(is_valid, "完整的 KZG 工作流程应该验证成功");
        
        println!("✅ 完整 KZG 工作流程测试通过!");
    }

    #[test]
    fn test_error_handling() {
        println!("🧪 测试错误处理...");
        
        let settings = KzgSettings::new(4096, 65);
        
        // 测试空 blob
        let empty_blob: Vec<Fr> = vec![];
        assert!(blob_to_kzg_commitment_mock(&empty_blob, &settings).is_err());
        
        // 测试错误大小的 blob
        let wrong_size_blob: Vec<Fr> = vec![Fr::zero(); 100];
        assert!(blob_to_kzg_commitment_mock(&wrong_size_blob, &settings).is_err());
        
        // 测试无效字节
        let wrong_bytes = [0u8; 16];
        assert!(Fr::from_bytes(&wrong_bytes).is_err());
        
        println!("✅ 错误处理测试通过!");
    }
}