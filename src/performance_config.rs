//! Performance configuration and optimization control
//!
//! This module provides a unified configuration system for controlling
//! various performance optimizations in MyQuat.

use crate::error::{MyQuatError, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceConfig {
    /// Parallel computing settings
    pub parallel: ParallelConfig,
    /// SIMD optimization settings
    pub simd: SimdConfig,
    /// GPU acceleration settings
    pub gpu: GpuConfig,
    /// Memory optimization settings
    pub memory: MemoryConfig,
    /// Cache settings
    pub cache: CacheConfig,
}

/// Parallel computing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Enable parallel computing
    pub enabled: bool,
    /// Number of threads (None = auto-detect)
    pub num_threads: Option<usize>,
    /// Minimum problem size to enable parallelization
    pub min_qubits_for_parallel: usize,
    /// Auto-enable based on problem size
    pub auto_enable: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        ParallelConfig {
            enabled: true,
            num_threads: None,          // Use all available cores
            min_qubits_for_parallel: 8, // Enable for >= 8 qubits
            auto_enable: true,
        }
    }
}

/// SIMD optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdConfig {
    /// Enable SIMD optimizations
    pub enabled: bool,
    /// Force enable even if CPU detection fails
    pub force_enable: bool,
    /// Minimum problem size to enable SIMD
    pub min_state_size: usize,
    /// Auto-enable based on CPU capabilities
    pub auto_enable: bool,
}

impl Default for SimdConfig {
    fn default() -> Self {
        SimdConfig {
            enabled: true,
            force_enable: false,
            min_state_size: 256, // Enable for >= 8 qubits
            auto_enable: true,
        }
    }
}

/// GPU acceleration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Enable GPU acceleration
    pub enabled: bool,
    /// Preferred GPU backend
    pub preferred_backend: GpuBackendPreference,
    /// Minimum problem size to enable GPU
    pub min_qubits_for_gpu: usize,
    /// Auto-enable based on problem size
    pub auto_enable: bool,
    /// Fallback to CPU if GPU fails
    pub fallback_to_cpu: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        GpuConfig {
            enabled: true,
            preferred_backend: GpuBackendPreference::Auto,
            min_qubits_for_gpu: 12, // Enable for >= 12 qubits
            auto_enable: true,
            fallback_to_cpu: true,
        }
    }
}

/// GPU backend preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuBackendPreference {
    Auto,
    Cuda,
    OpenCL,
    ComputeShader,
    Disabled,
}

/// Memory optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Enable memory pools
    pub enable_pools: bool,
    /// Enable zero-copy operations
    pub enable_zero_copy: bool,
    /// Memory pool size limits
    pub pool_max_size: usize,
    /// Enable memory-efficient state representation
    pub enable_efficient_state: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            enable_pools: true,
            enable_zero_copy: true,
            pool_max_size: 1000,
            enable_efficient_state: true,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable matrix computation cache
    pub enabled: bool,
    /// Maximum cache entries
    pub max_entries: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Cache eviction strategy
    pub eviction_strategy: CacheEvictionStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            enabled: true,
            max_entries: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            eviction_strategy: CacheEvictionStrategy::LRU,
        }
    }
}

/// Cache eviction strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionStrategy {
    LRU,
    LFU,
    FIFO,
    Random,
}

/// Global performance configuration manager
pub struct PerformanceManager {
    config: Arc<RwLock<PerformanceConfig>>,
}

