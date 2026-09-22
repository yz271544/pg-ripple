# 监控和可观察性

pg_ripple 通过 SQL 函数、PostgreSQL 的标准统计基础设施以及通过 `pg_ripple_http` 的 Prometheus 兼容指标提供内置监控。本页解释了要监控的内容、如何收集数据以及哪些阈值表示系统健康。

---

## pg_ripple.stats()

主要监控函数。返回带有关键指标的 JSONB 对象：

```sql
SELECT pg_ripple.stats();
```

### 输出字段

| 字段 | 类型 | 描述 |
|---|---|---|
| `total_triples` | int | 跨所有图的三元组总数（包括尚未合并的 delta 行） |
| `dedicated_predicates` | int | 具有自己的 VP 表的谓词数 |
| `htap_predicates` | int | 使用 HTAP delta/main 分割的谓词数 |
| `rare_triples` | int | 存储在合并的 `vp_rare` 表中的三元组 |
| `unmerged_delta_rows` | int | 跨所有 delta 表的总行数（来自共享内存计数器）。如果共享内存不可用则为 `-1` |
| `merge_worker_pid` | int | 后台合并工作线程的 PID。如果未运行则为 `0` |
| `live_statistics_enabled` | bool | pg_trickle 实时统计是否处于活动状态 |
| `encode_cache_capacity` | int | 共享编码缓存可以容纳的总条目数 |
| `encode_cache_utilization_pct` | int | 当前正在使用的缓存插槽的百分比 |
| `encode_cache_hits` | int | 自服务器启动以来的累积缓存命中数 |
| `encode_cache_misses` | int | 自服务器启动以来的累积缓存未命中数 |
| `encode_cache_evictions` | int | 累积驱逐数 |

### 示例输出

```json
{
  "total_triples": 4523891,
  "dedicated_predicates": 127,
  "htap_predicates": 127,
  "rare_triples": 2341,
  "unmerged_delta_rows": 8432,
  "merge_worker_pid": 12345,
  "live_statistics_enabled": false,
  "encode_cache_capacity": 65536,
  "encode_cache_utilization_pct": 72,
  "encode_cache_hits": 18934521,
  "encode_cache_misses": 234012,
  "encode_cache_evictions": 45123
}
```

### 计算缓存命中率

```sql
SELECT
    (s->>'encode_cache_hits')::bigint AS hits,
    (s->>'encode_cache_misses')::bigint AS misses,
    ROUND(
        (s->>'encode_cache_hits')::numeric /
        NULLIF((s->>'encode_cache_hits')::numeric + (s->>'encode_cache_misses')::numeric, 0),
        4
    ) AS hit_rate
FROM pg_ripple.stats() s;
```

```admonish warning title="缓存命中率阈值"
健康的系统应保持高于 **95%**（0.95）的缓存命中率。如果它下降到 90% 以下，增加 `pg_ripple.dictionary_cache_size` 并重新启动 PostgreSQL。持续低于 80% 表示工作集明显超过缓存容量。
```

---

## pg_ripple.canary()

一个健康检查函数，返回带有通过/失败指示符的 JSONB 对象：

```sql
SELECT pg_ripple.canary();
```

### 输出字段

| 字段 | 类型 | 健康值 | 描述 |
|---|---|---|---|
| `merge_worker` | text | `"ok"` | 如果合并工作线程 PID 在共享内存中则为 `"ok"`；否则为 `"stalled"` |
| `cache_hit_rate` | float | > 0.95 | 字典编码缓存命中率（0.0–1.0） |
| `catalog_consistent` | bool | `true` | `pg_class` 中的 VP 表计数与提升的谓词匹配 |
| `orphaned_rare_rows` | int | `0` | `vp_rare` 中针对已经具有专用 VP 表的谓词的行 |

### 解释结果

```sql
SELECT
    c->>'merge_worker' AS worker,
    (c->>'cache_hit_rate')::float AS hit_rate,
    (c->>'catalog_consistent')::bool AS catalog_ok,
    (c->>'orphaned_rare_rows')::int AS orphaned
FROM pg_ripple.canary() c;
```

```admonish tip title="使用 canary() 进行自动健康检查"
`canary()` 专为负载均衡器健康检查和监控系统而设计。定期调用它，并在 `merge_worker = 'stalled'`、`cache_hit_rate < 0.90` 或 `catalog_consistent = false` 时发出警报。
```

---

## 使用 sparql_explain() 进行 SPARQL 查询分析

使用 explain 函数分析 SPARQL 查询性能：

### 基本 SQL 生成

```sql
-- 查看生成的 SQL 而不执行
SELECT pg_ripple.sparql_explain(
    'SELECT ?name WHERE { ?s <http://schema.org/name> ?name }',
    false
);
```

### 完整的 EXPLAIN ANALYZE

```sql
-- 执行并显示时间 + 行数
SELECT pg_ripple.sparql_explain(
    'SELECT ?name WHERE { ?s <http://schema.org/name> ?name }',
    true
);
```

