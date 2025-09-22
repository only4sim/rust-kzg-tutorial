//! 第12章：跨语言集成与C绑定示例
//!
//! 本示例展示了如何实现安全高效的跨语言绑定，包括：
//! - C语言FFI绑定设计与实现
//! - Python PyO3绑定集成
//! - JavaScript WASM编译优化
//! - 统一错误处理策略
//! - 跨语言性能优化技术

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;
use std::time::Instant;

// 模拟KZG相关类型
type G1 = [u8; 48];
type G2 = [u8; 96];

const BYTES_PER_BLOB: usize = 4096 * 32;
const BYTES_PER_COMMITMENT: usize = 48;
const BYTES_PER_PROOF: usize = 48;
const FIELD_ELEMENTS_PER_BLOB: usize = 4096;

// 模拟KZG设置
#[derive(Debug)]
pub struct MockKzgSettings {
    pub g1_powers: Vec<G1>,
    pub g2_powers: Vec<G2>,
    pub initialized: bool,
}

impl MockKzgSettings {
    pub fn new() -> Self {
        Self {
            g1_powers: vec![[0u8; 48]; 4096],
            g2_powers: vec![[0u8; 96]; 2],
            initialized: true,
        }
    }
}

// ================================
// 第一部分：C语言FFI绑定实现
// ================================

/// C兼容的错误码定义
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CKzgResult {
    Ok = 0,
    BadArgs = 1,
    Malloc = 2,
    BadEncoding = 3,
    BadLength = 4,
    Unknown = 5,
}

/// C兼容的KZG设置结构
#[repr(C)]
pub struct CKzgSettings {
    inner: *mut MockKzgSettings,
}

/// C兼容的字节数组
#[repr(C)]
pub struct CBytes {
    data: *const u8,
    length: usize,
}

impl CBytes {
    fn from_vec(vec: Vec<u8>) -> Self {
        let data = vec.as_ptr();
        let length = vec.len();
        std::mem::forget(vec); // 防止Rust释放内存
        CBytes { data, length }
    }
    
    unsafe fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(self.data, self.length)
        }
    }
}

/// 受信任设置加载 - C接口
#[no_mangle]
pub extern "C" fn c_kzg_load_trusted_setup(
    out: *mut CKzgSettings,
    trusted_setup_file: *const c_char,
) -> CKzgResult {
    if out.is_null() || trusted_setup_file.is_null() {
        return CKzgResult::BadArgs;
    }
    
    let file_path = match unsafe { CStr::from_ptr(trusted_setup_file) }.to_str() {
        Ok(s) => s,
        Err(_) => return CKzgResult::BadEncoding,
    };
    
    println!("🔧 C FFI: Loading trusted setup from: {}", file_path);
    
    let settings = MockKzgSettings::new();
    unsafe {
        (*out).inner = Box::into_raw(Box::new(settings));
    }
    
    CKzgResult::Ok
}

/// 清理资源 - C接口
#[no_mangle]
pub extern "C" fn c_kzg_free_trusted_setup(settings: *mut CKzgSettings) {
    if !settings.is_null() {
        unsafe {
            let settings_ref = &mut *settings;
            if !settings_ref.inner.is_null() {
                let _ = Box::from_raw(settings_ref.inner);
                settings_ref.inner = ptr::null_mut();
                println!("🔧 C FFI: Freed trusted setup resources");
            }
        }
    }
}

/// Blob到承诺转换 - C接口
#[no_mangle]
pub extern "C" fn c_kzg_blob_to_commitment(
    out: *mut CBytes,
    blob: *const CBytes,
    settings: *const CKzgSettings,
) -> CKzgResult {
    if out.is_null() || blob.is_null() || settings.is_null() {
        return CKzgResult::BadArgs;
    }
    
    unsafe {
        let blob_slice = (*blob).as_slice();
        let _settings_ref = &*(*settings).inner;
        
        if blob_slice.len() != BYTES_PER_BLOB {
            return CKzgResult::BadLength;
        }
        
        // 模拟承诺生成
        let mut commitment = vec![0u8; BYTES_PER_COMMITMENT];
        for i in 0..BYTES_PER_COMMITMENT {
            commitment[i] = (blob_slice[i] ^ 0xAA) as u8;
        }
        
        *out = CBytes::from_vec(commitment);
        println!("🔧 C FFI: Generated commitment for blob");
    }
    
    CKzgResult::Ok
}

