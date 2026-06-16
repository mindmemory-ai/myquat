//! Gap analysis tool: compares MyQuat strategies against TKET benchmark results.
//!
//! Reads TKET results JSON from `experiments/tket_comparison/tket_results.json`
//! and produces a per-Hamiltonian, per-gate-type gap breakdown.
//!
//! Usage:
//!   cargo run --example gap_analysis -- [tket_results.json]

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "experiments/tket_comparison/tket_results.json".to_string());

    if !Path::new(&path).exists() {
        eprintln!("Error: {} not found. Run benchmark_vs_tket.py first.", path);
        eprintln!("  python3 experiments/tket_comparison/benchmark_vs_tket.py");
        std::process::exit(1);
    }

    let data: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;

    // Print header
    println!(
        "{:=^90}",
        format!(
            " MyQuat vs TKET Gap Analysis (steps={}) ",
            data["config"]["steps"]
        )
    );
    println!();

    // ── TKET results ──────────────────────────────────────────────────────
    println!("┌─ TKET Results ─────────────────────────────────────────────┐");
    println!(
        "{:<16} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Hamiltonian", "Strategy", "Raw", "Opt", "2Q", "Depth"
    );
    println!("{:-<60}", "");

    if let Some(hams) = data["hamiltonians"].as_object() {
        for (name, strats) in hams {
            if let Some(strats_obj) = strats.as_object() {
                for (strat, result) in strats_obj {
                    println!(
                        "{:<16} {:>8} {:>8} {:>8} {:>8} {:>8}",
                        name,
                        strat,
                        result["raw_gates"],
                        result["opt_gates"],
                        result["two_qubit_gates"],
                        result["depth"],
                    );
                }
            }
        }
    }
    println!();

    // ── Gate-type gap breakdown ──────────────────────────────────────────
    println!("┌─ Gate Type Gap (TKET PauliSimp+Greedy vs MyQuat PauliGadget) ─┐");

    // For each Hamiltonian, show per-gate-type counts
    if let Some(hams) = data["hamiltonians"].as_object() {
        for (name, strats) in hams {
            if let Some(strats_obj) = strats.as_object() {
                if let Some(tket) = strats_obj.get("PauliSimp+Greedy") {
                    let counts = &tket["opt_counts"];
                    let total = tket["opt_gates"].as_u64().unwrap_or(0);

                    // Collect gate types with consistent naming
                    let mut gate_summary: Vec<(String, u64)> = Vec::new();
                    if let Some(obj) = counts.as_object() {
                        for (k, v) in obj {
                            gate_summary.push((k.clone(), v.as_u64().unwrap_or(0)));
                        }
                    }
                    gate_summary.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

                    println!(
                        "\n  {} ({} gates, {} depth):",
                        name,
                        total,
                        tket["depth"].as_u64().unwrap_or(0)
                    );
                    let gate_str: String = gate_summary
                        .iter()
                        .map(|(k, v)| format!("{}:{}", k, v))
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!("    {}", gate_str);
                }
            }
        }
    }

    println!("\n┌─ MyQuat H2_4q Reference (PauliGadget OPT) ─────────────────┐");
    println!("  Run `cargo run --example compare_strategies` for full comparison.");
    println!("  Expected: 416 gates (166 CX, 130 Rz, 120 H)");
    println!();
    println!("┌─ Action Items ───────────────────────────────────────────────┐");
    println!("  1. Compare per-Hamiltonian gaps above with MyQuat output");
    println!("  2. Focus 11n optimization on Hamiltonians with largest Rz gap");
    println!("  3. Focus 11o optimization on Hamiltonians with largest CX gap");

    Ok(())
}
