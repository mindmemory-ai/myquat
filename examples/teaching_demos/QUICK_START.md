# MyQuat Teaching Demos Quick Start Guide

Author: gA4ss  
Date: 2026-02-12

## 🚀 Quick Start

### Run a Single Example

```bash
# Basic example - Bell state (5 minutes)
cargo run --example demo_01_bell_state

# Complete tutorial - Beginner (15 minutes)
cargo run --example demo_02_beginner_tutorial

# Visualization demo (10 minutes)
cargo run --example demo_04_visualization

# Hamiltonian forward compilation (15 minutes)
cargo run --example demo_05_hamiltonian_forward

# Hamiltonian backward analysis (15 minutes)
cargo run --example demo_06_hamiltonian_backward

# Grover search algorithm (15 minutes)
cargo run --example demo_07_grover_algorithm

# VQE chemistry simulation (20 minutes)
cargo run --example demo_08_vqe_chemistry
```

### Run All Examples

```bash
cd examples/teaching_demos
./test_all.sh
```

---

## 📋 Example List

| Number | Name | Topic | Difficulty | Duration |
|--------|------|-------|------------|----------|
| 01 | Bell State Preparation | Quantum Entanglement Basics | ⭐ | 5min |
| 02 | Beginner Tutorial | Complete API Introduction | ⭐⭐ | 15min |
| 03 | Quantum Gates Demo | All Standard Gates | ⭐⭐ | 10min |
| 04 | Visualization System | Circuit Visualization | ⭐⭐ | 10min |
| 05 | Hamiltonian Forward | Forward Compilation | ⭐⭐⭐ | 15min |
| 06 | Hamiltonian Backward | Backward Analysis | ⭐⭐⭐ | 15min |
| 07 | Grover Algorithm | Quantum Search | ⭐⭐⭐ | 15min |
| 08 | VQE Chemistry | Quantum Chemistry | ⭐⭐⭐⭐ | 20min |
| 09 | Interactive Tutorial | Complete Features | ⭐⭐⭐ | 30min |
| 10 | Adaptive Optimization | Circuit Optimization | ⭐⭐⭐ | 15min |

---

## 🎯 Recommended Presentation Order

### Quick Demo (30 minutes)
**For**: Technical sharing, quick showcase

```bash
# 1. Basic concepts (5 minutes)
cargo run --example demo_01_bell_state

# 2. Visualization system (5 minutes)
cargo run --example demo_04_visualization

# 3. Hamiltonian forward (10 minutes)
cargo run --example demo_05_hamiltonian_forward

# 4. Adaptive optimization (10 minutes)
cargo run --example demo_10_adaptive_optimization
```

### In-depth Demo (1 hour)
**For**: Technical training, detailed explanation

```bash
# 1. Basics and API (20 minutes)
cargo run --example demo_01_bell_state
cargo run --example demo_02_beginner_tutorial

# 2. Hamiltonian theory (30 minutes)
cargo run --example demo_05_hamiltonian_forward
cargo run --example demo_06_hamiltonian_backward

# 3. Quantum algorithms (10 minutes)
cargo run --example demo_07_grover_algorithm
```

### Complete Demo (2-3 hours)
**For**: Teaching courses, systematic training

Run all 10 examples in order.

---

## 💡 Presentation Tips

### Pre-presentation Preparation
```bash
# 1. Compile all examples in advance
cargo build --examples

# 2. Clear terminal, set appropriate font size
clear

# 3. Prepare backup screenshots of example outputs
```

### During Presentation
- **Control output speed**: Use `| less` or `| more` for paging
- **Save output**: Use `> output.txt` to save results
- **Highlight**: Mark key parts of code in advance
- **Interactive sessions**: Encourage audience questions and discussions

### Key Presentation Points

#### Demo 01 - Bell State
- **Highlight**: The magical phenomenon of quantum entanglement
- **Interactive**: Explain why measurement results are always consistent
- **Theory**: Difference between superposition and entanglement

#### Demo 04 - Visualization
- **Highlight**: Comparison of multiple visualization styles
- **Interactive**: Modify visualization parameters live
- **Application**: How to use for debugging and analysis

#### Demo 05 - Hamiltonian Forward
- **Highlight**: Mathematical principles of Trotter decomposition
- **Interactive**: Adjust step count to observe precision changes
- **Application**: Quantum chemistry simulation

#### Demo 06 - Hamiltonian Backward
- **Highlight**: How to understand the physical meaning of existing circuits
- **Interactive**: Show extraction from circuit to Hamiltonian
- **Application**: Circuit analysis and optimization

#### Demo 07 - Grover Algorithm
- **Highlight**: Practical example of quantum speedup
- **Interactive**: Calculate optimal iteration count
- **Application**: Search and optimization problems

#### Demo 10 - Adaptive Optimization
- **Highlight**: Intelligent circuit optimization system
- **Interactive**: Compare different optimization strategies
- **Application**: Practical circuit optimization

---

## 🖥️ Output Examples

### Bell State Output (demo_01)
```
=== Bell State Example ===
q[0] ─H──●──M────
q[1] ────⊕─────M─

Measurement results:
|00⟩: 50.2%
|11⟩: 49.8%
```

### Visualization Output (demo_04)
```
=== Circuit Visualization Demo ===

Compact style:
q[0] ─H──●─
q[1] ────⊕─

Detailed style:
q[0] ──[ H ]────●────
q[1] ────────[ X ]───

Statistics:
Total gates: 2
Circuit depth: 2
```

---

## 📊 Performance Notes

- **Compilation time**: First time ~1-2 minutes, subsequent incremental compilation ~5-10 seconds
- **Runtime**: Most examples < 1 second, VQE example ~5-10 seconds
- **Memory usage**: < 100MB
- **Release mode**: Use `cargo run --release` for 10-100x speedup

---

## 🐛 Common Issues

### Q: Compilation is slow?
A: First compilation needs to download dependencies, subsequent runs will be much faster. You can use `cargo build --examples` to compile in advance.

### Q: An example fails to run?
A: Check Rust version (requires >= 1.70), run `cargo clean && cargo build`.

### Q: Too much output, hard to read?
A: Use pipe commands:
```bash
cargo run --example demo_01_bell_state | less
cargo run --example demo_02_beginner_tutorial > output.txt
```

### Q: Want to modify example code?
A: Example source code is in `examples/teaching_demos/*.rs`, can be edited directly.

---

## 📚 More Resources

- **Complete documentation**: See `README.md`
- **Theoretical background**: See `docs/hamiltonian_circuit_theory/`
- **Best practices**: See `docs/BEST_PRACTICES.md`
- **API documentation**: Run `cargo doc --open`

---

## 🎓 Learning Suggestions

### First Time Use
1. Run demo_01 first to experience basic features
2. Read demo_02 code to learn API
3. Try modifying parameters to observe changes

### In-depth Learning
1. Read example source code and comments
2. Refer to theoretical documentation to understand principles
3. Try implementing your own examples

### Teaching Use
1. Prepare PPT in advance to accompany code demonstration
2. Prepare interactive questions to guide discussion
3. Provide after-class exercises to consolidate knowledge

---

Wishing you a successful presentation! 🎉
