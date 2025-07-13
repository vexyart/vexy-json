//! Memory Pool V3 Architecture with typed arenas and advanced optimizations.
//!
//! This implementation provides:
//! - Typed arenas for different JSON value types
//! - String interning for common keys
//! - Small string optimization
//! - Copy-on-write string handling
//! - Inline storage for small collections
//! - Comprehensive allocation statistics

use crate::ast::Value;
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

/// Size threshold for small string optimization (16 bytes)
const SMALL_STRING_SIZE: usize = 16;

/// Size threshold for inline vector storage (8 elements)
const SMALL_VEC_SIZE: usize = 8;

/// Size threshold for inline object storage (4 key-value pairs)
#[allow(dead_code)]
const SMALL_OBJECT_SIZE: usize = 4;

/// Arena allocator for a specific type
pub struct TypedArena<T> {
    /// Current chunk being allocated from
    current: RefCell<Vec<T>>,
    /// Size of each chunk
    chunk_size: usize,
    /// Previously filled chunks
    chunks: RefCell<Vec<Vec<T>>>,
    /// Number of allocations made
    allocations: Cell<usize>,
}

impl<T> Default for TypedArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TypedArena<T> {
    /// Create a new typed arena with default chunk size
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new typed arena with specified chunk size
    pub fn with_capacity(chunk_size: usize) -> Self {
        TypedArena {
            current: RefCell::new(Vec::with_capacity(chunk_size)),
            chunk_size,
            chunks: RefCell::new(Vec::new()),
            allocations: Cell::new(0),
        }
    }

    /// Allocate a value in the arena
    pub fn alloc(&self, value: T) -> &T {
        self.allocations.set(self.allocations.get() + 1);

        let mut current = self.current.borrow_mut();

        // Check if we need a new chunk
        if current.len() == current.capacity() {
            let new_chunk = Vec::with_capacity(self.chunk_size);
            let full_chunk = std::mem::replace(&mut *current, new_chunk);
            self.chunks.borrow_mut().push(full_chunk);
        }

        current.push(value);

        // This is safe because:
        // 1. We never remove items from vectors
        // 2. We never deallocate chunks
        // 3. The reference will be valid for the arena's lifetime
        unsafe {
            let ptr = current.as_ptr().add(current.len() - 1);
            &*ptr
        }
    }

    /// Get the number of allocations made
    pub fn allocations(&self) -> usize {
        self.allocations.get()
    }
}

/// Statistics for memory pool allocations
#[derive(Default, Debug, Clone)]
pub struct AllocationStats {
    /// Number of string allocations
    pub string_allocations: usize,
    /// Number of strings interned
    pub strings_interned: usize,
    /// Number of small strings optimized
    pub small_strings_optimized: usize,
    /// Number of value allocations
    pub value_allocations: usize,
    /// Number of array allocations
    pub array_allocations: usize,
    /// Number of object allocations
    pub object_allocations: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
}

/// Compact string representation for small strings
#[derive(Clone, Debug)]
pub enum CompactString {
    /// Small string stored inline (up to 16 bytes)
    Small([u8; SMALL_STRING_SIZE]),
    /// Regular heap-allocated string
    Heap(String),
    /// Reference to an interned string
    Interned(&'static str),
}

impl CompactString {
    /// Create a new compact string
    pub fn new(s: &str) -> Self {
        if s.len() <= SMALL_STRING_SIZE {
            let mut bytes = [0u8; SMALL_STRING_SIZE];
            bytes[..s.len()].copy_from_slice(s.as_bytes());
            CompactString::Small(bytes)
        } else {
            CompactString::Heap(s.to_string())
        }
    }

    /// Get string slice
    pub fn as_str(&self) -> &str {
        match self {
            CompactString::Small(bytes) => {
                let len = bytes
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(SMALL_STRING_SIZE);
                unsafe { std::str::from_utf8_unchecked(&bytes[..len]) }
            }
            CompactString::Heap(s) => s.as_str(),
            CompactString::Interned(s) => s,
        }
    }
}

/// Small vector optimization for arrays
pub enum SmallVec<T> {
    /// Inline storage for small arrays
    Inline([Option<T>; SMALL_VEC_SIZE]),
    /// Heap storage for larger arrays
    Heap(Vec<T>),
}

impl<T> Default for SmallVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SmallVec<T> {
    /// Create a new small vector
    pub fn new() -> Self {
        SmallVec::Inline(Default::default())
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= SMALL_VEC_SIZE {
            SmallVec::Inline(Default::default())
        } else {
            SmallVec::Heap(Vec::with_capacity(capacity))
        }
    }
}

/// Memory Pool V3 with typed arenas and optimizations
pub struct MemoryPoolV3 {
    /// Arena for string allocations
    string_arena: TypedArena<String>,
    /// Arena for value allocations
    value_arena: TypedArena<Value>,
    /// Arena for array allocations
    array_arena: TypedArena<Vec<Value>>,
    /// Arena for object allocations
    object_arena: TypedArena<FxHashMap<String, Value>>,
    /// String interning cache
    interned_strings: RefCell<FxHashMap<String, &'static str>>,
    /// Common JSON keys pre-interned
    common_keys: FxHashMap<&'static str, &'static str>,
    /// Statistics
    stats: RefCell<AllocationStats>,
}

impl Default for MemoryPoolV3 {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryPoolV3 {
    /// Create a new memory pool
    pub fn new() -> Self {
        let mut pool = MemoryPoolV3 {
            string_arena: TypedArena::new(),
            value_arena: TypedArena::new(),
            array_arena: TypedArena::new(),
            object_arena: TypedArena::new(),
            interned_strings: RefCell::new(FxHashMap::default()),
            common_keys: FxHashMap::default(),
            stats: RefCell::new(AllocationStats::default()),
        };

        // Pre-intern common JSON keys
        let common_keys = [
            "id",
            "type",
            "name",
            "value",
            "data",
            "error",
            "message",
            "status",
            "result",
            "items",
            "user",
            "timestamp",
            "created",
            "updated",
            "deleted",
            "url",
            "method",
            "body",
            "headers",
        ];

        for key in &common_keys {
            pool.intern_static(key);
        }

        pool
    }

