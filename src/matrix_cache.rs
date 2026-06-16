//! Matrix computation cache with LRU eviction policy
//!
//! This module provides intelligent caching for expensive matrix computations
//! commonly used in quantum circuit simulation.

use crate::error::Result;
use crate::gates::StandardGate;
use ndarray::Array2;
use num_complex::Complex64;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cache key for matrix computations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatrixCacheKey {
    /// Gate type
    gate_type: StandardGate,
    /// Parameters (rounded to avoid floating point precision issues)
    parameters: Vec<i64>, // Parameters multiplied by 1e6 and rounded
    /// Matrix dimensions
    dimensions: (usize, usize),
}

impl MatrixCacheKey {
    /// Create a new cache key
    pub fn new(gate_type: StandardGate, parameters: &[f64], dimensions: (usize, usize)) -> Self {
        let rounded_params: Vec<i64> = parameters
            .iter()
            .map(|&p| (p * 1_000_000.0).round() as i64)
            .collect();

        MatrixCacheKey {
            gate_type,
            parameters: rounded_params,
            dimensions,
        }
    }

    /// Create a key for a parameterless gate
    pub fn for_gate(gate_type: StandardGate, dimensions: (usize, usize)) -> Self {
        MatrixCacheKey {
            gate_type,
            parameters: Vec::new(),
            dimensions,
        }
    }
}

/// Cached matrix entry with metadata
#[derive(Debug, Clone)]
pub struct CachedMatrix {
    /// The cached matrix
    matrix: Array2<Complex64>,
    /// When this entry was created
    created_at: Instant,
    /// Last access time for LRU
    last_accessed: Instant,
    /// Number of times this entry has been accessed
    access_count: usize,
    /// Size in bytes (approximate)
    size_bytes: usize,
}

impl CachedMatrix {
    /// Create a new cached matrix entry
    pub fn new(matrix: Array2<Complex64>) -> Self {
        let size_bytes = matrix.len() * std::mem::size_of::<Complex64>();
        let now = Instant::now();

        CachedMatrix {
            matrix,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            size_bytes,
        }
    }

    /// Access the matrix (updates LRU metadata)
    pub fn access(&mut self) -> &Array2<Complex64> {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.matrix
    }

    /// Get the matrix without updating access metadata
    pub fn matrix(&self) -> &Array2<Complex64> {
        &self.matrix
    }

    /// Get the age of this cache entry
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time since last access
    pub fn time_since_access(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}

/// LRU cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum total memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum age for entries (entries older than this are evicted)
    pub max_age: Duration,
    /// Enable access-based eviction
    pub enable_lru: bool,
    /// Enable size-based eviction
    pub enable_size_limit: bool,
    /// Enable age-based eviction
    pub enable_age_limit: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            max_entries: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_age: Duration::from_secs(300),   // 5 minutes
            enable_lru: true,
            enable_size_limit: true,
            enable_age_limit: true,
        }
    }
}

/// Matrix computation cache with LRU eviction
pub struct MatrixCache {
    /// Cache storage
    cache: Mutex<HashMap<MatrixCacheKey, CachedMatrix>>,
    /// LRU tracking (most recently used at back)
    lru_order: Mutex<VecDeque<MatrixCacheKey>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Mutex<CacheStats>,
}

impl MatrixCache {
    /// Create a new matrix cache with default configuration
    pub fn new() -> Self {
        MatrixCache::with_config(CacheConfig::default())
    }