impl PerformanceManager {
    /// Create a new performance manager with default configuration
    pub fn new() -> Self {
        PerformanceManager {
            config: Arc::new(RwLock::new(PerformanceConfig::default())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        PerformanceManager {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> PerformanceConfig {
        self.config.read().unwrap().clone()
    }

    /// Update configuration
    pub fn update_config<F>(&self, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut PerformanceConfig),
    {
        let mut config = self.config.write().unwrap();
        update_fn(&mut config);
        Ok(())
    }

    /// Load configuration from file
    pub fn load_from_file(&self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            MyQuatError::circuit_error(format!("Failed to read config file: {}", e))
        })?;

        let config: PerformanceConfig = toml::from_str(&content)
            .map_err(|e| MyQuatError::circuit_error(format!("Failed to parse config: {}", e)))?;

        *self.config.write().unwrap() = config;
        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let config = self.config.read().unwrap();
        let content = toml::to_string_pretty(&*config).map_err(|e| {
            MyQuatError::circuit_error(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            MyQuatError::circuit_error(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Check if parallel computing should be enabled for given problem size
    pub fn should_use_parallel(&self, num_qubits: usize) -> bool {
        let config = self.config.read().unwrap();
        config.parallel.enabled
            && (config.parallel.auto_enable
                && num_qubits >= config.parallel.min_qubits_for_parallel)
    }

    /// Check if SIMD should be enabled for given problem size
    pub fn should_use_simd(&self, state_size: usize) -> bool {
        let config = self.config.read().unwrap();
        config.simd.enabled
            && (config.simd.auto_enable && state_size >= config.simd.min_state_size)
            && (crate::compute::simd_ops::SimdQuantumOps::is_available()
                || config.simd.force_enable)
    }

    /// Check if GPU should be enabled for given problem size
    pub fn should_use_gpu(&self, num_qubits: usize) -> bool {
        let config = self.config.read().unwrap();
        config.gpu.enabled
            && (config.gpu.auto_enable && num_qubits >= config.gpu.min_qubits_for_gpu)
            && crate::compute::GpuCapability::detect().is_available()
    }

    /// Get recommended number of threads
    pub fn get_num_threads(&self) -> usize {
        let config = self.config.read().unwrap();
        config.parallel.num_threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        })
    }

    /// Create performance report
    pub fn create_performance_report(&self) -> PerformanceReport {
        let config = self.config.read().unwrap();

        PerformanceReport {
            parallel_available: rayon::current_num_threads() > 1,
            simd_available: crate::compute::simd_ops::SimdQuantumOps::is_available(),
            gpu_available: crate::compute::GpuCapability::detect().is_available(),
            config: config.clone(),
            recommendations: self.generate_recommendations(&config),
        }
    }

    /// Generate performance recommendations
    fn generate_recommendations(&self, config: &PerformanceConfig) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check CPU capabilities
        let num_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        if num_cores > 1 && !config.parallel.enabled {
            recommendations.push(format!(
                "Consider enabling parallel computing - {} CPU cores detected",
                num_cores
            ));
        }

        // Check SIMD capabilities
        if crate::compute::simd_ops::SimdQuantumOps::is_available() && !config.simd.enabled {
            recommendations.push(
                "Consider enabling SIMD optimizations - AVX2/FMA support detected".to_string(),
            );
        }

        // Check GPU capabilities
        let gpu_cap = crate::compute::GpuCapability::detect();
        if gpu_cap.is_available() && !config.gpu.enabled {
            recommendations.push(format!(
                "Consider enabling GPU acceleration - {} detected",
                gpu_cap.best_backend()
            ));
        }

        recommendations
    }
}

impl Default for PerformanceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance capabilities and recommendations report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub parallel_available: bool,
    pub simd_available: bool,
    pub gpu_available: bool,
    pub config: PerformanceConfig,
    pub recommendations: Vec<String>,
}

impl std::fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "MyQuat 性能配置报告")?;
        writeln!(f, "{}", "=".repeat(30))?;

        writeln!(f, "\n硬件能力:")?;
        writeln!(
            f,
            "  并行计算: {}",
            if self.parallel_available {
                "✅ 可用"
            } else {
                "❌ 不可用"
            }
        )?;
        writeln!(
            f,
            "  SIMD优化: {}",
            if self.simd_available {
                "✅ 可用"
            } else {
                "❌ 不可用"
            }
        )?;
        writeln!(
            f,
            "  GPU加速: {}",
            if self.gpu_available {
                "✅ 可用"
            } else {
                "❌ 不可用"
            }
        )?;

        writeln!(f, "\n当前配置:")?;
        writeln!(
            f,
            "  并行计算: {}",
            if self.config.parallel.enabled {
                "启用"
            } else {
                "禁用"
            }
        )?;
        writeln!(
            f,
            "  SIMD优化: {}",
            if self.config.simd.enabled {
                "启用"
            } else {
                "禁用"
            }
        )?;
        writeln!(
            f,
            "  GPU加速: {}",
            if self.config.gpu.enabled {
                "启用"
            } else {
                "禁用"
            }
        )?;
        writeln!(
            f,
            "  内存池: {}",
            if self.config.memory.enable_pools {
                "启用"
            } else {
                "禁用"
            }
        )?;
        writeln!(
            f,
            "  矩阵缓存: {}",
            if self.config.cache.enabled {
                "启用"
            } else {
                "禁用"
            }
        )?;

        if !self.recommendations.is_empty() {
            writeln!(f, "\n性能建议:")?;
            for (i, rec) in self.recommendations.iter().enumerate() {
                writeln!(f, "  {}. {}", i + 1, rec)?;
            }
        }

        Ok(())
    }
}

/// Global performance manager instance
static GLOBAL_PERFORMANCE_MANAGER: std::sync::OnceLock<PerformanceManager> =
    std::sync::OnceLock::new();

/// Get the global performance manager
pub fn global_performance_manager() -> &'static PerformanceManager {
    GLOBAL_PERFORMANCE_MANAGER.get_or_init(PerformanceManager::new)
}

/// Initialize global performance manager with custom configuration
pub fn init_global_performance_manager(config: PerformanceConfig) {
    let _ = GLOBAL_PERFORMANCE_MANAGER.set(PerformanceManager::with_config(config));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PerformanceConfig::default();
        assert!(config.parallel.enabled);
        assert!(config.simd.enabled);
        assert!(config.gpu.enabled);
        assert!(config.memory.enable_pools);
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_performance_manager() {
        let manager = PerformanceManager::new();

        // Test decision making
        assert!(manager.should_use_parallel(10)); // >= 8 qubits
        assert!(!manager.should_use_parallel(6)); // < 8 qubits

        let state_size = 1024; // 10 qubits
        let _should_use_simd = manager.should_use_simd(state_size);
        // Result depends on CPU capabilities

        assert!(manager.get_num_threads() >= 1);
    }

    #[test]
    fn test_config_serialization() {
        let config = PerformanceConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: PerformanceConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.parallel.enabled, deserialized.parallel.enabled);
        assert_eq!(config.simd.enabled, deserialized.simd.enabled);
    }

    #[test]
    fn test_performance_report() {
        let manager = PerformanceManager::new();
        let report = manager.create_performance_report();

        // Should have some content
        let report_str = format!("{}", report);
        assert!(report_str.contains("性能配置报告"));
        assert!(report_str.contains("硬件能力"));
    }
}
