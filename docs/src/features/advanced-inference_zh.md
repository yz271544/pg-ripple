# 高级推理：WCOJ、DRed 和 Tabling

在发布的版本说明中出现了三个术语 —— **最坏情况最优连接（WCOJ）**、**DRed**（Decremental Re-evaluation，减量重计算）和 **tabling（制表）** —— 它们对 SQL 或 SPARQL 从业者来说没有明显的含义。本页解释了每个的作用、引擎何时使用它们以及它们为什么对您的查询很重要。

您无需配置它们中的任何一个。它们是自动的。但了解它们的存在有助于您理解为什么某些查询很快，为什么撤回是安全的，以及为什么递归查询不会无限循环。

---

## 最坏情况最优连接（WCOJ）

### 问题

经典的二元连接计划 —— 每个 SQL 查询规划器使用的那种 —— 对于某些三角形和团模式是次优的。给定三个表 R、S、T：

```sql
-- 三角形查询：查找所有三个成对边都存在的 A-B-C 三元组。
SELECT r.a, s.b, t.c
FROM   R r JOIN S s ON r.b = s.b
           JOIN T t ON s.c = t.c
           WHERE r.a = t.a;   -- 闭合三角形
```

二元连接计划首先处理 R⋈S（可能生成许多中间行），然后与 T 连接。对于密集三角形模式，中间结果可能比最终输出大二阶。连接是正确的，但是浪费。

**最坏情况最优连接**（具体来说，**Leapfrog Triejoin** 算法）同时处理所有三个关系，交错枚举以便中间结果永远不会大于最终结果。结果是*输出敏感*的性能：在稀疏结果上快，在密集结果上快，从不病态。

### pg_ripple 何时使用它

WCOJ 自动用于 SPARQL 中的多谓词星形模式和三角形模式。您在不知情的情况下触发它：

```sparql
# 三角形模式：Alice 认识某人，该人创造了某物，Alice 评价了该物。
SELECT ?mid ?thing WHERE {
    ex:alice foaf:knows ?mid .
    ?mid schema:creator ?thing .
    ex:alice ex:rated ?thing .
}
```

这是一个三角形（Alice → knows → mid → creator → thing ← rated ← Alice）。pg_ripple 的 SPARQL-to-SQL 转换器检测到此模式并发出 WCOJ 计划而不是两个二元连接。

### 何时需要注意

您几乎不需要考虑这一点。相关场景：如果您看到涉及三个或更多 VP 表的循环模式的 SPARQL 查询明显表现比预期差，请运行 `EXPLAIN SPARQL` 并检查计划是否显示 leapfrog 节点。如果它显示嵌套循环连接，则查询优化器可能错过了机会 —— 请提交一个带有查询的问题。

---

## DRed —— 减量重计算

### 问题

物化的 Datalog 推理创建派生的三元组。当您**删除**源三元组时，某些派生的三元组可能不再有效。天真的重推导将意味着从头开始重新运行整个推理 —— 对于大型图来说代价昂贵。

**DRed**（带删除的减量重计算）是一种更高效的算法：

1. 标记*可能*无效的派生三元组（"过度删除"集合）。
2. 仅使用幸存的源三元组从头开始重新推导过度删除集合中的所有内容。
3. 重新插入任何仍可推导的内容。

成本与*受影响的子图*成正比，而不是整个图。

### 在 pg_ripple 中

每当您使用 `DELETE` 或 `DELETE WHERE` 调用 `sparql_update` 删除用作活动规则集中主体原子的三元组时，DRed 会自动启动。

```sparql
# 删除源三元组。
DELETE DATA {
    <https://example.org/alice> foaf:knows <https://example.org/bob>
}
```

如果任何 Datalog 规则从 `foaf:knows` 派生三元组，则 pg_ripple 会将这些派生的三元组标记为 DRed 重计算并启动该算法。仍然可推导的派生三元组（通过其他路径）被保留；没有其他推导的派生三元组被删除。

### 这对您意味着什么

- **删除是安全的。** 您永远不必在删除后手动协调派生三元组和基础三元组。引擎会处理它。
- **删除的代价与受影响部分成正比。** 删除的代价与依赖于已删除事实的派生三元组数量成正比，而不是与总三元组数量成正比。对于良好分解的规则集，这个代价很小。
- **避免手动派生三元组清理。** 不要编写直接针对 `_pg_ripple.vp_*` 表的 `DELETE WHERE` 查询。DRed 簿记使用源三元组的 OID；直接表操作绕过它，可能导致派生集不一致。

---

## Tabling（制表）

### 问题

递归 Datalog 规则可以循环。给定：

```
ancestor(X, Y) :- parent(X, Y) .
ancestor(X, Z) :- ancestor(X, Y), ancestor(Y, Z) .
```

如果父图中存在循环（例如，使用 `owl:sameAs` 或 `skos:broader*` 时的实际风险），天真的求值器将永远扩展 `ancestor`。

**Tabling**（也称为*记忆化*或 *SLG 解析*）通过缓存 —— *制表* —— 中间结果来避免无限循环，并在递归调用将重新访问已处于活动堆栈上的目标时挂起求值。

### 在 pg_ripple 中

pg_ripple 的 Datalog 求值器使用自底向上的半朴素迭代（SN）作为其主要策略。Tabling 用作无法安全分层的规则集的回退。

在 SN 求值下：

1. 从基本事实开始。
2. 推导规则允许的每个新三元组（"增量"推导）。
3. 以新事实为种子重复，直到不产生新三元组（不动点）。

不动点条件确保终止。在不动点迭代期间使用 Tabling 来检测和打破循环推导链，否则会阻止收敛。

### 这对您意味着什么

- **您可以安全地编写相互递归的规则。** pg_ripple 不需要您手动识别不动点。
- **数据中的循环不会导致无限循环。** `owl:sameAs` 链、`skos:broader` 循环或 `rdfs:subClassOf` 菱形层次结构都可以处理。
- **良基语义用于否定。** 当规则使用失败即否定（主体中的 `NOT`）时，pg_ripple 使用良基语义（WFS）将第三个真值（"未知"）分配给通过否定参与循环的原子。这是任何 Datalog 系统中可用的递归下否定最严谨的处理。

---

## 它们如何组合

所有三种机制在同一查询/规则执行上协同工作：

| 机制 | 角色 |
|---|---|
| WCOJ（Leapfrog Triejoin） | 加速多路模式的*连接求值* |
| Tabling | 确保递归规则的*不动点终止* |
| DRed | 在基础三元组删除后维护派生三元组的*一致性* |

它们都不需要配置。它们是引擎正常操作的一部分，并根据查询形状、规则结构和更新类型自动激活。

---

## 另请参阅

- [推理与演绎](reasoning-and-inference.md) — 完整的 Datalog 参考。
- [Lattice Datalog](lattice-datalog.md) — 当您的推理需要传播*值*而不仅仅是事实时。
- [OWL 2 Profile](owl-profiles.md) — 特定的内置规则集。
- [SPARQL 查询调试器](../user-guide/explain-sparql.md) — 检查规划器选择的连接策略。

## 进一步阅读

- [博客：Leapfrog Triejoin](https://github.com/trickle-labs/pg-ripple/blob/main/blog/leapfrog-triejoin.md) — 最坏情况最优连接如何加速循环模式
- [博客：良基语义](https://github.com/trickle-labs/pg-ripple/blob/main/blog/well-founded-semantics.md) — 循环规则的三值逻辑
- [博客：用于目标导向推理的 Magic Sets](https://github.com/trickle-labs/pg-ripple/blob/main/blog/magic-sets-goal-directed.md) — 需求驱动的求值