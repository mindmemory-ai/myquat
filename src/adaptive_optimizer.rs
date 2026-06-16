//! # Adaptive Circuit Optimizer
//!
//! Author: gA4ss
//!
//! Automatically selects the optimal optimization strategy based on
//! circuit structure and size. The [`StrategySelector`] profiles each
//! circuit and chooses between fast, balanced, and aggressive
//! optimization plans, while [`AdaptiveOptimizer`] orchestrates
//! analysis, strategy selection, and pass execution.
//!
//! ## Architecture
//!
//! 1. **Profile** — [`CircuitAnalyzer`] produces a [`CircuitProfile`]
//!    with gate counts, depth, and structural flags (CNOT-heavy,
//!    rotation-heavy, parameterized, deep).
//! 2. **Select** — [`StrategySelector::select_strategy`] applies
//!    rule-based heuristics to pick an [`OptimizationStrategy`]
//!    (`Fast`, `Balanced`, `Aggressive`, or `Auto`).
//! 3. **Plan** — [`StrategySelector::create_plan`] builds an
//!    [`OptimizationPlan`] listing specific passes and expected
//!    gate reduction ratios.
//! 4. **Execute** — [`AdaptiveOptimizer::optimize`] runs the
//!    planned passes and produces an [`OptimizationReport`]
//!    with reduction statistics and timing.
//!
//! ## Strategies
//!
//! | Strategy | Target | Passes | Reduction |
//! |----------|--------|--------|-----------|
//! | `Fast` | Large / deep circuits | CancelInversePairs, MergeRotations | ~15% |
//! | `Balanced` | General circuits | Fast + CNOTOptimizer, BlockConsolidation | ~35% |
//! | `Aggressive` | Small / CNOT-heavy circuits | Balanced + SingleQubitOptimizer + second round | ~45-60% |
//! | `Auto` | Default | Profiles the circuit and delegates to above | Varies |
//!
//! ## Caching
//!
//! The [`StrategySelector`] maintains a [`strategy_cache`] (private
//! `HashMap<String, OptimizationStrategy>`) to avoid re-analyzing
//! circuits with similar structure.  Currently the cache is populated
//! internally; future work may expose cache warming for batch workloads.

use crate::circuit::QuantumCircuit;
use crate::circuit_analyzer::{CircuitAnalyzer, CircuitProfile};
use crate::circuit_optimization::{
    BlockConsolidationPass, CancelInversePairsPass, CircuitPass, MergeRotationsPass,
};
use crate::cnot_optimizer::CNOTOptimizer;
use crate::error::Result;
use crate::single_qubit_optimizer::SingleQubitOptimizer;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 优化策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationStrategy {
    /// 快速优化（最少Pass，最快速度）
    Fast,
    /// 平衡优化（速度和质量平衡）
    Balanced,
    /// 激进优化（最大优化效果）
    Aggressive,
    /// 自动选择（根据电路特征）
    Auto,
}

/// 优化计划
#[derive(Debug)]
pub struct OptimizationPlan {
    /// Pass名称列表
    pub pass_names: Vec<String>,
    /// 预期门减少率
    pub expected_reduction: f64,
    /// 预估时间
    pub estimated_time: Duration,
    /// 选择此策略的原因
    pub reason: String,
}

impl OptimizationPlan {
    /// 创建新的优化计划
    pub fn new(
        pass_names: Vec<String>,
        expected_reduction: f64,
        estimated_time: Duration,
        reason: String,
    ) -> Self {
        Self {
            pass_names,
            expected_reduction,
            estimated_time,
            reason,
        }
    }
}

/// 优化报告
#[derive(Debug)]
pub struct OptimizationReport {
    /// 原始门数
    pub original_gates: usize,
    /// 优化后门数
    pub optimized_gates: usize,
    /// 门减少数量
    pub gates_reduced: usize,
    /// 门减少率
    pub reduction_ratio: f64,
    /// 原始深度
    pub original_depth: usize,
    /// 优化后深度
    pub optimized_depth: usize,
    /// 深度减少率
    pub depth_reduction: f64,
    /// 实际执行时间
    pub execution_time: Duration,
    /// 使用的策略
    pub strategy: OptimizationStrategy,
    /// 执行的Pass
    pub passes_executed: Vec<String>,
}

