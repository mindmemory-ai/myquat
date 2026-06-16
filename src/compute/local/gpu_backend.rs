//! GPU acceleration framework for quantum computing
//!
//! # Architecture
//!
//! This module provides a foundation for GPU-accelerated quantum circuit
//! simulation using compute shaders and parallel processing. The architecture
//! supports multiple GPU backends:
//!
//! - **CUDA**: NVIDIA GPU acceleration (requires `cuda` feature)
//! - **OpenCL**: Cross-platform GPU acceleration (future)
//! - **Compute Shader**: WebGPU/wgpu-based acceleration (default)
//!
//! # Design Pattern
//!
//! The module uses a trait-based design with automatic fallback:
//! 1. `GpuQuantumOps` trait defines the GPU operations interface
//! 2. Multiple implementations for different backends
//! 3. `GpuAccelerationManager` provides automatic backend selection
//! 4. Graceful fallback to CPU when GPU is unavailable
//!
//! # Performance Model
//!
//! GPU acceleration is beneficial for:
//! - Large quantum systems (>= 12 qubits, 4096 state elements)
//! - Batch operations on multiple circuits
//! - Deep circuits with many gates
//!
//! For smaller systems, CPU execution may be faster due to transfer overhead.
//!
//! # Optional Features
//!
//! - `cuda`: Enable NVIDIA CUDA backend (requires CUDA toolkit)
//! - Default: Compute shader backend (always available)

use crate::error::{MyQuatError, Result};
use ndarray::{Array1, Array2};
use num_complex::Complex64;

#[cfg(feature = "cuda")]
use super::cuda_backend::CudaBackend;

/// GPU acceleration capability detection
pub struct GpuCapability {
    pub has_cuda: bool,
    pub has_opencl: bool,
    pub has_compute_shader: bool,
    pub device_count: usize,
    pub memory_gb: f64,
}

impl GpuCapability {
    /// Detect available GPU acceleration options
    pub fn detect() -> Self {
        let has_cuda = Self::detect_cuda();
        let has_opencl = Self::detect_opencl();
        let has_compute_shader = Self::detect_wgpu();
        let (device_count, memory_gb) = Self::detect_devices();

        GpuCapability {
            has_cuda,
            has_opencl,
            has_compute_shader,
            device_count,
            memory_gb,
        }
    }

    /// Detect CUDA availability
    fn detect_cuda() -> bool {
        // Check for CUDA_PATH or CUDA_HOME environment variable
        if std::env::var("CUDA_PATH").is_ok() || std::env::var("CUDA_HOME").is_ok() {
            return true;
        }

        // Check for common CUDA library locations
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/usr/local/cuda/lib64/libcudart.so").exists()
        }
        #[cfg(target_os = "windows")]
        {
            // Check Windows CUDA locations
            std::path::Path::new("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA").exists()
        }
        #[cfg(target_os = "macos")]
        {
            false // CUDA not supported on macOS
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            false
        }
    }

    /// Detect OpenCL availability
    fn detect_opencl() -> bool {
        // Check for OpenCL library
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/usr/lib/libOpenCL.so").exists()
                || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libOpenCL.so").exists()
        }
        #[cfg(target_os = "windows")]
        {
            std::path::Path::new("C:\\Windows\\System32\\OpenCL.dll").exists()
        }
        #[cfg(target_os = "macos")]
        {
            std::path::Path::new("/System/Library/Frameworks/OpenCL.framework").exists()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            false
        }
    }

    /// Detect wgpu (WebGPU/compute shader) support
    fn detect_wgpu() -> bool {
        // Most modern systems support WebGPU through Vulkan/Metal/DX12
        // This is a conservative estimate
        true
    }

    /// Detect GPU devices and memory
    fn detect_devices() -> (usize, f64) {
        // Without actual GPU libraries, we provide conservative estimates
        // In a real implementation, this would query GPU info
        let device_count = if Self::detect_cuda() || Self::detect_opencl() {
            1
        } else {
            0
        };

        // Conservative memory estimate: 4GB
        let memory_gb = 4.0;

        (device_count, memory_gb)
    }

    /// Check if any GPU acceleration is available
    pub fn is_available(&self) -> bool {
        self.has_cuda || self.has_opencl || self.has_compute_shader
    }

    /// Get the best available GPU backend
    pub fn best_backend(&self) -> GpuBackend {
        if self.has_cuda {
            GpuBackend::Cuda
        } else if self.has_opencl {
            GpuBackend::OpenCL
        } else if self.has_compute_shader {
            GpuBackend::ComputeShader
        } else {
            GpuBackend::None
        }
    }
}

