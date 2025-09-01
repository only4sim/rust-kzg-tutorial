use std::time::{Duration, Instant};
use std::collections::HashMap;

use rust_kzg_blst::{
    types::{
        fr::FsFr,
    },
    eip_4844::load_trusted_setup_filename_rust,
};

use kzg::{
    G1,
    eip_4844::{
        blob_to_kzg_commitment_rust, 
        compute_blob_kzg_proof_rust, 
        verify_blob_kzg_proof_rust,
        verify_blob_kzg_proof_batch_rust,
        FIELD_ELEMENTS_PER_BLOB,
        BYTES_PER_BLOB,
        BYTES_PER_FIELD_ELEMENT,
        BYTES_PER_COMMITMENT,
        BYTES_PER_PROOF,
    },
    eth::{
        FIELD_ELEMENTS_PER_EXT_BLOB,
        FIELD_ELEMENTS_PER_CELL,
        CELLS_PER_EXT_BLOB,
    },
    Fr,
};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// EIP-4844 核心常量演示
const MAX_BLOBS_PER_BLOCK: usize = 6;
const TARGET_SLOT_TIME: Duration = Duration::from_secs(12);

/// 性能分析器，用于收集和分析各种操作的性能数据
pub struct PerformanceProfiler {
    metrics: HashMap<String, Vec<Duration>>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }
    
    pub fn record_metric(&mut self, operation: &str, duration: Duration) {
        self.metrics.entry(operation.to_string()).or_insert_with(Vec::new).push(duration);
    }
    
    pub fn print_performance_summary(&self) {
        println!("\n📊 性能分析报告");
        println!("{}", "=".repeat(50));
        
        for (operation, times) in &self.metrics {
            if times.is_empty() {
                continue;
            }
            
            let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
            let min_time = times.iter().min().unwrap();
            let max_time = times.iter().max().unwrap();
            
            println!("🔹 {:<25}: 平均 {:8.2}ms, 范围 [{:6.2}ms - {:6.2}ms]", 
                    operation, 
                    avg_time.as_secs_f64() * 1000.0,
                    min_time.as_secs_f64() * 1000.0,
                    max_time.as_secs_f64() * 1000.0);
        }
    }
}

/// 生成测试用的随机 blob 数据
fn generate_random_blob() -> Result<Vec<FsFr>, String> {
    let mut blob = Vec::with_capacity(FIELD_ELEMENTS_PER_BLOB);
    
    // 使用直接创建域元素的方法，只使用非常小的值
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        // 创建有效的域元素 - 使用很小的值，只在最后一个字节设置
        let mut bytes = [0u8; 32];
        
        // 使用非常简单的模式，确保值很小
        let value = (i % 127) as u8;  // 只使用0-126的值
        bytes[31] = value;  // 在最后一个字节（最高有效位）设置值
        
        let element = FsFr::from_bytes(&bytes)
            .map_err(|e| format!("创建第 {} 个域元素失败: {}", i, e))?;
        blob.push(element);
    }
    
    Ok(blob)
}

/// 创建标准测试 blob
fn create_test_blob() -> Result<Vec<FsFr>, String> {
    generate_random_blob()
}

/// 演示 EIP-4844 基本概念和常量
fn demonstrate_eip4844_basics() {
    println!("🌐 第3章：以太坊数据分片 (EIP-4844) 应用场景");
    println!("{}", "=".repeat(60));
    
    println!("\n📦 3.1 EIP-4844 核心参数");
    println!("{}", "-".repeat(40));
    
    println!("   🔹 每个 Blob 的域元素数量: {}", FIELD_ELEMENTS_PER_BLOB);
    println!("   🔹 每个域元素字节数: {}", BYTES_PER_FIELD_ELEMENT);
    println!("   🔹 每个 Blob 总字节数: {} KB", BYTES_PER_BLOB / 1024);
    println!("   🔹 KZG 承诺大小: {} 字节", BYTES_PER_COMMITMENT);
    println!("   🔹 KZG 证明大小: {} 字节", BYTES_PER_PROOF);
    println!("   🔹 每区块最大 Blob 数: {}", MAX_BLOBS_PER_BLOCK);
    println!("   🔹 目标区块时间: {:?}", TARGET_SLOT_TIME);
    
    println!("\n📊 数据可用性采样 (DAS) 参数:");
    println!("   🔹 扩展 Blob 域元素数: {}", FIELD_ELEMENTS_PER_EXT_BLOB);
    println!("   🔹 每个采样单元大小: {}", FIELD_ELEMENTS_PER_CELL);
    println!("   🔹 总采样单元数: {}", CELLS_PER_EXT_BLOB);
    
    // 计算存储效率
    let commitment_overhead = BYTES_PER_COMMITMENT;
    let proof_overhead = BYTES_PER_PROOF;
    let total_overhead = commitment_overhead + proof_overhead;
    let efficiency = (BYTES_PER_BLOB as f64) / (BYTES_PER_BLOB + total_overhead) as f64 * 100.0;
    
    println!("\n💰 存储效率分析:");
    println!("   🔹 数据负载: {} KB", BYTES_PER_BLOB / 1024);
    println!("   🔹 加密开销: {} 字节 (承诺 + 证明)", total_overhead);
    println!("   🔹 存储效率: {:.2}%", efficiency);
}

