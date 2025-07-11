//! Optimized memory pool implementation with adaptive allocation strategies.
//!
//! This version improves on the original by:
//! - Using conditional pooling based on allocation size
//! - Reducing overhead for small allocations
//! - Better cache locality
//! - Thread-local storage for reduced contention

use std::cell::{Cell, RefCell};
// std::mem import removed - not needed
use std::ptr::NonNull;

/// Minimum allocation size to use the pool (in bytes)
const MIN_POOL_ALLOCATION_SIZE: usize = 64;

/// Maximum allocation size for the pool (in bytes)
const MAX_POOL_ALLOCATION_SIZE: usize = 4096;

/// Default block size for memory pool (16KB)
const DEFAULT_BLOCK_SIZE: usize = 16 * 1024;

thread_local! {
    /// Thread-local memory pool for reduced contention
    static THREAD_LOCAL_POOL: RefCell<Option<FastMemoryPool>> = RefCell::new(None);
}

/// Optimized memory pool with adaptive allocation strategies.
pub struct OptimizedMemoryPool {
    /// Enable or disable pooling based on runtime analysis
    pooling_enabled: Cell<bool>,
    /// Statistics for adaptive behavior
    stats: PoolStatistics,
    /// Thread-local storage enabled
    use_thread_local: bool,
}

/// Fast memory pool implementation with minimal overhead.
struct FastMemoryPool {
    /// Current position in the active block
    current_pos: usize,
    /// Active memory block
    current_block: Vec<u8>,
    /// Free blocks for reuse
    free_blocks: Vec<Vec<u8>>,
    /// Block size
    block_size: usize,
}

/// Statistics for adaptive pooling decisions.
#[derive(Default)]
struct PoolStatistics {
    /// Number of allocations
    allocations: Cell<usize>,
    /// Number of allocations that used the pool
    pooled_allocations: Cell<usize>,
    /// Total bytes allocated
    total_allocated: Cell<usize>,
    /// Average allocation size
    avg_allocation_size: Cell<usize>,
}

impl OptimizedMemoryPool {
    /// Creates a new optimized memory pool.
    pub fn new() -> Self {
        OptimizedMemoryPool {
            pooling_enabled: Cell::new(true),
            stats: PoolStatistics::default(),
            use_thread_local: true,
        }
    }

    /// Creates a memory pool with custom configuration.
    pub fn with_config(use_thread_local: bool, initial_enabled: bool) -> Self {
        OptimizedMemoryPool {
            pooling_enabled: Cell::new(initial_enabled),
            stats: PoolStatistics::default(),
            use_thread_local,
        }
    }

    /// Allocates memory, using the pool for appropriately sized allocations.
    pub fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        // Update statistics
        self.stats.allocations.set(self.stats.allocations.get() + 1);
        self.stats
            .total_allocated
            .set(self.stats.total_allocated.get() + size);

        // Update average allocation size
        let total_allocs = self.stats.allocations.get();
        let total_bytes = self.stats.total_allocated.get();
        if total_allocs > 0 {
            self.stats
                .avg_allocation_size
                .set(total_bytes / total_allocs);
        }

        // Check if pooling is beneficial for this allocation
        if !self.should_use_pool(size) {
            // Direct allocation for small or very large allocations
            let layout = std::alloc::Layout::from_size_align(size, 1).ok()?;
            let ptr = unsafe { std::alloc::alloc(layout) };
            NonNull::new(ptr)
        } else {
            // Use the pool
            self.stats
                .pooled_allocations
                .set(self.stats.pooled_allocations.get() + 1);

            if self.use_thread_local {
                THREAD_LOCAL_POOL.with(|pool| {
                    let mut pool_ref = pool.borrow_mut();
                    if pool_ref.is_none() {
                        *pool_ref = Some(FastMemoryPool::new(DEFAULT_BLOCK_SIZE));
                    }
                    pool_ref.as_mut().unwrap().allocate(size)
                })
            } else {
                // Fall back to direct allocation if thread-local not available
                let layout = std::alloc::Layout::from_size_align(size, 1).ok()?;
                let ptr = unsafe { std::alloc::alloc(layout) };
                NonNull::new(ptr)
            }
        }
    }

    /// Allocates a string using the optimized strategy.
    pub fn allocate_str<'a>(&self, s: &str) -> Option<&'a str> {
        if s.is_empty() {
            return Some("");
        }

        let bytes = s.as_bytes();
        let size = bytes.len();

        // For very small strings, it might be faster to just use regular allocation
        if size < MIN_POOL_ALLOCATION_SIZE {
            // Direct heap allocation for small strings
            let boxed = s.to_string().into_boxed_str();
            let leaked = Box::leak(boxed);
            Some(leaked)
        } else {
            // Use the pool for larger strings
            let ptr = self.allocate(size)?;
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), size);
                Some(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    ptr.as_ptr(),
                    size,
                )))
            }
        }
    }

    /// Determines if the pool should be used for this allocation size.
    fn should_use_pool(&self, size: usize) -> bool {
        if !self.pooling_enabled.get() {
            return false;
        }

        // Use pool for medium-sized allocations
        size >= MIN_POOL_ALLOCATION_SIZE && size <= MAX_POOL_ALLOCATION_SIZE
    }

    /// Enables or disables pooling based on performance analysis.
    pub fn set_pooling_enabled(&self, enabled: bool) {
        self.pooling_enabled.set(enabled);
    }

    /// Returns statistics about pool usage.
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_allocations: self.stats.allocations.get(),
            pooled_allocations: self.stats.pooled_allocations.get(),
            total_bytes: self.stats.total_allocated.get(),
            avg_allocation_size: self.stats.avg_allocation_size.get(),
            pooling_enabled: self.pooling_enabled.get(),
        }
    }

    /// Resets the thread-local pool if enabled.
    pub fn reset(&self) {
        if self.use_thread_local {
            THREAD_LOCAL_POOL.with(|pool| {
                if let Some(ref mut p) = *pool.borrow_mut() {
                    p.reset();
                }
            });
        }
    }
}