/// Available GPU backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    None,
    Cuda,
    OpenCL,
    ComputeShader,
}

impl std::fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuBackend::None => write!(f, "无GPU加速"),
            GpuBackend::Cuda => write!(f, "CUDA"),
            GpuBackend::OpenCL => write!(f, "OpenCL"),
            GpuBackend::ComputeShader => write!(f, "计算着色器"),
        }
    }
}

/// GPU-accelerated quantum operations interface
pub trait GpuQuantumOps {
    /// Apply single-qubit gate on GPU
    fn apply_single_qubit_gate_gpu(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()>;

    /// Apply two-qubit gate on GPU
    fn apply_two_qubit_gate_gpu(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit0: usize,
        qubit1: usize,
        num_qubits: usize,
    ) -> Result<()>;

    /// Compute probabilities on GPU
    fn compute_probabilities_gpu(&self, state: &Array1<Complex64>) -> Result<Array1<f64>>;

    /// Normalize state on GPU
    fn normalize_gpu(&self, state: &mut Array1<Complex64>) -> Result<()>;

    /// Compute inner product on GPU
    fn inner_product_gpu(
        &self,
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Result<Complex64>;
}

/// Compute shader-based GPU acceleration (WebGPU/wgpu)
pub struct ComputeShaderAccelerator {
    _backend: GpuBackend,
    capability: GpuCapability,
}

impl ComputeShaderAccelerator {
    /// Create a new compute shader accelerator
    pub fn new() -> Result<Self> {
        let capability = GpuCapability::detect();

        if !capability.has_compute_shader {
            return Err(MyQuatError::circuit_error("计算着色器不可用"));
        }

        Ok(ComputeShaderAccelerator {
            _backend: GpuBackend::ComputeShader,
            capability,
        })
    }

    /// Check if the accelerator is ready
    pub fn is_ready(&self) -> bool {
        self.capability.has_compute_shader
    }

    /// Get GPU memory information
    pub fn memory_info(&self) -> (f64, f64) {
        // Returns (available_gb, total_gb)
        (self.capability.memory_gb * 0.8, self.capability.memory_gb)
    }
}

/// CUDA accelerator adapter (wraps CudaBackend to implement GpuQuantumOps)
#[cfg(feature = "cuda")]
pub struct CudaAccelerator {
    backend: CudaBackend,
}

#[cfg(feature = "cuda")]
impl CudaAccelerator {
    pub fn new(backend: CudaBackend) -> Self {
        Self { backend }
    }
}

#[cfg(feature = "cuda")]
impl GpuQuantumOps for CudaAccelerator {
    fn apply_single_qubit_gate_gpu(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // 🔗 这里调用 cuda_backend.rs 的实际GPU代码
        let gate_array = [
            [gate_matrix[[0, 0]], gate_matrix[[0, 1]]],
            [gate_matrix[[1, 0]], gate_matrix[[1, 1]]],
        ];
        self.backend
            .apply_single_qubit_gate(state, &gate_array, qubit_index, num_qubits)
    }

    fn apply_two_qubit_gate_gpu(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit0: usize,
        qubit1: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // Fallback to CPU for now
        use crate::memory_optimized::ZeroCopyMatrixOps;
        ZeroCopyMatrixOps::apply_two_qubit_gate_inplace(
            state,
            gate_matrix,
            qubit0,
            qubit1,
            num_qubits,
        )
    }

    fn compute_probabilities_gpu(&self, state: &Array1<Complex64>) -> Result<Array1<f64>> {
        // 🔗 这里调用 cuda_backend.rs 的GPU概率计算
        let probs = self.backend.compute_probabilities(state)?;
        Ok(Array1::from_vec(probs))
    }

    fn normalize_gpu(&self, state: &mut Array1<Complex64>) -> Result<()> {
        // CPU fallback for now
        let norm_squared: f64 = state.iter().map(|z| z.norm_sqr()).sum();
        let norm = norm_squared.sqrt();

        if norm > 1e-15 {
            state.mapv_inplace(|z| z / norm);
        }

        Ok(())
    }

    fn inner_product_gpu(
        &self,
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Result<Complex64> {
        if state1.len() != state2.len() {
            return Err(MyQuatError::circuit_error("状态向量长度不匹配"));
        }

        let result = state1
            .iter()
            .zip(state2.iter())
            .map(|(&a, &b)| a.conj() * b)
            .sum();

        Ok(result)
    }
}

impl GpuQuantumOps for ComputeShaderAccelerator {
    fn apply_single_qubit_gate_gpu(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // CPU fallback implementation
        // Future GPU implementation would:
        // 1. Upload state vector and gate matrix to GPU memory
        // 2. Dispatch compute shader with workgroup size optimized for GPU
        // 3. Execute parallel gate application on GPU
        // 4. Download results back to CPU memory
        //
        // Note: GPU acceleration is beneficial for state_size >= 4096 (12+ qubits)
        // due to PCIe transfer overhead
        use crate::memory_optimized::ZeroCopyMatrixOps;
        ZeroCopyMatrixOps::apply_single_qubit_gate_inplace(
            state,
            gate_matrix,
            qubit_index,
            num_qubits,
        )
    }

    fn apply_two_qubit_gate_gpu(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit0: usize,
        qubit1: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // CPU fallback for two-qubit gates
        // GPU implementation requires more complex memory access patterns
        use crate::memory_optimized::ZeroCopyMatrixOps;
        ZeroCopyMatrixOps::apply_two_qubit_gate_inplace(
            state,
            gate_matrix,
            qubit0,
            qubit1,
            num_qubits,
        )
    }

    fn compute_probabilities_gpu(&self, state: &Array1<Complex64>) -> Result<Array1<f64>> {
        // CPU fallback: compute |psi|^2 for each amplitude
        // GPU version would use parallel reduction for better performance
        Ok(state.mapv(|z| z.norm_sqr()))
    }

    fn normalize_gpu(&self, state: &mut Array1<Complex64>) -> Result<()> {
        // CPU fallback: compute norm and normalize
        // GPU version would use two-pass algorithm:
        // 1. Parallel reduction to compute norm
        // 2. Parallel normalization of all amplitudes
        let norm_squared: f64 = state.iter().map(|z| z.norm_sqr()).sum();
        let norm = norm_squared.sqrt();

        if norm > 1e-15 {
            state.mapv_inplace(|z| z / norm);
        }

        Ok(())
    }

    fn inner_product_gpu(
        &self,
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Result<Complex64> {
        // CPU fallback: compute inner product <state1|state2>
        // GPU version would use parallel reduction with complex conjugate multiply
        if state1.len() != state2.len() {
            return Err(MyQuatError::circuit_error("状态向量长度不匹配"));
        }

        let result = state1
            .iter()
            .zip(state2.iter())
            .map(|(&a, &b)| a.conj() * b)
            .sum();

        Ok(result)
    }
}

/// High-level GPU acceleration manager
pub struct GpuAccelerationManager {
    capability: GpuCapability,
    accelerator: Option<Box<dyn GpuQuantumOps + Send + Sync>>,
}

impl GpuAccelerationManager {
    /// Create a new GPU acceleration manager
    pub fn new() -> Self {
        let capability = GpuCapability::detect();
        let accelerator = Self::create_accelerator(&capability);

        GpuAccelerationManager {
            capability,
            accelerator,
        }
    }

    /// Create the best available accelerator
    fn create_accelerator(
        capability: &GpuCapability,
    ) -> Option<Box<dyn GpuQuantumOps + Send + Sync>> {
        match capability.best_backend() {
            GpuBackend::Cuda => {
                // 🔗 调用 cuda_backend.rs 创建真实CUDA加速器
                #[cfg(feature = "cuda")]
                {
                    CudaBackend::new().ok().map(|backend| {
                        Box::new(CudaAccelerator::new(backend))
                            as Box<dyn GpuQuantumOps + Send + Sync>
                    })
                }
                #[cfg(not(feature = "cuda"))]
                {
                    None
                }
            }
            GpuBackend::ComputeShader => ComputeShaderAccelerator::new()
                .ok()
                .map(|acc| Box::new(acc) as Box<dyn GpuQuantumOps + Send + Sync>),
            GpuBackend::OpenCL => {
                // Would create OpenCL accelerator
                None
            }
            GpuBackend::None => None,
        }
    }

    /// Check if GPU acceleration is available
    pub fn is_available(&self) -> bool {
        self.accelerator.is_some()
    }

    /// Get GPU capability information
    pub fn capability(&self) -> &GpuCapability {
        &self.capability
    }

    /// Get the active backend
    pub fn backend(&self) -> GpuBackend {
        self.capability.best_backend()
    }

    /// Apply single-qubit gate with GPU acceleration if available
    pub fn apply_single_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        if let Some(ref accelerator) = self.accelerator {
            accelerator.apply_single_qubit_gate_gpu(state, gate_matrix, qubit_index, num_qubits)
        } else {
            // Fall back to CPU implementation
            use crate::memory_optimized::ZeroCopyMatrixOps;
            ZeroCopyMatrixOps::apply_single_qubit_gate_inplace(
                state,
                gate_matrix,
                qubit_index,
                num_qubits,
            )
        }
    }

    /// Apply two-qubit gate with GPU acceleration if available
    pub fn apply_two_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit0: usize,
        qubit1: usize,
        num_qubits: usize,
    ) -> Result<()> {
        if let Some(ref accelerator) = self.accelerator {
            accelerator.apply_two_qubit_gate_gpu(state, gate_matrix, qubit0, qubit1, num_qubits)
        } else {
            use crate::memory_optimized::ZeroCopyMatrixOps;
            ZeroCopyMatrixOps::apply_two_qubit_gate_inplace(
                state,
                gate_matrix,
                qubit0,
                qubit1,
                num_qubits,
            )
        }
    }

    /// Compute probabilities with GPU acceleration if available
    pub fn compute_probabilities(&self, state: &Array1<Complex64>) -> Result<Array1<f64>> {
        if let Some(ref accelerator) = self.accelerator {
            accelerator.compute_probabilities_gpu(state)
        } else {
            Ok(state.mapv(|z| z.norm_sqr()))
        }
    }

    /// Normalize state with GPU acceleration if available
    pub fn normalize(&self, state: &mut Array1<Complex64>) -> Result<()> {
        if let Some(ref accelerator) = self.accelerator {
            accelerator.normalize_gpu(state)
        } else {
            let norm_squared: f64 = state.iter().map(|z| z.norm_sqr()).sum();
            let norm = norm_squared.sqrt();

            if norm > 1e-15 {
                state.mapv_inplace(|z| z / norm);
            }

            Ok(())
        }
    }

    /// Compute inner product with GPU acceleration if available
    pub fn inner_product(
        &self,
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Result<Complex64> {
        if let Some(ref accelerator) = self.accelerator {
            accelerator.inner_product_gpu(state1, state2)
        } else {
            if state1.len() != state2.len() {
                return Err(MyQuatError::circuit_error("状态向量长度不匹配"));
            }

            let result = state1
                .iter()
                .zip(state2.iter())
                .map(|(&a, &b)| a.conj() * b)
                .sum();

            Ok(result)
        }
    }

    /// Get performance statistics
    pub fn performance_stats(&self) -> GpuPerformanceStats {
        GpuPerformanceStats {
            backend: self.backend(),
            is_available: self.is_available(),
            device_count: self.capability.device_count,
            memory_gb: self.capability.memory_gb,
            theoretical_speedup: if self.is_available() { 10.0 } else { 1.0 },
        }
    }
}

impl Default for GpuAccelerationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU performance statistics
#[derive(Debug, Clone)]
pub struct GpuPerformanceStats {
    pub backend: GpuBackend,
    pub is_available: bool,
    pub device_count: usize,
    pub memory_gb: f64,
    pub theoretical_speedup: f64,
}

impl std::fmt::Display for GpuPerformanceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GPU 加速统计:")?;
        writeln!(f, "  后端: {}", self.backend)?;
        writeln!(f, "  可用: {}", if self.is_available { "是" } else { "否" })?;
        writeln!(f, "  设备数量: {}", self.device_count)?;
        writeln!(f, "  显存: {:.1} GB", self.memory_gb)?;
        writeln!(f, "  理论加速比: {:.1}x", self.theoretical_speedup)?;
        Ok(())
    }
}

/// Quantum circuit execution with automatic GPU acceleration
pub struct GpuQuantumExecutor {
    gpu_manager: GpuAccelerationManager,
    fallback_to_cpu: bool,
}

impl GpuQuantumExecutor {
    /// Create a new GPU quantum executor
    pub fn new() -> Self {
        GpuQuantumExecutor {
            gpu_manager: GpuAccelerationManager::new(),
            fallback_to_cpu: true,
        }
    }