/// 演示 Blob 到 KZG 承诺的转换过程
fn demonstrate_blob_to_commitment() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔗 3.2 Blob 到 KZG 承诺转换");
    println!("{}", "-".repeat(40));
    
    // 加载受信任设置
    let trusted_setup_path = find_trusted_setup_file()?;
    let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path)?;
    
    // 创建测试 blob
    let blob = create_test_blob()?;
    println!("   ✅ 创建了包含 {} 个域元素的测试 blob", blob.len());
    
    // 计算 KZG 承诺
    let start = Instant::now();
    let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
    let commit_time = start.elapsed();
    
    println!("   🔹 KZG 承诺计算耗时: {:.2}ms", commit_time.as_secs_f64() * 1000.0);
    println!("   🔹 承诺十六进制表示: {:?}", hex::encode(commitment.to_bytes()));
    
    // 验证承诺的确定性
    let commitment2 = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
    println!("   🔹 承诺计算的确定性: {}", 
        if commitment.to_bytes() == commitment2.to_bytes() { "✅ 通过" } else { "❌ 失败" });
    
    Ok(())
}

/// 演示 KZG 证明生成和验证
fn demonstrate_proof_generation_verification() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 3.3 KZG 证明生成与验证");
    println!("{}", "-".repeat(40));
    
    let trusted_setup_path = find_trusted_setup_file()?;
    let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path)?;
    
    let blob = create_test_blob()?;
    let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
    
    // 生成 KZG 证明
    let start = Instant::now();
    let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings)?;
    let proof_time = start.elapsed();
    
    println!("   🔹 KZG 证明生成耗时: {:.2}ms", proof_time.as_secs_f64() * 1000.0);
    
    // 验证 KZG 证明
    let start = Instant::now();
    let is_valid = verify_blob_kzg_proof_rust(&blob, &commitment, &proof, &kzg_settings)?;
    let verify_time = start.elapsed();
    
    println!("   🔹 KZG 证明验证耗时: {:.2}ms", verify_time.as_secs_f64() * 1000.0);
    println!("   🔹 证明验证结果: {}", if is_valid { "✅ 有效" } else { "❌ 无效" });
    
    // 测试无效证明检测
    let mut invalid_blob = blob.clone();
    invalid_blob[0] = FsFr::from_u64(12345); // 修改第一个元素
    
    let invalid_result = verify_blob_kzg_proof_rust(&invalid_blob, &commitment, &proof, &kzg_settings)?;
    println!("   🔹 无效数据检测: {}", if !invalid_result { "✅ 正确检测到无效" } else { "❌ 未能检测无效" });
    
    Ok(())
}

