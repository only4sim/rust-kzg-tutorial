# 第12章：跨语言集成与C绑定

> **学习目标**: 掌握 Rust KZG 库的跨语言集成技术，包括安全FFI设计、Python绑定、JavaScript WASM集成和多语言生态建设

---

## 12.1 跨语言集成架构概览

###  多语言生态的重要性

在现代软件开发中，密码学库需要支持多种编程语言，以满足不同项目的需求：

```rust
// Rust KZG 库的多语言绑定架构
┌─────────────────────────────────────┐
│           应用层                    │
│  Python Apps | C Apps | JS Apps    │
├─────────────────────────────────────┤
│           绑定层                    │
│   PyO3     |   FFI   |   WASM      │
├─────────────────────────────────────┤
│         Rust KZG 核心库             │
│    (rust-kzg-blst)                 │
├─────────────────────────────────────┤
│         底层密码学实现              │
│      (BLST, SPPARK)                │
└─────────────────────────────────────┘
```

###  跨语言集成的核心挑战

1. **内存安全**: 跨语言边界的内存管理
2. **类型转换**: 不同语言的类型系统映射
3. **错误处理**: 统一的错误传播机制
4. **性能优化**: 最小化跨语言调用开销
5. **API设计**: 符合目标语言习惯的接口设计

---

## 12.2 C语言FFI设计与实现

###  FFI (Foreign Function Interface) 基础

FFI是Rust与其他语言交互的基础技术。让我们从安全的C绑定开始：

```rust
// src/ffi.rs - C语言绑定接口
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use rust_kzg_blst::*;

/// C兼容的KZG设置结构
#[repr(C)]
pub struct CKzgSettings {
    inner: *mut KzgSettings,
}

/// C兼容的错误码定义
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum CKzgResult {
    Ok = 0,
    BadArgs = 1,
    Malloc = 2,
    BadEncoding = 3,
    BadLength = 4,
    Unknown = 5,
}

/// 受信任设置加载 - C接口
#[no_mangle]
pub extern "C" fn c_kzg_load_trusted_setup(
    out: *mut CKzgSettings,
    trusted_setup_file: *const c_char,
) -> CKzgResult {
    // 输入验证
    if out.is_null() || trusted_setup_file.is_null() {
        return CKzgResult::BadArgs;
    }
    
    // 安全的字符串转换
    let file_path = match unsafe { CStr::from_ptr(trusted_setup_file) }.to_str() {
        Ok(s) => s,
        Err(_) => return CKzgResult::BadEncoding,
    };
    
    // 加载受信任设置
    match load_trusted_setup_filename_rust(file_path) {
        Ok(settings) => {
            unsafe {
                (*out).inner = Box::into_raw(Box::new(settings));
            }
            CKzgResult::Ok
        }
        Err(_) => CKzgResult::Unknown,
    }
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
            }
        }
    }
}
```

###  内存安全保证

```rust
/// 安全的字节数组处理
#[repr(C)]
pub struct CBytes {
    data: *const u8,
    length: usize,
}

impl CBytes {
    /// 从Rust Vec创建C字节数组
    fn from_vec(mut vec: Vec<u8>) -> Self {
        let data = vec.as_ptr();
        let length = vec.len();
        std::mem::forget(vec); // 防止Rust释放内存
        CBytes { data, length }
    }
    
    /// 转换为Rust slice（借用，不获取所有权）
    unsafe fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(self.data, self.length)
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
    // 安全性检查
    if out.is_null() || blob.is_null() || settings.is_null() {
        return CKzgResult::BadArgs;
    }
    
    unsafe {
        // 获取输入数据
        let blob_slice = (*blob).as_slice();
        let settings_ref = &*(*settings).inner;
        
        // 验证blob长度
        if blob_slice.len() != BYTES_PER_BLOB {
            return CKzgResult::BadLength;
        }
        
        // 转换为Fr数组
        let blob_fr = match bytes_to_blob(blob_slice) {
            Ok(blob) => blob,
            Err(_) => return CKzgResult::BadEncoding,
        };
        
        // 生成承诺
        match blob_to_kzg_commitment_rust(&blob_fr, settings_ref) {
            Ok(commitment) => {
                let commitment_bytes = g1_to_bytes(&commitment);
                *out = CBytes::from_vec(commitment_bytes.to_vec());
                CKzgResult::Ok
            }
            Err(_) => CKzgResult::Unknown,
        }
    }
}
```

###  C头文件生成

