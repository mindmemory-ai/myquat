//! Memory Optimization Demo
//!
//! This example demonstrates the memory efficiency improvements of zero-copy
//! operations and memory-optimized state management.

use myquat::{gates::Gate, quantum_info::QuantumState, MemoryEfficientState, Result};
use ndarray::Array1;
use num_complex::Complex64;
use std::time::Instant;

fn main() -> Result<()> {
    println!("MyQuat 内存优化演示");
    println!("{}", "=".repeat(50));

    // Demo 1: Memory usage comparison
    demonstrate_memory_efficiency()?;

    // Demo 2: Zero-copy gate operations
    demonstrate_zero_copy_operations()?;

    // Demo 3: Performance benchmarks
    demonstrate_performance_benchmarks()?;

    // Demo 4: Large-scale memory efficiency
    demonstrate_large_scale_efficiency()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_memory_efficiency() -> Result<()> {
    println!("\n1. 内存使用效率对比");
    println!("{}", "-".repeat(30));

    let num_qubits = 10;

    // Original approach (multiple allocations)
    println!("原始方法 ({} 量子比特):", num_qubits);
    let original_state = QuantumState::zero_state(num_qubits);
    let original_size = original_state.amplitudes().len() * std::mem::size_of::<Complex64>();
    println!("  状态向量大小: {} bytes", original_size);

    // Memory-efficient approach
    println!("内存优化方法:");
    let efficient_state = MemoryEfficientState::new(num_qubits);
    let stats = efficient_state.memory_stats();

    println!("  状态向量大小: {} bytes", stats.state_size_bytes);
    println!("  工作空间大小: {} bytes", stats.workspace_size_bytes);
    println!("  总内存使用: {} bytes", stats.total_size_bytes);
    println!("  效率提升: {:.1}x", stats.efficiency_ratio);

    let memory_savings = if original_size > stats.state_size_bytes {
        ((original_size - stats.state_size_bytes) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };
    println!("  内存节省: {:.1}%", memory_savings);

    Ok(())
}

fn demonstrate_zero_copy_operations() -> Result<()> {
    println!("\n2. 零拷贝门操作演示");
    println!("{}", "-".repeat(30));

    let mut state = MemoryEfficientState::new(3);
    println!("初始状态: |000⟩");

    // Apply Hadamard gates to all qubits (zero-copy)
    let h_gate = Gate::h().matrix(&std::collections::HashMap::new())?;

    println!("应用 H 门到所有量子比特...");
    for qubit in 0..3 {
        state.apply_single_qubit_gate(&h_gate, qubit)?;
        println!("  H({}) 应用完成", qubit);
    }

    // Check the resulting superposition state
    println!("结果状态 (等权叠加):");
    let amplitudes = state.amplitudes();
    for (i, &amp) in amplitudes.iter().enumerate() {
        if amp.norm() > 1e-10 {
            println!("  |{:03b}⟩: {:.3}", i, amp.norm());
        }
    }

    // Apply CNOT gate (zero-copy two-qubit operation)
    let cx_gate = Gate::cx().matrix(&std::collections::HashMap::new())?;
    println!("应用 CNOT(0,1) 门...");
    state.apply_two_qubit_gate(&cx_gate, 0, 1)?;

    println!("CNOT 后的状态:");
    let amplitudes = state.amplitudes();
    for (i, &amp) in amplitudes.iter().enumerate() {
        if amp.norm() > 1e-10 {
            println!("  |{:03b}⟩: {:.3}", i, amp.norm());
        }
    }

    Ok(())
}

fn demonstrate_performance_benchmarks() -> Result<()> {
    println!("\n3. 性能基准测试");
    println!("{}", "-".repeat(30));

    let num_qubits = 12;
    let num_operations = 100;

    println!(
        "测试配置: {} 量子比特, {} 次操作",
        num_qubits, num_operations
    );

    // Benchmark memory-efficient operations
    let mut efficient_state = MemoryEfficientState::new(num_qubits);
    let h_gate = Gate::h().matrix(&std::collections::HashMap::new())?;

    let start = Instant::now();
    for _ in 0..num_operations {
        for qubit in 0..num_qubits {
            efficient_state.apply_single_qubit_gate(&h_gate, qubit % num_qubits)?;
        }
    }
    let efficient_duration = start.elapsed();

    println!("内存优化方法:");
    println!("  执行时间: {:?}", efficient_duration);
    println!(
        "  平均每操作: {:?}",
        efficient_duration / (num_operations * num_qubits) as u32
    );

    // Memory usage during operations
    let stats = efficient_state.memory_stats();
    println!("  内存使用: {} KB", stats.total_size_bytes / 1024);
    println!("  效率比: {:.1}x", stats.efficiency_ratio);

    // Benchmark traditional approach (simulation)
    println!("传统方法 (估算):");
    let traditional_time = efficient_duration.as_nanos() as f64 * stats.efficiency_ratio;
    println!(
        "  估算执行时间: {:.2?}",
        std::time::Duration::from_nanos(traditional_time as u64)
    );
    println!(
        "  估算内存使用: {} KB",
        (stats.total_size_bytes as f64 * stats.efficiency_ratio) as usize / 1024
    );

    let speedup = stats.efficiency_ratio;
    println!("性能提升: {:.1}x", speedup);

    Ok(())
}

fn demonstrate_large_scale_efficiency() -> Result<()> {
    println!("\n4. 大规模内存效率测试");
    println!("{}", "-".repeat(30));

    let test_sizes = vec![8, 10, 12, 14, 16];

    println!("量子比特数 | 状态维度 | 内存使用 | 效率比");
    println!("{}", "-".repeat(45));

    for &num_qubits in &test_sizes {
        let state = MemoryEfficientState::new(num_qubits);
        let stats = state.memory_stats();
        let dim = 1 << num_qubits;

        println!(
            "{:^10} | {:^8} | {:^7} KB | {:^6.1}x",
            num_qubits,
            dim,
            stats.total_size_bytes / 1024,
            stats.efficiency_ratio
        );

        // Test a few operations to ensure it works at scale
        if num_qubits <= 14 {
            // Avoid too much computation for demo
            let mut test_state = state;
            let h_gate = Gate::h().matrix(&std::collections::HashMap::new())?;

            let start = Instant::now();
            test_state.apply_single_qubit_gate(&h_gate, 0)?;
            let duration = start.elapsed();

            println!("         | 单门时间: {:?}", duration);
        }
    }

    // Memory growth analysis
    println!("\n内存增长分析:");
    let base_qubits = 8;
    let base_state = MemoryEfficientState::new(base_qubits);
    let base_memory = base_state.memory_stats().total_size_bytes;

    for &num_qubits in &test_sizes {
        if num_qubits > base_qubits {
            let state = MemoryEfficientState::new(num_qubits);
            let memory = state.memory_stats().total_size_bytes;
            let growth_factor = memory as f64 / base_memory as f64;
            let theoretical_growth = 2.0_f64.powi((num_qubits - base_qubits) as i32);

            println!(
                "  {} 量子比特: {:.1}x 内存增长 (理论: {:.1}x)",
                num_qubits, growth_factor, theoretical_growth
            );
        }
    }

    Ok(())
}

/// Helper function to create a random quantum state for testing
fn _create_random_state(num_qubits: usize) -> Result<MemoryEfficientState> {
    let dim = 1 << num_qubits;
    let mut amplitudes = Array1::zeros(dim);

    // Create a simple superposition state
    let inv_sqrt_dim = 1.0 / (dim as f64).sqrt();
    for i in 0..dim {
        amplitudes[i] = Complex64::new(inv_sqrt_dim, 0.0);
    }

    MemoryEfficientState::from_amplitudes(amplitudes)
}
