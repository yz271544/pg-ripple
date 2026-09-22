# Citus + pg_ripple：端到端集成指南

> **版本**：v0.59.0 (CITUS-15)
>
> 本指南涵盖在多 worker 环境中部署带有 Citus 水平分片和
> 可选 CDC/IVM 兼容性的 pg_ripple。它假定您已经阅读了 [Citus 集成](citus-integration.md) 页面。

```admonish warning title="实验性资格"
当前 CI 不安装 Citus 也不运行多节点集群。它验证 Citus 缺失时的优雅降级。在生产使用之前，请对类似目标的临时集群运行完整的设置、正确性、重新平衡、故障和负载测试。
```

---

## 概述

Citus 集成公开了以下实现路径：

| 功能 | 版本 | 描述 |
|---------|---------|-------------|
| VP 表分布 | v0.58.0 | `enable_citus_sharding()` 分布 VP delta 表 |
| 合并隔离咨询锁 | v0.58.0 | 防止重新平衡期间的脑裂 |
| SPARQL 分片修剪 | v0.59.0 | 绑定主语查询可以携带分片路由键 |
| 重新平衡 NOTIFY | v0.59.0 | 用于下游维护钩子的 `merge_start`/`merge_end` 信号 |
| `explain_sparql` Citus 部分 | v0.59.0 | 使用 `EXPLAIN` 验证修剪 |
| `citus_rebalance_progress()` | v0.59.0 | 观察实时重新平衡状态 |

---

## 先决条件

1. **Citus 12+** 安装在协调器和所有 worker 上。
2. **pg_ripple 0.59.0** 安装在协调器上。
3. 根据需要的可选伴随：**pg_trickle 0.46.0+** 用于支持 IVM 的视图，
  以及 **pg_tide 0.33.0+** 用于中继/outbox CDC 传输。

---

## 第 1 步：在协调器上安装 pg_ripple 和 Citus

```sql
-- 在协调器节点上：
CREATE EXTENSION citus;
CREATE EXTENSION pg_ripple;

-- 验证 Citus 已检测到：
SELECT pg_ripple.citus_available();  -- 返回 true
```

## 第 2 步：配置分片 GUC

```sql
ALTER SYSTEM SET pg_ripple.citus_sharding_enabled = on;
-- 启用旧版 CDC/IVM 协同定位兼容性（防止跨分片删除）：
ALTER SYSTEM SET pg_ripple.citus_trickle_compat = on;
SELECT pg_reload_conf();
```

## 第 3 步：分布 VP 表

加载初始数据后，分发所有 VP 表：

```sql
SELECT predicate_id, table_name, status
FROM pg_ripple.enable_citus_sharding();
```

这会对每个 VP delta 表执行：
1. `ALTER TABLE … REPLICA IDENTITY FULL`（逻辑复制使用者所必需）
2. `create_distributed_table(…, 's', colocate_with => 'none')`（或 `'default'`）
3. `pg_notify('pg_ripple.vp_promoted', …)` — 下游维护钩子可以做出反应

## 第 4 步：验证分片修剪（v0.59.0）

加载一些具有已知主语 IRI 的数据后，验证分片修剪是否有效：

```sql
SELECT pg_ripple.explain_sparql(
    'SELECT ?p ?o WHERE { <http://example.org/Alice> ?p ?o }',
    false,      -- analyze
    true        -- citus (new in v0.59.0)
) -> 'citus';
```

修剪成功时的预期输出：

```json
{
  "available": true,
  "pruned_to_shard": 102008,
  "worker": "worker1:5432",
  "full_fanout_avoided": true,
  "estimated_rows_per_shard": 47
}
```

当 `full_fanout_avoided` 为 `false` 时，请检查：
- 主语 IRI 存在于字典中：`SELECT pg_ripple.encode_term('<http://example.org/Alice>', 0)`。
- VP delta 表已分布：`SELECT logicalrelid FROM pg_dist_partition WHERE logicalrelid::text LIKE '%delta%'`。

## 第 5 步：重新平衡 worker

在添加 Citus worker 时，使用了解 pg_ripple 的重新平衡器：

```sql
-- 监控进度（无正在运行的重新平衡时返回空）：
SELECT * FROM pg_ripple.citus_rebalance_progress();

-- 触发重新平衡：
SELECT pg_ripple.citus_rebalance();
```

`citus_rebalance()` 在获取隔离锁之前发出 `pg_ripple.merge_start`，
在释放锁之后发出 `pg_ripple.merge_end`。pg-trickle 监听这些通道
并暂停每个 worker 的槽轮询，直到重新平衡完成，从而防止重复的 CDC 传递。

在监控会话中监听信号：

```sql
LISTEN "pg_ripple.merge_start";
LISTEN "pg_ripple.merge_end";
-- （在另一个会话中运行 pg_ripple.citus_rebalance()，然后检查通知）
```

---

## GUC 参考

| GUC | 默认 | 描述 |
|-----|---------|-------------|
| `pg_ripple.citus_sharding_enabled` | `off` | 为 VP 表启用 Citus 分片分布 |
| `pg_ripple.citus_trickle_compat` | `off` | 使用 `colocate_with => 'none'` 以实现旧版 CDC/IVM 兼容性 |
| `pg_ripple.merge_fence_timeout_ms` | `0` | 等待合并隔离的最长时间（毫秒）（0 = 无隔离） |

---

## 故障排除

### `PT536`：Citus 扩展未安装

`pg_ripple.enable_citus_sharding()` 或 `citus_rebalance()` 引发了 PT536。

**修复**：在调用这些函数之前，在协调器上执行 `CREATE EXTENSION citus;`。

### explain 输出中的 `full_fanout_avoided: false`

主语 IRI 不在字典中（尚未为该主语加载三元组），
或者 VP 表未分布。

**修复**：为该主语至少加载一个三元组，然后重新运行 `enable_citus_sharding()`。

### pg-trickle 在重新平衡后停止

检查您的 pg-trickle 版本是否 ≥ 0.34.0，该版本处理 `pg_ripple.merge_start`
/ `merge_end` 通知并自动恢复槽轮询。

---

*另请参阅：[Citus 集成](citus-integration.md)、[高可用性](high-availability.md)*