```c
// include/rust_kzg.h - 自动生成的C头文件
#ifndef RUST_KZG_H
#define RUST_KZG_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// 类型定义
typedef struct CKzgSettings CKzgSettings;

typedef struct {
    const uint8_t* data;
    size_t length;
} CBytes;

typedef enum {
    C_KZG_OK = 0,
    C_KZG_BADARGS = 1,
    C_KZG_MALLOC = 2,
    C_KZG_BADENCODING = 3,
    C_KZG_BADLENGTH = 4,
    C_KZG_UNKNOWN = 5
} CKzgResult;

// 函数声明
CKzgResult c_kzg_load_trusted_setup(
    CKzgSettings* out,
    const char* trusted_setup_file
);

void c_kzg_free_trusted_setup(CKzgSettings* settings);

CKzgResult c_kzg_blob_to_commitment(
    CBytes* out,
    const CBytes* blob,
    const CKzgSettings* settings
);

CKzgResult c_kzg_compute_blob_proof(
    CBytes* out,
    const CBytes* blob,
    const CBytes* commitment,
    const CKzgSettings* settings
);

CKzgResult c_kzg_verify_blob_proof(
    bool* out,
    const CBytes* blob,
    const CBytes* commitment,
    const CBytes* proof,
    const CKzgSettings* settings
);

#ifdef __cplusplus
}
#endif

#endif // RUST_KZG_H
```

---

## 12.3 Python PyO3 绑定实现

###  PyO3 绑定架构

PyO3是Rust与Python交互的现代化框架，提供了安全高效的绑定机制：

```rust
// src/python.rs - Python绑定实现
use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use pyo3::types::PyBytes;
use rust_kzg_blst::*;
use std::sync::Arc;

/// Python可访问的KZG设置类
#[pyclass(name = "KzgSettings")]
pub struct PyKzgSettings {
    inner: Arc<KzgSettings>,
}

#[pymethods]
impl PyKzgSettings {
    /// 从文件加载受信任设置
    #[staticmethod]
    fn load_trusted_setup(file_path: &str) -> PyResult<Self> {
        match load_trusted_setup_filename_rust(file_path) {
            Ok(settings) => Ok(PyKzgSettings {
                inner: Arc::new(settings),
            }),
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Failed to load trusted setup: {}", e
            ))),
        }
    }
    
    /// 获取设置信息
    fn info(&self) -> PyResult<String> {
        Ok(format!(
            "KzgSettings(g1_count={}, g2_count={})",
            self.inner.fs.len(),
            self.inner.g2_monomial.len()
        ))
    }
}

/// Python可访问的Blob类
#[pyclass(name = "Blob")]
pub struct PyBlob {
    inner: Vec<Fr>,
}

#[pymethods]
impl PyBlob {
    /// 从字节数组创建Blob
    #[new]
    fn new(data: &PyBytes) -> PyResult<Self> {
        let bytes = data.as_bytes();
        if bytes.len() != BYTES_PER_BLOB {
            return Err(PyValueError::new_err(format!(
                "Blob must be exactly {} bytes", BYTES_PER_BLOB
            )));
        }
        
        match bytes_to_blob(bytes) {
            Ok(blob) => Ok(PyBlob { inner: blob }),
            Err(e) => Err(PyValueError::new_err(format!(
                "Invalid blob data: {}", e
            ))),
        }
    }
    
    /// 生成随机Blob（用于测试）
    #[staticmethod]
    fn random() -> PyResult<Self> {
        match create_random_blob() {
            Ok(blob) => Ok(PyBlob { inner: blob }),
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Failed to create random blob: {}", e
            ))),
        }
    }
    
    /// 转换为字节数组
    fn to_bytes(&self, py: Python) -> PyResult<PyObject> {
        let bytes = blob_to_bytes(&self.inner);
        Ok(PyBytes::new(py, &bytes).to_object(py))
    }
    
    /// 获取Blob大小
    fn __len__(&self) -> usize {
        FIELD_ELEMENTS_PER_BLOB
    }
    
    /// 字符串表示
    fn __repr__(&self) -> String {
        format!("Blob(size={})", FIELD_ELEMENTS_PER_BLOB)
    }
}

/// KZG操作类
#[pyclass(name = "KzgProver")]
pub struct PyKzgProver {
    settings: Arc<KzgSettings>,
}

#[pymethods]
impl PyKzgProver {
    /// 创建证明器
    #[new]
    fn new(settings: &PyKzgSettings) -> Self {
        PyKzgProver {
            settings: Arc::clone(&settings.inner),
        }
    }
    
    /// 生成承诺
    fn commit(&self, py: Python, blob: &PyBlob) -> PyResult<PyObject> {
        match blob_to_kzg_commitment_rust(&blob.inner, &self.settings) {
            Ok(commitment) => {
                let bytes = g1_to_bytes(&commitment);
                Ok(PyBytes::new(py, &bytes).to_object(py))
            }
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Failed to generate commitment: {}", e
            ))),
        }
    }
    
    /// 生成证明
    fn prove(&self, py: Python, blob: &PyBlob, commitment: &PyBytes) -> PyResult<PyObject> {
        let commitment_bytes = commitment.as_bytes();
        if commitment_bytes.len() != BYTES_PER_COMMITMENT {
            return Err(PyValueError::new_err("Invalid commitment length"));
        }
        
        let commitment_g1 = match bytes_to_g1(commitment_bytes) {
            Ok(g1) => g1,
            Err(e) => return Err(PyValueError::new_err(format!(
                "Invalid commitment: {}", e
            ))),
        };
        
        match compute_blob_kzg_proof_rust(&blob.inner, &commitment_g1, &self.settings) {
            Ok(proof) => {
                let bytes = g1_to_bytes(&proof);
                Ok(PyBytes::new(py, &bytes).to_object(py))
            }
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Failed to generate proof: {}", e
            ))),
        }
    }
    
    /// 验证证明
    fn verify(
        &self,
        blob: &PyBlob,
        commitment: &PyBytes,
        proof: &PyBytes,
    ) -> PyResult<bool> {
        let commitment_bytes = commitment.as_bytes();
        let proof_bytes = proof.as_bytes();
        
        if commitment_bytes.len() != BYTES_PER_COMMITMENT {
            return Err(PyValueError::new_err("Invalid commitment length"));
        }
        if proof_bytes.len() != BYTES_PER_PROOF {
            return Err(PyValueError::new_err("Invalid proof length"));
        }
        
        let commitment_g1 = match bytes_to_g1(commitment_bytes) {
            Ok(g1) => g1,
            Err(e) => return Err(PyValueError::new_err(format!(
                "Invalid commitment: {}", e
            ))),
        };
        
        let proof_g1 = match bytes_to_g1(proof_bytes) {
            Ok(g1) => g1,
            Err(e) => return Err(PyValueError::new_err(format!(
                "Invalid proof: {}", e
            ))),
        };
        
        match verify_blob_kzg_proof_rust(&blob.inner, &commitment_g1, &proof_g1, &self.settings) {
            Ok(is_valid) => Ok(is_valid),
            Err(e) => Err(PyRuntimeError::new_err(format!(
                "Verification failed: {}", e
            ))),
        }
    }
}

/// 批量操作支持
#[pyfunction]
fn batch_commit(
    py: Python,
    blobs: Vec<PyRef<PyBlob>>,
    settings: &PyKzgSettings,
) -> PyResult<Vec<PyObject>> {
    let blob_data: Vec<_> = blobs.iter().map(|b| &b.inner).collect();
    
    // 使用并行处理提升性能
    use rayon::prelude::*;
    let results: Result<Vec<_>, _> = blob_data
        .par_iter()
        .map(|blob| blob_to_kzg_commitment_rust(blob, &settings.inner))
        .collect();
    
    match results {
        Ok(commitments) => {
            let py_commitments: Vec<PyObject> = commitments
                .iter()
                .map(|c| {
                    let bytes = g1_to_bytes(c);
                    PyBytes::new(py, &bytes).to_object(py)
                })
                .collect();
            Ok(py_commitments)
        }
        Err(e) => Err(PyRuntimeError::new_err(format!(
            "Batch commit failed: {}", e
        ))),
    }
}

/// Python模块定义
#[pymodule]
fn rust_kzg(py: Python, m: &PyModule) -> PyResult<()> {
    // 添加类
    m.add_class::<PyKzgSettings>()?;
    m.add_class::<PyBlob>()?;
    m.add_class::<PyKzgProver>()?;
    
    // 添加函数
    m.add_function(wrap_pyfunction!(batch_commit, m)?)?;
    
    // 添加常量
    m.add("BYTES_PER_BLOB", BYTES_PER_BLOB)?;
    m.add("BYTES_PER_COMMITMENT", BYTES_PER_COMMITMENT)?;
    m.add("BYTES_PER_PROOF", BYTES_PER_PROOF)?;
    m.add("FIELD_ELEMENTS_PER_BLOB", FIELD_ELEMENTS_PER_BLOB)?;
    
    Ok(())
}
```