/// 演示批量验证的性能优势
fn demonstrate_batch_verification() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 3.4 批量验证性能优势");
    println!("{}", "-".repeat(40));
    
    let trusted_setup_path = find_trusted_setup_file()?;
    let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path)?;
    
    // 准备多个 blob 进行批量测试
    let blob_count = MAX_BLOBS_PER_BLOCK;
    let mut blobs = Vec::new();
    let mut commitments = Vec::new();
    let mut proofs = Vec::new();
    
    println!("   📦 准备 {} 个 blob 进行测试...", blob_count);
    
    for i in 0..blob_count {
        let blob = generate_random_blob()?;
        let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
        let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings)?;
        
        blobs.push(blob);
        commitments.push(commitment);
        proofs.push(proof);
        
        print!("   🔄 进度: {}/{}\r", i + 1, blob_count);
    }
    println!();
    
    // 单独验证 vs 批量验证性能对比
    
    // 1. 单独验证
    let start = Instant::now();
    for i in 0..blob_count {
        let _ = verify_blob_kzg_proof_rust(&blobs[i], &commitments[i], &proofs[i], &kzg_settings)?;
    }
    let individual_time = start.elapsed();
    
    // 2. 批量验证
    let start = Instant::now();
    let batch_result = verify_blob_kzg_proof_batch_rust(&blobs, &commitments, &proofs, &kzg_settings)?;
    let batch_time = start.elapsed();
    
    println!("   📊 性能对比结果:");
    println!("      🔹 单独验证总耗时: {:.2}ms", individual_time.as_secs_f64() * 1000.0);
    println!("      🔹 批量验证总耗时: {:.2}ms", batch_time.as_secs_f64() * 1000.0);
    println!("      🔹 性能提升: {:.1}x", individual_time.as_secs_f64() / batch_time.as_secs_f64());
    println!("      🔹 批量验证结果: {}", if batch_result { "✅ 全部有效" } else { "❌ 存在无效" });
    
    // 检查是否满足区块时间要求
    let meets_requirement = batch_time < TARGET_SLOT_TIME;
    println!("   ⏱️  满足 12s 区块时间要求: {}", 
        if meets_requirement { "✅ 是" } else { "❌ 否" });
    
    Ok(())
}

/// 演示并行计算的性能优势
#[cfg(feature = "parallel")]
fn demonstrate_parallel_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚖️ 3.5 并行计算性能优势");
    println!("{}", "-".repeat(40));
    
    let trusted_setup_path = find_trusted_setup_file()?;
    let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path)?;
    
    let mut rng = rand::thread_rng();
    let blob_count = 20; // 更多 blob 以显示并行优势
    
    let mut blobs = Vec::new();
    for _ in 0..blob_count {
        blobs.push(generate_random_blob()?);
    }
    
    // 串行计算承诺
    let start = Instant::now();
    let serial_commitments: Result<Vec<_>, _> = blobs
        .iter()
        .map(|blob| blob_to_kzg_commitment_rust(blob, &kzg_settings))
        .collect();
    let serial_time = start.elapsed();
    let serial_commitments = serial_commitments?;
    
    // 并行计算承诺
    let start = Instant::now();
    let parallel_commitments: Result<Vec<_>, _> = blobs
        .par_iter()
        .map(|blob| blob_to_kzg_commitment_rust(blob, &kzg_settings))
        .collect();
    let parallel_time = start.elapsed();
    let parallel_commitments = parallel_commitments?;
    
    // 验证结果一致性
    let results_match = serial_commitments.iter().zip(parallel_commitments.iter())
        .all(|(s, p)| s.to_bytes() == p.to_bytes());
    
    println!("   📊 并行计算性能对比 ({} 个 blob):", blob_count);
    println!("      🔹 串行计算耗时: {:.2}ms", serial_time.as_secs_f64() * 1000.0);
    println!("      🔹 并行计算耗时: {:.2}ms", parallel_time.as_secs_f64() * 1000.0);
    println!("      🔹 并行加速比: {:.1}x", serial_time.as_secs_f64() / parallel_time.as_secs_f64());
    println!("      🔹 结果一致性: {}", if results_match { "✅ 一致" } else { "❌ 不一致" });
    
    let cpu_count = num_cpus::get();
    println!("      🔹 系统CPU核心数: {}", cpu_count);
    
    Ok(())
}

