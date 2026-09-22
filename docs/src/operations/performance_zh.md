# 性能调优

pg_ripple 性能取决于三个相互作用的子系统：查询引擎、写入路径和字典缓存。本页提供诊断步骤和调优配方。根据您自己保留的基线衡量变化。

---

## 三个瓶颈领域

```
┌──────────────────────────────────────────────────────┐
│                       性能                            │
│                                                      │
│   ┌──────────┐   ┌──────────┐   ┌──────────────┐    │
│   │  查询    │   │  写入    │   │  缓存        │    │
│   │  引擎    │   │  路径    │   │  压力        │    │
│   │          │   │          │   │              │    │
│   │ 慢速    │   │ 合并工作 │   │ 字典未命中 → │    │
│   │ SPARQL  │   │ 线程滞后 │   │   表查找     │    │
│   │ 查询    │   │ delta   │   │              │    │
│   │          │   │ 膨胀    │   │              │    │
│   └──────────┘   └──────────┘   └──────────────┘    │
└──────────────────────────────────────────────────────┘
```

---

## 诊断工作流程

在调优之前，确定哪个子系统是瓶颈：

```sql
-- 第 1 步：整体健康状况
SELECT pg_ripple.canary();

-- 第 2 步：缓存命中率
SELECT
    (s->>'encode_cache_hits')::bigint AS hits,
    (s->>'encode_cache_misses')::bigint AS misses,
    ROUND(
        (s->>'encode_cache_hits')::numeric /
        NULLIF((s->>'encode_cache_hits')::numeric + (s->>'encode_cache_misses')::numeric, 0),
        4
    ) AS hit_rate
FROM pg_ripple.stats() s;

-- 第 3 步：Delta 累积
SELECT (pg_ripple.stats()->>'unmerged_delta_rows')::int AS delta_rows;

-- 第 4 步：最慢的查询
SELECT calls, mean_exec_time::numeric(10,2) AS avg_ms, LEFT(query, 100)
FROM pg_stat_statements
WHERE query LIKE '%_pg_ripple.vp_%'
ORDER BY mean_exec_time DESC
LIMIT 10;
```

