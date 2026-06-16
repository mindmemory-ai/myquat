#!/usr/bin/env python3
# generate_report.py
#
# Author: gA4ss
#
# Reads all exp01-exp07 JSON reports, generates summary tables and
# matplotlib figures, and outputs a comprehensive Markdown document.

import json
import os
import sys
import math
from pathlib import Path

try:
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    import matplotlib.ticker as ticker
    HAS_MPL = True
except ImportError:
    HAS_MPL = False
    print("[WARN] matplotlib not found; figures will be skipped.", file=sys.stderr)

REPORT_DIR = Path(__file__).parent / "reports"
FIG_DIR = REPORT_DIR / "figures"

def load_json(name):
    p = REPORT_DIR / name
    if not p.exists():
        print(f"[WARN] {p} not found, skipping.", file=sys.stderr)
        return None
    with open(p) as f:
        return json.load(f)

def fmt(v, digits=2):
    if v is None:
        return "N/A"
    if isinstance(v, float):
        return f"{v:.{digits}f}"
    return str(v)

def pct(v):
    return fmt(v, 1) + "%"

# =========================================================================
# Load all reports
# =========================================================================
d1 = load_json("exp01_trotter_detection.json")
d2 = load_json("exp02_hamiltonian_extraction.json")
d3 = load_json("exp03_deoptimization_pipeline.json")
d4 = load_json("exp04_vqe_recognition.json")
d5 = load_json("exp05_qdrift_detection.json")
d6 = load_json("exp06_scalability.json")
d7 = load_json("exp07_noise_robustness.json")

os.makedirs(FIG_DIR, exist_ok=True)

lines = []
def W(s=""):
    lines.append(s)

# =========================================================================
# Title
# =========================================================================
W("# Reverse Hamiltonian Extraction -- Experiment Report")
W()
ts = d1.get("timestamp", "") if d1 else ""
W(f"> Auto-generated from JSON reports.  Timestamp of first report: `{ts}`")
W()
W("---")
W()

# =========================================================================
# Overview table
# =========================================================================
W("## 1. Overview")
W()
W("| # | Experiment | Tests | Key Metric | Value |")
W("|---|-----------|------:|-----------|------:|")

if d1:
    s = d1["summary"]
    W(f"| 01 | Trotter Order Detection | {s['total_tests']} | Accuracy | {pct(s['overall_accuracy_pct'])} |")
if d2:
    s = d2["summary"]
    W(f"| 02 | Hamiltonian Extraction | {s['total_tests']} | Avg Coeff Error | {fmt(s['avg_relative_coeff_error'])} |")
if d3:
    s = d3["summary"]
    W(f"| 03 | Deoptimization Pipeline | {s['total_circuits']} | Avg Confidence | {fmt(s['avg_overall_confidence'])} |")
if d4:
    s = d4["summary"]
    W(f"| 04 | VQE Recognition | {s['total_tests']} | Positive Acc. | {pct(s['positive_accuracy_pct'])} |")
if d5:
    s = d5["summary"]
    W(f"| 05 | qDRIFT Detection | {s['total_tests']} | Detection Rate | {pct(s['positive_accuracy_pct'])} |")
if d6:
    s = d6["summary"]
    W(f"| 06 | Scalability | {s['gate_scaling_points']+s['qubit_scaling_points']+s['pipeline_scaling_points']} | Avg Throughput | {fmt(s['avg_extraction_throughput_gates_per_sec']/1e6,1)} M gates/s |")
if d7:
    s = d7["summary"]
    W(f"| 07 | Noise Robustness | {s['total_data_points']} | Data Points | {s['total_data_points']} |")
W()