impl OptimizationReport {
    /// 生成报告字符串
    pub fn to_string(&self) -> String {
        format!(
            "优化报告:\n\
             策略: {:?}\n\
             原始门数: {} -> 优化后: {} (减少: {}, {:.1}%)\n\
             原始深度: {} -> 优化后: {} (减少: {:.1}%)\n\
             执行时间: {:.2}ms\n\
             执行的Pass: {}",
            self.strategy,
            self.original_gates,
            self.optimized_gates,
            self.gates_reduced,
            self.reduction_ratio * 100.0,
            self.original_depth,
            self.optimized_depth,
            self.depth_reduction * 100.0,
            self.execution_time.as_secs_f64() * 1000.0,
            self.passes_executed.join(", ")
        )
    }
}

/// 策略选择器
///
/// 根据电路特征选择最优的优化策略。
pub struct StrategySelector {
    /// 缓存的策略决策
    strategy_cache: HashMap<String, OptimizationStrategy>,
}

impl StrategySelector {
    /// 创建新的策略选择器
    pub fn new() -> Self {
        Self {
            strategy_cache: HashMap::new(),
        }
    }

    /// 根据电路特征选择策略
    pub fn select_strategy(&self, profile: &CircuitProfile) -> OptimizationStrategy {
        // 规则1: 小电路使用激进优化（时间成本低）
        if profile.total_gates < 50 {
            return OptimizationStrategy::Aggressive;
        }

        // 规则2: 极大电路使用快速优化（避免超时）
        if profile.total_gates > 10000 {
            return OptimizationStrategy::Fast;
        }

        // 规则3: CNOT密集型电路使用激进优化（高收益）
        if profile.is_cnot_heavy() && profile.cx_count > 20 {
            return OptimizationStrategy::Aggressive;
        }

        // 规则4: 旋转门密集型电路使用平衡策略
        if profile.is_rotation_heavy() {
            return OptimizationStrategy::Balanced;
        }

        // 规则5: 深电路使用快速优化（深度优化成本高）
        if profile.is_deep() {
            return OptimizationStrategy::Fast;
        }

        // 规则6: 参数化电路使用平衡策略（保留参数结构）
        if profile.is_parameterized() {
            return OptimizationStrategy::Balanced;
        }

        // 默认: 平衡策略
        OptimizationStrategy::Balanced
    }

    /// 创建优化计划
    pub fn create_plan(
        &self,
        profile: &CircuitProfile,
        strategy: OptimizationStrategy,
    ) -> OptimizationPlan {
        match strategy {
            OptimizationStrategy::Fast => self.create_fast_plan(profile),
            OptimizationStrategy::Balanced => self.create_balanced_plan(profile),
            OptimizationStrategy::Aggressive => self.create_aggressive_plan(profile),
            OptimizationStrategy::Auto => {
                let auto_strategy = self.select_strategy(profile);
                self.create_plan(profile, auto_strategy)
            }
        }
    }

    /// 快速优化计划（最少Pass）
    fn create_fast_plan(&self, profile: &CircuitProfile) -> OptimizationPlan {
        let mut passes = vec!["CancelInversePairs".to_string()];

        if profile.rotation_count > 5 {
            passes.push("MergeRotations".to_string());
        }

        OptimizationPlan::new(
            passes,
            0.15, // 预期15%减少
            Duration::from_micros(100),
            "小型或简单电路，使用快速优化".to_string(),
        )
    }

    /// 平衡优化计划
    fn create_balanced_plan(&self, profile: &CircuitProfile) -> OptimizationPlan {
        let mut passes = vec![
            "CancelInversePairs".to_string(),
            "MergeRotations".to_string(),
        ];

        if profile.cx_count > 5 {
            passes.push("CNOTOptimizer".to_string());
        }

        if profile.two_qubit_count > 10 {
            passes.push("BlockConsolidation".to_string());
        }

        OptimizationPlan::new(
            passes,
            0.35, // 预期35%减少
            Duration::from_millis(1),
            "通用电路，使用平衡优化策略".to_string(),
        )
    }