###  Python使用示例

```python
# examples/python_example.py
import rust_kzg

def main():
    # 加载受信任设置
    settings = rust_kzg.KzgSettings.load_trusted_setup("assets/trusted_setup.txt")
    print(f"Loaded settings: {settings.info()}")
    
    # 创建证明器
    prover = rust_kzg.KzgProver(settings)
    
    # 生成随机Blob
    blob = rust_kzg.Blob.random()
    print(f"Created blob: {blob}")
    
    # 生成承诺
    commitment = prover.commit(blob)
    print(f"Commitment: {commitment.hex()}")
    
    # 生成证明
    proof = prover.prove(blob, commitment)
    print(f"Proof: {proof.hex()}")
    
    # 验证证明
    is_valid = prover.verify(blob, commitment, proof)
    print(f"Verification result: {is_valid}")
    
    # 批量处理示例
    blobs = [rust_kzg.Blob.random() for _ in range(10)]
    commitments = rust_kzg.batch_commit(blobs, settings)
    print(f"Batch processed {len(commitments)} commitments")

if __name__ == "__main__":
    main()
```

---

## 12.4 JavaScript WASM 集成

###  WebAssembly 编译配置

WASM使Rust代码能在浏览器和Node.js中运行：

```toml
# Cargo.toml - WASM配置
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = "0.3"
js-sys = "0.3"
wee_alloc = { version = "0.4", optional = true }
console_error_panic_hook = { version = "0.1", optional = true }

[dependencies.rust-kzg-blst]
path = "../rust-kzg/blst"
default-features = false
features = ["wasm"]

[features]
default = ["console_error_panic_hook"]
```

