#!/bin/bash
# Test script for teaching demos
# Author: gA4ss
# Date: 2026-02-12

echo "================================"
echo "MyQuat Teaching Demos Test Suite"
echo "================================"
echo ""

EXAMPLES=(
    "demo_01_bell_state"
    "demo_02_beginner_tutorial"
    "demo_03_comprehensive_gates"
    "demo_04_visualization"
    "demo_05_hamiltonian_forward"
    "demo_06_hamiltonian_backward"
    "demo_07_grover_algorithm"
    "demo_08_vqe_chemistry"
    "demo_09_interactive_tutorial"
    "demo_10_adaptive_optimization"
)

SUCCESS=0
FAILED=0
FAILED_EXAMPLES=()

for example in "${EXAMPLES[@]}"; do
    echo "Testing: $example"
    if cargo run --example "$example" > /dev/null 2>&1; then
        echo "  ✓ PASSED"
        ((SUCCESS++))
    else
        echo "  ✗ FAILED"
        ((FAILED++))
        FAILED_EXAMPLES+=("$example")
    fi
    echo ""
done

echo "================================"
echo "Test Summary"
echo "================================"
echo "Total:   ${#EXAMPLES[@]}"
echo "Passed:  $SUCCESS"
echo "Failed:  $FAILED"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo "Failed examples:"
    for failed in "${FAILED_EXAMPLES[@]}"; do
        echo "  - $failed"
    done
    exit 1
else
    echo ""
    echo "All tests passed!"
    exit 0
fi
