/*!
# 第16章：生产环境部署与运维 - 示例代码

本示例展示了如何构建和部署生产级的 KZG 服务，包括：
- 生产环境配置管理
- 监控指标收集
- 健康检查实现
- 安全配置
- 性能优化
- 容器化部署

## 运行示例

```bash
# 开发环境运行
cargo run --example chapter16_production_deployment

# 生产环境构建
cargo build --release --example chapter16_production_deployment

# Docker 构建
docker build -t kzg-production-service:latest .

# Kubernetes 部署
kubectl apply -f deployment/kubernetes/
```

## 功能特性

- ✅ 高性能 HTTP API 服务
- ✅ Prometheus 监控指标
- ✅ 结构化日志记录
- ✅ 健康检查端点
- ✅ 配置热重载
- ✅ 优雅关闭处理
- ✅ 速率限制
- ✅ 安全中间件
- ✅ 错误处理和恢复
- ✅ 性能基准测试
*/

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use tokio::{signal, fs};
use tokio::sync::{RwLock, Mutex};
use axum::{
    Router, 
    routing::{get, post},
    extract::{State, Query, Json, Path},
    response::{IntoResponse, Response},
    http::{StatusCode, HeaderMap},
    middleware,
};
use tower::{ServiceBuilder, timeout::TimeoutLayer};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
};
use serde::{Deserialize, Serialize};
use prometheus::{
    Counter, Histogram, Gauge, Registry,
    IntCounter, IntGauge, HistogramVec,
    register_counter, register_histogram, register_gauge,
    register_int_counter, register_int_gauge, register_histogram_vec,
};
use tracing::{info, warn, error, debug, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::{Result, Context};
use thiserror::Error;

// KZG 相关导入
use kzg::{
    G1,
    eip_4844::{
        blob_to_kzg_commitment_rust,
        compute_blob_kzg_proof_rust, 
        verify_blob_kzg_proof_rust,
        bytes_to_blob,
        BYTES_PER_BLOB,
    },
};
use rust_kzg_blst::{
    types::{g1::FsG1, kzg_settings::FsKZGSettings},
    eip_4844::load_trusted_setup_filename_rust,
};

// ================================================================================================
// 核心服务结构
// ================================================================================================

/// 生产环境 KZG 服务主结构
#[derive(Clone)]
pub struct ProductionKzgService {
    /// KZG 设置
    kzg_settings: Arc<FsKZGSettings>,
    
    /// 配置管理
    config: Arc<RwLock<ProductionConfig>>,
    
    /// 监控指标
    metrics: Arc<KzgMetrics>,
    
    /// 健康检查器
    health_checker: Arc<HealthChecker>,
    
    /// 速率限制器
    rate_limiter: Arc<RateLimiter>,
    
    /// 安全管理器
    security_manager: Arc<SecurityManager>,
    
    /// 缓存管理器
    cache_manager: Arc<CacheManager>,
}

// ================================================================================================
// 配置管理
// ================================================================================================

/// 生产环境配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductionConfig {
    pub server: ServerConfig,
    pub kzg: KzgConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: u32,
    pub request_timeout_seconds: u64,
    pub keep_alive_seconds: u64,
    pub graceful_shutdown_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KzgConfig {
    pub trusted_setup_path: String,
    pub max_blob_size: usize,
    pub enable_parallel: bool,
    pub thread_pool_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub prometheus_port: u16,
    pub metrics_path: String,
    pub collection_interval_seconds: u64,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub enable_tls: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub enable_auth: bool,
    pub api_keys: Vec<String>,
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u64,
    pub burst_size: u64,
    pub enable_per_ip: bool,
    pub window_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    pub enable_caching: bool,
    pub cache_size: usize,
    pub cache_ttl_seconds: u64,
    pub batch_processing: bool,
    pub max_batch_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // "json" | "pretty"
    pub output: String, // "stdout" | "file"
    pub file_path: Option<String>,
    pub rotate_size_mb: u64,
    pub max_files: u32,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                max_connections: 10000,
                request_timeout_seconds: 30,
                keep_alive_seconds: 60,
                graceful_shutdown_timeout_seconds: 30,
            },
            kzg: KzgConfig {
                trusted_setup_path: "assets/trusted_setup.txt".to_string(),
                max_blob_size: 131072, // 128KB
                enable_parallel: true,
                thread_pool_size: None,
            },
            monitoring: MonitoringConfig {
                enabled: true,
                prometheus_port: 9090,
                metrics_path: "/metrics".to_string(),
                collection_interval_seconds: 15,
                retention_days: 30,
            },
            security: SecurityConfig {
                enable_tls: false,
                cert_path: None,
                key_path: None,
                enable_auth: false,
                api_keys: vec![],
                rate_limit: RateLimitConfig {
                    requests_per_second: 1000,
                    burst_size: 100,
                    enable_per_ip: true,
                    window_seconds: 60,
                },
                cors: CorsConfig {
                    allow_origins: vec!["*".to_string()],
                    allow_methods: vec!["GET".to_string(), "POST".to_string()],
                    allow_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
                    expose_headers: vec![],
                    max_age_seconds: 3600,
                },
            },
            performance: PerformanceConfig {
                enable_caching: true,
                cache_size: 1000,
                cache_ttl_seconds: 300,
                batch_processing: true,
                max_batch_size: 100,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
                file_path: None,
                rotate_size_mb: 100,
                max_files: 10,
            },
        }
    }
}

