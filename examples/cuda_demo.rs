//! CUDA GPU Acceleration Demo
//! Author: gA4ss
//!
//! Demonstrates CUDA GPU acceleration capabilities
//!
//! Build with: cargo build --example cuda_demo --features cuda
//! Run with: LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH cargo run --example cuda_demo --features cuda

use myquat::{Parameter, QuantumCircuit, Result};

#[cfg(feature = "cuda")]
use myquat::compute::local::cuda_backend::CudaBackend;

fn main() -> Result<()> {
    println!("=== MyQuat CUDA GPU加速演示 ===\n");

    #[cfg(feature = "cuda")]
    {
        cuda_demo()?;
    }

    #[cfg(not(feature = "cuda"))]
    {
        println!("CUDA支持未启用");
        println!("请使用 --features cuda 编译以启用CUDA支持");
        println!("\n编译命令:");
        println!("  cargo build --example cuda_demo --features cuda");
        println!("\n运行命令:");
        println!("  LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH \\");
        println!("  cargo run --example cuda_demo --features cuda");
    }

    Ok(())
}

#[cfg(feature = "cuda")]
fn cuda_demo() -> Result<()> {
    println!("1. CUDA设备初始化");
    println!("==================\n");

    match CudaBackend::new() {
        Ok(backend) => {
            println!("✓ CUDA设备初始化成功");
            println!("  设备编号: {}", backend.device_ordinal());
            println!();

            // Test GPU memory allocation
            test_gpu_memory(&backend)?;

            // Test probability calculation
            test_probability_calculation(&backend)?;

            // Test gate application
            test_gate_application(&backend)?;

            println!("\n=== CUDA演示完成 ===");
            println!("\nCUDA GPU加速已成功集成到MyQuat!");
            println!("后续可以实现完整的CUDA kernel以获得10x+加速。");
        }
        Err(e) => {
            println!("✗ CUDA设备初始化失败: {}", e);
            println!("\n可能的原因:");
            println!("  1. CUDA未安装或版本不匹配");
            println!("  2. LD_LIBRARY_PATH未正确设置");
            println!("  3. 没有NVIDIA GPU");
            println!("\n请检查:");
            println!("  - nvidia-smi 命令是否工作");
            println!("  - CUDA_PATH 环境变量");
            println!("  - LD_LIBRARY_PATH 包含 /usr/local/cuda/lib64");
        }
    }

    Ok(())
}

#[cfg(feature = "cuda")]
fn test_gpu_memory(backend: &CudaBackend) -> Result<()> {
    println!("2. GPU内存测试");
    println!("===============\n");

    let test_sizes = vec![10, 50, 100, 500]; // MB

    for size in test_sizes {
        match backend.test_gpu_memory(size) {
            Ok(_) => println!("  ✓ {} MB 分配成功", size),
            Err(e) => println!("  ✗ {} MB 分配失败: {}", size, e),
        }
    }

    println!();
    Ok(())
}

#[cfg(feature = "cuda")]
fn test_probability_calculation(backend: &CudaBackend) -> Result<()> {
    use ndarray::Array1;
    use num_complex::Complex64;

    println!("3. 概率计算测试");
    println!("================\n");

    // Create a 3-qubit state |000> + |111> (GHZ state)
    let mut state_data = vec![Complex64::new(0.0, 0.0); 8];
    state_data[0] = Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0);
    state_data[7] = Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0);
    let state = Array1::from_vec(state_data);

    match backend.compute_probabilities(&state) {
        Ok(probs) => {
            println!("  状态: (|000⟩ + |111⟩)/√2");
            println!("  概率分布:");
            for (i, p) in probs.iter().enumerate() {
                if *p > 1e-10 {
                    println!("    |{:03b}⟩: {:.4}", i, p);
                }
            }

            let sum: f64 = probs.iter().sum();
            println!("  概率和: {:.10} (应为1.0)", sum);

            if (sum - 1.0).abs() < 1e-9 {
                println!("  ✓ 归一化正确");
            }
        }
        Err(e) => {
            println!("  ✗ 概率计算失败: {}", e);
        }
    }

    println!();
    Ok(())
}

#[cfg(feature = "cuda")]
fn test_gate_application(backend: &CudaBackend) -> Result<()> {
    use ndarray::Array1;
    use num_complex::Complex64;

    println!("4. 量子门应用测试");
    println!("==================\n");

    // Create initial state |00>
    let mut state_data = vec![Complex64::new(0.0, 0.0); 4];
    state_data[0] = Complex64::new(1.0, 0.0);
    let mut state = Array1::from_vec(state_data);

    println!("  初始状态: |00⟩");

    // Hadamard gate matrix
    let h_sqrt = 1.0 / 2.0_f64.sqrt();
    let h_gate = [
        [Complex64::new(h_sqrt, 0.0), Complex64::new(h_sqrt, 0.0)],
        [Complex64::new(h_sqrt, 0.0), Complex64::new(-h_sqrt, 0.0)],
    ];

    // Apply H gate to first qubit
    match backend.apply_single_qubit_gate(&mut state, &h_gate, 0, 2) {
        Ok(_) => {
            println!("  ✓ Hadamard门应用到qubit 0");
            println!("  最终状态: (|00⟩ + |10⟩)/√2");

            // Check probabilities
            let prob_00 = state[0].norm_sqr();
            let prob_10 = state[2].norm_sqr();
            println!("  P(|00⟩) = {:.4}", prob_00);
            println!("  P(|10⟩) = {:.4}", prob_10);

            if (prob_00 - 0.5).abs() < 0.01 && (prob_10 - 0.5).abs() < 0.01 {
                println!("  ✓ 结果正确");
            }
        }
        Err(e) => {
            println!("  ✗ 门应用失败: {}", e);
        }
    }

    println!();
    Ok(())
}
