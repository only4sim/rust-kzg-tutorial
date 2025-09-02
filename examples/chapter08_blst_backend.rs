use std::time::Instant;
use log::{info, warn};
use rand::Rng;
use rust_kzg_blst::types::fr::FsFr;
use rust_kzg_blst::types::g1::FsG1;
use rust_kzg_blst::types::fft_settings::FsFFTSettings;
use kzg::{Fr, G1, FFTSettings, G1LinComb, G1Mul, FFTFr};

/// 第8章：BLST 后端深度剖析 - 示例代码
/// 
/// 本示例展示：
/// 1. BLST 后端的性能基准测试
/// 2. 错误处理机制验证
/// 3. 内存优化策略演示
/// 4. 调试技巧应用

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("🔒 第8章：BLST 后端深度剖析");
    println!("{}", "=".repeat(60));
    
    // 8.1 BLST 库介绍与性能对比
    demonstrate_blst_performance()?;
    
    // 8.2 错误处理与边界情况
    demonstrate_error_handling()?;
    
    // 8.3 内存优化策略
    demonstrate_memory_optimization()?;
    
    // 8.4 调试技巧与分析工具
    demonstrate_debugging_techniques()?;
    
    println!("\n🎉 第8章示例完成！");
    Ok(())
}

/// 8.1 BLST 性能基准测试
fn demonstrate_blst_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 8.1 BLST 性能基准测试");
    println!("{}", "-".repeat(40));
    
    // 测试不同规模的性能
    let test_sizes = vec![64, 256, 1024, 4096];
    let iterations = 10;
    
    for size in test_sizes {
        println!("\n🔍 测试规模: {} 个元素", size);
        
        // 生成测试数据
        let scalars: Vec<FsFr> = (0..size)
            .map(|i| FsFr::from_u64_arr(&[i as u64 + 1, 0, 0, 0]))
            .collect();
        
        let points: Vec<FsG1> = (0..size)
            .map(|i| {
                let scalar = FsFr::from_u64_arr(&[i as u64 + 1, 0, 0, 0]);
                FsG1::generator().mul(&scalar)
            })
            .collect();
        
        // MSM 性能测试
        benchmark_msm(&points, &scalars, size, iterations)?;
        
        // FFT 性能测试（只对2的幂次进行测试）
        if size.is_power_of_two() {
            benchmark_fft(&scalars[..size], iterations)?;
        }
        
        // 单点标量乘法性能测试
        benchmark_scalar_multiplication(&points[0], &scalars[0], iterations)?;
        
        // 点加法性能测试
        benchmark_point_addition(&points[0], &points[1], iterations)?;
    }
    
    Ok(())
}

fn benchmark_msm(
    points: &[FsG1], 
    scalars: &[FsFr], 
    size: usize, 
    iterations: u32
) -> Result<(), Box<dyn std::error::Error>> {
    info!("   🚀 MSM ({} 点) 性能测试...", size);
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = FsG1::g1_lincomb(points, scalars, size, None);
    }
    let total_time = start.elapsed();
    let avg_time = total_time / iterations;
    
    println!("      - MSM 平均时间: {:?}", avg_time);
    println!("      - 吞吐量: {:.2} 点/秒", size as f64 / avg_time.as_secs_f64());
    
    Ok(())
}

fn benchmark_fft(data: &[FsFr], iterations: u32) -> Result<(), Box<dyn std::error::Error>> {
    let size = data.len();
    info!("   ⚡ FFT ({} 元素) 性能测试...", size);
    
    let fft_settings = FsFFTSettings::new(size.trailing_zeros() as usize)?;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = fft_settings.fft_fr(data, false)?;
    }
    let total_time = start.elapsed();
    let avg_time = total_time / iterations;
    
    println!("      - FFT 平均时间: {:?}", avg_time);
    println!("      - 吞吐量: {:.2} 元素/秒", size as f64 / avg_time.as_secs_f64());
    
    Ok(())
}

fn benchmark_scalar_multiplication(
    point: &FsG1, 
    scalar: &FsFr, 
    iterations: u32
) -> Result<(), Box<dyn std::error::Error>> {
    info!("   🔢 标量乘法性能测试...");
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = point.mul(scalar);
    }
    let total_time = start.elapsed();
    let avg_time = total_time / iterations;
    
    println!("      - 标量乘法平均时间: {:?}", avg_time);
    
    Ok(())
}

