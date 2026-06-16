// Adaptive Optimization Demo
// Author: gA4ss
//
// 演示自适应优化策略的功能，展示如何根据电路特征自动选择最优的优化Pass组合。

use myquat::adaptive_optimizer::{AdaptiveOptimizer, OptimizationStrategy};
use myquat::circuit::QuantumCircuit;
use myquat::circuit_analyzer::CircuitAnalyzer;
use myquat::error::Result;
use myquat::parameter::Parameter;

fn main() -> Result<()> {
    println!("=== 自适应电路优化演示 ===\n");

    // 示例1: 小型简单电路 - 应选择快速优化
    demo_small_circuit()?;

    // 示例2: CNOT密集型电路 - 应选择激进优化
    demo_cnot_heavy_circuit()?;

    // 示例3: 旋转门密集型电路 - 应选择平衡优化
    demo_rotation_heavy_circuit()?;

    // 示例4: 手动指定策略
    demo_manual_strategy()?;

    // 示例5: 详细分析和比较
    demo_detailed_analysis()?;

    Ok(())
}

/// 示例1: 小型简单电路
fn demo_small_circuit() -> Result<()> {
    println!("--- 示例1: 小型简单电路 ---");

    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.x(0)?;
    circuit.x(0)?; // 将被优化掉的门
    circuit.h(1)?;

    println!("原始电路:");
    println!("  量子比特数: {}", circuit.num_qubits());
    println!("  门数: {}", circuit.size());

    let optimizer = AdaptiveOptimizer::new().with_verbose(true);
    let mut opt_circuit = circuit.clone();
    let report = optimizer.optimize(&mut opt_circuit)?;

    println!("\n优化完成!");
    println!();

    Ok(())
}

/// 示例2: CNOT密集型电路
fn demo_cnot_heavy_circuit() -> Result<()> {
    println!("\n--- 示例2: CNOT密集型电路 ---");

    let mut circuit = QuantumCircuit::new(3, 0);

    // 创建大量CNOT门的电路
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.cx(1, 2)?;
    circuit.cx(0, 2)?;
    circuit.cx(1, 0)?;
    circuit.cx(2, 1)?;
    circuit.cx(0, 1)?;
    circuit.h(0)?;
    circuit.h(0)?; // 相邻的H门将被合并

    println!("原始电路:");
    println!("  量子比特数: {}", circuit.num_qubits());
    println!("  门数: {}", circuit.size());

    // 先分析电路特征
    let analyzer = CircuitAnalyzer::new();
    let profile = analyzer.analyze(&circuit);
    println!("\n电路特征:");
    println!("  CNOT门数: {}", profile.cx_count);
    println!("  CNOT密度: {:.2}%", profile.cx_density * 100.0);
    println!("  电路深度: {}", profile.depth);
    println!("  是否CNOT密集: {}", profile.is_cnot_heavy());

    let optimizer = AdaptiveOptimizer::new().with_verbose(true);
    let mut opt_circuit = circuit.clone();
    let report = optimizer.optimize(&mut opt_circuit)?;

    println!("\n优化完成!");
    println!();

    Ok(())
}

/// 示例3: 旋转门密集型电路
fn demo_rotation_heavy_circuit() -> Result<()> {
    println!("\n--- 示例3: 旋转门密集型电路 ---");

    let mut circuit = QuantumCircuit::new(3, 0);

    // 创建大量旋转门的电路
    circuit.rx(0, Parameter::Float(0.5))?;
    circuit.ry(0, Parameter::Float(0.3))?;
    circuit.rz(0, Parameter::Float(0.7))?;
    circuit.rx(1, Parameter::Float(0.2))?;
    circuit.ry(1, Parameter::Float(0.6))?;
    circuit.rz(1, Parameter::Float(0.4))?;
    circuit.cx(0, 1)?;
    circuit.rx(2, Parameter::Float(0.8))?;
    circuit.ry(2, Parameter::Float(0.1))?;
    circuit.rz(2, Parameter::Float(0.9))?;

    println!("原始电路:");
    println!("  量子比特数: {}", circuit.num_qubits());
    println!("  门数: {}", circuit.size());

    let analyzer = CircuitAnalyzer::new();
    let profile = analyzer.analyze(&circuit);
    println!("\n电路特征:");
    println!("  旋转门数: {}", profile.rotation_count);
    println!("  总门数: {}", profile.total_gates);
    println!("  电路深度: {}", profile.depth);
    println!("  是否旋转密集: {}", profile.is_rotation_heavy());

    let optimizer = AdaptiveOptimizer::new().with_verbose(true);
    let mut opt_circuit = circuit.clone();
    let report = optimizer.optimize(&mut opt_circuit)?;

    println!("\n优化完成!");
    println!();

    Ok(())
}

