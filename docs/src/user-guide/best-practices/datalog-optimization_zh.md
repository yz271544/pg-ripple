# Datalog 优化

本指南帮助您充分发挥 pg_ripple Datalog 引擎的性能。v0.29.0 引入了若干性能特性来补充半朴素求值：魔集、基于代价的连接重排、反连接否定、谓词过滤下推以及 delta 表索引。

---

## 在 infer() 和 infer_goal() 之间选择

| 使用场景 | 推荐函数 |
|----------|---------|
| 为查询工作负载物化所有内容 | `infer_with_stats()` |
| 一次性针对特定目标的问题 | `infer_goal()` |
| 由推理支持的 SPARQL VIEW | 先 `infer_with_stats()`，再查询 VP 表 |
| 大型本体、选择性查询、冷缓存 | `infer_goal()` |

`infer_goal()` 在目标模式能够排除大部分可推导事实时表现出色。例如，询问"Alice 拥有哪些类型？"只需要 RDFS 闭包的一小部分。在 100 万三元组的数据集上，对于 5 层 `rdfs:subClassOf` 层次结构，单个实体的 `infer_goal()` 比完全物化化快约 100 倍。

当您希望预先物化完整的闭包，然后从 VP 存储中为多个查询提供服务而无需重新运行推理时，请使用 `infer_with_stats()`。

---

## 解读 infer_with_stats() 输出

```sql
SELECT pg_ripple.infer_with_stats('rdfs');
```

示例输出：
```json
{
  "derived": 42,
  "iterations": 3,
  "eliminated_rules": []
}
```

| 字段 | 含义 |
|-------|------|
| `derived` | 插入的新三元组总数；0 表示已达到不动点 |
| `iterations` | 不动点深度；等于最长推导链的长度 |
| `eliminated_rules` | 在求值前通过子句消除移除的规则；减少每次迭代的 SQL 语句数量 |

**迭代次数很高？** 引擎需要多次迭代才能使关系饱和。这在深度传递性层次结构中是正常的。要调查问题，可以增加日志：`SET pg_ripple.inference_mode = 'on_demand'`，并查看生成的半朴素求值 SQL 的 `EXPLAIN ANALYZE` 输出（可通过 `list_rules()` 获取）。

**`eliminated_rules` 非空？** 这些规则在给定规则集中其他规则的前提下被证明是冗余的。无需采取任何操作；这是一项免费的优化。

---

## 诊断缓慢的不动点收敛

### 步骤 1：检查迭代次数

```sql
SELECT pg_ripple.infer_with_stats('my-rules')->>'iterations';
```

迭代次数超过 20 通常意味着以下之一：
- 深度递归链（对于层次数据是预期的）
- 包含许多交叉引用规则的规则集（考虑拆分为更细粒度的规则集）

### 步骤 2：检查 VP 表的基数

```sql
SELECT relname, reltuples::bigint AS estimated_rows
FROM pg_class
WHERE relname LIKE 'vp_%'
ORDER BY reltuples DESC;
```

规则体中谓词对应的 VP 表过大，会使每次迭代变慢。使用 `infer_goal()` 来限定范围。

### 步骤 3：强制对 VP 表运行 ANALYZE

基于代价的重排（`pg_ripple.datalog_cost_reorder`）使用 `pg_class.reltuples`。如果这些统计信息过时，重排效果可能不佳：

```sql
-- 刷新所有 VP 表的统计信息
ANALYZE;
```

### 步骤 4：检查 delta 表索引的创建

```sql
-- 临时降低阈值以为所有 delta 表建立索引
SET pg_ripple.delta_index_threshold = 1;
SELECT pg_ripple.infer_with_stats('my-rules');
```

如果索引有所帮助，您可以为工作负载永久降低该阈值。

---

## 按数据集大小调整 GUC

| 数据集大小 | 推荐设置 |
|------------|---------|
| < 10K triples | 默认设置即可 |
| 10K–500K triples | `delta_index_threshold = 100`；`datalog_antijoin_threshold = 500` |
| 500K–10M triples | `delta_index_threshold = 50`；为选择性查询启用 `magic_sets = true` |
| > 10M triples | 对选择性查询使用 `infer_goal()`；`magic_sets = true`；考虑分区 VP 表 |

---

## 魔集 GUC

`pg_ripple.magic_sets`（默认：`true`）控制 `infer_goal()` 是否应用魔集变换，或者回退到完全自底向上求值。

设置为 `false` 进行调试：两条路径应返回相同的 `matching` 计数；如果不同，则魔集实现存在 bug——请报告。

```sql
-- 调试：比较魔集与完全求值
SET pg_ripple.magic_sets = true;
SELECT pg_ripple.infer_goal('rdfs', '?x rdf:type foaf:Person') AS magic_result;

SET pg_ripple.magic_sets = false;
SELECT pg_ripple.infer_goal('rdfs', '?x rdf:type foaf:Person') AS full_result;
-- 两个结果中的 "matching" 字段应相同
```