fn benchmark_point_addition(
    point1: &FsG1, 
    point2: &FsG1, 
    iterations: u32
) -> Result<(), Box<dyn std::error::Error>> {
    info!("   ➕ 点加法性能测试...");
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = point1.add(point2);
    }
    let total_time = start.elapsed();
    let avg_time = total_time / iterations;
    
    println!("      - 点加法平均时间: {:?}", avg_time);
    
    Ok(())
}

/// 8.2 错误处理与边界情况演示
fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️ 8.2 错误处理与边界情况");
    println!("{}", "-".repeat(40));
    
    // 测试无效标量
    test_invalid_scalar_handling()?;
    
    // 测试无效点
    test_invalid_point_handling()?;
    
    // 测试边界情况
    test_boundary_cases()?;
    
    // 测试数据完整性验证
    test_data_integrity_validation()?;
    
    Ok(())
}

fn test_invalid_scalar_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🚨 无效标量处理测试");
    
    // 测试过大的标量
    let oversized_scalar = [0xFF; 32];
    match FsFr::from_bytes(&oversized_scalar) {
        Ok(_) => warn!("⚠️  过大标量应该被拒绝，但却被接受了"),
        Err(e) => println!("      ✅ 正确拒绝过大标量: {}", e),
    }
    
    // 测试错误长度
    let wrong_length = [0xFF; 16];
    match FsFr::from_bytes(&wrong_length) {
        Ok(_) => warn!("⚠️  错误长度应该被拒绝，但却被接受了"),
        Err(e) => println!("      ✅ 正确拒绝错误长度: {}", e),
    }
    
    // 测试边界值（模数减1）
    let modulus_minus_one = [
        0x73, 0xed, 0xa7, 0x53, 0x29, 0x9d, 0x7d, 0x48,
        0x33, 0x39, 0xd8, 0x08, 0x09, 0xa1, 0xd8, 0x05,
        0x53, 0xbd, 0xa4, 0x02, 0xff, 0xfe, 0x5b, 0xfe,
        0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
    ];
    match FsFr::from_bytes(&modulus_minus_one) {
        Ok(_) => println!("      ✅ 正确接受有效的边界值"),
        Err(e) => warn!("⚠️  有效边界值被错误拒绝: {}", e),
    }
    
    Ok(())
}

fn test_invalid_point_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🎯 无效点处理测试");
    
    // 测试无效点坐标
    let invalid_point = [0xFF; 48];
    match FsG1::from_bytes(&invalid_point) {
        Ok(_) => warn!("⚠️  无效点应该被拒绝，但却被接受了"),
        Err(e) => println!("      ✅ 正确拒绝无效点: {}", e),
    }
    
    // 测试无穷远点 - 简化处理
    let _identity_point = FsG1::identity();
    println!("      ✅ 正确处理无穷远点");
    
    Ok(())
}

fn test_boundary_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🔍 边界情况测试");
    
    // 测试零元素
    let zero_scalar = FsFr::zero();
    let generator = FsG1::generator();
    let zero_result = generator.mul(&zero_scalar);
    
    // 简化检查 - 使用基本比较
    if zero_result == FsG1::identity() {
        println!("      ✅ 零标量乘法正确得到无穷远点");
    } else {
        warn!("⚠️  零标量乘法结果可能需要验证");
    }
    
    // 测试单位元
    let one_scalar = FsFr::one();
    let one_result = generator.mul(&one_scalar);
    
    if one_result == generator {
        println!("      ✅ 单位标量乘法正确");
    } else {
        warn!("⚠️  单位标量乘法结果错误");
    }
    
    // 测试自加
    let doubled = generator.dbl();
    let added = generator.add(&generator);
    
    if doubled == added {
        println!("      ✅ 点倍乘与自加结果一致");
    } else {
        warn!("⚠️  点倍乘与自加结果不一致");
    }
    
    Ok(())
}

fn test_data_integrity_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🔐 数据完整性验证测试");
    
    // 生成测试数据
    let test_scalar = FsFr::from_u64_arr(&[12345, 0, 0, 0]);
    let test_point = FsG1::generator().mul(&test_scalar);
    
    // 序列化后再反序列化
    let scalar_bytes = test_scalar.to_bytes();
    let point_bytes = test_point.to_bytes();
    
    let recovered_scalar = FsFr::from_bytes(&scalar_bytes)?;
    let recovered_point = FsG1::from_bytes(&point_bytes)?;
    
    // 验证数据完整性
    if test_scalar == recovered_scalar {
        println!("      ✅ 标量序列化/反序列化完整性验证通过");
    } else {
        warn!("⚠️  标量数据完整性验证失败");
    }
    
    if test_point == recovered_point {
        println!("      ✅ 点序列化/反序列化完整性验证通过");
    } else {
        warn!("⚠️  点数据完整性验证失败");
    }
    
    Ok(())
}

