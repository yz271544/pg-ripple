# PageRank 与图分析（v0.88.0+）

pg_ripple v0.88.0 添加了一个**Datalog 原生的 PageRank 引擎**，完全在 PostgreSQL 内部运行，通过 SPI 和现有的 Datalog 聚合基础设施。所有分数都持久化在 `_pg_ripple.pagerank_scores` 中，可以使用标准 SQL 查询 —— 无需外部进程。v0.90.0 添加了 WCOJ 集成、收敛范数 GUC、IVM 全重计算阈值和 Count-Min Sketch 近似 top-N。v0.91.0 添加了 IVM Prometheus gauge 和 `explain_pagerank_json()`。

## 快速入门

```sql
-- 加载一些 RDF 三元组
SELECT pg_ripple.load_ntriples($nt$
  <http://example.org/Alice> <http://xmlns.com/foaf/0.1/knows> <http://example.org/Bob> .
  <http://example.org/Bob>   <http://xmlns.com/foaf/0.1/knows> <http://example.org/Carol> .
  <http://example.org/Carol> <http://xmlns.com/foaf/0.1/knows> <http://example.org/Alice> .
$nt$, 'http://example.org/graph');

-- 运行 PageRank（默认：damping=0.85，100 次迭代）
SELECT node_iri, score
FROM pg_ripple.pagerank_run()
ORDER BY score DESC;
```

## SQL 函数

### `pg_ripple.pagerank_run(...)`

运行迭代 PageRank 并持久化结果。为每个节点返回一行。

| 参数 | 类型 | 默认值 | 说明 |
|-----------|------|---------|-------------|
| `damping` | `float8` | `0.85` | 阻尼因子（远程跳转概率 = 1 − damping） |
| `max_iterations` | `int` | `100` | 最大幂迭代步数 |
| `convergence_delta` | `float8` | `0.0001` | L1 范数收敛阈值 |
| `direction` | `text` | `'forward'` | `'forward'`、`'reverse'` 或 `'undirected'` |
| `topic` | `text` | `''` | 主题标签 —— 启用主题敏感 PageRank |
| `edge_predicates` | `text[]` | `NULL` | 限制为这些谓词 IRI（默认：所有） |
| `graph_uri` | `text` | `NULL` | 要计算的命名图（默认：所有图） |
| `decay_rate` | `float8` | `0.0` | 时间衰减指数（0 = 禁用） |
| `bias` | `float8` | `0.15` | 个性化向量权重 |

所有参数都是可选的，可以作为**命名参数**传递（API-03，v0.91.0）：

```sql
-- 命名参数调用（建议为清楚起见）
SELECT node_iri, score
FROM pg_ripple.pagerank_run(
    damping          => 0.85,
    max_iterations   => 50,
    topic            => 'citation'
);

-- 限制为特定命名图，反向
SELECT node_iri, score
FROM pg_ripple.pagerank_run(
    graph_uri  => 'http://example.org/graph1',
    direction  => 'reverse'
)
ORDER BY score DESC LIMIT 20;
```

### `pg_ripple.centrality_run(metric text, ...)`

计算替代中心性度量并将结果存储在 `_pg_ripple.centrality_scores` 中。

支持的 `metric` 值：`'betweenness'`、`'closeness'`、`'degree'`、`'pagerank'`。

```sql
SELECT COUNT(*) FROM pg_ripple.centrality_run('betweenness');
SELECT * FROM _pg_ripple.centrality_scores ORDER BY score DESC LIMIT 10;
```

### `pg_ripple.explain_pagerank(node_iri text, top_k int DEFAULT 5)`

返回节点的分数解释树 —— 其他节点贡献了多少。

```sql
SELECT depth, contributor_iri, contribution, path
FROM pg_ripple.explain_pagerank('<http://example.org/Alice>', 5);
```

### `pg_ripple.explain_pagerank_json(node_iri text, top_k int DEFAULT 5)` *(API-05，v0.91.0)*

返回与 `explain_pagerank()` 相同的解释树，但序列化为 JSONB 对象。当您需要从 PL/pgSQL 函数或 REST 处理程序返回解释时使用：

```sql
SELECT pg_ripple.explain_pagerank_json('<http://example.org/Alice>');
-- 返回 JSONB：{"node_iri": "...", "total_score": 0.023, "contributors": [...]}

-- 在 pg_ripple_http /pagerank/explain/:iri?format=json 的 REST 查询中有用
```

### `pg_ripple.export_pagerank(format text, top_k bigint DEFAULT 10000, topic text DEFAULT '')`

以 `'csv'`、`'turtle'`、`'ntriples'` 或 `'jsonld'` 格式序列化分数。

