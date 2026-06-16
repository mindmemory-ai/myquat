//! Quantum Algorithms Demo
//!
//! This example demonstrates the quantum algorithms library including
//! QFT, Grover search, VQE, QAOA, and Phase Estimation.

use myquat::{
    AlgorithmRunner, GroverSearch, PhaseEstimation, QuantumCircuit, Result, StateVectorSimulator,
    VQEAnsatz, QAOA, QFT,
};
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("MyQuat 量子算法库演示");
    println!("{}", "=".repeat(50));

    // Demo 1: 量子傅里叶变换 (QFT)
    demonstrate_qft()?;

    // Demo 2: Grover 搜索算法
    demonstrate_grover_search()?;

    // Demo 3: 变分量子本征求解器 (VQE)
    demonstrate_vqe()?;

    // Demo 4: 量子近似优化算法 (QAOA)
    demonstrate_qaoa()?;

    // Demo 5: 量子相位估计
    demonstrate_phase_estimation()?;

    // Demo 6: 算法性能对比
    demonstrate_algorithm_comparison()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_qft() -> Result<()> {
    println!("\n1. 量子傅里叶变换 (QFT)");
    println!("{}", "-".repeat(30));

    let num_qubits = 3;
    let qft = QFT::new(num_qubits);

    // 构建 QFT 电路
    let qft_circuit = qft.build_circuit()?;
    println!("QFT 电路信息:");
    println!("• 量子比特数: {}", qft_circuit.num_qubits());
    println!("• 门数量: {}", qft_circuit.size());

    // 构建逆 QFT 电路
    let iqft_circuit = qft.build_inverse_circuit()?;
    println!("• 逆QFT门数量: {}", iqft_circuit.size());

    // 创建测试电路: 准备态 -> QFT -> 逆QFT
    let mut test_circuit = QuantumCircuit::new(num_qubits, num_qubits);

    // 准备一个简单的初始态 |001⟩
    test_circuit.x(0)?;

    // 应用 QFT (简化版本，直接添加门)
    for i in 0..num_qubits {
        test_circuit.h(i)?;
    }

    // 测量
    for i in 0..num_qubits {
        test_circuit.measure(i, i)?;
    }

    println!("\n执行 QFT 测试:");
    let mut simulator = StateVectorSimulator::new(num_qubits, num_qubits);
    let results = simulator.execute_circuit(&test_circuit)?;

    println!("测量结果分布:");
    for (bitstring, count) in &results {
        println!("  |{}⟩: {} 次", bitstring, count);
    }

    println!("✅ QFT 演示完成");
    Ok(())
}

fn demonstrate_grover_search() -> Result<()> {
    println!("\n2. Grover 搜索算法");
    println!("{}", "-".repeat(30));

    let num_qubits = 3;
    let marked_items = vec![3, 5]; // 搜索 |011⟩ 和 |101⟩

    let grover = GroverSearch::new(num_qubits, marked_items.clone())?;

    println!("Grover 搜索配置:");
    println!("• 搜索空间: {} 个状态", 1 << num_qubits);
    println!("• 标记项目: {:?}", marked_items);
    println!("• 最优迭代次数: {}", grover.optimal_iterations());

    // 构建 Grover 电路
    let grover_circuit = grover.build_circuit()?;
    println!("• 电路门数量: {}", grover_circuit.size());

    // 运行算法
    println!("\n执行 Grover 搜索:");
    let shots = 100;
    let results = AlgorithmRunner::run_algorithm(grover_circuit, shots)?;

    println!("搜索结果 ({} 次测量):", shots);
    let mut sorted_results: Vec<_> = results.iter().collect();
    sorted_results.sort_by(|a, b| b.1.cmp(a.1)); // 按计数降序排列

    for (bitstring, count) in sorted_results.iter().take(5) {
        let state_index = usize::from_str_radix(bitstring, 2).unwrap_or(0);
        let is_marked = marked_items.contains(&state_index);
        let marker = if is_marked { "🎯" } else { "  " };
        println!(
            "  {} |{}⟩ ({}): {} 次 ({:.1}%)",
            marker,
            bitstring,
            state_index,
            count,
            **count as f64 / shots as f64 * 100.0
        );
    }

    // 计算成功概率
    let success_prob =
        AlgorithmRunner::calculate_grover_success_probability(&results, &marked_items, num_qubits);
    println!("\n成功概率: {:.1}%", success_prob * 100.0);

    println!("✅ Grover 搜索演示完成");
    Ok(())
}

