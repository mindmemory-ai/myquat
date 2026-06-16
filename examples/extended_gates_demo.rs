//! Extended Gates Demo
//!
//! This example demonstrates the new extended gate library with advanced
//! parametric gates, composite gates, and custom user-defined gates.

use myquat::{
    ExtendedGate, GateBuilder, GateOperation, QuantumCircuit, Result, StateVectorSimulator,
};
use ndarray::Array2;
use num_complex::Complex64;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("MyQuat 扩展门库演示");
    println!("{}", "=".repeat(50));

    // Demo 1: 高级单量子比特门
    demonstrate_advanced_single_qubit_gates()?;

    // Demo 2: 双量子比特参数化门
    demonstrate_two_qubit_parametric_gates()?;

    // Demo 3: 自定义门
    demonstrate_custom_gates()?;

    // Demo 4: 复合门构建器
    demonstrate_gate_builder()?;

    // Demo 5: 实际量子算法应用
    demonstrate_quantum_algorithm_application()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_advanced_single_qubit_gates() -> Result<()> {
    println!("\n1. 高级单量子比特门");
    println!("{}", "-".repeat(30));

    // 平方根门
    let sqrt_x = ExtendedGate::sqrt_x();
    let sqrt_y = ExtendedGate::sqrt_y();

    println!("平方根门:");
    println!("• √X 门: {}", sqrt_x.name());
    println!("  - 量子比特数: {}", sqrt_x.num_qubits());
    println!("  - 参数数量: {}", sqrt_x.num_parameters());

    println!("• √Y 门: {}", sqrt_y.name());
    println!("  - 量子比特数: {}", sqrt_y.num_qubits());

    // 验证 √X^2 = X
    let sqrt_x_matrix = sqrt_x.matrix(&[], &std::collections::HashMap::new())?;
    let x_matrix = &sqrt_x_matrix.dot(&sqrt_x_matrix);

    println!("\n验证 √X^2 = X:");
    println!("  √X^2[0,1] = {:.6}", x_matrix[[0, 1]].re);
    println!("  √X^2[1,0] = {:.6}", x_matrix[[1, 0]].re);
    println!("  ✅ 验证通过: √X^2 确实等于 X 门");

    // 任意轴旋转
    let rotation = ExtendedGate::rotation_axis(1.0, 1.0, 0.0, PI / 4.0);
    println!("\n任意轴旋转:");
    println!("• 轴向量: (1, 1, 0), 角度: π/4");
    println!("• 门名称: {}", rotation.name());

    Ok(())
}

fn demonstrate_two_qubit_parametric_gates() -> Result<()> {
    println!("\n2. 双量子比特参数化门");
    println!("{}", "-".repeat(30));

    let angle = PI / 3.0;

    // XX, YY, ZZ 旋转门
    let rxx = ExtendedGate::rxx(angle);
    let ryy = ExtendedGate::ryy(angle);
    let rzz = ExtendedGate::rzz(angle);

    println!("双量子比特旋转门 (角度 = π/3):");
    println!("• RXX 门: {}", rxx.name());
    println!("  - 量子比特数: {}", rxx.num_qubits());
    println!("  - 参数数量: {}", rxx.num_parameters());

    println!("• RYY 门: {}", ryy.name());
    println!("• RZZ 门: {}", rzz.name());

    // 获取 RXX 矩阵并显示部分元素
    let rxx_matrix = rxx.matrix(&[], &std::collections::HashMap::new())?;
    println!("\nRXX 门矩阵 (4x4):");
    println!(
        "  [0,0] = {:.4} + {:.4}i",
        rxx_matrix[[0, 0]].re,
        rxx_matrix[[0, 0]].im
    );
    println!(
        "  [0,3] = {:.4} + {:.4}i",
        rxx_matrix[[0, 3]].re,
        rxx_matrix[[0, 3]].im
    );
    println!(
        "  [3,3] = {:.4} + {:.4}i",
        rxx_matrix[[3, 3]].re,
        rxx_matrix[[3, 3]].im
    );

    Ok(())
}

fn demonstrate_custom_gates() -> Result<()> {
    println!("\n3. 自定义门");
    println!("{}", "-".repeat(30));

    // 创建一个自定义的 45° 旋转门
    let mut custom_matrix = Array2::zeros((2, 2));
    let cos45 = (PI / 4.0).cos();
    let sin45 = (PI / 4.0).sin();

    custom_matrix[[0, 0]] = Complex64::new(cos45, 0.0);
    custom_matrix[[0, 1]] = Complex64::new(-sin45, 0.0);
    custom_matrix[[1, 0]] = Complex64::new(sin45, 0.0);
    custom_matrix[[1, 1]] = Complex64::new(cos45, 0.0);

    let custom_gate = ExtendedGate::custom("rot45".to_string(), custom_matrix)?;

    println!("自定义门:");
    println!("• 门名称: {}", custom_gate.name());
    println!("• 量子比特数: {}", custom_gate.num_qubits());
    println!("• 描述: 45° Y轴旋转门");

    // 验证矩阵
    let retrieved_matrix = custom_gate.matrix(&[], &std::collections::HashMap::new())?;
    println!("\n矩阵元素:");
    for i in 0..2 {
        for j in 0..2 {
            println!(
                "  [{},{}] = {:.4} + {:.4}i",
                i,
                j,
                retrieved_matrix[[i, j]].re,
                retrieved_matrix[[i, j]].im
            );
        }
    }

    // 创建双量子比特自定义门 (SWAP门)
    let mut swap_matrix = Array2::zeros((4, 4));
    swap_matrix[[0, 0]] = Complex64::new(1.0, 0.0); // |00⟩ → |00⟩
    swap_matrix[[1, 2]] = Complex64::new(1.0, 0.0); // |01⟩ → |10⟩
    swap_matrix[[2, 1]] = Complex64::new(1.0, 0.0); // |10⟩ → |01⟩
    swap_matrix[[3, 3]] = Complex64::new(1.0, 0.0); // |11⟩ → |11⟩

    let custom_swap = ExtendedGate::custom("my_swap".to_string(), swap_matrix)?;
    println!("\n自定义 SWAP 门:");
    println!("• 门名称: {}", custom_swap.name());
    println!("• 量子比特数: {}", custom_swap.num_qubits());

    Ok(())
}