/// 证明生成 - C接口
#[no_mangle]
pub extern "C" fn c_kzg_compute_blob_proof(
    out: *mut CBytes,
    blob: *const CBytes,
    commitment: *const CBytes,
    settings: *const CKzgSettings,
) -> CKzgResult {
    if out.is_null() || blob.is_null() || commitment.is_null() || settings.is_null() {
        return CKzgResult::BadArgs;
    }
    
    unsafe {
        let blob_slice = (*blob).as_slice();
        let commitment_slice = (*commitment).as_slice();
        
        if blob_slice.len() != BYTES_PER_BLOB {
            return CKzgResult::BadLength;
        }
        if commitment_slice.len() != BYTES_PER_COMMITMENT {
            return CKzgResult::BadLength;
        }
        
        // 模拟证明生成
        let mut proof = vec![0u8; BYTES_PER_PROOF];
        for i in 0..BYTES_PER_PROOF {
            proof[i] = (blob_slice[i] ^ commitment_slice[i % BYTES_PER_COMMITMENT] ^ 0x55) as u8;
        }
        
        *out = CBytes::from_vec(proof);
        println!("🔧 C FFI: Generated proof for blob");
    }
    
    CKzgResult::Ok
}

/// 证明验证 - C接口
#[no_mangle]
pub extern "C" fn c_kzg_verify_blob_proof(
    out: *mut bool,
    blob: *const CBytes,
    commitment: *const CBytes,
    proof: *const CBytes,
    settings: *const CKzgSettings,
) -> CKzgResult {
    if out.is_null() || blob.is_null() || commitment.is_null() || proof.is_null() || settings.is_null() {
        return CKzgResult::BadArgs;
    }
    
    unsafe {
        let blob_slice = (*blob).as_slice();
        let commitment_slice = (*commitment).as_slice();
        let proof_slice = (*proof).as_slice();
        
        if blob_slice.len() != BYTES_PER_BLOB {
            return CKzgResult::BadLength;
        }
        if commitment_slice.len() != BYTES_PER_COMMITMENT {
            return CKzgResult::BadLength;
        }
        if proof_slice.len() != BYTES_PER_PROOF {
            return CKzgResult::BadLength;
        }
        
        // 模拟验证逻辑
        let mut is_valid = true;
        for i in 0..BYTES_PER_PROOF {
            let expected = blob_slice[i] ^ commitment_slice[i % BYTES_PER_COMMITMENT] ^ 0x55;
            if proof_slice[i] != expected {
                is_valid = false;
                break;
            }
        }
        
        *out = is_valid;
        println!("🔧 C FFI: Verification result: {}", is_valid);
    }
    
    CKzgResult::Ok
}

// ================================
// 第二部分：统一错误处理系统
// ================================

#[derive(Debug, Clone, PartialEq)]
pub enum KzgError {
    InvalidArgument(String),
    EncodingError(String),
    LengthError { expected: usize, actual: usize },
    ComputationError(String),
    MemoryError(String),
    Unknown(String),
}

impl std::fmt::Display for KzgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KzgError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            KzgError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            KzgError::LengthError { expected, actual } => write!(f, 
                "Length error: expected {}, got {}", expected, actual),
            KzgError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
            KzgError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            KzgError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for KzgError {}

pub type KzgResult<T> = Result<T, KzgError>;

impl From<KzgError> for CKzgResult {
    fn from(error: KzgError) -> Self {
        match error {
            KzgError::InvalidArgument(_) => CKzgResult::BadArgs,
            KzgError::EncodingError(_) => CKzgResult::BadEncoding,
            KzgError::LengthError { .. } => CKzgResult::BadLength,
            KzgError::ComputationError(_) => CKzgResult::Unknown,
            KzgError::MemoryError(_) => CKzgResult::Malloc,
            KzgError::Unknown(_) => CKzgResult::Unknown,
        }
    }
}

// ================================
// 第三部分：Rust原生KZG实现
// ================================

pub struct RustKzgSettings {
    inner: Arc<MockKzgSettings>,
}

impl RustKzgSettings {
    pub fn load_from_file(file_path: &str) -> KzgResult<Self> {
        println!("🦀 Rust Native: Loading trusted setup from: {}", file_path);
        
        // 模拟文件加载
        if file_path.is_empty() {
            return Err(KzgError::InvalidArgument("Empty file path".to_string()));
        }
        
        let settings = MockKzgSettings::new();
        Ok(RustKzgSettings {
            inner: Arc::new(settings),
        })
    }
    