```sql
SELECT pg_ripple.export_pagerank('turtle', 1000);
```

### `pg_ripple.pagerank_queue_stats()`

返回用于监视的 IVM 脏边队列指标。

```sql
SELECT * FROM pg_ripple.pagerank_queue_stats();
-- queued_edges | max_delta | oldest_enqueue | estimated_drain_seconds
```

### `pg_ripple.vacuum_pagerank_dirty()`

从 `_pg_ripple.pagerank_dirty_edges` 中排出已处理的条目。

### `pg_ripple.pagerank_find_duplicates(metric text, centrality_threshold float8, fuzzy_threshold float8)`

返回通过中心性和标签相似性检测到的候选重复节点对。

## GUC 参数

| GUC | 默认值 | 说明 |
|-----|---------|-------------|
| `pg_ripple.pagerank_enabled` | `off` | 主开关 |
| `pg_ripple.pagerank_damping` | `0.85` | 阻尼因子 |
| `pg_ripple.pagerank_max_iterations` | `100` | 迭代上限 |
| `pg_ripple.pagerank_convergence_delta` | `0.0001` | 收敛阈值 |
| `pg_ripple.pagerank_convergence_norm` | `'l1'` | 收敛范数（`l1`、`l2`、`linf`） |
| `pg_ripple.pagerank_full_recompute_threshold` | `0.01` | 触发全重计算的过期分数 |
| `pg_ripple.pagerank_wcoj_threshold` | `10` | WCOJ 路径阈值（百万边） |
| `pg_ripple.pagerank_sketch_width` | `2000` | Count-Min Sketch 宽度（列） |
| `pg_ripple.pagerank_sketch_depth` | `5` | Count-Min Sketch 深度（行/哈希函数） |
| `pg_ripple.pagerank_temp_threshold` | `0` | 临时表阈值字节（0 = 自动） |
| `pg_ripple.pagerank_incremental` | `off` | 启用 IVM（k 跳刷新） |
| `pg_ripple.pagerank_confidence_weighted` | `off` | 按置信度分数加权边 |
| `pg_ripple.pagerank_partition` | `off` | 按命名图分区求值 |
| `pg_ripple.pagerank_probabilistic` | `off` | 概率 Datalog 分数边界 |
| `pg_ripple.pagerank_queue_warn_threshold` | `100000` | 脏边队列超过此值时发出警告 |

---

## 收敛范数（v0.90.0）

pg_ripple 使用 **L1 范数**（绝对差之和）来测试迭代之间的收敛，与 NetworkX 的默认行为一致。收敛阈值 `pg_ripple.pagerank_convergence_delta`（默认 0.0001）与分数增量向量的 L1 范数进行比较。

可以通过 `pg_ripple.pagerank_convergence_norm` GUC 选择替代范数：

| 值 | 公式 | 注释 |
|-------|---------|-------|
| `'l1'`（默认） | $\sum_i \|\Delta s_i\|$ | 计算最快；匹配 NetworkX |
| `'l2'` | $\sqrt{\sum_i \Delta s_i^2}$ | 匹配 igraph 默认 |
| `'linf'` | $\max_i \|\Delta s_i\|$ | 最保守；收敛最慢 |

```sql
-- 匹配 igraph 收敛行为
SET pg_ripple.pagerank_convergence_norm = 'l2';
SELECT node_iri, score FROM pg_ripple.pagerank_run();
```

---

## 增量刷新误差边界（v0.90.0）

有界 K 跳增量刷新近似于完整的 PageRank 重计算。K 跳增量传播后的理论误差边界为：

$$|\Delta \text{score}| \leq \alpha^K \times \max\_\text{delta\_per\_iteration}$$

其中 $\alpha$ 是阻尼因子（默认 0.85），$K$ 是 `pagerank_khop_limit`（默认 30）。在 K=30 时：

$$\alpha^{30} = 0.85^{30} \approx 0.0076$$

每个脏节点的最大误差是**每次迭代增量的 < 1%** —— 对于大多数用例是可接受的。对于高精度应用程序，请直接使用 `pagerank_run()`。

### 为您的图调优阻尼（v0.92.0）

默认阻尼因子 `pg_ripple.pagerank_damping = 0.85` 是为引用图和原始 Web 校准的（Brin & Page 1998）。对于其他图类型：

- **稀疏社交图**（Twitter 风格的关注）：建议 0.50–0.75。高阻尼在稀疏图中过度放大枢纽节点。
- **引用图**（论文引用论文）：0.85（默认）非常适合。
- **知识图谱**（多样化谓词类型）：0.85 是安全的默认值；考虑对密集谓词子集使用较低阻尼的按谓词过滤运行。
- **时间图** 带衰减（`decay_rate > 0`）：较低的阻尼（0.70–0.80）减少可能经过大幅时间折扣的远程链接的影响。

