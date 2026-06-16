//! Deep analysis: what PauliLevel actually produces for H2_4q
use myquat::hamiltonian::{self, PauliOperator, PauliString, *};
use myquat::*;
use num_complex::Complex64;

fn main() {
    let mut h = Hamiltonian::new(4);
    for (ps, coeff) in [
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
    ] {
        h.add_term(
            PauliString::from_str(ps).unwrap(),
            Complex64::new(coeff, 0.0),
        )
        .unwrap();
    }

    // Analyze QWC block structure
    println!("=== QWC Block Analysis ===\n");
    let mut blocks: Vec<Vec<(PauliString, Complex64)>> = vec![];
    'outer: for term in &h.terms {
        for block in &mut blocks {
            if block.iter().all(|(p, _)| is_qwc(p, &term.pauli_string)) {
                block.push((term.pauli_string.clone(), term.coefficient));
                continue 'outer;
            }
        }
        blocks.push(vec![(term.pauli_string.clone(), term.coefficient)]);
    }

    for (i, block) in blocks.iter().enumerate() {
        // Compute mstr
        let mstr = block
            .iter()
            .map(|(p, _)| p)
            .fold(PauliString::identity(4), |acc, p| pOR(&acc, p));
        let nodes: Vec<usize> = mstr
            .operators
            .iter()
            .enumerate()
            .filter(|(_, o)| **o != PauliOperator::I)
            .map(|(i, _)| i)
            .collect();
        println!(
            "Block {}: {} terms, mstr={}, nodes={:?}",
            i,
            block.len(),
            mstr.to_string_repr(),
            nodes
        );

        // Categorize terms by active set
        for (pauli, _) in block {
            let active: Vec<usize> = pauli
                .operators
                .iter()
                .enumerate()
                .filter(|(_, o)| **o != PauliOperator::I)
                .map(|(i, _)| i)
                .collect();
            let ops: Vec<String> = pauli.operators.iter().map(|o| format!("{:?}", o)).collect();
            println!(
                "  {} active={:?} ops={:?}",
                pauli.to_string_repr(),
                active,
                ops
            );
        }
    }

    // Show actual synthesized instructions (no optimization)
    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 1,
        evolution_time: 1.0,
        optimization_strategy: CompilationStrategy::PauliLevel,
        group_commuting_terms: true,
        apply_circuit_optimization: false,
        ..CompilerConfig::default()
    };
    let circuit = HamiltonianCompiler::new(config).compile(&h).unwrap();
    let insts = circuit.data().instructions();

    println!(
        "\n=== Full raw circuit ({} instructions) ===\n",
        insts.len()
    );
    for (j, inst) in insts.iter().enumerate() {
        let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
        let params: Vec<String> = inst
            .gate
            .parameters
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();
        let param_str = if params.is_empty() {
            String::new()
        } else {
            format!("({})", params.join(", "))
        };
        println!(
            "  {:3}: {:?}{} on {:?}",
            j, inst.gate.gate_type, param_str, qubits
        );
    }

    // Count how many non-prefix terms get their own tree
    // For sorted_nodes = [0,1,2,3], max_prefix_len = 2:
    // Prefix: {0}, {0,1}
    // Non-prefix: {1}, {2}, {3}, {1,2}, {1,3}, {2,3}, {0,2}, {0,3} etc.
    // Let me detect from the circuit
    println!("\n=== Structure Analysis ===");
    let cx_count = insts
        .iter()
        .filter(|i| i.gate.gate_type == StandardGate::CX)
        .count();
    println!(
        "CX count: {} ({} forward+back pairs)",
        cx_count,
        cx_count / 2
    );
    println!("Non-CX count: {}", insts.len() - cx_count);
    println!("Expected minimum: shared tree uses 1 CX each way = 2 CX");
    println!("Remaining {} CX come from per-term trees", cx_count - 2);
    println!(
        "Each per-term uses ~2 CX → {} non-prefix terms",
        (cx_count - 2) / 2
    );
    println!();
    println!(
        "NOTE: GateLevel uses 1 CX per term each way = 2*{} = {} CX",
        h.terms.len() - 1,
        2 * (h.terms.len() - 1)
    ); // -1 for identity term
}

fn is_qwc(a: &PauliString, b: &PauliString) -> bool {
    a.operators
        .iter()
        .zip(b.operators.iter())
        .all(|(oa, ob)| oa == ob || *oa == PauliOperator::I || *ob == PauliOperator::I)
}

fn pOR(a: &PauliString, b: &PauliString) -> PauliString {
    let ops: Vec<PauliOperator> = a
        .operators
        .iter()
        .zip(b.operators.iter())
        .map(|(oa, ob)| if *oa != PauliOperator::I { *oa } else { *ob })
        .collect();
    PauliString::new(ops, a.coefficient)
}
