# 联邦性能

本页面涵盖了在 pg_ripple 中从 SPARQL `SERVICE` 查询获得最佳性能的实用策略。

## 选择缓存 TTL

`pg_ripple.federation_cache_ttl` 控制远程结果在重新查询端点之前的重用时长。合适的值取决于源数据变化的速度。

| 数据类型 | 建议 TTL |
|---|---|
| 缓慢变化的参考数据（Wikidata 标签、DBpedia 类别） | 3600–86400 秒（1 小时至 1 天） |
| 每日批处理数据（已发布的报告、快照） | 3600 秒（1 小时） |
| 近实时数据（新闻、股票价格） | 0（禁用） |
| 高度动态的数据（传感器流） | 0（禁用） |

```sql
-- 将 Wikidata 标签缓存 1 小时
SET pg_ripple.federation_cache_ttl = 3600;

-- 检查缓存命中率（需要注册表扩展）
SELECT url,
       COUNT(*) AS rows_cached,
       MIN(cached_at) AS oldest_entry,
       MAX(expires_at) AS latest_expiry
FROM _pg_ripple.federation_cache
GROUP BY url;
```

**提示：** 在运行批量联邦会话会话之前，在会话级别设置 `federation_cache_ttl`。对于需要新鲜度的交互式查询，请将其重置为 0。

## 设置复杂性提示

当单个查询联系多个端点时，设置复杂性提示，以便快速端点先运行。这减少了挂钟总时间，因为：

1. **快速端点尽早解析**，绑定可能剪除后续模式的变量。
2. **在慢端点的失败会被更快地检测到**。

```sql
SELECT pg_ripple.register_endpoint('https://fast-mirror.example.com/sparql', NULL, 'fast');
SELECT pg_ripple.register_endpoint('https://main-kb.example.com/sparql', NULL, 'normal');
SELECT pg_ripple.register_endpoint('https://archive.example.com/sparql', NULL, 'slow');
```

或在注册后更新：

```sql
SELECT pg_ripple.set_endpoint_complexity('https://archive.example.com/sparql', 'slow');
```

查看当前提示：

```sql
SELECT url, complexity, enabled
FROM pg_ripple.list_endpoints()
ORDER BY complexity, url;
```

## 为变量投影设计查询

pg_ripple 自动将 `SELECT ?v1 ?v2 … WHERE { … }` 而不是 `SELECT *` 发送到远程端点。为了最大程度地减少数据，只绑定外部查询需要的变量的模式：

```sparql
-- 效率较低：内部模式绑定 ?s、?p、?o、?label、?type、?comment
-- 但外部查询只需要 ?label
SERVICE <https://kb.example.com/sparql> {
  ?s ?p ?o .
  ?s <rdfs:label> ?label .
  ?s <rdf:type> ?type .
  ?s <rdfs:comment> ?comment .
}

-- 效率较高：内部模式只绑定 ?label
SERVICE <https://kb.example.com/sparql> {
  ?s <rdfs:label> ?label .
}
```

即使远程端点不遵守投影（仍然返回所有列），显式投影也会减少注入到本地 SQL 查询中的内联 VALUES 子句的大小。

## 使用 federation_health 进行监控

`_pg_ripple.federation_health` 表记录每个 SERVICE 调用的结果。使用它来识别慢或不稳定的端点：

```sql
-- 过去一小时内每个端点的延迟百分位
SELECT url,
       PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY latency_ms) AS p50_ms,
       PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms) AS p95_ms,
       PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms) AS p99_ms,
       COUNT(*) AS total_calls,
       ROUND(100.0 * AVG(CASE WHEN success THEN 1.0 ELSE 0.0 END), 1) AS success_pct
FROM _pg_ripple.federation_health
WHERE probed_at >= now() - INTERVAL '1 hour'
GROUP BY url
ORDER BY p95_ms DESC;
```

使用 `pg_ripple.federation_adaptive_timeout = on` 自动为快速端点收紧超时，并为慢速端点留出更多缓冲：

```sql
SET pg_ripple.federation_adaptive_timeout = on;
-- 有效超时 = max(1s, p95_latency * 3)。
-- p95 为 200ms 的端点获得 0.6s 超时（最小为 1s）。
-- p95 为 5000ms 的端点获得 15s 超时。
```

## 使用 federation_cache 进行监控

```sql
-- 查看哪些查询正在被缓存
SELECT url,
       query_hash,
       pg_size_pretty(octet_length(result_jsonb::text)) AS result_size,
       cached_at,
       expires_at,
       CASE WHEN expires_at > now() THEN 'active' ELSE 'expired' END AS status
FROM _pg_ripple.federation_cache
ORDER BY cached_at DESC;
```

过期行由合并后台工作线程自动清理。要立即逐出：

```sql
DELETE FROM _pg_ripple.federation_cache WHERE expires_at <= now();
```

## Sidecar 与进程内的权衡

pg_ripple_http sidecar（`pg_ripple_http/`）在异步 Tokio 运行时中执行联邦请求，能够在单个查询内实现真正的并行 HTTP。进程内 SPI 路径（本页面）是顺序的。

| 方法 | 延迟 | 并发 | 设置 |
|---|---|---|---|
| 进程内 SPI（默认） | 每次调用 +1–5ms 开销 | 顺序 | 无 |
| pg_ripple_http sidecar | 几乎零开销，异步 | 并行 | 部署 sidecar 二进制 |

对于具有 3+ 个独立 SERVICE 子句的工作负载，sidecar 可显著加快速度。对于 1–2 个子句或应用批量检测优化（同一端点）的情况，进程内路径就足够了。

## 连接池提示

线程本地连接池（`federation_pool_size`）在同一后端会话中的多个 SERVICE 调用之间重用 TCP 和 TLS 连接。每个 PostgreSQL 后端都有自己的池。

```sql
-- 为查询多个端点的会话增加池大小
SET pg_ripple.federation_pool_size = 16;

-- 对于单次批量作业保持为 1 以减少内存使用
SET pg_ripple.federation_pool_size = 1;
```

**注意：** 池在会话中首次使用时创建，并且当 `federation_pool_size` 更改时不会重新创建。要使新设置生效，请启动新会话。