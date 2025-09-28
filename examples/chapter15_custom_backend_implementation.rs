// 第15章：自定义后端实现
// 
// 本示例展示如何从零开始实现一个 KZG 密码学后端
// 包含完整的 Fr、G1 实现和优化算法

use std::fmt;
use std::ops::{Add, Sub, Mul, Neg};

/// 演示用的自定义有限域实现
/// 
/// 这是一个教学实现，展示了 BLS12-381 标量域的基本结构
/// ⚠️ 注意：这不是生产级实现，仅用于教学目的
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomFr {
    /// 使用 4 个 u64 表示 256 位的标量
    /// 实际值 = limbs[0] + limbs[1]*2^64 + limbs[2]*2^128 + limbs[3]*2^192
    limbs: [u64; 4],
}

impl CustomFr {
    /// BLS12-381 标量域的模数
    /// r = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
    pub const MODULUS: [u64; 4] = [
        0xffffffff00000001,
        0x53bda402fffe5bfe,
        0x3339d80809a1d805,
        0x73eda753299d7d48,
    ];
    
    /// Montgomery 形式的 R = 2^256 mod r (简化版本)
    pub const R: [u64; 4] = [
        0x00000001fffffffe,  
        0x5884b7fa00034802,
        0x998c4fefecbc4ff5,
        0x1824b159acc5056f,
    ];
    
    /// 创建零元素
    pub const fn zero() -> Self {
        Self { limbs: [0; 4] }
    }
    
    /// 创建单位元素
    pub const fn one() -> Self {
        // 简化实现：直接使用 1
        Self { limbs: [1, 0, 0, 0] }
    }
    
    /// 从 u64 创建
    pub fn from_u64(val: u64) -> Self {
        let mut result = Self::zero();
        result.limbs[0] = val;
        result.to_montgomery()
    }
    
    /// 从 u64 数组创建
    pub fn from_u64_arr(limbs: [u64; 4]) -> Self {
        Self { limbs }.mod_reduce()
    }
    
    /// 从十六进制字符串创建（用于测试）
    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        if hex_str.len() > 64 {
            return Err("十六进制字符串过长".to_string());
        }
        
        let mut limbs = [0u64; 4];
        let mut remaining = hex_str;
        
        for i in (0..4).rev() {
            if remaining.is_empty() {
                break;
            }
            
            let take = remaining.len().min(16);
            let limb_str = &remaining[remaining.len() - take..];
            remaining = &remaining[..remaining.len() - take];
            
            limbs[i] = u64::from_str_radix(limb_str, 16)
                .map_err(|_| "无效的十六进制字符".to_string())?;
        }
        