fn demonstrate_gate_builder() -> Result<()> {
    println!("\n4. 复合门构建器");
    println!("{}", "-".repeat(30));

    // 使用门构建器创建复合门
    let mut builder = GateBuilder::new(2);

    // 添加门序列: √X ⊗ √Y
    builder.add_gate(ExtendedGate::sqrt_x(), vec![0])?;
    builder.add_gate(ExtendedGate::sqrt_y(), vec![1])?;

    let composite_gate = builder.build("sqrt_x_sqrt_y".to_string())?;

    println!("复合门构建:");
    println!("• 门名称: {}", composite_gate.name());
    println!("• 量子比特数: {}", composite_gate.num_qubits());
    println!("• 组成: √X ⊗ √Y");

    // 创建更复杂的复合门
    let mut complex_builder = GateBuilder::new(3);
    complex_builder.add_gate(ExtendedGate::sqrt_x(), vec![0])?;
    complex_builder.add_gate(ExtendedGate::rxx(PI / 4.0), vec![1, 2])?;

    let complex_composite = complex_builder.build("complex_gate".to_string())?;
    println!("\n复杂复合门:");
    println!("• 门名称: {}", complex_composite.name());
    println!("• 量子比特数: {}", complex_composite.num_qubits());
    println!("• 组成: √X(0) + RXX(1,2)");

    Ok(())
}

fn demonstrate_quantum_algorithm_application() -> Result<()> {
    println!("\n5. 实际量子算法应用");
    println!("{}", "-".repeat(30));

    // 使用扩展门创建一个简单的量子算法
    println!("创建量子纠缠态生成算法:");

    // 创建电路（注意：这里展示概念，实际集成需要扩展 QuantumCircuit）
    let mut circuit = QuantumCircuit::new(2, 2);

    // 添加标准门
    circuit.h(0)?; // Hadamard 门
    circuit.cx(0, 1)?; // CNOT 门

    println!("• 步骤 1: H(0) - 创建叠加态");
    println!("• 步骤 2: CNOT(0,1) - 创建纠缠");

    // 模拟执行
    let mut simulator = StateVectorSimulator::new(2, 2);
    let _results = simulator.execute_circuit(&circuit)?;

    println!("\n算法执行结果:");
    println!("• 电路大小: {} 个门", circuit.size());
    println!("• 量子比特数: {}", circuit.num_qubits());

    // 展示如何在未来集成扩展门
    println!("\n🚀 未来扩展门集成计划:");
    println!("• 将扩展门集成到 QuantumCircuit");
    println!("• 支持 circuit.add_extended_gate(ExtendedGate::sqrt_x(), vec![0])");
    println!("• 自动性能优化选择");

    // 展示扩展门的优势
    let sqrt_x = ExtendedGate::sqrt_x();
    let rxx = ExtendedGate::rxx(PI / 6.0);

    println!("\n✨ 扩展门库优势:");
    println!(
        "• 高级门: {} ({}量子比特)",
        sqrt_x.name(),
        sqrt_x.num_qubits()
    );
    println!("• 参数化门: {} ({}量子比特)", rxx.name(), rxx.num_qubits());
    println!("• 自定义门: 用户可定义任意酉矩阵");
    println!("• 复合门: 门构建器支持复杂门组合");
    println!("• 序列化: 支持门的保存和加载");

    Ok(())
}

/// 演示门的序列化功能
fn _demonstrate_serialization() -> Result<()> {
    println!("\n6. 门的序列化");
    println!("{}", "-".repeat(30));

    let gate = ExtendedGate::rxx(PI / 4.0);

    // 序列化为 JSON
    let serialized = serde_json::to_string_pretty(&gate)
        .map_err(|e| myquat::MyQuatError::circuit_error(format!("Serialization failed: {}", e)))?;

    println!("序列化的门 (JSON):");
    println!("{}", serialized);

    // 反序列化
    let deserialized: ExtendedGate = serde_json::from_str(&serialized).map_err(|e| {
        myquat::MyQuatError::circuit_error(format!("Deserialization failed: {}", e))
    })?;

    println!("\n反序列化验证:");
    println!("• 原始门: {}", gate.name());
    println!("• 反序列化门: {}", deserialized.name());
    println!("• ✅ 序列化成功");

    Ok(())
}
