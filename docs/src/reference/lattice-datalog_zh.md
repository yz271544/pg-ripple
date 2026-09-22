# Lattice-Based Datalog 参考（v0.36.0）

> **自 v0.36.0 起可用。** Lattice-Based Datalog（Datalog^L）通过单调格（lattice）聚合扩展了 pg_ripple 的 Datalog 引擎，使其能够进行不受分层（stratification）约束限制的递归聚合。

---

## 背景

标准 Datalog^agg 对聚合函数做分层处理（stratification）：聚合函数只能出现在其所聚合谓词的严格更高层（stratum）上。这使得递归聚合（例如在社会图中传播最小信任分数）无法在不手动展开循环（loop unrolling）的情况下表达。

**Lattice-Based Datalog** 放宽了这一限制，仅要求聚合操作对用户提供的*格*（lattice）是*单调*的。格是一个代数结构 (L, ⊔)，其中 ⊔ 是一个交换、结合、幂等的 join 运算，且具有底元素 ⊥。基于格的不动点（fixpoint）计算会因升链条件（ascending chain condition）而终止——格中不存在无穷严格升链。

### 主要参考文献

- Abo Khamis et al., *PODS 2017* — Datalog 中的格结构聚合
- Alvaro et al., *CIDR 2011* — 单调逻辑编程（Bloom^L）
- Green et al., *PODS 2007* — 溯源半环（provenance semirings）作为格的推广

---

## 内置格

pg_ripple 内置了四种覆盖最常见用例的格类型：

### MinLattice（`min`）

```
join:   LEAST(a, b)       (PostgreSQL: min 聚合)
bottom: +∞  (编码为 9223372036854775807 = i64::MAX)
```

**用例：** 信任传播、最短路径权重、最小成本路由。

**示例：** 沿路径传播最小信任分数——一条链的可信度受限于其最弱的环节。

### MaxLattice（`max`）

```
join:   GREATEST(a, b)   (PostgreSQL: max 聚合)
bottom: −∞  (编码为 -9223372036854775808 = i64::MIN)
```

**用例：** 可达性权重、最长路径标注、最大影响力分数。

### SetLattice（`set`）

```
join:   UNION (通过 array_agg 去重的数组合并)
bottom: {} (空集)
```

**用例：** 集合值溯源标注、多跳邻域集合。

### IntervalLattice（`interval`）

```
join:   区间包络 (下界取最大、上界取最大)
bottom: 空区间 (0)
```

**用例：** 时序推理、数值区间传播。

---

## 用户自定义格

可以注册自定义格，join 运算可以是任意已注册的 PostgreSQL 聚合函数：

```sql
-- 基于十进制权重的最小成本路由。
SELECT pg_ripple.create_lattice('route_cost', 'min', '1e308');

-- 自定义有界格（取值 0–100，join = LEAST）。
SELECT pg_ripple.create_lattice('reputation', 'min', '100');
```

`join_fn` 必须是一个已注册的 PostgreSQL 聚合函数（通过 `pg_proc` 校验）。如果注册时该函数尚不可见，会发出警告，但格仍会被存储——这允许在自定义聚合函数创建之前预先注册格。

---

## GUC 参数

| GUC | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `pg_ripple.lattice_max_iterations` | integer | 1000 | 触发错误码 PT540 警告并返回部分结果之前的最大不动点迭代次数。设为 0 表示不限制（不推荐）。 |

```sql
-- 修改迭代上限。
SET pg_ripple.lattice_max_iterations = 5000;

-- 查看当前设置。
SHOW pg_ripple.lattice_max_iterations;
```

---

## SQL 函数

### `pg_ripple.create_lattice(name, join_fn, bottom)` → `boolean`

在 `_pg_ripple.lattice_types` 目录表中注册一个新的格类型。

| 参数 | 类型 | 说明 |
|------|------|------|
| `name` | text | 唯一的格名称（区分大小写） |
| `join_fn` | text | PostgreSQL 聚合函数名 |
| `bottom` | text | 底元素（以文本字符串表示） |

新注册时返回 `true`，名称已存在时返回 `false`（幂等）。

```sql
SELECT pg_ripple.create_lattice('trust', 'min', '100');   -- true
SELECT pg_ripple.create_lattice('trust', 'min', '100');   -- false (幂等)
```

### `pg_ripple.list_lattices()` → `jsonb`