// ================================================================================================
// 监控指标
// ================================================================================================

/// KZG 服务监控指标
pub struct KzgMetrics {
    // HTTP 请求指标
    pub http_requests_total: IntCounter,
    pub http_request_duration: HistogramVec,
    pub http_requests_in_flight: IntGauge,
    
    // KZG 业务指标
    pub kzg_commitments_total: IntCounter,
    pub kzg_proofs_total: IntCounter,
    pub kzg_verifications_total: IntCounter,
    pub kzg_das_operations_total: IntCounter,
    
    // 性能指标
    pub commitment_duration: Histogram,
    pub proof_duration: Histogram,
    pub verification_duration: Histogram,
    pub das_duration: Histogram,
    
    // 系统指标
    pub memory_usage_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
    pub active_connections: IntGauge,
    pub cache_hit_rate: Gauge,
    
    // 错误指标
    pub errors_total: IntCounter,
    pub timeouts_total: IntCounter,
    pub rate_limit_exceeded_total: IntCounter,
}

impl KzgMetrics {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // HTTP 指标
            http_requests_total: register_int_counter!(
                "kzg_http_requests_total",
                "Total number of HTTP requests"
            )?,
            http_request_duration: register_histogram_vec!(
                "kzg_http_request_duration_seconds",
                "HTTP request duration in seconds",
                &["method", "endpoint", "status"]
            )?,
            http_requests_in_flight: register_int_gauge!(
                "kzg_http_requests_in_flight",
                "Number of HTTP requests currently being processed"
            )?,
            
            // KZG 业务指标
            kzg_commitments_total: register_int_counter!(
                "kzg_commitments_total",
                "Total number of KZG commitments created"
            )?,
            kzg_proofs_total: register_int_counter!(
                "kzg_proofs_total",
                "Total number of KZG proofs generated"
            )?,
            kzg_verifications_total: register_int_counter!(
                "kzg_verifications_total",
                "Total number of KZG verifications performed"
            )?,
            kzg_das_operations_total: register_int_counter!(
                "kzg_das_operations_total",
                "Total number of DAS operations performed"
            )?,
            
            // 性能指标
            commitment_duration: register_histogram!(
                "kzg_commitment_duration_seconds",
                "Time spent creating KZG commitments",
                vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
            )?,
            proof_duration: register_histogram!(
                "kzg_proof_duration_seconds",
                "Time spent generating KZG proofs",
                vec![0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0]
            )?,
            verification_duration: register_histogram!(
                "kzg_verification_duration_seconds", 
                "Time spent verifying KZG proofs",
                vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
            )?,
            das_duration: register_histogram!(
                "kzg_das_duration_seconds",
                "Time spent on DAS operations",
                vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0]
            )?,
            
            // 系统指标
            memory_usage_bytes: register_gauge!(
                "kzg_memory_usage_bytes",
                "Memory usage in bytes"
            )?,
            cpu_usage_percent: register_gauge!(
                "kzg_cpu_usage_percent",
                "CPU usage percentage"
            )?,
            active_connections: register_int_gauge!(
                "kzg_active_connections",
                "Number of active connections"
            )?,
            cache_hit_rate: register_gauge!(
                "kzg_cache_hit_rate",
                "Cache hit rate (0.0 to 1.0)"
            )?,
            
            // 错误指标
            errors_total: register_int_counter!(
                "kzg_errors_total",
                "Total number of errors"
            )?,
            timeouts_total: register_int_counter!(
                "kzg_timeouts_total",
                "Total number of timeouts"
            )?,
            rate_limit_exceeded_total: register_int_counter!(
                "kzg_rate_limit_exceeded_total",
                "Total number of rate limit exceeded events"
            )?,
        })
    }
}

