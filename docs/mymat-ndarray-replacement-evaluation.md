# mymat 替代 ndarray/nalgebra 可行性评估

**日期:** 2026-06-18 | **评估者:** Claude (gA4ss 大管家)

## 总体判断

**当前不可行，需要大量工作才能实现替代。**

mymat 具备了线性代数库约 70% 的功能（包括矩阵分解、求解器、矩阵函数等），但缺少量子计算所需的几个关键能力。myquat 全部 35 个源文件需要迁移，工程量巨大且风险高。

---

## 背景

### myquat 当前依赖

| 库 | 版本 | 用途 | 使用文件数 |
|----|------|------|-----------|
| `ndarray` | 0.15 | 多维数组 (Array1/Array2/ArrayView)、矩阵运算 | 34 |
| `ndarray-linalg` | 0.16 | Hermitian 特征值分解 (`Eigh`) | 2 |
| `nalgebra` | 0.33 | 复数 Schur 分解、SVD（用于 KAK 分解） | 3 |

### mymat 当前状态

- **版本:** 0.3.2
- **测试:** 90 lib tests passed, 215 total tests (13 ignored)
- **架构:** trait-based generic `T: Scalar` (f64/f32/Complex64)
- **后端:** CpuSingleBackend (完整), CpuMultiBackend (Rayon 并行)

---

## ✅ mymat 已有的能力（可直接替代）

### 数据结构

```rust
// ndarray                          →  mymat
Array2<Complex64>                   →  Matrix<Complex64>
Array1<Complex64>                   →  Vector<Complex64>
ArrayView2<Complex64>               →  MatrixView<'_, Complex64>
```

- `Matrix<T: Scalar = f64>` — row-major storage, `rows/cols/data` 字段
- `Vector<T: Scalar = f64>` — 一维向量
- `MatrixView<'a, T>` — 不可变零拷贝视图，支持 `row()` 切片
- `ComplexMatrix` / `ComplexVector` — 类型别名
- Serde 序列化支持
- `from_raw_parts()` 安全构造

### 基础运算 (`BasicOps<T>`)

| 操作 | mymat API |
|------|-----------|
| 加法 | `backend.add(&a, &b)` |
| 减法 | `backend.subtract(&a, &b)` |
| 矩阵乘法 | `backend.multiply(&a, &b)` |
| 矩阵-向量乘 | `backend.mat_vec_mul(&a, &b)` |
| 向量点积 | `backend.dot(&a, &b)` |
| 转置 | `backend.transpose(&a)` → 返回 owned Matrix |
| 逆矩阵 | `backend.inverse(&a)` → `MatResult<Matrix<T>>` |
| 标量乘 | `backend.scalar_multiply(&a, scalar)` |
| 范数 | `backend.norm(&a, NormType::Frobenius)` |

### 矩阵分解 (`DecompositionOps<T>`)

| 分解 | 算法 | 精度 |
|------|------|------|
| LU | 部分主元法，返回 (L, U, swap_count) | 高 |
| Cholesky | 严格 SPD 检查，非正定返回 `Err` | 高 |
| QR | Modified Gram-Schmidt + 一次重正交化 | 高 |
| SVD | One-sided Jacobi（直接在 A 列上操作） | cond ~1e15 |
| 特征值 | 对称 QR + Wilkinson shifts（三次收敛） | — |
| Schur | 准三角 T 矩阵 | — |
| Hessenberg | 正交相似变换 | — |
| RRQR | Rank-revealing QR | — |
| Polar | 极分解 | — |

### 矩阵函数 (`AdvancedMatrixFuncs<T>`)

`matrix_exp`, `matrix_log`, `matrix_sqrt`, `matrix_sin`, `matrix_cos`,
`matrix_sign`, `matrix_sector`, `matrix_power`, `matrix_pth_root`,
`solve_lyapunov`, `solve_discrete_lyapunov`, `solve_sylvester`

### 矩阵属性 (`MatrixProps<T>`)

