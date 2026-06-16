//! SIMD Performance Demo
//!
//! This example demonstrates the performance benefits of SIMD-optimized
//! quantum state vector operations compared to scalar implementations.
//!
//! Only available on x86_64 targets (uses AVX/SSE intrinsics).

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    println!("SIMD performance demo is only available on x86_64 targets.");
    println!("Current target does not support the required SIMD intrinsics.");
}

#[cfg(target_arch = "x86_64")]
mod x86_impl {
    use myquat::{AdaptiveSimdOps, Result, SimdQuantumOps};
    use ndarray::Array1;
    use num_complex::Complex64;
    use std::time::Instant;

    pub fn run() -> Result<()> {
        println!("MyQuat SIMD 性能演示");
        println!("{}", "=".repeat(50));

        // Check SIMD availability
        demonstrate_simd_capability();

        // Demo 1: Probability calculation performance
        demonstrate_probability_performance()?;

        // Demo 2: Normalization performance
        demonstrate_normalization_performance()?;

        // Demo 3: Inner product performance
        demonstrate_inner_product_performance()?;

        // Demo 4: Large-scale performance comparison
        demonstrate_large_scale_performance()?;

        println!("\n演示完成！");
        Ok(())
    }

    fn demonstrate_simd_capability() {
        println!("\n1. SIMD 能力检测");
        println!("{}", "-".repeat(30));

        println!("CPU SIMD 支持:");
        println!("  AVX2 支持: {}", is_x86_feature_detected!("avx2"));
        println!("  FMA 支持: {}", is_x86_feature_detected!("fma"));
        println!("  SIMD 可用: {}", SimdQuantumOps::is_available());
        println!("  SIMD 宽度: {} 个复数", SimdQuantumOps::simd_width());

        if SimdQuantumOps::is_available() {
            println!("✅ SIMD 优化已启用");
        } else {
            println!("⚠️  SIMD 不可用，将使用标量运算");
        }
    }

    fn demonstrate_probability_performance() -> Result<()> {
        println!("\n2. 概率计算性能对比");
        println!("{}", "-".repeat(30));

        let sizes = vec![1024, 4096, 16384, 65536]; // 10, 12, 14, 16 qubits

        for &size in &sizes {
            let qubits = (size as f64).log2() as usize;
            println!("\n测试规模: {} 量子比特 ({} 个状态)", qubits, size);

            // Create a random-like state
            let state: Array1<Complex64> = (0..size)
                .map(|i| {
                    let phase = (i as f64) * 0.1;
                    Complex64::new(phase.cos(), phase.sin()) / (size as f64).sqrt()
                })
                .collect();

            // Benchmark scalar implementation
            let start = Instant::now();
            let scalar_probs = SimdQuantumOps::compute_probabilities_fallback(&state);
            let scalar_duration = start.elapsed();

            // Benchmark SIMD implementation
            let start = Instant::now();
            let simd_probs = AdaptiveSimdOps::compute_probabilities(&state);
            let simd_duration = start.elapsed();

            // Verify results are the same
            let max_diff = scalar_probs
                .iter()
                .zip(simd_probs.iter())
                .map(|(&a, &b)| (a - b).abs())
                .fold(0.0, f64::max);

            println!("  标量实现: {:?}", scalar_duration);
            println!("  SIMD实现: {:?}", simd_duration);

            if simd_duration.as_nanos() > 0 {
                let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
                println!("  加速比: {:.1}x", speedup);
            }

            println!("  最大误差: {:.2e}", max_diff);

            if max_diff < 1e-14 {
                println!("  ✅ 结果验证通过");
            } else {
                println!("  ❌ 结果验证失败");
            }
        }

        Ok(())
    }

    fn demonstrate_normalization_performance() -> Result<()> {
        println!("\n3. 归一化性能对比");
        println!("{}", "-".repeat(30));

        let sizes = vec![1024, 4096, 16384];

        for &size in &sizes {
            let qubits = (size as f64).log2() as usize;
            println!("\n测试规模: {} 量子比特 ({} 个状态)", qubits, size);

            // Create unnormalized state
            let unnormalized_state: Array1<Complex64> = (0..size)
                .map(|i| Complex64::new((i as f64) * 0.01, (i as f64) * 0.02))
                .collect();

            // Test scalar normalization
            let mut scalar_state = unnormalized_state.clone();
            let start = Instant::now();
            SimdQuantumOps::normalize_fallback(&mut scalar_state);
            let scalar_duration = start.elapsed();

            // Test SIMD normalization
            let mut simd_state = unnormalized_state.clone();
            let start = Instant::now();
            AdaptiveSimdOps::normalize(&mut simd_state);
            let simd_duration = start.elapsed();

            // Verify normalization
            let scalar_norm: f64 = scalar_state.iter().map(|z| z.norm_sqr()).sum();
            let simd_norm: f64 = simd_state.iter().map(|z| z.norm_sqr()).sum();

            println!("  标量实现: {:?}", scalar_duration);
            println!("  SIMD实现: {:?}", simd_duration);

            if simd_duration.as_nanos() > 0 {
                let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
                println!("  加速比: {:.1}x", speedup);
            }

            println!("  标量归一化: {:.10}", scalar_norm);
            println!("  SIMD归一化: {:.10}", simd_norm);

            if (scalar_norm - 1.0).abs() < 1e-14 && (simd_norm - 1.0).abs() < 1e-14 {
                println!("  ✅ 归一化验证通过");
            } else {
                println!("  ❌ 归一化验证失败");
            }
        }

        Ok(())
    }

