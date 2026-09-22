# SPARQL 性能

通过表操作、需求变换和规则计划缓存加速 pg_ripple 上的 SPARQL 查询的最佳实践。

---

## 表操作 / 记忆化（v0.32.0）

当 `pg_ripple.tabling = on`（默认）时，`infer_wfs()` 调用的结果存储在数据库内缓存中，键是目标字符串的 XXH3-64 哈希。对同一个规则集的重复调用返回缓存结果，无需重新运行不动点算法。

```sql
-- 检查当前的表操作设置。
SHOW pg_ripple.tabling;      -- on
SHOW pg_ripple.tabling_ttl;  -- 300（秒）

-- 检查缓存的内容。
SELECT * FROM pg_ripple.tabling_stats() ORDER BY hits DESC LIMIT 10;
```

### 表操作最有帮助的场景

- **分析工作负载**，其中同一规则集被重复评估（例如，仪表板刷新、批处理作业）。
- **针对推理三元组的 SPARQL 查询**，其中在每个查询之前调用 `infer_wfs()` 以确保最新的物化。
- **短期应用程序**，在同一会话中多次查询相同的模式。

### TTL 配置

```sql
-- 缓存条目永不过期（最适合读取密集、很少变化的数据）。
SET pg_ripple.tabling_ttl = 0;

-- 缓存 10 分钟。
SET pg_ripple.tabling_ttl = 600;

-- 完全禁用表操作（始终重新计算 — 对测试和调试有用）。
SET pg_ripple.tabling = off;
```

### 缓存失效

表操作缓存在以下情况下自动失效：

- `pg_ripple.insert_triple()` 或 `pg_ripple.delete_triple()` — 任何数据更改
- `pg_ripple.load_rules()` 或 `pg_ripple.drop_rules()` — 任何规则更改

正常使用不需要手动失效。

---

## 需求变换（v0.31.0）

`infer_demand()` 不是物化整个规则集闭包，而是仅推导回答特定目标模式集所需的事实。对于针对大型规则库中小的谓词集的 SPARQL 查询，这可以将推理工作量减少 50–90%。

```sql
-- 仅推导 'knows' 和 'reachable' — 跳过不相关的谓词。
SELECT pg_ripple.infer_demand('social_rules',
    '[{"p": "<https://ex.org/knows>"},
      {"p": "<https://ex.org/reachable>"}]'
);

-- 然后正常查询。
SELECT * FROM pg_ripple.sparql('
  SELECT ?a ?b WHERE { ?a <https://ex.org/reachable> ?b }
');
```

当 `pg_ripple.demand_transform = on`（默认）时，`create_datalog_view()` 在指定多个目标模式时自动应用需求变换。

---

## 规则计划缓存（v0.30.0）

规则计划缓存（`pg_ripple.rule_plan_cache = on`，默认）存储每个规则的已编译 SQL 计划，避免在同一会话中的重复 `infer()` 或 `infer_wfs()` 调用上的重复编译开销。

```sql
-- 检查计划缓存设置。
SHOW pg_ripple.rule_plan_cache;       -- on
SHOW pg_ripple.rule_plan_cache_size;  -- 64

-- 为调试禁用（每次强制重新编译）。
SET pg_ripple.rule_plan_cache = off;
```

规则计划缓存以规则文本为键，因此规则的更改（通过 `load_rules` 或 `drop_rules`）会自动使受影响的计划缓存条目失效。

### 与表操作的交互

规则计划缓存和表操作缓存在默认情况下都处于活动状态并相互补充：

- **规则计划缓存**消除 SQL 编译开销 — 每个不动点迭代重用每个规则的已编译 SQL。
- **表操作缓存**消除整个不动点计算 — 如果目标是最近计算的且数据未更改，则立即返回缓存结果。

对于长时间运行的服务，将 `pg_ripple.tabling_ttl = 0`（无过期）与 `pg_ripple.rule_plan_cache = on` 结合使用，可提供最佳的重复查询性能。

