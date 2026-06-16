//! Comprehensive Gate-Level Diagnosis: All strategies × RAW/OPT × 1-step/10-step
//! Author: gA4ss — Phase 11l
//!
//! Purpose: Precisely characterize the 85-gate gap between PauliGadget and TKET.
//! Outputs per-gate-type breakdowns, per-pass reduction tracking, and block statistics.

use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, GadgetOptimizationStrategy, Hamiltonian,
    HamiltonianCompiler, PauliString, TrotterOrder,
};
// use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy; // unused currently
use myquat::Result;
use num_complex::Complex64;
use std::collections::HashMap;

// ── Gate counting utilities ──────────────────────────────────────────────

fn count_gate_types(circuit: &myquat::QuantumCircuit) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for inst in circuit.data().instructions() {
        let name = format!("{:?}", inst.gate.gate_type);
        *counts.entry(name).or_insert(0) += 1;
    }
    counts
}

fn get_key(counts: &HashMap<String, usize>, key: &str) -> usize {
    *counts.get(key).unwrap_or(&0)
}

fn total_gates(counts: &HashMap<String, usize>) -> usize {
    counts.values().sum()
}

fn print_row(label: &str, counts: &HashMap<String, usize>) {
    let cx = get_key(counts, "CX");
    let rz = get_key(counts, "Rz");
    let rx = get_key(counts, "Rx");
    let ry = get_key(counts, "Ry");
    let h = get_key(counts, "H");
    let s = get_key(counts, "S");
    let sdg = get_key(counts, "Sdg");
    let x = get_key(counts, "X");
    let y = get_key(counts, "Y");
    let z = get_key(counts, "Z");
    let known = cx + rz + ry + rx + h + s + sdg + x + y + z;
    let other = total_gates(counts) - known;
    let sq = rz + rx + ry + h + s + sdg + x + y + z + other;
    println!(
        "{:<24} {:>4} | CX:{:>4} Rz:{:>4} Rx:{:>3} H:{:>3} S:{:>3} Sdg:{:>3} other:{:>3} | SQ:{:>4}",
        label, total_gates(counts), cx, rz, rx, h, s, sdg, other, sq
    );
}

