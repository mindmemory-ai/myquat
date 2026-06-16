#!/bin/bash
# Run all reverse Hamiltonian extraction experiments
#
# Author: gA4ss
#
# Usage:
#   ./experiments/reverse_hamiltonian_extraction/run_all.sh        # run all
#   ./experiments/reverse_hamiltonian_extraction/run_all.sh 01     # run specific exp
#   ./experiments/reverse_hamiltonian_extraction/run_all.sh 01 03  # run multiple

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="$SCRIPT_DIR/reports"

mkdir -p "$REPORT_DIR"

EXPERIMENTS=(
    "exp01_trotter_detection"
    "exp02_hamiltonian_extraction"
    "exp03_deoptimization_pipeline"
    "exp04_vqe_recognition"
    "exp05_qdrift_detection"
    "exp06_scalability"
    "exp07_noise_robustness"
)

run_experiment() {
    local exp_name=$1
    echo "============================================"
    echo "Running: $exp_name"
    echo "============================================"
    local start_time=$(date +%s)

    cd "$PROJECT_DIR"
    cargo run --release --bin "$exp_name" 2>&1

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    echo "Completed $exp_name in ${duration}s"
    echo ""
}

if [ $# -gt 0 ]; then
    # Run specific experiments by number
    for num in "$@"; do
        idx=$((10#$num - 1))
        if [ $idx -ge 0 ] && [ $idx -lt ${#EXPERIMENTS[@]} ]; then
            run_experiment "${EXPERIMENTS[$idx]}"
        else
            echo "Invalid experiment number: $num (valid: 01-07)"
        fi
    done
else
    # Run all experiments
    echo "Running all 7 experiments..."
    echo ""
    for exp in "${EXPERIMENTS[@]}"; do
        run_experiment "$exp"
    done
fi

echo "============================================"
echo "All reports saved to: $REPORT_DIR/"
echo "============================================"
ls -la "$REPORT_DIR"/*.json 2>/dev/null || echo "No reports found."
