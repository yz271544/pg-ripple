# 可观测性参考

此页面是 pg_ripple 可观测性和监控功能的参考。

## 概述

pg_ripple 通过以下方式公开监控数据：
- `pg_ripple_http` 配套服务中的 Prometheus 指标端点（`/metrics`）
- OpenTelemetry (OTLP) 分布式跟踪
- 用于查询级统计的 `pg_stat_statements` 集成
- `explain_sparql()` 与 `analyze := true`，用于交互式查询调试
- `explain_datalog()` 用于 Datalog 推导计划可视化
- `explain_inference()` 用于派生树检查

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE '%observ%' OR feature_name LIKE '%explain%' OR feature_name LIKE '%monitor%';
```

## Prometheus 指标

`pg_ripple_http` 配套服务以 Prometheus 文本格式公开 `/metrics`。关键指标包括：

| 指标 | 描述 |
|---|---|
| `pg_ripple_sparql_queries_total` | 已执行的 SPARQL 查询总数 |
| `pg_ripple_sparql_query_duration_seconds` | 查询持续时间直方图 |
| `pg_ripple_triple_count` | 每个图的三元组总数 |
| `pg_ripple_merge_operations_total` | 后台合并操作 |
| `pg_ripple_dictionary_cache_hits_total` | 字典 LRU 缓存命中率 |
| `pg_ripple_construct_rule_firings_total` | CONSTRUCT 回写规则调用次数 |
| `pg_ripple_datalog_materialize_duration_seconds` | Datalog 推导墙钟时间 |

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.explain_sparql(query TEXT, analyze BOOLEAN) → TEXT` | SPARQL 查询的 JSON 解释计划 |
| `pg_ripple.explain_datalog(rule_set TEXT) → TEXT` | Datalog 规则集的执行计划 |
| `pg_ripple.explain_inference(rule_set TEXT, triple TEXT) → TEXT` | 特定三元组的派生树 |
| `pg_ripple.feature_status() → SETOF record` | 所有已实现功能的当前状态 |

## OpenTelemetry 跟踪

当设置 `pg_ripple.otlp_endpoint` 时，会为每个 SPARQL 查询执行导出 span。来自 `pg_ripple_http` 的 `traceparent` 标头将传播到扩展以进行端到端跟踪。

## 查询调试

`explain_sparql(query, analyze := true)` 执行查询并返回包含以下内容的 JSON 对象：
- 优化后的 SPARQL 代数树
- 生成的 SQL 计划（来自 `EXPLAIN ANALYZE`）
- 每个步骤的行数以及实际行与估计行
- 总墙钟时间

## 相关页面

- [审计日志](audit-log.md)
- [GUC 参考](guc-reference.md)
- [HTTP API 参考](http-api.md)
- [功能状态分类](feature-status-taxonomy.md)

---

## PostgreSQL 结构化日志（OBS-03，v0.91.0）

pg_ripple 使用 `pgrx::log!` 和 `pgrx::warning!` 进行 PostgreSQL 扩展内的所有诊断输出。这些会发出标准 PostgreSQL 日志消息，并显示在 PostgreSQL 服务器日志中。

当 `log_destination = jsonlog` 处于活动状态时（PostgreSQL 15+），PostgreSQL 本身将所有日志消息（包括来自 pg_ripple 的日志消息）序列化为以下格式的 JSON 对象：

```json
{
  "timestamp": "2026-05-03 10:00:00.123 UTC",
  "pid": 12345,
  "session_id": "abc123",
  "log_level": "LOG",
  "message": "pg_ripple merge worker: merged 10000 rows into vp_42_main"
}
```

**无重复字段**：pg_ripple 不发出自己的 JSON 日志包装器；所有结构化字段映射由 PostgreSQL 原生的 `log_destination = jsonlog` 设施处理。不存在双重序列化的风险。

要在 PostgreSQL 中启用 JSON 日志，请添加到 `postgresql.conf`：

```
log_destination = 'jsonlog'
logging_collector = on
log_directory = 'pg_log'
```

或者，对于组合的文本 + JSON 输出：

```
log_destination = 'stderr,jsonlog'
```