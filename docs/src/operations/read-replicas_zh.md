# 只读副本

pg_ripple_http（v0.120.0+）支持将只读 SPARQL 查询路由到 PostgreSQL 只读副本。本页介绍路由语义、合格的查询类型、连接池耗尽行为和 Prometheus 警报配方。

## 配置

设置 `PG_RIPPLE_REPLICA_DATABASE_URL` 环境变量以将 HTTP 配套服务指向您的只读副本 PostgreSQL 实例：

```bash
PG_RIPPLE_REPLICA_DATABASE_URL="postgres://user:pass@replica-host:5432/rippledb"
```

当此变量不存在时，配套服务以仅主模式运行，所有查询都进入主池（`PG_RIPPLE_DATABASE_URL`）。

## 路由语义 — `?replica=ok`

将 `?replica=ok` 附加到任何 SPARQL HTTP 端点 URL 以请求副本路由：

```
GET /sparql?query=SELECT+…&replica=ok
```

配套服务根据以下决策树评估请求：

1. **合格的查询类型？** — 只有 `SELECT`、`CONSTRUCT`、`ASK` 和 `DESCRIBE` 查询可以路由到副本。无论是否有 `?replica=ok`，`UPDATE`（写入）查询始终进入主节点。
2. **是否配置了副本池？** — 如果 `PG_RIPPLE_REPLICA_DATABASE_URL` 未设置，则请求静默回退到主节点。
3. **池连接是否可用？** — 如果副本池已耗尽（所有连接都在使用中），则请求回退到主节点。不会向调用者返回错误；无论如何响应都是相同的。
4. **副本是否健康？** — 如果副本返回连接错误，则配套服务回退到主节点并递增 `pg_ripple_http_errors_total`。

## 池耗尽回退

当副本池耗尽时，查询自动回退到主节点。不会引发任何客户端可见的错误。这种行为是有意为之的：副本路由是性能优化，不是硬性要求。如果您需要强制仅副本执行（例如，用于容量分离的工作负载），请在负载均衡器或代理层实施该策略。

## Prometheus 指标（OBS-M-01）

HTTP 配套服务公开两个 Prometheus 指标以观察副本池：

| 指标 | 类型 | 描述 |
|--------|------|-------------|
| `pg_ripple_http_replica_pool_size{pool="replica"}` | gauge | 总连接池容量 |
| `pg_ripple_http_replica_pool_available{pool="replica"}` | gauge | 当前空闲（可用）连接数 |

每次 `/metrics` 抓取时都会更新这两个指标。当没有配置副本池时，两个值都为 `0`。

### 示例 Prometheus 警报规则

```yaml
groups:
  - name: pg_ripple_replica_pool
    rules:
      # 当副本池完全饱和时发出警报。
      - alert: ReplicaPoolExhausted
        expr: |
          pg_ripple_http_replica_pool_available{pool="replica"} == 0
          and pg_ripple_http_replica_pool_size{pool="replica"} > 0
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "pg_ripple read-replica pool exhausted"
          description: >
            All {{ $value }} replica pool connections are in use. Queries are
            falling back to the primary. Consider increasing the pool size or
            reducing query concurrency.

      # 当可用连接低于池容量的 20 % 时发出警报。
      - alert: ReplicaPoolLow
        expr: |
          (pg_ripple_http_replica_pool_available{pool="replica"}
            / pg_ripple_http_replica_pool_size{pool="replica"}) < 0.2
        for: 5m
        labels:
          severity: info
        annotations:
          summary: "pg_ripple replica pool running low"
          description: >
            Only {{ $value | humanizePercentage }} of replica pool connections
            are idle. Pool may become exhausted under load.
```

## 另请参阅

- [配置参考](../reference/guc-reference.md)
- [OpenTelemetry 可观察性](observability-otel.md)
- [兼容性矩阵](compatibility.md)