# 高级推理：WCOJ、DRed 和 Tabling

pg_ripple 发布说明中有三个词 - **最坏情况最优连接**、**DRed**（Decremental Re-evaluation）和 **tabling** - 对于 SQL 或 SPARQL 从业人员来说没有明显的含义。本页介绍每个词的功能、引擎何时使用它们，以及它们对您的查询为何重要。

您无需配置其中任何一个。它们是自动的。但是，了解它们的存在有助于您理解为什么某些查询很快，为什么撤回是安全的，以及为什么递归查询不会无限循环。

---

## 最坏情况最优连接（WCOJ）

### 问题

经典的二元连接计划 - 每个 SQL 查询规划器使用的类型 - 对于某些三角形和团模式不是最优的。给定三个表 R、S、T：

```sql
-- 三角形查询：查找存在所有三个两两边的所有 A-B-C 三元组。
SELECT r.a, s.b, t.c
FROM   R r JOIN S s ON r.b = s.b
           JOIN T t ON s.c = t.c
           WHERE r.a = t.a;   -- 闭合三角形
```

二元连接计划首先处理 R⋈S（可能生成许多中间行），然后与 T 连接。对于密集的三角形模式，中间结果可能比最终输出大二四倍。连接是正确的，但是浪费的。

**最坏情况最优连接**（具体来说，**Leapfrog Triejoin** 算法）同时处理所有三个关系，交错枚举，以便中间结果永远不会大于最终结果。结果是*输出敏感*性能：在稀疏结果上快，在密集结果上快，永远不会病态。

### pg_ripple 何时使用它

WCOJ 自动用于 SPARQL 中的多谓词星形模式和三角形模式。您在不知情的情况下触发它：

```sparql
# 三角形模式：Alice 认识一个创建了 Alice 评分的事物的某人。
SELECT ?mid ?thing WHERE {
    ex:alice foaf:knows ?mid .
    ?mid schema:creator ?thing .
    ex:alice ex:rated ?thing .
}
```

这是一个三角形（Alice → knows → mid → creator → thing ← rated ← Alice）。pg_ripple 的 SPARQL-to-SQL 转换器检测到该模式并发出 WCOJ 计划，而不是两个二元连接。

### 何时关注

您几乎从不需要考虑这个。相关的场景：如果您看到涉及三个或更多 VP 表的 SPARQL 查询在循环模式下的执行速度明显低于预期，请运行 `EXPLAIN SPARQL` 并检查计划是否显示 leapfrog 节点。如果它显示嵌套循环连接，则查询优化器可能错过了这个机会 - 提交带有查询的问题。

---

## DRed — 减量重新评估

### 问题

物化的 Datalog 推理会创建派生的三元组。当您**删除**源三元组时，某些派生的三元组可能不再有效。天真的重新派生意味着从头开始重新运行整个推理 - 对于大型图来说是昂贵的。

**DRed**（带删除的减量重新评估）是一种更有效的算法：

1. 标记可能无效的派生三元组（"过度删除"集合）。
2. 仅使用幸存的源三元组从头开始重新派生过度删除集合中的所有内容。
3. 重新插入仍可派生的任何内容。

成本与*受影响的子图*成正比，而不是整个图。

### 在 pg_ripple 中

每当您使用删除活动规则集中作为主体原子使用的三元组的 `DELETE` 或 `DELETE WHERE` 调用 `sparql_update` 时，DRed 都会自动启动。

```sparql
# 删除源三元组。
DELETE DATA {
    <https://example.org/alice> foaf:knows <https://example.org/bob>
}
```

如果任何 Datalog 规则从 `foaf:knows` 派生三元组，pg_ripple 会将这些派生三元组标记为 DRed 重新评估并启动该算法。仍可证明的派生三元组（通过其他路径）被保留；没有其他派生的三元组被删除。

### 这对您意味着什么

- **删除是安全的。** 您永远不必在删除后手动协调派生三元组和基础三元组。引擎会处理它。
- **删除成比例地昂贵。** 删除的成本与依赖已删除事实的派生三元组的数量成比，而不是与总三元组总数成比。对于合理分解的规则集，这个成本很小。
- **避免手动派生三元组清理。** 不要编写直接针对 `_pg_ripple.vp_*` 表的 `DELETE WHERE` 查询。DRed 簿记使用源三元组的 OID；直接表操作绕过它并可能导致派生集不一致。

---

## Tabling

### 问题

递归 Datalog 规则可以循环。给定：

```
ancestor(X, Y) :- parent(X, Y) .
ancestor(X, Z) :- ancestor(X, Y), ancestor(Y, Z) .
```

天真的求值器如果父图中存在循环（例如，使用 `owl:sameAs` 或 `skos:broader*` 时存在真实风险），则会永远扩展 `ancestor`。

**Tabling**（也称为*记忆化*或 *SLG 解析*）通过缓存 — *tabling* — 中间结果来避免无限循环，并在递归调用将重新访问已位于活动堆栈上的目标时暂停求值。

### 在 pg_ripple 中

pg_ripple 的 Datalog 求值器使用自底向上半朴素迭代（SN）作为其主要策略。Tabling 用作无法安全分层化的规则集的后备方案。

在 SN 求值下：

1. 从基础事实开始。
2. 派生规则允许的每个新三元组（"delta"派生）。
3. 重复以新事实为种子，直到没有产生新三元组（不动点）。

不动点条件确保终止。Tabling 在不动点迭代期间用于检测和中断否则会阻止收敛的循环派生链。

### 这对您意味着什么

- **您可以安全地编写相互递归的规则。** pg_ripple 不需要您手动识别不动点。
- **数据中的循环不会导致无限循环。** `owl:sameAs` 链、`skos:broader` 循环或 `rdfs:subClassOf` 菱形层次结构都可以处理。
- **否定的良基语义。** 当规则在主体中使用失败即否定（`NOT`）时，pg_ripple 使用良基语义（WFS）为通过否定参与循环的原子分配第三个真值（"未知"）。这是在任何 Datalog 系统中可用的递归下否定的最有原则的处理。

---

## 它们如何组合

所有三个机制在同一查询/规则执行上协同工作：

| 机制 | 角色 |
|---|---|
| WCOJ（Leapfrog Triejoin） | 加速多路模式的*连接求值* |
| Tabling | 确保递归规则的*不动点终止* |
| DRed | 在基础三元组删除后维护派生三元组的*一致性* |

它们都不需要配置。它们是引擎正常操作的一部分，并根据查询形状、规则结构和更新类型自动激活。

---

## 另请参阅

- [推理和推理](reasoning-and-inference.md) — 完整的 Datalog 参考。
- [Lattice Datalog](lattice-datalog.md) — 当您的推理需要传播*值*而不仅仅是事实时。
- [OWL 2 Profiles](owl-profiles.md) — 特定的内置规则集。
- [SPARQL 查询调试器](../user-guide/explain-sparql.md) — 检查规划器选择的连接策略。

## 进一步阅读

- [博客：Leapfrog Triejoin](https://github.com/trickle-labs/pg-ripple/blob/main/blog/leapfrog-triejoin.md) — 最坏情况最优连接如何加速循环模式
- [博客：Well-Founded Semantics](https://github.com/trickle-labs/pg-ripple/blob/main/blog/well-founded-semantics.md) — 用于循环规则的三值逻辑
- [博客：Magic Sets for Goal-Directed Inference](https://github.com/trickle-labs/pg-ripple/blob/main/blog/magic-sets-goal-directed.md) — 需求驱动的求值