```rust
// src/wasm.rs - WASM绑定实现
use wasm_bindgen::prelude::*;
use js_sys::{Array, Uint8Array};
use web_sys::console;
use rust_kzg_blst::*;
use std::sync::Arc;

// 配置WASM运行时
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// 设置panic钩子（用于调试）
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// JavaScript可访问的KZG设置类
#[wasm_bindgen]
pub struct WasmKzgSettings {
    inner: Arc<KzgSettings>,
}

#[wasm_bindgen]
impl WasmKzgSettings {
    /// 从受信任设置数据创建设置
    #[wasm_bindgen(constructor)]
    pub fn new(trusted_setup_data: &[u8]) -> Result<WasmKzgSettings, JsValue> {
        set_panic_hook();
        
        // 在WASM环境中，我们需要从内存中的数据加载
        match load_trusted_setup_rust(trusted_setup_data) {
            Ok(settings) => Ok(WasmKzgSettings {
                inner: Arc::new(settings),
            }),
            Err(e) => Err(JsValue::from_str(&format!("Failed to load setup: {}", e))),
        }
    }
    
    /// 获取设置信息
    #[wasm_bindgen(getter)]
    pub fn info(&self) -> String {
        format!(
            "KzgSettings(g1_count={}, g2_count={})",
            self.inner.fs.len(),
            self.inner.g2_monomial.len()
        )
    }
}

/// JavaScript可访问的Blob类
#[wasm_bindgen]
pub struct WasmBlob {
    inner: Vec<Fr>,
}

#[wasm_bindgen]
impl WasmBlob {
    /// 从Uint8Array创建Blob
    #[wasm_bindgen(constructor)]
    pub fn new(data: &Uint8Array) -> Result<WasmBlob, JsValue> {
        let bytes = data.to_vec();
        if bytes.len() != BYTES_PER_BLOB {
            return Err(JsValue::from_str(&format!(
                "Blob must be exactly {} bytes", BYTES_PER_BLOB
            )));
        }
        
        match bytes_to_blob(&bytes) {
            Ok(blob) => Ok(WasmBlob { inner: blob }),
            Err(e) => Err(JsValue::from_str(&format!("Invalid blob data: {}", e))),
        }
    }
    
    /// 生成随机Blob
    #[wasm_bindgen(js_name = "random")]
    pub fn random() -> Result<WasmBlob, JsValue> {
        match create_random_blob() {
            Ok(blob) => Ok(WasmBlob { inner: blob }),
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to create random blob: {}", e
            ))),
        }
    }
    
    /// 转换为Uint8Array
    #[wasm_bindgen(js_name = "toBytes")]
    pub fn to_bytes(&self) -> Uint8Array {
        let bytes = blob_to_bytes(&self.inner);
        Uint8Array::from(&bytes[..])
    }
    
    /// 获取大小
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        FIELD_ELEMENTS_PER_BLOB
    }
}

/// KZG证明器
#[wasm_bindgen]
pub struct WasmKzgProver {
    settings: Arc<KzgSettings>,
}

#[wasm_bindgen]
impl WasmKzgProver {
    /// 创建证明器
    #[wasm_bindgen(constructor)]
    pub fn new(settings: &WasmKzgSettings) -> WasmKzgProver {
        WasmKzgProver {
            settings: Arc::clone(&settings.inner),
        }
    }
    
    /// 生成承诺
    #[wasm_bindgen]
    pub fn commit(&self, blob: &WasmBlob) -> Result<Uint8Array, JsValue> {
        match blob_to_kzg_commitment_rust(&blob.inner, &self.settings) {
            Ok(commitment) => {
                let bytes = g1_to_bytes(&commitment);
                Ok(Uint8Array::from(&bytes[..]))
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to generate commitment: {}", e
            ))),
        }
    }
    
    /// 生成证明
    #[wasm_bindgen]
    pub fn prove(&self, blob: &WasmBlob, commitment: &Uint8Array) -> Result<Uint8Array, JsValue> {
        let commitment_bytes = commitment.to_vec();
        if commitment_bytes.len() != BYTES_PER_COMMITMENT {
            return Err(JsValue::from_str("Invalid commitment length"));
        }
        
        let commitment_g1 = match bytes_to_g1(&commitment_bytes) {
            Ok(g1) => g1,
            Err(e) => return Err(JsValue::from_str(&format!("Invalid commitment: {}", e))),
        };
        
        match compute_blob_kzg_proof_rust(&blob.inner, &commitment_g1, &self.settings) {
            Ok(proof) => {
                let bytes = g1_to_bytes(&proof);
                Ok(Uint8Array::from(&bytes[..]))
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to generate proof: {}", e
            ))),
        }
    }
    
    /// 验证证明
    #[wasm_bindgen]
    pub fn verify(
        &self,
        blob: &WasmBlob,
        commitment: &Uint8Array,
        proof: &Uint8Array,
    ) -> Result<bool, JsValue> {
        let commitment_bytes = commitment.to_vec();
        let proof_bytes = proof.to_vec();
        
        if commitment_bytes.len() != BYTES_PER_COMMITMENT {
            return Err(JsValue::from_str("Invalid commitment length"));
        }
        if proof_bytes.len() != BYTES_PER_PROOF {
            return Err(JsValue::from_str("Invalid proof length"));
        }
        
        let commitment_g1 = match bytes_to_g1(&commitment_bytes) {
            Ok(g1) => g1,
            Err(e) => return Err(JsValue::from_str(&format!("Invalid commitment: {}", e))),
        };
        
        let proof_g1 = match bytes_to_g1(&proof_bytes) {
            Ok(g1) => g1,
            Err(e) => return Err(JsValue::from_str(&format!("Invalid proof: {}", e))),
        };
        
        match verify_blob_kzg_proof_rust(&blob.inner, &commitment_g1, &proof_g1, &self.settings) {
            Ok(is_valid) => Ok(is_valid),
            Err(e) => Err(JsValue::from_str(&format!("Verification failed: {}", e))),
        }
    }
}

/// 批量处理支持
#[wasm_bindgen]
pub fn batch_commit(
    blobs: &Array,
    settings: &WasmKzgSettings,
) -> Result<Array, JsValue> {
    let rust_blobs: Result<Vec<_>, _> = blobs
        .iter()
        .map(|blob_js| {
            blob_js
                .dyn_into::<WasmBlob>()
                .map_err(|_| JsValue::from_str("Invalid blob in array"))
        })
        .collect();
    
    let rust_blobs = rust_blobs?;
    let blob_data: Vec<_> = rust_blobs.iter().map(|b| &b.inner).collect();
    
    // 使用并行处理（在WASM中可能需要特殊处理）
    let results: Result<Vec<_>, _> = blob_data
        .iter()
        .map(|blob| blob_to_kzg_commitment_rust(blob, &settings.inner))
        .collect();
    
    match results {
        Ok(commitments) => {
            let js_array = Array::new();
            for commitment in commitments {
                let bytes = g1_to_bytes(&commitment);
                let uint8_array = Uint8Array::from(&bytes[..]);
                js_array.push(&uint8_array.into());
            }
            Ok(js_array)
        }
        Err(e) => Err(JsValue::from_str(&format!("Batch commit failed: {}", e))),
    }
}

/// 导出常量
#[wasm_bindgen]
pub fn bytes_per_blob() -> usize { BYTES_PER_BLOB }

#[wasm_bindgen]
pub fn bytes_per_commitment() -> usize { BYTES_PER_COMMITMENT }

#[wasm_bindgen]
pub fn bytes_per_proof() -> usize { BYTES_PER_PROOF }

#[wasm_bindgen]
pub fn field_elements_per_blob() -> usize { FIELD_ELEMENTS_PER_BLOB }
```