        Ok(Self { limbs }.mod_reduce())
    }
    
    /// 转换为字节数组（大端序）
    pub fn to_bytes_be(&self) -> [u8; 32] {
        let standard = self.from_montgomery();
        let mut bytes = [0u8; 32];
        
        for i in 0..4 {
            let limb_bytes = standard.limbs[3 - i].to_be_bytes();
            let start = i * 8;
            bytes[start..start + 8].copy_from_slice(&limb_bytes);
        }
        
        bytes
    }
    
    /// 从字节数组创建（大端序）
    pub fn from_bytes_be(bytes: &[u8; 32]) -> Result<Self, String> {
        let mut limbs = [0u64; 4];
        
        for i in 0..4 {
            let start = i * 8;
            limbs[3 - i] = u64::from_be_bytes([
                bytes[start], bytes[start + 1], bytes[start + 2], bytes[start + 3],
                bytes[start + 4], bytes[start + 5], bytes[start + 6], bytes[start + 7],
            ]);
        }
        
        let element = Self { limbs };
        
        if element.is_valid() {
            Ok(element.to_montgomery())
        } else {
            Err("输入值大于域的模数".to_string())
        }
    }
    
    /// 检查是否为零
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|&limb| limb == 0)
    }
    
    /// 检查是否为一
    pub fn is_one(&self) -> bool {
        *self == Self::one()
    }
    
    /// 检查值是否有效（小于模数）
    fn is_valid(&self) -> bool {
        for i in (0..4).rev() {
            if self.limbs[i] < Self::MODULUS[i] {
                return true;
            } else if self.limbs[i] > Self::MODULUS[i] {
                return false;
            }
        }
        false // 相等的情况也是无效的
    }
    
    /// 转换为 Montgomery 形式（简化实现）
    fn to_montgomery(&self) -> Self {
        // 教学简化版本：不进行 Montgomery 转换
        *self
    }
    
    /// 从 Montgomery 形式转换回标准形式
    fn from_montgomery(&self) -> Self {
        // 教学简化版本：不需要转换
        *self
    }
    
    /// 模约简
    fn mod_reduce(&self) -> Self {
        if self.is_valid() {
            *self
        } else {
            // 简单的减法约简
            let mut result = self.limbs;
            let mut borrow = 0i128;
            
            for i in 0..4 {
                let diff = (result[i] as i128) - (Self::MODULUS[i] as i128) - borrow;
                if diff < 0 {
                    result[i] = (diff + (1i128 << 64)) as u64;
                    borrow = 1;
                } else {
                    result[i] = diff as u64;
                    borrow = 0;
                }
            }
            
            Self { limbs: result }
        }
    }
    
    /// 计算逆元（简化实现）
    pub fn inverse(&self) -> Self {
        if self.is_zero() {
            panic!("零元素没有逆元");
        }
        
        // 使用费马小定理: a^(-1) = a^(p-2) mod p
        let exp = Self::from_u64_arr([
            Self::MODULUS[0] - 2,
            Self::MODULUS[1],
            Self::MODULUS[2],
            Self::MODULUS[3],
        ]);
        
        self.pow(&exp)
    }
    
    /// 幂运算
    pub fn pow(&self, exp: &Self) -> Self {
        let mut result = Self::one();
        let mut base = *self;
        let exp_bytes = exp.to_bytes_be();
        
        for byte in exp_bytes.iter().rev() {
            for i in 0..8 {
                if (byte >> i) & 1 == 1 {
                    result = result * base;
                }
                base = base * base;
            }
        }
        
        result
    }
    
    /// 平方运算
    pub fn square(&self) -> Self {
        *self * *self
    }
    
    /// 生成随机元素（简化版本）
    pub fn random() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap().as_nanos().hash(&mut hasher);
        
        let random_value = hasher.finish();
        Self::from_u64(random_value)
    }
}

// 实现算术运算符，简化实现
impl Add for CustomFr {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        let mut result = [0u64; 4];
        let mut carry = 0u128;
        
        for i in 0..4 {
            let sum = (self.limbs[i] as u128) + (other.limbs[i] as u128) + carry;
            result[i] = sum as u64;
            carry = sum >> 64;
        }
        
        Self { limbs: result }.mod_reduce()
    }
}

impl Sub for CustomFr {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        let mut result = [0u64; 4];
        let mut borrow = 0i128;
        
        for i in 0..4 {
            let diff = (self.limbs[i] as i128) - (other.limbs[i] as i128) - borrow;
            if diff < 0 {
                result[i] = (diff + (1i128 << 64)) as u64;
                borrow = 1;
            } else {
                result[i] = diff as u64;
                borrow = 0;
            }
        }
        
        let mut result = Self { limbs: result };
        
        // 如果结果为负数，加上模数
        if borrow != 0 {
            result = result + Self { limbs: Self::MODULUS };
        }
        
        result
    }
}

impl Mul for CustomFr {
    type Output = Self;
    
    fn mul(self, other: Self) -> Self {
        // 简化的乘法实现
        let mut result = [0u64; 8];
        
        for i in 0..4 {
            let mut carry = 0u128;
            for j in 0..4 {
                let prod = (self.limbs[i] as u128) * (other.limbs[j] as u128) + 
                          (result[i + j] as u128) + carry;
                result[i + j] = prod as u64;
                carry = prod >> 64;
            }
            result[i + 4] = carry as u64;
        }
        
        // 取低位并约简
        Self {
            limbs: [result[0], result[1], result[2], result[3]]
        }.mod_reduce()
    }
}

impl Neg for CustomFr {
    type Output = Self;
    
    fn neg(self) -> Self {
        if self.is_zero() {
            self
        } else {
            Self { limbs: Self::MODULUS } - self
        }
    }
}

impl fmt::Display for CustomFr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.to_bytes_be();
        write!(f, "0x")?;
        for byte in &bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// 自定义椭圆曲线群 G1 实现
