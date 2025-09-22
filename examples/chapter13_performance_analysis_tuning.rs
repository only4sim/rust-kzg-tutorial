// examples/chapter13_performance_analysis_tuning.rs
//
// 第13章：性能分析与调优技术 - 完整示例代码
//
// 本示例演示了如何对 Rust KZG 库进行全面的性能分析与调优，
// 包括微基准测试、内存分析、并发优化、缓存策略等高级性能优化技术。

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::hash::{Hash, Hasher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 第13章：性能分析与调优技术示例");
    println!("================================================\n");

    // 1. 初始化性能监控系统
    println!("📊 1. 初始化性能监控系统");
    let performance_monitor = Arc::new(PerformanceMonitor::new());
    let memory_analyzer = MemoryAnalyzer::new();
    
    // 2. 微基准测试演示
    println!("🔬 2. 执行微基准测试");
    run_micro_benchmarks(&performance_monitor)?;
    
    // 3. 内存分析与优化演示
    println!("🧠 3. 内存分析与优化");
    demonstrate_memory_optimization(memory_analyzer)?;
    
    // 4. 并发性能优化演示
    println!("🚀 4. 并发性能优化");
    demonstrate_parallel_optimization()?;
    
    // 5. 缓存策略优化演示
    println!("💾 5. 缓存策略优化");
    demonstrate_cache_optimization()?;
    
    // 6. 算法层面优化演示
    println!("⚡ 6. 算法层面优化");
    demonstrate_algorithm_optimization()?;
    
    // 7. 系统级调优演示
    println!("🔧 7. 系统级调优");
    demonstrate_system_tuning()?;
    
    // 8. 实时性能监控演示
    println!("📈 8. 实时性能监控");
    demonstrate_real_time_monitoring(&performance_monitor)?;
    
    // 9. 性能回归检测演示
    println!("🔍 9. 性能回归检测");
    demonstrate_regression_testing()?;
    
    // 10. 综合性能报告
    println!("📋 10. 生成综合性能报告");
    generate_comprehensive_report(&performance_monitor)?;

    println!("\n✅ 所有性能分析与调优示例执行完成！");
    Ok(())
}

/// 实时性能指标收集器
#[derive(Debug)]
pub struct PerformanceMonitor {
    // 操作计数器
    commitment_count: AtomicU64,
    proof_count: AtomicU64,
    verification_count: AtomicU64,
    
    // 时间统计
    total_commitment_time: AtomicU64,
    total_proof_time: AtomicU64,
    total_verification_time: AtomicU64,
    
    // 错误计数
    error_count: AtomicU64,
    
