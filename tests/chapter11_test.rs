// 第11章单元测试文件
// 测试高级API功能的正确性

use std::sync::Arc;
    use std::time::Duration;

    // 模拟的KZG类型，与chapter11_advanced_api.rs保持一致
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct Fr([u8; 32]);

    impl Fr {
        pub fn zero() -> Self {
            Self([0u8; 32])
        }
        
        pub fn one() -> Self {
            let mut bytes = [0u8; 32];
            bytes[31] = 1;
            Self(bytes)
        }
        
        pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
            if bytes.len() != 32 {
                return Err("Invalid byte length".to_string());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(bytes);
            Ok(Self(arr))
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct G1([u8; 48]);

    impl G1 {
        pub fn zero() -> Self {
            Self([0u8; 48])
        }
        
        pub fn generator() -> Self {
            let mut bytes = [0u8; 48];
            bytes[47] = 1;
            Self(bytes)
        }
    }

    #[derive(Debug)]
    pub struct MockKzgSettings {
        pub setup_size: usize,
    }

    impl MockKzgSettings {
        pub fn new() -> Self {
            Self {
                setup_size: 4096,
            }
        }
    }

    // 简化的批量处理器用于测试
    pub struct TestBatchProcessor {
        settings: Arc<MockKzgSettings>,
        chunk_size: usize,
    }

    impl TestBatchProcessor {
        pub fn new(settings: Arc<MockKzgSettings>) -> Self {
            Self {
                settings,
                chunk_size: 64,
            }
        }
        
        pub fn with_chunk_size(mut self, size: usize) -> Self {
            self.chunk_size = size;
            self
        }
        
        pub fn batch_commitments(&self, blobs: &[Vec<Fr>]) -> Result<Vec<G1>, String> {
            if blobs.is_empty() {
                return Ok(Vec::new());
            }
            
            // 模拟批量处理
            let mut commitments = Vec::new();
            for blob in blobs {
                if blob.is_empty() {
                    return Err("Empty blob".to_string());
                }
                commitments.push(G1::generator());
            }
            
            Ok(commitments)
        }
    }

    #[test]
    fn test_batch_processor_creation() {
        let settings = Arc::new(MockKzgSettings::new());
        let processor = TestBatchProcessor::new(settings);
        assert_eq!(processor.chunk_size, 64);
    }

    #[test]
    fn test_batch_processor_with_chunk_size() {
        let settings = Arc::new(MockKzgSettings::new());
        let processor = TestBatchProcessor::new(settings).with_chunk_size(32);
        assert_eq!(processor.chunk_size, 32);
    }

    #[test]
    fn test_batch_commitments_empty_input() {
        let settings = Arc::new(MockKzgSettings::new());
        let processor = TestBatchProcessor::new(settings);
        
        let result = processor.batch_commitments(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_batch_commitments_normal_input() {
        let settings = Arc::new(MockKzgSettings::new());
        let processor = TestBatchProcessor::new(settings);
        
        let blobs = vec![
            vec![Fr::one(); 4096],
            vec![Fr::zero(); 4096],
            vec![Fr::one(); 4096],
        ];
        
        let result = processor.batch_commitments(&blobs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_batch_commitments_empty_blob() {
        let settings = Arc::new(MockKzgSettings::new());
        let processor = TestBatchProcessor::new(settings);
        
        let blobs = vec![
            vec![Fr::one(); 4096],
            vec![], // 空blob
            vec![Fr::one(); 4096],
        ];
        
        let result = processor.batch_commitments(&blobs);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty blob");
    }

    // Arena分配器测试
    use std::alloc::{alloc, dealloc, Layout};
    use std::ptr::NonNull;

    pub struct TestArena {
        chunks: Vec<TestChunk>,
        current_chunk: usize,
        current_pos: usize,
    }

    struct TestChunk {
        data: NonNull<u8>,
        size: usize,
        capacity: usize,
    }

    impl TestArena {
        pub fn new() -> Self {
            Self::with_capacity(1024)
        }
        
        pub fn with_capacity(capacity: usize) -> Self {
            let mut arena = Self {
                chunks: Vec::new(),
                current_chunk: 0,
                current_pos: 0,
            };
            arena.add_chunk(capacity);
            arena
        }
        
        fn add_chunk(&mut self, size: usize) {
            let layout = Layout::from_size_align(size, 8).unwrap();
            let data = unsafe { alloc(layout) };
            
            if data.is_null() {
                panic!("Arena allocation failed");
            }
            
            self.chunks.push(TestChunk {
                data: NonNull::new(data).unwrap(),
                size: 0,
                capacity: size,
            });
        }
        
        pub fn alloc<T>(&mut self, count: usize) -> &mut [T] {
            let size = std::mem::size_of::<T>() * count;
            let align = std::mem::align_of::<T>();
            
            let current_pos = (self.current_pos + align - 1) & !(align - 1);
            
            if let Some(chunk) = self.chunks.get_mut(self.current_chunk) {
                if current_pos + size <= chunk.capacity {
                    let ptr = unsafe { chunk.data.as_ptr().add(current_pos) as *mut T };
                    self.current_pos = current_pos + size;
                    chunk.size = self.current_pos;
                    
                    return unsafe { std::slice::from_raw_parts_mut(ptr, count) };
                }
            }
            
            let new_chunk_size = std::cmp::max(size * 2, 1024);
            self.add_chunk(new_chunk_size);
            self.current_chunk = self.chunks.len() - 1;
            self.current_pos = 0;
            
            self.alloc(count)
        }
        
        pub fn reset(&mut self) {
            self.current_chunk = 0;
            self.current_pos = 0;
            for chunk in &mut self.chunks {
                chunk.size = 0;
            }
        }
        
        pub fn used_memory(&self) -> usize {
            self.chunks.iter().map(|chunk| chunk.size).sum()
        }
        
        pub fn total_memory(&self) -> usize {
            self.chunks.iter().map(|chunk| chunk.capacity).sum()
        }
    }

    impl Drop for TestArena {
        fn drop(&mut self) {
            for chunk in &self.chunks {
                let layout = Layout::from_size_align(chunk.capacity, 8).unwrap();
                unsafe {
                    dealloc(chunk.data.as_ptr(), layout);
                }
            }
        }
    }

    #[test]
    fn test_arena_creation() {
        let arena = TestArena::new();
        assert_eq!(arena.current_chunk, 0);
        assert_eq!(arena.current_pos, 0);
        assert_eq!(arena.chunks.len(), 1);
        assert_eq!(arena.total_memory(), 1024);
    }

    #[test]
    fn test_arena_with_capacity() {
        let arena = TestArena::with_capacity(2048);
        assert_eq!(arena.total_memory(), 2048);
    }

    #[test]
    fn test_arena_allocation() {
        let mut arena = TestArena::new();
        
        let data1: &mut [u32] = arena.alloc(100);
        assert_eq!(data1.len(), 100);
        
        let data2: &mut [u64] = arena.alloc(50);
        assert_eq!(data2.len(), 50);
        
        assert!(arena.used_memory() > 0);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = TestArena::new();
        
        let _data: &mut [u32] = arena.alloc(100);
        let used_before_reset = arena.used_memory();
        assert!(used_before_reset > 0);
        
        arena.reset();
        assert_eq!(arena.used_memory(), 0);
        assert_eq!(arena.current_pos, 0);
        assert_eq!(arena.current_chunk, 0);
    }

    // 内存池测试
    pub struct TestMemoryPool<T> {
        pool: Vec<Vec<T>>,
        capacity: usize,
        max_size: usize,
    }

    impl<T: Default + Clone> TestMemoryPool<T> {
        pub fn new(capacity: usize, max_size: usize) -> Self {
            Self {
                pool: Vec::with_capacity(max_size),
                capacity,
                max_size,
            }
        }
        
        pub fn get(&mut self) -> Vec<T> {
            self.pool.pop().unwrap_or_else(|| {
                vec![T::default(); self.capacity]
            })
        }
        
        pub fn put(&mut self, mut obj: Vec<T>) {
            if self.pool.len() < self.max_size {
                obj.clear();
                obj.resize(self.capacity, T::default());
                self.pool.push(obj);
            }
        }
        
        pub fn size(&self) -> usize {
            self.pool.len()
        }
    }

    #[test]
    fn test_memory_pool_creation() {
        let pool: TestMemoryPool<u32> = TestMemoryPool::new(100, 10);
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.capacity, 100);
        assert_eq!(pool.max_size, 10);
    }

    #[test]
    fn test_memory_pool_get_and_put() {
        let mut pool: TestMemoryPool<u32> = TestMemoryPool::new(100, 10);
        
        // 获取对象（池为空，创建新对象）
        let obj1 = pool.get();
        assert_eq!(obj1.len(), 100);
        assert_eq!(pool.size(), 0);
        
        // 归还对象
        pool.put(obj1);
        assert_eq!(pool.size(), 1);
        
        // 再次获取（从池中获取）
        let obj2 = pool.get();
        assert_eq!(obj2.len(), 100);
        assert_eq!(pool.size(), 0);
        
        pool.put(obj2);
        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_memory_pool_max_size() {
        let mut pool: TestMemoryPool<u32> = TestMemoryPool::new(10, 2);
        
        // 添加对象直到达到最大大小
        pool.put(vec![1; 10]);
        assert_eq!(pool.size(), 1);
        
        pool.put(vec![2; 10]);
        assert_eq!(pool.size(), 2);
        
        // 超过最大大小的对象会被丢弃
        pool.put(vec![3; 10]);
        assert_eq!(pool.size(), 2);
    }

    // 性能监控测试
    use std::time::Instant;

    #[derive(Debug, Clone)]
    pub struct TestPerformanceMetrics {
        pub operations_count: u64,
        pub total_time: Duration,
        pub error_count: u64,
    }

    impl Default for TestPerformanceMetrics {
        fn default() -> Self {
            Self {
                operations_count: 0,
                total_time: Duration::new(0, 0),
                error_count: 0,
            }
        }
    }

    pub struct TestPerformanceMonitor {
        metrics: TestPerformanceMetrics,
    }

    impl TestPerformanceMonitor {
        pub fn new() -> Self {
            Self {
                metrics: TestPerformanceMetrics::default(),
            }
        }
        
        pub fn measure<F, R>(&mut self, operation: F) -> Result<R, String>
        where
            F: FnOnce() -> Result<R, String>,
        {
            let start_time = Instant::now();
            let result = operation();
            let duration = start_time.elapsed();
            
            self.metrics.operations_count += 1;
            self.metrics.total_time += duration;
            
            if result.is_err() {
                self.metrics.error_count += 1;
            }
            
            result
        }
        
        pub fn get_metrics(&self) -> &TestPerformanceMetrics {
            &self.metrics
        }
        
        pub fn reset(&mut self) {
            self.metrics = TestPerformanceMetrics::default();
        }
    }

    #[test]
    fn test_performance_monitor_success() {
        let mut monitor = TestPerformanceMonitor::new();
        
        let result = monitor.measure(|| {
            std::thread::sleep(Duration::from_millis(10));
            Ok("success".to_string())
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.operations_count, 1);
        assert_eq!(metrics.error_count, 0);
        assert!(metrics.total_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_performance_monitor_error() {
        let mut monitor = TestPerformanceMonitor::new();
        
        let result: Result<String, String> = monitor.measure(|| {
            Err("test error".to_string())
        });
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "test error");
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.operations_count, 1);
        assert_eq!(metrics.error_count, 1);
    }

    #[test]
    fn test_performance_monitor_multiple_operations() {
        let mut monitor = TestPerformanceMonitor::new();
        
        // 执行多个操作
        for i in 0..5 {
            let result = monitor.measure(|| {
                if i == 2 {
                    Err("error".to_string())
                } else {
                    Ok(i)
                }
            });
            
            if i == 2 {
                assert!(result.is_err());
            } else {
                assert!(result.is_ok());
            }
        }
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.operations_count, 5);
        assert_eq!(metrics.error_count, 1);
    }

    #[test]
    fn test_performance_monitor_reset() {
        let mut monitor = TestPerformanceMonitor::new();
        
        let _ = monitor.measure(|| Ok("test".to_string()));
        assert_eq!(monitor.get_metrics().operations_count, 1);
        
        monitor.reset();
        assert_eq!(monitor.get_metrics().operations_count, 0);
        assert_eq!(monitor.get_metrics().error_count, 0);
        assert_eq!(monitor.get_metrics().total_time, Duration::new(0, 0));
    }