---

## SPARQL 属性路径和传递闭包

SPARQL 属性路径（例如 `+`、`*`、`/`）在查询时扩展为递归 CTE。对于大型图，这可能会很昂贵。考虑使用 Datalog 规则预先物化传递闭包：

```sql
-- 使用 Datalog 规则预先物化传递闭包。
SELECT pg_ripple.load_rules('
  ?x <https://ex.org/reachable> ?y :- ?x <https://ex.org/edge> ?y .
  ?x <https://ex.org/reachable> ?z :- ?x <https://ex.org/reachable> ?y ,
                                      ?y <https://ex.org/edge> ?z .
', 'reach');

SELECT pg_ripple.infer('reach');

-- 查询预先物化的闭包 — 查询时无递归 CTE。
SELECT * FROM pg_ripple.sparql('
  SELECT ?a ?b WHERE { ?a <https://ex.org/reachable> ?b }
');
```

结合表操作，此方法将传递闭包计算的开销分摊到多个查询上。

---

## 有界深度 SPARQL 属性路径（v0.34.0）

SPARQL 属性路径查询（`rdfs:subClassOf*`、`ex:knows+`）在内部依赖于 `WITH RECURSIVE` CTE。当图具有已知的有限层次结构深度时，使用深度边界预先物化闭包可以完全避免查询时的递归路径。

```sql
-- 物化有界闭包（最多 5 跳）
SET pg_ripple.datalog_max_depth = 5;
SELECT pg_ripple.load_rules(
  '?x <https://ex.org/reach> ?y :- ?x <https://ex.org/step> ?y . '
  '?x <https://ex.org/reach> ?z :- ?x <https://ex.org/reach> ?y , ?y <https://ex.org/step> ?z .',
  'bounded_reach'
);
SELECT pg_ripple.infer('bounded_reach');
SET pg_ripple.datalog_max_depth = 0;

-- 查询预先物化的有界闭包
SELECT * FROM pg_ripple.sparql('
  SELECT ?a ?b WHERE { ?a <https://ex.org/reach> ?b }
');
```

对于已知最大深度的层次结构（例如，从 SHACL `sh:maxDepth` 注释），此模式通常将属性路径查询延迟降低 30-60%，相对于无界内联递归 CTE。

---

## 针对循环模式的最坏情况最优连接（v0.36.0）

标准的哈希连接和嵌套循环算法对于*循环* SPARQL BGP（包含图形的查询图，包含循环）并非最坏情况最优，例如三角形查询：

```sparql
SELECT ?a ?b ?c WHERE {
    ?a <ex:knows> ?b .
    ?b <ex:knows> ?c .
    ?c <ex:knows> ?a .
}
```

当 `pg_ripple.wcoj_enabled = on`（默认）时，pg_ripple 自动检测循环 BGP 并强制 PostgreSQL 规划器走向排序合并连接，利用 VP 表上的 `(s, o)` B-tree 索引。这模拟了 Leapfrog Triejoin 算法的键局部性属性。

```sql
-- 检查 WCOJ 设置。
SHOW pg_ripple.wcoj_enabled;     -- on
SHOW pg_ripple.wcoj_min_tables;  -- 3（WCOJ 启动前的最小 VP 连接数）

-- 检测 BGP 是否循环（用于查询计划检查有用）。
SELECT pg_ripple.wcoj_is_cyclic('[["a","b"],["b","c"],["c","a"]]');  -- true
SELECT pg_ripple.wcoj_is_cyclic('[["root","a"],["root","b"]]');       -- false

-- 使用 WCOJ 开启/关闭对三角形查询进行基准测试。
SELECT pg_ripple.wcoj_triangle_query('https://example.org/knows');
-- 返回：{"triangle_count": N, "wcoj_applied": true, "predicate_iri": "..."}
```

### WCOJ 最有帮助的场景