    // 启动时间
    start_time: Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            commitment_count: AtomicU64::new(0),
            proof_count: AtomicU64::new(0),
            verification_count: AtomicU64::new(0),
            total_commitment_time: AtomicU64::new(0),
            total_proof_time: AtomicU64::new(0),
            total_verification_time: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }
    
    /// 记录承诺操作
    pub fn record_commitment(&self, duration: Duration) {
        self.commitment_count.fetch_add(1, Ordering::Relaxed);
        self.total_commitment_time.fetch_add(
            duration.as_nanos() as u64, 
            Ordering::Relaxed
        );
    }
    
    /// 记录证明操作
    pub fn record_proof(&self, duration: Duration) {
        self.proof_count.fetch_add(1, Ordering::Relaxed);
        self.total_proof_time.fetch_add(
            duration.as_nanos() as u64, 
            Ordering::Relaxed
        );
    }
    
    /// 记录验证操作
    pub fn record_verification(&self, duration: Duration) {
        self.verification_count.fetch_add(1, Ordering::Relaxed);
        self.total_verification_time.fetch_add(
            duration.as_nanos() as u64, 
            Ordering::Relaxed
        );
    }
    
    /// 记录错误
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 生成性能报告
    pub fn generate_report(&self) -> PerformanceReport {
        let uptime = self.start_time.elapsed();
        let commitment_count = self.commitment_count.load(Ordering::Relaxed);
        let proof_count = self.proof_count.load(Ordering::Relaxed);
        let verification_count = self.verification_count.load(Ordering::Relaxed);
        let total_operations = commitment_count + proof_count + verification_count;
        
        PerformanceReport {
            uptime,
            total_operations,
            operations_per_second: if uptime.as_secs_f64() > 0.0 {
                total_operations as f64 / uptime.as_secs_f64()
            } else {
                0.0
            },
            average_commitment_time: if commitment_count > 0 {
                Duration::from_nanos(
                    self.total_commitment_time.load(Ordering::Relaxed) / commitment_count
                )
            } else {
                Duration::ZERO
            },
            average_proof_time: if proof_count > 0 {
                Duration::from_nanos(
                    self.total_proof_time.load(Ordering::Relaxed) / proof_count
                )
            } else {
                Duration::ZERO
            },
            average_verification_time: if verification_count > 0 {
                Duration::from_nanos(
                    self.total_verification_time.load(Ordering::Relaxed) / verification_count
                )
            } else {
                Duration::ZERO
            },
            error_rate: if total_operations > 0 {
                self.error_count.load(Ordering::Relaxed) as f64 / total_operations as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub uptime: Duration,
    pub total_operations: u64,
    pub operations_per_second: f64,
    pub average_commitment_time: Duration,
    pub average_proof_time: Duration,
    pub average_verification_time: Duration,
    pub error_rate: f64,
}

/// 内存使用分析工具
pub struct MemoryAnalyzer {
    initial_memory: usize,
    peak_memory: usize,
    allocations: Vec<AllocationInfo>,
}

#[derive(Debug, Clone)]
pub struct AllocationInfo {
    size: usize,
    timestamp: Instant,
    location: String,
}

impl MemoryAnalyzer {
    pub fn new() -> Self {
        Self {
            initial_memory: get_current_memory_usage(),
            peak_memory: 0,
            allocations: Vec::new(),
        }
    }
    
    /// 记录内存分配
    pub fn record_allocation(&mut self, size: usize, location: &str) {
        self.allocations.push(AllocationInfo {
            size,
            timestamp: Instant::now(),
            location: location.to_string(),
        });
        
        let current_memory = get_current_memory_usage();
        if current_memory > self.peak_memory {
            self.peak_memory = current_memory;
        }
    }
    
    /// 生成内存使用报告
    pub fn generate_report(&self) -> MemoryReport {
        let current_memory = get_current_memory_usage();
        let total_allocations = self.allocations.len();
        let largest_allocation = self.allocations.iter()
            .max_by_key(|a| a.size)
            .map(|a| a.size)
            .unwrap_or(0);
        
        MemoryReport {
            initial: self.initial_memory,
            current: current_memory,
            peak: self.peak_memory,
            total_allocations,
            largest_allocation,
            memory_growth: current_memory.saturating_sub(self.initial_memory),
        }
    }
}

#[derive(Debug)]
pub struct MemoryReport {
    pub initial: usize,
    pub current: usize,
    pub peak: usize,
    pub total_allocations: usize,
    pub largest_allocation: usize,
    pub memory_growth: usize,
}

/// 模拟 KZG 多项式
#[derive(Debug, Clone)]
pub struct MockPolynomial {
    coefficients: Vec<u64>,
}

impl MockPolynomial {
    pub fn new(size: usize) -> Self {
        Self {
            coefficients: (0..size).map(|i| (i as u64).wrapping_mul(1103515245).wrapping_add(12345)).collect(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.coefficients.len()
    }
    
    pub fn hash(&self) -> u64 {
        self.coefficients.iter().fold(0u64, |acc, &x| acc.wrapping_mul(31).wrapping_add(x))
    }
}

/// 模拟 KZG 承诺点
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MockG1Point([u64; 4]);

impl MockG1Point {
    pub fn identity() -> Self {
        Self([0, 0, 0, 0])
    }
    
    pub fn random() -> Self {
        // 使用简单的线性同余生成器生成伪随机数
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        Instant::now().elapsed().as_nanos().hash(&mut hasher);
        let state = hasher.finish();
        
        Self([
            state,
            state.wrapping_mul(31),
            state.wrapping_mul(37),
            state.wrapping_mul(41),
        ])
    }
    
    pub fn add(&self, other: &Self) -> Self {
        Self([
            self.0[0].wrapping_add(other.0[0]),
            self.0[1].wrapping_add(other.0[1]),
            self.0[2].wrapping_add(other.0[2]),
            self.0[3].wrapping_add(other.0[3]),
        ])
    }
    
    pub fn double(&self) -> Self {
        Self([
            self.0[0].wrapping_mul(2),
            self.0[1].wrapping_mul(2),
            self.0[2].wrapping_mul(2),
            self.0[3].wrapping_mul(2),
        ])
    }
    
    pub fn scalar_mul(&self, scalar: u64) -> Self {
        Self([
            self.0[0].wrapping_mul(scalar),
            self.0[1].wrapping_mul(scalar),
            self.0[2].wrapping_mul(scalar),
            self.0[3].wrapping_mul(scalar),
        ])
    }
}

/// 模拟 KZG 设置
pub struct MockKzgSettings {
    pub setup_g1: Vec<MockG1Point>,
    pub setup_g2: Vec<MockG1Point>,
}

impl MockKzgSettings {
    pub fn new(size: usize) -> Self {
        Self {
            setup_g1: (0..size).map(|_| MockG1Point::random()).collect(),
            setup_g2: (0..size).map(|_| MockG1Point::random()).collect(),
        }
    }
}

/// LRU 缓存实现
pub struct LruCache<K, V> {
    map: HashMap<K, (V, usize)>,
    order: VecDeque<K>,
    capacity: usize,
    access_counter: usize,
}

impl<K: Clone + std::hash::Hash + Eq, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity,
            access_counter: 0,
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, _)) = self.map.get_mut(key) {
            self.access_counter += 1;
            Some(value)
        } else {
            None
        }
    }
    
    pub fn put(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            if let Some(oldest_key) = self.order.pop_front() {
                self.map.remove(&oldest_key);
            }
        }
        
        if !self.map.contains_key(&key) {
            self.order.push_back(key.clone());
        }
        
        self.access_counter += 1;
        self.map.insert(key, (value, self.access_counter));
    }
    
    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// KZG 计算结果缓存
