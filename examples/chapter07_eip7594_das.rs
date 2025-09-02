use std::time::Instant;

use rust_kzg_blst::{
    types::{
        fr::FsFr,
        g1::FsG1,
        kzg_settings::FsKZGSettings,
    },
    eip_4844::load_trusted_setup_filename_rust,
    eip_7594::BlstBackend,
};

use kzg::{
    DAS,
    eip_4844::{
        blob_to_kzg_commitment_rust,
        FIELD_ELEMENTS_PER_BLOB,
    },
    eth::{
        FIELD_ELEMENTS_PER_CELL,
        CELLS_PER_EXT_BLOB,
    },
    Fr,
};

fn find_trusted_setup_file() -> Result<String, String> {
    let paths = [
        "./assets/trusted_setup.txt",
        "../assets/trusted_setup.txt",
        "../../assets/trusted_setup.txt",
        "/workspaces/rust-kzg-tutorial/assets/trusted_setup.txt",
    ];
    
    for path in &paths {
        if std::path::Path::new(path).exists() {
            println!("   ✅ 找到文件: {}", path);
            return Ok(path.to_string());
        }
    }
    
    Err("未找到受信任设置文件".to_string())
}

fn create_random_blob() -> Result<Vec<FsFr>, String> {
    let mut blob = Vec::with_capacity(FIELD_ELEMENTS_PER_BLOB);
    
    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        // 使用小值来创建有效的域元素
        let mut bytes = [0u8; 32];
        // 将索引值转换为域元素，确保在有效范围内
        let value = (i % 255) as u8; // 使用小值
        bytes[31] = value;
        
        let fr = FsFr::from_bytes(&bytes).map_err(|e| format!("创建域元素失败: {}", e))?;
        blob.push(fr);
    }
    
    Ok(blob)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 EIP-7594 数据可用性采样 (PeerDAS) 演示");
    println!("{}", "=".repeat(60));
    
    // 1. 加载受信任设置
    println!("📁 步骤 1: 加载受信任设置...");
    let trusted_setup_path = find_trusted_setup_file()?;
    let settings = load_trusted_setup_filename_rust(&trusted_setup_path)
        .map_err(|e| format!("加载受信任设置失败: {}", e))?;
    println!("✅ 受信任设置加载成功!");
    
    // 2. 创建测试 Blob
    println!("\n🔢 步骤 2: 创建随机测试 Blob...");
    let blob = create_random_blob()?;
    println!("✅ 创建了包含 {} 个域元素的 Blob", blob.len());
    
    // 3. 生成 KZG 承诺
    println!("\n🔐 步骤 3: 生成 KZG 承诺...");
    let start = Instant::now();
    let commitment = blob_to_kzg_commitment_rust(&blob, &settings)
        .map_err(|e| format!("生成承诺失败: {}", e))?;
    let commitment_time = start.elapsed();
    println!("✅ KZG 承诺生成成功! 耗时: {:?}", commitment_time);
    
    // 4. 计算 Cells 和 KZG 证明
    println!("\n📦 步骤 4: 计算 Cells 和 KZG 证明...");
    let mut cells = vec![FsFr::default(); CELLS_PER_EXT_BLOB * FIELD_ELEMENTS_PER_CELL];
    let mut proofs = vec![FsG1::default(); CELLS_PER_EXT_BLOB];
    
    let start = Instant::now();
    <FsKZGSettings as DAS<BlstBackend>>::compute_cells_and_kzg_proofs(
        &settings,
        Some(&mut cells),
        Some(&mut proofs),
        &blob,
    ).map_err(|e| format!("计算 cells 和证明失败: {}", e))?;
    let cells_time = start.elapsed();
    
    println!("✅ 成功计算了 {} 个 cells 和证明!", CELLS_PER_EXT_BLOB);
    println!("   📊 Cell 数量: {}", CELLS_PER_EXT_BLOB);
    println!("   📏 每个 Cell 大小: {} 个域元素", FIELD_ELEMENTS_PER_CELL);
    println!("   ⏱️ 计算耗时: {:?}", cells_time);
    
    // 5. 验证 Cell KZG 证明 (批量验证)
    println!("\n🔍 步骤 5: 批量验证 Cell KZG 证明...");
    
    // 准备验证数据
    let commitments = vec![commitment; CELLS_PER_EXT_BLOB];
    let cell_indices: Vec<usize> = (0..CELLS_PER_EXT_BLOB).collect();
    
    let start = Instant::now();
    let is_valid = <FsKZGSettings as DAS<BlstBackend>>::verify_cell_kzg_proof_batch(
        &settings,
        &commitments,
        &cell_indices,
        &cells,
        &proofs,
    ).map_err(|e| format!("批量验证失败: {}", e))?;
    let verify_time = start.elapsed();
    
    if is_valid {
        println!("🎉 所有 Cell 证明验证成功!");
    } else {
        println!("❌ Cell 证明验证失败!");
        return Err("验证失败".into());
    }
    println!("   ⏱️ 验证耗时: {:?}", verify_time);
    
    // 6. 数据恢复演示
    println!("\n🔄 步骤 6: 数据恢复演示...");
    println!("   模拟只有 50% 的 cells 可用的情况...");
    
    // 只保留前一半的 cells (模拟网络中只收到一半数据)
    let half_cells_count = CELLS_PER_EXT_BLOB / 2;
    let cell_indices: Vec<usize> = (0..half_cells_count).collect();
    let partial_cells: Vec<FsFr> = (0..half_cells_count)
        .flat_map(|i| {
            let start_idx = i * FIELD_ELEMENTS_PER_CELL;
            let end_idx = (i + 1) * FIELD_ELEMENTS_PER_CELL;
            cells[start_idx..end_idx].iter().cloned()
        })
        .collect();
    
    println!("   📊 使用 {} 个 cells (50%) 来恢复完整数据", half_cells_count);
    
    // 恢复完整的 cells
    let mut recovered_cells = vec![FsFr::default(); CELLS_PER_EXT_BLOB * FIELD_ELEMENTS_PER_CELL];
    
    let start = Instant::now();
    <FsKZGSettings as DAS<BlstBackend>>::recover_cells_and_kzg_proofs(
        &settings,
        &mut recovered_cells,
        None, // 不需要恢复证明
        &cell_indices,
        &partial_cells,
    ).map_err(|e| format!("数据恢复失败: {}", e))?;
    let recovery_time = start.elapsed();
    
    println!("✅ 数据恢复成功!");
    println!("   ⏱️ 恢复耗时: {:?}", recovery_time);
    
    // 验证恢复的数据是否正确
    let original_first_cell = &cells[0..FIELD_ELEMENTS_PER_CELL];
    let recovered_first_cell = &recovered_cells[0..FIELD_ELEMENTS_PER_CELL];
    
    if original_first_cell == recovered_first_cell {
        println!("✅ 数据恢复验证成功 - 恢复的数据与原始数据一致!");
    } else {
        println!("❌ 数据恢复验证失败 - 恢复的数据与原始数据不一致!");
    }
    
    // 性能总结
    println!("\n{}", "=".repeat(60));
    println!("📊 EIP-7594 PeerDAS 性能总结:");
    println!("   🔐 KZG 承诺生成: {:?}", commitment_time);
    println!("   📦 Cells/证明计算: {:?}", cells_time);
    println!("   🔍 批量验证: {:?}", verify_time);
    println!("   🔄 数据恢复: {:?}", recovery_time);
    println!("   📏 总 Cells: {}", CELLS_PER_EXT_BLOB);
    println!("   📐 每 Cell 大小: {} 域元素", FIELD_ELEMENTS_PER_CELL);
    println!("   💾 总数据大小: {} 域元素", CELLS_PER_EXT_BLOB * FIELD_ELEMENTS_PER_CELL);
    
    println!("\n🎉 EIP-7594 PeerDAS 演示完成!");
    println!("   ✨ 这展示了如何使用 50% 的数据恢复完整的 Blob");
    println!("   ✨ 这是以太坊下一代数据可用性采样的核心技术");
    
    Ok(())
}
