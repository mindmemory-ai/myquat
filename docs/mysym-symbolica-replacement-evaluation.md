# mysym 替代 symbolica 可行性评估

**日期:** 2026-06-18 | **评估者:** Claude (gA4ss 大管家)

## 总体判断

**mysym 结构完备，理论上可以替代，但需要编写 myquat 适配器并修复符号→数值桥接。**

与 mymat 不同，mysym 并非能力缺失型差距——它拥有 symbolica 被使用的几乎所有功能（表达式系统、解析器、微积分、化简、求解器）。真正的差距在于 **软件工程层面**：myquat 的 `MySymBackend` 适配器完全是占位代码（`unimplemented!()`），而 QM Solver 的符号→数值桥接函数也是存根（返回全零）。

---

## 背景

### myquat 的 symbolica 使用

| 文件 | 用途 |
|------|------|
| `src/symbolic/symbolica_adapter.rs` | `SymbolicaBackend` — 包装 symbolica 的 `Atom` 类型 |
| `src/symbolic/backend.rs` | `SymbolicBackend` trait — 39 个方法的抽象接口 |
| `src/symbolic/mysym_adapter.rs` | `MySymBackend` — **全部 `unimplemented!()`** |
| `src/qm_solver/tise_solver.rs` | TISE 求解器 — 使用 `backend.parse()`, `backend.variable()`, 算术运算 |
| `src/qm_solver/hilbert_space.rs` | 希尔伯特空间 — 使用 `SymbolicaExpression` 作为泛型参数 |
| `src/qm_solver/numerical_methods.rs` | 数值方法 — **`symbolic_to_numerical` 存根返回全零** |
| `src/qm_solver/quantum_chemistry.rs` | 量子化学 |
| 其他 8 个 qm_solver 文件 | 导入 `symbolic` 模块 |

### 实际调用的 `SymbolicBackend` 方法

```
variable()     — 创建符号变量 (m, hbar, omega, L, ...)
constant()     — 创建数值常量 (0.0, 2.0, -13.6, ...)
parse()        — 解析表达式字符串 ("p^2", "(1/2)*m*omega^2*x^2", ...)
add/sub/mul/div/pow — 算术运算
neg()          — 取负
exp/sin/cos/sqrt/ln/abs/conjugate — 数学函数
differentiate() — 求导
substitute()   — 变量替换
simplify() / expand() / factor() / collect() — 化简
solve() / solve_system() — (存根，未实际使用)
matrix / matrix_mul / determinant / eigenvalues / trace / commutator — 矩阵运算
expectation_value / time_evolution_operator — 量子力学专用 (存根)
```

### 关键发现：符号→数值桥接是存根

```rust
// src/qm_solver/numerical_methods.rs:1374-1390
pub fn symbolic_to_numerical<B, E>(
    _symbolic_hamiltonian: &E,
    grid_points: usize,
    _backend: &B,
) -> SymbolicResult<Vec<Vec<f64>>> {
    let n = grid_points;
    let matrix = vec![vec![0.0; n]; n];  // 返回全零！
    Ok(matrix)
}
```

`autodiff_gradient` 同样返回全零。这意味着 **即使使用 symbolica，QM Solver 的符号→数值混合计算路径也未实现**。当前 symbolica 的实际作用仅限于构建符号表达式和计算分析解（如无限深势阱的能量公式 $E_n = \frac{n^2\pi^2\hbar^2}{2mL^2}$），这些解是手动编码的解析公式，不依赖数值评估。

---

## ✅ mysym 已有能力（直接对应 SymbolicBackend 需求）

### 表达式系统 (mysym-core)