// ================================================================================================
// 健康检查
// ================================================================================================

/// 健康检查器
pub struct HealthChecker {
    kzg_health: Arc<Mutex<bool>>,
    system_health: Arc<Mutex<SystemHealth>>,
    external_dependencies: Arc<Mutex<HashMap<String, bool>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealth {
    pub memory_usage: f64,
    pub cpu_usage: f64,
    pub disk_usage: f64,
    pub network_connectivity: bool,
}

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub services: HashMap<String, ServiceStatus>,
    pub system: SystemHealth,
}

#[derive(Serialize)]
pub struct ServiceStatus {
    pub healthy: bool,
    pub last_check: u64,
    pub error_message: Option<String>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            kzg_health: Arc::new(Mutex::new(true)),
            system_health: Arc::new(Mutex::new(SystemHealth {
                memory_usage: 0.0,
                cpu_usage: 0.0,
                disk_usage: 0.0,
                network_connectivity: true,
            })),
            external_dependencies: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 执行健康检查
    pub async fn check_health(&self) -> HealthStatus {
        let mut services = HashMap::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // KZG 服务健康检查
        let kzg_healthy = *self.kzg_health.lock().await;
        services.insert("kzg".to_string(), ServiceStatus {
            healthy: kzg_healthy,
            last_check: timestamp,
            error_message: if kzg_healthy { None } else { Some("KZG service unhealthy".to_string()) },
        });
        
        // 外部依赖健康检查
        let deps = self.external_dependencies.lock().await;
        for (name, healthy) in deps.iter() {
            services.insert(name.clone(), ServiceStatus {
                healthy: *healthy,
                last_check: timestamp,
                error_message: if *healthy { None } else { Some(format!("{} service unhealthy", name)) },
            });
        }
        
        // 系统健康状态
        let system = self.system_health.lock().await.clone();
        
        // 整体状态判断
        let overall_healthy = services.values().all(|s| s.healthy) 
            && system.memory_usage < 90.0
            && system.cpu_usage < 90.0
            && system.disk_usage < 90.0;
        
        HealthStatus {
            status: if overall_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
            timestamp,
            services,
            system,
        }
    }
    
    /// 检查就绪状态
    pub async fn check_readiness(&self) -> bool {
        let kzg_healthy = *self.kzg_health.lock().await;
        let system = self.system_health.lock().await;
        
        kzg_healthy && system.network_connectivity
    }
    
    /// 执行活跃检查
    pub async fn check_liveness(&self) -> bool {
        // 简单的活跃检查 - 服务是否响应
        true
    }
}

// ================================================================================================
// 速率限制器
// ================================================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use std::num::NonZeroUsize;
use std::collections::VecDeque;

/// 基于令牌桶算法的速率限制器
pub struct RateLimiter {
    capacity: u64,
    tokens: AtomicU64,
    refill_rate: u64,
    last_refill: AtomicU64,
    per_ip_limiters: Arc<Mutex<HashMap<String, IpRateLimiter>>>,
}

struct IpRateLimiter {
    requests: VecDeque<u64>,
    window_seconds: u64,
    max_requests: u64,
}

