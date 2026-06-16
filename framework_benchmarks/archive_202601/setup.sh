#!/bin/bash
# Framework Benchmarks Setup Script
# Author: gA4ss
# 
# This script creates a Python virtual environment and installs
# quantum computing frameworks for performance comparison.

set -e

echo "=================================="
echo "Framework Benchmarks Setup"
echo "=================================="
echo ""

# Check Python version
PYTHON_CMD=$(command -v python3 || command -v python)
if [ -z "$PYTHON_CMD" ]; then
    echo "Error: Python not found. Please install Python 3.8+."
    exit 1
fi

PYTHON_VERSION=$($PYTHON_CMD --version | awk '{print $2}')
echo "Found Python: $PYTHON_VERSION"
echo ""

# Create virtual environment
VENV_DIR="venv"
if [ -d "$VENV_DIR" ]; then
    echo "Virtual environment already exists. Removing..."
    rm -rf "$VENV_DIR"
fi

echo "Creating virtual environment..."
$PYTHON_CMD -m venv "$VENV_DIR"

# Activate virtual environment
echo "Activating virtual environment..."
source "$VENV_DIR/bin/activate"

# Upgrade pip
echo ""
echo "Upgrading pip..."
pip install --upgrade pip setuptools wheel

# Install quantum frameworks
echo ""
echo "Installing Qiskit..."
pip install qiskit==1.0.0
echo "Note: Skipping qiskit-aer (requires C++ compilation, not needed for benchmarks)"

echo ""
echo "Installing Cirq..."
pip install cirq==1.3.0 cirq-core==1.3.0

echo ""
echo "Installing PennyLane..."
pip install pennylane==0.34.0

# Install additional dependencies
echo ""
echo "Installing additional dependencies..."
pip install numpy scipy matplotlib pandas tabulate

echo ""
echo "=================================="
echo "Setup Complete!"
echo "=================================="
echo ""
echo "To activate the environment, run:"
echo "  source venv/bin/activate"
echo ""
echo "To run benchmarks, execute:"
echo "  python run_benchmarks.py"
echo ""