| SymbolicBackend 方法 | mysym 等价 | 状态 |
|---------------------|-----------|------|
| `variable("x")` | `sym("x")` → `Arc<dyn Expr>` | ✅ |
| `constant(2.0)` | `Float::new(2.0)` / `Integer::new(2)` | ✅ |
| `complex_constant(1.0, 2.0)` | `Complex::new(1.0, 2.0)` | ✅ |
| `parse("x^2 + 1")` | `mysym_io::parser::parse_expr("x^2 + 1")` | ✅ |
| `add(a, b)` | `a + b` (通过 `Add` 运算) | ✅ |
| `sub(a, b)` | `a - b` | ✅ |
| `mul(a, b)` | `a * b` (通过 `Mul` 运算) | ✅ |
| `div(a, b)` | `a / b` | ✅ |
| `pow(base, exp)` | `base.pow(exp)` (通过 `Pow` 运算) | ✅ |
| `neg(expr)` | `-expr` | ✅ |
| `exp(expr)` | `Function::new(FuncKind::Exp, expr)` | ✅ |
| `sin/cos/sqrt/ln/abs` | `FuncKind::Sin/Cos/Sqrt/Log/Abs` | ✅ |
| `conjugate(expr)` | `FuncKind::ConjugateFn` | ✅ |
| `differentiate(expr, var, n)` | `expr.diff(&symbol)` x n 次 | ✅ |
| `integrate(expr, var)` | `mysym_algebra::integrate::integrate()` (3728 行) | ✅ |
| `simplify(expr)` | `SimplifyEngine::apply()` (1077 行) | ✅ |
| `expand(expr)` | `mysym_algebra::expand::expand()` (1874 行) | ✅ |
| `factor(expr)` | `mysym_algebra::factor::factor()` (2972 行) | ✅ |
| `collect(expr, var)` | `mysym_algebra::poly::collect()` (4141 行) | ✅ |
| `substitute(expr, map)` | `expr.subs(old, new)` | ✅ |
| `solve(eq, var)` | `mysym_algebra::solve::solve()` (2713 行) | ✅ |
| `solve_system(eqs, vars)` | `solve()` 多变量版本 | ✅ |

### 数值评估 (mysym-core)

```
Expr::evalf(precision) → Result<Float, EvalfError>
Expr::evalf_precise() → Option<NumericValue>
NumericValue::to_f64() → f64
```

`NumericValue` 支持精确整数、精确有理数和 f64 浮点三种表示，`to_f64()` 可以将任意一种转为 `f64`。

### 矩阵运算 (mysym-linalg)

| 方法 | mysym 等价 |
|------|-----------|
| `matrix(elements)` | `Matrix::new(rows, cols, elements)` (4682 行实现) |
| `matrix_mul(a, b)` | `a * b` or `a.mat_mul(&b)` |
| `determinant(m)` | `m.det()` |
| `eigenvalues(m)` | `eigen.rs` (611 行) |
| `trace(m)` | `m.trace()` |
| `commutator(a, b)` | `a * b - b * a` |

### 整体规模

| 指标 | 数值 |
|------|------|
| 总代码行数 | **95,296** 行 Rust |
| 测试函数 | **746** + 8 property + 22 doc-tests（列出的） |
| 子 crate | 6 个 (core, algebra, linalg, domains, io, facade) |
| 支持的函数 | 70+ (FuncKind 包含初等、双曲、反双曲、特殊函数) |
| Risch 积分引擎 | 7,856 行（11 个文件） |
| ODE 求解器 | 4,361 行 |
| 多项式系统 | 4,141 行 (poly) + 1,098 行 (Groebner) |
| 因式分解 | 2,972 行 |
| 解析器 | 983 行 (支持隐式乘法) |

---

## ❌ 关键差距

### 1. myquat 适配器完全未实现

```rust
// src/symbolic/mysym_adapter.rs — 当前状态
impl SymbolicBackend for MySymBackend {
    fn variable(&self, _name: &str) -> SymbolicResult<Self::Expression> {
        Self::not_implemented()  // 全部 39 个方法都是这样
    }
}
```

**需要实现：**
- `MySymExpression` wrapper for `Arc<dyn mysym::Expr>` — 需满足 `Clone + Debug + Display + SymbolicExpression`
- 所有 39 个方法的具体实现（将 mysym 类型桥接到 SymbolicBackend trait）

### 2. 符号→数值桥接存根

QM Solver 的 `symbolic_to_numerical()` 和 `autodiff_gradient()` 返回全零——对 symbolica 和 mysym 都是存根。这是 myquat 自身未完成的功能，不是 mysym 的缺失。

### 3. 类型系统差异

| 方面 | symbolica | mysym |
|------|-----------|-------|
| 表达式类型 | `Atom` (具体类型) | `Arc<dyn Expr>` (trait object) |
| Clone | `Atom: Clone` | `Arc<dyn Expr>: Clone` ✅ |
| 内存管理 | 引用计数 (内部) | `Arc` (显式引用计数) |
| 线程安全 | 全局 `State` (非线程安全?) | `Arc` 支持 `Send + Sync` |
| 解析方式 | `Atom::parse(str)` | `mysym_io::parser::parse_expr(str)` |

### 4. 性能未知

- symbolica 是 C++ 级性能的 Rust 符号计算库（使用 arena 分配和高度优化的多项式算法）
- mysym 使用 `Arc<dyn Expr>` trait object，每次运算都涉及虚函数调用和堆分配
- 对于 QM Solver 的轻量级符号使用（构建解析公式），性能差异可能不显著

### 5. 稳定性和测试状态

