# 可选功能降级语义

> **v0.64.0 TRUTH-10**：此页面记录 pg_ripple 中每个可选功能的预期降级行为。操作员可以使用 `pg_ripple.feature_status()` 在运行时检查实时状态。

---

## 概述

pg_ripple 区分**必需的核心功能**（始终处于活动状态）和依赖于外部扩展、配置或未来开发里程碑的**可选功能**。当可选功能不可用时，pg_ripple 会优雅地降级 — 它不会 panic，不会静默返回错误的结果，并且始终发出警告或结构化错误。

`pg_ripple.feature_status()` SQL 函数返回每个主要功能的一行，其中包含诚实的状态值。将其集成到您的监控管道中：

```sql
SELECT feature_name, status, degraded_reason
FROM pg_ripple.feature_status()
WHERE status IN ('degraded', 'stub', 'planned')
ORDER BY feature_name;
```

`pg_ripple_http /ready` 端点在其响应正文中包括一个 `partial_features` 数组，其中列出了所有非 `implemented` 的功能，以便操作员在路由生产流量之前评估就绪情况。

---

## 功能降级参考

### Arrow Flight（`arrow_flight`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `stub` |
| 缺少依赖项时的返回 | `{"status":"stub","message":"Arrow IPC streaming not yet implemented"}` |
| 警告代码 | 无（带有 stub 正文的 HTTP 200） |
| 就绪行为 | 在 `/ready` `partial_features` 中报告为 `stub` |
| 计划的实现 | v0.66.0 |

**详细**：`pg_ripple_http` 中的 `/flight/do_get` 端点返回 JSON stub 正文而不是 Arrow IPC 数据。这不是崩溃或错误 — 它是占位符，直到实现 Arrow IPC 序列化层。在 v0.66.0 之前，请勿依赖 Arrow Flight 输出用于生产数据管道。

---

### WCOJ / 最坏情况最优连接（`wcoj`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `planner_hint` |
| 行为 | 计划时的循环 BGP 连接重排序；无自定义执行器 |
| 警告代码 | 无 |
| 就绪行为 | 在 `/ready` 中报告为 `planner_hint` |
| 计划的实现 | v0.66.0（真正的 Leapfrog Triejoin 执行器） |

**详细**：WCOJ 被实现为 SPARQL 到 SQL 的规划器优化，它对循环模式连接进行重排序以减少中间结果大小。尚未实现在运行时对排序迭代器求交的真正 Leapfrog Triejoin 执行器。对于循环 BGP（例如三角形查询），pg_ripple 使用 PostgreSQL 的原生哈希连接或归并连接，对于高度循环的图可能比真正的 WCOJ 执行器慢。

---

### SHACL-SPARQL 规则（`shacl_sparql_rule`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `planned` |
| 调用时的返回 | 规则已存储但永不触发 |
| 警告代码 | 加载 `sh:SPARQLRule` 形状时的 `WARNING` 级别日志 |
| 就绪行为 | 在 `/ready` 中报告为 `planned` |
| 计划的实现 | v0.65.0 |

**详细**：`sh:SPARQLRule` 形状被解析、存储在 SHACL 目录中，并与其目标类关联。但是，它们不会被路由通过推导内核，也不会在推理或验证运行期间触发。对 SPARQL 支持的验证使用 `sh:SPARQLConstraint`（已实现）。

---

### CONSTRUCT 回写（`construct_writeback`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `manual_refresh` |
| 行为 | 规则仅在通过 `pg_ripple.apply_construct_rules()` 显式调用时应用 |
| 警告代码 | 无 |
| 就绪行为 | 在 `/ready` 中报告为 `manual_refresh` |
| 计划的实现 | v0.65.0（增量增量维护） |

**详细**：CONSTRUCT 回写规则是正确的 — 它们在调用时产生正确的输出。但是，它们在源数据更改时不会增量维护派生的三元组。在批量加载或 SPARQL UPDATE 之后，操作员必须手动调用 `pg_ripple.apply_construct_rules()` 来刷新派生的三元组。增量增量维护（在插入/删除时自动触发）计划在 v0.65.0 中提供。

---

### Citus SERVICE 修剪（`citus_service_pruning`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `planned` |
| 调用时的返回 | 查询执行但无分片修剪（正确但较慢） |
| 警告代码 | 无 |
| 就绪行为 | 在 `/ready` 中报告为 `planned` |
| 计划的实现 | v0.66.0 |

