//! 第14章：安全性分析与加固技术 - 代码示例
//! 
//! 本文件演示 KZG 库的安全性分析与加固相关技术：
//! 1. 侧信道攻击防护与常量时间操作
//! 2. 内存安全管理与敏感数据清除
//! 3. 受信任设置验证与完整性检查
//! 4. 模糊测试基础框架
//! 5. 生产环境安全配置实践
//!
//! 重点关注密码学实现中的安全风险识别和防护措施。

use std::time::{Duration, Instant};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

/// 安全配置结构体
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub enable_timing_protection: bool,
    pub enable_memory_protection: bool,
    pub trusted_setup_hash: Vec<u8>,
    pub max_operation_time: Duration,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_timing_protection: true,
            enable_memory_protection: true,
            trusted_setup_hash: vec![],
            max_operation_time: Duration::from_millis(1000),
        }
    }
}

/// 侧信道防护：常量时间比较
/// 
/// 防止时序攻击的关键函数，确保比较操作的执行时间
/// 不依赖于输入数据的内容，避免通过时间分析推断敏感信息。
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut res = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        res |= x ^ y;
    }
    
    // 确保所有字节都被处理，不会因为提前退出而泄露信息
    res == 0
}

/// 时序攻击检测：分析函数执行时间分布
pub fn timing_analysis_detector<F>(func: F, inputs: &[Vec<u8>]) -> HashMap<String, Duration>
where
    F: Fn(&[u8]) -> bool,
{
    let mut timings = HashMap::new();
    
    for input in inputs {
        let start = Instant::now();
        let _ = func(input);
        let duration = start.elapsed();
        
        let key = format!("len_{}", input.len());
        let entry = timings.entry(key).or_insert(Duration::from_nanos(0));
        *entry = (*entry + duration) / 2; // 简单平均
    }
    
    timings
}

/// 内存安全：敏感数据零化
/// 
/// 确保敏感数据在使用后立即从内存中清除，防止内存转储攻击。
/// 使用 volatile 操作确保编译器不会优化掉清零操作。
pub fn zeroize_secret(secret: &mut [u8]) {
    // 使用 volatile 写入确保不被编译器优化
    for x in secret.iter_mut() {
        unsafe {
            std::ptr::write_volatile(x as *mut u8, 0);
        }
    }
    
    // 内存屏障确保清零操作完成
    std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
}

/// 安全内存分配器（模拟）
pub struct SecureMemoryPool {
    allocated: HashMap<usize, Vec<u8>>,
    next_id: usize,
}