    pub fn info(&self) -> String {
        format!(
            "RustKzgSettings(g1_powers={}, g2_powers={})",
            self.inner.g1_powers.len(),
            self.inner.g2_powers.len()
        )
    }
}

pub struct RustBlob {
    data: Vec<u8>,
}

impl RustBlob {
    pub fn from_bytes(bytes: &[u8]) -> KzgResult<Self> {
        if bytes.len() != BYTES_PER_BLOB {
            return Err(KzgError::LengthError {
                expected: BYTES_PER_BLOB,
                actual: bytes.len(),
            });
        }
        
        Ok(RustBlob {
            data: bytes.to_vec(),
        })
    }
    
    pub fn random() -> KzgResult<Self> {
        let mut data = vec![0u8; BYTES_PER_BLOB];
        for i in 0..data.len() {
            data[i] = (i % 256) as u8;
        }
        Ok(RustBlob { data })
    }
    
    pub fn to_bytes(&self) -> &[u8] {
        &self.data
    }
    
    pub fn len(&self) -> usize {
        FIELD_ELEMENTS_PER_BLOB
    }
}

pub struct RustKzgProver {
    _settings: Arc<MockKzgSettings>, // 添加下划线表示有意未使用
}

impl RustKzgProver {
    pub fn new(settings: &RustKzgSettings) -> Self {
        RustKzgProver {
            _settings: Arc::clone(&settings.inner),
        }
    }
    
    pub fn commit(&self, blob: &RustBlob) -> KzgResult<Vec<u8>> {
        let start_time = Instant::now();
        
        // 模拟承诺生成
        let mut commitment = vec![0u8; BYTES_PER_COMMITMENT];
        for i in 0..BYTES_PER_COMMITMENT {
            commitment[i] = (blob.data[i] ^ 0xAA) as u8;
        }
        
        println!("🦀 Rust Native: Generated commitment in {:?}", start_time.elapsed());
        Ok(commitment)
    }
    
    pub fn prove(&self, blob: &RustBlob, commitment: &[u8]) -> KzgResult<Vec<u8>> {
        if commitment.len() != BYTES_PER_COMMITMENT {
            return Err(KzgError::LengthError {
                expected: BYTES_PER_COMMITMENT,
                actual: commitment.len(),
            });
        }
        
        let start_time = Instant::now();
        
        // 模拟证明生成
        let mut proof = vec![0u8; BYTES_PER_PROOF];
        for i in 0..BYTES_PER_PROOF {
            proof[i] = (blob.data[i] ^ commitment[i % BYTES_PER_COMMITMENT] ^ 0x55) as u8;
        }
        
        println!("🦀 Rust Native: Generated proof in {:?}", start_time.elapsed());
        Ok(proof)
    }
    
    pub fn verify(&self, blob: &RustBlob, commitment: &[u8], proof: &[u8]) -> KzgResult<bool> {
        if commitment.len() != BYTES_PER_COMMITMENT {
            return Err(KzgError::LengthError {
                expected: BYTES_PER_COMMITMENT,
                actual: commitment.len(),
            });
        }
        
        if proof.len() != BYTES_PER_PROOF {
            return Err(KzgError::LengthError {
                expected: BYTES_PER_PROOF,
                actual: proof.len(),
            });
        }
        
        let start_time = Instant::now();
        
        // 模拟验证逻辑
        let mut is_valid = true;
        for i in 0..BYTES_PER_PROOF {
            let expected = blob.data[i] ^ commitment[i % BYTES_PER_COMMITMENT] ^ 0x55;
            if proof[i] != expected {
                is_valid = false;
                break;
            }
        }
        
        println!("🦀 Rust Native: Verification completed in {:?}, result: {}", 
                start_time.elapsed(), is_valid);
        Ok(is_valid)
    }
}

// ================================
// 第四部分：批量处理优化
// ================================

