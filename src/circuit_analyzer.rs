// Circuit Analyzer for Feature Extraction
// Author: gA4ss
//
// 电路特征分析模块，用于提取电路特性以支持自适应优化策略选择。

use crate::circuit::QuantumCircuit;
use crate::gates::StandardGate;
use std::collections::{HashMap, HashSet};

/// 电路特征分析结果
///
/// 包含电路的各种统计特征，用于优化策略选择。
#[derive(Debug, Clone)]
pub struct CircuitProfile {
    /// CNOT密度 (CNOT数量 / 总门数)
    pub cx_density: f64,

    /// 旋转门数量
    pub rotation_count: usize,

    /// 电路深度
    pub depth: usize,

    /// 量子比特连通性度量 (0-1之间)
    pub qubit_connectivity: f64,

    /// 参数化门数量
    pub parameter_count: usize,

    /// 纠缠度 (双量子比特门比例)
    pub entanglement_degree: f64,

    /// 总门数
    pub total_gates: usize,

    /// CNOT门数量
    pub cx_count: usize,

    /// 单量子比特门数量
    pub single_qubit_count: usize,

    /// 双量子比特门数量
    pub two_qubit_count: usize,

    /// 使用的量子比特数
    pub active_qubits: usize,

    /// 平均每个量子比特的门数
    pub gates_per_qubit: f64,

    /// 是否包含测量
    pub has_measurement: bool,

    /// 电路宽度（量子比特数）
    pub circuit_width: usize,
}

impl CircuitProfile {
    /// 创建新的空电路特征
    pub fn new() -> Self {
        Self {
            cx_density: 0.0,
            rotation_count: 0,
            depth: 0,
            qubit_connectivity: 0.0,
            parameter_count: 0,
            entanglement_degree: 0.0,
            total_gates: 0,
            cx_count: 0,
            single_qubit_count: 0,
            two_qubit_count: 0,
            active_qubits: 0,
            gates_per_qubit: 0.0,
            has_measurement: false,
            circuit_width: 0,
        }
    }

    /// 判断是否为CNOT密集型电路
    pub fn is_cnot_heavy(&self) -> bool {
        self.cx_density > 0.3
    }

    /// 判断是否为旋转门密集型电路
    pub fn is_rotation_heavy(&self) -> bool {
        let rotation_ratio = if self.total_gates > 0 {
            self.rotation_count as f64 / self.total_gates as f64
        } else {
            0.0
        };
        rotation_ratio > 0.5
    }

    /// 判断是否为参数化电路
    pub fn is_parameterized(&self) -> bool {
        self.parameter_count > 0
    }

    /// 判断是否为浅电路
    pub fn is_shallow(&self) -> bool {
        self.depth < 20
    }

    /// 判断是否为深电路
    pub fn is_deep(&self) -> bool {
        self.depth > 100
    }

    /// 判断是否为宽电路（多量子比特）
    pub fn is_wide(&self) -> bool {
        self.circuit_width > 10
    }
}

impl Default for CircuitProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// 电路分析器
///
/// 分析量子电路的各种特征，用于优化策略选择。
pub struct CircuitAnalyzer {
    /// 是否计算详细统计信息
    compute_detailed_stats: bool,
}

impl CircuitAnalyzer {
    /// 创建新的电路分析器
    pub fn new() -> Self {
        Self {
            compute_detailed_stats: true,
        }
    }

    /// 创建简化版分析器（只计算基本统计）
    pub fn simplified() -> Self {
        Self {
            compute_detailed_stats: false,
        }
    }

    /// 分析电路，返回特征描述
    pub fn analyze(&self, circuit: &QuantumCircuit) -> CircuitProfile {
        let mut profile = CircuitProfile::new();

        profile.circuit_width = circuit.num_qubits();

        // 基础统计
        self.compute_basic_stats(circuit, &mut profile);

        // 计算深度
        profile.depth = self.compute_circuit_depth(circuit);

        // 计算量子比特连通性
        profile.qubit_connectivity = self.compute_qubit_connectivity(circuit);

        // 计算活跃量子比特数
        profile.active_qubits = self.count_active_qubits(circuit);

        // 计算每量子比特的平均门数
        if profile.active_qubits > 0 {
            profile.gates_per_qubit = profile.total_gates as f64 / profile.active_qubits as f64;
        }

        // 计算纠缠度
        if profile.total_gates > 0 {
            profile.entanglement_degree =
                profile.two_qubit_count as f64 / profile.total_gates as f64;
        }

        // 计算CNOT密度
        if profile.total_gates > 0 {
            profile.cx_density = profile.cx_count as f64 / profile.total_gates as f64;
        }

        profile
    }