---

## 反连接否定

对于规则体中包含 `NOT` 的规则，编译器使用以下两种方式之一：

- **NOT EXISTS** — 适用于较小的 VP 表（有利于规划器的嵌套循环消除）
- **LEFT JOIN … IS NULL**（反连接）— 适用于较大的 VP 表（允许哈希反连接和合并反连接计划）

阈值由 `pg_ripple.datalog_antijoin_threshold` 控制（默认：1000 行）。

```sql
-- 强制对所有否定原子使用反连接，无论表大小如何
SET pg_ripple.datalog_antijoin_threshold = 1;

-- 强制对所有否定原子使用 NOT EXISTS
SET pg_ripple.datalog_antijoin_threshold = 0;
```

实际上，默认值（1000）与 PostgreSQL 自身关于何时哈希反连接有益的规划器启发式相匹配。

---

## 基准测试：魔集与完全物化

包含的 `benchmarks/magic_sets.sql` 文件演示了在大型类层次结构上的 RDFS 闭包中 `infer()` 与 `infer_goal()` 之间的性能差异。

运行方式：
```bash
cargo pgrx run pg18
# 在 psql 中：
\i benchmarks/magic_sets.sql
```

---

## 聚合规则 — 分层与性能（v0.30.0）

### 聚合分层

聚合字面量（`COUNT`、`SUM`、`MIN`、`MAX`、`AVG`）增加一个**分层约束**：聚合规则必须在其分组所涉及的所有数据完全物化之后才能求值。pg_ripple 通过其基于 SCC 的分层器自动强制执行此约束。如果检测到通过聚合形成的循环（例如，派生谓词 P 馈入产生另一谓词 Q 的聚合，而 Q 又馈入 P），引擎会发出 `WARNING PT510` 并跳过聚合规则。

**避免通过聚合形成的循环：**

```sql
-- ✗ 错误：foaf:knows 由规则 1 派生，但规则 2 对 foaf:knows 进行聚合。
--   如果聚合结果反馈到 foaf:knows，则违反 PT510。
SELECT pg_ripple.load_rules(
  '?x <foaf:knows> ?y :- ?x <ex:follows> ?y .
   ?x <ex:followCount> ?n :- COUNT(?y WHERE ?x <foaf:knows> ?y) = ?n .
   ?x <ex:follows> ?y :- ?x <ex:followCount> ?n , ?n > 1 .', -- 循环！
  'bad_set');

-- ✓ 正确：仅对基础数据进行聚合；结果是没有反引后继的新谓词。
SELECT pg_ripple.load_rules(
  '?x <ex:followCount> ?n :- COUNT(?y WHERE ?x <ex:follows> ?y) = ?n .',
  'good_set');
```

### 聚合规则的性能提示

1. **对包含聚合字面量的规则集运行 `infer_agg()` 而非 `infer()`**。`infer()` 会静默跳过聚合字面量；`infer_agg()` 会对其进行求值。

2. **计划缓存命中率**：在热缓存上，第二次及后续的 `infer_agg()` 调用会完全跳过编译。检查命中率：

   ```sql
   SELECT * FROM pg_ripple.rule_plan_cache_stats();
   -- rule_set     | hits | misses | entries
   -- my_analytics |    9 |      1 |       1
   ```

   如果命中率 < 90%，可能表明 `load_rules()` 被不必要地调用（每次 `load_rules()` 都会使该规则集的缓存失效）。

3. **为聚合原子使用窄谓词**：基于 `COUNT(?y WHERE ?x <ex:knows> ?y)` 会扫描 `ex:knows` VP 表。确保该谓词在 `(s, o)` 上有 B-tree 索引。

4. **将聚合规则批量放入单个规则集中**：同一规则集的多个聚合规则在单次 `infer_agg()` 调用中一起编译；将它们拆分为多个规则集会成倍增加 GROUP BY 查询的数量。

---

## 规则计划缓存调优（v0.30.0）

计划缓存可避免每次 `infer_agg()` 调用时重新编译规则 SQL。由两个 GUC 控制：

| GUC | 默认值 | 效果 |
|-----|--------|------|
| `pg_ripple.rule_plan_cache` | `true` | 主开关 — 设置为 `false` 以调试缓存相关问题 |
| `pg_ripple.rule_plan_cache_size` | `64` | 最大缓存规则集数；溢出时淘汰最旧的条目 |

**容量调整指南：**

- 如果您的应用程序少于 64 个规则集（典型情况），默认值即可。
- 对于 > 64 个规则集，增加 `rule_plan_cache_size` 以避免持续淘汰：
  ```sql
  ALTER SYSTEM SET pg_ripple.rule_plan_cache_size = 256;
  SELECT pg_reload_conf();
  ```