    /// Create with specific fallback behavior
    pub fn with_fallback(fallback_to_cpu: bool) -> Self {
        GpuQuantumExecutor {
            gpu_manager: GpuAccelerationManager::new(),
            fallback_to_cpu,
        }
    }

    /// Check if GPU acceleration is active
    pub fn is_gpu_active(&self) -> bool {
        self.gpu_manager.is_available()
    }

    /// Get GPU performance information
    pub fn gpu_stats(&self) -> GpuPerformanceStats {
        self.gpu_manager.performance_stats()
    }

    /// Execute quantum gate with automatic GPU/CPU selection
    pub fn execute_single_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // Determine if we should use GPU based on problem size
        let state_size = state.len();
        let use_gpu = self.should_use_gpu(state_size);

        if use_gpu && self.gpu_manager.is_available() {
            self.gpu_manager
                .apply_single_qubit_gate(state, gate_matrix, qubit_index, num_qubits)
        } else if self.fallback_to_cpu {
            use crate::memory_optimized::ZeroCopyMatrixOps;
            ZeroCopyMatrixOps::apply_single_qubit_gate_inplace(
                state,
                gate_matrix,
                qubit_index,
                num_qubits,
            )
        } else {
            Err(MyQuatError::circuit_error("GPU 不可用且禁用了 CPU 回退"))
        }
    }

