//! Memory pool utilities for efficient allocation management
//!
//! This module provides object pools and memory pools to reduce allocation overhead
//! in high-performance quantum circuit simulation.

// Removed unused import
use ndarray::{Array1, Array2};
use num_complex::Complex64;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Object pool for reusing `Array1<Complex64>` objects
pub struct Array1Pool {
    /// Pool of available arrays
    pool: Mutex<VecDeque<Array1<Complex64>>>,
    /// Maximum pool size to prevent unbounded growth
    max_size: usize,
    /// Default array size for new allocations
    default_size: usize,
}

impl Array1Pool {
    /// Create a new Array1 pool
    pub fn new(default_size: usize, max_pool_size: usize) -> Self {
        Array1Pool {
            pool: Mutex::new(VecDeque::new()),
            max_size: max_pool_size,
            default_size,
        }
    }

    /// Get an array from the pool or create a new one
    pub fn get(&self, size: usize) -> Array1<Complex64> {
        let mut pool = self.pool.lock().unwrap();

        // Try to find a suitable array in the pool
        for _ in 0..pool.len() {
            if let Some(mut array) = pool.pop_front() {
                if array.len() == size {
                    // Found a perfect match, zero it out and return
                    array.fill(Complex64::new(0.0, 0.0));
                    return array;
                } else if array.len() >= size {
                    // Array is larger than needed, resize it
                    array = Array1::zeros(size);
                    return array;
                } else {
                    // Array is too small, put it back and continue searching
                    pool.push_back(array);
                }
            }
        }

        // No suitable array found, create a new one
        Array1::zeros(size)
    }

    /// Return an array to the pool
    pub fn return_array(&self, array: Array1<Complex64>) {
        let mut pool = self.pool.lock().unwrap();

        // Only add to pool if we haven't exceeded max size
        if pool.len() < self.max_size {
            pool.push_back(array);
        }
        // Otherwise, let the array be dropped (freed)
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let pool = self.pool.lock().unwrap();
        PoolStats {
            available_objects: pool.len(),
            max_size: self.max_size,
            default_size: self.default_size,
        }
    }

    /// Clear the pool and free all cached arrays
    pub fn clear(&self) {
        let mut pool = self.pool.lock().unwrap();
        pool.clear();
    }
}

/// Object pool for reusing `Array2<Complex64>` matrices
pub struct Array2Pool {
    /// Pool of available matrices
    pool: Mutex<VecDeque<Array2<Complex64>>>,
    /// Maximum pool size
    max_size: usize,
    /// Common matrix sizes to pre-allocate
    common_sizes: Vec<(usize, usize)>,
}

impl Array2Pool {
    /// Create a new Array2 pool with common quantum gate sizes
    pub fn new_for_quantum_gates(max_pool_size: usize) -> Self {
        Array2Pool {
            pool: Mutex::new(VecDeque::new()),
            max_size: max_pool_size,
            common_sizes: vec![
                (2, 2),   // Single-qubit gates
                (4, 4),   // Two-qubit gates
                (8, 8),   // Three-qubit gates
                (16, 16), // Four-qubit gates
            ],
        }
    }

    /// Get a matrix from the pool or create a new one
    pub fn get(&self, rows: usize, cols: usize) -> Array2<Complex64> {
        let mut pool = self.pool.lock().unwrap();

        // Try to find a suitable matrix in the pool
        for _ in 0..pool.len() {
            if let Some(mut matrix) = pool.pop_front() {
                if matrix.dim() == (rows, cols) {
                    // Perfect match, zero it out and return
                    matrix.fill(Complex64::new(0.0, 0.0));
                    return matrix;
                } else if matrix.nrows() >= rows && matrix.ncols() >= cols {
                    // Matrix is larger, create a new one of exact size
                    return Array2::zeros((rows, cols));
                } else {
                    // Matrix is too small, put it back
                    pool.push_back(matrix);
                }
            }
        }

        // No suitable matrix found, create a new one
        Array2::zeros((rows, cols))
    }

    /// Return a matrix to the pool
    pub fn return_matrix(&self, matrix: Array2<Complex64>) {
        let mut pool = self.pool.lock().unwrap();

        // Only add common sizes to avoid memory bloat
        let dim = matrix.dim();
        if self.common_sizes.contains(&dim) && pool.len() < self.max_size {
            pool.push_back(matrix);
        }
    }