/// 示例4: 手动指定优化策略
fn demo_manual_strategy() -> Result<()> {
    println!("\n--- 示例4: 手动指定优化策略 ---");

    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.rx(0, Parameter::Float(0.5))?;
    circuit.ry(0, Parameter::Float(0.3))?;
    circuit.cx(0, 1)?;

    println!("原始电路门数: {}", circuit.size());

    let optimizer = AdaptiveOptimizer::new();

    // 测试不同的优化策略
    let strategies = vec![
        (OptimizationStrategy::Fast, "快速"),
        (OptimizationStrategy::Balanced, "平衡"),
        (OptimizationStrategy::Aggressive, "激进"),
    ];

    for (strategy, name) in strategies {
        let mut opt_circuit = circuit.clone();
        let report = optimizer.optimize_with_strategy(&mut opt_circuit, strategy)?;

        println!("\n{} 优化策略:", name);
        println!("  优化后门数: {}", report.optimized_gates);
        println!("  减少门数: {}", report.gates_reduced);
        println!("  减少率: {:.2}%", report.reduction_ratio * 100.0);
        println!("  执行时间: {:?}", report.execution_time);
        println!("  执行的Pass: {:?}", report.passes_executed);
    }

    println!();

    Ok(())
}

/// 示例5: 详细分析和比较
fn demo_detailed_analysis() -> Result<()> {
    println!("\n--- 示例5: 详细电路分析和优化比较 ---");

    // 创建一个复杂的电路
    let mut circuit = QuantumCircuit::new(4, 0);

    // Bell态对
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.h(2)?;
    circuit.cx(2, 3)?;

    // 添加一些可优化的门
    circuit.x(0)?;
    circuit.x(0)?; // 逆门对
    circuit.rx(1, Parameter::Float(0.5))?;
    circuit.ry(1, Parameter::Float(0.3))?;
    circuit.rz(1, Parameter::Float(0.7))?;
    circuit.cx(0, 1)?;
    circuit.cx(1, 2)?;
    circuit.h(3)?;
    circuit.h(3)?; // 逆门对

    println!("原始电路统计:");
    println!("  量子比特数: {}", circuit.num_qubits());
    println!("  总门数: {}", circuit.size());

    // 详细分析电路特征
    let analyzer = CircuitAnalyzer::new();
    let profile = analyzer.analyze(&circuit);

    println!("\n电路特征分析:");
    println!("  总门数: {}", profile.total_gates);
    println!("  单量子比特门: {}", profile.single_qubit_count);
    println!("  双量子比特门: {}", profile.two_qubit_count);
    println!("  CNOT门数: {}", profile.cx_count);
    println!("  旋转门数: {}", profile.rotation_count);
    println!("  电路深度: {}", profile.depth);
    println!("  电路宽度: {}", profile.circuit_width);
    println!("  活跃量子比特数: {}", profile.active_qubits);
    println!("  每比特平均门数: {:.2}", profile.gates_per_qubit);
    println!("  CNOT密度: {:.2}%", profile.cx_density * 100.0);
    println!("  量子比特连通性: {:.2}", profile.qubit_connectivity);

    println!("\n电路类型判断:");
    println!("  是否CNOT密集: {}", profile.is_cnot_heavy());
    println!("  是否旋转密集: {}", profile.is_rotation_heavy());
    println!("  是否参数化: {}", profile.is_parameterized());
    println!("  是否深层电路: {}", profile.is_deep());
    println!("  是否宽层电路: {}", profile.is_wide());

    // 使用自适应优化器
    let optimizer = AdaptiveOptimizer::new();

    // 让优化器自动选择策略
    println!("\n执行自适应优化...");
    let selected_strategy = optimizer.select_strategy(&profile);
    println!("自动选择的策略: {:?}", selected_strategy);

    let plan = optimizer.create_plan(&profile, selected_strategy);
    println!("\n优化计划:");
    println!("  将执行的Pass: {:?}", plan.pass_names);
    println!("  预期减少率: {:.2}%", plan.expected_reduction * 100.0);
    println!("  预估时间: {:?}", plan.estimated_time);
    println!("  选择原因: {}", plan.reason);

    // 执行优化
    let mut opt_circuit = circuit.clone();
    let report = optimizer.optimize(&mut opt_circuit)?;

    println!("\n优化结果:");
    println!("  原始门数: {}", report.original_gates);
    println!("  优化后门数: {}", report.optimized_gates);
    println!("  减少门数: {}", report.gates_reduced);
    println!("  减少率: {:.2}%", report.reduction_ratio * 100.0);
    println!("  原始深度: {}", report.original_depth);
    println!("  优化后深度: {}", report.optimized_depth);
    println!("  深度减少: {:.2}%", report.depth_reduction * 100.0);
    println!("  执行时间: {:?}", report.execution_time);
    println!("  实际执行的Pass: {:?}", report.passes_executed);

    // 计算优化效益
    let benefit = optimizer.evaluate_benefit(&circuit, &opt_circuit);
    println!("\n优化效益评分: {:.2}", benefit);

    println!();

    Ok(())
}
