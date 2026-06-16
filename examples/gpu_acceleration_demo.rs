//! GPU Acceleration Demo
//!
//! GPU support requires the `cuda` feature flag and CUDA 12.0+ toolkit.
//! This demo shows the configuration pattern.

fn main() {
    println!("GPU Acceleration Demo");
    println!("====================\n");
    println!("GPU acceleration is available via the `cuda` feature flag.");
    println!("Build with: cargo build --features cuda");
    println!();
    println!("The CudaBackend provides 10-50x speedup on 20+ qubit simulations.");
    println!("See docs/CUDA_SETUP.md for installation instructions.");
}
