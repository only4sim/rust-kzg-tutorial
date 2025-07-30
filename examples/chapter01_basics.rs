// 第1章配套示例代码：椭圆曲线密码学基础操作
// 本示例演示如何使用 Rust KZG 库进行基本的椭圆曲线操作

use rust_kzg_blst::{types::fr::FsFr, types::g1::FsG1};
use kzg::{Fr, G1, G1Mul};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔢 第1章：椭圆曲线密码学基础操作演示");
    println!("{}", "=".repeat(50));

    // 1.1 标量 (Fr) 操作演示
    demonstrate_scalar_operations()?;
    
    // 1.2 椭圆曲线点 (G1) 操作演示  
    demonstrate_point_operations()?;
    
    // 1.3 标量乘法演示
    demonstrate_scalar_multiplication()?;
    
    // 1.4 多项式操作实验
    polynomial_experiment()?;

    println!("\n🎉 第1章示例演示完成！");
    println!("你现在已经掌握了椭圆曲线密码学的基础操作。");
    
    Ok(())
}

/// 演示标量域 Fr 的基本操作
fn demonstrate_scalar_operations() -> Result<(), String> {
    println!("\n📊 1.1 标量域 Fr 操作");
    println!("{}", "-".repeat(30));
    
    // 创建标量元素
    let zero = FsFr::zero();         // 零元素
    let one = FsFr::one();          // 单位元素
    
    println!("零元素验证: {}", zero.is_zero());
    println!("单位元素验证: {}", one.is_one());
    
    // 从字节创建标量 - 注意：需要确保字节数组表示有效的域元素
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes[31] = 5; // 设置为小值，确保有效性
    
    let scalar = FsFr::from_bytes(&scalar_bytes)
        .map_err(|e| format!("创建标量失败: {}", e))?;
    println!("从字节创建的标量: 成功");
    
    // 标量运算
    let _sum = one.add(&scalar);      // 加法
    let _product = scalar.mul(&scalar); // 乘法
    let inverse = scalar.inverse();   // 求逆
    
    println!("标量加法、乘法、求逆: 完成");
    
    // 验证乘法逆元性质: a * a^(-1) = 1
    let should_be_one = scalar.mul(&inverse);
    println!("验证 a * a^(-1) = 1: {}", should_be_one.equals(&one));
    
    Ok(())
}

/// 演示椭圆曲线点 G1 的基本操作
fn demonstrate_point_operations() -> Result<(), String> {
    println!("\n📈 1.2 椭圆曲线点 G1 操作");
    println!("{}", "-".repeat(30));
    
    // 获取生成元
    let generator = FsG1::generator();
    println!("生成元 G: 获取成功");
    
    // 无穷远点（群的单位元）
    let identity = FsG1::identity();
    println!("无穷远点 O: 获取成功");
    
    // 点加法: G + G = 2G
    let doubled_g = generator.add(&generator);
    println!("点加法 G + G: 完成");
    
    // 点减法: 2G - G = G
    let should_be_g = doubled_g.sub(&generator);
    println!("点减法 2G - G = G: {}", should_be_g.equals(&generator));
    
    // 验证群的单位元性质: G + O = G
    let g_plus_o = generator.add(&identity);
    println!("验证 G + O = G: {}", g_plus_o.equals(&generator));
    
    // 点的序列化和反序列化
    let compressed = generator.to_bytes();
    let decompressed = FsG1::from_bytes(&compressed)
        .map_err(|e| format!("反序列化失败: {}", e))?;
    println!("点的序列化/反序列化: {}", 
             generator.equals(&decompressed));
    
    Ok(())
}

