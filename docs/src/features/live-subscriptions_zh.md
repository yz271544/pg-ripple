# 实时 SPARQL 订阅

**状态**：实验性 — v0.73.0（SUB-01）  
**API**：`pg_ripple.subscribe_sparql()` / `pg_ripple.unsubscribe_sparql()`  
**HTTP**：`GET /subscribe/:subscription_id`（Server-Sent Events）  
**另请参阅**：[CDC 订阅](cdc-subscriptions.md) · [实时视图](live-views-and-subscriptions.md)  

---

## 概述

实时 SPARQL 订阅允许应用程序在 SPARQL SELECT 查询的结果更改时接收实时通知。每当图写入或删除影响已注册的子图监视的图时，pg_ripple 调用 `pg_notify('pg_ripple_subscription_<id>', ...)`，并将更新的查询结果作为 JSONB 负载发送。

`pg_ripple_http` 配套服务公开一个 `GET /subscribe/:id` 端点，该端点将 PostgreSQL `LISTEN` 通知转换为任何 HTTP 客户端都可以使用的 Server-Sent Events（SSE）。

---

## 快速入门

### 1. 注册订阅

```sql
-- 注册一个订阅，每当向默认图写入三元组并且查询产生不同的结果时触发。
SELECT pg_ripple.subscribe_sparql(
    'my-sub-01',
    'SELECT ?s ?label WHERE { ?s <https://schema.org/name> ?label }',
    NULL   -- NULL = 监视所有图
);
```

### 2. 在任何 PostgreSQL 客户端中侦听更改

```sql
LISTEN pg_ripple_subscription_my-sub-01;
-- 每个 NOTIFY 负载都是更新结果的 JSONB 字符串。
```

### 3. 通过 HTTP SSE 侦听

```bash
curl -N http://localhost:7878/subscribe/my-sub-01 \
     -H "Authorization: Bearer $TOKEN"
```

每个 Server-Sent Event 都具有 `event: sparql_result` 和 `data: <jsonb>` 字段。

### 4. 取消注册

```sql
SELECT pg_ripple.unsubscribe_sparql('my-sub-01');
```

---

## SQL API 参考

### `subscribe_sparql(subscription_id, query, graph_iri)`

```sql
pg_ripple.subscribe_sparql(
    subscription_id TEXT,
    query           TEXT,
    graph_iri       TEXT DEFAULT NULL
) RETURNS VOID
```

注册订阅。如果具有相同 ID 的订阅已存在，则引发错误。

- `subscription_id` — 唯一标识符；在通道名称 `pg_ripple_subscription_<id>` 中使用。
- `query` — 更改时要重新评估的 SPARQL SELECT 查询。
- `graph_iri` — 如果设置，则仅当此命名图被写入或删除时订阅才会触发。`NULL` 会因任何图写入而触发。

### `unsubscribe_sparql(subscription_id)`

```sql
pg_ripple.unsubscribe_sparql(subscription_id TEXT) RETURNS VOID
```

删除订阅。如果 ID 不存在，则静默成功。

---

## HTTP SSE 端点

`GET /subscribe/:subscription_id`

| 标头 | 值 |
|---|---|
| `Authorization` | `Bearer <token>`（当配置了身份验证时）|
| `Accept` | `text/event-stream` |

**事件格式**：

```
event: sparql_result
id: <notification_id>
data: {"bindings": [...]}

event: keepalive
data: {}
```

每 15 秒发送一个 `keepalive` 事件，以通过代理和负载均衡器保持 TCP 连接的活动状态。

---

## 限制

- **负载大小**：`pg_notify` 有 8 KB 负载限制。当更新后的结果集超过此限制时，pg_ripple 会发送 `{"changed": true}` 信号而不是完整结果。然后客户端必须重新查询以获取新结果。
- **至少一次传递**：SSE 是一种即发即弃协议。如果客户端断开连接并重新连接，它将收到下一个通知，但可能会错过中间的通知。
- **原型**：这是一个实验性实现。订阅机制在变更路径中是同步的；在订阅图上的高写入速率可能会增加延迟。

---

## 实施说明

订阅存储在 `_pg_ripple.sparql_subscriptions` 中：

```sql
SELECT * FROM _pg_ripple.sparql_subscriptions;
-- subscription_id | query | graph_iri | created_at
```

变更日志（`src/storage/mutation_journal.rs`）在刷新 CWB 钩子后调用 `crate::subscriptions::notify_affected_subscriptions()`。此函数查询订阅目录，重新执行任何匹配的订阅的 SPARQL 查询，并调用 `pg_notify`。

---

## 另请参阅

- [CDC 订阅](cdc-subscriptions.md) — 基于 pg-trickle 的变更数据捕获，用于 CDC 中继模式。
- [实时视图](live-views-and-subscriptions.md) — 自动刷新派生图的 SPARQL CONSTRUCT 实时视图。