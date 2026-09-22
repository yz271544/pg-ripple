# CDC（变更数据捕获）参考

此页面是 pg_ripple 的 CDC（变更数据捕获）集成的参考。

## 概述

pg_ripple 可以订阅 PostgreSQL 逻辑复制流并将行级变更事件转换为 RDF 三元组。CDC 桥接支持：

- 通过 `pg_logical_slot_get_changes()` 进行 PostgreSQL 逻辑解码
- 可配置的表到 RDF 映射（主语模板、谓词模板、对象模板）
- Outbox 桥接工作线程以实现可靠的至少一次传递
- 映射到命名图的 CDC 生命周期事件（insert、update、delete）
- 用于下游消费者的 JSON-LD 事件序列化

所有 CDC 摄入的三元组都记录在变更日志中，确保为 CDC 派生数据触发 CONSTRUCT 回写规则。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE 'cdc%';
```

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.register_cdc_subscription(slot_name TEXT, mapping_name TEXT, target_graph TEXT) → void` | 注册 CDC 订阅 |
| `pg_ripple.drop_cdc_subscription(slot_name TEXT) → void` | 删除 CDC 订阅 |
| `pg_ripple.list_cdc_subscriptions() → SETOF record` | 列出活动的 CDC 订阅 |
| `pg_ripple.process_cdc_batch(slot_name TEXT, batch_size INT) → BIGINT` | 处理插槽的下一批 |

## 映射配置

每个 CDC 订阅引用一个 `mapping_name`，它定义表行如何转换为三元组。映射指定：

- **主语模板**：使用列值的 URI 模式（例如，`http://example.org/employee/{id}`）
- **谓词映射**：列名 → 谓词 IRI
- **对象类型**：字面值（带 datatype/lang）、IRI 或空白节点

映射存储在 `_pg_ripple.cdc_subscriptions` 中。

## PG18 逻辑解码

在 PostgreSQL 18 上，CDC 桥接使用内置的逻辑复制基础结构。为每个订阅创建一个复制插槽，桥接工作线程使用 `pg_logical_slot_get_changes()` 轮询更改。

## 可靠性和传递

outbox 桥接模式确保至少一次传递：

1. 更改首先写入 `_pg_ripple.cdc_outbox`。
2. 桥接工作线程以事务方式处理 outbox。
3. 成功处理的条目被删除；失败的重试。

## 相关页面

- [架构内部](architecture.md)
- [功能状态分类](feature-status-taxonomy.md)