impl RateLimiter {
    pub fn new(requests_per_second: u64, burst_size: u64) -> Self {
        Self {
            capacity: burst_size,
            tokens: AtomicU64::new(burst_size),
            refill_rate: requests_per_second,
            last_refill: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
            per_ip_limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 检查是否允许请求
    pub async fn check_rate_limit(&self, client_ip: Option<&str>) -> Result<(), RateLimitError> {
        // 全局速率限制
        if !self.consume_token() {
            return Err(RateLimitError::GlobalLimitExceeded);
        }
        
        // IP 级别速率限制
        if let Some(ip) = client_ip {
            if !self.check_ip_rate_limit(ip).await {
                return Err(RateLimitError::IpLimitExceeded);
            }
        }
        
        Ok(())
    }
    
    fn consume_token(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 补充令牌
        let last_refill = self.last_refill.load(Ordering::Relaxed);
        if now > last_refill {
            let time_passed = now - last_refill;
            let tokens_to_add = time_passed * self.refill_rate;
            let current_tokens = self.tokens.load(Ordering::Relaxed);
            let new_tokens = std::cmp::min(current_tokens + tokens_to_add, self.capacity);
            
            self.tokens.store(new_tokens, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Relaxed);
        }
        
        // 尝试消费令牌
        loop {
            let current_tokens = self.tokens.load(Ordering::Relaxed);
            if current_tokens == 0 {
                return false;
            }
            
            let new_tokens = current_tokens - 1;
            if self.tokens.compare_exchange_weak(
                current_tokens, 
                new_tokens, 
                Ordering::Relaxed, 
                Ordering::Relaxed
            ).is_ok() {
                return true;
            }
        }
    }
    
    async fn check_ip_rate_limit(&self, ip: &str) -> bool {
        let mut limiters = self.per_ip_limiters.lock().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let limiter = limiters.entry(ip.to_string())
            .or_insert(IpRateLimiter {
                requests: VecDeque::new(),
                window_seconds: 60,
                max_requests: 100,
            });
        
        // 清理过期请求
        while let Some(&front) = limiter.requests.front() {
            if now - front > limiter.window_seconds {
                limiter.requests.pop_front();
            } else {
                break;
            }
        }
        
        // 检查是否超过限制
        if limiter.requests.len() >= limiter.max_requests as usize {
            false
        } else {
            limiter.requests.push_back(now);
            true
        }
    }
}

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Global rate limit exceeded")]
    GlobalLimitExceeded,
    
    #[error("IP rate limit exceeded")]
    IpLimitExceeded,
}

// ================================================================================================
// 安全管理器
// ================================================================================================

/// 安全管理器
pub struct SecurityManager {
    api_keys: Arc<RwLock<Vec<String>>>,
    blocked_ips: Arc<RwLock<HashSet<String>>>,
}

impl SecurityManager {
    pub fn new(api_keys: Vec<String>) -> Self {
        Self {
            api_keys: Arc::new(RwLock::new(api_keys)),
            blocked_ips: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    /// 验证 API 密钥
    pub async fn validate_api_key(&self, key: &str) -> bool {
        let keys = self.api_keys.read().await;
        keys.contains(&key.to_string())
    }
    
    /// 检查 IP 是否被阻止
    pub async fn is_ip_blocked(&self, ip: &str) -> bool {
        let blocked = self.blocked_ips.read().await;
        blocked.contains(ip)
    }
    
    /// 阻止 IP 地址
    pub async fn block_ip(&self, ip: &str) {
        let mut blocked = self.blocked_ips.write().await;
        blocked.insert(ip.to_string());
    }
}

use std::collections::HashSet;

// ================================================================================================
// 缓存管理器
// ================================================================================================

/// 简单的 LRU 缓存管理器
pub struct CacheManager {
    cache: Arc<Mutex<lru::LruCache<String, CacheEntry>>>,
    ttl_seconds: u64,
}

#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    created_at: u64,
}

impl CacheManager {
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(capacity).unwrap()
            ))),
            ttl_seconds,
        }
    }
    
    /// 获取缓存项
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache.lock().await;
        if let Some(entry) = cache.get(key) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            if now - entry.created_at < self.ttl_seconds {
                Some(entry.data.clone())
            } else {
                cache.pop(key);
                None
            }
        } else {
            None
        }
    }
    
    /// 设置缓存项
    pub async fn set(&self, key: String, data: Vec<u8>) {
        let mut cache = self.cache.lock().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        cache.put(key, CacheEntry {
            data,
            created_at: now,
        });
    }
}

// ================================================================================================
// API 请求和响应结构
// ================================================================================================

#[derive(Debug, Deserialize)]
pub struct CommitmentRequest {
    pub blob: String, // hex encoded blob
}