impl Default for SecureMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureMemoryPool {
    pub fn new() -> Self {
        Self {
            allocated: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// 分配安全内存区域
    pub fn allocate(&mut self, size: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let buffer = vec![0u8; size];
        // 在实际实现中，这里应该使用 mlock() 锁定内存页
        self.allocated.insert(id, buffer);
        id
    }
    
    /// 安全释放内存
    pub fn deallocate(&mut self, id: usize) {
        if let Some(mut buffer) = self.allocated.remove(&id) {
            zeroize_secret(&mut buffer);
            // 在实际实现中，这里应该调用 munlock() 解锁内存页
        }
    }
}

/// 受信任设置验证
/// 
/// 验证受信任设置的完整性和真实性，确保没有被篡改。
/// 在生产环境中，这是关键的安全检查步骤。
pub fn verify_trusted_setup(setup: &[u8], expected_hash: &[u8]) -> Result<bool, String> {
    if setup.is_empty() {
        return Err("Empty trusted setup".to_string());
    }
    
    // 计算设置文件的 SHA-256 哈希
    let mut hasher = Sha256::new();
    hasher.update(setup);
    let computed_hash = hasher.finalize();
    
    // 使用常量时间比较防止侧信道攻击
    if !constant_time_eq(&computed_hash, expected_hash) {
        return Err("Trusted setup hash mismatch".to_string());
    }
    
    // 验证设置文件的结构完整性（简化版）
    if setup.len() < 32 {
        return Err("Trusted setup too small".to_string());
    }
    
    // 检查魔数（示例）
    if setup[0] != 0x42 {
        return Err("Invalid trusted setup magic number".to_string());
    }
    
    Ok(true)
}

/// 多方验证协议（简化版）
pub struct MultiPartyVerifier {
    signatures: Vec<Vec<u8>>,
    public_keys: Vec<Vec<u8>>,
    threshold: usize,
}

impl MultiPartyVerifier {
    pub fn new(threshold: usize) -> Self {
        Self {
            signatures: Vec::new(),
            public_keys: Vec::new(),
            threshold,
        }
    }
    
    pub fn add_signature(&mut self, signature: Vec<u8>, public_key: Vec<u8>) {
        self.signatures.push(signature);
        self.public_keys.push(public_key);
    }
    
    /// 验证是否达到阈值签名要求
    pub fn verify_threshold(&self) -> bool {
        // 在真实实现中，这里应该验证每个签名的有效性
        self.signatures.len() >= self.threshold
    }
}

/// 模糊测试框架
pub struct FuzzTestSuite {
    test_cases: Vec<Vec<u8>>,
    crash_count: usize,
    timeout_count: usize,
}

impl Default for FuzzTestSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzTestSuite {
    pub fn new() -> Self {
        Self {
            test_cases: Vec::new(),
            crash_count: 0,
            timeout_count: 0,
        }
    }
    
    /// 生成随机测试用例
    pub fn generate_test_case(&mut self, size: usize) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        size.hash(&mut hasher);
        let seed = hasher.finish();
        
        let mut test_case = Vec::with_capacity(size);
        let mut rng_state = seed;
        
        for _ in 0..size {
            // 简单的线性同余发生器
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            test_case.push((rng_state >> 8) as u8);
        }
        
        self.test_cases.push(test_case);
    }
    
    /// 执行模糊测试
    pub fn run_fuzz_test<F>(&mut self, target_func: F, timeout: Duration) -> FuzzResult
    where
        F: Fn(&[u8]) -> Result<(), String>,
    {
        let mut results = FuzzResult::new();
        
        for test_case in &self.test_cases {
            let start = Instant::now();
            
            match target_func(test_case) {
                Ok(_) => results.passed += 1,
                Err(e) => {
                    results.failed += 1;
                    results.errors.push(format!("Input len {}: {}", test_case.len(), e));
                }
            }
            
            if start.elapsed() > timeout {
                self.timeout_count += 1;
                results.timeouts += 1;
            }
        }
        
        results
    }

    /// 报告模糊测试结果
    /// Report fuzzing test results
    pub fn report_results(&self) {
        println!("\n📊 模糊测试结果 / Fuzz Test Results");
        println!("  测试用例总数 / Total test cases: {}", self.test_cases.len());
        println!("  检测到的崩溃 / Crashes detected: {}", self.crash_count);
        println!("  超时次数 / Timeout count: {}", self.timeout_count);

        if self.crash_count == 0 && self.timeout_count == 0 {
            println!("✅ 未发现安全问题 / No security issues found");
        } else {
            println!("⚠️  发现潜在安全问题，需要进一步分析");
            println!("   Potential security issues found, further analysis needed");
        }
    }
}

/// 模糊测试结果
#[derive(Debug)]
pub struct FuzzResult {
    pub passed: usize,
    pub failed: usize,
    pub timeouts: usize,
    pub errors: Vec<String>,
}

impl FuzzResult {
    pub fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            timeouts: 0,
            errors: Vec::new(),
        }
    }
}

/// 模糊测试入口示例
/// Fuzz Test Entry Point Example
///
/// 注意：这是一个教学示例，展示如何构建模糊测试基础设施
/// Note: This is an educational example showing how to build fuzzing infrastructure
///
/// 在实际项目中，推荐使用以下工具：
/// In real projects, use these tools instead:
///   - cargo-fuzz: https://github.com/rust-fuzz/cargo-fuzz
///   - AFL (American Fuzzy Lop): https://github.com/AFLplusplus/AFLplusplus
///   - libFuzzer: https://llvm.org/docs/LibFuzzer.html
///
/// 使用方法 / Usage:
///   cargo run --example chapter14_security_analysis_hardening
///
#[allow(dead_code)]
pub fn fuzz_target(data: &[u8]) {
    // 测试常量时间比较函数
    let reference = [0u8; 32];
    let _ = constant_time_eq(data, &reference);
    
    // 测试受信任设置验证
    let expected_hash = [0u8; 32];
    let _ = verify_trusted_setup(data, &expected_hash);
    
    // 测试内存清零功能
    if !data.is_empty() {
        let mut mutable_data = data.to_vec();
        zeroize_secret(&mut mutable_data);
    }
}

/// 生产环境安全配置示例
pub fn setup_production_security() -> SecurityConfig {
    let mut config = SecurityConfig::default();
    
    // 启用所有安全保护
    config.enable_timing_protection = true;
    config.enable_memory_protection = true;
    
    // 设置受信任设置的预期哈希（示例）
    let mut hasher = Sha256::new();
    hasher.update(b"trusted_setup_v1.0");
    config.trusted_setup_hash = hasher.finalize().to_vec();
    
    // 设置操作超时限制
    config.max_operation_time = Duration::from_millis(500);
    
    config
}

