// Hello KZG - KZG 承诺方案入门示例
// 这是一个完整的 KZG 操作流程演示

use kzg::eip_4844::{
    blob_to_kzg_commitment_rust, 
    compute_blob_kzg_proof_rust,
    verify_blob_kzg_proof_rust,
    FIELD_ELEMENTS_PER_BLOB,
};
use kzg::Fr;
use rust_kzg_blst::eip_4844::load_trusted_setup_filename_rust;
use rust_kzg_blst::{
    types::kzg_settings::FsKZGSettings,
    types::fr::FsFr,
};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Hello KZG World!");
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
    let commitment = blob_to_kzg_commitment_rust(&blob, &kzg_settings)?;
    let commitment_time = start.elapsed();
    println!("✅ KZG 承诺生成成功! 耗时: {:?}\n", commitment_time);

    // 4. 生成证明
    println!("📝 步骤 4: 生成 KZG 证明...");
    let start = Instant::now();
    let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &kzg_settings)?;
    let proof_time = start.elapsed();
    println!("✅ KZG 证明生成成功! 耗时: {:?}\n", proof_time);

    // 5. 验证证明
    println!("🔍 步骤 5: 验证 KZG 证明...");
    let start = Instant::now();
    let is_valid = verify_blob_kzg_proof_rust(
        &blob, &commitment, &proof, &kzg_settings
    )?;
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
    println!("   �� 总耗时: {:?}", commitment_time + proof_time + verify_time);

    // 7. 演示数据大小
    println!("\n📏 数据大小统计:");
    println!("   📊 Blob 数据: {} 个域元素 (≈ 128KB)", blob.len());
    println!("   🔐 承诺大小: 48 字节 (G1 群元素)");
    println!("   📝 证明大小: 48 字节 (G1 群元素)");
    println!("   💾 压缩比: {:.2}%", (96.0 / (blob.len() * 32) as f64) * 100.0);

    Ok(())
}

/// 智能加载受信任设置文件
/// 会尝试多个可能的路径，自动找到文件位置
fn load_trusted_setup_from_file() -> Result<FsKZGSettings, Box<dyn std::error::Error>> {
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
        if std::path::Path::new(path).exists() {
            println!("   ✅ 找到文件: {}", path);
            return Ok(load_trusted_setup_filename_rust(path)?);
        } else {
            println!("   ❌ 未找到: {}", path);
        }
    }

    Err(format!(
        "❌ 未找到受信任设置文件!\n\
         请确保以下任一路径存在 trusted_setup.txt:\n\
         {:#?}\n\
         \n\
         📥 下载命令:\n\
         mkdir -p assets\n\
         cd assets\n\
         wget https://github.com/ethereum/c-kzg-4844/raw/main/src/trusted_setup.txt",
        possible_paths
    ).into())
}

/// 创建有效的测试 Blob 数据
/// Blob 必须包含 4096 个有效的域元素
fn create_test_blob() -> Result<Vec<FsFr>, String> {
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
        
        let element = FsFr::from_bytes(&bytes)
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

#[cfg(test)]
mod tests {
    use super::*;
    use kzg::G1;

    #[test]
    fn test_blob_creation() -> Result<(), String> {
        println!("🧪 测试 Blob 创建...");
        let blob = create_test_blob()?;
        
        // 验证 blob 大小
        assert_eq!(blob.len(), 4096, "Blob 大小应为 4096");
        
        // 验证前几个元素
        for (i, element) in blob.iter().take(10).enumerate() {
            println!("   元素 {}: {:?}", i, element.is_zero());
        }
        
        println!("✅ Blob 创建测试通过!");
        Ok(())
    }

    #[test] 
    fn test_kzg_commitment_consistency() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 测试 KZG 承诺一致性...");
        
        // 注意：这个测试需要受信任设置文件存在
        if let Ok(settings) = load_trusted_setup_from_file() {
            let blob = create_test_blob()?;
            
            // 多次生成承诺应该得到相同结果
            let commitment1 = blob_to_kzg_commitment_rust(&blob, &settings)?;
            let commitment2 = blob_to_kzg_commitment_rust(&blob, &settings)?;
            
            assert!(commitment1.equals(&commitment2), "承诺应该保持一致");
            println!("✅ KZG 承诺一致性测试通过!");
        } else {
            println!("⚠️  跳过 KZG 测试 (未找到受信任设置文件)");
        }
        
        Ok(())
    }

    #[test]
    fn test_full_kzg_workflow() -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 测试完整 KZG 工作流程...");
        
        if let Ok(settings) = load_trusted_setup_from_file() {
            let blob = create_test_blob()?;
            
            // 完整的承诺-证明-验证流程
            let commitment = blob_to_kzg_commitment_rust(&blob, &settings)?;
            let proof = compute_blob_kzg_proof_rust(&blob, &commitment, &settings)?;
            let is_valid = verify_blob_kzg_proof_rust(&blob, &commitment, &proof, &settings)?;
            
            assert!(is_valid, "完整的 KZG 工作流程应该验证成功");
            println!("✅ 完整 KZG 工作流程测试通过!");
        } else {
            println!("⚠️  跳过 KZG 工作流程测试 (未找到受信任设置文件)");
        }
        
        Ok(())
    }
}

// 运行示例的方法：
// cargo run --example hello_kzg
//
// 运行测试的方法：
// cargo test --example hello_kzg
//
// 带详细输出运行测试：
// cargo test --example hello_kzg -- --nocapture