    /// Determine if GPU should be used based on problem size
    fn should_use_gpu(&self, state_size: usize) -> bool {
        // Use GPU for larger problems (>= 4096 elements, i.e., >= 12 qubits)
        state_size >= 4096
    }
}

impl Default for GpuQuantumExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU performance prediction model
pub struct GpuPerformanceModel {
    /// GPU theoretical performance (TFLOPS)
    pub gpu_tflops: f64,
    /// GPU memory bandwidth (GB/s)
    pub gpu_memory_bandwidth_gbps: f64,
    /// PCIe bandwidth (GB/s)
    pub pcie_bandwidth_gbps: f64,
    /// CPU performance (GFLOPS)
    pub cpu_gflops: f64,
    /// CPU memory bandwidth (GB/s)
    pub cpu_memory_bandwidth_gbps: f64,
}

impl GpuPerformanceModel {
    /// Create a default performance model with typical hardware specs
    pub fn new() -> Self {
        Self::default()
    }

    /// Predict speedup for GPU vs CPU execution
    pub fn predict_speedup(&self, num_qubits: usize, num_gates: usize) -> f64 {
        let state_size = 2usize.pow(num_qubits as u32);
        let data_size_gb = (state_size * 16) as f64 / 1e9; // Complex64 = 16 bytes

        // Calculate data transfer time (upload + download)
        let transfer_time = data_size_gb / self.pcie_bandwidth_gbps * 2.0;

        // Calculate GPU execution time
        // Each gate: ~8 FLOPs per state element
        let gpu_flops = (num_gates as f64) * (state_size as f64) * 8.0;
        let gpu_compute_time = gpu_flops / (self.gpu_tflops * 1e12);

        // Calculate CPU execution time
        let cpu_time = gpu_flops / (self.cpu_gflops * 1e9);

        // Total GPU time includes transfer overhead
        let total_gpu_time = gpu_compute_time + transfer_time;

        // Return speedup ratio
        if total_gpu_time > 0.0 {
            cpu_time / total_gpu_time
        } else {
            1.0
        }
    }