    /// Create a new matrix cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        MatrixCache {
            cache: Mutex::new(HashMap::new()),
            lru_order: Mutex::new(VecDeque::new()),
            config,
            stats: Mutex::new(CacheStats::new()),
        }
    }

    /// Get a matrix from cache or compute it
    pub fn get_or_compute<F>(&self, key: MatrixCacheKey, compute_fn: F) -> Result<Array2<Complex64>>
    where
        F: FnOnce() -> Result<Array2<Complex64>>,
    {
        // Try to get from cache first
        if let Some(matrix) = self.get(&key) {
            return Ok(matrix);
        }

        // Not in cache, compute it
        let matrix = compute_fn()?;

        // Store in cache
        self.put(key, matrix.clone())?;

        Ok(matrix)
    }

    /// Get a matrix from cache
    pub fn get(&self, key: &MatrixCacheKey) -> Option<Array2<Complex64>> {
        let mut cache = self.cache.lock().unwrap();
        let mut lru_order = self.lru_order.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(cached) = cache.get_mut(key) {
            // Update LRU order
            if self.config.enable_lru {
                // Remove from current position
                if let Some(pos) = lru_order.iter().position(|k| k == key) {
                    lru_order.remove(pos);
                }
                // Add to back (most recent)
                lru_order.push_back(key.clone());
            }

            // Update statistics
            stats.hits += 1;
            stats.total_accesses += 1;

            Some(cached.access().clone())
        } else {
            // Update statistics
            stats.misses += 1;
            stats.total_accesses += 1;
            None
        }
    }

    /// Put a matrix in cache
    pub fn put(&self, key: MatrixCacheKey, matrix: Array2<Complex64>) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        let mut lru_order = self.lru_order.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        // Create cached entry
        let cached = CachedMatrix::new(matrix);
        let entry_size = cached.size_bytes;

        // Insert into cache first
        cache.insert(key.clone(), cached);

        // Update LRU order
        if self.config.enable_lru {
            lru_order.push_back(key);
        }

        // Update statistics
        stats.entries_added += 1;
        stats.current_memory_bytes += entry_size;

        // Check if we need to evict entries after insertion
        self.evict_if_needed(&mut cache, &mut lru_order, &mut stats)?;

        Ok(())
    }

    /// Evict entries if cache limits are exceeded
    fn evict_if_needed(
        &self,
        cache: &mut HashMap<MatrixCacheKey, CachedMatrix>,
        lru_order: &mut VecDeque<MatrixCacheKey>,
        stats: &mut CacheStats,
    ) -> Result<()> {
        // Age-based eviction
        if self.config.enable_age_limit {
            let mut to_remove = Vec::new();
            for (key, entry) in cache.iter() {
                if entry.age() > self.config.max_age {
                    to_remove.push(key.clone());
                }
            }

            for key in to_remove {
                self.remove_entry(cache, lru_order, stats, &key);
            }
        }

        // Size-based eviction
        if self.config.enable_size_limit {
            while stats.current_memory_bytes > self.config.max_memory_bytes && !cache.is_empty() {
                if let Some(key) = lru_order.pop_front() {
                    // Remove from cache only (already removed from lru_order)
                    if let Some(entry) = cache.remove(&key) {
                        stats.entries_evicted += 1;
                        stats.current_memory_bytes =
                            stats.current_memory_bytes.saturating_sub(entry.size_bytes);
                    }
                } else {
                    break;
                }
            }
        }

        // Count-based eviction
        while cache.len() > self.config.max_entries && !cache.is_empty() {
            if let Some(key) = lru_order.pop_front() {
                // Remove from cache only (already removed from lru_order)
                if let Some(entry) = cache.remove(&key) {
                    stats.entries_evicted += 1;
                    stats.current_memory_bytes =
                        stats.current_memory_bytes.saturating_sub(entry.size_bytes);
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Remove a specific entry from cache
    fn remove_entry(
        &self,
        cache: &mut HashMap<MatrixCacheKey, CachedMatrix>,
        lru_order: &mut VecDeque<MatrixCacheKey>,
        stats: &mut CacheStats,
        key: &MatrixCacheKey,
    ) {
        if let Some(entry) = cache.remove(key) {
            stats.entries_evicted += 1;
            stats.current_memory_bytes =
                stats.current_memory_bytes.saturating_sub(entry.size_bytes);

            // Remove from LRU order if present
            if let Some(pos) = lru_order.iter().position(|k| k == key) {
                lru_order.remove(pos);
            }
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut lru_order = self.lru_order.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        cache.clear();
        lru_order.clear();
        stats.current_memory_bytes = 0;
        stats.entries_evicted += cache.len();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    /// Check if cache contains a key
    pub fn contains_key(&self, key: &MatrixCacheKey) -> bool {
        self.cache.lock().unwrap().contains_key(key)
    }
}

impl Default for MatrixCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Total access attempts
    pub total_accesses: usize,
    /// Number of entries added
    pub entries_added: usize,
    /// Number of entries evicted
    pub entries_evicted: usize,
    /// Current memory usage in bytes
    pub current_memory_bytes: usize,
}

impl CacheStats {
    fn new() -> Self {
        CacheStats {
            hits: 0,
            misses: 0,
            total_accesses: 0,
            entries_added: 0,
            entries_evicted: 0,
            current_memory_bytes: 0,
        }
    }

    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        if self.total_accesses == 0 {
            0.0
        } else {
            (self.hits as f64 / self.total_accesses as f64) * 100.0
        }
    }

    /// Calculate miss rate as a percentage
    pub fn miss_rate(&self) -> f64 {
        100.0 - self.hit_rate()
    }

    /// Get current memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.current_memory_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Global matrix cache instance
static GLOBAL_CACHE: std::sync::OnceLock<Arc<MatrixCache>> = std::sync::OnceLock::new();

/// Get the global matrix cache instance
pub fn global_matrix_cache() -> Arc<MatrixCache> {
    GLOBAL_CACHE
        .get_or_init(|| Arc::new(MatrixCache::new()))
        .clone()
}

/// Initialize global cache with custom configuration
pub fn init_global_cache(config: CacheConfig) {
    let _ = GLOBAL_CACHE.set(Arc::new(MatrixCache::with_config(config)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::StandardGate;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_key_creation() {
        let key1 = MatrixCacheKey::new(StandardGate::Rx, &[1.5708], (2, 2));
        let key2 = MatrixCacheKey::new(StandardGate::Rx, &[1.5708], (2, 2));
        let key3 = MatrixCacheKey::new(StandardGate::Rx, &[1.5709], (2, 2));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_basic_cache_operations() {
        let cache = MatrixCache::new();
        let key = MatrixCacheKey::for_gate(StandardGate::H, (2, 2));
        let matrix = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));

        // Initially empty
        assert!(cache.get(&key).is_none());

        // Put and get
        cache.put(key.clone(), matrix.clone()).unwrap();
        let retrieved = cache.get(&key).unwrap();

        assert_eq!(matrix, retrieved);

        // Check statistics
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.entries_added, 1);
    }

    #[test]
    fn test_lru_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            enable_lru: true,
            enable_size_limit: false,
            enable_age_limit: false,
            ..Default::default()
        };

        let cache = MatrixCache::with_config(config);
        let matrix = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));

        let key1 = MatrixCacheKey::for_gate(StandardGate::H, (2, 2));
        let key2 = MatrixCacheKey::for_gate(StandardGate::X, (2, 2));
        let key3 = MatrixCacheKey::for_gate(StandardGate::Y, (2, 2));

        // Fill cache
        cache.put(key1.clone(), matrix.clone()).unwrap();
        cache.put(key2.clone(), matrix.clone()).unwrap();

        // Both should be present
        assert!(cache.contains_key(&key1));
        assert!(cache.contains_key(&key2));

        // Add third entry (should evict first)
        cache.put(key3.clone(), matrix.clone()).unwrap();

        // First should be evicted, others present
        assert!(!cache.contains_key(&key1));
        assert!(cache.contains_key(&key2));
        assert!(cache.contains_key(&key3));
    }

    #[test]
    fn test_get_or_compute() {
        let cache = MatrixCache::new();
        let key = MatrixCacheKey::for_gate(StandardGate::H, (2, 2));

        let mut compute_count = 0;
        let compute_fn = || {
            compute_count += 1;
            Ok(Array2::eye(2).mapv(|x| Complex64::new(x, 0.0)))
        };

        // First call should compute
        let matrix1 = cache.get_or_compute(key.clone(), compute_fn).unwrap();
        assert_eq!(compute_count, 1);

        // Second call should use cache
        let matrix2 = cache
            .get_or_compute(key.clone(), || {
                compute_count += 1;
                Ok(Array2::eye(2).mapv(|x| Complex64::new(x, 0.0)))
            })
            .unwrap();
        assert_eq!(compute_count, 1); // Should not increment
        assert_eq!(matrix1, matrix2);
    }

    #[test]
    fn test_age_based_eviction() {
        let config = CacheConfig {
            max_age: Duration::from_millis(10),
            enable_age_limit: true,
            enable_lru: false,
            enable_size_limit: false,
            ..Default::default()
        };

        let cache = MatrixCache::with_config(config);
        let key = MatrixCacheKey::for_gate(StandardGate::H, (2, 2));
        let matrix = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));

        // Put entry
        cache.put(key.clone(), matrix.clone()).unwrap();
        assert!(cache.contains_key(&key));

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Try to put another entry (should trigger age-based cleanup)
        let key2 = MatrixCacheKey::for_gate(StandardGate::X, (2, 2));
        cache.put(key2, matrix).unwrap();

        // Original entry should be evicted
        assert!(!cache.contains_key(&key));
    }

    #[test]
    fn test_cache_statistics() {
        let cache = MatrixCache::new();
        let key = MatrixCacheKey::for_gate(StandardGate::H, (2, 2));
        let matrix = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));

        // Initial stats
        let stats = cache.stats();
        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.total_accesses, 0);

        // Miss
        cache.get(&key);
        let stats = cache.stats();
        assert_eq!(stats.miss_rate(), 100.0);
        assert_eq!(stats.misses, 1);

        // Put and hit
        cache.put(key.clone(), matrix).unwrap();
        cache.get(&key);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 50.0);
    }
}