# =========================================================================
# Exp01
# =========================================================================
W("---")
W()
W("## 2. Experiment 01 -- Trotter Order Detection")
W()
if d1:
    s = d1["summary"]
    W(f"- **Total tests**: {s['total_tests']}")
    W(f"- **Overall accuracy**: {pct(s['overall_accuracy_pct'])}")
    W(f"- **Runtime**: {s['total_time_ms']} ms")
    W()

    # Per-order table
    W("### 2.1 Accuracy by Trotter Order")
    W()
    W("| Order | Tests | Correct | Accuracy | Avg Confidence | Avg Time (us) |")
    W("|-------|------:|--------:|---------:|---------------:|--------------:|")
    for item in d1.get("per_order_summary", []):
        W(f"| {item['order']} | {item['total']} | {item['correct']} | {pct(item['accuracy_pct'])} | {fmt(item['avg_confidence'])} | {fmt(item['avg_analysis_time_us'])} |")
    W()

    # Per-hamiltonian table
    W("### 2.2 Accuracy by Hamiltonian Type")
    W()
    W("| Hamiltonian | Tests | Correct | Accuracy |")
    W("|------------|------:|--------:|---------:|")
    for item in d1.get("per_hamiltonian_summary", []):
        W(f"| {item['hamiltonian']} | {item['total']} | {item['correct']} | {pct(item['accuracy_pct'])} |")
    W()

    # Figure: accuracy by order
    if HAS_MPL:
        orders = [item["order"] for item in d1["per_order_summary"]]
        accs = [item["accuracy_pct"] for item in d1["per_order_summary"]]
        fig, ax = plt.subplots(figsize=(7, 4))
        ax.bar(orders, accs, color="#4C72B0")
        ax.set_ylabel("Accuracy (%)")
        ax.set_xlabel("Trotter Order")
        ax.set_title("Trotter Order Detection Accuracy")
        ax.set_ylim(0, 105)
        fig.tight_layout()
        fig.savefig(FIG_DIR / "exp01_order_accuracy.png", dpi=150)
        plt.close(fig)
        W("![Trotter Order Detection Accuracy](figures/exp01_order_accuracy.png)")
        W()

# =========================================================================
# Exp02
# =========================================================================
W("---")
W()
W("## 3. Experiment 02 -- Hamiltonian Coefficient Extraction")
W()
if d2:
    s = d2["summary"]
    W(f"- **Total tests**: {s['total_tests']}")
    W(f"- **Avg term match rate**: {pct(s['avg_term_match_rate']*100)}")
    W(f"- **Avg relative coefficient error**: {fmt(s['avg_relative_coeff_error'])}")
    W()

    W("### 3.1 Per Hamiltonian Type")
    W()
    W("| Type | Count | Avg Match Rate | Avg Coeff Error |")
    W("|------|------:|---------------:|----------------:|")
    for item in d2.get("per_type_summary", []):
        W(f"| {item['type']} | {item['count']} | {pct(item['avg_term_match_rate']*100)} | {fmt(item['avg_coeff_error'])} |")
    W()

    # Figure: per-type bar
    if HAS_MPL:
        types = [item["type"] for item in d2["per_type_summary"]]
        errs = [item["avg_coeff_error"] for item in d2["per_type_summary"]]
        fig, ax = plt.subplots(figsize=(6, 4))
        ax.bar(types, errs, color="#DD8452")
        ax.set_ylabel("Avg Relative Coefficient Error")
        ax.set_xlabel("Hamiltonian Type")
        ax.set_title("Coefficient Extraction Error by Type")
        fig.tight_layout()
        fig.savefig(FIG_DIR / "exp02_coeff_error.png", dpi=150)
        plt.close(fig)
        W("![Coefficient Extraction Error](figures/exp02_coeff_error.png)")
        W()