    /// Pre-populate the pool with common matrix sizes
    pub fn pre_populate(&self) {
        let mut pool = self.pool.lock().unwrap();

        for &(rows, cols) in &self.common_sizes {
            if pool.len() < self.max_size {
                pool.push_back(Array2::zeros((rows, cols)));
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let pool = self.pool.lock().unwrap();
        PoolStats {
            available_objects: pool.len(),
            max_size: self.max_size,
            default_size: 0, // Not applicable for matrices
        }
    }
}

/// Global memory pool manager for quantum computations
pub struct QuantumMemoryPool {
    /// Pool for state vectors
    array1_pool: Arc<Array1Pool>,
    /// Pool for gate matrices
    array2_pool: Arc<Array2Pool>,
    /// Statistics tracking
    stats: Mutex<GlobalPoolStats>,
}

impl QuantumMemoryPool {
    /// Create a new quantum memory pool with default settings
    pub fn new() -> Self {
        QuantumMemoryPool {
            array1_pool: Arc::new(Array1Pool::new(1024, 50)), // Default for ~10 qubits
            array2_pool: Arc::new(Array2Pool::new_for_quantum_gates(100)),
            stats: Mutex::new(GlobalPoolStats::new()),
        }
    }

    /// Create a memory pool optimized for a specific number of qubits
    pub fn new_for_qubits(max_qubits: usize) -> Self {
        let default_state_size = 1 << max_qubits;
        let pool_size = (max_qubits * 10).min(100); // Reasonable pool size

        QuantumMemoryPool {
            array1_pool: Arc::new(Array1Pool::new(default_state_size, pool_size)),
            array2_pool: Arc::new(Array2Pool::new_for_quantum_gates(pool_size)),
            stats: Mutex::new(GlobalPoolStats::new()),
        }
    }

    /// Get a state vector from the pool
    pub fn get_state_vector(&self, num_qubits: usize) -> Array1<Complex64> {
        let size = 1 << num_qubits;
        let array = self.array1_pool.get(size);

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.state_vectors_allocated += 1;
        }

        array
    }

    /// Return a state vector to the pool
    pub fn return_state_vector(&self, array: Array1<Complex64>) {
        self.array1_pool.return_array(array);

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.state_vectors_returned += 1;
        }
    }

    /// Get a gate matrix from the pool
    pub fn get_gate_matrix(&self, rows: usize, cols: usize) -> Array2<Complex64> {
        let matrix = self.array2_pool.get(rows, cols);

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.matrices_allocated += 1;
        }

        matrix
    }

    /// Return a gate matrix to the pool
    pub fn return_gate_matrix(&self, matrix: Array2<Complex64>) {
        self.array2_pool.return_matrix(matrix);

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.matrices_returned += 1;
        }
    }

    /// Pre-populate pools for better performance
    pub fn warm_up(&self) {
        self.array2_pool.pre_populate();
    }

    /// Get comprehensive pool statistics
    pub fn global_stats(&self) -> GlobalPoolStats {
        let stats = self.stats.lock().unwrap();
        let mut global_stats = stats.clone();

        // Add current pool states
        let array1_stats = self.array1_pool.stats();
        let array2_stats = self.array2_pool.stats();

        global_stats.current_array1_pool_size = array1_stats.available_objects;
        global_stats.current_array2_pool_size = array2_stats.available_objects;

        global_stats
    }

    /// Clear all pools and reset statistics
    pub fn reset(&self) {
        self.array1_pool.clear();
        self.array2_pool.stats(); // Just to ensure pool is accessible

        let mut stats = self.stats.lock().unwrap();
        *stats = GlobalPoolStats::new();
    }
}

impl Default for QuantumMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for individual pools
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available_objects: usize,
    pub max_size: usize,
    pub default_size: usize,
}

/// Global statistics for all memory pools
#[derive(Debug, Clone)]
pub struct GlobalPoolStats {
    pub state_vectors_allocated: usize,
    pub state_vectors_returned: usize,
    pub matrices_allocated: usize,
    pub matrices_returned: usize,
    pub current_array1_pool_size: usize,
    pub current_array2_pool_size: usize,
}