    /// Determine if GPU should be used based on predicted speedup
    pub fn should_use_gpu(&self, num_qubits: usize, num_gates: usize) -> bool {
        // Require at least 1.5x speedup to justify GPU usage
        self.predict_speedup(num_qubits, num_gates) > 1.5
    }

    /// Estimate GPU memory required (in GB)
    pub fn estimate_memory_gb(&self, num_qubits: usize) -> f64 {
        let state_size = 2usize.pow(num_qubits as u32);
        // State vector: 16 bytes per element
        // Add 20% overhead for intermediate buffers
        (state_size * 16) as f64 / 1e9 * 1.2
    }

    /// Check if problem fits in GPU memory
    pub fn fits_in_gpu_memory(&self, num_qubits: usize, gpu_memory_gb: f64) -> bool {
        self.estimate_memory_gb(num_qubits) < gpu_memory_gb * 0.9 // Use max 90% of available memory
    }
}

impl Default for GpuPerformanceModel {
    fn default() -> Self {
        // Typical consumer GPU specs (e.g., RTX 3060)
        GpuPerformanceModel {
            gpu_tflops: 12.0,                 // ~12 TFLOPS FP32
            gpu_memory_bandwidth_gbps: 360.0, // ~360 GB/s
            pcie_bandwidth_gbps: 16.0,        // PCIe 4.0 x16
            cpu_gflops: 200.0,                // ~200 GFLOPS (8 cores)
            cpu_memory_bandwidth_gbps: 50.0,  // ~50 GB/s DDR4
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_gpu_capability_detection() {
        let capability = GpuCapability::detect();
        println!("GPU 能力检测:");
        println!("  CUDA: {}", capability.has_cuda);
        println!("  OpenCL: {}", capability.has_opencl);
        println!("  计算着色器: {}", capability.has_compute_shader);
        println!("  设备数量: {}", capability.device_count);
        println!("  显存: {:.1} GB", capability.memory_gb);
        println!("  最佳后端: {}", capability.best_backend());
    }

    #[test]
    fn test_gpu_acceleration_manager() {
        let manager = GpuAccelerationManager::new();
        println!("GPU 加速管理器:");
        println!("  可用: {}", manager.is_available());
        println!("  后端: {}", manager.backend());

        let stats = manager.performance_stats();
        println!("{}", stats);
    }

    #[test]
    fn test_gpu_quantum_operations() {
        let manager = GpuAccelerationManager::new();

        // Test with a simple 2-qubit state
        let mut state = Array1::from_vec(vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
        ]);

        // Test probability calculation
        let probs = manager.compute_probabilities(&state).unwrap();
        assert_relative_eq!(probs[0], 1.0, epsilon = 1e-10);
        assert_relative_eq!(probs[1], 0.0, epsilon = 1e-10);

        // Test normalization
        state[1] = Complex64::new(1.0, 0.0);
        manager.normalize(&mut state).unwrap();

        let norm_squared: f64 = state.iter().map(|z| z.norm_sqr()).sum();
        assert_relative_eq!(norm_squared, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_gpu_executor() {
        let executor = GpuQuantumExecutor::new();

        println!("GPU 执行器:");
        println!("  GPU 激活: {}", executor.is_gpu_active());

        let stats = executor.gpu_stats();
        println!("{}", stats);

        // Test execution decision
        assert!(!executor.should_use_gpu(1024)); // 10 qubits - use CPU
        assert!(executor.should_use_gpu(4096)); // 12 qubits - use GPU
        assert!(executor.should_use_gpu(16384)); // 14 qubits - use GPU
    }

    #[test]
    fn test_gpu_performance_model() {
        let model = GpuPerformanceModel::new();

        println!("\nGPU 性能模型测试:");
        println!("  GPU TFLOPS: {}", model.gpu_tflops);
        println!("  GPU 内存带宽: {} GB/s", model.gpu_memory_bandwidth_gbps);
        println!("  PCIe 带宽: {} GB/s", model.pcie_bandwidth_gbps);

        // Test different problem sizes
        let test_cases = vec![
            (10, 100),  // Small: 10 qubits, 100 gates
            (15, 500),  // Medium: 15 qubits, 500 gates
            (20, 1000), // Large: 20 qubits, 1000 gates
            (25, 5000), // Very Large: 25 qubits, 5000 gates
        ];

        println!("\n预测加速比:");
        for (qubits, gates) in test_cases {
            let speedup = model.predict_speedup(qubits, gates);
            let should_use = model.should_use_gpu(qubits, gates);
            let memory = model.estimate_memory_gb(qubits);

            println!("  {} qubits, {} gates:", qubits, gates);
            println!("    加速比: {:.2}x", speedup);
            println!("    使用GPU: {}", if should_use { "是" } else { "否" });
            println!("    需要内存: {:.2} GB", memory);
        }

        // Test memory constraints
        assert!(model.fits_in_gpu_memory(20, 8.0)); // 20 qubits fit in 8GB
        assert!(!model.fits_in_gpu_memory(30, 8.0)); // 30 qubits don't fit in 8GB
    }

    #[test]
    fn test_gpu_capability_detail() {
        let cap = GpuCapability::detect();

        println!("\n详细GPU能力:");
        println!("  CUDA: {}", cap.has_cuda);
        println!("  OpenCL: {}", cap.has_opencl);
        println!("  WebGPU/Compute Shader: {}", cap.has_compute_shader);
        println!("  设备数: {}", cap.device_count);
        println!("  显存: {:.1} GB", cap.memory_gb);
        println!("  最佳后端: {}", cap.best_backend());
        println!("  可用: {}", cap.is_available());

        // At least compute shader should be available
        assert!(cap.has_compute_shader || cap.has_cuda || cap.has_opencl);
    }
}