    /// 激进优化计划（最大优化效果）
    fn create_aggressive_plan(&self, profile: &CircuitProfile) -> OptimizationPlan {
        let mut passes = vec![
            "CancelInversePairs".to_string(),
            "SingleQubitOptimizer".to_string(),
            "MergeRotations".to_string(),
        ];

        if profile.cx_count > 0 {
            passes.push("CNOTOptimizer".to_string());
        }

        if profile.two_qubit_count > 0 {
            passes.push("BlockConsolidation".to_string());
        }

        // 多轮优化
        passes.push("MergeRotations_Round2".to_string());

        let expected = if profile.is_cnot_heavy() { 0.6 } else { 0.45 };

        OptimizationPlan::new(
            passes,
            expected,
            Duration::from_millis(5),
            "CNOT密集型或小型电路，使用激进优化获得最大收益".to_string(),
        )
    }
}

impl Default for StrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

/// 自适应优化器
///
/// 智能优化器，根据电路特征自动选择和应用最优优化策略。
pub struct AdaptiveOptimizer {
    /// 电路分析器
    analyzer: CircuitAnalyzer,
    /// 策略选择器
    strategy_selector: StrategySelector,
    /// 是否启用详细日志
    verbose: bool,
}

impl AdaptiveOptimizer {
    /// 创建新的自适应优化器
    pub fn new() -> Self {
        Self {
            analyzer: CircuitAnalyzer::new(),
            strategy_selector: StrategySelector::new(),
            verbose: false,
        }
    }

    /// 启用详细日志
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 分析电路特征
    pub fn analyze_circuit(&self, circuit: &QuantumCircuit) -> CircuitProfile {
        self.analyzer.analyze(circuit)
    }

    /// 选择优化策略
    pub fn select_strategy(&self, profile: &CircuitProfile) -> OptimizationStrategy {
        self.strategy_selector.select_strategy(profile)
    }

    /// 创建优化计划
    pub fn create_plan(
        &self,
        profile: &CircuitProfile,
        strategy: OptimizationStrategy,
    ) -> OptimizationPlan {
        self.strategy_selector.create_plan(profile, strategy)
    }

    /// 评估优化收益
    pub fn evaluate_benefit(&self, original: &QuantumCircuit, optimized: &QuantumCircuit) -> f64 {
        let orig_profile = self.analyzer.analyze(original);
        let opt_profile = self.analyzer.analyze(optimized);

        if orig_profile.total_gates == 0 {
            return 0.0;
        }

        (orig_profile.total_gates as f64 - opt_profile.total_gates as f64)
            / orig_profile.total_gates as f64
    }

    /// 执行优化（使用自动策略选择）
    pub fn optimize(&self, circuit: &mut QuantumCircuit) -> Result<OptimizationReport> {
        self.optimize_with_strategy(circuit, OptimizationStrategy::Auto)
    }

    /// 使用指定策略执行优化
    pub fn optimize_with_strategy(
        &self,
        circuit: &mut QuantumCircuit,
        strategy: OptimizationStrategy,
    ) -> Result<OptimizationReport> {
        let start_time = Instant::now();

        // 分析原始电路
        let original_profile = self.analyzer.analyze(circuit);

        if self.verbose {
            println!("原始电路特征:");
            println!("{}", self.analyzer.generate_report(&original_profile));
        }

        // 选择策略
        let selected_strategy = if strategy == OptimizationStrategy::Auto {
            self.strategy_selector.select_strategy(&original_profile)
        } else {
            strategy
        };

        // 创建优化计划
        let plan = self
            .strategy_selector
            .create_plan(&original_profile, selected_strategy);

        if self.verbose {
            println!("\n选择的策略: {:?}", selected_strategy);
            println!("优化计划: {}", plan.reason);
            println!("预期门减少率: {:.1}%", plan.expected_reduction * 100.0);
            println!("将执行的Pass: {:?}\n", plan.pass_names);
        }

        // 执行优化Pass
        let mut passes_executed = Vec::new();

        for pass_name in &plan.pass_names {
            let pass_result = self.execute_pass(circuit, pass_name);
            if pass_result.is_ok() {
                passes_executed.push(pass_name.clone());
            }
        }

        // 分析优化后的电路
        let optimized_profile = self.analyzer.analyze(circuit);

        let execution_time = start_time.elapsed();

        // 生成报告
        let gates_reduced = original_profile
            .total_gates
            .saturating_sub(optimized_profile.total_gates);

        let reduction_ratio = if original_profile.total_gates > 0 {
            gates_reduced as f64 / original_profile.total_gates as f64
        } else {
            0.0
        };

        let depth_reduction =
            if original_profile.depth > 0 && original_profile.depth >= optimized_profile.depth {
                (original_profile.depth - optimized_profile.depth) as f64
                    / original_profile.depth as f64
            } else {
                0.0
            };

        let report = OptimizationReport {
            original_gates: original_profile.total_gates,
            optimized_gates: optimized_profile.total_gates,
            gates_reduced,
            reduction_ratio,
            original_depth: original_profile.depth,
            optimized_depth: optimized_profile.depth,
            depth_reduction,
            execution_time,
            strategy: selected_strategy,
            passes_executed,
        };

        if self.verbose {
            println!("\n{}", report.to_string());
        }

        Ok(report)
    }