// ── Main ─────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    // H2_4q Hamiltonian: 4 qubits, 15 terms
    let mut hamiltonian = Hamiltonian::new(4);
    let term_data = [
        ("IIII", -0.8105),
        ("IIIZ", 0.1721),
        ("IIZI", -0.2228),
        ("IZII", 0.1721),
        ("ZIII", -0.2228),
        ("IIZZ", 0.1686),
        ("IZIZ", 0.1205),
        ("IZZI", 0.1686),
        ("ZIIZ", 0.1686),
        ("ZIZI", 0.1205),
        ("ZZII", 0.1686),
        ("IIXX", 0.0454),
        ("IIYY", 0.0454),
        ("XXII", 0.0454),
        ("YYII", 0.0454),
    ];

    for (pauli_str, coeff) in &term_data {
        hamiltonian.add_term(
            PauliString::from_str(pauli_str)?,
            Complex64::new(*coeff, 0.0),
        )?;
    }

    // ══════════════════════════════════════════════════════════════════════
    // SECTION 1: 1-Step RAW Analysis (no optimization, single step)
    // ══════════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SECTION 1: 1-Step RAW (no optimization, trotter_steps=1)    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!(
        "{:<24} {:>4} | {:>45} | {:>5}",
        "Strategy", "Total", "Breakdown", "SQ"
    );
    println!("{}", "-".repeat(90));

    for strategy in &[
        CompilationStrategy::PauliLevel,
        CompilationStrategy::GateLevel,
        CompilationStrategy::PauliGadget,
    ] {
        let config = CompilerConfig {
            trotter_steps: 1,
            optimization_strategy: *strategy,
            apply_circuit_optimization: false,
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            clifford_enhanced_blocks: true,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config.clone());
        let circuit = compiler.compile(&hamiltonian)?;
        let counts = count_gate_types(&circuit);

        let label = match strategy {
            CompilationStrategy::PauliLevel => "PauliLevel 1-step RAW",
            CompilationStrategy::GateLevel => "GateLevel 1-step RAW",
            CompilationStrategy::PauliGadget => "PauliGadget 1-step RAW",
        };
        print_row(label, &counts);
    }
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // SECTION 2: 10-Step RAW Analysis (no optimization, 10 steps)
    // ══════════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SECTION 2: 10-Step RAW (no optimization, trotter_steps=10)  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!(
        "{:<24} {:>4} | {:>45} | {:>5}",
        "Strategy", "Total", "Breakdown", "SQ"
    );
    println!("{}", "-".repeat(90));

    for strategy in &[
        CompilationStrategy::PauliLevel,
        CompilationStrategy::GateLevel,
        CompilationStrategy::PauliGadget,
    ] {
        let config = CompilerConfig {
            trotter_steps: 10,
            optimization_strategy: *strategy,
            apply_circuit_optimization: false,
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            clifford_enhanced_blocks: true,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config.clone());
        let circuit = compiler.compile(&hamiltonian)?;
        let counts = count_gate_types(&circuit);

        let label = match strategy {
            CompilationStrategy::PauliLevel => "PauliLevel 10-step RAW",
            CompilationStrategy::GateLevel => "GateLevel 10-step RAW",
            CompilationStrategy::PauliGadget => "PauliGadget 10-step RAW",
        };
        print_row(label, &counts);
    }
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // SECTION 3: 10-Step OPT (with full optimization)
    // ══════════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SECTION 3: 10-Step OPT (full optimization, trotter_steps=10)║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!(
        "{:<24} {:>4} | {:>45} | {:>5}",
        "Strategy", "Total", "Breakdown", "SQ"
    );
    println!("{}", "-".repeat(90));

    let mut ref_unitary = None;

    for strategy in &[
        CompilationStrategy::PauliLevel,
        CompilationStrategy::GateLevel,
        CompilationStrategy::PauliGadget,
    ] {
        let config = CompilerConfig {
            trotter_steps: 10,
            optimization_strategy: *strategy,
            apply_circuit_optimization: true,
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            clifford_enhanced_blocks: true,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config.clone());
        let circuit = compiler.compile(&hamiltonian)?;
        let counts = count_gate_types(&circuit);

        let label = match strategy {
            CompilationStrategy::PauliLevel => "PauliLevel 10-step OPT",
            CompilationStrategy::GateLevel => "GateLevel 10-step OPT",
            CompilationStrategy::PauliGadget => "PauliGadget 10-step OPT",
        };
        print_row(label, &counts);

        // Store reference unitary for fidelity comparison
        if ref_unitary.is_none() {
            use std::collections::HashMap;
            if let Some(u) = circuit.unitary(&HashMap::new()).ok() {
                ref_unitary = Some(u);
            }
        }
    }
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // SECTION 4: Per-Pass Optimization Tracking (PauliGadget only)
    // ══════════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SECTION 4: Per-Pass Reduction (PauliGadget, 10 steps)       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Build RAW circuit
    let raw_config = CompilerConfig {
        trotter_steps: 10,
        optimization_strategy: CompilationStrategy::PauliGadget,
        apply_circuit_optimization: false,
        pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
        clifford_enhanced_blocks: true,
        ..Default::default()
    };
    let raw_compiler = HamiltonianCompiler::new(raw_config);
    let mut circuit = raw_compiler.compile(&hamiltonian)?;

    let raw_counts = count_gate_types(&circuit);
    println!(
        "{:<30} {:>4} | CX:{:>4} Rz:{:>4} Rx:{:>3} H:{:>3} S:{:>3} Sdg:{:>3} other:{:>3}",
        "RAW (as synthesized)",
        total_gates(&raw_counts),
        get_key(&raw_counts, "CX"),
        get_key(&raw_counts, "Rz"),
        get_key(&raw_counts, "Rx"),
        get_key(&raw_counts, "H"),
        get_key(&raw_counts, "S"),
        get_key(&raw_counts, "Sdg"),
        total_gates(&raw_counts)
            - get_key(&raw_counts, "CX")
            - get_key(&raw_counts, "Rz")
            - get_key(&raw_counts, "Rx")
            - get_key(&raw_counts, "H")
            - get_key(&raw_counts, "S")
            - get_key(&raw_counts, "Sdg"),
    );

    // Step 1: TrotterAwarePass
    {
        use myquat::circuit_optimization::{CircuitPass, TrotterAwarePass};
        TrotterAwarePass::new().run(&mut circuit)?;
        let counts = count_gate_types(&circuit);
        println!(
            "{:<30} {:>4} | CX:{:>4} Rz:{:>4} Rx:{:>3} H:{:>3} S:{:>3} Sdg:{:>3} other:{:>3}",
            "After TrotterAwarePass",
            total_gates(&counts),
            get_key(&counts, "CX"),
            get_key(&counts, "Rz"),
            get_key(&counts, "Rx"),
            get_key(&counts, "H"),
            get_key(&counts, "S"),
            get_key(&counts, "Sdg"),
            total_gates(&counts)
                - get_key(&counts, "CX")
                - get_key(&counts, "Rz")
                - get_key(&counts, "Rx")
                - get_key(&counts, "H")
                - get_key(&counts, "S")
                - get_key(&counts, "Sdg"),
        );
    }

    // Step 2: Iterative convergence loop (level_2 + level_4_core)
    {
        use myquat::circuit_optimization::PassManager;
        let mut prev_size = circuit.size();
        let mut iterations = 0;
        loop {
            PassManager::level_2().run(&mut circuit)?;
            if circuit.size() > 50 {
                PassManager::level_4_core().run(&mut circuit)?;
            }
            let new_size = circuit.size();
            if new_size >= prev_size {
                break;
            }
            prev_size = new_size;
            iterations += 1;
        }
        let counts = count_gate_types(&circuit);
        println!(
            "{:<30} {:>4} | CX:{:>4} Rz:{:>4} Rx:{:>3} H:{:>3} S:{:>3} Sdg:{:>3} other:{:>3}",
            format!("After convergence ({} iter)", iterations),
            total_gates(&counts),
            get_key(&counts, "CX"),
            get_key(&counts, "Rz"),
            get_key(&counts, "Rx"),
            get_key(&counts, "H"),
            get_key(&counts, "S"),
            get_key(&counts, "Sdg"),
            total_gates(&counts)
                - get_key(&counts, "CX")
                - get_key(&counts, "Rz")
                - get_key(&counts, "Rx")
                - get_key(&counts, "H")
                - get_key(&counts, "S")
                - get_key(&counts, "Sdg"),
        );
    }

    // Step 3: PhasePolynomialPass + post-cleanup
    {
        use myquat::circuit_optimization::CircuitPass;
        use myquat::circuit_optimization::{CommutativeCancellationPass, TemplateMatchingPass};
        use myquat::cnot_optimizer::CNOTOptimizer;
        use myquat::single_qubit_optimizer::SingleQubitOptimizer;

        myquat::phase_polynomial::PhasePolynomialPass::default().run(&mut circuit)?;
        let counts = count_gate_types(&circuit);
        println!(
            "{:<30} {:>4} | CX:{:>4} Rz:{:>4} Rx:{:>3} H:{:>3} S:{:>3} Sdg:{:>3} other:{:>3}",
            "After PhasePolynomialPass",
            total_gates(&counts),
            get_key(&counts, "CX"),
            get_key(&counts, "Rz"),
            get_key(&counts, "Rx"),
            get_key(&counts, "H"),
            get_key(&counts, "S"),
            get_key(&counts, "Sdg"),
            total_gates(&counts)
                - get_key(&counts, "CX")
                - get_key(&counts, "Rz")
                - get_key(&counts, "Rx")
                - get_key(&counts, "H")
                - get_key(&counts, "S")
                - get_key(&counts, "Sdg"),
        );

        CNOTOptimizer::new().run(&mut circuit)?;
        CommutativeCancellationPass::new().run(&mut circuit)?;
        TemplateMatchingPass::new().run(&mut circuit)?;
        SingleQubitOptimizer::new().run(&mut circuit)?;
        let counts = count_gate_types(&circuit);
        println!(
            "{:<30} {:>4} | CX:{:>4} Rz:{:>4} Rx:{:>3} H:{:>3} S:{:>3} Sdg:{:>3} other:{:>3}",
            "After final cleanup",
            total_gates(&counts),
            get_key(&counts, "CX"),
            get_key(&counts, "Rz"),
            get_key(&counts, "Rx"),
            get_key(&counts, "H"),
            get_key(&counts, "S"),
            get_key(&counts, "Sdg"),
            total_gates(&counts)
                - get_key(&counts, "CX")
                - get_key(&counts, "Rz")
                - get_key(&counts, "Rx")
                - get_key(&counts, "H")
                - get_key(&counts, "S")
                - get_key(&counts, "Sdg"),
        );

        // Total reduction summary
        let final_counts = count_gate_types(&circuit);
        println!(
            "\n  Reduction: {} → {} gates ({:.1}% removed)",
            total_gates(&raw_counts),
            total_gates(&final_counts),
            (1.0 - total_gates(&final_counts) as f64 / total_gates(&raw_counts) as f64) * 100.0,
        );
        println!(
            "  CX:  {} → {}  ({:.1}% removed)",
            get_key(&raw_counts, "CX"),
            get_key(&final_counts, "CX"),
            (1.0 - get_key(&final_counts, "CX") as f64 / get_key(&raw_counts, "CX") as f64) * 100.0,
        );
        let sq_raw = total_gates(&raw_counts) - get_key(&raw_counts, "CX");
        let sq_final = total_gates(&final_counts) - get_key(&final_counts, "CX");
        println!(
            "  SQ:  {} → {}  ({:.1}% removed)",
            sq_raw,
            sq_final,
            (1.0 - sq_final as f64 / sq_raw as f64) * 100.0,
        );
        println!(
            "  S+Sdg: {} → {}  ({:.1}% removed)",
            get_key(&raw_counts, "S") + get_key(&raw_counts, "Sdg"),
            get_key(&final_counts, "S") + get_key(&final_counts, "Sdg"),
            (1.0 - (get_key(&final_counts, "S") + get_key(&final_counts, "Sdg")) as f64
                / (get_key(&raw_counts, "S") + get_key(&raw_counts, "Sdg")) as f64)
                * 100.0,
        );
    }

    println!();

    // ══════════════════════════════════════════════════════════════════════
    // SECTION 5: Fidelity Check
    // ══════════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SECTION 5: Fidelity Check (PauliGadget vs GateLevel ref)    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    use std::collections::HashMap as StdHashMap;
    let u_pg = circuit.unitary(&StdHashMap::new()).ok();
    if let (Some(up), Some(ug)) = (&u_pg, &ref_unitary) {
        let n = ug.nrows();
        let mut overlap = Complex64::new(0.0, 0.0);
        for i in 0..n {
            for j in 0..n {
                overlap += up[(i, j)].conj() * ug[(i, j)];
            }
        }
        let phi = overlap.arg();
        // Phase-insensitive fidelity: |⟨U_PG|e^{iφ}·U_GL⟩| / dim
        let phase_insensitive_overlap = overlap * Complex64::new((-phi).cos(), (-phi).sin());
        let fidelity = phase_insensitive_overlap.norm() / (n as f64);
        let dist = (1.0 - fidelity).sqrt();
        println!("  Phase-insensitive distance to GateLevel: {:.6}", dist);
        if dist < 1e-6 {
            println!("  ✅ PASS — unitaries match");
        } else if dist < 1e-3 {
            println!("  ⚠️  WARNING — small deviation detected");
        } else {
            println!("  ❌ FAIL — fidelity regression!");
        }
    } else {
        println!("  Could not compute unitaries for fidelity check.");
    }
    println!();

    // ══════════════════════════════════════════════════════════════════════
    // SECTION 6: Clifford Overhead Breakdown
    // ══════════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SECTION 6: Clifford Overhead Breakdown (1-step RAW)         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // PauliGadget 1-step RAW — S/Sdg come from GreedyPauliSimp annotations
    {
        let config = CompilerConfig {
            trotter_steps: 1,
            optimization_strategy: CompilationStrategy::PauliGadget,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let c = compiler.compile(&hamiltonian)?;
        let counts = count_gate_types(&c);

        let cx = get_key(&counts, "CX");
        let sz = get_key(&counts, "Rz");
        let h = get_key(&counts, "H");
        let rx = get_key(&counts, "Rx");
        let s = get_key(&counts, "S");
        let sdg = get_key(&counts, "Sdg");

        println!("  PauliGadget 1-step RAW Clifford breakdown:");
        println!(
            "    Basis change H:   {}  (from X→Z for X-type operators)",
            h
        );
        println!(
            "    Basis change Rx:  {}  (from Y→Z for Y-type operators)",
            rx
        );
        println!(
            "    Clifford S:       {}  (post-conjugation from GreedyPauliSimp)",
            s
        );
        println!(
            "    Clifford Sdg:     {}  (pre-conjugation from GreedyPauliSimp)",
            sdg
        );
        println!(
            "    S+Sdg overhead:   {}  (pure Clifford annotation gates)",
            s + sdg
        );
        println!("    CX in CNOT trees: {}", cx);
        println!("    Rz rotations:     {}", sz);
    }
    println!();

    // ── GateLevel vs PauliGadget 1-step: where does the extra overhead come from? ──
    {
        let gl_config = CompilerConfig {
            trotter_steps: 1,
            optimization_strategy: CompilationStrategy::GateLevel,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let gl_c = HamiltonianCompiler::new(gl_config).compile(&hamiltonian)?;
        let gl = count_gate_types(&gl_c);

        let pg_config = CompilerConfig {
            trotter_steps: 1,
            optimization_strategy: CompilationStrategy::PauliGadget,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let pg_c = HamiltonianCompiler::new(pg_config).compile(&hamiltonian)?;
        let pg = count_gate_types(&pg_c);

        println!("  1-step RAW comparison (GateLevel vs PauliGadget):");
        println!(
            "    GateLevel:    {} total ({} CX + {} SQ)",
            total_gates(&gl),
            get_key(&gl, "CX"),
            total_gates(&gl) - get_key(&gl, "CX")
        );
        println!(
            "    PauliGadget:  {} total ({} CX + {} SQ)",
            total_gates(&pg),
            get_key(&pg, "CX"),
            total_gates(&pg) - get_key(&pg, "CX")
        );
        println!(
            "    Difference:   {:+} total ({:+} CX + {:+} SQ)",
            total_gates(&pg) as i64 - total_gates(&gl) as i64,
            get_key(&pg, "CX") as i64 - get_key(&gl, "CX") as i64,
            (total_gates(&pg) - get_key(&pg, "CX")) as i64
                - (total_gates(&gl) - get_key(&gl, "CX")) as i64,
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // SECTION 7: {CX, Rz} Segment Analysis
    // ══════════════════════════════════════════════════════════════════════
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SECTION 7: {{CX, Rz}} Segment Analysis (PauliGadget 1-step)  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Count consecutive {CX, Rz} segments in the RAW circuit
    {
        let config = CompilerConfig {
            trotter_steps: 1,
            optimization_strategy: CompilationStrategy::PauliGadget,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let c = compiler.compile(&hamiltonian)?;

        let mut segments: Vec<usize> = Vec::new(); // lengths of each segment
        let mut current_segment_len = 0usize;
        let mut in_segment = false;

        for inst in c.data().instructions() {
            let is_cx_or_rz = matches!(
                inst.gate.gate_type,
                myquat::StandardGate::CX | myquat::StandardGate::Rz
            );
            if is_cx_or_rz {
                if !in_segment {
                    in_segment = true;
                    current_segment_len = 0;
                }
                current_segment_len += 1;
            } else {
                if in_segment {
                    segments.push(current_segment_len);
                    in_segment = false;
                }
            }
        }
        if in_segment {
            segments.push(current_segment_len);
        }

        segments.sort();
        println!("  {{CX, Rz}} segments found: {}", segments.len());
        if !segments.is_empty() {
            println!(
                "  Min segment: {} gates, Max segment: {} gates",
                segments.first().unwrap(),
                segments.last().unwrap()
            );
            let avg = segments.iter().sum::<usize>() as f64 / segments.len() as f64;
            println!("  Average segment: {:.1} gates", avg);
        }

        // How many non-CX, non-Rz gates are between segments?
        let non_segment = total_gates(&count_gate_types(&c))
            - c.data()
                .instructions()
                .iter()
                .filter(|inst| {
                    matches!(
                        inst.gate.gate_type,
                        myquat::StandardGate::CX | myquat::StandardGate::Rz
                    )
                })
                .count();
        println!("  Non-{{CX, Rz}} gates (between segments): {}", non_segment);
    }

    Ok(())
}
