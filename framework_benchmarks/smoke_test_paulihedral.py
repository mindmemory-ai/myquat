#!/usr/bin/env python3
"""Smoke test for Paulihedral after import fixes.

Usage:
    conda activate quantum
    python smoke_test_paulihedral.py
"""

import sys

PASSED = 0
FAILED = 0


def check(name, condition, detail=""):
    global PASSED, FAILED
    if condition:
        print(f"  PASS: {name}")
        PASSED += 1
    else:
        print(f"  FAIL: {name}  {detail}")
        FAILED += 1


def main():
    global PASSED, FAILED
    print("=" * 60)
    print("Paulihedral Smoke Test")
    print("=" * 60)

    # Test 1: basic import
    print("\n1. Basic import")
    try:
        import paulihedral
        check("import paulihedral", True)
    except Exception as e:
        check("import paulihedral", False, str(e))

    # Test 2: synthesis_FT import
    print("\n2. synthesis_FT import")
    try:
        from paulihedral.synthesis_FT import block_opt_FT, pauliString
        check("from paulihedral.synthesis_FT import block_opt_FT, pauliString", True)
    except Exception as e:
        check("from paulihedral.synthesis_FT import ...", False, str(e))
        print(f"\n  Cannot continue — synthesis_FT import failed")
        print(f"\nResult: {PASSED} passed, {FAILED} failed")
        return FAILED == 0

    # Test 3: synthesis_SC import
    print("\n3. synthesis_SC import")
    try:
        from paulihedral.synthesis_SC import block_opt_SC
        check("from paulihedral.synthesis_SC import block_opt_SC", True)
    except Exception as e:
        check("from paulihedral.synthesis_SC import block_opt_SC", False, str(e))

    # Test 4: synthesis_sd import
    print("\n4. synthesis_sd import")
    try:
        from paulihedral.synthesis_sd import simple_dfs_path
        check("from paulihedral.synthesis_sd import simple_dfs_path", True)
    except Exception as e:
        check("from paulihedral.synthesis_sd import simple_dfs_path", False, str(e))

    # Test 5: qubit_place import
    print("\n5. qubit_place import")
    try:
        from paulihedral.qubit_place import qaim_place
        check("from paulihedral.qubit_place import qaim_place", True)
    except Exception as e:
        check("from paulihedral.qubit_place import qaim_place", False, str(e))

    # Test 6: Trivial synthesis (2-qubit, single term)
    print("\n6. Trivial synthesis (2q, 1 term)")
    try:
        ps = pauliString("XX", coeff=0.5)
        ps_layers = [[[ps]]]
        qc = block_opt_FT(ps_layers, time_parameter=1.0)
        n_gates = qc.size()
        check("2-qubit single-term synthesis", n_gates > 0,
              f"({n_gates} gates, depth {qc.depth()})")
    except Exception as e:
        check("2-qubit single-term synthesis", False, str(e))

    # Test 7: Multi-term synthesis (2-qubit, 3 terms)
    print("\n7. Multi-term synthesis (2q, 3 terms)")
    try:
        terms = [
            pauliString("XX", coeff=0.5),
            pauliString("YY", coeff=0.3),
            pauliString("ZZ", coeff=-0.2),
        ]
        ps_layers = [[terms]]  # all in one layer/group
        qc = block_opt_FT(ps_layers, time_parameter=1.0)
        n_gates = qc.size()
        check("2-qubit multi-term synthesis", n_gates > 0,
              f"({n_gates} gates, depth {qc.depth()})")
    except Exception as e:
        check("2-qubit multi-term synthesis", False, str(e))

    # Test 8: 4-qubit synthesis
    print("\n8. 4-qubit synthesis")
    try:
        terms = [
            pauliString("XXII", coeff=0.5),
            pauliString("IIXX", coeff=0.3),
            pauliString("ZIZI", coeff=-0.2),
        ]
        ps_layers = [[terms]]
        qc = block_opt_FT(ps_layers, time_parameter=1.0)
        n_gates = qc.size()
        check("4-qubit multi-term synthesis", n_gates > 0,
              f"({n_gates} gates, depth {qc.depth()})")
    except Exception as e:
        check("4-qubit multi-term synthesis", False, str(e))

    # Summary
    print(f"\n{'=' * 60}")
    print(f"Result: {PASSED} passed, {FAILED} failed")
    print(f"{'=' * 60}")

    return FAILED == 0


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
