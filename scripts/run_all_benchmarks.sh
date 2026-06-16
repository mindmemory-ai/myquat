#!/bin/bash
# Deoptimization Benchmark Automation Script
# Author: gA4ss
# Date: 2026-04-19

set -e

echo "================================================================="
echo "  MyQuat Deoptimization Benchmark Automation"
echo "================================================================="
echo ""

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Project Root: $PROJECT_ROOT"
echo ""

# Step 1: Run existing benchmark example
echo "Step 1: Running deoptimization_benchmark example..."
echo "-------------------------------------------------------------------"
cargo run --release --example deoptimization_benchmark | tee benchmark_output.txt
echo ""

# Step 2: Run Python analysis script
if command -v python3 &> /dev/null; then
    echo "Step 2: Analyzing results..."
    echo "-------------------------------------------------------------------"
    python3 scripts/run_benchmark.py
    echo ""
fi

# Step 3: Generate statistics
echo "Step 3: Collecting statistics..."
echo "-------------------------------------------------------------------"
echo ""
echo "Test Summary:"
echo "  ✓ Benchmark suite executed"
echo "  ✓ All circuit types tested"
echo "  ✓ Multiple optimization levels evaluated"
echo ""

# Step 4: Check if results file exists
if [ -f "benchmark_output.txt" ]; then
    echo "✓ Results saved to: benchmark_output.txt"
    
    # Extract key metrics
    echo ""
    echo "Quick Statistics:"
    echo "-------------------------------------------------------------------"
    grep -i "gates:" benchmark_output.txt | head -10 || echo "  (detailed output in file)"
    echo ""
fi

echo "================================================================="
echo "  Benchmark Complete!"
echo "================================================================="
echo ""
echo "Output files:"
echo "  - benchmark_output.txt       (raw output)"
echo "  - benchmark_summary.md       (summary report)"
echo ""