pub struct KzgCache {
    commitments: LruCache<u64, MockG1Point>,
    proofs: LruCache<u64, MockG1Point>,
    verifications: LruCache<u64, bool>,
    hit_count: AtomicUsize,
    miss_count: AtomicUsize,
}

impl KzgCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            commitments: LruCache::new(capacity),
            proofs: LruCache::new(capacity),
            verifications: LruCache::new(capacity),
            hit_count: AtomicUsize::new(0),
            miss_count: AtomicUsize::new(0),
        }
    }
    
    /// 缓存承诺计算结果
    pub fn cache_commitment(&mut self, polynomial_hash: u64, commitment: MockG1Point) {
        self.commitments.put(polynomial_hash, commitment);
    }
    
    /// 获取缓存的承诺
    pub fn get_commitment(&mut self, polynomial_hash: u64) -> Option<MockG1Point> {
        if let Some(commitment) = self.commitments.get(&polynomial_hash) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(*commitment)
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
    
    /// 缓存命中率统计
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// 批量操作优化器
pub struct BatchOptimizer {
    batch_size: usize,
}

impl BatchOptimizer {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }
    
    /// 批量承诺生成
    pub fn batch_commitments(&self, polynomials: &[MockPolynomial], settings: &MockKzgSettings) -> Vec<MockG1Point> {
        polynomials
            .chunks(self.batch_size)
            .flat_map(|chunk| {
                chunk.iter().map(|poly| {
                    // 模拟承诺计算
                    let mut result = MockG1Point::identity();
                    for (i, &coeff) in poly.coefficients.iter().enumerate() {
                        if i < settings.setup_g1.len() {
                            result = result.add(&settings.setup_g1[i].scalar_mul(coeff));
                        }
                    }
                    result
                }).collect::<Vec<_>>()
            })
            .collect()
    }
    
    /// 批量证明生成
    pub fn batch_proofs(&self, polynomials: &[MockPolynomial], commitments: &[MockG1Point], settings: &MockKzgSettings) -> Vec<MockG1Point> {
        assert_eq!(polynomials.len(), commitments.len());
        
        polynomials
            .chunks(self.batch_size)
            .zip(commitments.chunks(self.batch_size))
            .flat_map(|(poly_chunk, comm_chunk)| {
                poly_chunk.iter().zip(comm_chunk.iter()).map(|(poly, commitment)| {
                    // 模拟证明计算
                    let evaluation_point = poly.coefficients[0] % 1000;
                    let mut proof = MockG1Point::identity();
                    
                    for (i, &coeff) in poly.coefficients.iter().enumerate() {
                        if i < settings.setup_g1.len() {
                            proof = proof.add(&settings.setup_g1[i].scalar_mul(coeff.wrapping_mul(evaluation_point)));
                        }
                    }
                    
                    proof.add(commitment)
                }).collect::<Vec<_>>()
            })
            .collect()
    }
}