# =========================================================================
# Exp03
# =========================================================================
W("---")
W()
W("## 4. Experiment 03 -- Deoptimization Pipeline")
W()
if d3:
    s = d3["summary"]
    W(f"- **Total circuits**: {s['total_circuits']}")
    W(f"- **Successful tests**: {s['successful_pipeline_tests']}")
    W(f"- **Avg overall confidence**: {fmt(s['avg_overall_confidence'])}")
    W()

    W("### 4.1 Per-Strategy Summary")
    W()
    W("| Strategy | Avg Confidence | Avg Apply Time (us) |")
    W("|----------|---------------:|--------------------:|")
    for item in d3.get("per_strategy_summary", []):
        W(f"| {item['strategy']} | {fmt(item['avg_confidence'])} | {fmt(item['avg_apply_time_us'])} |")
    W()

    W("### 4.2 Per-Circuit-Type Summary")
    W()
    W("| Circuit Type | Count | Avg Confidence | Avg Restore Time (us) | Early Stop % |")
    W("|-------------|------:|---------------:|----------------------:|-------------:|")
    for item in d3.get("per_circuit_type_summary", []):
        W(f"| {item['circuit_type']} | {item['count']} | {fmt(item['avg_confidence'])} | {fmt(item['avg_restore_time_us'])} | {pct(item['early_stop_pct'])} |")
    W()

    # Figure: per-circuit-type confidence
    if HAS_MPL:
        ctypes = [item["circuit_type"] for item in d3["per_circuit_type_summary"]]
        confs = [item["avg_confidence"] for item in d3["per_circuit_type_summary"]]
        fig, ax = plt.subplots(figsize=(8, 4))
        bars = ax.barh(ctypes, confs, color="#55A868")
        ax.set_xlabel("Avg Confidence")
        ax.set_title("Pipeline Confidence by Circuit Type")
        ax.set_xlim(0, 1.05)
        for bar, v in zip(bars, confs):
            ax.text(v + 0.02, bar.get_y() + bar.get_height()/2,
                    f"{v:.2f}", va='center', fontsize=9)
        fig.tight_layout()
        fig.savefig(FIG_DIR / "exp03_circuit_type_conf.png", dpi=150)
        plt.close(fig)
        W("![Pipeline Confidence by Circuit Type](figures/exp03_circuit_type_conf.png)")
        W()

    # Figure: per-strategy confidence heatmap-like grouped bar
    if HAS_MPL:
        strats = d3.get("per_strategy_summary", [])
        if strats:
            fig, ax = plt.subplots(figsize=(9, 5))
            snames = [s["strategy"] for s in strats]
            all_ctypes = set()
            for s in strats:
                for c in s.get("confidence_by_circuit_type", []):
                    all_ctypes.add(c["circuit_type"])
            all_ctypes = sorted(all_ctypes)
            import numpy as np
            x = np.arange(len(all_ctypes))
            width = 0.15
            for i, s in enumerate(strats):
                cmap = {c["circuit_type"]: c["avg_confidence"] for c in s.get("confidence_by_circuit_type", [])}
                vals = [cmap.get(ct, 0) for ct in all_ctypes]
                ax.bar(x + i*width, vals, width, label=s["strategy"])
            ax.set_xticks(x + width * (len(strats)-1)/2)
            ax.set_xticklabels(all_ctypes, rotation=30, ha="right", fontsize=8)
            ax.set_ylabel("Avg Confidence")
            ax.set_title("Strategy Confidence by Circuit Type")
            ax.legend(fontsize=8)
            ax.set_ylim(0, 1.15)
            fig.tight_layout()
            fig.savefig(FIG_DIR / "exp03_strategy_heatmap.png", dpi=150)
            plt.close(fig)
            W("![Strategy Confidence by Circuit Type](figures/exp03_strategy_heatmap.png)")
            W()

# =========================================================================
# Exp04
# =========================================================================
W("---")
W()
W("## 5. Experiment 04 -- VQE Ansatz Recognition")
W()
if d4:
    s = d4["summary"]
    W(f"- **Total tests**: {s['total_tests']}  (positive: {s['positive_tests']}, negative: {s['negative_tests']}, UCCSD: {s['uccsd_tests']})")
    W(f"- **Positive accuracy**: {pct(s['positive_accuracy_pct'])}")
    W(f"- **False positive rate**: {pct(s['false_positive_rate_pct'])}")
    W()

    W("### 5.1 Per-Ansatz Summary")
    W()
    W("| Ansatz Type | Tests | Correct | Accuracy | Avg Confidence |")
    W("|------------|------:|--------:|---------:|---------------:|")
    for item in d4.get("per_ansatz_summary", []):
        W(f"| {item['ansatz_type']} | {item['total']} | {item['correct']} | {pct(item['accuracy_pct'])} | {fmt(item['avg_confidence'])} |")
    W()

    W("### 5.2 Per-Qubit Summary (Positive Tests)")
    W()
    W("| Qubits | Tests | Correct | Accuracy |")
    W("|-------:|------:|--------:|---------:|")
    for item in d4.get("per_qubit_summary", []):
        W(f"| {item['num_qubits']} | {item['total']} | {item['correct']} | {pct(item['accuracy_pct'])} |")
    W()

    if HAS_MPL:
        ansatz = [item["ansatz_type"] for item in d4["per_ansatz_summary"]]
        accs = [item["accuracy_pct"] for item in d4["per_ansatz_summary"]]
        confs = [item["avg_confidence"] for item in d4["per_ansatz_summary"]]
        fig, ax1 = plt.subplots(figsize=(6, 4))
        x_pos = range(len(ansatz))
        bars = ax1.bar(x_pos, accs, color="#4C72B0", alpha=0.8, label="Accuracy (%)")
        ax1.set_ylabel("Accuracy (%)")
        ax1.set_ylim(0, 115)
        ax2 = ax1.twinx()
        ax2.plot(x_pos, confs, 'ro-', label="Avg Confidence")
        ax2.set_ylabel("Avg Confidence")
        ax2.set_ylim(0, 1.15)
        ax1.set_xticks(x_pos)
        ax1.set_xticklabels(ansatz)
        ax1.set_title("VQE Ansatz Recognition")
        ax1.legend(loc="upper left")
        ax2.legend(loc="upper right")
        fig.tight_layout()
        fig.savefig(FIG_DIR / "exp04_ansatz_accuracy.png", dpi=150)
        plt.close(fig)
        W("![VQE Ansatz Recognition](figures/exp04_ansatz_accuracy.png)")
        W()