`determinant`, `trace`, `rank`, `condition_number`, `is_symmetric`,
`is_positive_definite`, `is_orthogonal`, `is_unitary`, `is_hermitian`,
`is_normal`, `is_diagonal`, `is_triangular`, `is_sparse`, `is_toeplitz`,
`frobenius_norm`, `spectral_norm`, `nuclear_norm`, `spectral_radius`

### 数据构造 (`DataOps<T>`)

`zeros`, `ones`, `identity`, `diagonal`, `random`, `linspace`, `slice`,
`reshape`, `concatenate`, `toeplitz`, `hankel`, `circulant`, `vandermonde`,
`hilbert`, `pascal`, `from_2d_array`

### 其他

- **小型静态矩阵:** `SMatrix<T, R, C>` — 栈分配 2×2/4×4 等，适合门矩阵
- **并行后端:** `CpuMultiBackend` — Rayon 加速分解 rank-1 更新
- **稀疏矩阵:** CSR (`SparseMatrix`), CSC (`SparseMatrixCSC`) + 迭代求解器
- **求解器系统:** LU/QR/SVD/Cholesky 直接求解 + CG/GMRES/BiCGSTAB + 正则化
- **预处理器:** Jacobi, ILU(0), IC(0)
- **Matrix Market I/O:** `.mtx` 文件读写
- **数值精度:** `PrecisionConfig` 全局精度控制 + 专用容差

---

## ❌ 关键缺失

### 1. Kronecker 积（张量积）— 阻塞级

**影响范围:** 30+ 处使用，所有量子门扩展的核心运算

```rust
// myquat 当前: gates.rs, circuit.rs, custom_gate_matrix.rs 等
let expanded = ndarray::kron(&gate1, &gate2);  // H ⊗ I, I ⊗ X 等
```

mymat **完全没有** Kronecker 积实现。在量子计算中，将单量子比特门扩展到完整希尔伯特空间必须使用 Kronecker 积：

```
U₀ 作用在 qubit 0 上 → U₀ ⊗ I ⊗ I ⊗ I (4 qubit 系统)
CNOT 分解为投影算子 → |0⟩⟨0| ⊗ I + |1⟩⟨1| ⊗ X
```

**需添加:** `BasicOps::kron(&self, a: &Matrix<T>, b: &Matrix<T>) -> Matrix<T>` (约 50 行)

### 2. 复数 Hermitian 特征值分解 — 阻塞级

**影响范围:** `density_matrix.rs` (von Neumann 熵、fidelity 计算), `numerical_methods.rs` (量子态对角化)

```rust
// myquat 当前
use ndarray_linalg::Eigh;
matrix.eigh(ndarray_linalg::UPLO::Lower)  // 复数 Hermitian 矩阵
```

mymat 的 `eigen()` 基于对称 QR 迭代（Wilkinson shifts），设计目标是**实对称矩阵**。对于 `Matrix<Complex64>` 的 Hermitian 矩阵：

- `eigen()` 返回 `MatResult<(Vector<T>, Matrix<T>)>` — 特征值为 `Vector<T>` (_无虚部_)，这对于复数 Hermitian 矩阵是错误的
- `ndarray-linalg::Eigh` 调用 LAPACK `zheev`，专门优化了 Hermitian 情况的数值稳定性
- 密度矩阵的 von Neumann 熵计算依赖正确的 Hermitian 特征值分解

### 3. 复数 Schur 分解 — 需验证

**影响范围:** `two_qubit_decompose.rs` — KAK 分解核心（双量子比特优化）

```rust
// myquat 当前
use nalgebra::Schur;
nalgebra::Schur::try_new(matrix4x4, 1e-10, 100).unwrap()
```

mymat 声明了 `fn schur(&self, a: &Matrix<T>) -> (Matrix<T>, Matrix<T>)`，但：
- 测试覆盖**仅限于 f64 矩阵**
- 复数 Schur 需要处理 2×2 块（共轭特征值对），代码中的 `advanced_matrix_funcs.rs:597` 有相关逻辑但未在测试中为复数矩阵验证
- KAK 分解是 myquat 双量子比特优化的核心算法，任何 bug 都会导致错误的编译结果