/// 演示关键路径性能分析
fn demonstrate_critical_path_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 3.6 关键路径性能分析");
    println!("{}", "-".repeat(40));
    
    let trusted_setup_path = find_trusted_setup_file()?;
    let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path)?;
    
    let mut profiler = PerformanceProfiler::new();
    let iterations = 10; // 多次测试取平均值
    
    println!("   🔄 执行 {} 次测试以获得准确数据...", iterations);
    
    for i in 0..iterations {
        let blob = create_test_blob()?;
        
        // 1. Blob 到承诺转换
        let start = Instant::now();
        let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
        profiler.record_metric("blob_to_commitment", start.elapsed());
        
        // 2. 证明生成
        let start = Instant::now();
        let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings)?;
        profiler.record_metric("proof_generation", start.elapsed());
        
        // 3. 证明验证
        let start = Instant::now();
        let _ = verify_blob_kzg_proof_rust(&blob, &commitment, &proof, &kzg_settings)?;
        profiler.record_metric("proof_verification", start.elapsed());
        
        print!("   📈 完成: {}/{}\r", i + 1, iterations);
    }
    println!();
    
    profiler.print_performance_summary();
    
    // 分析瓶颈
    println!("\n🎯 性能瓶颈分析:");
    if let (Some(commit_times), Some(proof_times), Some(verify_times)) = (
        profiler.metrics.get("blob_to_commitment"),
        profiler.metrics.get("proof_generation"),
        profiler.metrics.get("proof_verification")
    ) {
        let commit_avg = commit_times.iter().sum::<Duration>() / commit_times.len() as u32;
        let proof_avg = proof_times.iter().sum::<Duration>() / proof_times.len() as u32;
        let verify_avg = verify_times.iter().sum::<Duration>() / verify_times.len() as u32;
        
        let total = commit_avg + proof_avg + verify_avg;
        
        println!("   🔹 承诺计算占比: {:.1}%", commit_avg.as_secs_f64() / total.as_secs_f64() * 100.0);
        println!("   🔹 证明生成占比: {:.1}%", proof_avg.as_secs_f64() / total.as_secs_f64() * 100.0);
        println!("   🔹 证明验证占比: {:.1}%", verify_avg.as_secs_f64() / total.as_secs_f64() * 100.0);
        
        // 检查是否为计算密集型
        if proof_avg > commit_avg && proof_avg > verify_avg {
            println!("   💡 证明生成是性能瓶颈，建议优化：");
            println!("      - 使用更快的椭圆曲线库");
            println!("      - 启用并行计算");
            println!("      - 考虑硬件加速");
        }
    }
    
    Ok(())
}

/// 演示网络级性能要求验证
fn demonstrate_network_performance_requirements() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌐 3.7 网络级性能要求验证");
    println!("{}", "-".repeat(40));
    
    let trusted_setup_path = find_trusted_setup_file()?;
    let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path)?;
    
    // 模拟最坏情况：满负载区块
    let blobs: Result<Vec<_>, _> = (0..MAX_BLOBS_PER_BLOCK)
        .map(|_| generate_random_blob())
        .collect();
    let blobs = blobs?;
    
    println!("   📦 模拟满负载区块验证 ({} 个 blob)", MAX_BLOBS_PER_BLOCK);
    
    // 计算承诺
    let start = Instant::now();
    let commitments: Result<Vec<_>, _> = blobs
        .iter()
        .map(|blob| blob_to_kzg_commitment_rust(blob, &kzg_settings))
        .collect();
    let commitments = commitments?;
    let commit_total_time = start.elapsed();
    
    // 生成证明
    let start = Instant::now();
    let proofs: Result<Vec<_>, _> = blobs
        .iter()
        .zip(&commitments)
        .map(|(blob, commitment)| compute_blob_kzg_proof_rust(blob, commitment, &kzg_settings))
        .collect();
    let proofs = proofs?;
    let proof_total_time = start.elapsed();
    
    // 批量验证
    let start = Instant::now();
    let batch_valid = verify_blob_kzg_proof_batch_rust(&blobs, &commitments, &proofs, &kzg_settings)?;
    println!("批量验证结果: {}", batch_valid);
    let verify_total_time = start.elapsed();
    
    let total_processing_time = commit_total_time + proof_total_time + verify_total_time;
    
    println!("\n   📊 网络性能分析:");
    println!("      🔹 总数据量: {:.1} KB", (blobs.len() * BYTES_PER_BLOB) as f64 / 1024.0);
    println!("      🔹 承诺计算总时间: {:.2}ms", commit_total_time.as_secs_f64() * 1000.0);
    println!("      🔹 证明生成总时间: {:.2}ms", proof_total_time.as_secs_f64() * 1000.0);
    println!("      🔹 批量验证总时间: {:.2}ms", verify_total_time.as_secs_f64() * 1000.0);
    println!("      🔹 总处理时间: {:.2}ms", total_processing_time.as_secs_f64() * 1000.0);
    
    // 检查性能要求
    let meets_target = verify_total_time < TARGET_SLOT_TIME;
    let performance_margin = TARGET_SLOT_TIME.as_secs_f64() / verify_total_time.as_secs_f64();
    
    println!("\n   ⏱️  性能要求评估:");
    println!("      🔹 目标时间限制: {:.0}s", TARGET_SLOT_TIME.as_secs_f64());
    println!("      🔹 实际验证时间: {:.3}s", verify_total_time.as_secs_f64());
    println!("      🔹 性能裕度: {:.1}x", performance_margin);
    println!("      🔹 满足要求: {}", if meets_target { "✅ 是" } else { "❌ 否" });
    
    if !meets_target {
        println!("\n   ⚠️  性能优化建议:");
        println!("      - 启用并行处理 (--features parallel)");
        println!("      - 使用更快的椭圆曲线后端");
        println!("      - 考虑硬件加速 (GPU)");
        println!("      - 优化受信任设置加载");
    }
    
    // 计算数据吞吐量
    let data_throughput = (blobs.len() * BYTES_PER_BLOB) as f64 / verify_total_time.as_secs_f64();
    println!("      🔹 数据处理吞吐量: {:.1} KB/s", data_throughput / 1024.0);
    
    Ok(())
}