以 JSON 数组形式返回所有已注册的格类型（包括内置和用户自定义）。

```sql
SELECT jsonb_pretty(pg_ripple.list_lattices());
```

每个条目包含：`name`、`join_fn`、`bottom`、`builtin`。

### `pg_ripple.infer_lattice(rule_set, lattice_name)` → `jsonb`

对 `rule_set` 中的所有活动规则，使用指定的格运行单调不动点计算。

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `rule_set` | `'custom'` | 规则集名称，与 `load_rules()` 中使用的一致 |
| `lattice_name` | `'min'` | 用于头谓词 join 的格类型 |

返回 JSONB：

```json
{
  "derived":         42,
  "iterations":       5,
  "lattice":       "min",
  "rule_set":  "my_rules"
}
```

错误：
- `infer_lattice: unknown lattice type '...'` — 格未注册；请先调用 `create_lattice()`。
- `PT540` WARNING — 未在 `lattice_max_iterations` 内收敛。

---

## 目录表（Catalog table）

格类型存储在 `_pg_ripple.lattice_types` 中：

```sql
SELECT * FROM _pg_ripple.lattice_types;
```

| 列 | 类型 | 说明 |
|----|------|------|
| `name` | text | 主键；格标识符 |
| `join_fn` | text | PostgreSQL 聚合函数名 |
| `bottom` | text | 以文本表示的底元素 |
| `builtin` | boolean | 预注册的格为 true |
| `created_at` | timestamptz | 注册时间戳 |

---

## 完整示例：信任传播

本示例在社会图中传播最小信任分数。间接联系的可信度受限于路径上最弱的环节。

```sql
-- 1. 创建扩展并配置格。
CREATE EXTENSION IF NOT EXISTS pg_ripple;
SELECT pg_ripple.create_lattice('trust', 'min', '100');

-- 2. 插入直接信任关系（分数：0=不信任，100=完全信任）。
SELECT pg_ripple.load_ntriples($$
  <https://trust.example/alice> <https://trust.example/directTrust> "90"^^<xsd:integer> .
  <https://trust.example/bob>   <https://trust.example/directTrust> "70"^^<xsd:integer> .
  <https://trust.example/carol> <https://trust.example/directTrust> "85"^^<xsd:integer> .
  <https://trust.example/alice> <https://trust.example/knows>       <https://trust.example/bob> .
  <https://trust.example/bob>   <https://trust.example/knows>       <https://trust.example/carol> .
$$);

-- 3. 编写信任传播规则（使用 Datalog 语法）。
SELECT pg_ripple.load_rules($$
  ?y <https://trust.example/transitTrust> ?min_t :-
    ?x <https://trust.example/knows> ?y ,
    ?x <https://trust.example/directTrust> ?t1 ,
    ?y <https://trust.example/directTrust> ?t2 .
$$, 'trust_rules');

-- 4. 运行基于格的不动点计算。
SELECT pg_ripple.infer_lattice('trust_rules', 'trust');

-- 5. 查询传播后的信任值。
SELECT * FROM pg_ripple.sparql($$
  SELECT ?x ?t WHERE { ?x <https://trust.example/transitTrust> ?t }
$$);
```

---

## 错误码 PT540

**含义：** 格不动点未能在配置的迭代上限内收敛。

**触发：** 当超过 `pg_ripple.lattice_max_iterations` 时，以 PostgreSQL WARNING（而非 ERROR）发出。

**处理选项：**

1. 提高上限：
   ```sql
   SET pg_ripple.lattice_max_iterations = 10000;
   ```

2. 验证格是有限的：规则中使用的每个值域，从底元素出发可达的不同元素数量必须是有限的。

3. 验证单调性：规则体中的每个运算必须相对于格的序是单调的。递归规则中的非单调运算（例如否定）会破坏收敛保证。

---

## 与其他 pg_ripple 推理模式的关系

| 功能 | 分层要求 | 聚合 | 递归 |
|------|----------|------|------|
| `infer()` — 标准 Datalog | 需要分层 | 不支持 | 受限 |
| `infer_wfs()` — 良基语义（Well-Founded Semantics） | 无 | 不支持 | 完整 |
| `infer_lattice()` — Datalog^L | 无 | 单调格 join | 完整 |

当您需要带有收敛保证的递归聚合时使用 `infer_lattice()`，例如：最短路径、信任传播、集合可达性标注。

---

*自 v0.36.0 引入。*