# =========================================================================
# Exp05
# =========================================================================
W("---")
W()
W("## 6. Experiment 05 -- qDRIFT Detection")
W()
if d5:
    s = d5["summary"]
    W(f"- **Total tests**: {s['total_tests']}  (positive: {s['positive_tests']}, negative: {s['negative_tests']})")
    W(f"- **Detection rate**: {pct(s['positive_accuracy_pct'])}")
    W(f"- **False positive rate**: {pct(s['false_positive_rate_pct'])}")
    W()

    zz = d5.get("zz_comparison", {})
    W("### 6.1 ZZ Gadget Comparison")
    W()
    W("| Variant | Count | Avg Confidence |")
    W("|---------|------:|---------------:|")
    W(f"| Without ZZ | {zz.get('without_zz_count','N/A')} | {fmt(zz.get('without_zz_avg_confidence'))} |")
    W(f"| With ZZ | {zz.get('with_zz_count','N/A')} | {fmt(zz.get('with_zz_avg_confidence'))} |")
    W()

    W("### 6.2 Detection by Qubit Count")
    W()
    W("| Qubits | Tests | Correct | Accuracy | Avg Confidence |")
    W("|-------:|------:|--------:|---------:|---------------:|")
    for item in d5.get("per_qubit_summary", []):
        W(f"| {item['num_qubits']} | {item['total']} | {item['correct']} | {pct(item['accuracy_pct'])} | {fmt(item['avg_confidence'])} |")
    W()

    W("### 6.3 Detection by Sample Count")
    W()
    W("| Samples | Tests | Correct | Accuracy | Avg Confidence |")
    W("|--------:|------:|--------:|---------:|---------------:|")
    for item in d5.get("per_sample_summary", []):
        W(f"| {item['num_samples']} | {item['total']} | {item['correct']} | {pct(item['accuracy_pct'])} | {fmt(item['avg_confidence'])} |")
    W()

    if HAS_MPL:
        pqs = d5.get("per_qubit_summary", [])
        if pqs:
            fig, ax = plt.subplots(figsize=(6, 4))
            qs = [item["num_qubits"] for item in pqs]
            confs = [item["avg_confidence"] for item in pqs]
            ax.plot(qs, confs, 's-', color="#C44E52")
            ax.set_xlabel("Qubits")
            ax.set_ylabel("Avg Confidence")
            ax.set_title("qDRIFT Detection Confidence vs Qubit Count")
            ax.set_ylim(0, 1.1)
            fig.tight_layout()
            fig.savefig(FIG_DIR / "exp05_qubit_confidence.png", dpi=150)
            plt.close(fig)
            W("![qDRIFT Detection vs Qubits](figures/exp05_qubit_confidence.png)")
            W()