```sql
-- 为稀疏社交图调优
SET pg_ripple.pagerank_damping = 0.65;
SELECT node_iri, score FROM pg_ripple.pagerank_run();
```

### 自动全重计算

当特定主题的 `pagerank_scores` 中 `stale = true` 行的比例超过 `pg_ripple.pagerank_full_recompute_threshold`（默认 0.01 = 1%）时，下一个 IVM 工作进程周期会自动触发该主题的完整 `pagerank_run()`。

```sql
-- 较低阈值：当 0.5% 的分数过期时触发全重计算
SET pg_ripple.pagerank_full_recompute_threshold = 0.005;
```

---

## Count-Min Sketch 参数（v0.90.0）

top-K PageRank 查询路径使用 Count-Min Sketch 跟踪近似频率分布，而无需物化完整的分数表。两个 GUC 控制草图内存：

| GUC | 默认值 | 公式 |
|-----|---------|---------|
| `pg_ripple.pagerank_sketch_width` | `2000` | 每个哈希函数的列数 |
| `pg_ripple.pagerank_sketch_depth` | `5` | 哈希函数数量（行） |

内存使用量：每个活动主题 `width × depth × 8 字节`。使用默认值：`2000 × 5 × 8 = 80 KB` 每个主题 —— 即使是数千个主题也可以忽略。

**误差边界**：以概率 $1 - e^{-\text{depth}}$，频率估计最多高估总流量的 $e / \text{width}$。

```sql
-- 以更多内存为代价提高大图的准确性
SET pg_ripple.pagerank_sketch_width = 10000;
SET pg_ripple.pagerank_sketch_depth = 7;
```

## REST API（pg_ripple_http）

| 端点 | 方法 | 说明 |
|----------|--------|-------------|
| `/pagerank/run` | POST | 触发计算 |
| `/pagerank/results` | GET | Top-N 分数 |
| `/pagerank/status` | GET | 上次运行的元数据 |
| `/pagerank/vacuum-dirty` | POST | 排出队列 |
| `/pagerank/export` | GET | 导出分数 |
| `/pagerank/explain/{node_iri}` | GET | 分数解释 |
| `/pagerank/queue-stats` | GET | IVM 队列指标 |
| `/centrality/run` | POST | 计算中心性 |
| `/centrality/results` | GET | 中心性分数 |
| `/pagerank/find-duplicates` | POST | 实体去重 |

## 存储

分数持久化在 `_pg_ripple.pagerank_scores` 中：

```sql
\d _pg_ripple.pagerank_scores
-- node BIGINT, topic TEXT, score FLOAT8,
-- score_lower FLOAT8, score_upper FLOAT8,
-- computed_at TIMESTAMPTZ, iterations INT, converged BOOL,
-- stale BOOL, stale_since TIMESTAMPTZ
```

IVM 脏边队列（`_pg_ripple.pagerank_dirty_edges`）跟踪自上次完整运行以来哪些边已更改，从而启用 k 跳增量刷新（PR-TRICKLE-01）。

## `owl:sameAs` 去重（v0.92.0）

当 `owl:sameAs` 规范化处于活动状态（v0.23.0）时，pg_ripple 在计算 PageRank 之前将实体集群合并为规范代表。这种去重确保对等效实体的边不会被重复计算，否则会夸大具有大量 `sameAs` 集群的节点的 PageRank 分数。

有关规范化详情，请参阅[推理与演绎](reasoning-and-inference.md)。

## 可移植性说明（STD-04，v0.92.0）

`pg:pagerank()`、`pg:centrality()` 和 `pg:topN_approx()` 是**pg_ripple 特定的扩展函数**。它们不是由 W3C SPARQL 1.1 规范定义的，也不是可移植到其他 SPARQL 端点（Apache Jena、Virtuoso、Stardog 等）。

在联邦查询中，请使用完整的 IRI 形式并确保 pg_ripple 特定端点是 SERVICE 目标。在针对非 pg_ripple 端点的 SERVICE 块中包含 `pg:pagerank()` 的联邦查询将失败，并显示 `SPARQL function not found`。

## 功能状态

```sql
SELECT feature, status, version FROM pg_ripple.feature_status()
WHERE feature LIKE 'pagerank%' OR feature LIKE 'centrality%';
```

## 进一步阅读

- [博客：pg_ripple 中的 PageRank](https://github.com/trickle-labs/pg-ripple/blob/main/blog/pagerank.md) — 具有 IVM 和 Prometheus 集成的 Datalog 原生 PageRank