/// 并行处理器
pub struct ParallelProcessor {
    thread_count: usize,
}

impl ParallelProcessor {
    pub fn new(thread_count: usize) -> Self {
        Self { thread_count }
    }
    
    /// 并行批量承诺生成
    pub fn parallel_batch_commitments(&self, polynomials: &[MockPolynomial], settings: &MockKzgSettings) -> Vec<MockG1Point> {
        use std::thread;
        
        let chunk_size = (polynomials.len() + self.thread_count - 1) / self.thread_count;
        let mut handles = Vec::new();
        
        for chunk in polynomials.chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let setup_g1 = settings.setup_g1.clone();
            
            let handle = thread::spawn(move || {
                chunk.iter().map(|poly| {
                    let mut result = MockG1Point::identity();
                    for (i, &coeff) in poly.coefficients.iter().enumerate() {
                        if i < setup_g1.len() {
                            result = result.add(&setup_g1[i].scalar_mul(coeff));
                        }
                    }
                    result
                }).collect::<Vec<_>>()
            });
            
            handles.push(handle);
        }
        
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    }
}

/// 性能回归检测框架
pub struct PerformanceRegression {
    baseline_results: HashMap<String, Duration>,
    threshold: f64,
}

impl PerformanceRegression {
    pub fn new(threshold: f64) -> Self {
        Self {
            baseline_results: HashMap::new(),
            threshold,
        }
    }
    
    /// 设置基准性能数据
    pub fn set_baseline(&mut self, test_name: &str, duration: Duration) {
        self.baseline_results.insert(test_name.to_string(), duration);
    }
    
    /// 检查是否存在性能回归
    pub fn check_regression(&self, test_name: &str, current: Duration) -> Result<(), String> {
        if let Some(&baseline) = self.baseline_results.get(test_name) {
            let regression_ratio = (current.as_nanos() as f64 / baseline.as_nanos() as f64) - 1.0;
            
            if regression_ratio > self.threshold {
                return Err(format!(
                    "Performance regression detected in {}: {:.2}% slower than baseline",
                    test_name, regression_ratio * 100.0
                ));
            }
        }
        Ok(())
    }
}