### 带有格式选项的 explain_sparql()

`explain_sparql()` 函数（v0.23.0）提供更多输出格式：

```sql
-- 仅生成的 SQL
SELECT pg_ripple.explain_sparql(
    'SELECT ?s ?o WHERE { ?s <http://xmlns.com/foaf/0.1/knows> ?o }',
    'sql'
);

-- EXPLAIN ANALYZE 作为文本（默认）
SELECT pg_ripple.explain_sparql(
    'SELECT ?s ?o WHERE { ?s <http://xmlns.com/foaf/0.1/knows> ?o }',
    'text'
);

-- EXPLAIN ANALYZE 作为 JSON（用于程序化消费）
SELECT pg_ripple.explain_sparql(
    'SELECT ?s ?o WHERE { ?s <http://xmlns.com/foaf/0.1/knows> ?o }',
    'json'
);

-- SPARQL 代数树（用于调试优化器）
SELECT pg_ripple.explain_sparql(
    'SELECT ?s ?o WHERE { ?s <http://xmlns.com/foaf/0.1/knows> ?o }',
    'sparql_algebra'
);
```

```admonish info title="在 EXPLAIN 输出中要注意的内容"
- **vp_rare 上的 Seq Scan** — 谓词尚未提升。考虑降低 `vp_promotion_threshold` 或加载更多数据。
- **具有高行估计的嵌套循环** — BGP 重排序可能不是最佳的。检查 `bgp_reorder` 是否打开。
- **具有高循环计数的递归 CTE** — 属性路径很深。检查 `max_path_depth` 设置。
- **排序 + 唯一** — 可能会通过 SHACL `sh:maxCount 1` 提示避免的 `DISTINCT`。
```

---

## pg_stat_statements 集成

pg_ripple 生成由 `pg_stat_statements` 跟踪的标准 SQL。这使您可以深入了解实际 SQL 性能：

```sql
-- 启用 pg_stat_statements（如果尚未启用）
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- 查找最慢的 SPARQL 生成的查询
SELECT
    calls,
    mean_exec_time::numeric(10,2) AS avg_ms,
    total_exec_time::numeric(10,2) AS total_ms,
    rows,
    LEFT(query, 120) AS query_prefix
FROM pg_stat_statements
WHERE query LIKE '%_pg_ripple.vp_%'
ORDER BY mean_exec_time DESC
LIMIT 20;
```

### 识别热 VP 表

```sql
-- 哪些 VP 表被扫描最多？
SELECT
    regexp_matches(query, '_pg_ripple\.(vp_\d+)', 'g') AS vp_table,
    sum(calls) AS total_calls,
    sum(total_exec_time)::numeric(10,2) AS total_ms
FROM pg_stat_statements
WHERE query LIKE '%_pg_ripple.vp_%'
GROUP BY 1
ORDER BY total_ms DESC
LIMIT 10;
```

---

## Prometheus 指标（pg_ripple_http）

`pg_ripple_http` 伴随服务在 `/metrics` 端点公开 Prometheus 兼容的指标：

```bash
curl http://localhost:7878/metrics
```

### `/metrics` 的身份验证模型（METRICS-AUTH-DOC-01）

> **按设计，`/metrics` 端点是未经身份验证的。** 即使配置了 `PG_RIPPLE_BEARER_TOKEN` 并且所有 SPARQL/Datalog/admin 路由都需要身份验证，它也不需要 `Authorization: Bearer` 令牌。

**理由**：Prometheus 抓取器通常在 `pg_ripple_http` 所在的同一专用网络内运行，不需要身份验证的抓取目标。公开有关查询率和错误计数的计数器和仪表数据不会泄露图数据或连接凭据。

**操作员指南**：
- 如果您将 `pg_ripple_http` 直接暴露在公共接口上，请将 `/metrics` 放在反向代理访问控制规则后面（例如，nginx 中的 `location /metrics { allow 10.0.0.0/8; deny all; }`）。
- 或者，使用专用的抓取网络接口（例如，`--metrics-bind 127.0.0.1:9090`），如果未来版本添加每个端点的绑定配置。
- `/metrics/extension` 端点（来自 PostgreSQL 扩展的流式计数器）遵循相同的未经身份验证模型。

**安全影响**：指标包含请求计数、错误计数和延迟总计 — 没有用户数据或凭据。指标负载没有 SSRF 或注入风险。

### 可用指标

| 指标 | 类型 | 描述 |
|---|---|---|
| `pg_ripple_http_sparql_queries_total` | counter | 已执行的 SPARQL 查询总数 |
| `pg_ripple_http_datalog_queries_total` | counter | Datalog API 调用总数 |
| `pg_ripple_http_errors_total` | counter | 查询错误总数 |
| `pg_ripple_http_query_duration_seconds_total` | counter | 累积查询执行时间 |
| `pg_ripple_http_pool_size` | gauge | 当前连接池大小 |