fn demonstrate_vqe() -> Result<()> {
    println!("\n3. 变分量子本征求解器 (VQE)");
    println!("{}", "-".repeat(30));

    let num_qubits = 2;
    let num_layers = 2;
    let vqe = VQEAnsatz::new(num_qubits, num_layers);

    println!("VQE 配置:");
    println!("• 量子比特数: {}", num_qubits);
    println!("• 层数: {}", num_layers);
    println!("• 参数数量: {}", vqe.num_parameters());

    // 生成随机参数
    let parameters: Vec<f64> = (0..vqe.num_parameters())
        .map(|i| (i as f64 * 0.1) % (2.0 * PI))
        .collect();

    println!("\n参数设置:");
    for (i, &param) in parameters.iter().enumerate() {
        println!("  θ[{}] = {:.3}", i, param);
    }

    // 构建 VQE ansatz 电路
    let vqe_circuit = vqe.build_hardware_efficient_ansatz(&parameters)?;
    println!("\nVQE 电路信息:");
    println!("• 门数量: {}", vqe_circuit.size());

    // 添加测量
    let mut measured_circuit = QuantumCircuit::new(num_qubits, num_qubits);

    // 复制 VQE 电路的门 (简化实现)
    for i in 0..num_qubits {
        measured_circuit.ry(i, myquat::Parameter::Float(parameters[0]))?;
        measured_circuit.rz(i, myquat::Parameter::Float(parameters[1]))?;
    }
    measured_circuit.cx(0, 1)?;

    // 测量
    for i in 0..num_qubits {
        measured_circuit.measure(i, i)?;
    }

    // 运行电路
    println!("\n执行 VQE ansatz:");
    let shots = 100;
    let results = AlgorithmRunner::run_algorithm(measured_circuit, shots)?;

    println!("状态分布:");
    for (bitstring, count) in &results {
        println!(
            "  |{}⟩: {} 次 ({:.1}%)",
            bitstring,
            count,
            *count as f64 / shots as f64 * 100.0
        );
    }

    println!("✅ VQE 演示完成");
    Ok(())
}

fn demonstrate_qaoa() -> Result<()> {
    println!("\n4. 量子近似优化算法 (QAOA)");
    println!("{}", "-".repeat(30));

    let num_qubits = 4;
    let num_layers = 2;
    let qaoa = QAOA::new(num_qubits, num_layers);

    // 定义 MaxCut 问题的图 (正方形图)
    let graph_edges = vec![(0, 1), (1, 2), (2, 3), (3, 0)];

    println!("QAOA MaxCut 问题:");
    println!("• 节点数: {}", num_qubits);
    println!("• 边数: {}", graph_edges.len());
    println!("• 层数: {}", num_layers);
    println!("• 图结构: {:?}", graph_edges);

    // QAOA 参数 (γ, β) 对
    let parameters = vec![0.5, 0.3, 0.7, 0.2]; // 2层 × 2参数

    println!("\nQAOA 参数:");
    for layer in 0..num_layers {
        let gamma = parameters[2 * layer];
        let beta = parameters[2 * layer + 1];
        println!("  层 {}: γ = {:.3}, β = {:.3}", layer + 1, gamma, beta);
    }

    // 构建 QAOA 电路
    let qaoa_circuit = qaoa.build_maxcut_circuit(&graph_edges, &parameters)?;
    println!("\nQAOA 电路信息:");
    println!("• 门数量: {}", qaoa_circuit.size());

    // 运行算法
    println!("\n执行 QAOA:");
    let shots = 100;
    let results = AlgorithmRunner::run_algorithm(qaoa_circuit, shots)?;

    println!("优化结果:");
    let mut sorted_results: Vec<_> = results.iter().collect();
    sorted_results.sort_by(|a, b| b.1.cmp(a.1));

    for (bitstring, count) in sorted_results.iter().take(5) {
        let cut_value = calculate_maxcut_value(bitstring, &graph_edges);
        println!(
            "  |{}⟩: {} 次 ({:.1}%) - Cut值: {}",
            bitstring,
            count,
            **count as f64 / shots as f64 * 100.0,
            cut_value
        );
    }

    println!("✅ QAOA 演示完成");
    Ok(())
}