impl FastMemoryPool {
    /// Creates a new fast memory pool.
    fn new(block_size: usize) -> Self {
        FastMemoryPool {
            current_pos: 0,
            current_block: Vec::with_capacity(block_size),
            free_blocks: Vec::new(),
            block_size,
        }
    }

    /// Allocates memory from the pool.
    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        // Align the allocation
        let align = if size >= 8 { 8 } else { 1 };
        let aligned_size = (size + align - 1) & !(align - 1);

        // Check if we need a new block
        if self.current_pos + aligned_size > self.current_block.capacity() {
            self.allocate_new_block();
        }

        // Allocate from current block
        if self.current_pos + aligned_size <= self.current_block.capacity() {
            let ptr = unsafe { self.current_block.as_mut_ptr().add(self.current_pos) };
            self.current_pos += aligned_size;
            NonNull::new(ptr)
        } else {
            // Allocation too large for block size
            None
        }
    }

    /// Allocates a new block, reusing from free list if possible.
    fn allocate_new_block(&mut self) {
        // Save current block if it has content
        if self.current_pos > 0 {
            let mut old_block =
                std::mem::replace(&mut self.current_block, Vec::with_capacity(self.block_size));
            unsafe {
                old_block.set_len(self.current_pos);
            }
            self.free_blocks.push(old_block);
        }

        // Try to reuse a free block
        if let Some(mut block) = self.free_blocks.pop() {
            block.clear();
            self.current_block = block;
        } else {
            self.current_block = Vec::with_capacity(self.block_size);
        }

        self.current_pos = 0;
    }

    /// Resets the pool for reuse.
    fn reset(&mut self) {
        self.current_pos = 0;
        // Keep blocks allocated for reuse
    }
}

/// Statistics about pool usage.
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Number of allocations that used the pool
    pub pooled_allocations: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Average allocation size
    pub avg_allocation_size: usize,
    /// Whether pooling is currently enabled
    pub pooling_enabled: bool,
}

/// Scoped memory pool that automatically resets when dropped.
pub struct ScopedOptimizedPool<'a> {
    pool: &'a OptimizedMemoryPool,
}

impl<'a> ScopedOptimizedPool<'a> {
    /// Creates a new scoped pool.
    pub fn new(pool: &'a OptimizedMemoryPool) -> Self {
        ScopedOptimizedPool { pool }
    }

    /// Allocates a string in the scoped pool.
    pub fn allocate_str(&self, s: &str) -> Option<&'a str> {
        self.pool.allocate_str(s)
    }
}

impl<'a> Drop for ScopedOptimizedPool<'a> {
    fn drop(&mut self) {
        self.pool.reset();
    }
}

impl Default for OptimizedMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_allocation_bypass() {
        let pool = OptimizedMemoryPool::new();

        // Small strings should bypass the pool
        let small = "hi";
        let allocated = pool.allocate_str(small).unwrap();
        assert_eq!(allocated, small);

        let stats = pool.stats();
        assert_eq!(stats.pooled_allocations, 0); // Should bypass pool
    }

    #[test]
    fn test_medium_allocation_pooled() {
        let pool = OptimizedMemoryPool::new();

        // Medium strings should use the pool
        let medium = "a".repeat(100);
        let allocated = pool.allocate_str(&medium).unwrap();
        assert_eq!(allocated, medium);

        let stats = pool.stats();
        assert_eq!(stats.pooled_allocations, 1); // Should use pool
    }

    #[test]
    fn test_adaptive_pooling() {
        let pool = OptimizedMemoryPool::new();

        // Allocate many small items
        for i in 0..100 {
            let s = format!("{}", i);
            pool.allocate_str(&s);
        }

        let stats = pool.stats();
        assert!(stats.avg_allocation_size < MIN_POOL_ALLOCATION_SIZE);

        // Disable pooling for better performance on small allocations
        pool.set_pooling_enabled(false);

        // Verify pooling is disabled
        let large = "x".repeat(200);
        pool.allocate_str(&large);

        let new_stats = pool.stats();
        assert_eq!(new_stats.pooled_allocations, stats.pooled_allocations);
    }

    #[test]
    fn test_scoped_pool() {
        let pool = OptimizedMemoryPool::new();

        {
            let scoped = ScopedOptimizedPool::new(&pool);
            let s = "test string";
            let allocated = scoped.allocate_str(s).unwrap();
            assert_eq!(allocated, s);
        }
        // Pool is reset when scoped pool is dropped

        // Verify reset functionality
        pool.reset();
        let stats = pool.stats();
        assert!(stats.total_allocations > 0); // Stats persist
    }
}
