//! Quantum State Collapse Simulation Demo
//!
//! This example demonstrates the new quantum state collapse simulation capabilities
//! of the MyQuat library, including measurement operations and state collapse.

use myquat::{
    quantum_info::{QuantumInfo, QuantumState},
    QuantumCircuit, Result, StateVectorSimulator,
};

fn main() -> Result<()> {
    println!("MyQuat 量子态坍缩模拟演示");
    println!("{}", "=".repeat(50));

    // Demo 1: Basic measurement and state collapse
    demonstrate_basic_measurement()?;

    // Demo 2: Bell state entanglement and measurement
    demonstrate_bell_state_measurement()?;

    // Demo 3: Circuit execution with measurements
    demonstrate_circuit_execution()?;

    // Demo 4: Multiple shots simulation
    demonstrate_multiple_shots()?;

    // Demo 5: Intermediate measurements
    demonstrate_intermediate_measurements()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_basic_measurement() -> Result<()> {
    println!("\n1. 基础测量和态坍缩演示");
    println!("{}", "-".repeat(30));

    // Create |+⟩ state (equal superposition)
    let plus_state = QuantumState::plus_state(1);
    let mut simulator = StateVectorSimulator::new_with_state(plus_state, 1);
    simulator.set_seed(42); // For reproducible results

    println!("初始状态: |+⟩ = (|0⟩ + |1⟩)/√2");
    println!("测量前概率: P(0) = 0.5, P(1) = 0.5");

    // Perform measurement
    let result = simulator.measure_qubit(0, 0)?;

    println!(
        "测量结果: {} (概率: {:.3})",
        result.bit_value, result.probability
    );
    println!("测量后状态已坍缩到 |{}⟩", result.bit_value);

    // Verify state collapse
    let (prob_0, prob_1) = simulator.calculate_measurement_probabilities(0)?;
    println!("坍缩后概率: P(0) = {:.3}, P(1) = {:.3}", prob_0, prob_1);

    Ok(())
}

fn demonstrate_bell_state_measurement() -> Result<()> {
    println!("\n2. Bell 态纠缠和测量演示");
    println!("{}", "-".repeat(30));

    // Create Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2
    let bell_state = QuantumInfo::bell_state(0)?;
    let mut simulator = StateVectorSimulator::new_with_state(bell_state, 2);
    simulator.set_seed(123);

    println!("初始状态: |Φ+⟩ = (|00⟩ + |11⟩)/√2");
    println!("这是一个最大纠缠态");

    // Measure first qubit
    let result1 = simulator.measure_qubit(0, 0)?;
    println!(
        "测量第一个量子比特: {} (概率: {:.3})",
        result1.bit_value, result1.probability
    );

    // Measure second qubit
    let result2 = simulator.measure_qubit(1, 1)?;
    println!(
        "测量第二个量子比特: {} (概率: {:.3})",
        result2.bit_value, result2.probability
    );

    println!(
        "由于纠缠，两个测量结果必须相同: {} = {}",
        result1.bit_value, result2.bit_value
    );

    Ok(())
}

fn demonstrate_circuit_execution() -> Result<()> {
    println!("\n3. 量子电路执行演示");
    println!("{}", "-".repeat(30));

    // Create a simple quantum circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?; // Create superposition on qubit 0
    circuit.cx(0, 1)?; // Create entanglement
    circuit.measure_all()?; // Measure both qubits

    println!("电路结构:");
    println!("  H(0)  - 在量子比特0上应用Hadamard门");
    println!("  CX(0,1) - 应用CNOT门创建纠缠");
    println!("  测量所有量子比特");

    // Execute circuit
    let mut simulator = StateVectorSimulator::new(2, 2);
    simulator.set_seed(456);

    let counts = simulator.execute_circuit(&circuit)?;

    println!("执行结果:");
    for (bit_string, count) in &counts {
        println!("  {}: {} 次", bit_string, count);
    }

    println!("由于纠缠，只能得到 '00' 或 '11' 结果");

    Ok(())
}

fn demonstrate_multiple_shots() -> Result<()> {
    println!("\n4. 多次运行统计演示");
    println!("{}", "-".repeat(30));

    // Create a simple circuit with superposition
    let mut circuit = QuantumCircuit::new(1, 1);
    circuit.h(0)?; // Create equal superposition
    circuit.measure(0, 0)?;

    println!("电路: H(0) + 测量");
    println!("理论上应该得到 50% 的 '0' 和 50% 的 '1'");

    // Run multiple times
    let mut simulator = StateVectorSimulator::new(1, 1);
    let shots = 1000;
    let counts = simulator.run_circuit_multiple(&circuit, shots)?;

    println!("运行 {} 次的统计结果:", shots);
    let mut total = 0;
    for (bit_string, count) in &counts {
        let percentage = (*count as f64 / shots as f64) * 100.0;
        println!("  '{}': {} 次 ({:.1}%)", bit_string, count, percentage);
        total += count;
    }
    println!("总计: {} 次", total);

    Ok(())
}

fn demonstrate_intermediate_measurements() -> Result<()> {
    println!("\n5. 中间测量演示");
    println!("{}", "-".repeat(30));

    // Create a circuit with intermediate measurement
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?; // Create superposition
    circuit.measure(0, 0)?; // Intermediate measurement
    circuit.cx(0, 1)?; // Conditional operation based on measurement
    circuit.measure(1, 1)?; // Final measurement

    println!("电路结构:");
    println!("  H(0) - 创建叠加态");
    println!("  测量量子比特0 -> 经典比特0");
    println!("  CX(0,1) - 基于测量结果的条件操作");
    println!("  测量量子比特1 -> 经典比特1");

    // Execute multiple times to see statistics
    let mut simulator = StateVectorSimulator::new(2, 2);
    let shots = 100;
    let counts = simulator.run_circuit_multiple(&circuit, shots)?;

    println!("运行 {} 次的结果:", shots);
    for (bit_string, count) in &counts {
        let percentage = (*count as f64 / shots as f64) * 100.0;
        println!("  '{}': {} 次 ({:.1}%)", bit_string, count, percentage);
    }

    println!("注意: 由于中间测量，第二个量子比特的状态依赖于第一个的测量结果");

    Ok(())
}