| 症状 | 可能的瓶颈 | 部分 |
|---|---|---|
| VP 查询的 `mean_exec_time` 高 | 查询引擎 | [查询性能](#query-performance) |
| `delta_rows` 无限增长 | 写入路径 / 合并 | [写入吞吐量](#write-throughput) |
| 缓存命中率 < 95% | 字典缓存 | [缓存压力](#cache-pressure) |
| `merge_worker_pid = 0` | 合并工作线程未运行 | [写入吞吐量](#write-throughput) |

---

## 查询性能

### 建立基线

代码库不发布生产规模数据集的当前延迟范围。在每次调优更改前后捕获 `EXPLAIN`、固定装置标识、硬件、软件版本和原始时间。

### 调优：慢速单个查询

**第 1 步：获取 EXPLAIN 输出**

```sql
SELECT pg_ripple.explain_sparql(
    'SELECT ?name WHERE {
        ?person <http://schema.org/knows> ?friend .
        ?friend <http://schema.org/name> ?name
    }',
    'text'
);
```

**第 2 步：检查常见问题**

| EXPLAIN 模式 | 问题 | 修复 |
|---|---|---|
| `vp_rare 上的 Seq Scan` | 谓词低于提升阈值 | 降低 `vp_promotion_threshold` 或加载更多数据 |
| 具有数百万行的 `Nested Loop` | 连接顺序不佳 | 验证 `bgp_reorder = on`；对 VP 表运行 `ANALYZE` |
| 大型结果上的 `Sort + Unique` | 不必要的 DISTINCT | 为功能谓词添加 SHACL `sh:maxCount 1` |
| 具有高循环的 `CTE Scan` | 无界属性路径 | 降低 `max_path_depth`；添加 FILTER 边界 |
| 具有大构建侧的 `Hash Join` | 在高基数谓词上连接 | 重写查询以首先过滤大谓词 |

**第 3 步：启用计划缓存**

```sql
-- 缓存重复查询的已编译 SQL
SET pg_ripple.plan_cache_size = 512;
```

计划缓存避免了重复 SPARQL 模式的重复解析、优化和 SQL 生成工作。根据观察到的工作集对其进行大小调整。

### 调优：整体查询吞吐量

对于具有许多并发查询的工作负载：

```ini
# 启用复杂连接的并行查询
pg_ripple.parallel_query_min_joins = 2

# PostgreSQL 并行执行
max_parallel_workers_per_gather = 4
max_parallel_workers = 8

# 复杂连接的更大 work_mem
work_mem = '128MB'
```

```admonish tip title="BGP 重排序"
默认情况下保持启用 `bgp_reorder`。仅当保留的计划和时间显示工作负载出现回归时才禁用它。
```

---

## 写入吞吐量

### 衡量写入性能

使用有界基准测试生产者进行冒烟测量，然后测试生产中使用的确切解析器、批大小、持久性设置和存储。代码库不发布当前生产规模的吞吐量范围。

### 调优：合并工作线程滞后

如果 `unmerged_delta_rows` 持续增长，则合并工作线程无法跟上写入速率。

**诊断：**

```sql
-- 检查 delta 累积
SELECT (pg_ripple.stats()->>'unmerged_delta_rows')::int AS delta;
-- 60 秒后再次运行 — 如果 delta 在增长，则合并滞后
```

**解决方案（按影响顺序）：**

1. **降低 merge_threshold** — 更频繁地合并较小的批次：
   ```sql
   ALTER SYSTEM SET pg_ripple.merge_threshold = 5000;
   SELECT pg_reload_conf();
   ```

2. **增加合并频率** — 减少轮询间隔：
   ```sql
   ALTER SYSTEM SET pg_ripple.merge_interval_secs = 15;
   SELECT pg_reload_conf();
   ```

3. **手动压缩** — 强制立即合并：
   ```sql
   SELECT pg_ripple.compact();
   ```

4. **单独的写入窗口** — 在非高峰时段批处理写入，然后压缩。

### 调优：批量加载性能

对于大型初始数据加载：

```sql
-- 暂时禁用 SHACL 验证
SET pg_ripple.shacl_mode = 'off';

-- 使用基于文件的加载以获得最佳吞吐量
SELECT pg_ripple.load_turtle_file('/data/large_dataset.ttl');

-- 重新启用验证
SET pg_ripple.shacl_mode = 'async';

-- 强制合并将数据移动到主表
SELECT pg_ripple.compact();
```

```admonish info title="缓存背压"
在批量加载期间，pg_ripple 根据 `cache_budget` 监控缓存利用率。当利用率超过 90% 时，会自动减少批大小以防止内存不足情况。如果您看到批量加载比预期慢，请检查 `stats()` 中的 `encode_cache_utilization_pct`。
```

---

## 缓存压力

### 诊断

```sql
SELECT
    (s->>'encode_cache_capacity')::int AS capacity,
    (s->>'encode_cache_utilization_pct')::int AS util_pct,
    (s->>'encode_cache_hits')::bigint AS hits,
    (s->>'encode_cache_misses')::bigint AS misses,
    (s->>'encode_cache_evictions')::bigint AS evictions
FROM pg_ripple.stats() s;
```

| 指标 | 健康 | 需要采取行动 |
|---|---|---|
| 命中率 > 95% | 正常运行 | 无 |
| 命中率 90–95% | 边缘 | 考虑增加缓存 |
| 命中率 < 90% | 缓存抖动 | 增加 `dictionary_cache_size` |
| 利用率 > 90% | 接近满 | 增加 `cache_budget` |
| 驱逐 > 命中次数的 10% | 高 churn | 工作集超过缓存 |

### 调整字典缓存大小

经验法则：缓存应至少保存 80% 的唯一项。

```sql
-- 计算唯一项数
SELECT count(*) AS unique_terms FROM _pg_ripple.dictionary;
```

| 唯一项 | 推荐的 `dictionary_cache_size` | 内存（约） |
|---|---|---|
| < 50K | 8,192 | ~2 MB |
| 50K – 500K | 65,536 | ~13 MB |
| 500K – 5M | 262,144 | ~50 MB |
| 5M – 50M | 500,000 | ~100 MB |
| > 50M | 1,000,000（最大）| ~200 MB |

```admonish warning title="需要重新启动"
更改 `dictionary_cache_size` 需要重新启动 PostgreSQL，因为在 postmaster 启动时分配共享内存。在初始部署期间计划您的缓存大小。
```

---

## 工作负载特定配方

### 读密集型分析

针对具有罕见写入的复杂 SPARQL 查询进行了优化：

```ini
# 用于多样化查询形状的大型计划缓存
pg_ripple.plan_cache_size = 2048

# BGP 优化
pg_ripple.bgp_reorder = on
pg_ripple.parallel_query_min_joins = 2

# 大型字典缓存
pg_ripple.dictionary_cache_size = 262144
pg_ripple.cache_budget = 256

# 不频繁的合并（写入很少）
pg_ripple.merge_threshold = 100000
pg_ripple.merge_interval_secs = 300

# PostgreSQL
shared_buffers = '4GB'
effective_cache_size = '12GB'
work_mem = '256MB'
random_page_cost = 1.1
```

记录产生的 P95 延迟并将其与您的服务级别目标进行比较。

### 写密集型摄取

针对具有周期性查询的连续数据摄取进行了优化：

```ini
# 较小的计划缓存（不同的查询更少）
pg_ripple.plan_cache_size = 64

# 积极的合并以保持 delta 较小
pg_ripple.merge_threshold = 5000
pg_ripple.merge_interval_secs = 10
pg_ripple.latch_trigger_threshold = 2000
pg_ripple.auto_analyze = on

# 大型缓存以处理编码压力
pg_ripple.dictionary_cache_size = 500000
pg_ripple.cache_budget = 512

# 摄取期间禁用 SHACL
pg_ripple.shacl_mode = 'off'

# PostgreSQL — 针对写入进行优化
shared_buffers = '2GB'
wal_buffers = '64MB'
checkpoint_completion_target = 0.9
max_wal_size = '4GB'
```

预期：50K+ 三元组/秒的持续摄取，合并滞后 < 30 秒。

### 混合 HTAP（读 + 写）

平衡并发查询和写入：

```ini
# 适中的计划缓存
pg_ripple.plan_cache_size = 512

# 平衡的合并 — 既不太频繁也不太懒惰
pg_ripple.merge_threshold = 25000
pg_ripple.merge_interval_secs = 30
pg_ripple.latch_trigger_threshold = 10000
pg_ripple.auto_analyze = on

# 良好的缓存覆盖
pg_ripple.dictionary_cache_size = 131072
pg_ripple.cache_budget = 128

# 异步 SHACL，这样写入就不会被阻塞
pg_ripple.shacl_mode = 'async'

# 用于读查询的 BGP 优化
pg_ripple.bgp_reorder = on

# PostgreSQL
shared_buffers = '4GB'
effective_cache_size = '12GB'
work_mem = '128MB'
max_parallel_workers_per_gather = 2
```

预期：读 P95 < 30ms，写入吞吐量 > 20K 三元组/秒，合并滞后 < 60 秒。

---

## 对您的部署进行基准测试

使用内置的 `compact()` 函数和 `pg_stat_statements` 建立基线：

```sql
-- 重置统计信息
SELECT pg_stat_statements_reset();

-- 运行您的工作负载（查询、插入等）

-- 收集结果
SELECT
    calls,
    mean_exec_time::numeric(10,2) AS avg_ms,
    stddev_exec_time::numeric(10,2) AS stddev_ms,
    min_exec_time::numeric(10,2) AS min_ms,
    max_exec_time::numeric(10,2) AS max_ms,
    rows,
    LEFT(query, 80) AS query_prefix
FROM pg_stat_statements
WHERE query LIKE '%_pg_ripple%'
ORDER BY total_exec_time DESC
LIMIT 20;
```

```admonish tip title="迭代调优"
一次更改一个参数，重新运行您的基准测试，然后进行比较。按影响顺序排列的最重要参数是：
1. `dictionary_cache_size`（缓存命中率）
2. `bgp_reorder`（查询规划）
3. `merge_threshold`（读取新鲜度与写入吞吐量）
4. `plan_cache_size`（重复查询开销）
5. PostgreSQL `work_mem`（复杂连接性能）
```
