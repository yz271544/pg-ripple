# SPARQL 1.2 属性路径状态

此页面记录了所有八个 SPARQL 1.2 属性路径代数运算符在 pg_ripple v0.124.0 中的执行状态。

```admonish success title="所有运算符受支持"
截至 v0.124.0，pg_ripple 执行 spargebra 0.4.6 定义的八个 `PropertyPathExpression` 变体（功能 `sparql-12` + `sep-0006`）。在此版本中修复了 PATH-BNODE-01 bug：由 spargebra 分解为两个由匿名空白节点连接的 `GraphPattern::Path` 节点的序列路径现在产生正确的 INNER JOIN 而不是笛卡尔积。
```

---

## 运算符覆盖表

| SPARQL 语法 | spargebra 变体 | SQL 策略 | 状态 | 起始版本 |
|---|---|---|---|---|
| `<p>` | `NamedNode` | 直接 VP 表扫描 | ✅ 完整 | v0.5.0 |
| `^p` | `Reverse` | `SELECT o AS s, s AS o` 交换 | ✅ 完整 | v0.5.0 |
| `a/b` | `Sequence` | 在中间节点上的子查询 INNER JOIN | ✅ 完整 | v0.5.0 |
| `a\|b` | `Alternative` | `UNION ALL` | ✅ 完整 | v0.5.0 |
| `p+` | `OneOrMore` | `WITH RECURSIVE` + `CYCLE` (PG 18) | ✅ 完整 | v0.5.0 |
| `p*` | `ZeroOrMore` | `WITH RECURSIVE` + `CYCLE` + 零跳 `UNION ALL` | ✅ 完整 | v0.5.0 |
| `p?` | `ZeroOrOne` | 带标识行的直接 `UNION ALL` | ✅ 完整 | v0.5.0 |
| `!(p1\|p2)` | `NegatedPropertySet` | `vp_rare WHERE p NOT IN (...)` | ✅ 完整 | v0.5.0 |

---

## Bug 修复：PATH-BNODE-01 — 通过空白节点连接加入路径（v0.124.0）

**根本原因。** spargebra/sparopt 优化器有时将诸如 `hop*/hop` 之类的序列路径表达式分解为两个由匿名空白节点连接的独立 `GraphPattern::Path` 代数节点：

```
{ <a> (hop)* _:b0 . _:b0 hop ?x . }
```

在 v0.124.0 之前，`sqlgen.rs` 中的 `GraphPattern::Path` 转换器仅处理主语/对象绑定的 `TermPattern::Variable`。空白节点被静默忽略，因此两个路径子查询由 SQL `FROM` 子句中的笛卡尔积而不是共享列上的 `INNER JOIN` 连接。这产生了 N × M 重复行（例如，对于 5 跳链，30 行而不是 5 行）。

**修复。** 路径转换器现在为主语和对象都调用 `bgp::bind_term()`，它统一处理 `Variable`、`BlankNode`、`NamedNode` 和 `Literal`。当空白节点出现在两个路径片段中时，`Fragment::merge` 添加连接条件（例如 `_t0.o = _t1.s`）。

**受影响的模式（示例）。**

| SPARQL 路径 | 由 spargebra 分解 | 修复前 | 修复后 |
|---|---|---|---|
| `hop*/hop` | `hop* _:b . _:b hop` | 30 行（×6） | 5 行 ✅ |
| `hop?/hop` | `hop? _:b . _:b hop` | 10 行（×5） | 2 行 ✅ |
| `^hop/!hop` | `^hop _:b . _:b !hop` | 6 行（×2） | 3 行 ✅ |

---

## 回归测试覆盖范围

回归测试套件包括在 v0.124.0 中添加的两个专用测试文件：

| 文件 | 测试 | 覆盖范围 |
|---|---|---|
| `tests/pg_regress/sql/sparql12_property_paths.sql` | 20 | 所有 8 个运算符 + 5 个复合组合 |
| `tests/pg_regress/sql/sparql12_owl_chain_nhop.sql` | 5 | OWL `owl:propertyChainAxiom` n=4 和 n=5 跳链 |

截至 v0.124.0，所有 25 个测试均通过。

---

## 已知限制

| 限制 | 票号 | 解决方法 |
|---|---|---|
| 带有嵌套递归 CTE 的 `GRAPH ?g { path }` 可能不会通过所有跳传播图上下文 | PATH-G-01 | 使用命名图过滤器：`GRAPH <uri> { path }` |
| 最大路径深度由 `pg_ripple.max_path_depth` GUC（默认值 100）控制 | — | 对于非常深的图增加 GUC |