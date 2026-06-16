//! Easy API Demo
//!
//! This example demonstrates the easy-to-use API that makes MyQuat
//! accessible to beginners and simplifies common quantum computing tasks.

use myquat::{EasyAlgorithms, EasyAnalysis, EasyCircuit, EasySimulator, Result};
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("MyQuat 易用API演示");
    println!("{}", "=".repeat(50));

    // Demo 1: 简单电路构建
    demonstrate_easy_circuit_building()?;

    // Demo 2: 流式API使用
    demonstrate_fluent_api()?;

    // Demo 3: 预定义算法
    demonstrate_predefined_algorithms()?;

    // Demo 4: 简化的模拟器
    demonstrate_easy_simulator()?;

    // Demo 5: 分析和可视化工具
    demonstrate_analysis_tools()?;

    // Demo 6: 初学者友好的示例
    demonstrate_beginner_examples()?;

    println!("\n演示完成！");
    Ok(())
}

fn demonstrate_easy_circuit_building() -> Result<()> {
    println!("\n1. 简单电路构建");
    println!("{}", "-".repeat(30));

    // 传统方式 vs 易用API对比
    println!("易用API的优势:");
    println!("• 流式接口 (Fluent API)");
    println!("• 自动错误处理");
    println!("• 简化的参数传递");
    println!("• 内置常用操作");

    // 创建一个简单的2量子比特电路
    let mut circuit = EasyCircuit::new(2);

    println!("\n创建2量子比特电路:");
    println!("let mut circuit = EasyCircuit::new(2);");

    // 应用门操作
    circuit
        .h(0)? // Hadamard门
        .cnot(0, 1)? // CNOT门
        .measure_all()?; // 测量所有量子比特

    println!("circuit.h(0)?.cnot(0, 1)?.measure_all()?;");

    EasyAnalysis::print_circuit_info(&circuit);

    println!("✅ 简单电路构建演示完成");
    Ok(())
}

fn demonstrate_fluent_api() -> Result<()> {
    println!("\n2. 流式API使用");
    println!("{}", "-".repeat(30));

    println!("流式API特点:");
    println!("• 链式调用");
    println!("• 可读性强");
    println!("• 错误传播");

    // 创建一个更复杂的电路
    let mut circuit = EasyCircuit::new(3);

    println!("\n构建3量子比特GHZ态:");
    circuit
        .h(0)? // 第一个量子比特叠加
        .cnot(0, 1)? // 纠缠第二个量子比特
        .cnot(0, 2)? // 纠缠第三个量子比特
        .measure_all()?; // 测量所有量子比特

    println!("circuit.h(0)?.cnot(0, 1)?.cnot(0, 2)?.measure_all()?;");

    EasyAnalysis::print_circuit_info(&circuit);

    // 演示旋转门的简化使用
    println!("\n旋转门简化使用:");
    let mut rotation_circuit = EasyCircuit::new(1);
    rotation_circuit
        .rx(0, std::f64::consts::PI / 4.0)? // X轴旋转π/4
        .ry(0, std::f64::consts::PI / 3.0)? // Y轴旋转π/3
        .rz(0, std::f64::consts::PI / 6.0)? // Z轴旋转π/6
        .measure_all()?;

    println!("rotation_circuit.rx(0, π/4)?.ry(0, π/3)?.rz(0, π/6)?;");

    println!("✅ 流式API演示完成");
    Ok(())
}

fn demonstrate_predefined_algorithms() -> Result<()> {
    println!("\n3. 预定义算法");
    println!("{}", "-".repeat(30));

    println!("预定义算法优势:");
    println!("• 一行代码创建复杂电路");
    println!("• 经过验证的实现");
    println!("• 参数自动优化");

    // Bell态
    println!("\n创建Bell态:");
    let bell_circuit = EasyAlgorithms::bell_state()?;
    println!("let bell_circuit = EasyAlgorithms::bell_state()?;");
    EasyAnalysis::print_circuit_info(&bell_circuit);

    // GHZ态
    println!("\n创建4量子比特GHZ态:");
    let ghz_circuit = EasyAlgorithms::ghz_state(4)?;
    println!("let ghz_circuit = EasyAlgorithms::ghz_state(4)?;");
    EasyAnalysis::print_circuit_info(&ghz_circuit);

    // 量子随机数生成器
    println!("\n创建量子随机数生成器:");
    let random_circuit = EasyAlgorithms::random_bits(5)?;
    println!("let random_circuit = EasyAlgorithms::random_bits(5)?;");
    EasyAnalysis::print_circuit_info(&random_circuit);

    // Grover搜索
    println!("\n创建Grover搜索电路:");
    let grover_circuit = EasyAlgorithms::grover_search(3, 5)?; // 搜索状态|101⟩
    println!("let grover_circuit = EasyAlgorithms::grover_search(3, 5)?;");
    EasyAnalysis::print_circuit_info(&grover_circuit);

    // QFT
    println!("\n创建量子傅里叶变换:");
    let qft_circuit = EasyAlgorithms::qft(3)?;
    println!("let qft_circuit = EasyAlgorithms::qft(3)?;");
    EasyAnalysis::print_circuit_info(&qft_circuit);

    println!("✅ 预定义算法演示完成");
    Ok(())
}

