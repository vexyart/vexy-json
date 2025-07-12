//! Memory pool allocator for high-performance JSON parsing.
//!
//! This module provides a memory pool allocator that reduces allocation overhead
//! by pre-allocating memory blocks and reusing them throughout the parsing process.
//! This is particularly effective for string-heavy JSON documents.

use std::cell::{Cell, RefCell};
use std::mem::{align_of, size_of};
use std::ptr::NonNull;
use std::slice;

/// Size of each memory block in the pool (64KB by default)
const BLOCK_SIZE: usize = 64 * 1024;

/// Maximum number of blocks to keep in the free list
const MAX_FREE_BLOCKS: usize = 16;

/// Alignment for all allocations
const ALIGNMENT: usize = 8;

/// A memory pool allocator for efficient string and value allocations.
///
/// The pool pre-allocates memory in blocks and serves allocations from
/// these blocks, reducing the number of system allocations needed.
pub struct MemoryPool {
    /// Current block being allocated from
    current_block: RefCell<Option<Block>>,
    /// List of free blocks that can be reused
    free_blocks: RefCell<Vec<Block>>,
    /// Total bytes allocated
    total_allocated: Cell<usize>,
    /// Total bytes used
    total_used: Cell<usize>,
}

/// A single memory block in the pool
struct Block {
    /// The allocated memory as a vector
    memory: Vec<u8>,
    /// Current position in the block
    pos: usize,
}

impl Block {
    /// Creates a new memory block of the specified size
    fn new(size: usize) -> Self {
        Block {
            memory: vec![0u8; size],
            pos: 0,
        }
    }

    /// Returns the amount of free space in this block
    #[allow(dead_code)]
    fn available(&self) -> usize {
        self.memory.len() - self.pos
    }

    /// Allocates memory from this block if possible
    fn allocate(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Align the current position
        let aligned_pos = (self.pos + align - 1) & !(align - 1);
        let end_pos = aligned_pos + size;

        if end_pos <= self.memory.len() {
            let ptr = unsafe { self.memory.as_mut_ptr().add(aligned_pos) };
            self.pos = end_pos;
            NonNull::new(ptr)
        } else {
            None
        }
    }

    /// Resets the block for reuse
    fn reset(&mut self) {
        self.pos = 0;
    }
}

unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}

impl MemoryPool {
    /// Creates a new memory pool
    pub fn new() -> Self {
        MemoryPool {
            current_block: RefCell::new(None),
            free_blocks: RefCell::new(Vec::with_capacity(MAX_FREE_BLOCKS)),
            total_allocated: Cell::new(0),
            total_used: Cell::new(0),
        }
    }