mysym 测试总数 ~776，但实际运行状态未能在此次分析中确认（编译超时）。作为参考系：mysym 代码行数 95k 而 myquat 为 87k——两者规模相当。

---

## 📊 工作量估算

| 工作项 | 位置 | 工作量 | 风险 |
|--------|------|--------|------|
| 实现 `MySymExpression` wrapper | myquat | 0.5 天 | 低 |
| 实现 `MySymBackend` 39 个方法 | myquat | 2-3 天 | 中 |
| 修复 `symbolic_to_numerical` (数值评估链) | myquat | 2-3 天 | **高** |
| 实现 `autodiff_gradient` 真实计算 | myquat | 1-2 天 | 中 |
| 适配 QM Solver 泛型参数 (12 文件) | myquat | 1-2 天 | 中 |
| 回归测试 | myquat | 1-2 天 | 中 |
| 性能基准对比 (symbolica vs mysym) | both | 1 天 | 低 |

**保守估计：** 1.5-3 周全职工作。

---

## 🎯 对比：mymat 替代 vs mysym 替代

| 维度 | mymat 替代 ndarray | mysym 替代 symbolica |
|------|-------------------|---------------------|
| 受影响文件 | 35 个 | 15 个 |
| 缺失能力 | 有（Kronecker, Hermitian eigen） | 几乎没有 |
| 适配器现状 | 不存在 | 占位存根 |
| 实际使用深度 | **深**（模拟器核心路径） | **浅**（主要是符号公式构建） |
| 数值桥接 | 不适用 | **存根**（对两边都是） |
| 公共 API 影响 | Breaking (`ArrayView2`) | 无（`SymbolicBackend` 是内部 trait） |
| 替换风险 | **极高** | **中** |
| 短期可行性 | ❌ 不可行 | **⚠️ 有条件可行** |

---

## 🎯 建议方案

### 短期（v0.2.0）：实现 mysym 后端

**mysym 替代 symbolica 比 mymat 替代 ndarray 容易得多。**

原因：
1. mysym 功能完备——不需要在 mysym 中添加新能力
2. 实际影响范围小（15 个文件 vs 35 个）
3. symbolica 在 myquat 中的使用深度较浅（构建公式，不是数值模拟核心）
4. `SymbolicBackend` trait 已经是抽象接口——替换后端不需要改 QM Solver 代码

**推荐做法：**
1. 实现 `MySymExpression` wrapper (`Arc<dyn mysym::Expr>`)
2. 实现 `MySymBackend` 的 39 个方法
3. 修复 `symbolic_to_numerical` 存根——用 mysym 的 `evalf()` + `to_f64()` 做真正的数值评估
4. 通过 feature flag 提供 `mysym-backend` 选项

### 中期（v0.3.0）：默认后端切换

- 如果 mysym 后端通过所有 QM Solver 测试
- 性能不低于 symbolica（对于 QM Solver 使用场景）
- 可考虑将 mysym 设为默认符号后端

### 长期：独立演进

- mysym 本身是 95k 行的独立项目，有其自己的发展方向
- 不应为了 myquat 的需求而限制 mysym 的架构
- 保持 `SymbolicBackend` trait 的抽象性，支持两个后端并存

---

## 附录：适配器实现要点

`MySymExpression` 需要满足的 trait bounds：

```rust
#[derive(Clone)]
pub struct MySymExpression {
    expr: Arc<dyn mysym::Expr>,  // mysym 的核心表达式类型
}

impl SymbolicExpression for MySymExpression {
    fn to_string(&self) -> String { self.expr.to_string() }
    fn is_zero(&self) -> bool { self.expr.is_zero() }
    fn is_one(&self) -> bool { self.expr.is_one() }
    fn is_constant(&self) -> bool { self.expr.is_constant() }
    fn degree(&self, var: &str) -> Option<usize> { /* 从 poly 模块获取 */ }
}

impl Display for MySymExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expr)
    }
}
```

`MySymBackend` 的关键方法映射：

```rust
fn variable(&self, name: &str) -> SymbolicResult<Self::Expression> {
    Ok(MySymExpression::new(mysym::sym(name)))
}

fn parse(&self, expr: &str) -> SymbolicResult<Self::Expression> {
    let parsed = mysym_io::parser::parse_expr(expr)
        .map_err(|e| SymbolicError::InvalidExpression(e.to_string()))?;
    Ok(MySymExpression::new(parsed))
}

fn add(&self, a: &E, b: &E) -> SymbolicResult<E> {
    Ok(MySymExpression::new(a.expr.clone() + b.expr.clone()))
}
// ... 其余方法类似
```