/// 寻找受信任设置文件
fn find_trusted_setup_file() -> Result<String, Box<dyn std::error::Error>> {
    let possible_paths = [
        "assets/trusted_setup.txt",
        "../assets/trusted_setup.txt",
        "../../rust-kzg/src/trusted_setup.txt",
        "../rust-kzg/src/trusted_setup.txt",
        "trusted_setup.txt",
        "src/trusted_setup.txt",
    ];
    
    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    Err("无法找到 trusted_setup.txt 文件".into())
}

/// 主函数：运行所有演示
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动 EIP-4844 应用场景演示程序");
    println!("{}", "=".repeat(60));
    
    // 基础概念演示
    demonstrate_eip4844_basics();
    
    // Blob 到承诺转换演示
    demonstrate_blob_to_commitment()?;
    
    // 证明生成和验证演示
    demonstrate_proof_generation_verification()?;
    
    // 批量验证性能演示
    demonstrate_batch_verification()?;
    
    // 并行计算演示 (如果启用了 parallel 特性)
    #[cfg(feature = "parallel")]
    demonstrate_parallel_performance()?;
    
    // 关键路径性能分析
    demonstrate_critical_path_analysis()?;
    
    // 网络级性能要求验证
    demonstrate_network_performance_requirements()?;
    
    println!("\n🎉 演示完成！");
    println!("通过本章的学习，您已经了解了：");
    println!("  ✅ EIP-4844 的技术背景和设计目标");
    println!("  ✅ Blob 数据结构和 KZG 承诺的工作原理");
    println!("  ✅ 证明生成、验证和批量优化技术");
    println!("  ✅ 并行计算的性能优势");
    println!("  ✅ 网络级性能要求和优化方向");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blob_generation() {
        let blob = generate_random_blob().unwrap();
        assert_eq!(blob.len(), FIELD_ELEMENTS_PER_BLOB);
    }
    
    #[test]
    fn test_commitment_deterministic() {
        let trusted_setup_path = find_trusted_setup_file().unwrap();
        let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path).unwrap();
        
        let blob = create_test_blob().unwrap();
        let commitment1 = blob_to_kzg_commitment_rust(&blob, &kzg_settings).unwrap();
        let commitment2 = blob_to_kzg_commitment_rust(&blob, &kzg_settings).unwrap();
        
        assert_eq!(commitment1.to_bytes(), commitment2.to_bytes());
    }
    
    #[test]
    fn test_proof_verification() {
        let trusted_setup_path = find_trusted_setup_file().unwrap();
        let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path).unwrap();
        
        let blob = create_test_blob().unwrap();
        let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings).unwrap();
        let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings).unwrap();
        
        let is_valid = verify_blob_kzg_proof_rust(&blob, &commitment, &proof, &kzg_settings).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_batch_verification() {
        let trusted_setup_path = find_trusted_setup_file().unwrap();
        let kzg_settings = load_trusted_setup_filename_rust(&trusted_setup_path).unwrap();
        
        let mut blobs = Vec::new();
        let mut commitments = Vec::new();
        let mut proofs = Vec::new();
        
        for _ in 0..3 {
            let blob = generate_random_blob().unwrap();
            let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings).unwrap();
            let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings).unwrap();
            
            blobs.push(blob);
            commitments.push(commitment);
            proofs.push(proof);
        }
        
        let batch_result = verify_blob_kzg_proof_batch_rust(&blobs, &commitments, &proofs, &kzg_settings).unwrap();
        assert!(batch_result);
    }
}