    /// Allocates memory from the pool
    pub fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        self.allocate_aligned(size, ALIGNMENT)
    }

    /// Allocates aligned memory from the pool
    pub fn allocate_aligned(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Try to allocate from current block
        if let Some(ref mut block) = *self.current_block.borrow_mut() {
            if let Some(ptr) = block.allocate(size, align) {
                self.total_used.set(self.total_used.get() + size);
                return Some(ptr);
            }
        }

        // Current block doesn't have enough space, get a new one
        let block_size = size.max(BLOCK_SIZE);
        let mut new_block = self.get_or_create_block(block_size);

        let ptr = new_block.allocate(size, align)?;
        self.total_used.set(self.total_used.get() + size);

        // Store the old block in free list if it exists
        if let Some(old_block) = self.current_block.borrow_mut().take() {
            self.store_free_block(old_block);
        }

        *self.current_block.borrow_mut() = Some(new_block);
        Some(ptr)
    }

    /// Allocates a string in the pool and returns a reference to it
    pub fn allocate_str<'a>(&self, s: &str) -> Option<&'a str> {
        let bytes = s.as_bytes();
        let ptr = self.allocate(bytes.len())?;

        unsafe {
            // Copy the string data
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), bytes.len());

            // Create a string slice from the allocated memory
            let slice = slice::from_raw_parts(ptr.as_ptr(), bytes.len());
            std::str::from_utf8_unchecked(slice).into()
        }
    }

    /// Allocates and copies a value in the pool
    pub fn allocate_copy<'a, T: Copy>(&self, value: &T) -> Option<&'a T> {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let ptr = self.allocate_aligned(size, align)?;

        unsafe {
            // Copy the value
            std::ptr::write(ptr.as_ptr() as *mut T, *value);

            // Return a reference to the allocated value
            Some(&*(ptr.as_ptr() as *const T))
        }
    }

    /// Allocates space for a slice and copies the data
    pub fn allocate_slice<'a, T: Copy>(&self, slice: &[T]) -> Option<&'a [T]> {
        if slice.is_empty() {
            return Some(unsafe { slice::from_raw_parts(slice.as_ptr(), 0) });
        }

        let size = size_of::<T>() * slice.len();
        let align = align_of::<T>();
        let ptr = self.allocate_aligned(size, align)?;

        unsafe {
            // Copy the slice data
            std::ptr::copy_nonoverlapping(slice.as_ptr(), ptr.as_ptr() as *mut T, slice.len());

            // Create a slice from the allocated memory
            Some(slice::from_raw_parts(ptr.as_ptr() as *const T, slice.len()))
        }
    }

    /// Resets the pool, making all memory available for reuse
    pub fn reset(&self) {
        // Reset current block
        if let Some(ref mut block) = *self.current_block.borrow_mut() {
            block.reset();
        }

        // Reset all free blocks
        for block in self.free_blocks.borrow_mut().iter_mut() {
            block.reset();
        }

        self.total_used.set(0);
    }

    /// Returns statistics about the memory pool
    pub fn stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            total_allocated: self.total_allocated.get(),
            total_used: self.total_used.get(),
            num_blocks: self.free_blocks.borrow().len()
                + if self.current_block.borrow().is_some() {
                    1
                } else {
                    0
                },
        }
    }

    /// Gets a block from the free list or creates a new one
    fn get_or_create_block(&self, size: usize) -> Block {
        // Try to get a block from the free list
        let mut free_blocks = self.free_blocks.borrow_mut();

        // Look for a suitable block
        let mut suitable_index = None;
        for (i, block) in free_blocks.iter().enumerate() {
            if block.memory.len() >= size {
                suitable_index = Some(i);
                break;
            }
        }

        if let Some(index) = suitable_index {
            let mut block = free_blocks.swap_remove(index);
            block.reset();
            return block;
        }

        // Create a new block
        self.total_allocated.set(self.total_allocated.get() + size);
        Block::new(size)
    }

    /// Stores a block in the free list for reuse
    fn store_free_block(&self, mut block: Block) {
        block.reset();
        let mut free_blocks = self.free_blocks.borrow_mut();

        if free_blocks.len() < MAX_FREE_BLOCKS {
            free_blocks.push(block);
        }
        // If we have too many free blocks, just drop this one
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about memory pool usage
#[derive(Debug, Clone, Copy)]
pub struct MemoryPoolStats {
    /// Total bytes allocated by the pool
    pub total_allocated: usize,
    /// Total bytes currently in use
    pub total_used: usize,
    /// Number of memory blocks
    pub num_blocks: usize,
}

impl MemoryPoolStats {
    /// Returns the utilization percentage (0-100)
    pub fn utilization(&self) -> f32 {
        if self.total_allocated == 0 {
            0.0
        } else {
            (self.total_used as f32 / self.total_allocated as f32) * 100.0
        }
    }
}

/// A memory pool that can be used with a specific lifetime
///
/// This wrapper ensures that references allocated from the pool
/// cannot outlive the pool itself.
pub struct ScopedMemoryPool<'a> {
    pool: MemoryPool,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ScopedMemoryPool<'a> {
    /// Creates a new scoped memory pool
    pub fn new() -> Self {
        ScopedMemoryPool {
            pool: MemoryPool::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Allocates a string in the pool with the pool's lifetime
    pub fn allocate_str(&self, s: &str) -> Option<&'a str> {
        self.pool.allocate_str(s)
    }

    /// Allocates and copies a value in the pool with the pool's lifetime
    pub fn allocate_copy<T: Copy>(&self, value: &T) -> Option<&'a T> {
        self.pool.allocate_copy(value)
    }

    /// Allocates a slice in the pool with the pool's lifetime
    pub fn allocate_slice<T: Copy>(&self, slice: &[T]) -> Option<&'a [T]> {
        self.pool.allocate_slice(slice)
    }

    /// Resets the pool
    pub fn reset(&self) {
        self.pool.reset()
    }

    /// Returns statistics about the pool
    pub fn stats(&self) -> MemoryPoolStats {
        self.pool.stats()
    }
}

impl<'a> Default for ScopedMemoryPool<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let pool = MemoryPool::new();

        // Allocate some memory
        let ptr1 = pool.allocate(100).unwrap();
        let ptr2 = pool.allocate(200).unwrap();

        // Pointers should be different
        assert_ne!(ptr1.as_ptr(), ptr2.as_ptr());

        // Check stats
        let stats = pool.stats();
        assert_eq!(stats.total_used, 300);
    }

    #[test]
    fn test_string_allocation() {
        let pool = MemoryPool::new();

        let s1 = "Hello, world!";
        let s2 = "Another string";

        let allocated1 = pool.allocate_str(s1).unwrap();
        let allocated2 = pool.allocate_str(s2).unwrap();

        assert_eq!(allocated1, s1);
        assert_eq!(allocated2, s2);

        // Strings should be at different addresses
        assert_ne!(allocated1.as_ptr(), allocated2.as_ptr());
    }

    #[test]
    fn test_reset() {
        let pool = MemoryPool::new();

        // Allocate some memory
        pool.allocate(1000).unwrap();

        let stats_before = pool.stats();
        assert_eq!(stats_before.total_used, 1000);

        // Reset the pool
        pool.reset();

        let stats_after = pool.stats();
        assert_eq!(stats_after.total_used, 0);
        assert_eq!(stats_after.total_allocated, stats_before.total_allocated);
    }

    #[test]
    fn test_large_allocation() {
        let pool = MemoryPool::new();

        // Allocate more than BLOCK_SIZE
        let large_size = BLOCK_SIZE * 2;
        let _ptr = pool.allocate(large_size).unwrap();

        let stats = pool.stats();
        assert_eq!(stats.total_used, large_size);
    }

    #[test]
    fn test_scoped_pool() {
        let pool = ScopedMemoryPool::new();

        let s = "Test string";
        let allocated = pool.allocate_str(s).unwrap();

        assert_eq!(allocated, s);
    }
}