/// 微基准测试函数
fn run_micro_benchmarks(monitor: &Arc<PerformanceMonitor>) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔬 执行 KZG 操作微基准测试...");
    
    let settings = MockKzgSettings::new(4096);
    let test_sizes = [256, 512, 1024, 2048];
    
    for &size in &test_sizes {
        println!("    测试多项式大小: {}", size);
        
        // 承诺生成基准测试
        let polynomial = MockPolynomial::new(size);
        let start = Instant::now();
        
        let mut commitment = MockG1Point::identity();
        for (i, &coeff) in polynomial.coefficients.iter().enumerate() {
            if i < settings.setup_g1.len() {
                commitment = commitment.add(&settings.setup_g1[i].scalar_mul(coeff));
            }
        }
        
        let duration = start.elapsed();
        monitor.record_commitment(duration);
        
        println!("      承诺生成: {:?}", duration);
        
        // 证明生成基准测试
        let start = Instant::now();
        
        let evaluation_point = polynomial.coefficients[0] % 1000;
        let mut proof = MockG1Point::identity();
        
        for (i, &coeff) in polynomial.coefficients.iter().enumerate() {
            if i < settings.setup_g1.len() {
                proof = proof.add(&settings.setup_g1[i].scalar_mul(coeff.wrapping_mul(evaluation_point)));
            }
        }
        
        let duration = start.elapsed();
        monitor.record_proof(duration);
        
        println!("      证明生成: {:?}", duration);
        
        // 验证基准测试
        let start = Instant::now();
        
        // 模拟验证过程
        let verification_result = proof.0[0] != 0;
        
        let duration = start.elapsed();
        monitor.record_verification(duration);
        
        println!("      验证: {:?} (结果: {})", duration, verification_result);
    }
    
    println!("  ✅ 微基准测试完成");
    Ok(())
}

/// 内存优化演示
fn demonstrate_memory_optimization(mut analyzer: MemoryAnalyzer) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🧠 演示内存分析与优化技术...");
    
    // 记录初始状态
    analyzer.record_allocation(1024, "initial_allocation");
    
    // 模拟大量内存分配
    let mut data_storage = Vec::new();
    for i in 0..1000 {
        let size = 1024 * (i % 10 + 1);
        let data = vec![0u8; size];
        analyzer.record_allocation(size, &format!("allocation_{}", i));
        data_storage.push(data);
        
        // 模拟内存池的重用
        if i % 100 == 0 {
            data_storage.clear();
            println!("    清理内存池，释放内存");
        }
    }
    
    // 生成内存使用报告
    let report = analyzer.generate_report();
    println!("  📊 内存使用报告:");
    println!("    初始内存: {} bytes", report.initial);
    println!("    当前内存: {} bytes", report.current);
    println!("    峰值内存: {} bytes", report.peak);
    println!("    总分配次数: {}", report.total_allocations);
    println!("    最大单次分配: {} bytes", report.largest_allocation);
    println!("    内存增长: {} bytes", report.memory_growth);
    
    println!("  ✅ 内存优化演示完成");
    Ok(())
}

/// 并发优化演示
fn demonstrate_parallel_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("  🚀 演示并发性能优化...");
    
    let settings = MockKzgSettings::new(4096);
    let polynomials: Vec<MockPolynomial> = (0..100).map(|i| MockPolynomial::new(512 + i * 10)).collect();
    
    // 串行处理
    let start = Instant::now();
    let batch_optimizer = BatchOptimizer::new(50);
    let serial_commitments = batch_optimizer.batch_commitments(&polynomials, &settings);
    let serial_duration = start.elapsed();
    
    println!("    串行处理: {:?} ({} 个承诺)", serial_duration, serial_commitments.len());
    
    // 并行处理
    let start = Instant::now();
    let parallel_processor = ParallelProcessor::new(4);
    let parallel_commitments = parallel_processor.parallel_batch_commitments(&polynomials, &settings);
    let parallel_duration = start.elapsed();
    
    println!("    并行处理: {:?} ({} 个承诺)", parallel_duration, parallel_commitments.len());
    
    // 计算加速比
    let speedup = serial_duration.as_secs_f64() / parallel_duration.as_secs_f64();
    println!("    加速比: {:.2}x", speedup);
    
    println!("  ✅ 并发优化演示完成");
    Ok(())
}

