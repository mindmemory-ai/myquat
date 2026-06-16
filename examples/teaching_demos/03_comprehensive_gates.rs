use myquat::{Parameter, QuantumCircuit};
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MyQuat 量子门完整性演示");
    println!("{}", "=".repeat(50));

    // 演示单量子比特门
    demonstrate_single_qubit_gates()?;

    // 演示双量子比特门
    demonstrate_two_qubit_gates()?;

    // 演示三量子比特门
    demonstrate_three_qubit_gates()?;

    // 演示参数化门
    demonstrate_parametric_gates()?;

    // 演示量子算法应用
    demonstrate_quantum_algorithms()?;

    println!("\n所有量子门演示完成！");
    Ok(())
}

fn demonstrate_single_qubit_gates() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n单量子比特门演示");
    println!("{}", "-".repeat(30));

    let mut circuit = QuantumCircuit::new(1, 1);

    // Pauli 门
    println!("Pauli 门:");
    circuit.i(0)?; // 恒等门
    circuit.x(0)?; // Pauli-X (NOT)
    circuit.y(0)?; // Pauli-Y
    circuit.z(0)?; // Pauli-Z
    println!("  I, X, Y, Z 门已添加");

    // Clifford 门
    println!("Clifford 门:");
    circuit.h(0)?; // Hadamard
    circuit.s(0)?; // S 门
    circuit.sdg(0)?; // S 共轭
    circuit.t(0)?; // T 门
    circuit.tdg(0)?; // T 共轭
    println!("  H, S, S†, T, T† 门已添加");

    // 旋转门
    println!("旋转门:");
    circuit.rx(0, Parameter::new_float(PI / 4.0))?;
    circuit.ry(0, Parameter::new_float(PI / 3.0))?;
    circuit.rz(0, Parameter::new_float(PI / 6.0))?;
    circuit.p(0, Parameter::new_float(PI / 2.0))?;
    println!("  Rx, Ry, Rz, P 门已添加");

    // 通用门
    println!("通用门:");
    circuit.u1(0, Parameter::new_float(PI / 4.0))?;
    circuit.u2(0, Parameter::new_float(0.0), Parameter::new_float(PI))?;
    circuit.u3(
        0,
        Parameter::new_float(PI / 2.0),
        Parameter::new_float(0.0),
        Parameter::new_float(PI),
    )?;
    circuit.u(
        0,
        Parameter::new_float(PI / 3.0),
        Parameter::new_float(PI / 6.0),
        Parameter::new_float(PI / 4.0),
    )?;
    println!("  U1, U2, U3, U 门已添加");

    println!("单量子比特电路统计:");
    println!("  - 量子比特数: {}", circuit.num_qubits());
    println!("  - 门数量: {}", circuit.size());
    println!("  - 电路深度: {}", circuit.depth());

    Ok(())
}

fn demonstrate_two_qubit_gates() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n双量子比特门演示");
    println!("{}", "-".repeat(30));

    let mut circuit = QuantumCircuit::new(2, 2);

    // 受控门
    println!("受控门:");
    circuit.cx(0, 1)?; // CNOT
    circuit.cy(0, 1)?; // 受控 Y
    circuit.cz(0, 1)?; // 受控 Z
    circuit.ch(0, 1)?; // 受控 Hadamard
    println!("  CX, CY, CZ, CH 门已添加");

    // 受控旋转门
    println!("受控旋转门:");
    circuit.crx(0, 1, Parameter::new_float(PI / 4.0))?;
    circuit.cry(0, 1, Parameter::new_float(PI / 3.0))?;
    circuit.crz(0, 1, Parameter::new_float(PI / 6.0))?;
    circuit.cp(0, 1, Parameter::new_float(PI / 2.0))?;
    println!("  CRx, CRy, CRz, CP 门已添加");

    // 交换门
    println!("交换门:");
    circuit.swap(0, 1)?; // SWAP
    circuit.iswap(0, 1)?; // iSWAP
    println!("  SWAP, iSWAP 门已添加");

    println!("双量子比特电路统计:");
    println!("  - 量子比特数: {}", circuit.num_qubits());
    println!("  - 门数量: {}", circuit.size());
    println!("  - 电路深度: {}", circuit.depth());

    Ok(())
}