**详细**：SERVICE 结果分片修剪将 SPARQL SERVICE 调用路由到保存相关数据的特定 Citus 工作器分片，避免全集群扇出。这种优化是计划的，但尚未集成到 SPARQL 到 SQL 的翻译器中。查询返回正确的结果，但可能扫描比必要的更多分片。

---

### Citus HLL COUNT(DISTINCT)（`citus_hll_distinct`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `planned` |
| 调用时的返回 | COUNT(DISTINCT) 使用精确计数（正确但较慢） |
| 警告代码 | 无 |
| 就绪行为 | 在 `/ready` 中报告为 `planned` |
| 计划的实现 | v0.66.0 |

**详细**：HyperLogLog 近似 COUNT(DISTINCT) 使用概率草图来计算跨 Citus 分片的不同值，无需分片间协调。SQL 聚合生成层尚未发出 HLL 函数调用。COUNT(DISTINCT) 当前使用精确计数，这是正确的，但需要跨分片的数据移动。

---

### SPARQL Cursor 流（`sparql_cursor_streaming`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `planned` |
| 行为 | `/sparql/stream` 端点在流式传输之前具体化完整结果集 |
| 警告代码 | 无 |
| 就绪行为 | 在 `/ready` 中报告为 `planned` |
| 计划的实现 | v0.66.0 |

**详细**：`/sparql/stream` 端点将响应流式传输为分块传输编码，但在发送第一个块之前完整结果集已在内存中具体化。真正的增量流式传输（行从 PostgreSQL 到达时立即发送给客户端）计划在 v0.66.0 中提供。对于非常大的结果集，请改用 SQL 中的 `pg_ripple.sparql_cursor()`。

---

### 向量混合搜索（`vector_hybrid_search`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `experimental` |
| 依赖项 | `pgvector` 扩展 |
| 缺少 pgvector 时的返回 | 精确最近邻搜索（正确但较慢） |
| 警告代码 | `WARNING: pgvector not installed; falling back to exact similarity search` |
| 就绪行为 | 在 `/ready` 中报告为 `experimental`，带有 degraded_reason |

**详细**：安装 pgvector 后，混合搜索使用 HNSW 近似最近邻索引进行亚毫秒相似度搜索。没有 pgvector，pg_ripple 回退到精确余弦距离计算，这始终是正确的，但在嵌入数量上是 O(n)。为生产向量工作负载安装 pgvector。

---

### CDC 订阅（`cdc_subscriptions`）

| 属性 | 值 |
|---|---|
| v0.63.0 中的状态 | `experimental` |
| 依赖项 | 订阅目录注册无；仅 pg_tide 需要中继/outbox 传输 |
| 缺少 pg_tide 时的返回 | `create_subscription()` 仍记录订阅；中继依赖的传递路径不可用 |
| 就绪行为 | 订阅表面保持 pre-1.0 时，在 `/ready` 中报告为 `experimental` |

**详细**：`create_subscription()` 将订阅元数据写入 `_pg_ripple.subscriptions`，不需要 pg_trickle。将更改发布到外部系统的中继/outbox 管道需要 pg_tide；当 pg_tide 不存在时，这些中继路径会降级，但订阅目录 API 仍然可用。

---

## 在代码中检查功能状态

```sql
-- 列出所有非已实现的功能及其降级原因。
SELECT feature_name, status, degraded_reason
FROM pg_ripple.feature_status()
WHERE status != 'implemented'
ORDER BY status, feature_name;
```

```sql
-- 检查特定功能。
SELECT status, degraded_reason
FROM pg_ripple.feature_status()
WHERE feature_name = 'arrow_flight';
```

```bash
# 检查 /ready 是否有降级功能（pg_ripple_http）。
curl -s http://localhost:7878/ready | jq '.partial_features'
```

---

## 指标和日志可见性

可选功能降级通过以下方式可见：

1. **`pg_ripple.feature_status()`** — SQL 函数；每个功能一行
2. **`pg_ripple_http /ready`** — 包括 `partial_features` 数组
3. **PostgreSQL 日志** — 当可选扩展不存在且调用依赖于它们的功能时，发出 `WARNING` 级别消息
4. **Prometheus 指标** — 当 stub 功能返回错误响应时，`pg_ripple_http_errors_total` 计数器递增

---

*另请参阅：[功能状态 SQL API](../reference/sql-functions.md#feature-status)、[已知限制](https://github.com/trickle-labs/pg-ripple/blob/main/README.md#known-limitations-in-v0630)*