fn demonstrate_phase_estimation() -> Result<()> {
    println!("\n5. 量子相位估计");
    println!("{}", "-".repeat(30));

    let num_counting = 3;
    let num_eigenstate = 1;
    let pe = PhaseEstimation::new(num_counting, num_eigenstate);

    println!("相位估计配置:");
    println!("• 计数量子比特: {}", num_counting);
    println!("• 本征态量子比特: {}", num_eigenstate);
    println!("• 相位精度: {:.3}", 1.0 / (1 << num_counting) as f64);

    // 构建相位估计电路
    let pe_circuit = pe.build_circuit(None)?;
    println!("\n电路信息:");
    println!("• 总量子比特数: {}", pe_circuit.num_qubits());
    println!("• 门数量: {}", pe_circuit.size());

    // 运行算法
    println!("\n执行相位估计:");
    let shots = 100;
    let results = AlgorithmRunner::run_algorithm(pe_circuit, shots)?;

    println!("相位估计结果:");
    for (bitstring, count) in &results {
        let phase_estimate = estimate_phase_from_bitstring(bitstring, num_counting);
        println!(
            "  |{}⟩: {} 次 - 估计相位: {:.3}π",
            bitstring, count, phase_estimate
        );
    }

    println!("✅ 相位估计演示完成");
    Ok(())
}

fn demonstrate_algorithm_comparison() -> Result<()> {
    println!("\n6. 算法性能对比");
    println!("{}", "-".repeat(30));

    let num_qubits = 3;

    // 创建不同算法的电路
    let qft = QFT::new(num_qubits);
    let qft_circuit = qft.build_circuit()?;

    let grover = GroverSearch::new(num_qubits, vec![3])?;
    let grover_circuit = grover.build_circuit()?;

    let vqe = VQEAnsatz::new(num_qubits, 1);
    let params: Vec<f64> = (0..vqe.num_parameters()).map(|i| i as f64 * 0.1).collect();
    let vqe_circuit = vqe.build_hardware_efficient_ansatz(&params)?;

    println!("算法复杂度对比 ({} 量子比特):", num_qubits);
    println!(
        "{:<15} | {:<8} | {:<12} | {}",
        "算法", "门数", "参数数", "描述"
    );
    println!("{}", "-".repeat(50));

    println!(
        "{:<15} | {:<8} | {:<12} | {}",
        "QFT",
        qft_circuit.size(),
        0,
        "量子傅里叶变换"
    );

    println!(
        "{:<15} | {:<8} | {:<12} | {}",
        "Grover",
        grover_circuit.size(),
        0,
        format!("{} 次迭代", grover.optimal_iterations())
    );

    println!(
        "{:<15} | {:<8} | {:<12} | {}",
        "VQE",
        vqe_circuit.size(),
        vqe.num_parameters(),
        "1层硬件高效ansatz"
    );

    // 理论复杂度分析
    println!("\n理论复杂度分析:");
    println!("• QFT: O(n²) 门，n = 量子比特数");
    println!("• Grover: O(√N) 迭代，N = 搜索空间大小");
    println!("• VQE: O(L×n) 门，L = 层数，n = 量子比特数");
    println!("• QAOA: O(p×m) 门，p = 层数，m = 边数");

    // 应用场景
    println!("\n应用场景:");
    println!("• QFT: 周期查找、Shor算法、量子相位估计");
    println!("• Grover: 数据库搜索、SAT问题、密码分析");
    println!("• VQE: 分子模拟、材料科学、量子化学");
    println!("• QAOA: 组合优化、图着色、旅行商问题");

    println!("✅ 算法对比演示完成");
    Ok(())
}

// 辅助函数
fn calculate_maxcut_value(bitstring: &str, edges: &[(usize, usize)]) -> usize {
    let bits: Vec<char> = bitstring.chars().collect();
    let mut cut_value = 0;

    for &(i, j) in edges {
        if i < bits.len() && j < bits.len() {
            if bits[i] != bits[j] {
                cut_value += 1;
            }
        }
    }

    cut_value
}

fn estimate_phase_from_bitstring(bitstring: &str, num_bits: usize) -> f64 {
    if let Ok(value) = usize::from_str_radix(bitstring, 2) {
        2.0 * (value as f64) / (1 << num_bits) as f64
    } else {
        0.0
    }
}