### Prometheus 抓取配置

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'pg_ripple_http'
    scrape_interval: 15s
    static_configs:
      - targets: ['pg-ripple-http:7878']
    metrics_path: /metrics
    # 不需要 bearer_token — /metrics 是未经身份验证的。
    # 在网络级别限制访问（请参阅上面的操作员指南）。
```

### 仪表板的派生指标

使用 PromQL 计算有用的速率：

```promql
# 每秒查询数
rate(pg_ripple_http_sparql_queries_total[5m])

# 错误率
rate(pg_ripple_http_errors_total[5m]) / rate(pg_ripple_http_sparql_queries_total[5m])

# 平均查询延迟
rate(pg_ripple_http_query_duration_seconds_total[5m]) / rate(pg_ripple_http_sparql_queries_total[5m])
```

---

## 监控合并工作线程

后台合并工作线程对于 HTAP 性能至关重要。通过多个渠道对其进行监控：

### 共享内存状态

```sql
-- 合并工作线程是否在运行？
SELECT (pg_ripple.stats()->>'merge_worker_pid')::int AS pid;
-- 如果未运行则返回 0
```

### Delta 表大小

```sql
-- 检查每个谓词的 delta 累积
SELECT
    p.id AS predicate_id,
    d.value AS predicate_iri,
    p.triple_count,
    (SELECT count(*) FROM format('_pg_ripple.vp_%s_delta', p.id)::regclass) AS delta_rows
FROM _pg_ripple.predicates p
JOIN _pg_ripple.dictionary d ON d.id = p.id
WHERE p.htap = true
ORDER BY p.triple_count DESC
LIMIT 10;
```

### 合并工作线程日志

合并工作线程记录到 PostgreSQL 的标准日志：

```
LOG:  pg_ripple merge worker: merge cycle complete
LOG:  pg_ripple merge worker: processed 3 async validation item(s)
WARNING:  pg_ripple merge worker: watchdog timeout (300s)
```

```admonish warning title="看门狗超时"
如果您在 PostgreSQL 日志中看到 `watchdog timeout` 警告，则合并工作线程已停止。常见原因：
- 在 VP 表上持有锁的长时间运行的事务
- `worker_database` 指向错误的数据库
- `postgresql.conf` 中的 `max_worker_processes` 不足
```

---

## 健康检查阈值

使用这些阈值进行告警：

| 指标 | 绿色 | 黄色 | 红色 |
|---|---|---|---|
| 缓存命中率 | > 95% | 90–95% | < 90% |
| 合并工作线程 PID | > 0 | — | = 0 |
| Delta 行数（总数） | < 2× merge_threshold | 2–5× | > 5× |
| 目录一致 | true | — | false |
| 孤立稀有行数 | 0 | 1–100 | > 100 |
| 查询错误率 | < 1% | 1–5% | > 5% |
| 平均查询延迟 | < 100ms | 100–500ms | > 500ms |

### 自动监控查询

从您的监控系统定期运行此查询：

```sql
SELECT
    CASE
        WHEN (c->>'merge_worker') = 'ok'
             AND (c->>'cache_hit_rate')::float > 0.90
             AND (c->>'catalog_consistent')::bool
             AND (c->>'orphaned_rare_rows')::int = 0
        THEN 'healthy'
        WHEN (c->>'merge_worker') = 'stalled'
             OR (c->>'cache_hit_rate')::float < 0.80
        THEN 'critical'
        ELSE 'warning'
    END AS status,
    c->>'merge_worker' AS worker,
    c->>'cache_hit_rate' AS hit_rate,
    c->>'catalog_consistent' AS catalog,
    c->>'orphaned_rare_rows' AS orphaned
FROM pg_ripple.canary() c;
```

---

## 谓词清单

监控谓词分布以发现不平衡：

```sql
SELECT
    p.id,
    d.value AS predicate_iri,
    p.triple_count,
    p.table_oid IS NOT NULL AS has_vp_table,
    CASE WHEN p.htap THEN 'htap' ELSE 'flat' END AS storage_mode
FROM _pg_ripple.predicates p
JOIN _pg_ripple.dictionary d ON d.id = p.id
ORDER BY p.triple_count DESC
LIMIT 20;
```

```admonish tip title="偏斜的谓词"
如果一个谓词的三元组数比下一个多 10 倍，则其 VP 表主导存储和合并时间。考虑按命名图对数据进行分区或过滤查询以避免该谓词的全扫描。
```

---

## 基于日志的监控

为 SPARQL 工作负载可见性配置 PostgreSQL 日志记录：

```ini
# postgresql.conf
log_min_duration_statement = 500    # 记录慢于 500 毫秒的查询
log_statement = 'none'              # 不记录每个语句
log_line_prefix = '%t [%p] %d '     # 时间戳、PID、数据库
```

SPARQL 生成的 SQL 出现在 PostgreSQL 日志中，带有 VP 表引用，便于将慢速日志条目与特定 SPARQL 模式相关联。