- 内存开销较低：每个缓存条目存储几个 SQL 字符串（通常约 1–5 KB）。

**缓存失效：**

缓存在以下情况下针对每个规则集自动失效：
- 针对该规则集调用 `pg_ripple.load_rules()`（新规则可能更改已编译的 SQL）
- 针对该规则集调用 `pg_ripple.drop_rules()`

缓存在各后端之间**不**共享（其是进程本地的）。每个新的后端连接都以空缓存启动，因此每个后端的首次 `infer_agg()` 调用总是会产生编译步骤。

---

## 需求变换与魔集（v0.31.0）

需求变换和魔集（`infer_goal()`）都是目标制导的推理技术，仅推导回答查询所需的事实。它们的范围有所不同：

| 技术 | 函数 | 最适合 |
|-----------|------|--------|
| 魔集 | `infer_goal(rule_set, goal)` | 单个目标谓词，一个特定的目标模式 |
| 需求变换 | `infer_demand(rule_set, demands)` | 多个目标谓词，相互依赖的规则 |

### 何时使用 `infer_demand()` 而非 `infer_goal()`

在以下情况下使用 `infer_demand()`：

1. **一个查询中的多个派生谓词**：SPARQL 查询涉及多个共享公共基础谓词的派生谓词。`infer_demand()` 计算联合需求集，并在单次扫描中推导所有需要的事实。

2. **相互递归规则**：谓词 A 的规则引用谓词 B，而 B 又引用 A。魔集处理一个入口点；需求变换通过完整依赖图传播绑定需求。

3. **选择性分析**：您只需要部分派生谓词的结果，而不是完全物化。

```sql
-- 仅派生 "manager" 和 "department" 谓词，忽略不相关的 HR 谓词。
SELECT pg_ripple.infer_demand('hr_rules',
    '[{"p": "<https://hr.example.org/manager>"},
      {"p": "<https://hr.example.org/department>"}]'
);
```

### 在 `create_datalog_view()` 中自动应用

当 `pg_ripple.demand_transform = on`（默认）时，`create_datalog_view()` 在指定多个目标模式时会自动应用需求变换。这使得物化视图更具选择性。

将 `pg_ripple.demand_transform = off` 设置为在视图定义内回退到完全推理。

### 需求变换与 `owl:sameAs`

当 `pg_ripple.sameas_reasoning = on`（默认）时，`infer_demand()` 在需求过滤的推理之前应用 `owl:sameAs` 规范化预处理。这确保即使涉及实体别名，结果也是正确的，同时仍将推理限制在最低所需的工作上。

---

## 良基语义与表操作（v0.32.0）

### 何时使用 `infer_wfs()`

在以下情况下使用 `infer_wfs()` 而非 `infer()`：

- 您的规则包含**相互否定**（通过 `NOT` 形成循环），`infer()` 会以分层错误拒绝。
- 您希望获得**三值结果**：无法解析的事实标记为 *unknown*（未知），而不是导致错误。
- 您需要对开放世界本体进行推理，其中事实的缺失并不等同于其否定。

对于纯正的或可分层的程序，`infer_wfs()` 会检测可分层性，并委托给与 `infer()` 相同的半朴素引擎——没有性能损失。

```sql
-- 测试规则集是否可分层，而无需承诺完全推理。
SELECT (pg_ripple.infer_wfs('my_rules') ->> 'stratifiable')::boolean AS stratifiable;
```

### 调整 WFS 迭代上限

GUC `pg_ripple.wfs_max_iterations`（默认 `100`）限制交替不动点的轮次。如果出现 WARNING PT520，请增加上限或检查规则中的非终止模式：

```sql
SET pg_ripple.wfs_max_iterations = 500;
SELECT pg_ripple.infer_wfs('large_ontology');
```

### 表操作调优

表操作缓存（`pg_ripple.tabling = on`）可避免在重复相同的调用时重新运行不动点。关键设置：

```sql
-- 为调试禁用表操作（始终重新计算）。
SET pg_ripple.tabling = off;

-- 将 TTL 设置为 10 分钟（默认是 5 分钟）。
SET pg_ripple.tabling_ttl = 600;

-- 将 TTL 设置为 0 表示条目永不过期。
SET pg_ripple.tabling_ttl = 0;

-- 检查缓存内容和命中率。
SELECT * FROM pg_ripple.tabling_stats() ORDER BY hits DESC;
```

缓存在数据更改（`insert_triple`、`delete_triple`）和规则更改（`load_rules`、`drop_rules`）时自动失效，无需手动管理缓存。

---

## 有界深度推理（v0.34.0）