/// 演示标量乘法的重要性质
fn demonstrate_scalar_multiplication() -> Result<(), String> {
    println!("\n⚡ 1.3 标量乘法演示");
    println!("{}", "-".repeat(30));
    
    let generator = FsG1::generator();
    
    // 创建两个小的标量，确保有效性
    let mut scalar_a_bytes = [0u8; 32];
    scalar_a_bytes[31] = 3;
    let scalar_a = FsFr::from_bytes(&scalar_a_bytes)?;
    
    let mut scalar_b_bytes = [0u8; 32];
    scalar_b_bytes[31] = 5;
    let scalar_b = FsFr::from_bytes(&scalar_b_bytes)?;
    
    // 标量乘法: aG, bG
    let point_a = generator.mul(&scalar_a);
    let point_b = generator.mul(&scalar_b);
    
    println!("计算 aG 和 bG: 完成");
    
    // 验证分配律: (a + b)G = aG + bG
    let sum_scalar = scalar_a.add(&scalar_b);
    let left_side = generator.mul(&sum_scalar);    // (a + b)G
    let right_side = point_a.add(&point_b);       // aG + bG
    
    println!("验证分配律 (a+b)G = aG + bG: {}", 
             left_side.equals(&right_side));
    
    // 验证结合律: a(bG) = (ab)G
    let product_scalar = scalar_a.mul(&scalar_b);
    let left_side = point_b.mul(&scalar_a);        // a(bG)
    let right_side = generator.mul(&product_scalar); // (ab)G
    
    println!("验证结合律 a(bG) = (ab)G: {}", 
             left_side.equals(&right_side));
    
    // 演示大数标量乘法的效率
    let mut large_scalar_bytes = [0u8; 32];
    large_scalar_bytes[31] = 255;  // 只设置最低字节，避免超出域大小
    let large_scalar = FsFr::from_bytes(&large_scalar_bytes)?;
    
    let start = std::time::Instant::now();
    let _large_result = generator.mul(&large_scalar);
    let duration = start.elapsed();
    
    println!("大数标量乘法耗时: {:?}", duration);
    
    Ok(())
}

/// 多项式操作实验
fn polynomial_experiment() -> Result<(), String> {
    println!("\n🧪 1.4 多项式操作实验");
    println!("{}", "-".repeat(30));
    
    // 定义多项式 f(x) = 2 + 3x + x²
    // 使用有效的小标量
    let mut coeff_2_bytes = [0u8; 32];
    coeff_2_bytes[31] = 2;
    let coeff_2 = FsFr::from_bytes(&coeff_2_bytes)?;
    
    let mut coeff_3_bytes = [0u8; 32];
    coeff_3_bytes[31] = 3;
    let coeff_3 = FsFr::from_bytes(&coeff_3_bytes)?;
    
    let mut coeff_1_bytes = [0u8; 32];
    coeff_1_bytes[31] = 1;
    let coeff_1 = FsFr::from_bytes(&coeff_1_bytes)?;
    
    let f = vec![coeff_2, coeff_3, coeff_1];  // [2, 3, 1]
    
    // 创建求值点 x = 5
    let mut x_bytes = [0u8; 32];
    x_bytes[31] = 5;
    let x = FsFr::from_bytes(&x_bytes)?;
    
    // 计算 f(5) = 2 + 3*5 + 1*25 = 42
    let result = evaluate_polynomial(&f, x);
    
    // 验证结果
    let mut expected_bytes = [0u8; 32];
    expected_bytes[31] = 42;
    let expected = FsFr::from_bytes(&expected_bytes)?;
    
    println!("f(5) 计算结果验证: {}", result.equals(&expected));
    
    // 演示多项式加法的同态性
    let g = vec![coeff_1, coeff_2, coeff_3]; // g(x) = 1 + 2x + 3x²
    
    // f(x) + g(x) = (2+1) + (3+2)x + (1+3)x² = 3 + 5x + 4x²
    let sum_poly = add_polynomials(&f, &g);
    
    // 验证在 x=5 处的值
    let f_at_5 = evaluate_polynomial(&f, x);
    let g_at_5 = evaluate_polynomial(&g, x);
    let sum_at_5 = evaluate_polynomial(&sum_poly, x);
    let expected_sum = f_at_5.add(&g_at_5);
    
    println!("多项式加法同态性验证: {}", sum_at_5.equals(&expected_sum));
    
    println!("多项式操作实验完成！");
    Ok(())
}

// 辅助函数：多项式求值
fn evaluate_polynomial(coeffs: &[FsFr], x: FsFr) -> FsFr {
    let mut result = FsFr::zero();
    let mut x_power = FsFr::one();
    
    for coeff in coeffs.iter() {
        let term = coeff.mul(&x_power);
        result = result.add(&term);
        x_power = x_power.mul(&x);
    }
    
    result
}

// 辅助函数：多项式加法
fn add_polynomials(f: &[FsFr], g: &[FsFr]) -> Vec<FsFr> {
    let max_len = f.len().max(g.len());
    let mut result = Vec::with_capacity(max_len);
    
    for i in 0..max_len {
        let f_coeff = if i < f.len() { f[i].clone() } else { FsFr::zero() };
        let g_coeff = if i < g.len() { g[i].clone() } else { FsFr::zero() };
        result.push(f_coeff.add(&g_coeff));
    }
    
    result
}