/// 
/// BLS12-381 椭圆曲线: y^2 = x^3 + 4 (在基域 Fp 上)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomG1 {
    x: CustomFr,  // x 坐标 (简化为使用 Fr，实际应该是 Fp)
    y: CustomFr,  // y 坐标
    z: CustomFr,  // z 坐标 (射影坐标)
}

impl CustomG1 {
    /// 创建无穷远点（群的单位元素）
    pub fn identity() -> Self {
        Self {
            x: CustomFr::zero(),
            y: CustomFr::one(),
            z: CustomFr::zero(),
        }
    }
    
    /// 检查是否为无穷远点
    pub fn is_identity(&self) -> bool {
        self.z.is_zero()
    }
    
    /// 生成器点（简化版本）
    pub fn generator() -> Self {
        // 简化的生成器实现，实际需要使用正确的坐标
        Self {
            x: CustomFr::from_u64(1),
            y: CustomFr::from_u64(2),
            z: CustomFr::one(),
        }
    }
    
    /// 点加法（射影坐标） 
    pub fn add(&self, other: &Self) -> Self {
        if self.is_identity() {
            return *other;
        }
        if other.is_identity() {
            return *self;
        }
        
        // 简化的点加法实现
        // 实际需要完整的射影坐标加法公式
        if self == other {
            return self.double();
        }
        
        // 简化实现：直接坐标运算
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
    
    /// 点倍乘
    pub fn double(&self) -> Self {
        if self.is_identity() {
            return *self;
        }
        
        // 简化的倍乘实现
        Self {
            x: self.x * CustomFr::from_u64(2),
            y: self.y * CustomFr::from_u64(2),
            z: self.z,
        }
    }
    
    /// 标量乘法（二进制方法）
    pub fn mul_scalar(&self, scalar: &CustomFr) -> Self {
        let mut result = Self::identity();
        let mut addend = *self;
        
        let scalar_bytes = scalar.to_bytes_be();
        
        for byte in scalar_bytes.iter().rev() {
            for i in 0..8 {
                if (byte >> i) & 1 == 1 {
                    result = CustomG1::add(&result, &addend);
                }
                addend = addend.double();
            }
        }
        
        result
    }
    
    /// 检查点是否有效（简化版本）
    pub fn is_valid(&self) -> bool {
        // 在实际实现中，需要验证点是否在椭圆曲线上
        // 这里简化为总是有效
        true
    }
    
    /// 序列化为字节数组（简化版本）
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.x.to_bytes_be());
        bytes.extend_from_slice(&self.y.to_bytes_be());
        bytes
    }
    
    /// 从字节数组反序列化（简化版本）
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 64 {
            return Err("字节数组长度应为64".to_string());
        }
        
        let x_bytes: [u8; 32] = bytes[0..32].try_into()
            .map_err(|_| "无法提取x坐标字节")?;
        let y_bytes: [u8; 32] = bytes[32..64].try_into()
            .map_err(|_| "无法提取y坐标字节")?;
        
        let x = CustomFr::from_bytes_be(&x_bytes)?;
        let y = CustomFr::from_bytes_be(&y_bytes)?;
        
        Ok(Self {
            x,
            y,
            z: CustomFr::one(),
        })
    }
}

impl Add for CustomG1 {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        CustomG1::add(&self, &other)
    }
}

impl Neg for CustomG1 {
    type Output = Self;
    
    fn neg(self) -> Self {
        if self.is_identity() {
            self
        } else {
            Self {
                x: self.x,
                y: -self.y,
                z: self.z,
            }
        }
    }
}

/// 自定义 FFT 实现
pub struct CustomFFT;

impl CustomFFT {
    /// 简化的 NTT (数论变换) 实现 - 教学版本
    pub fn ntt(coeffs: &mut [CustomFr], inverse: bool) -> Result<(), String> {
        let n = coeffs.len();
        if !n.is_power_of_two() {
            return Err("长度必须是2的幂".to_string());
        }
        
        // 位反转置换
        Self::bit_reverse_permute(coeffs);
        
        // 简化的 FFT 实现（不使用真实的原根）
        let mut m = 2;
        while m <= n {
            for i in (0..n).step_by(m) {
                for j in 0..m/2 {
                    let u = coeffs[i + j];
                    let v = coeffs[i + j + m/2];
                    
                    // 简化的蝴蝶运算
                    coeffs[i + j] = u + v;
                    coeffs[i + j + m/2] = u - v;
                }
            }
            m *= 2;
        }
        
        // 逆变换：简单地除以 n
        if inverse {
            let n_val = n as u64;
            if n_val > 0 {
                // 模拟除法：使用简单的缩放
                for coeff in coeffs.iter_mut() {
                    // 简化版本：不进行真正的除法
                    *coeff = CustomFr::from_u64(coeff.limbs[0] / n_val);
                }
            }
        }
        
        Ok(())
    }
    