###  JavaScript使用示例

```javascript
// examples/javascript_example.js
import init, {
    WasmKzgSettings,
    WasmBlob,
    WasmKzgProver,
    batch_commit,
    bytes_per_blob,
    bytes_per_commitment,
    bytes_per_proof
} from './pkg/rust_kzg_wasm.js';

async function loadTrustedSetup() {
    // 在实际应用中，你需要加载真实的受信任设置数据
    const response = await fetch('assets/trusted_setup.txt');
    const data = new Uint8Array(await response.arrayBuffer());
    return data;
}

async function main() {
    // 初始化WASM模块
    await init();
    
    console.log(`Constants:
        BYTES_PER_BLOB: ${bytes_per_blob()}
        BYTES_PER_COMMITMENT: ${bytes_per_commitment()}
        BYTES_PER_PROOF: ${bytes_per_proof()}`);
    
    try {
        // 加载受信任设置
        const trustedSetupData = await loadTrustedSetup();
        const settings = new WasmKzgSettings(trustedSetupData);
        console.log(`Loaded settings: ${settings.info}`);
        
        // 创建证明器
        const prover = new WasmKzgProver(settings);
        
        // 生成随机Blob
        const blob = WasmBlob.random();
        console.log(`Created blob with length: ${blob.length}`);
        
        // 生成承诺
        const commitment = prover.commit(blob);
        console.log(`Commitment (${commitment.length} bytes):`, 
                   Array.from(commitment).map(b => b.toString(16).padStart(2, '0')).join(''));
        
        // 生成证明
        const proof = prover.prove(blob, commitment);
        console.log(`Proof (${proof.length} bytes):`,
                   Array.from(proof).map(b => b.toString(16).padStart(2, '0')).join(''));
        
        // 验证证明
        const isValid = prover.verify(blob, commitment, proof);
        console.log(`Verification result: ${isValid}`);
        
        // 批量处理示例
        const blobs = [];
        for (let i = 0; i < 5; i++) {
            blobs.push(WasmBlob.random());
        }
        
        const commitments = batch_commit(blobs, settings);
        console.log(`Batch processed ${commitments.length} commitments`);
        
    } catch (error) {
        console.error('Error:', error);
    }
}

// 在浏览器中运行
if (typeof window !== 'undefined') {
    window.addEventListener('DOMContentLoaded', main);
} else {
    // 在Node.js中运行
    main().catch(console.error);
}
```