    fn demonstrate_inner_product_performance() -> Result<()> {
        println!("\n4. 内积计算性能对比");
        println!("{}", "-".repeat(30));

        let sizes = vec![1024, 4096, 16384];

        for &size in &sizes {
            let qubits = (size as f64).log2() as usize;
            println!("\n测试规模: {} 量子比特 ({} 个状态)", qubits, size);

            // Create two normalized states
            let state1: Array1<Complex64> = {
                let unnormalized: Array1<Complex64> = (0..size)
                    .map(|i| Complex64::new((i as f64).sin(), (i as f64).cos()))
                    .collect();
                let norm = unnormalized
                    .iter()
                    .map(|z| z.norm_sqr())
                    .sum::<f64>()
                    .sqrt();
                unnormalized.mapv(|z| z / norm)
            };

            let state2: Array1<Complex64> = {
                let unnormalized: Array1<Complex64> = (0..size)
                    .map(|i| Complex64::new((i as f64 * 0.1).cos(), (i as f64 * 0.1).sin()))
                    .collect();
                let norm = unnormalized
                    .iter()
                    .map(|z| z.norm_sqr())
                    .sum::<f64>()
                    .sqrt();
                unnormalized.mapv(|z| z / norm)
            };

            // Test scalar inner product
            let start = Instant::now();
            let scalar_result = SimdQuantumOps::inner_product_fallback(&state1, &state2);
            let scalar_duration = start.elapsed();

            // Test SIMD inner product
            let start = Instant::now();
            let simd_result = AdaptiveSimdOps::inner_product(&state1, &state2);
            let simd_duration = start.elapsed();

            println!("  标量实现: {:?}", scalar_duration);
            println!("  SIMD实现: {:?}", simd_duration);

            if simd_duration.as_nanos() > 0 {
                let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
                println!("  加速比: {:.1}x", speedup);
            }

            let diff = (scalar_result - simd_result).norm();
            println!("  标量结果: {:.6}", scalar_result);
            println!("  SIMD结果: {:.6}", simd_result);
            println!("  误差: {:.2e}", diff);

            if diff < 1e-14 {
                println!("  ✅ 结果验证通过");
            } else {
                println!("  ❌ 结果验证失败");
            }
        }

        Ok(())
    }

    fn demonstrate_large_scale_performance() -> Result<()> {
        println!("\n5. 大规模性能测试");
        println!("{}", "-".repeat(30));

        let size = 65536; // 16 qubits
        let iterations = 10;

        println!("大规模测试: 16 量子比特, {} 次迭代", iterations);

        // Create test state
        let mut test_state: Array1<Complex64> = (0..size)
            .map(|i| {
                let phase = (i as f64) * 0.001;
                Complex64::new(phase.cos(), phase.sin())
            })
            .collect();

        // Normalize initially
        AdaptiveSimdOps::normalize(&mut test_state);

        println!("\n执行 {} 次概率计算...", iterations);

        // Benchmark probability calculations
        let start = Instant::now();
        for _ in 0..iterations {
            let _probs = SimdQuantumOps::compute_probabilities_fallback(&test_state);
        }
        let scalar_total = start.elapsed();

        let start = Instant::now();
        for _ in 0..iterations {
            let _probs = AdaptiveSimdOps::compute_probabilities(&test_state);
        }
        let simd_total = start.elapsed();

        println!("标量实现总时间: {:?}", scalar_total);
        println!("SIMD实现总时间: {:?}", simd_total);
        println!("平均标量时间: {:?}", scalar_total / iterations);
        println!("平均SIMD时间: {:?}", simd_total / iterations);

        if simd_total.as_nanos() > 0 {
            let speedup = scalar_total.as_nanos() as f64 / simd_total.as_nanos() as f64;
            println!("总体加速比: {:.1}x", speedup);
        }

        // Memory bandwidth estimation
        let bytes_per_iteration = size * std::mem::size_of::<Complex64>() * 2; // Read + write
        let total_bytes = bytes_per_iteration * iterations as usize;
        let scalar_bandwidth = total_bytes as f64 / scalar_total.as_secs_f64() / 1e9;
        let simd_bandwidth = total_bytes as f64 / simd_total.as_secs_f64() / 1e9;

        println!("\n内存带宽估算:");
        println!("标量实现: {:.1} GB/s", scalar_bandwidth);
        println!("SIMD实现: {:.1} GB/s", simd_bandwidth);

        Ok(())
    }
} // close x86_impl module

#[cfg(target_arch = "x86_64")]
fn main() -> myquat::Result<()> {
    x86_impl::run()
}