### 4. 复数 SVD — 需验证

**影响范围:** `two_qubit_decompose.rs:1342` (KAK 分解中计算规范形式)

```rust
// myquat 当前
let svd = nalgebra::linalg::SVD::new(r, true, true);
let u = svd.u.unwrap_or_else(nalgebra::Matrix4::identity);
```

mymat 的 `svd()` 使用 one-sided Jacobi，代码中有 `.conj()` 操作（说明考虑了复数），但**所有测试都是 f64**。复数 SVD 需要正确处理共轭和内积的符号约定。

### 5. 逐元素操作 — 影响开发效率

| ndarray API | myquat 使用频率 | mymat 等价 | 替代复杂度 |
|-------------|---------------|-----------|-----------|
| `a.mapv(\|x\| f(x))` | 高 (20+ 处) | 无，需手写 for 循环 | 中等 |
| `a.fill(val)` | 中 (15+ 处) | 无，需 new + set 循环 | 低 |
| `a.conj()` (逐元素共轭) | 中 | `Scalar::conj()` 只在元素级 | 低（可加方法） |
| `a[[i, j]]` 索引语法 | 极高 (500+ 处) | `a.get(i, j)` / `a.set(i, j, val)` | 高（量太大） |
| `a.t()` (zero-copy view) | 中 | `transpose()` 返回 owned | 性能退化 |

### 6. SIMD 加速 — 性能差距

- ndarray 通过 `matrixmultiply` 获得 SIMD 加速
- mymat 的纯 Rust 循环在非 BLAS 路径上没有 SIMD
- mymat 的 CBLAS 集成仅支持 `f64`/`f32`，不支持 `Complex64`
- 量子模拟器对矩阵乘法的性能极其敏感

---

## 📊 迁移工作量估算

| 工作项 | 位置 | 类型 | 工作量 | 风险 |
|--------|------|------|--------|------|
| 添加 Kronecker 积 | mymat | 新增功能 | 0.5 天 | 低 |
| 添加 element-wise map/fill/conj | mymat | 新增功能 | 0.5 天 | 低 |
| 添加 conjugate-transpose | mymat | 新增功能 | 0.5 天 | 低 |
| 实现/验证 Hermitian eigenvalue | mymat | Bug fix + LAPACK FFI | 2-3 天 | **高** |
| 验证复数 Schur 分解 | mymat | 测试 + 修复 | 1-2 天 | **高** |
| 验证复数 SVD | mymat | 测试 + 修复 | 1-2 天 | 中 |
| 增强 2D 索引支持 | mymat | API 增强 | 1 天 | 低 |
| myquat API 迁移（35 文件） | myquat | 重构 | 5-7 天 | **极高** |
| myquat 公共 API 变更 | myquat | Breaking change | 1-2 天 | **高** |
| 回归测试（1085 tests） | myquat | QA | 2-3 天 | 中 |
| 性能基准对比 | both | QA | 1 天 | 低 |

**保守估计：** 3-5 周全职工作。

---

## 🔧 迁移涉及的文件清单

需要修改的 35 个 myquat 源文件：