---

## 12.5 统一错误处理策略

###  跨语言错误处理设计

不同语言有不同的错误处理机制，我们需要设计统一的策略：

```rust
// src/error.rs - 统一错误处理
use std::fmt;

/// 跨语言兼容的错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum KzgError {
    /// 参数错误
    InvalidArgument(String),
    /// 编码错误
    EncodingError(String),
    /// 长度错误
    LengthError { expected: usize, actual: usize },
    /// 计算错误
    ComputationError(String),
    /// 内存错误
    MemoryError(String),
    /// 未知错误
    Unknown(String),
}

impl fmt::Display for KzgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// 统一的结果类型
pub type KzgResult<T> = Result<T, KzgError>;

/// 错误码枚举（用于C接口）
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KzgErrorCode {
    Ok = 0,
    InvalidArgument = 1,
    EncodingError = 2,
    LengthError = 3,
    ComputationError = 4,
    MemoryError = 5,
    Unknown = 99,
}

impl From<KzgError> for KzgErrorCode {
    fn from(error: KzgError) -> Self {
        match error {
            KzgError::InvalidArgument(_) => KzgErrorCode::InvalidArgument,
            KzgError::EncodingError(_) => KzgErrorCode::EncodingError,
            KzgError::LengthError { .. } => KzgErrorCode::LengthError,
            KzgError::ComputationError(_) => KzgErrorCode::ComputationError,
            KzgError::MemoryError(_) => KzgErrorCode::MemoryError,
            KzgError::Unknown(_) => KzgErrorCode::Unknown,
        }
    }
}

/// Python错误转换
#[cfg(feature = "python")]
impl From<KzgError> for pyo3::PyErr {
    fn from(error: KzgError) -> Self {
        use pyo3::exceptions::*;
        match error {
            KzgError::InvalidArgument(msg) => PyValueError::new_err(msg),
            KzgError::EncodingError(msg) => PyValueError::new_err(msg),
            KzgError::LengthError { expected, actual } => PyValueError::new_err(
                format!("Length error: expected {}, got {}", expected, actual)
            ),
            KzgError::ComputationError(msg) => PyRuntimeError::new_err(msg),
            KzgError::MemoryError(msg) => PyMemoryError::new_err(msg),
            KzgError::Unknown(msg) => PyRuntimeError::new_err(msg),
        }
    }
}

/// JavaScript错误转换
#[cfg(feature = "wasm")]
impl From<KzgError> for wasm_bindgen::JsValue {
    fn from(error: KzgError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}
```

---

## 12.6 性能基准与优化

###  跨语言性能对比

不同绑定方式的性能特点：

```rust
// benches/cross_language_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_kzg_blst::*;
use std::time::Duration;

fn bench_native_rust(c: &mut Criterion) {
    let settings = load_trusted_setup_filename_rust("assets/trusted_setup.txt")
        .expect("Failed to load trusted setup");
    let blob = create_random_blob().expect("Failed to create blob");
    
    c.bench_function("native_rust_commit", |b| {
        b.iter(|| {
            blob_to_kzg_commitment_rust(&blob, &settings)
        })
    });
}

fn bench_c_ffi_overhead(c: &mut Criterion) {
    // 测量FFI调用开销
    let mut group = c.benchmark_group("ffi_overhead");
    
    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("array_passing", size), size, |b, &size| {
            let data: Vec<u8> = (0..size).map(|i| i as u8).collect();
            b.iter(|| {
                // 模拟FFI数组传递开销
                unsafe {
                    let ptr = data.as_ptr();
                    let len = data.len();
                    // FFI调用模拟
                    std::ptr::read_volatile(&ptr);
                    std::ptr::read_volatile(&len);
                }
            });
        });
    }
    group.finish();
}

#[cfg(feature = "python")]
fn bench_python_binding(c: &mut Criterion) {
    use pyo3::prelude::*;
    
    c.bench_function("python_binding_overhead", |b| {
        b.iter(|| {
            Python::with_gil(|py| {
                // 测量Python绑定的开销
                let module = PyModule::from_code(
                    py,
                    r#"
import rust_kzg
settings = rust_kzg.KzgSettings.load_trusted_setup("assets/trusted_setup.txt")
blob = rust_kzg.Blob.random()
prover = rust_kzg.KzgProver(settings)
commitment = prover.commit(blob)
"#,
                    "test_module.py",
                    "test_module",
                ).unwrap();
                module
            })
        })
    });
}

criterion_group!(
    benches,
    bench_native_rust,
    bench_c_ffi_overhead,
    #[cfg(feature = "python")]
    bench_python_binding
);
criterion_main!(benches);
```

