# GUC 调优指南

本页面将每个 `pg_ripple` GUC 参数映射到工作负载特征。
将其用作调优部署的起点。

> 完整的 GUC 列表和默认值，请参阅[配置参考](configuration.md)。

---

## 工作负载类别调优矩阵

下表显示了五个常见部署配置文件的推荐 GUC 设置。
显示的值将覆盖默认值；省略该行以保留默认值。

| GUC | OLTP（写密集） | SPARQL 分析 | Datalog/推理 | 联邦 | 开发 |
|---|---|---|---|---|---|
| `htap_delta_max_rows` | `500 000` | `1 000 000` | `200 000` | `200 000` | `10 000` |
| `merge_interval_secs` | `30` | `300` | `60` | `120` | `10` |
| `auto_analyze` | `on` | `on` | `on` | `off` | `on` |
| `plan_cache_size` | `256` | `2 048` | `512` | `128` | `64` |
| `sparql_max_algebra_depth` | `64` | `512` | `256` | `128` | `256` |
| `sparql_max_triple_patterns` | `1 024` | `8 192` | `4 096` | `2 048` | `4 096` |
| `max_path_depth` | `32` | `128` | `64` | `32` | `64` |
| `export_batch_size` | `5 000` | `50 000` | `10 000` | `10 000` | `10 000` |
| `dictionary_cache_size` | `131 072` | `262 144` | `65 536` | `65 536` | `16 384` |
| `datalog_parallel_workers` | `1` | `1` | `4` | `1` | `1` |
| `datalog_parallel_threshold` | _(n/a)_ | _(n/a)_ | `10 000` | _(n/a)_ | _(n/a)_ |
| `tracing_exporter` | `none` | `none` | `none` | `otlp` | `stdout` |
| `tracing_otlp_endpoint` | _(n/a)_ | _(n/a)_ | _(n/a)_ | `http://jaeger:4317` | _(n/a)_ |

### 配置文件描述

**OLTP（写密集）**
高比率的连续三元组摄取。较短的合并间隔和较小的 delta 阈值
确保合并工作线程跟上进度。`plan_cache_size` 适中；
减少 `sparql_max_triple_patterns` 以拒绝意外的全扫描查询。

**SPARQL 分析**
针对大型、主要是静态的数据集的复杂 SELECT/CONSTRUCT 查询。大型计划缓存
减少了翻译开销。高 `sparql_max_triple_patterns` 允许复杂查询。
不频繁的合并减少了后台 I/O。

**Datalog/推理**
OWL RL/自定义规则物化。启用并行工作线程和高阈值以
并行化独立的层。保持 `merge_interval_secs` 适中 — 合并工作线程
与推理竞争 I/O。

**联邦**
大量使用 `SERVICE {}` 查询远程 SPARQL 端点。启用 OTLP 跟踪以
观察每个端点的延迟。减少本地 `plan_cache_size`，因为远程查询
在形状上有很大差异。

**开发**
本地开发人员机器。小型缓存、短合并间隔以获得快速反馈。
启用 `tracing_exporter = 'stdout'` 以在没有 APM 堆栈的情况下轻松调试。

---

## 安全限制

始终在生产环境中设置这些以防止失控或恶意查询：

```sql
-- 拒绝深度嵌套的代数树（例如，通过用户输入注入）
SET pg_ripple.sparql_max_algebra_depth = 256;

-- 拒绝具有不合理数量三元组模式的查询
SET pg_ripple.sparql_max_triple_patterns = 4096;

-- 限制递归属性路径深度
SET pg_ripple.max_path_depth = 64;
```

其他建议请参阅[安全强化](security.md)。

## HTTP 流式传输

流式安全的 SPARQL Results JSON、CSV、TSV 和 N-Triples 响应使用
一个已检出的只读 PostgreSQL 连接和有界的响应合并器。
调整 `PG_RIPPLE_HTTP_STREAM_CHUNK_BYTES` 以提高网络效率，
并在用户提供项时保持 `PG_RIPPLE_HTTP_STREAM_MAX_ROW_BYTES` 有界。
将 `PG_RIPPLE_HTTP_QUERY_TIMEOUT_MS` 设置在其 `900000` 毫秒最大值以下以适应租户
工作负载，并使用 `PG_RIPPLE_HTTP_STREAM_IDLE_TIMEOUT_MS` 释放慢速或
停滞的流。断开连接和截止日期到期会取消 PostgreSQL 查询；
当无法证明回滚或取消时，连接将被丢弃。

