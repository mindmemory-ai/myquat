# MyQuat 示例程序索引

**版本**: v1.0  
**最后更新**: 2026-05-16

本文档提供了 MyQuat 库所有示例程序的分类索引，帮助您快速找到所需的示例。

---

## 快速导航

- [教学示例](#教学示例) - 适合初学者
- [核心算法](#核心算法) - 经典量子算法
- [功能演示](#功能演示) - 库功能展示
- [哈密顿量](#哈密顿量) - 量子化学和模拟
- [量子化学](#量子化学) - 分子模拟
- [后端集成](#后端集成) - 云服务和硬件
- [高级特性](#高级特性) - 性能优化
- [量子力学](#量子力学) - 物理教学
- [基础示例](#基础示例) - 简单入门
- [教学演示](#教学演示) - 系统化教程

---

## 教学示例

适合量子计算初学者的入门教程。

| 文件 | 描述 | 难度 |
|------|------|------|
| `beginner_tutorial.rs` | 初学者完整教程，从量子比特到简单算法 | ⭐ |
| `interactive_tutorial.rs` | 交互式教程，涵盖所有主要功能 | ⭐⭐ |
| `easy_api_demo.rs` | 简易 API 演示，快速上手 | ⭐ |

**推荐学习顺序**: beginner_tutorial → interactive_tutorial → easy_api_demo

---

## 核心算法

经典量子算法的完整实现。

### Shor 算法
| 文件 | 描述 | 关键概念 |
|------|------|---------|
| `shor_algorithm.rs` | 整数分解的量子算法 | 量子傅里叶变换、周期查找、RSA 威胁 |

### Grover 算法
| 文件 | 描述 | 关键概念 |
|------|------|---------|
| `grover_enhanced.rs` | 增强版 Grover 搜索算法 | 振幅放大、Oracle 设计、多目标搜索 |

### 变分量子算法
| 文件 | 描述 | 关键概念 |
|------|------|---------|
| `vqe.rs` | 变分量子本征求解器 | UCCSD ansatz、能量优化 |
| `vqe_h2.rs` | VQE 求解 H2 分子基态能量 | 量子化学、分子哈密顿量 |
| `qaoa_maxcut.rs` | QAOA 求解最大割问题 | 组合优化、混合算法 |

### 量子傅里叶变换
| 文件 | 描述 | 关键概念 |
|------|------|---------|
| `qft.rs` | 量子傅里叶变换及其应用 | 相位估计、周期查找 |
| `phase_estimation_demo.rs` | 量子相位估计算法 | 本征值估计、精度控制 |

### 量子机器学习
| 文件 | 描述 | 关键概念 |
|------|------|---------|
| `quantum_machine_learning.rs` | 量子机器学习算法套件 | 特征映射、变分分类器、量子核 |
| `quantum_algorithms_demo.rs` | 多种量子算法综合演示 | 算法对比、性能分析 |

---

## 功能演示

MyQuat 库核心功能的演示程序。

### 可视化
| 文件 | 描述 | 功能 |
|------|------|------|
| `visualization_demo.rs` | 电路可视化完整演示 | ASCII 艺术、SVG 导出、统计分析 |

### 错误处理
| 文件 | 描述 | 功能 |
|------|------|------|
| `error_handling_demo.rs` | 错误处理系统演示 | 错误分类、恢复策略、统计报告 |
| `error_mitigation_demo.rs` | 量子错误缓解技术 | ZNE、对称性验证、Richardson 外推 |
| `noise_modeling_demo.rs` | NISQ 设备噪声建模 | 退极化、退相干、测量误差 |

### 设备和拓扑
| 文件 | 描述 | 功能 |
|------|------|------|
| `device_topology_demo.rs` | 量子设备拓扑约束 | 连接图、SWAP 插入、路由优化 |

### 内存和性能
| 文件 | 描述 | 功能 |
|------|------|------|
| `memory_pool_demo.rs` | 内存池管理系统 | 对象池、内存复用 |
| `memory_optimization_demo.rs` | 内存优化技术 | 稀疏表示、压缩存储 |
| `performance_optimization.rs` | 性能优化综合演示 | SIMD、并行化、缓存优化 |

### 电路优化
| 文件 | 描述 | 功能 |
|------|------|------|
| `optimization_layers_demo.rs` | 多层优化系统 | 门消除、门融合、深度优化 |
| `adaptive_optimization_demo.rs` | 自适应优化策略 | 动态优化、性能监控 |

### 扩展功能
| 文件 | 描述 | 功能 |
|------|------|------|
| `extended_gates_demo.rs` | 扩展量子门演示 | 自定义门、复合门 |
| `extended_qasm_demo.rs` | 扩展 QASM 支持 | QASM 3.0、自定义指令 |

---

## 哈密顿量

量子哈密顿量的构建、优化和模拟。

### 基础功能
| 文件 | 描述 | 应用 |
|------|------|------|
| `hamiltonian_demo.rs` | 哈密顿量基础操作 | Pauli 字符串、系数管理 |
| `hamiltonian_forward_demo.rs` | 哈密顿量正向编译 | Trotter 分解、电路生成 |
| `hamiltonian_backward_demo.rs` | 电路反向提取哈密顿量 | 电路分析、模式识别 |

### 优化技术
| 文件 | 描述 | 技术 |
|------|------|------|
| `hamiltonian_optimization_demo.rs` | 哈密顿量优化技术 | Pauli 分组、QWC 优化 |
| `hamiltonian_optimizer_demo.rs` | 哈密顿量优化器 | 自动优化、性能分析 |

### Trotter 分解
| 文件 | 描述 | 方法 |
|------|------|------|
| `adaptive_trotter_demo.rs` | 自适应 Trotter 分解 | 误差控制、步长优化 |
| `higher_order_trotter_demo.rs` | 高阶 Trotter 方法 | Suzuki 公式、精度提升 |
| `trotter_templates.rs` | Trotter 模板库 | 预定义模板、快速构建 |

---

## 量子化学

分子模拟和量子化学应用。

| 文件 | 描述 | 分子系统 |
|------|------|---------|
| `chemistry_demo.rs` | 量子化学基础演示 | 通用化学系统 |
| `quantum_chemistry_demo.rs` | 量子化学完整流程 | 费米子变换、VQE 求解 |
| `h2_full_comparison.rs` | H2 分子完整对比 | 不同方法对比、精度分析 |

---

## 后端集成

云服务和硬件后端集成。

| 文件 | 描述 | 后端 |
|------|------|------|
| `backend_integration_demo.rs` | 后端集成框架 | 通用后端接口 |
| `cloud_backend_demo.rs` | 云量子计算后端 | IBM Quantum、AWS Braket |
| `cloud_config_demo.rs` | 云服务配置管理 | 认证、配置、连接 |
| `unified_cloud_demo.rs` | 统一云服务接口 | 多云支持、自动切换 |

---

## 高级特性

高级功能和性能优化。

### 电路反优化
| 文件 | 描述 | 技术 |
|------|------|------|
| `deoptimization_demo.rs` | 电路反优化技术 | 模式识别、电路还原 |

### 硬件加速
| 文件 | 描述 | 技术 |
|------|------|------|
| `gpu_acceleration_demo.rs` | GPU 加速演示 | CUDA、OpenCL |
| `cuda_demo.rs` | CUDA 专用演示 | NVIDIA GPU 优化 |
| `simd_performance_demo.rs` | SIMD 向量化 | AVX、NEON 指令集 |

### 符号计算
| 文件 | 描述 | 功能 |
|------|------|------|
| `symbolic_hamiltonian_demo.rs` | 符号哈密顿量 | 符号表达式、参数化 |
| `symbolic_quantum_mechanics.rs` | 符号量子力学 | 解析求解、符号推导 |

---

## 量子力学

量子力学基础和物理教学。

### 基本概念
| 文件 | 描述 | 物理概念 |
|------|------|---------|
| `qm_entanglement.rs` | 量子纠缠 | Bell 态、EPR 对 |
| `qm_spin_systems.rs` | 自旋系统 | Pauli 矩阵、自旋算符 |

### 量子系统
| 文件 | 描述 | 系统 |
|------|------|------|
| `qm_harmonic_oscillator.rs` | 量子谐振子 | 能级、波函数 |
| `qm_hydrogen_atom.rs` | 氢原子 | 能级结构、轨道 |

### 动力学
| 文件 | 描述 | 概念 |
|------|------|------|
| `qm_time_evolution.rs` | 时间演化 | Schrödinger 方程、演化算符 |
| `quantum_dynamics_demo.rs` | 量子动力学 | 动力学模拟、可观测量演化 |

### 高级主题
| 文件 | 描述 | 理论 |
|------|------|------|
| `chapter2_stationary_states.rs` | 定态理论 | 本征态、能量本征值 |
| `angular_momentum_demo.rs` | 角动量 | 角动量算符、耦合 |
| `perturbation_theory_demo.rs` | 微扰理论 | 能级修正、波函数修正 |

---

## 基础示例

简单的入门示例。

| 文件 | 描述 | 概念 |
|------|------|------|
| `bell_state.rs` | Bell 态制备 | 纠缠态、超密编码 |
| `quantum_collapse_demo.rs` | 量子态坍缩 | 测量、波函数坍缩 |
| `comprehensive_gates_demo.rs` | 综合门操作演示 | 所有标准门、门组合 |

---

## 教学演示

`teaching_demos/` 目录包含系统化的教学示例（13 个文件）：

1. **01_basic_circuits.rs** - 基础电路构建
2. **02_quantum_gates.rs** - 量子门操作
3. **03_measurements.rs** - 量子测量
4. **04_visualization.rs** - 电路可视化
5. **05_algorithms.rs** - 基础算法
6. **06_optimization.rs** - 电路优化
7. **07_simulation.rs** - 量子模拟
8. **08_noise_and_errors.rs** - 噪声和错误
9. **09_interactive_tutorial.rs** - 交互式教程
10. **10_advanced_features.rs** - 高级特性
11. **11_hamiltonian.rs** - 哈密顿量
12. **12_chemistry.rs** - 量子化学
13. **13_machine_learning.rs** - 量子机器学习

**推荐**: 按顺序学习，从 01 到 13。

---

## 学习路径建议

### 初学者路径
1. `beginner_tutorial.rs` - 了解基础概念
2. `bell_state.rs` - 理解纠缠
3. `qft.rs` - 学习 QFT
4. `grover_enhanced.rs` - 掌握搜索算法
5. `teaching_demos/` - 系统化学习

### 算法开发者路径
1. `interactive_tutorial.rs` - 快速上手
2. `quantum_algorithms_demo.rs` - 算法概览
3. `vqe.rs` + `qaoa_maxcut.rs` - NISQ 算法
4. `quantum_machine_learning.rs` - QML 应用
5. `optimization_layers_demo.rs` - 优化技术

### 量子化学路径
1. `chemistry_demo.rs` - 化学基础
2. `hamiltonian_demo.rs` - 哈密顿量构建
3. `vqe_h2.rs` - VQE 应用
4. `quantum_chemistry_demo.rs` - 完整流程
5. `h2_full_comparison.rs` - 方法对比

### 性能优化路径
1. `performance_optimization.rs` - 优化概览
2. `memory_pool_demo.rs` - 内存管理
3. `simd_performance_demo.rs` - SIMD 加速
4. `gpu_acceleration_demo.rs` - GPU 加速
5. `optimization_layers_demo.rs` - 电路优化

---

## 运行示例

### 编译单个示例
```bash
cargo build --example bell_state
```

### 运行示例
```bash
cargo run --example bell_state
```

### 编译所有示例
```bash
cargo build --examples
```

### 查看示例代码
```bash
cat examples/bell_state.rs
```

---

## 贡献指南

如果您想添加新的示例：

1. 确保示例有明确的教学或演示目的
2. 添加详细的文档注释
3. 包含 `Author: gA4ss` 标注
4. 遵循现有代码风格
5. 更新本索引文档

---

## 相关文档

- [README.md](README.md) - 项目概述
- [BEST_PRACTICES.md](BEST_PRACTICES.md) - 最佳实践
- [API 文档](https://docs.rs/myquat) - 完整 API 参考

---

**总示例数**: 69 个主示例 + 13 个教学示例 = 82 个  
**覆盖领域**: 算法、化学、优化、可视化、错误处理、硬件集成  
**难度范围**: 初学者 → 高级开发者
