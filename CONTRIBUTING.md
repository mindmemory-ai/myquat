# Contributing Guide

Thank you for your interest in the MyQuat project! We welcome all forms of contributions, including bug reports, feature suggestions, documentation improvements, and code contributions.

## Code of Conduct

By participating in this project, you agree to abide by our code of conduct:

- Respect others
- Maintain professionalism and courtesy
- Accept constructive criticism
- Focus on what is best for the project
- Show empathy towards community members

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue and include:

- **Clear Title**: Concise description of the problem
- **Detailed Description**: Complete problem description
- **Reproduction Steps**: How to reproduce the issue
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happened
- **Environment Information**:
  - Rust version (`rustc --version`)
  - MyQuat version
  - Operating system
  - Relevant dependency versions

**Example**:

```markdown
### Bug Description
State vector simulator experiences memory overflow when handling 30+ qubits

### Reproduction Steps
1. Create a 30 qubit circuit
2. Add 100 random gates
3. Run simulator.run()

### Environment
- Rust: 1.79.0
- MyQuat: 0.1.0
- OS: Ubuntu 22.04
- RAM: 16GB
```

### Feature Requests

If you have an idea for a new feature, please create an issue and include:

- **Feature Description**: What feature you want
- **Use Case**: Why this feature is needed
- **Suggested Implementation**: (Optional) How to implement it
- **Alternatives**: (Optional) Other possible approaches

### Code Contributions

#### Development Workflow

1. **Fork the Repository**
   ```bash
   # Click the Fork button on GitHub
   git clone https://github.com/YOUR_USERNAME/myquat.git
   cd myquat
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/amazing-feature
   # or
   git checkout -b fix/bug-description
   ```

3. **Make Changes**
   - Write code
   - Add tests
   - Update documentation

4. **Test Changes**
   ```bash
   # Run all tests
   cargo test
   
   # Check code style
   cargo fmt --check
   
   # Run linter
   cargo clippy -- -D warnings
   
   # (Optional) Run benchmarks
   cargo bench
   ```

5. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add amazing feature"
   ```

6. **Push Branch**
   ```bash
   git push origin feature/amazing-feature
   ```

7. **Create Pull Request**
   - Visit the GitHub repository
   - Click "New Pull Request"
   - Fill in the PR description
   - Wait for review

#### Commit Message Convention

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `style`: Code formatting (no functional changes)
- `refactor`: Refactoring
- `perf`: Performance optimization
- `test`: Test-related
- `chore`: Build/tooling-related

**Example**:

```
feat(simulator): add density matrix simulator

Implement a new density matrix simulator for mixed state
quantum systems. Supports:
- Partial trace operations
- Depolarizing noise
- Thermal relaxation

Closes #123
```

#### Code Style

Follow the official Rust style guide:

```bash
# Format code
cargo fmt

# Check code
cargo clippy
```

**Key Principles**:

1. **Naming Conventions**
   ```rust
   // Modules, functions: snake_case
   mod quantum_circuit;
   fn apply_hadamard() {}
   
   // Types, Traits: PascalCase
   struct QuantumCircuit;
   trait Simulator;
   
   // Constants: SCREAMING_SNAKE_CASE
   const MAX_QUBITS: usize = 30;
   ```

2. **Documentation Comments**
   ```rust
   /// Applies a Hadamard gate to the specified qubit.
   ///
   /// # Arguments
   ///
   /// * `qubit` - The target qubit index
   ///
   /// # Examples
   ///
   /// ```
   /// let mut circuit = QuantumCircuit::new(2, 0);
   /// circuit.h(0)?;
   /// ```
   ///
   /// # Errors
   ///
   /// Returns an error if the qubit index is out of bounds.
   pub fn h(&mut self, qubit: usize) -> Result<()> {
       // Implementation
   }
   ```

3. **Error Handling**
   ```rust
   // Use Result type
   pub fn apply_gate(&mut self, gate: Gate) -> Result<()> {
       if !self.is_valid_qubit(gate.target) {
           return Err(MyQuatError::invalid_qubit(gate.target));
       }
       // ...
       Ok(())
   }
   ```

4. **Testing**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_hadamard_gate() {
           let mut circuit = QuantumCircuit::new(1, 0);
           assert!(circuit.h(0).is_ok());
           assert_eq!(circuit.size(), 1);
       }
       
       #[test]
       fn test_invalid_qubit() {
           let mut circuit = QuantumCircuit::new(1, 0);
           assert!(circuit.h(2).is_err());
       }
   }
   ```

#### Performance Considerations

1. **Avoid Unnecessary Allocations**
   ```rust
   // Good: In-place modification
   fn normalize_inplace(state: &mut Array1<Complex64>) {
       let norm = compute_norm(state);
       state.mapv_inplace(|x| x / norm);
   }
   
   // Avoid: Creating new array
   fn normalize(state: &Array1<Complex64>) -> Array1<Complex64> {
       let norm = compute_norm(state);
       state.mapv(|x| x / norm)
   }
   ```

