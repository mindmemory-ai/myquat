#!/bin/bash
# Comprehensive Framework Comparison Runner
# Author: gA4ss
#
# Activates the virtual environment and runs the comparison benchmarks.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=================================="
echo "Framework Comparison Benchmark"
echo "=================================="
echo ""

# Check if virtual environment exists
VENV_DIR="venv"
if [ ! -d "$VENV_DIR" ]; then
    echo "Virtual environment not found. Running setup..."
    bash setup.sh
fi

# Activate virtual environment
echo "Activating virtual environment..."
source "$VENV_DIR/bin/activate"

# Verify frameworks are installed
echo ""
echo "Checking installed frameworks..."
python3 -c "import qiskit; print(f'  Qiskit: {qiskit.__version__}')" 2>/dev/null || echo "  Qiskit: NOT INSTALLED"
python3 -c "import cirq; print(f'  Cirq: {cirq.__version__}')" 2>/dev/null || echo "  Cirq: NOT INSTALLED"
python3 -c "import pennylane; print(f'  PennyLane: {pennylane.__version__}')" 2>/dev/null || echo "  PennyLane: NOT INSTALLED"

# Build MyQuat benchmark tool
echo ""
echo "Building MyQuat benchmark tool..."
cd "$SCRIPT_DIR/.."
cargo build --release --example benchmark_hamiltonian 2>/dev/null || {
    echo "Warning: Failed to build MyQuat benchmark tool"
}
cd "$SCRIPT_DIR"

# Run comprehensive comparison
echo ""
echo "Running comprehensive comparison..."
echo ""
python3 comprehensive_comparison.py

echo ""
echo "=================================="
echo "Benchmark Complete!"
echo "=================================="
