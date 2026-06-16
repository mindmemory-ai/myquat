//! Memory Pool Demo
//!
//! This example demonstrates the performance benefits of using memory pools
//! to reduce allocation overhead in quantum circuit simulation.

use myquat::{Array1Pool, Array2Pool, QuantumMemoryPool, Result};
use ndarray::Array1;
use num_complex::Complex64;
use std::time::Instant;

fn main() -> Result<()> {
    println!("MyQuat 内存池演示");
    println!("{}", "=".repeat(50));

    // Demo 1: Basic pool usage
    demonstrate_basic_pool_usage()?;

    // Demo 2: Performance comparison
    demonstrate_performance_comparison()?;

    // Demo 3: Quantum memory pool
    demonstrate_quantum_memory_pool()?;

    // Demo 4: Memory reuse efficiency
    demonstrate_memory_reuse_efficiency()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_basic_pool_usage() -> Result<()> {
    println!("\n1. 基础内存池使用");
    println!("{}", "-".repeat(30));

    // Create pools
    let array1_pool = Array1Pool::new(1024, 10);
    let array2_pool = Array2Pool::new_for_quantum_gates(20);

    println!("创建内存池:");
    println!("  Array1Pool: 默认大小 1024, 最大池大小 10");
    println!("  Array2Pool: 量子门优化, 最大池大小 20");

    // Use Array1 pool
    println!("\n使用 Array1 池:");
    let mut arrays = Vec::new();
    for i in 0..5 {
        let array = array1_pool.get(1024);
        println!("  获取数组 {}: 长度 {}", i, array.len());
        arrays.push(array);
    }

    // Return arrays to pool
    println!("  归还数组到池中...");
    for (i, array) in arrays.into_iter().enumerate() {
        array1_pool.return_array(array);
        println!("  归还数组 {}", i);
    }

    let stats = array1_pool.stats();
    println!("  池统计: {} 个可用对象", stats.available_objects);

    // Use Array2 pool
    println!("\n使用 Array2 池:");
    let mut matrices = Vec::new();
    let sizes = vec![(2, 2), (4, 4), (2, 2), (4, 4)];

    for (i, &(rows, cols)) in sizes.iter().enumerate() {
        let matrix = array2_pool.get(rows, cols);
        println!("  获取矩阵 {}: {}x{}", i, rows, cols);
        matrices.push(matrix);
    }

    // Return matrices
    println!("  归还矩阵到池中...");
    for (i, matrix) in matrices.into_iter().enumerate() {
        array2_pool.return_matrix(matrix);
        println!("  归还矩阵 {}", i);
    }

    let stats = array2_pool.stats();
    println!("  池统计: {} 个可用对象", stats.available_objects);

    Ok(())
}

fn demonstrate_performance_comparison() -> Result<()> {
    println!("\n2. 性能对比测试");
    println!("{}", "-".repeat(30));

    let iterations = 1000;
    let array_size = 4096; // 12 qubits

    println!("测试配置: {} 次迭代, 数组大小 {}", iterations, array_size);

    // Test without pool (direct allocation)
    println!("\n不使用内存池 (直接分配):");
    let start = Instant::now();
    for _ in 0..iterations {
        let _array = Array1::<Complex64>::zeros(array_size);
        // Array is automatically dropped
    }
    let direct_duration = start.elapsed();
    println!("  执行时间: {:?}", direct_duration);
    println!("  平均每次分配: {:?}", direct_duration / iterations);

    // Test with pool
    println!("\n使用内存池:");
    let pool = Array1Pool::new(array_size, 50);

    let start = Instant::now();
    for _ in 0..iterations {
        let array = pool.get(array_size);
        pool.return_array(array);
    }
    let pooled_duration = start.elapsed();
    println!("  执行时间: {:?}", pooled_duration);
    println!("  平均每次操作: {:?}", pooled_duration / iterations);

    // Calculate speedup
    let speedup = direct_duration.as_nanos() as f64 / pooled_duration.as_nanos() as f64;
    println!("  性能提升: {:.1}x", speedup);

    let stats = pool.stats();
    println!("  最终池状态: {} 个可用对象", stats.available_objects);

    Ok(())
}

fn demonstrate_quantum_memory_pool() -> Result<()> {
    println!("\n3. 量子内存池演示");
    println!("{}", "-".repeat(30));

    let max_qubits = 10;
    let pool = QuantumMemoryPool::new_for_qubits(max_qubits);

    println!("创建量子内存池 (最大 {} 量子比特)", max_qubits);

    // Warm up the pool
    pool.warm_up();
    println!("预热内存池...");

    // Simulate quantum circuit execution
    println!("\n模拟量子电路执行:");
    for circuit_num in 1..=5 {
        println!("  电路 {}:", circuit_num);

        // Get state vector for different qubit counts
        let num_qubits = (circuit_num % max_qubits) + 1;
        let state = pool.get_state_vector(num_qubits);
        println!(
            "    获取 {} 量子比特状态向量 (大小: {})",
            num_qubits,
            state.len()
        );

        // Get some gate matrices
        let h_gate = pool.get_gate_matrix(2, 2);
        let cx_gate = pool.get_gate_matrix(4, 4);
        println!("    获取门矩阵: H(2x2), CX(4x4)");

        // Simulate some computation time
        std::thread::sleep(std::time::Duration::from_millis(1));

        // Return to pool
        pool.return_state_vector(state);
        pool.return_gate_matrix(h_gate);
        pool.return_gate_matrix(cx_gate);
        println!("    归还对象到池中");
    }

    // Show statistics
    let stats = pool.global_stats();
    println!("\n全局池统计:");
    println!("  状态向量分配: {}", stats.state_vectors_allocated);
    println!("  状态向量归还: {}", stats.state_vectors_returned);
    println!("  矩阵分配: {}", stats.matrices_allocated);
    println!("  矩阵归还: {}", stats.matrices_returned);
    println!("  重用效率: {:.1}%", stats.reuse_efficiency() * 100.0);
    println!(
        "  估算内存节省: {} bytes",
        stats.estimated_memory_savings_bytes()
    );

    Ok(())
}

fn demonstrate_memory_reuse_efficiency() -> Result<()> {
    println!("\n4. 内存重用效率测试");
    println!("{}", "-".repeat(30));

    let pool = QuantumMemoryPool::new_for_qubits(8);

    // Test different usage patterns
    let patterns = vec![
        ("顺序使用", vec![3, 3, 3, 3, 3]),
        ("混合大小", vec![2, 4, 3, 4, 2]),
        ("递增大小", vec![1, 2, 3, 4, 5]),
        ("随机模式", vec![3, 1, 4, 2, 3]),
    ];

    for (pattern_name, qubit_counts) in patterns {
        println!("\n测试模式: {}", pattern_name);
        pool.reset(); // Reset statistics

        let mut objects = Vec::new();

        // Allocate phase
        for &qubits in &qubit_counts {
            let state = pool.get_state_vector(qubits);
            let gate = pool.get_gate_matrix(2, 2);
            objects.push((state, gate));
            println!("  分配: {} 量子比特状态 + 2x2 矩阵", qubits);
        }

        // Return phase
        for (i, (state, gate)) in objects.into_iter().enumerate() {
            pool.return_state_vector(state);
            pool.return_gate_matrix(gate);
            println!("  归还对象 {}", i + 1);
        }

        // Second allocation phase (should reuse)
        let mut reused_objects = Vec::new();
        for &qubits in &qubit_counts {
            let state = pool.get_state_vector(qubits);
            let gate = pool.get_gate_matrix(2, 2);
            reused_objects.push((state, gate));
        }

        // Clean up
        for (state, gate) in reused_objects {
            pool.return_state_vector(state);
            pool.return_gate_matrix(gate);
        }

        let stats = pool.global_stats();
        println!("  重用效率: {:.1}%", stats.reuse_efficiency() * 100.0);
        println!(
            "  总分配: {} 个对象",
            stats.state_vectors_allocated + stats.matrices_allocated
        );
        println!(
            "  总归还: {} 个对象",
            stats.state_vectors_returned + stats.matrices_returned
        );
    }

    Ok(())
}

/// Helper function to simulate memory-intensive quantum operations
fn _simulate_quantum_algorithm(
    pool: &QuantumMemoryPool,
    num_qubits: usize,
    steps: usize,
) -> Result<()> {
    for step in 0..steps {
        // Get working memory
        let mut state = pool.get_state_vector(num_qubits);
        let temp_state = pool.get_state_vector(num_qubits);
        let gate_matrix = pool.get_gate_matrix(2, 2);

        // Simulate some computation
        for i in 0..state.len().min(100) {
            state[i] = Complex64::new(step as f64, i as f64);
        }

        // Return memory to pool
        pool.return_state_vector(state);
        pool.return_state_vector(temp_state);
        pool.return_gate_matrix(gate_matrix);
    }

    Ok(())
}