2. **Use Iterators**
   ```rust
   // Good: Lazy evaluation
   let sum: f64 = state.iter()
       .map(|c| c.norm_sqr())
       .sum();
   
   // Avoid: Intermediate collection
   let norms: Vec<f64> = state.iter()
       .map(|c| c.norm_sqr())
       .collect();
   let sum: f64 = norms.iter().sum();
   ```

3. **Parallel Processing**
   ```rust
   use rayon::prelude::*;
   
   // Use parallel iterators for large data
   if data.len() > 1000 {
       data.par_iter_mut().for_each(|x| process(x));
   } else {
       data.iter_mut().for_each(|x| process(x));
   }
   ```

### Documentation Contributions

Documentation is equally important! You can:

- Improve existing documentation
- Add tutorials and examples
- Fix spelling/grammar errors
- Translate documentation

**Documentation Locations**:
- API documentation: `///` comments in code
- User guides: `docs/` directory
- Examples: `examples/` directory

### Example Programs

When adding new examples:

1. Create file `examples/your_example.rs`
2. Add file header comment
   ```rust
   //! Your Example Title
   //! Author: Your Name
   //!
   //! Brief description of what this example demonstrates.
   ```
3. Include complete runnable code
4. Add comments explaining key steps
5. Add entry to README's example list

## Development Environment Setup

### Required Tools

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy

# Optional: CUDA support
# Download and install CUDA Toolkit 12.0+
```

### Building the Project

```bash
# Clone repository
git clone https://github.com/mindmemory-ai/myquat.git
cd myquat

# Build
cargo build

# Run tests
cargo test

# Build documentation
cargo doc --open
```

### Optional Features

```bash
# CUDA GPU acceleration
cargo build --features cuda
cargo test --features cuda

# All features
cargo build --all-features
```

## Pull Request Process

### PR Checklist

Before submitting a PR, ensure:

- [ ] Code passes `cargo test`
- [ ] Code passes `cargo clippy`
- [ ] Code is formatted with `cargo fmt`
- [ ] Appropriate tests are added
- [ ] Related documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Commit messages follow conventions
- [ ] No unrelated changes
- [ ] All merge conflicts are resolved

### PR Description Template

```markdown
## Change Type
- [ ] Bug fix
- [ ] New feature
- [ ] Refactoring
- [ ] Documentation update
- [ ] Performance optimization
- [ ] Test improvement

## Change Description
<!-- Describe your changes -->

## Related Issue
<!-- e.g., Closes #123 -->

## Testing
<!-- How to test these changes -->

## Screenshots (if applicable)
<!-- Add screenshots -->

## Additional Notes
<!-- Other things to note -->
```

### Review Process

1. **Local Checks**: Run tests and linting locally before submitting
2. **Code Review**: Maintainers will review your code
3. **Discussion**: There may be suggestions for improvements
4. **Merge**: After review approval, merge to main branch

## Release Process

(Maintainers only)

1. Update version number (`Cargo.toml`)
2. Update CHANGELOG.md
3. Create git tag
4. Push to GitHub
5. Publish to crates.io

```bash
# Update version
vim Cargo.toml  # Modify version field

# Update changelog
vim CHANGELOG.md

# Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v0.2.0"

# Create tag
git tag v0.2.0
git push origin main --tags

# Publish to crates.io
cargo publish
```

## Testing Guide

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        // Test code
    }
    
    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_case() {
        // Code that should panic
    }
}
```

### Integration Tests

Create test files in the `tests/` directory:

```rust
// tests/integration_test.rs
use myquat::*;

#[test]
fn test_end_to_end_workflow() {
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    
    let simulator = StateVectorSimulator::new();
    let result = simulator.run(&circuit, 1000).unwrap();
    
    assert!(result.counts().len() > 0);
}
```

### Benchmark Tests

Create benchmarks in the `benches/` directory:

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use myquat::*;

fn benchmark_hadamard(c: &mut Criterion) {
    c.bench_function("hadamard 20 qubits", |b| {
        let mut circuit = QuantumCircuit::new(20, 0);
        b.iter(|| {
            circuit.h(black_box(0)).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_hadamard);
criterion_main!(benches);
```

## Community

### Getting Help

- Create an Issue to ask questions
- Check the documentation
- Refer to example code

### Stay Connected

- GitHub Issues: Bug reports and discussions
- Pull Requests: Code contributions
- Email: logic.yan@me.com

## License

By contributing to MyQuat, you agree that your contributions will be licensed under the Apache 2.0 License.

## Acknowledgments

Thank you to all contributors! Your time and effort make MyQuat better.

---

**Thank you again for your contribution!** 🎉
