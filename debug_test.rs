use rust_kzg_blst::types::{fr::FsFr, g1::FsG1};
use kzg::{Fr, G1, G1Mul};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generator = FsG1::generator();
    
    // 点加法: G + G = 2G
    let doubled_g = generator.add(&generator);
    
    // 用标量乘法验证: 2 * G
    let mut scalar_2_bytes = [0u8; 32];
    scalar_2_bytes[31] = 2;
    let scalar_2 = FsFr::from_bytes(&scalar_2_bytes)?;
    let doubled_g_scalar = generator.mul(&scalar_2);
    
    println!("G + G == 2*G: {}", doubled_g.equals(&doubled_g_scalar));
    
    // 点减法: 2G - G
    let result = doubled_g.sub(&generator);
    
    // 应该等于 G
    println!("2G - G == G: {}", result.equals(&generator));
    
    // 或者应该等于 1*G
    let mut scalar_1_bytes = [0u8; 32];
    scalar_1_bytes[31] = 1;
    let scalar_1 = FsFr::from_bytes(&scalar_1_bytes)?;
    let g_scalar = generator.mul(&scalar_1);
    
    println!("2G - G == 1*G: {}", result.equals(&g_scalar));
    println!("G == 1*G: {}", generator.equals(&g_scalar));
    
    Ok(())
}
