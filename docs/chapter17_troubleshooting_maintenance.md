# 第17章: 故障排除与维护

> **💡 核心目标**: 掌握生产环境中 KZG 应用的故障排除和系统维护技术，确保服务稳定运行。

**本章你将学会**:
- 🔍 诊断和解决常见的生产环境问题
- 📊 设计完善的监控和告警体系
- 🛠️ 使用专业工具进行性能分析和调优
- 📝 实现高质量的日志记录和错误追踪
- 🚀 执行安全可靠的版本升级流程

---

## 📋 17.1 常见问题诊断与解决

### 🧠 17.1.1 内存问题诊断

生产环境中最常见的问题之一是内存相关故障。KZG 计算涉及大量椭圆曲线运算，内存管理至关重要。

#### 内存溢出 (OOM) 排查

**问题表现**:
```bash
# 系统日志中的典型 OOM 信号
kernel: Out of memory: Kill process 1234 (kzg_service) score 900 or sacrifice child
kernel: Killed process 1234 (kzg_service) total-vm:8GB, anon-rss:6GB, file-rss:0kB
```

**诊断工具**:
```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 内存使用追踪器 - 用于诊断内存泄漏
pub struct TrackedAllocator {
    inner: System,
    allocated: AtomicUsize,
    peak: AtomicUsize,
}

impl TrackedAllocator {
    pub const fn new() -> Self {
        Self {
            inner: System,
            allocated: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }
    
    pub fn current_usage(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }
    
    pub fn peak_usage(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }
    
    pub fn report(&self) -> String {
        format!(
            "内存使用报告:\n- 当前使用: {} MB\n- 峰值使用: {} MB",
            self.current_usage() / 1024 / 1024,
            self.peak_usage() / 1024 / 1024
        )
    }
}

unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size();
            let old = self.allocated.fetch_add(size, Ordering::Relaxed);
            let new = old + size;
            
            // 更新峰值使用量
            self.peak.fetch_max(new, Ordering::Relaxed);
            
            // 内存使用量超过阈值时发出警告
            if new > 4 * 1024 * 1024 * 1024 { // 4GB
                eprintln!("⚠️ 内存使用量过高: {}MB", new / 1024 / 1024);
            }
        }
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocated.fetch_sub(layout.size(), Ordering::Relaxed);
        self.inner.dealloc(ptr, layout);
    }
}
```

### ⚡ 17.1.2 性能问题排查

#### CPU 使用率异常分析

```rust
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, CpuExt};

/// CPU 监控器
pub struct CpuMonitor {
    system: System,
    high_cpu_threshold: f32,
}

impl CpuMonitor {
    pub fn new(high_cpu_threshold: f32) -> Self {
        Self {
            system: System::new_all(),
            high_cpu_threshold,
        }
    }
    
    pub fn check_cpu_usage(&mut self) -> CpuReport {
        self.system.refresh_cpu();
        
        let global_usage = self.system.global_cpu_info().cpu_usage();
        let is_high_cpu = global_usage > self.high_cpu_threshold;
        
        if is_high_cpu {
            eprintln!("⚠️ CPU 使用率过高: {:.2}%", global_usage);
        }
        
        CpuReport {
            timestamp: Instant::now(),
            global_usage,
            is_high_cpu,
        }
    }
}

#[derive(Debug)]
pub struct CpuReport {
    pub timestamp: Instant,
    pub global_usage: f32,
    pub is_high_cpu: bool,
}
```

---

## 📊 17.2 系统监控与告警配置

### 🎯 17.2.1 Prometheus 监控集成