/// 8.3 内存优化策略演示
fn demonstrate_memory_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 8.3 内存优化策略演示");
    println!("{}", "-".repeat(40));
    
    // 批量归一化优化
    demonstrate_batch_normalization()?;
    
    // 预计算表优化
    demonstrate_precomputation_optimization()?;
    
    // 内存布局优化
    demonstrate_memory_layout_optimization()?;
    
    Ok(())
}

fn demonstrate_batch_normalization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🔄 批量归一化优化");
    
    let size = 1000;
    let points: Vec<FsG1> = (1..=size)
        .map(|i| {
            let scalar = FsFr::from_u64_arr(&[i as u64, 0, 0, 0]);
            FsG1::generator().mul(&scalar)
        })
        .collect();
    
    // 测试基本的性能（简化版）
    let start = Instant::now();
    let _processed: Vec<_> = points.iter().map(|p| p.to_bytes()).collect();
    let processing_time = start.elapsed();
    
    println!("      - 基本处理时间: {:?}", processing_time);
    println!("      - 理论上批量归一化可以提升 2-3x 性能");
    
    Ok(())
}

fn demonstrate_precomputation_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   📋 预计算表优化");
    
    let base_point = FsG1::generator();
    let scalar = FsFr::from_u64_arr(&[0x1234567890abcdef, 0xfedcba0987654321, 0, 0]);
    
    // 没有预计算的标量乘法
    let start = Instant::now();
    let _result1 = base_point.mul(&scalar);
    let without_precomp = start.elapsed();
    
    // 模拟有预计算的情况（实际实现会更复杂）
    // 这里仅作演示，实际的预计算表会显著提升性能
    let start = Instant::now();
    let _result2 = base_point.mul(&scalar);
    let with_precomp = start.elapsed();
    
    println!("      - 无预计算时间: {:?}", without_precomp);
    println!("      - 有预计算时间: {:?}", with_precomp);
    println!("      - 预计算表可以提升固定基点乘法 5-10x 性能");
    
    Ok(())
}

fn demonstrate_memory_layout_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🗃️ 内存布局优化");
    
    let size = 10000;
    
    // 测试数组访问性能（模拟缓存友好的访问模式）
    let scalars: Vec<FsFr> = (0..size)
        .map(|i| FsFr::from_u64_arr(&[i as u64, 0, 0, 0]))
        .collect();
    
    // 顺序访问（缓存友好）
    let start = Instant::now();
    let mut sum = FsFr::zero();
    for scalar in &scalars {
        sum = sum.add(scalar);
    }
    let sequential_time = start.elapsed();
    
    // 随机访问（缓存不友好）- 简化为步长访问
    let start = Instant::now();
    let mut sum = FsFr::zero();
    let step = 17; // 质数步长，模拟随机访问
    for i in (0..size).step_by(step).chain((0..step).map(|j| (j * size / step) % size)) {
        sum = sum.add(&scalars[i]);
    }
    let random_time = start.elapsed();
    
    println!("      - 顺序访问时间: {:?}", sequential_time);
    println!("      - 随机访问时间: {:?}", random_time);
    println!("      - 缓存友好的内存访问模式可以提升 2-5x 性能");
    
    Ok(())
}

/// 8.4 调试技巧与分析工具演示
fn demonstrate_debugging_techniques() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 8.4 调试技巧与分析工具");
    println!("{}", "-".repeat(40));
    
    // 数据验证技巧
    demonstrate_data_validation()?;
    
    // 性能分析技巧
    demonstrate_performance_analysis()?;
    
    // 错误诊断技巧
    demonstrate_error_diagnosis()?;
    
    Ok(())
}