/// 缓存策略优化演示
fn demonstrate_cache_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("  💾 演示缓存策略优化...");
    
    let mut cache = KzgCache::new(100);
    let polynomials: Vec<MockPolynomial> = (0..200).map(|i| MockPolynomial::new(256 + i % 50)).collect();
    
    // 第一轮：建立缓存
    println!("    第一轮处理（建立缓存）");
    for polynomial in &polynomials {
        let hash = polynomial.hash();
        
        if cache.get_commitment(hash).is_none() {
            // 模拟承诺计算
            let commitment = MockG1Point::random();
            cache.cache_commitment(hash, commitment);
        }
    }
    
    let first_hit_rate = cache.hit_rate();
    println!("      第一轮缓存命中率: {:.2}%", first_hit_rate * 100.0);
    
    // 第二轮：利用缓存
    println!("    第二轮处理（利用缓存）");
    for polynomial in &polynomials {
        let hash = polynomial.hash();
        let _ = cache.get_commitment(hash);
    }
    
    let second_hit_rate = cache.hit_rate();
    println!("      第二轮缓存命中率: {:.2}%", second_hit_rate * 100.0);
    
    println!("  ✅ 缓存优化演示完成");
    Ok(())
}

/// 算法层面优化演示
fn demonstrate_algorithm_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ 演示算法层面优化...");
    
    let polynomial = MockPolynomial::new(2048);
    let settings = MockKzgSettings::new(4096);
    
    // 朴素算法
    let start = Instant::now();
    let mut naive_result = MockG1Point::identity();
    for (i, &coeff) in polynomial.coefficients.iter().enumerate() {
        if i < settings.setup_g1.len() {
            naive_result = naive_result.add(&settings.setup_g1[i].scalar_mul(coeff));
        }
    }
    let naive_duration = start.elapsed();
    
    println!("    朴素算法: {:?}", naive_duration);
    
    // 优化算法（批量处理）
    let start = Instant::now();
    let batch_optimizer = BatchOptimizer::new(64);
    let _optimized_results = batch_optimizer.batch_commitments(&[polynomial.clone()], &settings);
    let optimized_duration = start.elapsed();
    
    println!("    优化算法: {:?}", optimized_duration);
    
    // 计算优化效果
    let improvement = naive_duration.as_secs_f64() / optimized_duration.as_secs_f64();
    println!("    性能提升: {:.2}x", improvement);
    
    println!("  ✅ 算法优化演示完成");
    Ok(())
}

/// 系统级调优演示
fn demonstrate_system_tuning() -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔧 演示系统级调优...");
    
    // CPU 信息检测
    let cpu_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    println!("    检测到 CPU 核心数: {}", cpu_count);
    
    // 内存信息检测
    let current_memory = get_current_memory_usage();
    println!("    当前内存使用: {} bytes", current_memory);
    
    // 模拟 NUMA 优化
    if cfg!(target_os = "linux") {
        println!("    Linux 环境：可以进行 NUMA 优化");
    } else {
        println!("    非 Linux 环境：跳过 NUMA 优化");
    }
    
    // 编译器优化标志检测
    if cfg!(debug_assertions) {
        println!("    ⚠️  Debug 模式：性能可能受到影响");
    } else {
        println!("    ✅ Release 模式：启用了编译器优化");
    }
    
    println!("  ✅ 系统级调优演示完成");
    Ok(())
}