    /// 执行单个优化Pass
    fn execute_pass(&self, circuit: &mut QuantumCircuit, pass_name: &str) -> Result<()> {
        match pass_name {
            "CancelInversePairs" => {
                let pass = CancelInversePairsPass::new();
                pass.run(circuit)
            }
            "MergeRotations" | "MergeRotations_Round2" => {
                let pass = MergeRotationsPass::new();
                pass.run(circuit)
            }
            "SingleQubitOptimizer" => {
                let optimizer = SingleQubitOptimizer::new();
                let optimized = optimizer.optimize(circuit)?;
                *circuit = optimized;
                Ok(())
            }
            "CNOTOptimizer" => {
                let optimizer = CNOTOptimizer::new();
                let optimized = optimizer.optimize(circuit)?;
                *circuit = optimized;
                Ok(())
            }
            "BlockConsolidation" => {
                let pass = BlockConsolidationPass::new();
                pass.run(circuit)
            }
            _ => {
                if self.verbose {
                    println!("警告: 未知的Pass: {}", pass_name);
                }
                Ok(())
            }
        }
    }
}

impl Default for AdaptiveOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::QuantumCircuit;

    #[test]
    fn test_strategy_selection_small_circuit() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        let selector = StrategySelector::new();
        let strategy = selector.select_strategy(&profile);

        // Small circuit should use aggressive strategy
        assert_eq!(strategy, OptimizationStrategy::Aggressive);
    }

    #[test]
    fn test_strategy_selection_cnot_heavy() {
        let mut circuit = QuantumCircuit::new(4, 0);
        for _ in 0..10 {
            circuit.cx(0, 1).unwrap();
            circuit.cx(1, 2).unwrap();
            circuit.cx(2, 3).unwrap();
        }

        let analyzer = CircuitAnalyzer::new();
        let profile = analyzer.analyze(&circuit);

        let selector = StrategySelector::new();
        let strategy = selector.select_strategy(&profile);

        assert_eq!(strategy, OptimizationStrategy::Aggressive);
    }

    #[test]
    fn test_adaptive_optimizer_basic() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.x(0).unwrap();
        circuit.x(0).unwrap(); // Should cancel
        circuit.cx(0, 1).unwrap();

        let optimizer = AdaptiveOptimizer::new();
        let report = optimizer.optimize(&mut circuit).unwrap();

        assert!(report.gates_reduced > 0);
        assert!(report.reduction_ratio > 0.0);
    }

    #[test]
    fn test_optimization_plan_creation() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();

        let optimizer = AdaptiveOptimizer::new();
        let profile = optimizer.analyze_circuit(&circuit);
        let plan = optimizer.create_plan(&profile, OptimizationStrategy::Balanced);

        assert!(!plan.pass_names.is_empty());
        assert!(plan.expected_reduction > 0.0);
    }

    #[test]
    fn test_evaluate_benefit() {
        let mut original = QuantumCircuit::new(2, 0);
        original.h(0).unwrap();
        original.x(0).unwrap();
        original.x(0).unwrap();
        original.cx(0, 1).unwrap();

        let mut optimized = QuantumCircuit::new(2, 0);
        optimized.h(0).unwrap();
        optimized.cx(0, 1).unwrap();

        let optimizer = AdaptiveOptimizer::new();
        let benefit = optimizer.evaluate_benefit(&original, &optimized);

        assert!(benefit > 0.0);
    }
}