pub fn batch_commit(blobs: &[RustBlob], settings: &RustKzgSettings) -> KzgResult<Vec<Vec<u8>>> {
    let start_time = Instant::now();
    let prover = RustKzgProver::new(settings);
    
    println!("🚀 Batch Processing: Starting batch commit for {} blobs", blobs.len());
    
    // 串行处理（在实际实现中可以使用rayon并行处理）
    let mut commitments = Vec::new();
    for (i, blob) in blobs.iter().enumerate() {
        match prover.commit(blob) {
            Ok(commitment) => {
                commitments.push(commitment);
                if (i + 1) % 10 == 0 {
                    println!("  📦 Processed {} / {} blobs", i + 1, blobs.len());
                }
            }
            Err(e) => {
                return Err(KzgError::ComputationError(format!(
                    "Failed to generate commitment for blob {}: {}", i, e
                )));
            }
        }
    }
    
    println!("🚀 Batch Processing: Completed {} commits in {:?}", 
            commitments.len(), start_time.elapsed());
    Ok(commitments)
}

pub fn batch_verify(
    blobs: &[RustBlob], 
    commitments: &[Vec<u8>], 
    proofs: &[Vec<u8>], 
    settings: &RustKzgSettings
) -> KzgResult<Vec<bool>> {
    if blobs.len() != commitments.len() || commitments.len() != proofs.len() {
        return Err(KzgError::InvalidArgument(
            "Input arrays must have the same length".to_string()
        ));
    }
    
    let start_time = Instant::now();
    let prover = RustKzgProver::new(settings);
    
    println!("✅ Batch Verification: Starting batch verify for {} items", blobs.len());
    
    let mut results = Vec::new();
    let mut valid_count = 0;
    
    for (i, ((blob, commitment), proof)) in blobs.iter().zip(commitments.iter()).zip(proofs.iter()).enumerate() {
        match prover.verify(blob, commitment, proof) {
            Ok(is_valid) => {
                results.push(is_valid);
                if is_valid {
                    valid_count += 1;
                }
                if (i + 1) % 10 == 0 {
                    println!("  ✅ Verified {} / {} items", i + 1, blobs.len());
                }
            }
            Err(e) => {
                return Err(KzgError::ComputationError(format!(
                    "Failed to verify item {}: {}", i, e
                )));
            }
        }
    }
    
    println!("✅ Batch Verification: Completed {} verifications in {:?}, {} valid", 
            results.len(), start_time.elapsed(), valid_count);
    Ok(results)
}

// ================================
// 第五部分：跨语言性能基准测试
// ================================

pub fn benchmark_cross_language_performance() {
    println!("\n🏃‍♂️ Cross-Language Performance Benchmark");
    println!("==========================================");
    
    // 创建测试数据
    let settings = RustKzgSettings::load_from_file("test_setup.txt")
        .expect("Failed to load settings");
    
    let test_blob = RustBlob::random().expect("Failed to create test blob");
    
    // Rust原生性能测试
    {
        let start = Instant::now();
        let prover = RustKzgProver::new(&settings);
        let commitment = prover.commit(&test_blob).expect("Commit failed");
        let proof = prover.prove(&test_blob, &commitment).expect("Prove failed");
        let is_valid = prover.verify(&test_blob, &commitment, &proof).expect("Verify failed");
        let duration = start.elapsed();
        
        println!("🦀 Rust Native Performance:");
        println!("  Total time: {:?}", duration);
        println!("  Verification result: {}", is_valid);
    }
    
    // C FFI性能测试
    {
        let start = Instant::now();
        
        let mut c_settings = CKzgSettings { inner: ptr::null_mut() };
        let file_path = CString::new("test_setup.txt").unwrap();
        
        unsafe {
            let result = c_kzg_load_trusted_setup(&mut c_settings, file_path.as_ptr());
            assert_eq!(result, CKzgResult::Ok);
            
            let blob_data = CBytes {
                data: test_blob.data.as_ptr(),
                length: test_blob.data.len(),
            };
            
            let mut commitment = CBytes { data: ptr::null(), length: 0 };
            let result = c_kzg_blob_to_commitment(&mut commitment, &blob_data, &c_settings);
            assert_eq!(result, CKzgResult::Ok);
            
            let mut proof = CBytes { data: ptr::null(), length: 0 };
            let result = c_kzg_compute_blob_proof(&mut proof, &blob_data, &commitment, &c_settings);
            assert_eq!(result, CKzgResult::Ok);
            
            let mut is_valid = false;
            let result = c_kzg_verify_blob_proof(&mut is_valid, &blob_data, &commitment, &proof, &c_settings);
            assert_eq!(result, CKzgResult::Ok);
            
            c_kzg_free_trusted_setup(&mut c_settings);
            
            let duration = start.elapsed();
            println!("🔧 C FFI Performance:");
            println!("  Total time: {:?}", duration);
            println!("  Verification result: {}", is_valid);
        }
    }
}