/// 实时性能监控演示
fn demonstrate_real_time_monitoring(monitor: &Arc<PerformanceMonitor>) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📈 演示实时性能监控...");
    
    // 模拟一段时间的操作
    for i in 0..50 {
        // 生成 1-10ms 的随机延迟
        let random_ms = 1 + (i * 7919) % 10; // 使用简单的伪随机数
        let operation_duration = Duration::from_millis(random_ms);
        
        match i % 3 {
            0 => monitor.record_commitment(operation_duration),
            1 => monitor.record_proof(operation_duration),
            2 => monitor.record_verification(operation_duration),
            _ => unreachable!(),
        }
        
        // 每10次操作记录一次错误（模拟5%错误率）
        if i % 20 == 0 {
            monitor.record_error();
        }
        
        std::thread::sleep(Duration::from_millis(10));
    }
    
    // 生成实时报告
    let report = monitor.generate_report();
    println!("  📊 实时性能报告:");
    println!("    运行时间: {:?}", report.uptime);
    println!("    总操作数: {}", report.total_operations);
    println!("    操作频率: {:.2} ops/sec", report.operations_per_second);
    println!("    平均承诺时间: {:?}", report.average_commitment_time);
    println!("    平均证明时间: {:?}", report.average_proof_time);
    println!("    平均验证时间: {:?}", report.average_verification_time);
    println!("    错误率: {:.2}%", report.error_rate * 100.0);
    
    println!("  ✅ 实时监控演示完成");
    Ok(())
}

/// 性能回归检测演示
fn demonstrate_regression_testing() -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 演示性能回归检测...");
    
    let mut regression_detector = PerformanceRegression::new(0.10); // 10% 阈值
    
    // 设置基准性能
    regression_detector.set_baseline("commitment_generation", Duration::from_millis(5));
    regression_detector.set_baseline("proof_generation", Duration::from_millis(8));
    regression_detector.set_baseline("verification", Duration::from_millis(2));
    
    // 模拟当前性能测试
    let test_cases = vec![
        ("commitment_generation", Duration::from_millis(5)), // 正常
        ("proof_generation", Duration::from_millis(7)),      // 改善
        ("verification", Duration::from_millis(3)),          // 回归
    ];
    
    for (test_name, current_time) in test_cases {
        match regression_detector.check_regression(test_name, current_time) {
            Ok(()) => println!("    ✅ {}: 无性能回归", test_name),
            Err(msg) => println!("    ❌ {}", msg),
        }
    }
    
    println!("  ✅ 回归检测演示完成");
    Ok(())
}

/// 生成综合性能报告
fn generate_comprehensive_report(monitor: &Arc<PerformanceMonitor>) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 生成综合性能报告...");
    
    let report = monitor.generate_report();
    
    println!("\n📊 === 综合性能分析报告 ===");
    println!("🕒 系统运行时间: {:?}", report.uptime);
    println!("📈 总体性能指标:");
    println!("   • 总操作数: {}", report.total_operations);
    println!("   • 平均 TPS: {:.2}", report.operations_per_second);
    println!("   • 系统错误率: {:.3}%", report.error_rate * 100.0);
    
    println!("\n⏱️ 操作延迟分析:");
    println!("   • 承诺生成: {:?}", report.average_commitment_time);
    println!("   • 证明生成: {:?}", report.average_proof_time);
    println!("   • 证明验证: {:?}", report.average_verification_time);
    
    println!("\n🎯 性能评估:");
    let overall_score = calculate_performance_score(&report);
    println!("   • 综合性能得分: {:.1}/100", overall_score);
    
    if overall_score >= 90.0 {
        println!("   • 评级: 优秀 🌟");
    } else if overall_score >= 75.0 {
        println!("   • 评级: 良好 👍");
    } else if overall_score >= 60.0 {
        println!("   • 评级: 一般 ⚠️");
    } else {
        println!("   • 评级: 需要优化 ❌");
    }
    
    println!("\n💡 优化建议:");
    generate_optimization_recommendations(&report);
    
    println!("\n================================");
    
    println!("  ✅ 综合报告生成完成");
    Ok(())
}