fn demonstrate_easy_simulator() -> Result<()> {
    println!("\n4. 简化的模拟器");
    println!("{}", "-".repeat(30));

    println!("简化模拟器特点:");
    println!("• 自动配置");
    println!("• 统计分析");
    println!("• 结果解释");

    // 创建Bell态电路
    let bell_circuit = EasyAlgorithms::bell_state()?;
    let mut simulator = EasySimulator::new(2);

    println!("\n执行Bell态电路:");
    let shots = 1000;
    let results = simulator.run_shots(bell_circuit.circuit(), shots)?;

    EasyAnalysis::print_results(&results, shots);

    // 计算成功概率
    let bell_states = ["00", "11"];
    let success_prob =
        simulator.success_probability(bell_circuit.circuit(), &bell_states, shots)?;

    println!("\nBell态成功概率: {:.1}%", success_prob * 100.0);

    // 最可能的结果
    let most_probable = simulator.most_probable(bell_circuit.circuit(), shots)?;
    println!("最可能的测量结果: |{}⟩", most_probable);

    // 量子随机数生成演示
    println!("\n量子随机数生成:");
    let random_circuit = EasyAlgorithms::random_bits(3)?;
    let random_results = simulator.run_shots(random_circuit.circuit(), 100)?;

    println!("随机比特分布:");
    EasyAnalysis::print_results(&random_results, 100);

    println!("✅ 简化模拟器演示完成");
    Ok(())
}

fn demonstrate_analysis_tools() -> Result<()> {
    println!("\n5. 分析和可视化工具");
    println!("{}", "-".repeat(30));

    println!("分析工具功能:");
    println!("• 电路信息统计");
    println!("• 结果可视化");
    println!("• 性能建议");
    println!("• 保真度计算");

    // 创建一个复杂电路进行分析
    let mut complex_circuit = EasyCircuit::new(4);
    complex_circuit
        .h(0)?
        .h(1)?
        .h(2)?
        .h(3)?
        .cnot(0, 1)?
        .cnot(1, 2)?
        .cnot(2, 3)?
        .rx(0, 0.5)?
        .ry(1, 0.3)?
        .rz(2, 0.7)?
        .measure_all()?;

    println!("\n复杂电路分析:");
    EasyAnalysis::print_circuit_info(&complex_circuit);
    EasyAnalysis::suggest_optimizations(&complex_circuit);

    // 保真度计算演示
    println!("\n保真度分析:");
    let mut simulator = EasySimulator::new(2);
    let bell_circuit = EasyAlgorithms::bell_state()?;
    let results = simulator.run_shots(bell_circuit.circuit(), 1000)?;

    // 理论Bell态分布
    let mut expected = HashMap::new();
    expected.insert("00".to_string(), 0.5);
    expected.insert("11".to_string(), 0.5);

    EasyAnalysis::print_fidelity(&expected, &results, 1000);

    println!("✅ 分析工具演示完成");
    Ok(())
}

fn demonstrate_beginner_examples() -> Result<()> {
    println!("\n6. 初学者友好示例");
    println!("{}", "-".repeat(30));

    println!("初学者教程:");

    // 示例1: 第一个量子程序
    println!("\n📚 示例1: 我的第一个量子程序");
    println!("目标: 创建叠加态 (|0⟩ + |1⟩)/√2");

    let mut first_program = EasyCircuit::new(1);
    first_program.h(0)?.measure_all()?;

    println!("代码:");
    println!("  let mut circuit = EasyCircuit::new(1);");
    println!("  circuit.h(0)?.measure_all()?;");

    let mut simulator = EasySimulator::new(1);
    let results = simulator.run_shots(first_program.circuit(), 100)?;
    EasyAnalysis::print_results(&results, 100);

    // 示例2: 量子纠缠
    println!("\n📚 示例2: 量子纠缠");
    println!("目标: 创建纠缠态 (|00⟩ + |11⟩)/√2");

    let entangled_circuit = EasyAlgorithms::bell_state()?;
    println!("代码:");
    println!("  let circuit = EasyAlgorithms::bell_state()?;");

    let results = simulator.run_shots(entangled_circuit.circuit(), 100)?;
    EasyAnalysis::print_results(&results, 100);

    // 示例3: 量子搜索
    println!("\n📚 示例3: 量子搜索");
    println!("目标: 在8个状态中搜索|101⟩");

    let search_circuit = EasyAlgorithms::grover_search(3, 5)?; // 5 = |101⟩
    println!("代码:");
    println!("  let circuit = EasyAlgorithms::grover_search(3, 5)?;");

    let mut search_simulator = EasySimulator::new(3);
    let search_results = search_simulator.run_shots(search_circuit.circuit(), 100)?;
    EasyAnalysis::print_results(&search_results, 100);

    // 学习建议
    println!("\n🎓 学习建议:");
    println!("• 从单量子比特操作开始");
    println!("• 理解叠加和纠缠概念");
    println!("• 使用预定义算法学习经典量子算法");
    println!("• 通过分析工具理解电路性能");
    println!("• 实验不同参数和电路结构");

    // 常见错误提示
    println!("\n⚠️  常见错误提示:");
    println!("• 量子比特索引从0开始");
    println!("• 测量会破坏量子态");
    println!("• 角度参数使用弧度制");
    println!("• 大电路建议使用性能优化");

    println!("✅ 初学者示例演示完成");
    Ok(())
}

/// 演示错误处理的简化
fn _demonstrate_error_handling() -> Result<()> {
    println!("\n7. 简化的错误处理");
    println!("{}", "-".repeat(30));

    println!("错误处理特点:");
    println!("• 自动错误传播");
    println!("• 友好的错误信息");
    println!("• 防御性编程");

    // 演示自动错误处理
    let mut circuit = EasyCircuit::new(2);

    // 这些操作会自动处理错误
    match circuit
        .h(0)
        .and_then(|c| c.cnot(0, 1))
        .and_then(|c| c.measure_all())
    {
        Ok(_) => println!("✅ 电路构建成功"),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 演示边界检查
    println!("\n边界检查演示:");
    let result = circuit.h(10); // 超出范围的量子比特
    match result {
        Ok(_) => println!("操作成功"),
        Err(e) => println!("自动捕获错误: {}", e),
    }

    Ok(())
}