###  性能优化建议

| 绑定类型 | 性能开销 | 优化策略 |
|----------|----------|----------|
| **C FFI** | ~5-10% | 批量操作、减少调用次数 |
| **Python PyO3** | ~20-30% | 使用numpy、批量处理 |
| **JavaScript WASM** | ~10-20% | 异步处理、内存复用 |

---

## 12.7 实际应用案例

###  多语言生态示例

```python
# Python科学计算集成
import numpy as np
import rust_kzg

def kzg_batch_analysis(data_matrix):
    """使用KZG对数据矩阵进行批量分析"""
    settings = rust_kzg.KzgSettings.load_trusted_setup("setup.txt")
    prover = rust_kzg.KzgProver(settings)
    
    # 将numpy数组转换为Blob
    blobs = []
    for row in data_matrix:
        # 填充到正确大小
        padded_data = np.pad(row, (0, rust_kzg.FIELD_ELEMENTS_PER_BLOB - len(row)))
        blob_bytes = padded_data.tobytes()
        blobs.append(rust_kzg.Blob(blob_bytes))
    
    # 批量生成承诺
    commitments = rust_kzg.batch_commit(blobs, settings)
    return commitments
```

```javascript
// JavaScript Web应用集成
class KzgManager {
    constructor() {
        this.settings = null;
        this.prover = null;
        this.isInitialized = false;
    }
    
    async initialize(trustedSetupUrl) {
        try {
            await init(); // 初始化WASM
            
            const response = await fetch(trustedSetupUrl);
            const setupData = new Uint8Array(await response.arrayBuffer());
            
            this.settings = new WasmKzgSettings(setupData);
            this.prover = new WasmKzgProver(this.settings);
            this.isInitialized = true;
            
            console.log('KZG Manager initialized');
        } catch (error) {
            console.error('Failed to initialize KZG Manager:', error);
            throw error;
        }
    }
    
    async processFileBlob(file) {
        if (!this.isInitialized) {
            throw new Error('KZG Manager not initialized');
        }
        
        const arrayBuffer = await file.arrayBuffer();
        const data = new Uint8Array(arrayBuffer);
        
        // 处理大文件：分块处理
        const chunkSize = bytes_per_blob();
        const chunks = [];
        
        for (let offset = 0; offset < data.length; offset += chunkSize) {
            const chunk = data.slice(offset, offset + chunkSize);
            if (chunk.length < chunkSize) {
                // 填充最后一个块
                const padded = new Uint8Array(chunkSize);
                padded.set(chunk);
                chunks.push(padded);
            } else {
                chunks.push(chunk);
            }
        }
        
        // 批量处理
        const blobs = chunks.map(chunk => new WasmBlob(chunk));
        const commitments = batch_commit(blobs, this.settings);
        
        return {
            chunks: chunks.length,
            commitments: Array.from(commitments)
        };
    }
}

// 使用示例
const kzgManager = new KzgManager();
await kzgManager.initialize('/assets/trusted_setup.txt');

document.getElementById('file-input').addEventListener('change', async (event) => {
    const file = event.target.files[0];
    if (file) {
        try {
            const result = await kzgManager.processFileBlob(file);
            console.log(`Processed ${result.chunks} chunks, generated ${result.commitments.length} commitments`);
        } catch (error) {
            console.error('Error processing file:', error);
        }
    }
});
```

---

## 12.8 测试与验证

###  跨语言一致性测试

```rust
// tests/cross_language_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    
    #[test]
    fn test_c_ffi_consistency() {
        // 编译C测试程序
        let output = Command::new("gcc")
            .args(&[
                "-o", "test_c_ffi",
                "tests/c_test.c",
                "-L", "target/release",
                "-lrust_kzg_ffi"
            ])
            .output()
            .expect("Failed to compile C test");
        
        assert!(output.status.success(), "C compilation failed");
        
        // 运行C测试
        let output = Command::new("./test_c_ffi")
            .output()
            .expect("Failed to run C test");
        
        assert!(output.status.success(), "C test failed");
    }
    
    #[test]
    #[cfg(feature = "python")]
    fn test_python_consistency() {
        use pyo3::prelude::*;
        
        Python::with_gil(|py| {
            let result = py.run(
                r#"
import rust_kzg
import sys

# 测试基本功能
settings = rust_kzg.KzgSettings.load_trusted_setup("assets/trusted_setup.txt")
blob = rust_kzg.Blob.random()
prover = rust_kzg.KzgProver(settings)

commitment = prover.commit(blob)
proof = prover.prove(blob, commitment)
is_valid = prover.verify(blob, commitment, proof)

assert is_valid, "Python verification failed"
print("Python test passed")
"#,
                None,
                None,
            );
            
            assert!(result.is_ok(), "Python test execution failed");
        });
    }
    
    #[test]
    #[cfg(feature = "wasm")]
    fn test_wasm_consistency() {
        // WASM测试需要在浏览器或Node.js环境中运行
        // 这里我们模拟基本的功能测试
        let settings_data = std::fs::read("assets/trusted_setup.txt")
            .expect("Failed to read trusted setup");
        
        let settings = WasmKzgSettings::new(&settings_data)
            .expect("Failed to create WASM settings");
        
        let blob = WasmBlob::random()
            .expect("Failed to create random blob");
        
        let prover = WasmKzgProver::new(&settings);
        
        let commitment = prover.commit(&blob)
            .expect("Failed to generate commitment");
        
        let proof = prover.prove(&blob, &commitment)
            .expect("Failed to generate proof");
        
        let is_valid = prover.verify(&blob, &commitment, &proof)
            .expect("Failed to verify proof");
        
        assert!(is_valid, "WASM verification failed");
    }
}
```