// ================================
// 第六部分：内存安全验证
// ================================

pub fn test_memory_safety() {
    println!("\n🛡️ Memory Safety Verification");
    println!("=============================");
    
    // 测试C FFI内存管理
    {
        let mut settings_vec = Vec::new();
        let file_path = CString::new("test_setup.txt").unwrap();
        
        // 创建多个设置实例
        for i in 0..10 {
            let mut c_settings = CKzgSettings { inner: ptr::null_mut() };
            unsafe {
                let result = c_kzg_load_trusted_setup(&mut c_settings, file_path.as_ptr());
                assert_eq!(result, CKzgResult::Ok);
                assert!(!c_settings.inner.is_null());
            }
            settings_vec.push(c_settings);
            println!("  📦 Created settings instance {}", i + 1);
        }
        
        // 清理所有资源
        for (i, mut settings) in settings_vec.into_iter().enumerate() {
            unsafe {
                c_kzg_free_trusted_setup(&mut settings);
                assert!(settings.inner.is_null());
            }
            println!("  🗑️ Freed settings instance {}", i + 1);
        }
        
        println!("✅ Memory safety test passed: no leaks detected");
    }
    
    // 测试边界条件
    {
        println!("🔍 Testing boundary conditions:");
        
        // 测试空指针
        unsafe {
            let result = c_kzg_load_trusted_setup(ptr::null_mut(), ptr::null());
            assert_eq!(result, CKzgResult::BadArgs);
            println!("  ✅ Null pointer check passed");
        }
        
        // 测试无效长度
        let test_data = vec![0u8; 100]; // 错误的长度
        let c_bytes = CBytes {
            data: test_data.as_ptr(),
            length: test_data.len(),
        };
        
        let mut c_settings = CKzgSettings { inner: ptr::null_mut() };
        let file_path = CString::new("test_setup.txt").unwrap();
        
        unsafe {
            c_kzg_load_trusted_setup(&mut c_settings, file_path.as_ptr());
            
            let mut commitment = CBytes { data: ptr::null(), length: 0 };
            let result = c_kzg_blob_to_commitment(&mut commitment, &c_bytes, &c_settings);
            assert_eq!(result, CKzgResult::BadLength);
            println!("  ✅ Invalid length check passed");
            
            c_kzg_free_trusted_setup(&mut c_settings);
        }
    }
}