    /// 计算基础统计信息
    fn compute_basic_stats(&self, circuit: &QuantumCircuit, profile: &mut CircuitProfile) {
        let instructions = circuit.data().instructions();

        for inst in instructions {
            if inst.is_measurement() {
                profile.has_measurement = true;
                continue;
            }

            profile.total_gates += 1;

            let num_qubits = inst.qubits.len();

            // 统计单量子比特和双量子比特门
            if num_qubits == 1 {
                profile.single_qubit_count += 1;
            } else if num_qubits == 2 {
                profile.two_qubit_count += 1;
            }

            // 统计CNOT门
            if matches!(inst.gate.gate_type, StandardGate::CX) {
                profile.cx_count += 1;
            }

            // 统计旋转门
            if self.is_rotation_gate(&inst.gate.gate_type) {
                profile.rotation_count += 1;
            }

            // 统计参数化门
            if !inst.gate.parameters.is_empty() {
                profile.parameter_count += 1;
            }
        }
    }

    /// 判断是否为旋转门
    fn is_rotation_gate(&self, gate_type: &StandardGate) -> bool {
        matches!(
            gate_type,
            StandardGate::Rx
                | StandardGate::Ry
                | StandardGate::Rz
                | StandardGate::U1
                | StandardGate::U2
                | StandardGate::U3
                | StandardGate::P
        )
    }

    /// 计算电路深度
    ///
    /// 使用简化算法：为每个量子比特维护当前时间戳
    fn compute_circuit_depth(&self, circuit: &QuantumCircuit) -> usize {
        let mut qubit_times: HashMap<usize, usize> = HashMap::new();
        let instructions = circuit.data().instructions();

        for inst in instructions {
            if inst.is_measurement() {
                continue;
            }

            // 找到所有相关量子比特的最大时间
            let max_time = inst
                .qubits
                .iter()
                .map(|q| *qubit_times.get(&q.index()).unwrap_or(&0))
                .max()
                .unwrap_or(0);

            // 更新所有相关量子比特的时间
            let new_time = max_time + 1;
            for q in &inst.qubits {
                qubit_times.insert(q.index(), new_time);
            }
        }

        // 返回最大深度
        qubit_times.values().max().copied().unwrap_or(0)
    }

    /// 计算量子比特连通性
    ///
    /// 基于双量子比特门涉及的量子比特对数量
    fn compute_qubit_connectivity(&self, circuit: &QuantumCircuit) -> f64 {
        let num_qubits = circuit.num_qubits();
        if num_qubits <= 1 {
            return 0.0;
        }

        let instructions = circuit.data().instructions();
        let mut connected_pairs: HashSet<(usize, usize)> = HashSet::new();

        for inst in instructions {
            if inst.qubits.len() == 2 {
                let q0 = inst.qubits[0].index();
                let q1 = inst.qubits[1].index();
                let pair = if q0 < q1 { (q0, q1) } else { (q1, q0) };
                connected_pairs.insert(pair);
            }
        }

        // 连通性 = 实际连接对数 / 可能的最大连接对数
        let max_pairs = num_qubits * (num_qubits - 1) / 2;
        if max_pairs > 0 {
            connected_pairs.len() as f64 / max_pairs as f64
        } else {
            0.0
        }
    }

    /// 统计活跃量子比特数（有门作用的量子比特）
    fn count_active_qubits(&self, circuit: &QuantumCircuit) -> usize {
        let instructions = circuit.data().instructions();
        let mut active: HashSet<usize> = HashSet::new();

        for inst in instructions {
            for q in &inst.qubits {
                active.insert(q.index());
            }
        }

        active.len()
    }