---

## 内存占用估算

| 组件 | 内存 | 公式 |
|---|---|---|
| 字典后端缓存 | ~`dictionary_cache_size × 80 B` | 每个后端连接 |
| SPARQL 计划缓存 | ~`plan_cache_size × 2 KB` | 每个后端连接 |
| Delta 表（HTAP） | ~`htap_delta_max_rows × 24 B` | 每个 VP 谓词 |
| 合并工作线程缓冲区 | ~`export_batch_size × 40 B` | 全局（一个工作线程）|

对于具有 50 个并发连接和默认 GUC 值的部署，
除了 PostgreSQL 的 `shared_buffers` 之外，
为 pg_ripple 自己的数据结构预算大约 **400 MB**。

---

## 已弃用的 GUC

| 旧名称 | 替代 | 移除于 |
|---|---|---|
| `pg_ripple.property_path_max_depth` | `pg_ripple.max_path_depth` | v1.0.0 |

今后请设置 `pg_ripple.max_path_depth`。旧名称仍被接受
但在服务器日志中发出弃用通知。

---

## 即用型配置配置文件

将这些块复制粘贴到 `postgresql.conf`（然后 `SELECT pg_reload_conf()`）。

### 小型实例配置文件（≤8 核，≤32 GB RAM）

适用于开发、暂存或小型生产部署：

```ini
# pg_ripple — 小型实例配置文件
pg_ripple.dictionary_cache_size     = 65536
pg_ripple.cache_budget_mb           = 64
pg_ripple.merge_threshold           = 10000
pg_ripple.merge_workers             = 1
pg_ripple.merge_interval_secs       = 60
pg_ripple.datalog_parallel_workers  = 2
pg_ripple.sparql_max_rows           = 50000
pg_ripple.export_max_rows           = 100000
pg_ripple.plan_cache_capacity       = 256
pg_ripple.federation_timeout        = 30
pg_ripple.federation_parallel_max   = 2
```

### 大型实例配置文件（≥32 核，≥128 GB RAM）

适用于大规模 SPARQL 分析、高吞吐量摄取或密集 Datalog 推理：

```ini
# pg_ripple — 大型实例配置文件
pg_ripple.dictionary_cache_size     = 1000000
pg_ripple.cache_budget_mb           = 512
pg_ripple.merge_threshold           = 50000
pg_ripple.merge_workers             = 4
pg_ripple.merge_interval_secs       = 120
pg_ripple.datalog_parallel_workers  = 8
pg_ripple.sparql_max_rows           = 500000
pg_ripple.export_max_rows           = 1000000
pg_ripple.plan_cache_capacity       = 2048
pg_ripple.federation_timeout        = 60
pg_ripple.federation_parallel_max   = 8
pg_ripple.pagerank_max_iterations   = 200
pg_ripple.topn_pushdown             = on
```

### 高安全性 / 多租户配置文件

为共享或多租户部署应用严格的资源限制：

```ini
# pg_ripple — 高安全性配置文件
pg_ripple.sparql_max_rows           = 10000
pg_ripple.sparql_overflow_action    = 'error'
pg_ripple.sparql_max_algebra_depth  = 128
pg_ripple.sparql_max_triple_patterns = 1024
pg_ripple.export_max_rows           = 10000
pg_ripple.federation_endpoint_policy = 'allowlist'
pg_ripple.federation_allow_unregistered_service_endpoints = off
pg_ripple.arrow_unsigned_tickets_allowed = off
pg_ripple.strict_sparql_filters     = on
pg_ripple.datalog_max_derived       = 1000000
pg_ripple.wfs_max_iterations        = 50
pg_ripple.rls_bypass                = off
```

### Datalog 推理配置文件

针对密集 OWL RL / Datalog 物化工作负载进行了优化：

```ini
# pg_ripple — datalog 推理配置文件
pg_ripple.inference_mode            = 'materialized'
pg_ripple.magic_sets                = on
pg_ripple.datalog_cost_reorder      = on
pg_ripple.dred_enabled              = on
pg_ripple.datalog_parallel_workers  = 8
pg_ripple.datalog_parallel_threshold = 5000
pg_ripple.owl_profile               = 'RL'
pg_ripple.tabling                   = on
pg_ripple.tabling_ttl               = 600
pg_ripple.rule_plan_cache           = on
pg_ripple.rule_plan_cache_size      = 128
pg_ripple.wfs_max_iterations        = 200
```