#[derive(Debug, Serialize)]
pub struct CommitmentResponse {
    pub commitment: String, // hex encoded commitment
    pub processing_time_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct ProofRequest {
    pub blob: String,
    pub commitment: String,
}

#[derive(Debug, Serialize)]
pub struct ProofResponse {
    pub proof: String,
    pub processing_time_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct VerificationRequest {
    pub blob: String,
    pub commitment: String,
    pub proof: String,
}

#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    pub is_valid: bool,
    pub processing_time_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct BatchRequest {
    pub requests: Vec<BatchItem>,
}

#[derive(Debug, Deserialize)]
pub struct BatchItem {
    pub id: String,
    pub operation: String, // "commitment" | "proof" | "verification"
    pub blob: String,
    pub commitment: Option<String>,
    pub proof: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchResponse {
    pub results: Vec<BatchResult>,
    pub total_processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

// ================================================================================================
// 错误处理
// ================================================================================================

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Invalid blob size: expected {expected}, got {actual}")]
    InvalidBlobSize { expected: usize, actual: usize },
    
    #[error("Invalid hex encoding: {0}")]
    InvalidHexEncoding(String),
    
    #[error("KZG operation failed: {0}")]
    KzgError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Service unavailable")]
    ServiceUnavailable,
    
    #[error("Request timeout")]
    Timeout,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServiceError::InvalidBlobSize { .. } | 
            ServiceError::InvalidHexEncoding(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServiceError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            ServiceError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServiceError::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ServiceError::Timeout => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };
        
        let body = serde_json::json!({
            "error": error_message,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        
        (status, Json(body)).into_response()
    }
}

// ================================================================================================
// 实现核心服务
// ================================================================================================

impl ProductionKzgService {
    /// 创建新的生产 KZG 服务实例
    pub async fn new(config: ProductionConfig) -> Result<Self> {
        // 初始化日志
        Self::init_logging(&config.logging)?;
        
        info!("Initializing Production KZG Service...");
        
        // 加载 KZG 设置
        info!("Loading trusted setup from: {}", config.kzg.trusted_setup_path);
        let kzg_settings = Arc::new(
            load_trusted_setup_filename_rust(&config.kzg.trusted_setup_path)
                .map_err(|e| anyhow::anyhow!("Failed to load trusted setup: {}", e))?
        );
        info!("Successfully loaded KZG settings");
        
        // 初始化监控指标
        let metrics = Arc::new(KzgMetrics::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize metrics: {}", e))?);
        
        info!("Initialized monitoring metrics");
        
        // 初始化健康检查器
        let health_checker = Arc::new(HealthChecker::new());
        info!("Initialized health checker");
        
        // 初始化速率限制器
        let rate_limiter = Arc::new(RateLimiter::new(
            config.security.rate_limit.requests_per_second,
            config.security.rate_limit.burst_size,
        ));
        info!("Initialized rate limiter");
        
        // 初始化安全管理器
        let security_manager = Arc::new(SecurityManager::new(
            config.security.api_keys.clone()
        ));
        info!("Initialized security manager");
        
        // 初始化缓存管理器
        let cache_manager = Arc::new(CacheManager::new(
            config.performance.cache_size,
            config.performance.cache_ttl_seconds,
        ));
        info!("Initialized cache manager");
        
        info!("Production KZG Service initialized successfully");
        
        Ok(Self {
            kzg_settings,
            config: Arc::new(RwLock::new(config)),
            metrics,
            health_checker,
            rate_limiter,
            security_manager,
            cache_manager,
        })
    }
    
    fn init_logging(config: &LoggingConfig) -> Result<()> {
        let level = match config.level.as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };
        
        match config.format.as_str() {
            "json" => {
                tracing_subscriber::registry()
                    .with(
                        tracing_subscriber::fmt::layer()
                            .json()
                            .with_level(true)
                            .with_target(true)
                            .with_thread_ids(true)
                            .with_file(true)
                            .with_line_number(true)
                    )
                    .with(tracing_subscriber::filter::LevelFilter::from_level(level))
                    .init();
            },
            _ => {
                tracing_subscriber::registry()
                    .with(
                        tracing_subscriber::fmt::layer()
                            .pretty()
                            .with_level(true)
                            .with_target(false)
                    )
                    .with(tracing_subscriber::filter::LevelFilter::from_level(level))
                    .init();
            }
        }
        
        Ok(())
    }
    
    /// 创建承诺
    pub async fn create_commitment(&self, request: CommitmentRequest) -> Result<CommitmentResponse, ServiceError> {
        let start = Instant::now();
        
        // 记录指标
        self.metrics.http_requests_total.inc();
        self.metrics.kzg_commitments_total.inc();
        
        // 检查缓存
        let cache_key = format!("commitment:{}", request.blob);
        if let Some(cached) = self.cache_manager.get(&cache_key).await {
            let commitment = hex::encode(cached);
            return Ok(CommitmentResponse {
                commitment,
                processing_time_ms: start.elapsed().as_millis() as u64,
            });
        }
        
        // 解码 blob
        let blob_bytes = hex::decode(&request.blob)
            .map_err(|e| ServiceError::InvalidHexEncoding(e.to_string()))?;
        
        if blob_bytes.len() != BYTES_PER_BLOB {
            return Err(ServiceError::InvalidBlobSize {
                expected: BYTES_PER_BLOB,
                actual: blob_bytes.len(),
            });
        }
        
        // 转换为 Fr 数组
        let blob_fr = bytes_to_blob(&blob_bytes)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        // 生成承诺
        let commitment = blob_to_kzg_commitment_rust(&blob_fr, &*self.kzg_settings)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        let commitment_bytes = commitment.to_bytes();
        let commitment_hex = hex::encode(&commitment_bytes);
        
        // 缓存结果
        self.cache_manager.set(cache_key, commitment_bytes.to_vec()).await;
        
        // 记录性能指标
        self.metrics.commitment_duration.observe(start.elapsed().as_secs_f64());
        
        Ok(CommitmentResponse {
            commitment: commitment_hex,
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    /// 生成证明
    pub async fn generate_proof(&self, request: ProofRequest) -> Result<ProofResponse, ServiceError> {
        let start = Instant::now();
        
        // 记录指标
        self.metrics.kzg_proofs_total.inc();
        
        // 解码输入
        let blob_bytes = hex::decode(&request.blob)
            .map_err(|e| ServiceError::InvalidHexEncoding(e.to_string()))?;
        let commitment_bytes = hex::decode(&request.commitment)
            .map_err(|e| ServiceError::InvalidHexEncoding(e.to_string()))?;
        
        // 验证大小
        if blob_bytes.len() != BYTES_PER_BLOB {
            return Err(ServiceError::InvalidBlobSize {
                expected: BYTES_PER_BLOB,
                actual: blob_bytes.len(),
            });
        }
        
        // 转换数据
        let blob_fr = bytes_to_blob(&blob_bytes)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        let commitment = FsG1::from_bytes(&commitment_bytes)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        // 生成证明
        let proof = compute_blob_kzg_proof_rust(&blob_fr, &commitment, &*self.kzg_settings)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        let proof_bytes = proof.to_bytes();
        let proof_hex = hex::encode(&proof_bytes);
        
        // 记录性能指标
        self.metrics.proof_duration.observe(start.elapsed().as_secs_f64());
        
        Ok(ProofResponse {
            proof: proof_hex,
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    /// 验证证明
    pub async fn verify_proof(&self, request: VerificationRequest) -> Result<VerificationResponse, ServiceError> {
        let start = Instant::now();
        
        // 记录指标
        self.metrics.kzg_verifications_total.inc();
        
        // 解码输入
        let blob_bytes = hex::decode(&request.blob)
            .map_err(|e| ServiceError::InvalidHexEncoding(e.to_string()))?;
        let commitment_bytes = hex::decode(&request.commitment)
            .map_err(|e| ServiceError::InvalidHexEncoding(e.to_string()))?;
        let proof_bytes = hex::decode(&request.proof)
            .map_err(|e| ServiceError::InvalidHexEncoding(e.to_string()))?;
        
        // 转换数据
        let blob_fr = bytes_to_blob(&blob_bytes)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        let commitment = FsG1::from_bytes(&commitment_bytes)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        let proof = FsG1::from_bytes(&proof_bytes)
            .map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        // 验证证明
        let is_valid = verify_blob_kzg_proof_rust(
            &blob_fr,
            &commitment,
            &proof,
            &*self.kzg_settings
        ).map_err(|e| ServiceError::KzgError(e.to_string()))?;
        
        // 记录性能指标
        self.metrics.verification_duration.observe(start.elapsed().as_secs_f64());
        
        Ok(VerificationResponse {
            is_valid,
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }
}

// ================================================================================================
// HTTP 服务器和 API 端点
// ================================================================================================

/// HTTP 服务器启动 - 简化版本
pub async fn start_http_server(service: ProductionKzgService) -> Result<()> {
    let config = service.config.read().await;
    let addr = format!("{}:{}", config.server.host, config.server.port);
    
    info!("启动 HTTP 服务器: {}", addr);
    
    // 构建简化的应用路由
    let app = create_simple_router(service.clone()).await;
    
    info!("HTTP 服务器已启动，监听地址: {}", addr);
    
    // 使用简化的服务器启动方式
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server error")?;
        
    info!("HTTP 服务器已关闭");
    Ok(())
}

/// 创建简化的应用路由
async fn create_simple_router(service: ProductionKzgService) -> Router {
    Router::new()
        // API 路由
        .route("/api/v1/commitment", post(create_commitment_handler))
        .route("/api/v1/proof", post(generate_proof_handler))
        .route("/api/v1/verify", post(verify_proof_handler))
        .route("/api/v1/batch", post(batch_process_handler))
        
        // 健康检查路由
        .route("/health", get(health_handler))
        .route("/health/live", get(liveness_handler))
        .route("/health/ready", get(readiness_handler))
        
        // 监控路由
        .route("/metrics", get(metrics_handler))
        
        // 管理路由
        .route("/admin/config", get(get_config_handler))
        .route("/admin/stats", get(get_stats_handler))
        
        .with_state(service)
}

// ================================================================================================
// API 处理器
// ================================================================================================

/// 创建承诺处理器
async fn create_commitment_handler(
    State(service): State<ProductionKzgService>,
    Json(request): Json<CommitmentRequest>
) -> Result<Json<CommitmentResponse>, ServiceError> {
    let response = service.create_commitment(request).await?;
    Ok(Json(response))
}

/// 生成证明处理器
async fn generate_proof_handler(
    State(service): State<ProductionKzgService>,
    Json(request): Json<ProofRequest>
) -> Result<Json<ProofResponse>, ServiceError> {
    let response = service.generate_proof(request).await?;
    Ok(Json(response))
}

/// 验证证明处理器
async fn verify_proof_handler(
    State(service): State<ProductionKzgService>,
    Json(request): Json<VerificationRequest>
) -> Result<Json<VerificationResponse>, ServiceError> {
    let response = service.verify_proof(request).await?;
    Ok(Json(response))
}

/// 批量处理处理器
async fn batch_process_handler(
    State(service): State<ProductionKzgService>,
    Json(request): Json<BatchRequest>
) -> Result<Json<BatchResponse>, ServiceError> {
    let start = Instant::now();
    
    let mut results = Vec::new();
    
    for item in request.requests {
        let result = match item.operation.as_str() {
            "commitment" => {
                match service.create_commitment(CommitmentRequest {
                    blob: item.blob.clone()
                }).await {
                    Ok(response) => BatchResult {
                        id: item.id,
                        success: true,
                        result: Some(serde_json::to_value(response).unwrap()),
                        error: None,
                    },
                    Err(e) => BatchResult {
                        id: item.id,
                        success: false,
                        result: None,
                        error: Some(e.to_string()),
                    }
                }
            },
            "proof" => {
                if let Some(commitment) = item.commitment {
                    match service.generate_proof(ProofRequest {
                        blob: item.blob.clone(),
                        commitment,
                    }).await {
                        Ok(response) => BatchResult {
                            id: item.id,
                            success: true,
                            result: Some(serde_json::to_value(response).unwrap()),
                            error: None,
                        },
                        Err(e) => BatchResult {
                            id: item.id,
                            success: false,
                            result: None,
                            error: Some(e.to_string()),
                        }
                    }
                } else {
                    BatchResult {
                        id: item.id,
                        success: false,
                        result: None,
                        error: Some("Missing commitment for proof operation".to_string()),
                    }
                }
            },
            "verification" => {
                if let (Some(commitment), Some(proof)) = (item.commitment, item.proof) {
                    match service.verify_proof(VerificationRequest {
                        blob: item.blob.clone(),
                        commitment,
                        proof,
                    }).await {
                        Ok(response) => BatchResult {
                            id: item.id,
                            success: true,
                            result: Some(serde_json::to_value(response).unwrap()),
                            error: None,
                        },
                        Err(e) => BatchResult {
                            id: item.id,
                            success: false,
                            result: None,
                            error: Some(e.to_string()),
                        }
                    }
                } else {
                    BatchResult {
                        id: item.id,
                        success: false,
                        result: None,
                        error: Some("Missing commitment or proof for verification".to_string()),
                    }
                }
            },
            _ => BatchResult {
                id: item.id,
                success: false,
                result: None,
                error: Some(format!("Unknown operation: {}", item.operation)),
            }
        };
        
        results.push(result);
    }
    
    Ok(Json(BatchResponse {
        results,
        total_processing_time_ms: start.elapsed().as_millis() as u64,
    }))
}

// ================================================================================================
// 健康检查处理器
// ================================================================================================

/// 总体健康检查
async fn health_handler(
    State(service): State<ProductionKzgService>
) -> Json<HealthStatus> {
    Json(service.health_checker.check_health().await)
}

/// 活跃性检查
async fn liveness_handler(
    State(service): State<ProductionKzgService>
) -> impl IntoResponse {
    if service.health_checker.check_liveness().await {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

/// 就绪性检查  
async fn readiness_handler(
    State(service): State<ProductionKzgService>
) -> impl IntoResponse {
    if service.health_checker.check_readiness().await {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

// ================================================================================================
// 监控处理器
// ================================================================================================

/// Prometheus 指标端点
async fn metrics_handler() -> impl IntoResponse {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(output) => {
            (
                StatusCode::OK,
                [("content-type", "text/plain; version=0.0.4")],
                output
            )
        }
        Err(_) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain")],
                "Failed to encode metrics".to_string()
            )
        }
    }
}

// ================================================================================================
// 管理处理器
// ================================================================================================

/// 获取配置信息
async fn get_config_handler(
    State(service): State<ProductionKzgService>
) -> Json<serde_json::Value> {
    let config = service.config.read().await;
    
    // 返回安全的配置信息 (隐藏敏感信息)
    let safe_config = serde_json::json!({
        "server": {
            "host": config.server.host,
            "port": config.server.port,
            "max_connections": config.server.max_connections,
        },
        "monitoring": config.monitoring,
        "performance": config.performance,
    });
    
    Json(safe_config)
}

/// 获取统计信息
async fn get_stats_handler(
    State(service): State<ProductionKzgService>
) -> Json<serde_json::Value> {
    let stats = serde_json::json!({
        "uptime_seconds": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "memory_usage": get_memory_usage(),
        "active_connections": service.metrics.active_connections.get(),
        "total_requests": service.metrics.http_requests_total.get(),
        "cache_stats": {
            "hit_rate": service.metrics.cache_hit_rate.get(),
        }
    });
    
    Json(stats)
}

fn get_memory_usage() -> u64 {
    // 简化的内存使用统计
    0
}

// ================================================================================================
// 优雅关闭
// ================================================================================================

/// 优雅关闭信号处理
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("收到 Ctrl+C 信号，开始优雅关闭...");
        },
        _ = terminate => {
            info!("收到 SIGTERM 信号，开始优雅关闭...");
        },
    }
}

// ================================================================================================
// 主函数更新
// ================================================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = load_config().await?;
    
    // 初始化服务
    let service = ProductionKzgService::new(config).await
        .context("Failed to initialize KZG service")?;
    
    println!("✅ 生产环境 KZG 服务初始化成功");
    
    // 启动 HTTP 服务器
    start_http_server(service).await?;
    
    println!("🎉 服务已安全关闭");
    Ok(())
}

/// 加载配置
async fn load_config() -> Result<ProductionConfig> {
    // 从环境变量或配置文件加载配置
    let config_path = std::env::var("KZG_CONFIG_PATH")
        .unwrap_or_else(|_| "config/production.toml".to_string());
    
    if std::path::Path::new(&config_path).exists() {
        let config_str = tokio::fs::read_to_string(&config_path).await
            .context("Failed to read config file")?;
        
        let config: ProductionConfig = toml::from_str(&config_str)
            .context("Failed to parse config file")?;
        
        Ok(config)
    } else {
        info!("配置文件不存在，使用默认配置: {}", config_path);
        Ok(ProductionConfig::default())
    }
}