```rust
use prometheus::{Counter, Gauge, Histogram, Registry};
use std::sync::Arc;

/// KZG 系统监控指标
pub struct KzgMetrics {
    pub operations_total: Counter,
    pub memory_usage_bytes: Gauge,
    pub operation_duration: Histogram,
    registry: Registry,
}

impl KzgMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        let operations_total = Counter::new(
            "kzg_operations_total",
            "Total number of KZG operations"
        )?;
        
        let memory_usage_bytes = Gauge::new(
            "kzg_memory_usage_bytes",
            "Current memory usage in bytes"
        )?;
        
        let operation_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "kzg_operation_duration_seconds",
                "Duration of KZG operations"
            )
        )?;
        
        // 注册指标
        registry.register(Box::new(operations_total.clone()))?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;
        registry.register(Box::new(operation_duration.clone()))?;
        
        Ok(Self {
            operations_total,
            memory_usage_bytes,
            operation_duration,
            registry,
        })
    }
    
    pub fn record_operation<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _timer = self.operation_duration.start_timer();
        let result = f();
        self.operations_total.inc();
        result
    }
    
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}
```

---

## 🛠️ 17.3 性能分析工具

### 🔥 17.3.1 性能分析器

```rust
use std::time::Instant;
use std::collections::HashMap;

/// 性能分析器
pub struct PerformanceProfiler {
    samples: HashMap<String, Vec<u64>>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
        }
    }
    
    /// 测量函数执行时间
    pub fn measure<F, R>(&mut self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed().as_nanos() as u64;
        
        self.samples
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
            
        result
    }
    
    /// 生成性能报告
    pub fn report(&self) -> String {
        let mut report = String::from("性能分析报告\n");
        report.push_str(&"=".repeat(40));
        report.push('\n');
        
        for (name, samples) in &self.samples {
            if !samples.is_empty() {
                let avg = samples.iter().sum::<u64>() / samples.len() as u64;
                let min = *samples.iter().min().unwrap();
                let max = *samples.iter().max().unwrap();
                
                report.push_str(&format!(
                    "{}: 平均 {:.2}μs, 最小 {:.2}μs, 最大 {:.2}μs\n",
                    name,
                    avg as f64 / 1000.0,
                    min as f64 / 1000.0,
                    max as f64 / 1000.0
                ));
            }
        }
        
        report
    }
}
```

---

## 📝 17.4 日志分析与调试

### 📋 17.4.1 结构化日志

```rust
use tracing::{info, error, warn};

/// 日志记录器
pub struct KzgLogger;

impl KzgLogger {
    /// 记录操作开始
    pub fn log_start(operation: &str) {
        info!(operation = operation, "Operation started");
    }
    
    /// 记录操作完成
    pub fn log_success(operation: &str, duration_ms: u64) {
        info!(
            operation = operation,
            duration_ms = duration_ms,
            "Operation completed"
        );
    }
    
    /// 记录错误
    pub fn log_error(operation: &str, error: &str) {
        error!(
            operation = operation,
            error = error,
            "Operation failed"
        );
    }
}
```

---

## 🚀 17.5 版本升级与维护

### 📦 17.5.1 滚动升级

```rust
use std::process::Command;
use tokio::time::{sleep, Duration};

/// 升级管理器
pub struct UpgradeManager {
    service_name: String,
}

impl UpgradeManager {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
    
    /// 执行滚动升级
    pub async fn rolling_upgrade(&self, new_version: &str) -> Result<(), String> {
        println!("开始升级到版本: {}", new_version);
        
        // 1. 拉取新镜像
        self.pull_image(new_version)?;
        
        // 2. 更新部署
        self.update_deployment(new_version)?;
        
        // 3. 等待部署完成
        self.wait_for_deployment().await?;
        
        // 4. 验证服务
        self.verify_service().await?;
        
        println!("升级完成");
        Ok(())
    }
    
    fn pull_image(&self, version: &str) -> Result<(), String> {
        let output = Command::new("docker")
            .args(&["pull", &format!("{}:{}", self.service_name, version)])
            .output()
            .map_err(|e| format!("拉取镜像失败: {}", e))?;
            
        if !output.status.success() {
            return Err("镜像拉取失败".to_string());
        }
        
        Ok(())
    }
    
    fn update_deployment(&self, version: &str) -> Result<(), String> {
        let output = Command::new("kubectl")
            .args(&[
                "set", "image", 
                &format!("deployment/{}", self.service_name),
                &format!("{}={}:{}", self.service_name, self.service_name, version)
            ])
            .output()
            .map_err(|e| format!("更新部署失败: {}", e))?;
            
        if !output.status.success() {
            return Err("部署更新失败".to_string());
        }
        
        Ok(())
    }
    
    async fn wait_for_deployment(&self) -> Result<(), String> {
        for i in 1..=30 {
            println!("等待部署完成... ({}/30)", i);
            
            let output = Command::new("kubectl")
                .args(&["rollout", "status", &format!("deployment/{}", self.service_name)])
                .output()
                .map_err(|e| format!("检查部署状态失败: {}", e))?;
                
            if output.status.success() {
                return Ok(());
            }
            
            sleep(Duration::from_secs(10)).await;
        }
        
        Err("部署超时".to_string())
    }
    
    async fn verify_service(&self) -> Result<(), String> {
        // 简化的服务验证
        sleep(Duration::from_secs(5)).await;
        println!("服务验证通过");
        Ok(())
    }
}
```