```
src/circuit.rs                  — QuantumCircuit unitary 构建
src/circuit_optimization.rs     — 优化 pass (fidelity 计算)
src/compute/backend_trait.rs    — 后端接口 (apply_unitary)
src/compute/cloud/aws_backend.rs
src/compute/cloud/ibm_backend.rs
src/compute/local/cpu_backend.rs
src/compute/local/cuda_backend.rs
src/compute/local/gpu_backend.rs
src/compute/local/parallel_backend.rs
src/compute/local/simd_backend.rs
src/compute/parallel_ops.rs
src/compute/parallel_simulator.rs
src/compute/simd_ops.rs
src/compute/simd_two_qubit_gates.rs
src/custom_gate_matrix.rs       — 自定义门矩阵
src/density_matrix.rs           — 密度矩阵 (ndarray + nalgebra)
src/error.rs
src/error_mitigation.rs         — ZNE
src/gate_decomposition.rs       — U3/ZYZ 分解
src/gate_expansion.rs           — 门扩展到完整空间
src/gate_inverse.rs             — 门逆矩阵
src/gates.rs                    — 45+ 标准门矩阵定义
src/gates_extended.rs
src/matrix_cache.rs             — 矩阵缓存
src/memory_optimized.rs         — MemoryEfficientState
src/memory_pool.rs              — 内存池
src/noise_models.rs
src/optimization_passes.rs      — GateFusion, single_qubit_matrix
src/phase_polynomial.rs         — 相位多项式
src/qm_solver/numerical_methods.rs — 数值方法 (eigh)
src/quantum_info.rs             — 量子信息论
src/simulator.rs                — 核心模拟器
src/two_qubit_decompose.rs      — KAK 分解 (ndarray + nalgebra)
src/utils.rs
```

---

## 🎯 建议方案

### 短期（v0.1.x ~ v0.2.0）: 不替换

1. **风险太高** — 35 个文件、87k 行代码、1085 个测试
2. **关键 bug 未知** — 复数 Schur/SVD/Hermitian-eigen 未经充分测试
3. **性能不确定** — mymat 纯 Rust 循环可能比 ndarray + matrixmultiply 慢
4. **公共 API 不兼容** — `ArrayView2` 是 myquat 的公共类型，替换会 break 下游

### 中期（v0.3.0+）: 分阶段引入

1. **Phase 1: 补齐 mymat 能力**
   - 添加 Kronecker 积
   - 添加 element-wise map/fill/conj
   - 实现 Hermitian 特征值分解（包装 LAPACK `zheev`）
   - 为复数矩阵添加全面测试

2. **Phase 2: 非关键路径迁移**
   - 用 feature flag 引入 mymat（`mymat-backend`）
   - 先迁移 gate_decomposition, circuit_analysis 等非性能关键路径
   - 保持 ndarray 作为默认后端

3. **Phase 3: 模拟器迁移**
   - 基准测试确认性能无退化
   - 迁移 simulator.rs 和 compute 后端
   - 完整回归测试

### 长期: 统一线性代数层

- 如果 mymat 的所有能力都达到生产级，可考虑完全替换
- 提供 `From<ndarray::Array2>` / `Into<ndarray::Array2>` 桥接，降低迁移摩擦

---

## 附录：API 对照表

| ndarray | mymat 等价 | 状态 |
|---------|-----------|------|
| `Array2::zeros((r, c))` | `Matrix::new(r, c)` | ✅ |
| `Array2::eye(n)` | `backend.identity(n)` | ✅ |
| `Array2::from_shape_vec((r,c), data)` | `backend.matrix(r, c, data)` | ✅ |
| `a.dot(&b)` | `backend.multiply(&a, &b)` | ✅ |
| `a.t()` | `backend.transpose(&a)` | ✅ (returns owned) |
| `a.view()` | `a.view()` | ✅ |
| `a[[i, j]]` | `a.get(i, j)` / `a.set(i, j, v)` | ✅ (不同语法) |
| `a.mapv(f)` | 手写 for 循环 | ❌ |
| `a.fill(v)` | 手写 for 循环 | ❌ |
| `a.conj()` (matrix) | 手写 for 循环 | ❌ |
| `ndarray::kron(&a, &b)` | **不存在** | ❌ |
| `a.eigh(UPLO)` (complex Hermitian) | **不存在** | ❌ |
| `nalgebra::Schur::try_new(m)` | `backend.schur(&m)` (未验证复数) | ⚠️ |
| `nalgebra::linalg::SVD::new(m)` | `backend.svd(&m)` (未验证复数) | ⚠️ |