# =========================================================================
# Exp06
# =========================================================================
W("---")
W()
W("## 7. Experiment 06 -- Scalability and Performance")
W()
if d6:
    s = d6["summary"]
    W(f"- **Avg extraction throughput**: {fmt(s['avg_extraction_throughput_gates_per_sec']/1e6, 1)} M gates/sec")
    W()

    W("### 7.1 Gate Count Scaling (4 qubits, Ising 1st-order)")
    W()
    W("| Trotter Steps | Gates | Avg Time (us) | Throughput (M gates/s) | us/gate |")
    W("|--------------:|------:|--------------:|-----------------------:|--------:|")
    for item in d6.get("gate_count_scaling", []):
        tp = item["throughput_gates_per_sec"] / 1e6
        W(f"| {item['trotter_steps']} | {item['gate_count']} | {fmt(item['avg_time_us'])} | {fmt(tp, 1)} | {fmt(item['us_per_gate'], 3)} |")
    W()

    W("### 7.2 Qubit Count Scaling (2 Trotter steps)")
    W()
    W("| Qubits | Gates | Avg Time (us) | Throughput (M gates/s) |")
    W("|-------:|------:|--------------:|-----------------------:|")
    for item in d6.get("qubit_count_scaling", []):
        tp = item["throughput_gates_per_sec"] / 1e6
        W(f"| {item['num_qubits']} | {item['gate_count']} | {fmt(item['avg_time_us'])} | {fmt(tp, 1)} |")
    W()

    # Figures
    if HAS_MPL:
        gs = d6.get("gate_count_scaling", [])
        if gs:
            fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(11, 4.5))
            gates = [item["gate_count"] for item in gs]
            times = [item["avg_time_us"] for item in gs]
            ax1.plot(gates, times, 'o-', color="#4C72B0")
            ax1.set_xlabel("Gate Count")
            ax1.set_ylabel("Avg Extraction Time (us)")
            ax1.set_title("Extraction Time vs Gate Count")

            qs_data = d6.get("qubit_count_scaling", [])
            qubits = [item["num_qubits"] for item in qs_data]
            q_times = [item["avg_time_us"] for item in qs_data]
            ax2.plot(qubits, q_times, 's-', color="#DD8452")
            ax2.set_xlabel("Qubits")
            ax2.set_ylabel("Avg Extraction Time (us)")
            ax2.set_title("Extraction Time vs Qubit Count")
            fig.tight_layout()
            fig.savefig(FIG_DIR / "exp06_scaling.png", dpi=150)
            plt.close(fig)
            W("![Scalability](figures/exp06_scaling.png)")
            W()

    # Pipeline throughput by circuit type
    pt = d6.get("pipeline_throughput", [])
    if pt:
        W("### 7.3 Pipeline Throughput by Circuit Type")
        W()
        # Group by circuit_type
        by_type = {}
        for item in pt:
            ct = item["circuit_type"]
            by_type.setdefault(ct, []).append(item)
        W("| Circuit Type | Avg Gates | Avg Time (us) | Avg Throughput (M gates/s) |")
        W("|-------------|----------:|--------------:|---------------------------:|")
        for ct, items in sorted(by_type.items()):
            avg_g = sum(i["gate_count"] for i in items) / len(items)
            avg_t = sum(i["avg_time_us"] for i in items) / len(items)
            avg_tp = sum(i["throughput_gates_per_sec"] for i in items) / len(items) / 1e6
            W(f"| {ct} | {fmt(avg_g, 0)} | {fmt(avg_t)} | {fmt(avg_tp, 1)} |")
        W()