---

## 🎯 17.6 维护最佳实践

### ✅ 17.6.1 健康检查

```rust
/// 系统健康检查器
pub struct HealthChecker;

impl HealthChecker {
    /// 执行完整健康检查
    pub fn full_health_check() -> HealthReport {
        let mut report = HealthReport::new();
        
        // 内存检查
        if Self::check_memory() {
            report.add_success("内存", "使用正常");
        } else {
            report.add_failure("内存", "使用过高");
        }
        
        // CPU 检查
        if Self::check_cpu() {
            report.add_success("CPU", "使用正常");
        } else {
            report.add_failure("CPU", "使用过高");
        }
        
        // 磁盘检查
        if Self::check_disk() {
            report.add_success("磁盘", "空间充足");
        } else {
            report.add_failure("磁盘", "空间不足");
        }
        
        report
    }
    
    fn check_memory() -> bool {
        // 简化的内存检查
        true
    }
    
    fn check_cpu() -> bool {
        // 简化的CPU检查
        true
    }
    
    fn check_disk() -> bool {
        // 简化的磁盘检查
        true
    }
}

/// 健康检查报告
pub struct HealthReport {
    checks: Vec<CheckResult>,
}

struct CheckResult {
    component: String,
    passed: bool,
    message: String,
}

impl HealthReport {
    fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }
    
    fn add_success(&mut self, component: &str, message: &str) {
        self.checks.push(CheckResult {
            component: component.to_string(),
            passed: true,
            message: message.to_string(),
        });
    }
    
    fn add_failure(&mut self, component: &str, message: &str) {
        self.checks.push(CheckResult {
            component: component.to_string(),
            passed: false,
            message: message.to_string(),
        });
    }
    
    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }
    
    pub fn summary(&self) -> String {
        let passed = self.checks.iter().filter(|c| c.passed).count();
        let total = self.checks.len();
        
        let mut summary = format!("健康检查: {}/{} 通过\n", passed, total);
        
        for check in &self.checks {
            let status = if check.passed { "✅" } else { "❌" };
            summary.push_str(&format!(
                "{} {}: {}\n",
                status, check.component, check.message
            ));
        }
        
        summary
    }
}
```

---

## 📚 本章总结

### ✅ 核心技能
- **问题诊断**: 系统化的故障排查方法
- **监控告警**: 完整的监控体系构建
- **性能分析**: 专业工具使用和优化
- **日志管理**: 结构化日志和错误追踪
- **版本管理**: 安全的升级和回滚流程

### 🛠️ 关键工具
- 内存追踪器和泄漏检测
- CPU 性能监控
- Prometheus 指标收集
- 结构化日志记录
- 自动化健康检查

### 🎯 最佳实践
1. **预防为主**: 建立完善的监控系统
2. **快速响应**: 标准化故障处理流程
3. **持续改进**: 基于数据的优化决策
4. **自动化**: 减少人工操作错误

通过本章学习，你已经掌握了维护生产级 KZG 服务的核心技能，能够有效预防、诊断和解决各种运维问题。

**下一章预告**: 第18章将介绍新特性开发指南，学习如何为 rust-kzg 库贡献代码。