fn demonstrate_three_qubit_gates() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n三量子比特门演示");
    println!("{}", "-".repeat(30));

    let mut circuit = QuantumCircuit::new(3, 3);

    // 三量子比特门
    println!("三量子比特门:");
    circuit.ccx(0, 1, 2)?; // Toffoli (CCX)
    circuit.cswap(0, 1, 2)?; // Fredkin (CSWAP)
    println!("  CCX (Toffoli), CSWAP (Fredkin) 门已添加");

    // 多受控门
    println!("多受控门:");
    circuit.mcx(0, 1, 2)?; // 多受控 X
    circuit.mcy(0, 1, 2)?; // 多受控 Y
    circuit.mcz(0, 1, 2)?; // 多受控 Z
    println!("  MCX, MCY, MCZ 门已添加");

    // 使用别名
    println!("门别名:");
    circuit.toffoli(0, 1, 2)?; // Toffoli 别名
    circuit.fredkin(0, 1, 2)?; // Fredkin 别名
    println!("  Toffoli, Fredkin 别名已添加");

    println!("三量子比特电路统计:");
    println!("  - 量子比特数: {}", circuit.num_qubits());
    println!("  - 门数量: {}", circuit.size());
    println!("  - 电路深度: {}", circuit.depth());

    Ok(())
}

fn demonstrate_parametric_gates() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n参数化门演示");
    println!("{}", "-".repeat(30));

    let mut circuit = QuantumCircuit::new(2, 0);

    // 符号参数
    println!("符号参数:");
    circuit.rx(0, Parameter::new_symbol("theta"))?;
    circuit.ry(1, Parameter::new_symbol("phi"))?;
    circuit.crz(0, 1, Parameter::new_symbol("lambda"))?;
    println!("  使用符号参数 theta, phi, lambda");

    // 数值参数
    println!("数值参数:");
    circuit.rz(0, Parameter::new_float(2.0 * PI / 3.0))?;
    circuit.cp(0, 1, Parameter::new_float(PI / 4.0))?;
    println!("  使用数值参数");

    // 检查参数化特性
    println!("参数化特性:");
    println!("  - 是否参数化: {}", circuit.data().is_parametric());
    println!("  - 符号列表: {:?}", circuit.data().symbols());

    // 参数绑定
    println!("参数绑定:");
    let mut symbols = std::collections::HashMap::new();
    symbols.insert("theta".to_string(), PI / 4.0);
    symbols.insert("phi".to_string(), PI / 6.0);
    symbols.insert("lambda".to_string(), PI / 3.0);

    let bound_circuit = circuit.bind_parameters(&symbols)?;
    println!("  参数已绑定到具体数值");
    println!(
        "  - 绑定后是否参数化: {}",
        bound_circuit.data().is_parametric()
    );

    Ok(())
}

fn demonstrate_quantum_algorithms() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n量子算法应用演示");
    println!("{}", "-".repeat(30));

    // Bell 态制备
    println!("Bell 态制备:");
    let mut bell_circuit = QuantumCircuit::new(2, 2);
    bell_circuit.h(0)?;
    bell_circuit.cx(0, 1)?;
    bell_circuit.measure_all()?;
    println!("  |Φ+⟩ = (|00⟩ + |11⟩)/√2 制备完成");

    // 量子隐形传态
    println!("量子隐形传态:");
    let mut teleport_circuit = QuantumCircuit::new(3, 3);
    // 创建纠缠对
    teleport_circuit.h(1)?;
    teleport_circuit.cx(1, 2)?;
    // Bell 测量
    teleport_circuit.cx(0, 1)?;
    teleport_circuit.h(0)?;
    teleport_circuit.measure(0, 0)?;
    teleport_circuit.measure(1, 1)?;
    println!("  量子隐形传态电路构建完成");

    // Grover 算法 Oracle
    println!("Grover 算法 Oracle:");
    let mut grover_circuit = QuantumCircuit::new(3, 0);
    // 标记 |111⟩ 状态
    grover_circuit.ccx(0, 1, 2)?;
    grover_circuit.z(2)?;
    grover_circuit.ccx(0, 1, 2)?;
    println!("  3-qubit Grover Oracle 构建完成");

    // 量子加法器
    println!("量子加法器:");
    let mut adder_circuit = QuantumCircuit::new(4, 0);
    // 半加器逻辑
    adder_circuit.cx(0, 2)?; // sum = a ⊕ b
    adder_circuit.cx(1, 2)?;
    adder_circuit.ccx(0, 1, 3)?; // carry = a ∧ b
    println!("  量子半加器构建完成");

    // 变分量子算法层
    println!("变分量子算法:");
    let mut vqa_circuit = QuantumCircuit::new(3, 0);
    // 参数化层
    for i in 0..3 {
        vqa_circuit.ry(i, Parameter::new_symbol(&format!("theta_{}", i)))?;
    }
    // 纠缠层
    for i in 0..2 {
        vqa_circuit.cx(i, i + 1)?;
    }
    // 另一个参数化层
    for i in 0..3 {
        vqa_circuit.rz(i, Parameter::new_symbol(&format!("phi_{}", i)))?;
    }
    println!("  VQA ansatz 构建完成");
    println!("  - 参数数量: {}", vqa_circuit.data().symbols().len());

    Ok(())
}
