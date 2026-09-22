# Lattice Datalog — 何时和为什么

[Lattice Datalog 参考](../reference/lattice-datalog.md) 记录了每个函数和 GUC。本页回答一个不同的问题：**您是否应该使用 lattice？** 大多数用户从不需要。本页可帮助您做出决定。

---

## 简短的回答

当您需要沿着图边传播*值*时使用 lattice — 而不是*是否*可以从一个节点到达另一个节点，而是*该节点上某物值多少*，并且传播是**递归的**。

如果您的规则不是递归的，则标准 Datalog 聚合（跨层的 `COUNT`、`SUM`、`AVG`）更简单。

---

## 一个具体的直觉

标准 Datalog 询问：*能否从 A 到达 B？*

Lattice Datalog 询问：*从 A 到 B 的最佳方式是什么？*

同一张图，两个不同的问题：

```
A ──0.9──► B ──0.85──► C ──0.95──► D
```

| 问题 | 答案 | 工具 |
|---|---|---|
| D 可从 A 到达吗？ | 是 | 标准 Datalog |
| 到达 D 的任何路径上的最大单跳权重是多少？ | 0.95 | 标准 Datalog + MAX 聚合（非递归）|
| 从 A 到 D 的最佳路径上的最小权重是多少？ | 0.85（瓶颈）| **Lattice Datalog 与 `min`** |
| 沿最佳路径的权重乘积是多少？ | 0.726 | **自定义 lattice** |
| 从 A 到 D 的每条路径都通过哪些中间节点？ | {B, C} | **SetLattice** |

一旦"最佳路径"是递归的 — 您事先不知道哪个方向是最佳的 — 您就需要一个 lattice。

---

## 选择正确的内置 lattice

### `min` — 最弱链接推理

```
链条的强度是其最弱链接的强度。
```

在以下情况下使用 `min`：
- 通过网络传播**信任/置信度**（结果仅与最少信任的步骤一样可信）。
- 计算**最短路径**，其中路径成本是最大边权重。
- 在流网络中查找**瓶颈容量**（流受最窄管道限制）。

```sql
-- 加载规则（信任传播，受最弱跳跃瓶颈限制）。
SELECT pg_ripple.load_rules($RULES$
?x ex:trusts ?y :- ?x ex:directlyTrusts ?y .
?x ex:trusts ?z :- ?x ex:directlyTrusts ?y, ?y ex:trusts ?z .
@lattice ex:trusts confidence min .
$RULES$, 'trust');

SELECT pg_ripple.infer_lattice('trust', 'min');
```

### `max` — 最佳情况推理

在以下情况下使用 `max`：
- 传播**声誉/认可**分数，其中拥有一个高评级的连接就足够了。
- 在 DAG 中查找**最长路径权重**。
- 任何"乐观"或"最佳证据胜出"场景。

```sql
@lattice ex:endorses score max .
```

### `set` — 来源和多值推理

在以下情况下使用 `set`：
- 您需要跟踪哪些**源三元组**证明派生事实（来源半环）。
- 您需要所有派生路径上所有证人的**并集**，而不仅仅是一个。
- 每个节点从多个父节点收集贡献，并且您需要所有这些贡献。

```sql
-- 收集支持假设的所有论文的集合。
@lattice ex:supports evidence set .
```

注意：set-lattice 结果可能很大。对于大型图，请考虑最大集合大小或布隆过滤器近似。

### `interval` — 当真值有时间范围时

在推理时间重叠时使用 `interval`：*"A 和 B 在其有效区间重叠的期间都成立"*。

```sql
-- 派生事实仅在主体区间的交集期间有效。
@lattice ex:validDuring interval interval .
```

---

## 在 `min` 和自定义乘法 lattice 之间选择

当图腾出许多跳跃时，差异很重要：

| 跳跃 | `min` 结果 | 乘法结果 |
|---|---|---|
| A→B (0.9) | 0.9 | 0.9 |
| A→B→C (×0.85) | 0.85 | 0.765 |
| A→B→C→D (×0.95) | 0.85 | 0.726 |

- **`min`**："我最不可靠的来源有多可靠？" — 适用于信任链，其中一个坏链接会使所有内容无效。
- **乘法**："假设独立性的组合概率是多少？" — 适用于贝叶斯风格的置信度传播。

对于乘法情况，注册自定义聚合：

```sql
-- 第 1 步：创建乘法聚合。
CREATE OR REPLACE FUNCTION prob_mul(state FLOAT8, val FLOAT8)
    RETURNS FLOAT8 LANGUAGE sql IMMUTABLE AS $$ SELECT COALESCE(state, 1.0) * val $$;

CREATE AGGREGATE prob_product(FLOAT8) (SFUNC = prob_mul, STYPE = FLOAT8, INITCOND = '1.0');

-- 第 2 步：注册 lattice。
SELECT pg_ripple.create_lattice('probability', 'prob_product', '0.0');

-- 第 3 步：运行推理。
SELECT pg_ripple.infer_lattice('my_rules', 'probability');
```

---

## 我真的需要 lattice 吗？

在使用 lattice 之前，检查更简单的方法是否有效：

| 场景 | 更简单的替代方案 |
|---|---|
| 计数可到达的边 | SQL 中的标准 `WITH RECURSIVE` |
| 沿固定深度路径的权重求和 | 带算术的 `SPARQL SELECT` |
| 非递归规则的聚合 | 带 `@agg` 指令的标准 Datalog |
| 置信度是加法的，而不是乘法或最小 | 非递归规则中的 `SUM` 聚合 |

Lattice 增加了一个复杂性：不动点循环。如果您可以将相同的计算表示为单个 SQL `WITH RECURSIVE` 查询或非递归 Datalog 规则，那么几乎总是更简单和更快。

---

## 收敛和终止

如果 lattice 具有**升链条件** — 没有无限严格升链 — 则保证每个 lattice 计算都会终止。所有四个内置集都满足此条件：
- `min` 在有界整数/浮点数上：仅降序。
- `max` 在有界整数/浮点数上：升序但有界。
- `set` 的有限宇宙：有限幂集，每个链终止。
- `interval` 在有界时间戳上：有界。

无界域上的自定义 lattice 可能会发散。将 `pg_ripple.lattice_max_iterations`（默认值：1000）设置为安全上限。如果达到上限，引擎会发出 `PT540` 警告并返回当前（部分）不动点。

---

## 另请参阅

- [Lattice Datalog 参考](../reference/lattice-datalog.md) — 所有 SQL 函数、GUC 和目录表。
- [Cookbook：概率规则](../cookbook/probabilistic-rules.md) — 端到端详细示例。
- [推理和推理](reasoning-and-inference.md) — 完整的 Datalog 故事。