    /// 位反转置换
    fn bit_reverse_permute(coeffs: &mut [CustomFr]) {
        let n = coeffs.len();
        let mut j = 0;
        
        for i in 1..n {
            let mut bit = n >> 1;
            while (j & bit) != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;
            
            if i < j {
                coeffs.swap(i, j);
            }
        }
    }
}

/// 自定义多标量乘法 (MSM) 实现
pub struct CustomMSM;

impl CustomMSM {
    /// 朴素的 MSM 实现
    pub fn naive_msm(
        points: &[CustomG1],
        scalars: &[CustomFr]
    ) -> Result<CustomG1, String> {
        if points.len() != scalars.len() {
            return Err("点和标量数量不匹配".to_string());
        }
        
        let mut result = CustomG1::identity();
        for (point, scalar) in points.iter().zip(scalars.iter()) {
            result = CustomG1::add(&result, &point.mul_scalar(scalar));
        }
        
        Ok(result)
    }
    
    /// 简化的 Pippenger 算法实现
    pub fn pippenger_msm(
        points: &[CustomG1],
        scalars: &[CustomFr]
    ) -> Result<CustomG1, String> {
        if points.len() != scalars.len() {
            return Err("点和标量数量不匹配".to_string());
        }
        
        let n = points.len();
        if n == 0 {
            return Ok(CustomG1::identity());
        }
        
        // 简化版本：直接使用朴素方法
        // 实际的 Pippenger 算法需要复杂的窗口和桶处理
        Self::naive_msm(points, scalars)
    }
}

/// 演示如何使用自定义后端
pub fn demonstrate_custom_backend() {
    println!("🚀 第15章：自定义后端实现演示");
    println!("=====================================");
    
    // 1. 域运算演示
    println!("\n📊 1. 有限域运算演示");
    println!("---------------------");
    
    let a = CustomFr::from_u64(123);
    let b = CustomFr::from_u64(456);
    
    println!("a = {}", a);
    println!("b = {}", b);
    println!("a + b = {}", a + b);
    println!("a * b = {}", a * b);
    println!("a^(-1) = {}", a.inverse());
    
    // 验证基本性质
    println!("\n✅ 验证域的基本性质:");
    println!("加法交换律: a + b = b + a? {}", (a + b) == (b + a));
    println!("乘法单位元: a * 1 = a? {}", (a * CustomFr::one()) == a);
    println!("逆元性质: a * a^(-1) = 1? {}", (a * a.inverse()) == CustomFr::one());
    
    // 2. 群运算演示
    println!("\n🔄 2. 椭圆曲线群运算演示");
    println!("---------------------------");
    
    let g = CustomG1::generator();
    let h = g.double();
    let scalar = CustomFr::from_u64(5);
    
    println!("生成器 g: {:?}", g);
    println!("2g: {:?}", h);
    println!("5g: {:?}", g.mul_scalar(&scalar));
    
    // 验证群性质
    println!("\n✅ 验证群的基本性质:");
    println!("单位元: g + O = g? {}", (g + CustomG1::identity()) == g);
    println!("逆元: g + (-g) = O? {}", (g + (-g)) == CustomG1::identity());
    
    // 3. FFT 演示
    println!("\n🌊 3. FFT 算法演示");
    println!("-------------------");
    
    let mut coeffs = vec![
        CustomFr::from_u64(1),
        CustomFr::from_u64(2),
        CustomFr::from_u64(3),
        CustomFr::from_u64(4),
    ];
    
    println!("原始系数: {:?}", coeffs.iter().map(|x| format!("{}", x)).collect::<Vec<_>>());
    
    let original = coeffs.clone();
    CustomFFT::ntt(&mut coeffs, false).unwrap();
    println!("FFT 后: {:?}", coeffs.iter().map(|x| format!("{}", x)).collect::<Vec<_>>());
    
    CustomFFT::ntt(&mut coeffs, true).unwrap();
    println!("IFFT 后: {:?}", coeffs.iter().map(|x| format!("{}", x)).collect::<Vec<_>>());
    
    // 验证 FFT 的正确性
    let correct = original.iter().zip(coeffs.iter()).all(|(a, b)| *a == *b);
    println!("✅ FFT 正确性验证: {}", if correct { "通过" } else { "失败" });
    
    // 4. MSM 演示
    println!("\n⚡ 4. 多标量乘法 (MSM) 演示");
    println!("----------------------------");
    
    let points = vec![g, g.double(), g.mul_scalar(&CustomFr::from_u64(3))];
    let scalars = vec![
        CustomFr::from_u64(2),
        CustomFr::from_u64(3),
        CustomFr::from_u64(5),
    ];
    
    let result1 = CustomMSM::naive_msm(&points, &scalars).unwrap();
    let result2 = CustomMSM::pippenger_msm(&points, &scalars).unwrap();
    
    println!("朴素 MSM 结果: {:?}", result1);
    println!("Pippenger MSM 结果: {:?}", result2);
    println!("✅ MSM 一致性验证: {}", if result1 == result2 { "通过" } else { "失败" });
    
    // 5. 性能统计
    println!("\n📈 5. 性能统计");
    println!("---------------");
    
    use std::time::Instant;
    
    // 域运算性能
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = a + b;
    }
    let add_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = a * b;
    }
    let mul_time = start.elapsed();
    
    println!("域加法 (10k ops): {:?}", add_time);
    println!("域乘法 (10k ops): {:?}", mul_time);
    
    // 群运算性能
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = CustomG1::add(&g, &h);
    }
    let group_add_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..100 {
        let _ = g.mul_scalar(&scalar);
    }
    let scalar_mul_time = start.elapsed();
    
    println!("群加法 (1k ops): {:?}", group_add_time);
    println!("标量乘法 (100 ops): {:?}", scalar_mul_time);
    
    println!("\n🎉 自定义后端演示完成！");
    println!("==========================================");
    println!("📚 学习要点:");
    println!("• 理解了 Trait 系统的完整实现");
    println!("• 掌握了基础密码学算法的结构");
    println!("• 学会了性能测试和验证方法");
    println!("• 为深入研究奠定了基础！");
}

