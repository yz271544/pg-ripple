# OpenTelemetry 可观察性指南

pg_ripple 为每个 SPARQL 查询、Datalog 推理运行、SHACL 验证和合并循环发出 OpenTelemetry span。
本页记录了 span 名称、属性和示例 Prometheus/Grafana 查询。

## 分布式跟踪（v0.61.0+）

从 v0.61.0 开始，`pg_ripple_http` HTTP 服务从
传入请求中提取 [W3C `traceparent`](https://www.w3.org/TR/trace-context/) 标头，
并通过 `pg_ripple.tracing_traceparent` 会话 GUC 转发到扩展。在该请求期间发出的每个 span
都标记有发起跟踪 ID，从而提供从负载均衡器通过 HTTP 服务到查询引擎的无中断跟踪。

### 启用 traceparent 传播

`pg_ripple_http` 服务自动读取 `traceparent` 标头。
无需配置。要验证传播是否有效，请发送一个
带有 `traceparent` 标头的请求并检查您的跟踪后端以获取关联的 span。

```bash
curl -H "traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01" \
     -H "Content-Type: application/sparql-query" \
     --data "SELECT * WHERE { ?s ?p ?o } LIMIT 10" \
     http://localhost:8080/sparql
```

## Span 名称参考

| Span 名称 | 描述 | 关键属性 |
|---|---|---|
| `sparql.select` | SPARQL SELECT 查询执行 | `query`、`result_count`、`duration_ms` |
| `sparql.update` | SPARQL UPDATE 执行 | `operation_type`、`triples_modified` |
| `sparql.ask` | SPARQL ASK 查询 | `result` |
| `sparql.construct` | SPARQL CONSTRUCT 查询 | `result_count` |
| `sparql.describe` | SPARQL DESCRIBE 查询 | `subject_iri` |
| `datalog.infer` | Datalog 推理运行 | `rule_set`、`new_triples`、`iterations` |
| `datalog.retract` | DRed 撤回运行 | `rule_set`、`retracted_triples` |
| `shacl.validate` | SHACL 验证通过 | `shapes_count`、`violations_count` |
| `shacl.rule.compile` | SHACL-AF 规则编译 | `rule_count`、`compiled_count` |
| `htap.merge` | HTAP 合并循环 | `predicate_id`、`rows_merged`、`duration_ms` |
| `federation.call` | 联邦 SPARQL 端点调用 | `endpoint`、`duration_ms`、`error` |
| `bulk_load` | N-Triples / Turtle 批量加载 | `format`、`triple_count`、`duration_ms` |

## 标准属性

所有 pg_ripple span 都包含以下属性：

| 属性 | 类型 | 描述 |
|---|---|---|
| `db.system` | string | 始终为 `"postgresql"` |
| `db.name` | string | PostgreSQL 数据库名称 |
| `pg_ripple.version` | string | 扩展版本（例如 `"0.61.0"`） |
| `pg_ripple.traceparent` | string | 从 HTTP 层转发的 W3C traceparent（如果已设置） |

## 示例 Prometheus 查询

### 查询延迟 p95（5 分钟窗口）

```promql
histogram_quantile(0.95,
  sum(rate(pg_ripple_span_duration_ms_bucket{span="sparql.select"}[5m])) by (le)
)
```

### 按端点的联邦错误率

```promql
sum(rate(pg_ripple_federation_errors_total[5m])) by (endpoint)
/
sum(rate(pg_ripple_federation_calls_total[5m])) by (endpoint)
```

### HTAP 合并吞吐量（行/秒）

```promql
sum(rate(pg_ripple_htap_merge_rows_total[1m]))
```

### SHACL 违规率

```promql
sum(rate(pg_ripple_shacl_violations_total[5m])) by (shape)
```

## Grafana 仪表板

示例 Grafana 仪表板 JSON 可在
`docs/fixtures/grafana_pg_ripple_dashboard.json` 获取。通过
**仪表板 → 导入 → 上传 JSON 文件** 导入它。

## 跟踪后端

pg_ripple 的 OTel 导出器支持任何 OTLP 兼容的后端：

| 后端 | 配置 |
|---|---|
| Datadog | 设置 `OTEL_EXPORTER_OTLP_ENDPOINT=https://trace.agent.datadoghq.com` |
| Honeycomb | 设置 `OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io` 和 `OTEL_EXPORTER_OTLP_HEADERS=x-honeycomb-team=<API_KEY>` |
| Grafana Tempo | 设置 `OTEL_EXPORTER_OTLP_ENDPOINT=http://tempo:4317` |
| Jaeger | 设置 `OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317` |

这些环境变量在启动时由 `pg_ripple_http` 服务读取。