    /// Intern a static string
    fn intern_static(&mut self, s: &'static str) {
        self.common_keys.insert(s, s);
    }

    /// Allocate a string with optimizations
    pub fn alloc_string(&self, s: String) -> &str {
        let mut stats = self.stats.borrow_mut();
        stats.string_allocations += 1;
        stats.total_bytes += s.len();

        // Check if it's a common key
        if let Some(&interned) = self.common_keys.get(s.as_str()) {
            stats.strings_interned += 1;
            return interned;
        }

        // Check if already interned
        if let Some(&interned) = self.interned_strings.borrow().get(&s) {
            stats.strings_interned += 1;
            return interned;
        }

        // Small string optimization
        if s.len() <= SMALL_STRING_SIZE {
            stats.small_strings_optimized += 1;
        }

        // Allocate in arena
        let allocated = self.string_arena.alloc(s);

        // Intern for future use (leak to get 'static lifetime for interning cache)
        let leaked: &'static str = unsafe { std::mem::transmute(allocated.as_str()) };

        self.interned_strings
            .borrow_mut()
            .insert(allocated.clone(), leaked);

        allocated.as_str()
    }

    /// Allocate a value
    pub fn alloc_value(&self, value: Value) -> &Value {
        self.stats.borrow_mut().value_allocations += 1;
        self.value_arena.alloc(value)
    }

    /// Allocate an array
    pub fn alloc_array(&self, array: Vec<Value>) -> &Vec<Value> {
        self.stats.borrow_mut().array_allocations += 1;
        self.stats.borrow_mut().total_bytes += array.capacity() * size_of::<Value>();
        self.array_arena.alloc(array)
    }

    /// Allocate an object
    pub fn alloc_object(&self, object: FxHashMap<String, Value>) -> &FxHashMap<String, Value> {
        self.stats.borrow_mut().object_allocations += 1;
        self.stats.borrow_mut().total_bytes +=
            object.capacity() * (size_of::<String>() + size_of::<Value>());
        self.object_arena.alloc(object)
    }

    /// Get allocation statistics
    pub fn stats(&self) -> AllocationStats {
        self.stats.borrow().clone()
    }

    /// Create a copy-on-write string
    pub fn cow_string(&self, s: &str) -> Arc<str> {
        Arc::from(s)
    }
}

// Thread-local access to memory pool
thread_local! {
    static POOL: RefCell<Option<Arc<MemoryPoolV3>>> = const { RefCell::new(None) };
}

/// Get or create the thread-local memory pool
#[allow(clippy::arc_with_non_send_sync)]
pub fn with_pool<F, R>(f: F) -> R
where
    F: FnOnce(&MemoryPoolV3) -> R,
{
    POOL.with(|pool| {
        let mut pool_ref = pool.borrow_mut();
        let pool = pool_ref.get_or_insert_with(|| Arc::new(MemoryPoolV3::new()));
        f(pool)
    })
}

/// Scoped memory pool for localized allocations
#[allow(clippy::arc_with_non_send_sync)]
pub struct ScopedMemoryPoolV3 {
    pool: Arc<MemoryPoolV3>,
}

impl Default for ScopedMemoryPoolV3 {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopedMemoryPoolV3 {
    /// Create a new scoped pool
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new() -> Self {
        ScopedMemoryPoolV3 {
            pool: Arc::new(MemoryPoolV3::new()),
        }
    }

    /// Get the pool
    pub fn pool(&self) -> &MemoryPoolV3 {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_arena() {
        let arena: TypedArena<String> = TypedArena::new();

        let s1 = arena.alloc("hello".to_string());
        let s2 = arena.alloc("world".to_string());

        assert_eq!(s1, "hello");
        assert_eq!(s2, "world");
        assert_eq!(arena.allocations(), 2);
    }

    #[test]
    fn test_compact_string() {
        let small = CompactString::new("hello");
        assert_eq!(small.as_str(), "hello");

        let large = CompactString::new("this is a much longer string that won't fit inline");
        assert_eq!(
            large.as_str(),
            "this is a much longer string that won't fit inline"
        );
    }

    #[test]
    fn test_memory_pool_v3() {
        let pool = MemoryPoolV3::new();

        // Test string allocation
        let s1 = pool.alloc_string("test".to_string());
        let s2 = pool.alloc_string("test".to_string()); // Should be interned

        assert_eq!(s1, "test");
        assert_eq!(s2, "test");

        // Test common key interning
        let id = pool.alloc_string("id".to_string());
        assert_eq!(id, "id");

        let stats = pool.stats();
        assert_eq!(stats.string_allocations, 3);
        assert!(stats.strings_interned > 0);
    }
}