- **社交图三角形查询** — 查找互相关注或共同合著者。
- **传递闭包模式** — 属性路径重写为连接链。
- **循环约束检查** — 检测有向图中的循环。

### 调优

```sql
-- 如果只想为限制专门的多跳连接提高 WCOJ 的阈值。
SET pg_ripple.wcoj_min_tables = 5;

-- 如果怀疑 WCOJ 导致错误计划，则全局禁用 WCOJ。
SET pg_ripple.wcoj_enabled = off;
```

> **性能预期：** 对于具有 1M 条边的 VP 表上的三角形查询，WCOJ 将查询时间从 > 10s（哈希连接计划）降低到 < 1s（具有 B-tree 开发的排序合并计划）。

---

## 并行推理后的物化新鲜度（v0.35.0）

当 `pg_ripple.datalog_parallel_workers > 1` 时，Datalog 引擎将规则划分为独立组，并在单个事务内按最佳顺序执行它们。在 `infer_with_stats()` 或 `infer()` 返回后，SPARQL 查询立即观察所有派生事实 — 同一会话内没有陈旧窗口。

```sql
-- 批量加载后，重新物化派生事实。
SELECT pg_ripple.load_turtle($$ <Alice> a <Person> . $$);
SET pg_ripple.datalog_parallel_workers = 4;
SELECT pg_ripple.infer_with_stats('owl-rl');

-- SPARQL 现在可以看到所有派生的 rdf:type、rdfs:subClassOf、owl:sameAs 事实。
SELECT pg_ripple.sparql('SELECT ?x ?type WHERE { ?x a ?type . }');
```

**提示：** 检查 `infer_with_stats()` 输出中的 `parallel_groups`，验证您的规则集是否受益于并行化。值为 1 表示所有规则都在单个依赖链中；值 > 1 确认可以并发执行。

```sql
-- 在调整工作线程之前检查并行组计数。
SELECT pg_ripple.infer_with_stats('owl-rl')->>'parallel_groups';  -- 例如："3"
```

---

## TopN 下推（v0.46.0）

当 SPARQL SELECT 查询同时包含 `ORDER BY` 和 `LIMIT N`（没有 `OFFSET` 也没有 `DISTINCT`）时，pg_ripple 将 `LIMIT N` 子句直接嵌入到生成的 SQL 中，而不是获取所有匹配行并在字典解码后丢弃多余的。这消除了解码永远不会返回的行的开销。</thinking>

### 下推适用的条件

| 条件 | 是否应用下推？ |
|---|---|
| `ORDER BY … LIMIT N` | 是 |
| `ORDER BY … LIMIT N OFFSET M`（M > 0） | 否（存在 OFFSET） |
| `SELECT DISTINCT … ORDER BY … LIMIT N` | 否（DISTINCT 在范围内） |
| `LIMIT N` 不带 `ORDER BY` | 否（无序） |

### 使用 EXPLAIN 验证下推

使用 `sparql_explain()` 确认下推已应用。在 JSON 输出中查找 `"topn_applied": true`：

```sql
SELECT pg_ripple.sparql_explain(
  'SELECT ?s ?score
   WHERE { ?s <http://example.org/score> ?score }
   ORDER BY DESC(?score)
   LIMIT 10',
  false
) -> 'topn_applied';
-- 返回：true
```

解释输出中的 `"plan"` 键将在下推处于活动状态时直接在 VP 扫描之上显示 `Limit` 节点。

### 禁用下推

下推由 `pg_ripple.topn_pushdown`（默认 `on`）控制。禁用它以进行调试或如果您怀疑结果不正确：

```sql
SET pg_ripple.topn_pushdown = off;
-- 运行查询
SET pg_ripple.topn_pushdown = on;
```

> **性能预期：** 在 1M 三元组的数据集上，带 `ORDER BY` 的 `LIMIT 10` 查询将字典解码调用从 O(result_set) 减少到 O(10)。当结果集相对于 LIMIT 较大时，改进最大。