当您的本体具有已知的最大层次结构深度时——例如，最多 5 层的类层次结构——您可以设置 `pg_ripple.datalog_max_depth` 以提前停止递归。这避免了运行最终的空不动点迭代，并可将时间受限层次结构的推理时间减少 20-50%。

```sql
-- 最多 5 层的属性层次结构
SET pg_ripple.datalog_max_depth = 5;
SELECT pg_ripple.infer('my_rules');
-- 在本事务之后重置为无限制
SET pg_ripple.datalog_max_depth = 0;
```

在最大深度未知时使用 `0`（默认）。将边界设置得过低会静默截断闭包。

## 删除时使用 DRed 与完全重算（v0.34.0）

默认情况下，删除基础三元组会触发 Delete-Rederive（DRed）算法：仅*可能*从已删除事实派生出的三元组会被过度删除，而具有其他派生路径的三元组会立即重新插入。这比丢弃并重新计算整个闭包要便宜得多。

| 场景 | 建议 |
|----------|------|
| 删除很少（< 写入的 1%） | `dred_enabled = true`（默认） |
| 批量删除数千个三元组 | `dred_enabled = false`，然后调用一次 `infer()` |
| 规则集频繁更改 | 使用 `add_rule()` / `remove_rule()` 进行手术式更新 |

```sql
-- 在批量删除期间禁用 DRed
SET pg_ripple.dred_enabled = false;
-- ... 批量删除 ...
SELECT pg_ripple.infer('my_rules');   -- 一次性完全重算
SET pg_ripple.dred_enabled = true;
```

---

## 并行层求值（v0.35.0）

在单个层内，派生*不同*谓词且没有共享体依赖的规则是完全独立的——它们的 `INSERT … SELECT` 语句涉及不同的 VP 表。pg_ripple 在推理时分析这种依赖结构，并将规则划分为并行组。`infer_with_stats()` 函数报告此分析。

### 解读 infer_with_stats() 中的并行字段

```sql
SELECT pg_ripple.infer_with_stats('owl-rl');
```

示例输出：
```json
{
  "derived": 1240,
  "iterations": 4,
  "eliminated_rules": [],
  "parallel_groups": 3,
  "max_concurrent": 3
}
```

| 字段 | 含义 |
|-------|------|
| `parallel_groups` | 在规则集中检测到的独立规则组数 |
| `max_concurrent` | 有效的并发工作线程数：`min(parallel_groups, datalog_parallel_workers)` |

`parallel_groups` 值为 1 表示所有规则形成单个依赖链——不可能并行。值 > 1 表示可以并发执行的独立组。

### 调整 datalog_parallel_workers

| 硬件 | 推荐设置 |
|----------|---------|
| 单核或低内存实例 | `datalog_parallel_workers = 1`（串行） |
| 2–4 核服务器 | `datalog_parallel_workers = 2` |
| 8+ 核服务器 | `datalog_parallel_workers = 4`（默认） |
| 专用推理工作负载 | 设置为从 `infer_with_stats()` 获得的 `parallel_groups` 值 |

```sql
-- 在调整工作线程之前检查规则集有多少并行组。
SELECT pg_ripple.infer_with_stats('my_rules')->>'parallel_groups' AS groups;

-- 然后设置匹配的工作线程数（上限为 CPU 数 - 3，为系统工作线程保留）。
SET pg_ripple.datalog_parallel_workers = 4;
```

### 避免小规则集上的开销

并行分析步骤会增加少量开销（依赖图构建）。对于小规则集或数据集，此开销超过并行化带来的好处。使用 `datalog_parallel_threshold` 在估计总行数低于阈值时跳过分析：

```sql
-- 跳过小层（< 5000 行）的并行分析。
SET pg_ripple.datalog_parallel_threshold = 5000;

-- 无论层大小如何，始终分析。
SET pg_ripple.datalog_parallel_threshold = 0;
```

默认阈值 10,000 行可在典型开发数据集上消除开销，同时为生产规模的本体闭包启用并行化。

### SPARQL 物化新鲜度

并行求值减少了从批量 `insert_triple` 调用到完全物化状态的挂钟时间。在调用 `pg_ripple.infer()` 或 `pg_ripple.infer_with_stats()` 之后，针对派生 VP 表的 SPARQL 查询会立即观察到新派生的事实：

```sql
-- 插入新数据。
SELECT pg_ripple.load_turtle($$ ... $$);

-- 并行物化派生谓词。
SET pg_ripple.datalog_parallel_workers = 4;
SELECT pg_ripple.infer_with_stats('owl-rl');

-- SPARQL 查询现在可以看到所有派生的事实。
SELECT pg_ripple.sparql_query($$
    SELECT ?x ?type WHERE { ?x a ?type . }
$$);
```

陈旧窗口（数据到达与派生事实的查询可见性之间的间隔）按独立规则组数和可用工作线程数的比例缩小。