impl GlobalPoolStats {
    fn new() -> Self {
        GlobalPoolStats {
            state_vectors_allocated: 0,
            state_vectors_returned: 0,
            matrices_allocated: 0,
            matrices_returned: 0,
            current_array1_pool_size: 0,
            current_array2_pool_size: 0,
        }
    }

    /// Calculate the reuse efficiency
    pub fn reuse_efficiency(&self) -> f64 {
        let total_allocated = self.state_vectors_allocated + self.matrices_allocated;
        let total_returned = self.state_vectors_returned + self.matrices_returned;

        if total_allocated == 0 {
            0.0
        } else {
            total_returned as f64 / total_allocated as f64
        }
    }

    /// Calculate memory savings (rough estimate)
    pub fn estimated_memory_savings_bytes(&self) -> usize {
        // Rough estimate: each reused object saves one allocation
        let state_vector_savings =
            self.state_vectors_returned * 1024 * std::mem::size_of::<Complex64>();
        let matrix_savings = self.matrices_returned * 16 * std::mem::size_of::<Complex64>();

        state_vector_savings + matrix_savings
    }
}

/// RAII wrapper for automatic pool management
pub struct PooledArray1 {
    array: Option<Array1<Complex64>>,
    pool: Arc<Array1Pool>,
}

impl PooledArray1 {
    /// Create a new pooled array
    pub fn new(pool: Arc<Array1Pool>, size: usize) -> Self {
        PooledArray1 {
            array: Some(pool.get(size)),
            pool,
        }
    }

    /// Get a reference to the array
    pub fn as_ref(&self) -> &Array1<Complex64> {
        self.array.as_ref().unwrap()
    }

    /// Get a mutable reference to the array
    pub fn as_mut(&mut self) -> &mut Array1<Complex64> {
        self.array.as_mut().unwrap()
    }
}

impl Drop for PooledArray1 {
    fn drop(&mut self) {
        if let Some(array) = self.array.take() {
            self.pool.return_array(array);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array1_pool() {
        let pool = Array1Pool::new(1024, 10);

        // Get an array
        let array1 = pool.get(1024);
        assert_eq!(array1.len(), 1024);

        // Return it
        pool.return_array(array1);

        // Get another array (should reuse the first one)
        let array2 = pool.get(1024);
        assert_eq!(array2.len(), 1024);

        let stats = pool.stats();
        assert_eq!(stats.default_size, 1024);
        assert!(stats.available_objects <= stats.max_size);
    }

    #[test]
    fn test_array2_pool() {
        let pool = Array2Pool::new_for_quantum_gates(10);

        // Get a 2x2 matrix (single-qubit gate)
        let matrix1 = pool.get(2, 2);
        assert_eq!(matrix1.dim(), (2, 2));

        // Return it
        pool.return_matrix(matrix1);

        // Get another 2x2 matrix
        let matrix2 = pool.get(2, 2);
        assert_eq!(matrix2.dim(), (2, 2));

        let stats = pool.stats();
        assert!(stats.available_objects <= stats.max_size);
    }

    #[test]
    fn test_quantum_memory_pool() {
        let pool = QuantumMemoryPool::new_for_qubits(5);

        // Get a state vector for 3 qubits
        let state = pool.get_state_vector(3);
        assert_eq!(state.len(), 8);

        // Get a gate matrix
        let gate = pool.get_gate_matrix(2, 2);
        assert_eq!(gate.dim(), (2, 2));

        // Return them
        pool.return_state_vector(state);
        pool.return_gate_matrix(gate);

        // Check statistics
        let stats = pool.global_stats();
        assert_eq!(stats.state_vectors_allocated, 1);
        assert_eq!(stats.state_vectors_returned, 1);
        assert_eq!(stats.matrices_allocated, 1);
        assert_eq!(stats.matrices_returned, 1);
        assert_eq!(stats.reuse_efficiency(), 1.0);
    }

    #[test]
    fn test_pooled_array1_raii() {
        let pool = Arc::new(Array1Pool::new(1024, 10));

        {
            let _pooled = PooledArray1::new(pool.clone(), 1024);
            // Array is automatically returned when pooled goes out of scope
        }

        // Pool should now have one array available
        let stats = pool.stats();
        assert_eq!(stats.available_objects, 1);
    }
}