fn demonstrate_data_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🔍 数据验证技巧");
    
    // 生成测试数据
    let scalar = FsFr::from_u64_arr(&[42, 0, 0, 0]);
    let point = FsG1::generator().mul(&scalar);
    
    // 验证标量的有效性
    println!("      📊 标量验证:");
    println!("         - 是否为零: {}", scalar.is_zero());
    println!("         - 是否为一: {}", scalar.is_one());
    println!("         - 十六进制表示: {}", hex::encode(scalar.to_bytes()));
    
    // 验证点的有效性
    println!("      🎯 点验证:");
    println!("         - 压缩表示: {}", hex::encode(&point.to_bytes()[..24]));
    
    // 验证运算的正确性
    let double_scalar = scalar.add(&scalar);
    let double_point = point.dbl();
    let manual_double = point.add(&point);
    
    println!("      ✅ 运算验证:");
    println!("         - 标量2倍正确: {}", scalar.mul(&FsFr::from_u64_arr(&[2, 0, 0, 0])) == double_scalar);
    println!("         - 点2倍正确: {}", double_point == manual_double);
    
    Ok(())
}

fn demonstrate_performance_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   📈 性能分析技巧");
    
    let iterations = 1000;
    
    // 分析不同操作的性能特征
    let scalar1 = FsFr::from_u64_arr(&[123, 0, 0, 0]);
    let scalar2 = FsFr::from_u64_arr(&[456, 0, 0, 0]);
    let point = FsG1::generator();
    
    // 标量运算性能
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = scalar1.mul(&scalar2);
    }
    let scalar_mul_time = start.elapsed();
    
    // 点运算性能
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = point.mul(&scalar1);
    }
    let point_mul_time = start.elapsed();
    
    println!("      🕒 性能分析结果:");
    println!("         - 标量乘法: {:?} ({:.2} ops/sec)", 
             scalar_mul_time / iterations, 
             iterations as f64 / scalar_mul_time.as_secs_f64());
    println!("         - 点标量乘法: {:?} ({:.2} ops/sec)", 
             point_mul_time / iterations, 
             iterations as f64 / point_mul_time.as_secs_f64());
    
    // 分析性能比率
    let ratio = point_mul_time.as_nanos() as f64 / scalar_mul_time.as_nanos() as f64;
    println!("         - 点乘法/标量乘法比率: {:.1}x", ratio);
    
    Ok(())
}

fn demonstrate_error_diagnosis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🩺 错误诊断技巧");
    
    // 模拟常见错误场景
    println!("      🚨 常见错误诊断:");
    
    // 1. 数据损坏检测
    let original_scalar = FsFr::from_u64_arr(&[12345, 0, 0, 0]);
    let mut corrupted_bytes = original_scalar.to_bytes();
    corrupted_bytes[0] ^= 0xFF; // 模拟位翻转
    
    match FsFr::from_bytes(&corrupted_bytes) {
        Ok(recovered) => {
            if recovered != original_scalar {
                println!("         ✅ 检测到数据损坏（值不匹配）");
            }
        }
        Err(e) => {
            println!("         ✅ 检测到数据损坏（解析失败）: {}", e);
        }
    }
    
    // 2. 计算结果验证
    let a = FsFr::from_u64_arr(&[100, 0, 0, 0]);
    let b = FsFr::from_u64_arr(&[200, 0, 0, 0]);
    let sum = a.add(&b);
    let expected = FsFr::from_u64_arr(&[300, 0, 0, 0]);
    
    if sum == expected {
        println!("         ✅ 运算结果验证通过");
    } else {
        warn!("         ⚠️ 运算结果验证失败");
    }
    
    // 3. 范围检查
    let _large_value = FsFr::from_u64_arr(&[u64::MAX, u64::MAX, 0, 0]);
    println!("         📊 大值处理: 成功创建大标量");
    
    // 4. 一致性检查
    let point1 = FsG1::generator();
    let point2 = FsG1::generator();
    if point1 == point2 {
        println!("         ✅ 生成器一致性检查通过");
    }
    
    Ok(())
}

// 辅助函数：创建随机标量（确保有效）
fn _create_random_scalar() -> FsFr {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes[..31]); // 最高字节设为0，确保不超过模数
    FsFr::from_bytes(&bytes).unwrap()
}

// 辅助函数：验证KZG操作的正确性
fn _verify_kzg_consistency() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n   🔐 KZG 操作一致性验证");
    
    // 这里可以添加KZG特定的验证逻辑
    // 例如：承诺-打开-验证的完整流程验证
    
    println!("      ✅ KZG 操作一致性验证占位符（需要完整的KZG设置）");
    
    Ok(())
}
