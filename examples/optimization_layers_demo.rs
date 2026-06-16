//! Optimization Layers Demo
//!
//! This example demonstrates how Rayon, SIMD, and GPU optimizations
//! work together at different levels of quantum computation.

use myquat::{performance_config::global_performance_manager, QuantumCircuit, Result};

fn main() -> Result<()> {
    println!("MyQuat 优化层次演示");
    println!("{}", "=".repeat(50));

    // Demo 1: 优化技术的独立性和协同性
    demonstrate_optimization_independence()?;

    // Demo 2: 不同计算阶段的优化应用
    demonstrate_computation_stages()?;

    // Demo 3: 优化技术的选择策略
    demonstrate_selection_strategy()?;

    // Demo 4: 性能对比分析
    demonstrate_performance_comparison()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_optimization_independence() -> Result<()> {
    println!("\n1. 优化技术的独立性和协同性");
    println!("{}", "-".repeat(40));

    println!("三种优化技术的作用层次:");
    println!(
        "
┌─────────────────────────────────────────────────────────┐
│                   量子电路执行                          │
├─────────────────────────────────────────────────────────┤
│  GPU 优化层 (设备级)                                   │
│  ├─ 整个状态向量 → GPU 显存                           │
│  ├─ 所有门操作 → GPU 核心并行                         │
│  └─ 结果 → CPU 内存                                   │
├─────────────────────────────────────────────────────────┤
│  Rayon 优化层 (线程级) - 在 CPU 上                    │
│  ├─ 状态向量分块 → 多个 CPU 线程                      │
│  ├─ 每个门操作 → 线程池并行执行                       │
│  └─ 结果合并 → 主线程                                 │
├─────────────────────────────────────────────────────────┤
│  SIMD 优化层 (指令级) - 在每个 CPU 线程内             │
│  ├─ 复数运算 → AVX2 向量指令                          │
│  ├─ 4个复数 → 同时计算                                │
│  └─ 向量化门矩阵乘法                                   │
└─────────────────────────────────────────────────────────┘"
    );

    println!("\n关键点:");
    println!("• GPU vs CPU: 互斥选择 (要么GPU要么CPU)");
    println!("• Rayon + SIMD: 协同工作 (CPU上的线程级+指令级优化)");
    println!("• 选择优先级: GPU > Rayon+SIMD > 标量CPU");

    Ok(())
}

fn demonstrate_computation_stages() -> Result<()> {
    println!("\n2. 不同计算阶段的优化应用");
    println!("{}", "-".repeat(40));

    let manager = global_performance_manager();

    println!("量子门操作的计算阶段:");
    println!(
        "
阶段1: 门矩阵准备
├─ 缓存查找 (Matrix Cache)
├─ 参数计算 (标量运算)
└─ 矩阵构建 (可能用SIMD优化)

阶段2: 状态向量操作 ⭐ 主要优化目标
├─ GPU路径: 整个状态 → GPU计算 → 结果返回
├─ CPU路径: 
│   ├─ Rayon: 状态分块到多线程
│   └─ SIMD: 每线程内向量化计算
└─ 标量路径: 单线程逐元素计算

阶段3: 结果处理
├─ 概率计算 (可用SIMD优化)
├─ 归一化 (可用Rayon+SIMD)
└─ 测量采样 (标量运算)"
    );

    // 演示不同规模的选择
    println!("\n不同问题规模的优化选择:");
    for qubits in [6, 10, 14] {
        let state_size = 1 << qubits;
        println!("\n{} 量子比特 ({} 状态):", qubits, state_size);

        if manager.should_use_gpu(qubits) {
            println!("  选择: GPU 优化");
            println!("  原因: 问题规模大，GPU并行度高");
            println!("  执行: 整个状态向量 → GPU显存 → 并行计算");
        } else if manager.should_use_parallel(qubits) {
            println!("  选择: Rayon + SIMD 协同优化");
            println!("  原因: 中等规模，多核CPU效率高");
            if manager.should_use_simd(state_size) {
                println!("  执行: 状态分块 → 多线程 → 每线程SIMD向量化");
            } else {
                println!("  执行: 状态分块 → 多线程 → 标量计算");
            }
        } else {
            println!("  选择: 标量 CPU");
            println!("  原因: 问题规模小，优化开销大于收益");
            println!("  执行: 单线程逐元素计算");
        }
    }

    Ok(())
}

fn demonstrate_selection_strategy() -> Result<()> {
    println!("\n3. 优化技术的选择策略");
    println!("{}", "-".repeat(40));

    println!("选择决策树:");
    println!(
        "
问题规模 >= 12 qubits?
├─ 是 → GPU可用?
│   ├─ 是 → 使用GPU (独占整个计算)
│   └─ 否 → 使用Rayon+SIMD
└─ 否 → 问题规模 >= 8 qubits?
    ├─ 是 → 使用Rayon+SIMD协同
    └─ 否 → 使用标量CPU"
    );

    println!("\n互斥关系:");
    println!("• GPU vs CPU: 完全互斥");
    println!("  - GPU模式: 状态向量完全在GPU上计算");
    println!("  - CPU模式: 状态向量在CPU上计算");

    println!("\n协同关系:");
    println!("• Rayon + SIMD: 在CPU模式下协同工作");
    println!("  - Rayon: 将状态向量分块给多个线程");
    println!("  - SIMD: 每个线程内使用向量指令加速");

    println!("\n性能特点:");
    println!("• GPU: 高延迟，高吞吐量 (适合大规模)");
    println!("• Rayon: 中延迟，中吞吐量 (适合中规模)");
    println!("• SIMD: 低延迟，中吞吐量 (适合向量运算)");

    Ok(())
}

fn demonstrate_performance_comparison() -> Result<()> {
    println!("\n4. 性能对比分析");
    println!("{}", "-".repeat(40));

    // 创建测试电路
    let qubits = 10;
    let mut circuit = QuantumCircuit::new(qubits, 0);

    // 添加一些门操作
    for i in 0..qubits {
        circuit.h(i)?;
    }
    for i in 0..qubits - 1 {
        circuit.cx(i, i + 1)?;
    }

    println!("测试电路: {} 量子比特, {} 个门", qubits, circuit.size());

    // 模拟不同优化策略的性能
    println!("\n理论性能分析:");

    let state_size = 1 << qubits;
    let operations_per_gate = state_size; // 简化估算
    let total_operations = circuit.size() * operations_per_gate;

    println!("• 总计算量: {} 次复数运算", total_operations);

    // 标量CPU
    let scalar_time_us = total_operations / 1_000_000; // 假设1M ops/sec
    println!("• 标量CPU: ~{}μs (基准)", scalar_time_us);

    // SIMD优化
    let simd_speedup = 4.0; // AVX2理论4x加速
    let simd_time_us = (scalar_time_us as f64 / simd_speedup) as u64;
    println!("• SIMD优化: ~{}μs ({:.1}x加速)", simd_time_us, simd_speedup);

    // Rayon并行
    let num_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let parallel_speedup = (num_cores as f64 * 0.8).min(8.0); // 考虑开销和扩展性
    let parallel_time_us = (scalar_time_us as f64 / parallel_speedup) as u64;
    println!(
        "• Rayon并行: ~{}μs ({:.1}x加速)",
        parallel_time_us, parallel_speedup
    );

    // Rayon + SIMD协同
    let combined_speedup = simd_speedup * parallel_speedup * 0.9; // 考虑协同效率
    let combined_time_us = (scalar_time_us as f64 / combined_speedup) as u64;
    println!(
        "• Rayon+SIMD: ~{}μs ({:.1}x加速)",
        combined_time_us, combined_speedup
    );

    // GPU加速
    let gpu_speedup = 50.0; // GPU理论加速比
    let gpu_overhead_us = 100; // GPU数据传输开销
    let gpu_time_us = (scalar_time_us as f64 / gpu_speedup) as u64 + gpu_overhead_us;
    println!(
        "• GPU加速: ~{}μs ({:.1}x加速，含传输开销)",
        gpu_time_us,
        scalar_time_us as f64 / gpu_time_us as f64
    );

    println!("\n实际选择策略:");
    let manager = global_performance_manager();
    if manager.should_use_gpu(qubits) {
        println!("✅ 当前会选择: GPU加速");
    } else if manager.should_use_parallel(qubits) && manager.should_use_simd(state_size) {
        println!("✅ 当前会选择: Rayon + SIMD 协同");
    } else if manager.should_use_parallel(qubits) {
        println!("✅ 当前会选择: Rayon 并行");
    } else {
        println!("✅ 当前会选择: 标量 CPU");
    }

    Ok(())
}