    /// 生成电路分析报告
    pub fn generate_report(&self, profile: &CircuitProfile) -> String {
        let mut report = String::new();

        report.push_str("电路特征分析报告\n");
        report.push_str("==================\n\n");

        report.push_str("电路规模:\n");
        report.push_str(&format!("  - 总门数: {}\n", profile.total_gates));
        report.push_str(&format!(
            "  - 电路宽度: {} 量子比特\n",
            profile.circuit_width
        ));
        report.push_str(&format!("  - 活跃量子比特: {}\n", profile.active_qubits));
        report.push_str(&format!("  - 电路深度: {}\n", profile.depth));
        report.push_str(&format!(
            "  - 每量子比特平均门数: {:.2}\n\n",
            profile.gates_per_qubit
        ));

        report.push_str("门类型分布:\n");
        report.push_str(&format!(
            "  - 单量子比特门: {} ({:.1}%)\n",
            profile.single_qubit_count,
            if profile.total_gates > 0 {
                profile.single_qubit_count as f64 / profile.total_gates as f64 * 100.0
            } else {
                0.0
            }
        ));
        report.push_str(&format!(
            "  - 双量子比特门: {} ({:.1}%)\n",
            profile.two_qubit_count,
            if profile.total_gates > 0 {
                profile.two_qubit_count as f64 / profile.total_gates as f64 * 100.0
            } else {
                0.0
            }
        ));
        report.push_str(&format!(
            "  - CNOT门: {} ({:.1}%)\n",
            profile.cx_count,
            profile.cx_density * 100.0
        ));
        report.push_str(&format!(
            "  - 旋转门: {} ({:.1}%)\n\n",
            profile.rotation_count,
            if profile.total_gates > 0 {
                profile.rotation_count as f64 / profile.total_gates as f64 * 100.0
            } else {
                0.0
            }
        ));

        report.push_str("电路特性:\n");
        report.push_str(&format!("  - CNOT密度: {:.3}\n", profile.cx_density));
        report.push_str(&format!("  - 纠缠度: {:.3}\n", profile.entanglement_degree));
        report.push_str(&format!(
            "  - 量子比特连通性: {:.3}\n",
            profile.qubit_connectivity
        ));
        report.push_str(&format!("  - 参数化门数: {}\n", profile.parameter_count));
        report.push_str(&format!(
            "  - 包含测量: {}\n\n",
            if profile.has_measurement {
                "是"
            } else {
                "否"
            }
        ));

        report.push_str("电路分类:\n");
        report.push_str(&format!(
            "  - CNOT密集型: {}\n",
            if profile.is_cnot_heavy() {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  - 旋转门密集型: {}\n",
            if profile.is_rotation_heavy() {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  - 参数化电路: {}\n",
            if profile.is_parameterized() {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  - 浅电路 (<20层): {}\n",
            if profile.is_shallow() { "是" } else { "否" }
        ));
        report.push_str(&format!(
            "  - 深电路 (>100层): {}\n",
            if profile.is_deep() { "是" } else { "否" }
        ));
        report.push_str(&format!(
            "  - 宽电路 (>10比特): {}\n",
            if profile.is_wide() { "是" } else { "否" }
        ));

        report
    }
}

impl Default for CircuitAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::QuantumCircuit;
    use crate::parameter::Parameter;

    #[test]
    fn test_empty_circuit_analysis() {
        let circuit = QuantumCircuit::new(2, 0);
        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        assert_eq!(profile.total_gates, 0);
        assert_eq!(profile.depth, 0);
        assert_eq!(profile.circuit_width, 2);
    }

    #[test]
    fn test_single_qubit_circuit() {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.h(0).unwrap();
        circuit
            .rz(0, Parameter::Float(std::f64::consts::PI / 4.0))
            .unwrap();
        circuit.x(0).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        assert_eq!(profile.total_gates, 3);
        assert_eq!(profile.single_qubit_count, 3);
        assert_eq!(profile.two_qubit_count, 0);
        assert_eq!(profile.rotation_count, 1);
        assert_eq!(profile.depth, 3);
    }

    #[test]
    fn test_cnot_heavy_circuit() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();
        circuit.cx(0, 2).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        assert_eq!(profile.cx_count, 3);
        assert_eq!(profile.two_qubit_count, 3);
        assert!(profile.is_cnot_heavy());
        assert_eq!(profile.active_qubits, 3);
    }

    #[test]
    fn test_rotation_heavy_circuit() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.rx(0, Parameter::Float(0.5)).unwrap();
        circuit.ry(0, Parameter::Float(0.5)).unwrap();
        circuit.rz(0, Parameter::Float(0.5)).unwrap();
        circuit.rx(1, Parameter::Float(0.5)).unwrap();
        circuit.ry(1, Parameter::Float(0.5)).unwrap();
        circuit.cx(0, 1).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        assert_eq!(profile.rotation_count, 5);
        assert!(profile.is_rotation_heavy());
    }

    #[test]
    fn test_circuit_depth_calculation() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.h(1).unwrap();
        circuit.h(2).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        // H gates on different qubits: depth 1
        // CX(0,1): depth 2
        // CX(1,2): depth 3
        assert_eq!(profile.depth, 3);
    }

    #[test]
    fn test_qubit_connectivity() {
        let mut circuit = QuantumCircuit::new(4, 0);
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();
        circuit.cx(2, 3).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        // 3 connected pairs out of 6 possible = 0.5
        assert!((profile.qubit_connectivity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_report_generation() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);
        let report = analyzer.generate_report(&profile);

        assert!(report.contains("电路特征分析报告"));
        assert!(report.contains("总门数: 2"));
    }
}
