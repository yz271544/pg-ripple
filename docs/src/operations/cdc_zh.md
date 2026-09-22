# 使用 pg_ripple 进行变更数据捕获 (CDC)

pg_ripple 与 PostgreSQL 的逻辑复制基础设施集成，以在插入、更新或删除三元组时提供实时通知。
此功能称为变更数据捕获 (CDC)。

## 概述

CDC 订阅允许应用程序在 RDF 图更改时立即做出反应 — 例如，使缓存失效、触发下游处理或将更新流式传输到外部系统。

pg_ripple CDC 构建于以下之上：
- PostgreSQL 逻辑复制槽（`pg_logical_slot_get_changes`）
- `_pg_ripple.cdc_queue` 表（由 VP delta 表上的触发器填充）
- `pg_ripple.cdc_subscribe()` / `pg_ripple.cdc_poll()` API

## 配置

```sql
-- 启用 CDC（如果尚未存在则创建队列表和触发器）
SELECT pg_ripple.cdc_enable();

-- 订阅特定谓词 IRI（NULL = 所有谓词）
SELECT pg_ripple.cdc_subscribe(
    subscriber_id := 'my-app',
    predicate_iri := 'http://schema.org/name'
);
```

## 轮询变更

```sql
-- 为订阅者 'my-app' 轮询最多 100 个变更
SELECT * FROM pg_ripple.cdc_poll('my-app', max_events := 100);
```

返回 `(event_type TEXT, s TEXT, p TEXT, o TEXT, graph_iri TEXT, event_time TIMESTAMPTZ)` 的行。

## 调整 CDC 队列

| 参数 | 默认 | 描述 |
|-----------|---------|-------------|
| `pg_ripple.cdc_queue_max_age` | `'7 days'` | 早于此时间的行会被自动清除 |
| `pg_ripple.cdc_batch_size` | `1000` | 每个 vacuum 周期处理的最大事件数 |
| `pg_ripple.cdc_slow_subscriber_timeout` | `'1 hour'` | 如果订阅者在此时间窗口内没有轮询，则断开连接 |

```sql
-- 将队列保留时间增加到 30 天
ALTER SYSTEM SET pg_ripple.cdc_queue_max_age = '30 days';
SELECT pg_reload_conf();
```

## 处理慢速订阅者

如果订阅者轮询速度慢，CDC 队列可能会无限增长。当订阅者超过 `cdc_slow_subscriber_timeout` 时，
pg_ripple 会记录警告：

```
WARNING: CDC subscriber 'my-app' has not polled in 2h 15m (threshold: 1h);
         consider increasing poll frequency or raising cdc_slow_subscriber_timeout
```

要强制断开过时订阅者的连接：

```sql
SELECT pg_ripple.cdc_unsubscribe('my-app');
```

## 监控队列深度

```sql
SELECT
    subscriber_id,
    count(*) AS pending_events,
    min(event_time) AS oldest_event,
    max(event_time) AS newest_event
FROM _pg_ripple.cdc_queue
GROUP BY subscriber_id
ORDER BY pending_events DESC;
```

## 示例：通过 LISTEN/NOTIFY 对新三元组做出反应

pg_ripple 可以选择在每次 CDC 事件入队时在 `pg_ripple_cdc` 通道上发送 PostgreSQL `NOTIFY`：

```sql
SELECT pg_ripple.cdc_enable_notify();

-- 在您的应用程序中：
LISTEN pg_ripple_cdc;
-- 当 NOTIFY 到达时，调用 cdc_poll() 来检索实际事件。
```

## 限制

- CDC 捕获对 VP delta 表的 DML；合并工作线程添加到主表的行不会被重新捕获
  （它们在插入时通过 delta 被捕获）。
- `pg_dump` 不包括 CDC 队列内容；恢复从空队列开始。
- 如果直接使用 `pg_logical`，则必须单独管理逻辑复制槽。