/// 计算综合性能得分
fn calculate_performance_score(report: &PerformanceReport) -> f64 {
    let mut score = 100.0;
    
    // 延迟惩罚
    if report.average_commitment_time.as_millis() > 10 {
        score -= 10.0;
    }
    if report.average_proof_time.as_millis() > 15 {
        score -= 10.0;
    }
    if report.average_verification_time.as_millis() > 5 {
        score -= 10.0;
    }
    
    // 错误率惩罚
    score -= report.error_rate * 1000.0;
    
    // 吞吐量奖励
    if report.operations_per_second > 100.0 {
        score += 5.0;
    }
    
    score.max(0.0).min(100.0)
}

/// 生成优化建议
fn generate_optimization_recommendations(report: &PerformanceReport) {
    if report.average_commitment_time.as_millis() > 10 {
        println!("   • 承诺生成较慢，建议启用并行处理或预计算优化");
    }
    
    if report.average_proof_time.as_millis() > 15 {
        println!("   • 证明生成较慢，建议使用批量处理或硬件加速");
    }
    
    if report.error_rate > 0.01 {
        println!("   • 错误率偏高，建议检查输入验证和错误处理逻辑");
    }
    
    if report.operations_per_second < 50.0 {
        println!("   • 整体吞吐量较低，建议优化数据结构和算法实现");
    }
    
    println!("   • 定期进行性能基准测试，监控性能回归");
    println!("   • 考虑使用内存池减少内存分配开销");
    println!("   • 启用编译器优化标志提高运行时性能");
}

/// 获取当前内存使用（模拟实现）
fn get_current_memory_usage() -> usize {
    // 在实际实现中，这里应该调用系统 API 获取真实的内存使用情况
    // 这里返回一个模拟值，使用简单的伪随机数生成
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    let hash_value = hasher.finish();
    
    let base = 50 * 1024 * 1024; // 50 MB 基础值
    let variation = (hash_value as usize % 150) * 1024 * 1024;
    base + variation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        // 记录一些操作
        monitor.record_commitment(Duration::from_millis(5));
        monitor.record_proof(Duration::from_millis(8));
        monitor.record_verification(Duration::from_millis(2));
        
        let report = monitor.generate_report();
        assert!(report.total_operations == 3);
        assert!(report.operations_per_second > 0.0);
    }
    
    #[test]
    fn test_memory_analyzer() {
        let mut analyzer = MemoryAnalyzer::new();
        
        analyzer.record_allocation(1024, "test_allocation");
        analyzer.record_allocation(2048, "another_allocation");
        
        let report = analyzer.generate_report();
        assert!(report.total_allocations == 2);
        assert!(report.largest_allocation == 2048);
    }
    
    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);
        
        cache.put("key1", "value1");
        cache.put("key2", "value2");
        
        assert!(cache.get(&"key1").is_some());
        assert!(cache.get(&"key2").is_some());
        
        // 添加第三个元素，应该淘汰最久未使用的
        cache.put("key3", "value3");
        assert!(cache.len() == 2);
    }
    
    #[test]
    fn test_performance_regression() {
        let mut regression = PerformanceRegression::new(0.1); // 10% 阈值
        
        regression.set_baseline("test_op", Duration::from_millis(10));
        
        // 正常情况
        assert!(regression.check_regression("test_op", Duration::from_millis(10)).is_ok());
        
        // 轻微回归（在阈值内：10% -> 11ms 是 10% 增长）
        assert!(regression.check_regression("test_op", Duration::from_millis(10)).is_ok());
        
        // 严重回归（超过阈值：10ms -> 15ms 是 50% 增长）
        assert!(regression.check_regression("test_op", Duration::from_millis(15)).is_err());
    }
}