---

## 12.9 部署与分发

###  构建配置

```toml
# build.rs - 构建脚本
use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // C绑定构建
    if cfg!(feature = "c-bindings") {
        build_c_bindings(&target, &out_dir);
    }
    
    // Python绑定构建
    if cfg!(feature = "python") {
        build_python_bindings(&target, &out_dir);
    }
    
    // WASM构建
    if target.contains("wasm32") {
        build_wasm(&out_dir);
    }
}

fn build_c_bindings(target: &str, out_dir: &PathBuf) {
    // 生成C头文件
    let bindings = cbindgen::Builder::new()
        .with_crate(".")
        .with_language(cbindgen::Language::C)
        .with_include_guard("RUST_KZG_H")
        .generate()
        .expect("Unable to generate C bindings");
    
    let header_path = out_dir.join("rust_kzg.h");
    bindings.write_to_file(&header_path);
    
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,librust_kzg.so");
}

fn build_python_bindings(target: &str, out_dir: &PathBuf) {
    // PyO3构建配置
    pyo3_build_config::add_extension_module_link_args();
}

fn build_wasm(out_dir: &PathBuf) {
    // WASM特定优化
    println!("cargo:rustc-link-arg=--max-memory=134217728"); // 128MB
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=stack-size=1048576"); // 1MB stack
}
```

###  CI/CD 配置

```yaml
# .github/workflows/cross_language.yml
name: Cross Language Integration

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  c-bindings:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Build C bindings
      run: cargo build --release --features c-bindings
      
    - name: Test C integration
      run: |
        gcc -o test_c tests/c_test.c -L target/release -lrust_kzg_ffi
        ./test_c

  python-bindings:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: [3.8, 3.9, "3.10", "3.11"]
    steps:
    - uses: actions/checkout@v3
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v4
      with:
        python-version: ${{ matrix.python-version }}
        
    - name: Install dependencies
      run: |
        pip install maturin pytest numpy
        
    - name: Build Python bindings
      run: maturin develop --features python
      
    - name: Test Python integration
      run: pytest tests/test_python.py

  wasm-bindings:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Install Rust and wasm-pack
      run: |
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
        
    - name: Build WASM bindings
      run: wasm-pack build --target web --features wasm
      
    - name: Test WASM in Node.js
      run: |
        cd pkg
        npm install
        node ../tests/node_test.js
```

---

## 12.10 最佳实践总结

###  跨语言集成的关键原则

1. **安全第一**: 严格的输入验证和内存管理
2. **性能优化**: 批量操作减少跨语言调用开销
3. **错误处理**: 统一的错误类型和转换机制
4. **API设计**: 符合目标语言的惯用法
5. **测试覆盖**: 完整的跨语言一致性测试

###  安全考虑

- **内存安全**: 使用RAII模式管理资源
- **输入验证**: 在边界进行严格检查
- **错误传播**: 不允许跨语言panic
- **版本兼容**: 稳定的ABI设计

###  性能优化

- **批量处理**: 减少单次调用开销
- **内存复用**: 避免频繁分配/释放
- **异步支持**: 非阻塞的长时间操作
- **缓存策略**: 重用计算结果

---

##  本章总结

通过本章学习，你掌握了：

1. **FFI设计模式**: 安全高效的C语言绑定实现
2. **PyO3集成**: 现代化的Python绑定开发
3. **WASM编译**: 浏览器和Node.js环境支持
4. **错误处理统一**: 跨语言的错误传播机制
5. **性能优化**: 最小化跨语言调用开销
6. **测试策略**: 确保多语言一致性的验证方法
7. **部署实践**: 完整的构建和分发流程

###  下一步学习

- **第13章**: 性能分析与调优技术
- **第14章**: 安全性分析与加固
- **第15章**: 自定义后端实现指南

通过跨语言集成，Rust KZG库能够服务于更广泛的开发者社区，在不同的技术栈中发挥重要作用！

---

** 恭喜完成第12章！你已经掌握了现代密码学库的跨语言集成核心技术！**