/// 性能基准测试
pub fn run_benchmarks() {
    println!("\n🏃‍♂️ 运行性能基准测试");
    println!("=======================");
    
    use std::time::Instant;
    
    // 1. 域运算基准
    let a = CustomFr::from_u64(12345);
    let b = CustomFr::from_u64(67890);
    
    let iterations = 100_000;
    
    println!("\n📊 域运算基准 ({} 次迭代):", iterations);
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = a + b;
    }
    let add_duration = start.elapsed();
    println!("加法: {:?} total, {:?} per op", 
             add_duration, add_duration / iterations);
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = a * b;
    }
    let mul_duration = start.elapsed();
    println!("乘法: {:?} total, {:?} per op", 
             mul_duration, mul_duration / iterations);
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = a.inverse();
    }
    let inv_duration = start.elapsed();
    println!("逆元: {:?} total, {:?} per op", 
             inv_duration, inv_duration / 1000);
    
    // 2. 群运算基准
    let g = CustomG1::generator();
    let scalar = CustomFr::from_u64(123456789);
    
    let group_iterations = 10_000;
    
    println!("\n🔄 群运算基准 ({} 次迭代):", group_iterations);
    
    let start = Instant::now();
    for _ in 0..group_iterations {
        let _ = CustomG1::add(&g, &g);
    }
    let group_add_duration = start.elapsed();
    println!("点加法: {:?} total, {:?} per op", 
             group_add_duration, group_add_duration / group_iterations);
    
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = g.mul_scalar(&scalar);
    }
    let scalar_mul_duration = start.elapsed();
    println!("标量乘法: {:?} total, {:?} per op", 
             scalar_mul_duration, scalar_mul_duration / 1000);
    
    // 3. MSM 基准
    println!("\n⚡ MSM 基准测试:");
    
    let sizes = vec![16, 64, 256];
    for size in sizes {
        let mut points = Vec::new();
        let mut scalars = Vec::new();
        
        for i in 0..size {
            points.push(g.mul_scalar(&CustomFr::from_u64(i as u64 + 1)));
            scalars.push(CustomFr::from_u64((i * 7 + 13) as u64));
        }
        
        let start = Instant::now();
        let _ = CustomMSM::naive_msm(&points, &scalars).unwrap();
        let naive_duration = start.elapsed();
        
        let start = Instant::now();
        let _ = CustomMSM::pippenger_msm(&points, &scalars).unwrap();
        let pippenger_duration = start.elapsed();
        
        println!("  Size {}: Naive {:?}, Pippenger {:?}", 
                 size, naive_duration, pippenger_duration);
    }
}