/// 安全审计工具
pub struct SecurityAuditor {
    config: SecurityConfig,
    violations: Vec<String>,
}

impl SecurityAuditor {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            violations: Vec::new(),
        }
    }
    
    /// 审计时序安全性
    pub fn audit_timing_safety<F>(&mut self, name: &str, func: F) 
    where
        F: Fn() -> Duration,
    {
        let execution_time = func();
        
        if execution_time > self.config.max_operation_time {
            self.violations.push(format!(
                "Timing violation in {}: {:?} > {:?}", 
                name, execution_time, self.config.max_operation_time
            ));
        }
    }
    
    /// 获取违规报告
    pub fn get_violations(&self) -> &[String] {
        &self.violations
    }
    
    /// 生成安全报告
    pub fn generate_report(&self) -> String {
        if self.violations.is_empty() {
            "✅ 所有安全检查通过".to_string()
        } else {
            format!("⚠️  发现 {} 个安全违规:\n{}", 
                    self.violations.len(),
                    self.violations.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        let a = [1u8; 32];
        let b = [1u8; 32];
        let c = [2u8; 32];
        assert!(constant_time_eq(&a, &b));
        assert!(!constant_time_eq(&a, &c));
        
        // 测试不同长度
        let short = [1u8; 16];
        assert!(!constant_time_eq(&a, &short));
    }

    #[test]
    fn test_zeroize_secret() {
        let mut secret = [42u8; 16];
        zeroize_secret(&mut secret);
        assert!(secret.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_verify_trusted_setup() {
        let setup_data = [0x42, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                         0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
                         0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
                         0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
                         0x20]; // 33 bytes, > 32
        
        // 计算正确的哈希
        let mut hasher = Sha256::new();
        hasher.update(&setup_data);
        let expected_hash = hasher.finalize();
        
        assert!(verify_trusted_setup(&setup_data, &expected_hash).is_ok());
        
        // 测试错误的哈希
        let wrong_hash = [0u8; 32];
        assert!(verify_trusted_setup(&setup_data, &wrong_hash).is_err());
        
        // 测试空数据
        assert!(verify_trusted_setup(&[], &expected_hash).is_err());
    }

    #[test]
    fn test_secure_memory_pool() {
        let mut pool = SecureMemoryPool::new();
        
        let id1 = pool.allocate(64);
        let id2 = pool.allocate(128);
        
        assert_ne!(id1, id2);
        
        pool.deallocate(id1);
        pool.deallocate(id2);
        
        // 验证内存池清空后的状态
        assert!(pool.allocated.is_empty());
    }

    #[test]
    fn test_multi_party_verifier() {
        let mut verifier = MultiPartyVerifier::new(3);
        
        verifier.add_signature(vec![1, 2, 3], vec![4, 5, 6]);
        verifier.add_signature(vec![7, 8, 9], vec![10, 11, 12]);
        
        assert!(!verifier.verify_threshold());
        
        verifier.add_signature(vec![13, 14, 15], vec![16, 17, 18]);
        
        assert!(verifier.verify_threshold());
    }

    #[test]
    fn test_fuzz_test_suite() {
        let mut suite = FuzzTestSuite::new();
        
        suite.generate_test_case(10);
        suite.generate_test_case(20);
        suite.generate_test_case(50);
        
        let test_func = |data: &[u8]| -> Result<(), String> {
            if data.len() > 30 {
                Err("Too large".to_string())
            } else {
                Ok(())
            }
        };
        
        let result = suite.run_fuzz_test(test_func, Duration::from_millis(100));
        
        assert_eq!(result.passed + result.failed, 3);
        assert!(result.failed > 0); // 至少一个大于30字节的测试用例失败
    }

    #[test]
    fn test_timing_analysis_detector() {
        let inputs = vec![
            vec![1u8; 10],
            vec![2u8; 20],
            vec![3u8; 10],
        ];
        
        let timings = timing_analysis_detector(|data| data[0] == 1, &inputs);
        
        assert!(timings.contains_key("len_10"));
        assert!(timings.contains_key("len_20"));
    }

    #[test]
    fn test_security_auditor() {
        let config = setup_production_security();
        let mut auditor = SecurityAuditor::new(config);
        
        // 模拟一个快速操作
        auditor.audit_timing_safety("fast_op", || Duration::from_millis(100));
        
        // 模拟一个慢操作
        auditor.audit_timing_safety("slow_op", || Duration::from_millis(1000));
        
        let violations = auditor.get_violations();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("slow_op"));
        
        let report = auditor.generate_report();
        assert!(report.contains("发现 1 个安全违规"));
    }
}

/// 主函数：演示安全性分析与加固技术
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 第14章：安全性分析与加固技术演示");
    println!("{}", "=".repeat(60));
    
    // 1. 演示常量时间比较
    println!("\n1. 常量时间比较演示");
    let data1 = [1u8; 32];
    let data2 = [1u8; 32];
    let data3 = [2u8; 32];
    
    println!("   data1 == data2: {}", constant_time_eq(&data1, &data2));
    println!("   data1 == data3: {}", constant_time_eq(&data1, &data3));
    
    // 2. 演示时序分析
    println!("\n2. 时序分析演示");
    let inputs = vec![
        vec![1u8; 10],
        vec![2u8; 20],
        vec![3u8; 10],
        vec![4u8; 30],
    ];
    
    let timings = timing_analysis_detector(|data| {
        // 模拟一个可能有时序泄露的函数
        std::thread::sleep(std::time::Duration::from_micros(data.len() as u64 * 10));
        data[0] == 1
    }, &inputs);
    
    for (key, timing) in timings {
        println!("   {}: {:?}", key, timing);
    }
    
    // 3. 演示安全内存管理
    println!("\n3. 安全内存管理演示");
    let mut pool = SecureMemoryPool::new();
    let id1 = pool.allocate(64);
    let id2 = pool.allocate(128);
    
    println!("   分配内存块 ID: {}, {}", id1, id2);
    
    pool.deallocate(id1);
    pool.deallocate(id2);
    println!("   内存块已安全释放");
    
    // 4. 演示受信任设置验证
    println!("\n4. 受信任设置验证演示");
    let setup_data = [0x42u8; 64]; // 64字节的测试数据
    
    let mut hasher = Sha256::new();
    hasher.update(&setup_data);
    let expected_hash = hasher.finalize();
    
    match verify_trusted_setup(&setup_data, &expected_hash) {
        Ok(true) => println!("   ✅ 受信任设置验证通过"),
        Ok(false) => println!("   ❌ 受信任设置验证失败"),
        Err(e) => println!("   ⚠️  验证错误: {}", e),
    }
    
    // 5. 演示模糊测试
    println!("\n5. 模糊测试演示");
    let mut fuzz_suite = FuzzTestSuite::new();
    
    // 生成测试用例
    fuzz_suite.generate_test_case(10);
    fuzz_suite.generate_test_case(20);
    fuzz_suite.generate_test_case(50);
    
    let test_func = |data: &[u8]| -> Result<(), String> {
        if data.is_empty() {
            return Err("空数据".to_string());
        }
        if data.len() > 30 {
            return Err("数据过大".to_string());
        }
        Ok(())
    };
    
    let result = fuzz_suite.run_fuzz_test(test_func, Duration::from_millis(100));
    println!("   测试结果: 通过 {}, 失败 {}, 超时 {}", 
             result.passed, result.failed, result.timeouts);
    
    if !result.errors.is_empty() {
        println!("   错误详情:");
        for error in &result.errors {
            println!("   - {}", error);
        }
    }
    
    // 6. 演示安全配置
    println!("\n6. 生产环境安全配置演示");
    let config = setup_production_security();
    println!("   安全配置:");
    println!("   - 时序保护: {}", config.enable_timing_protection);
    println!("   - 内存保护: {}", config.enable_memory_protection);
    println!("   - 最大操作时间: {:?}", config.max_operation_time);
    
    // 7. 演示安全审计
    println!("\n7. 安全审计演示");
    let mut auditor = SecurityAuditor::new(config);
    
    // 模拟快速操作
    auditor.audit_timing_safety("快速操作", || Duration::from_millis(100));
    
    // 模拟慢操作
    auditor.audit_timing_safety("慢操作", || Duration::from_millis(1000));
    
    let report = auditor.generate_report();
    println!("   {}", report);
    
    println!("\n🎉 第14章演示完成！");
    println!("\n💡 关键要点:");
    println!("   • 常量时间操作防止时序攻击");
    println!("   • 安全内存管理避免敏感数据泄露");
    println!("   • 受信任设置验证确保系统完整性");
    println!("   • 模糊测试发现潜在安全漏洞");
    println!("   • 自动化安全审计提供持续保障");
    
    Ok(())
}