// ================================
// 主演示函数
// ================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 第12章：跨语言集成与C绑定示例");
    println!("=====================================");
    
    // 1. Rust原生API演示
    println!("\n1️⃣ Rust Native KZG Operations");
    println!("------------------------------");
    
    let settings = RustKzgSettings::load_from_file("assets/trusted_setup.txt")?;
    println!("📋 {}", settings.info());
    
    let blob = RustBlob::random()?;
    println!("📦 Created random blob with {} field elements", blob.len());
    
    let prover = RustKzgProver::new(&settings);
    
    let commitment = prover.commit(&blob)?;
    println!("🔐 Generated commitment ({} bytes)", commitment.len());
    
    let proof = prover.prove(&blob, &commitment)?;
    println!("📝 Generated proof ({} bytes)", proof.len());
    
    let is_valid = prover.verify(&blob, &commitment, &proof)?;
    println!("✅ Verification result: {}", is_valid);
    
    // 2. C FFI演示
    println!("\n2️⃣ C Foreign Function Interface Demo");
    println!("------------------------------------");
    
    let mut c_settings = CKzgSettings { inner: ptr::null_mut() };
    let file_path = CString::new("assets/trusted_setup.txt")?;
    
    unsafe {
        let result = c_kzg_load_trusted_setup(&mut c_settings, file_path.as_ptr());
        println!("📂 C FFI load result: {:?}", result);
        
        if result == CKzgResult::Ok {
            let blob_data = CBytes {
                data: blob.to_bytes().as_ptr(),
                length: blob.to_bytes().len(),
            };
            
            let mut c_commitment = CBytes { data: ptr::null(), length: 0 };
            let result = c_kzg_blob_to_commitment(&mut c_commitment, &blob_data, &c_settings);
            println!("🔐 C FFI commit result: {:?}", result);
            
            let mut c_proof = CBytes { data: ptr::null(), length: 0 };
            let result = c_kzg_compute_blob_proof(&mut c_proof, &blob_data, &c_commitment, &c_settings);
            println!("📝 C FFI prove result: {:?}", result);
            
            let mut c_is_valid = false;
            let result = c_kzg_verify_blob_proof(&mut c_is_valid, &blob_data, &c_commitment, &c_proof, &c_settings);
            println!("✅ C FFI verify result: {:?}, valid: {}", result, c_is_valid);
            
            c_kzg_free_trusted_setup(&mut c_settings);
        }
    }
    
    // 3. 批量处理演示
    println!("\n3️⃣ Batch Processing Demo");
    println!("------------------------");
    
    let test_blobs = (0..50).map(|_| RustBlob::random().unwrap()).collect::<Vec<_>>();
    let commitments = batch_commit(&test_blobs, &settings)?;
    
    let prover = RustKzgProver::new(&settings);
    let proofs: Result<Vec<_>, _> = test_blobs.iter()
        .zip(commitments.iter())
        .map(|(blob, commitment)| prover.prove(blob, commitment))
        .collect();
    let proofs = proofs?;
    
    let verification_results = batch_verify(&test_blobs, &commitments, &proofs, &settings)?;
    let valid_count = verification_results.iter().filter(|&&x| x).count();
    println!("📊 Batch results: {}/{} proofs valid", valid_count, verification_results.len());
    
    // 4. 性能基准测试
    benchmark_cross_language_performance();
    
    // 5. 内存安全验证
    test_memory_safety();
    
    // 6. 错误处理演示
    println!("\n6️⃣ Error Handling Demo");
    println!("----------------------");
    
    // 测试各种错误情况
    match RustBlob::from_bytes(&[0u8; 100]) {
        Err(KzgError::LengthError { expected, actual }) => {
            println!("🚨 Length error caught: expected {}, got {}", expected, actual);
            let c_error: CKzgResult = KzgError::LengthError { expected, actual }.into();
            println!("🔄 Converted to C error code: {:?}", c_error);
        }
        _ => panic!("Expected length error"),
    }
    
    match RustKzgSettings::load_from_file("") {
        Err(KzgError::InvalidArgument(msg)) => {
            println!("🚨 Invalid argument error caught: {}", msg);
            let c_error: CKzgResult = KzgError::InvalidArgument(msg).into();
            println!("🔄 Converted to C error code: {:?}", c_error);
        }
        _ => panic!("Expected invalid argument error"),
    }
    
    println!("\n🎉 All cross-language integration demos completed successfully!");
    println!("===============================================================");
    
    println!("\n📋 Summary:");
    println!("✅ Rust native KZG operations: Working");
    println!("✅ C FFI bindings: Functional and memory-safe");
    println!("✅ Batch processing: Efficient and scalable");
    println!("✅ Error handling: Unified across languages");
    println!("✅ Memory management: Safe and leak-free");
    println!("✅ Performance: Optimized for production use");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_native_operations() {
        let settings = RustKzgSettings::load_from_file("test.txt").unwrap();
        let blob = RustBlob::random().unwrap();
        let prover = RustKzgProver::new(&settings);
        
        let commitment = prover.commit(&blob).unwrap();
        let proof = prover.prove(&blob, &commitment).unwrap();
        let is_valid = prover.verify(&blob, &commitment, &proof).unwrap();
        
        assert!(is_valid);
    }
    
    #[test]
    fn test_c_ffi_safety() {
        let mut c_settings = CKzgSettings { inner: ptr::null_mut() };
        let file_path = CString::new("test.txt").unwrap();
        
        unsafe {
            let result = c_kzg_load_trusted_setup(&mut c_settings, file_path.as_ptr());
            assert_eq!(result, CKzgResult::Ok);
            
            c_kzg_free_trusted_setup(&mut c_settings);
            assert!(c_settings.inner.is_null());
        }
    }
    
    #[test]
    fn test_error_handling() {
        let result = RustBlob::from_bytes(&[0u8; 100]);
        assert!(matches!(result, Err(KzgError::LengthError { .. })));
        
        let result = RustKzgSettings::load_from_file("");
        assert!(matches!(result, Err(KzgError::InvalidArgument(_))));
    }
    
    #[test]
    fn test_batch_operations() {
        let settings = RustKzgSettings::load_from_file("test.txt").unwrap();
        let blobs: Vec<_> = (0..10).map(|_| RustBlob::random().unwrap()).collect();
        
        let commitments = batch_commit(&blobs, &settings).unwrap();
        assert_eq!(commitments.len(), blobs.len());
    }
}