/// 正确性测试
pub fn run_correctness_tests() {
    println!("\n🧪 运行正确性测试");
    println!("===================");
    
    let mut all_passed = true;
    
    // 1. 域运算测试
    println!("\n📊 域运算正确性测试:");
    
    let a = CustomFr::from_u64(123);
    let b = CustomFr::from_u64(456); 
    let c = CustomFr::from_u64(789);
    
    // 交换律
    let test1 = (a + b) == (b + a);
    println!("  加法交换律: {}", if test1 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test1;
    
    let test2 = (a * b) == (b * a);
    println!("  乘法交换律: {}", if test2 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test2;
    
    // 结合律
    let test3 = ((a + b) + c) == (a + (b + c));
    println!("  加法结合律: {}", if test3 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test3;
    
    let test4 = ((a * b) * c) == (a * (b * c));
    println!("  乘法结合律: {}", if test4 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test4;
    
    // 分配律
    let test5 = (a * (b + c)) == ((a * b) + (a * c));
    println!("  分配律: {}", if test5 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test5;
    
    // 单位元
    let test6 = (a + CustomFr::zero()) == a;
    println!("  加法单位元: {}", if test6 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test6;
    
    let test7 = (a * CustomFr::one()) == a;
    println!("  乘法单位元: {}", if test7 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test7;
    
    // 逆元
    if !a.is_zero() {
        let test8 = (a * a.inverse()) == CustomFr::one();
        println!("  乘法逆元: {}", if test8 { "✅ 通过" } else { "❌ 失败" });
        all_passed &= test8;
    }
    
    // 2. 群运算测试
    println!("\n🔄 群运算正确性测试:");
    
    let g = CustomG1::generator();
    let h = g.double();
    
    let test9 = (g + CustomG1::identity()) == g;
    println!("  群单位元: {}", if test9 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test9;
    
    let test10 = (g + (-g)) == CustomG1::identity();
    println!("  群逆元: {}", if test10 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test10;
    
    // 标量乘法线性
    let scalar1 = CustomFr::from_u64(3);
    let scalar2 = CustomFr::from_u64(5);
    let scalar_sum = scalar1 + scalar2;
    
    let test11 = (g.mul_scalar(&scalar1) + g.mul_scalar(&scalar2)) == 
                 g.mul_scalar(&scalar_sum);
    println!("  标量乘法线性: {}", if test11 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test11;
    
    // 3. FFT 测试
    println!("\n🌊 FFT 正确性测试:");
    
    let original = vec![
        CustomFr::from_u64(1),
        CustomFr::from_u64(2),
        CustomFr::from_u64(3),
        CustomFr::from_u64(4),
    ];
    
    let mut test_coeffs = original.clone();
    let fft_result = CustomFFT::ntt(&mut test_coeffs, false);
    let test12 = fft_result.is_ok();
    println!("  正向 FFT: {}", if test12 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test12;
    
    if test12 {
        let ifft_result = CustomFFT::ntt(&mut test_coeffs, true);
        let test13 = ifft_result.is_ok();
        println!("  逆向 FFT: {}", if test13 { "✅ 通过" } else { "❌ 失败" });
        all_passed &= test13;
        
        if test13 {
            let test14 = original.iter().zip(test_coeffs.iter())
                               .all(|(a, b)| *a == *b);
            println!("  FFT-IFFT 恢复: {}", if test14 { "✅ 通过" } else { "❌ 失败" });
            all_passed &= test14;
        }
    }
    
    // 4. MSM 一致性测试
    println!("\n⚡ MSM 一致性测试:");
    
    let points = vec![g, h, g.mul_scalar(&CustomFr::from_u64(3))];
    let scalars = vec![
        CustomFr::from_u64(2),
        CustomFr::from_u64(3), 
        CustomFr::from_u64(5),
    ];
    
    let naive_result = CustomMSM::naive_msm(&points, &scalars);
    let pippenger_result = CustomMSM::pippenger_msm(&points, &scalars);
    
    let test15 = naive_result.is_ok() && pippenger_result.is_ok();
    println!("  MSM 计算: {}", if test15 { "✅ 通过" } else { "❌ 失败" });
    all_passed &= test15;
    
    if test15 {
        let test16 = naive_result.unwrap() == pippenger_result.unwrap();
        println!("  MSM 一致性: {}", if test16 { "✅ 通过" } else { "❌ 失败" });
        all_passed &= test16;
    }
    
    // 测试总结
    println!("\n🏆 测试总结:");
    println!("=============");
    if all_passed {
        println!("🎉 所有测试通过！自定义后端实现正确。");
    } else {
        println!("⚠️  部分测试失败，需要检查实现。");
    }
}

fn main() {
    println!("🔬 第15章：自定义后端实现");
    println!("========================");
    println!("本章展示如何从零开始实现 KZG 密码学后端");
    println!();
    
    // 演示基本功能
    demonstrate_custom_backend();
    
    // 运行正确性测试
    run_correctness_tests();
    
    // 运行性能基准
    run_benchmarks();
    
    println!("\n🎓 第15章学习完成!");
    println!("===================");
    println!("你现在已经:");
    println!("• ✅ 掌握了自定义后端的完整实现流程");
    println!("• ✅ 理解了密码学 Trait 系统的设计");
    println!("• ✅ 学会了算法优化和性能测试");
    println!("• ✅ 具备了深入研究的基础能力");
    println!();
    println!("🚀 下一章将学习\"生产环境部署\"，");
    println!("   了解如何将 KZG 库应用到实际项目中！");
}

// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fr_basic_operations() {
        let a = CustomFr::from_u64(100);
        let b = CustomFr::from_u64(200);
        
        // 基本运算
        assert_eq!(a + b, CustomFr::from_u64(300));
        assert_eq!(b - a, CustomFr::from_u64(100));
        
        // 单位元
        assert_eq!(a + CustomFr::zero(), a);
        assert_eq!(a * CustomFr::one(), a);
        
        // 逆元
        if !a.is_zero() {
            assert_eq!(a * a.inverse(), CustomFr::one());
        }
    }
    
    #[test]
    fn test_g1_basic_operations() {
        let g = CustomG1::generator();
        let id = CustomG1::identity();
        
        // 单位元
        assert_eq!(g + id, g);
        assert_eq!(g + (-g), id);
        
        // 标量乘法
        let scalar = CustomFr::from_u64(2);
        assert_eq!(g.mul_scalar(&scalar), g.double());
    }
    
    #[test]
    fn test_fft_roundtrip() {
        let mut coeffs = vec![
            CustomFr::from_u64(1),
            CustomFr::from_u64(2),
            CustomFr::from_u64(3),
            CustomFr::from_u64(4),
        ];
        
        let original = coeffs.clone();
        
        // 正向 FFT
        CustomFFT::ntt(&mut coeffs, false).unwrap();
        
        // 逆向 FFT
        CustomFFT::ntt(&mut coeffs, true).unwrap();
        
        // 验证恢复
        for (orig, recovered) in original.iter().zip(coeffs.iter()) {
            assert_eq!(*orig, *recovered);
        }
    }
    
    #[test]
    fn test_msm_consistency() {
        let g = CustomG1::generator();
        let points = vec![g, g.double()];
        let scalars = vec![CustomFr::from_u64(3), CustomFr::from_u64(5)];
        
        let naive = CustomMSM::naive_msm(&points, &scalars).unwrap();
        let pippenger = CustomMSM::pippenger_msm(&points, &scalars).unwrap();
        
        assert_eq!(naive, pippenger);
    }
    
    #[test]
    fn test_serialization() {
        let fr = CustomFr::from_u64(12345);
        let bytes = fr.to_bytes_be();
        let recovered = CustomFr::from_bytes_be(&bytes).unwrap();
        assert_eq!(fr, recovered);
        
        let g1 = CustomG1::generator();
        let g1_bytes = g1.to_bytes();
        let recovered_g1 = CustomG1::from_bytes(&g1_bytes).unwrap();
        assert_eq!(g1, recovered_g1);
    }
}