# =========================================================================
# Exp07
# =========================================================================
W("---")
W()
W("## 8. Experiment 07 -- Noise Robustness")
W()
if d7:
    s = d7["summary"]
    W(f"- **Total data points**: {s['total_data_points']}")
    W(f"- **Runtime**: {s['total_time_ms']} ms")
    W()

    W("### 8.1 Metrics vs Noise Level (Averaged)")
    W()
    W("| Noise sigma | Avg Coeff Error | Order Detection Acc. | Deopt Confidence |")
    W("|------------:|----------------:|---------------------:|-----------------:|")
    for item in d7.get("noise_level_summary", []):
        W(f"| {fmt(item['noise_sigma'], 3)} | {fmt(item['avg_coeff_error'])} | {pct(item['avg_order_detection_accuracy']*100)} | {fmt(item['avg_deopt_confidence'])} |")
    W()

    W("### 8.2 Clean vs Noisy (sigma=0.1) Comparison")
    W()
    W("| Case | Clean Coeff Err | Noisy Coeff Err | Clean Order Acc | Noisy Order Acc | Clean Deopt Conf | Noisy Deopt Conf |")
    W("|------|----------------:|----------------:|----------------:|----------------:|-----------------:|-----------------:|")
    for item in d7.get("clean_vs_noisy_comparison", []):
        W(f"| {item['case']} | {fmt(item['clean_coeff_error'])} | {fmt(item['noisy_01_coeff_error'])} | {fmt(item['clean_order_accuracy'])} | {fmt(item['noisy_01_order_accuracy'])} | {fmt(item['clean_deopt_confidence'])} | {fmt(item['noisy_01_deopt_confidence'])} |")
    W()

    if HAS_MPL:
        ns = d7.get("noise_level_summary", [])
        if ns:
            sigmas = [item["noise_sigma"] for item in ns]
            oda = [item["avg_order_detection_accuracy"] * 100 for item in ns]
            dc = [item["avg_deopt_confidence"] for item in ns]

            fig, ax1 = plt.subplots(figsize=(7, 4.5))
            ax1.plot(sigmas, oda, 'o-', color="#4C72B0", label="Order Detection Acc. (%)")
            ax1.set_xlabel("Noise sigma")
            ax1.set_ylabel("Order Detection Accuracy (%)", color="#4C72B0")
            ax1.set_ylim(-5, 105)
            ax2 = ax1.twinx()
            ax2.plot(sigmas, dc, 's-', color="#C44E52", label="Deopt Confidence")
            ax2.set_ylabel("Deopt Confidence", color="#C44E52")
            ax2.set_ylim(0, 1.1)
            ax1.set_title("Extraction Quality vs Noise Level")
            ax1.legend(loc="upper left", fontsize=9)
            ax2.legend(loc="upper right", fontsize=9)
            fig.tight_layout()
            fig.savefig(FIG_DIR / "exp07_noise.png", dpi=150)
            plt.close(fig)
            W("![Noise Robustness](figures/exp07_noise.png)")
            W()

# =========================================================================
# Conclusion
# =========================================================================
W("---")
W()
W("## 9. Summary")
W()
W("| Experiment | Status | Key Finding |")
W("|-----------|--------|------------|")
if d1:
    s = d1["summary"]
    W(f"| 01 Trotter Detection | {s['total_tests']} tests | 1st/2nd order detectable; higher orders need improvement |")
if d2:
    s = d2["summary"]
    W(f"| 02 Hamiltonian Extraction | {s['total_tests']} tests | Coefficient extraction requires deeper analysis |")
if d3:
    s = d3["summary"]
    W(f"| 03 Deoptimization Pipeline | {s['total_circuits']} circuits | KAK best for Hamiltonian/QAOA; VQE/qDRIFT strategies effective |")
if d4:
    s = d4["summary"]
    W(f"| 04 VQE Recognition | {s['positive_tests']} positive | 100% true-positive rate; false-positive filtering needed |")
if d5:
    s = d5["summary"]
    W(f"| 05 qDRIFT Detection | {s['positive_tests']} positive | 100% detection; specificity improvement needed |")
if d6:
    s = d6["summary"]
    tp = s["avg_extraction_throughput_gates_per_sec"] / 1e6
    W(f"| 06 Scalability | {s['gate_scaling_points']+s['qubit_scaling_points']} points | ~{fmt(tp,1)} M gates/sec; linear time scaling |")
if d7:
    s = d7["summary"]
    W(f"| 07 Noise Robustness | {s['total_data_points']} points | Order detection degrades rapidly above sigma > 0.01 |")
W()

# =========================================================================
# Write output
# =========================================================================
output_path = REPORT_DIR / "EXPERIMENT_REPORT.md"
with open(output_path, "w") as f:
    f.write("\n".join(lines) + "\n")
print(f"Report written to {output_path}")
if HAS_MPL:
    figs = list(FIG_DIR.glob("*.png"))
    print(f"Figures generated: {len(figs)}")
    for fig in sorted(figs):
        print(f"  - {fig.name}")
