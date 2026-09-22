# 并行合并工作线程池

> 在 v0.42.0 中添加

## 概述

pg_ripple 使用**垂直分区（VP）架构**，其中每个唯一谓词都有自己的存储表。合并工作线程池使读取优化的 `_main` 分区与写入优化的 `_delta` 表保持同步。

默认情况下，单个后台工作线程按顺序处理所有谓词。对于具有许多不同谓词的工作负载 — 例如具有 50 多个属性类型的丰富本体 — 并行工作线程池可以显著提高写入吞吐量。

## 配置

### `pg_ripple.merge_workers`（仅启动）

控制并行合并工作线程进程的数量。必须在 `postgresql.conf` 中或服务器启动之前设置；不能在会话级别使用 `SET` 更改。

```ini
# postgresql.conf
shared_preload_libraries = 'pg_ripple'
pg_ripple.merge_workers = 4
```

- **默认值**：`1`（单个工作线程，原始行为）
- **范围**：`1` 到 `16`
- **类型**：`integer`，`PGC_POSTMASTER`（仅启动）

### `pg_ripple.merge_threshold`

触发合并之前 VP delta 表中的最小行数。增加此值会降低合并频率但会增加每次合并的成本。

```sql
SET pg_ripple.merge_threshold = 50000;  -- 默认值：10000
```

### `pg_ripple.merge_interval_secs`

合并工作线程轮询周期之间的最大秒数。

```sql
SET pg_ripple.merge_interval_secs = 30;  -- 默认值：60
```

## 工作原理

使用 `merge_workers = N`，pg_ripple 生成 N 个后台工作线程进程。每个工作线程拥有 VP 谓词的不相交轮询子集：

- **Worker 0** 处理 `pred_id % N == 0` 的谓词
- **Worker 1** 处理 `pred_id % N == 1` 的谓词
- … 以此类推

**咨询锁定**防止竞争：在合并谓词之前，工作线程调用 `pg_try_advisory_lock(pred_id)`。如果另一个工作线程已持有锁，则跳过该谓词。

**工作窃取**：在处理其分配的谓词之后，空闲工作线程检查任何"外部"谓词（不在其轮询片中）是否具有高于合并阈值的 delta 表且未持有锁。如果是，它会窃取该工作。这可以防止单个重载谓词延迟合并循环。

## 监控

使用 `pg_ripple.diagnostic_report()` 检查合并工作线程活动：

```sql
SELECT value FROM pg_ripple.diagnostic_report()
WHERE key LIKE 'merge_%';
```

或查询后台工作线程状态：

```sql
SELECT pid, application_name, state
FROM pg_stat_activity
WHERE application_name LIKE 'pg_ripple merge%';
```

## 选择正确的工作线程数

| 谓词数 | 建议的工作线程数 |
|---|---|
| < 20 | 1（默认）|
| 20–100 | 2–4 |
| 100–500 | 4–8 |
| > 500 | 8–16 |

对于大多数工作负载，瓶颈不是工作线程数，而是合并阈值和间隔。在扩展工作线程之前先调整这些。

## 重新启动要求

由于 `merge_workers` 是 `PGC_POSTMASTER` GUC，更改仅在 PostgreSQL 重新启动后生效：

```bash
# 更新 postgresql.conf 后：
pg_ctl restart -D $PGDATA
```
