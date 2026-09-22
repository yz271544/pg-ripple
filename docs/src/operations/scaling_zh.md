# 扩展

pg_ripple 在单个 PostgreSQL 实例内进行垂直扩展，并通过流复制对读取流量进行水平扩展。本页介绍如何分配资源、调整合并工作线程、设置只读副本，并了解当前的限制。

```admonish info title="当前的扩展模型"
pg_ripple 完全在 PostgreSQL 内运行。它继承了 PostgreSQL 的单写入者架构：一个主节点处理所有写入，读取副本可以服务于只读 SPARQL 查询。通过 [Citus 集成](citus-integration.md) 提供实验性的水平分片，但当前 CI 不运行 Citus 集群。
```

---

## 垂直扩展

最具影响力的扩展杠杆是为您的单个 PostgreSQL 实例提供更多资源。

### 内存

内存影响三个关键领域：

| 资源 | 由...控制 | 影响 |
|---|---|---|
| 字典 LRU 缓存 | `pg_ripple.dictionary_cache_size` | 减少 IRI/字面量查找的磁盘 I/O。每个 SPARQL 查询在解码时都会访问字典。 |
| PostgreSQL 共享缓冲区 | `shared_buffers` | 缓存 VP 表页。更大 = 连接的磁盘读取更少。 |
| 工作内存 | `work_mem` | 用于 SPARQL 生成的 SQL 中的排序、哈希连接和哈希聚合。 |

#### 字典缓存大小调整

字典缓存在服务器启动时在共享内存中分配。每个条目大约消耗 200 字节。

```sql
-- 检查当前使用率
SELECT
  s->>'encode_cache_capacity' AS capacity,
  s->>'encode_cache_utilization_pct' AS utilization_pct,
  ROUND(
    (s->>'encode_cache_hits')::numeric /
    NULLIF((s->>'encode_cache_hits')::numeric + (s->>'encode_cache_misses')::numeric, 0),
    4
  ) AS hit_rate
FROM pg_ripple.stats() s;
```

| 命中率 | 操作 |
|---|---|
| > 95% | 健康 — 无需更改 |
| 90–95% | 考虑增加 `dictionary_cache_size` |
| < 90% | 将 `dictionary_cache_size` 加倍并重新启动 |

```admonish tip title="经验法则"
将 `dictionary_cache_size` 设置为至少是您总唯一 IRI + 字面量的 10%。对于具有 5M 唯一项的数据集，从 500K 条目开始（约 100MB 共享内存）。
```

#### PostgreSQL 内存设置

```ini
# postgresql.conf — 用于 64GB 服务器，主要工作负载为 pg_ripple
shared_buffers = 16GB
effective_cache_size = 48GB
work_mem = 256MB
maintenance_work_mem = 2GB
```

```admonish warning title="work_mem 和 SPARQL"
具有多个连接、UNION 或聚合的复杂 SPARQL 查询可以生成许多哈希操作。PostgreSQL **每个查询的每个操作**分配 `work_mem`。从保守值（64MB–256MB）开始，如果您在日志中看到"临时文件"条目，则增加。
```

### CPU

| 工作负载 | CPU 收益 |
|---|---|
| SPARQL 查询执行 | 更多核心 → 大型连接的更多并行工作线程 |
| 合并工作线程 | 每个谓词单线程，但合并跨谓词并发运行 |
| 批量加载 | `load_turtle` / `load_ntriples` 是 I/O 密集型；CPU 有助于字典编码 |
| Datalog 推理 | 半朴素不动点是 CPU 密集型；受益于更快的核心 |

设置 `max_parallel_workers_per_gather` 以允许 PostgreSQL 并行化大型 VP 表扫描：

```ini
max_parallel_workers_per_gather = 4
max_parallel_workers = 8
parallel_setup_cost = 100
parallel_tuple_cost = 0.001
```

pg_ripple 的 `parallel_query_min_joins` GUC 控制 SPARQL 引擎何时在生成的 SQL 中启用并行提示（默认值：3 个连接）。

### 存储

| 层级 | 建议 |
|---|---|
| NVMe SSD | 适用于所有工作负载。字典查找和 VP 表连接的随机 I/O。 |
| SATA SSD | 可接受用于中等数据集。 |
| HDD | 不推荐。字典查找和 VP 连接是随机 I/O 密集型的。 |

```admonish tip title="分离 WAL 和数据"
将 `pg_wal` 放在与主数据目录分开的 NVMe 设备上。pg_ripple 的批量加载和合并操作会产生大量 WAL 流量。
```

---

## 合并工作线程调优

HTAP 合并工作线程是 pg_ripple 特定的、最重要的扩展旋钮。它控制 delta 行（最近的写入）整合到主 BRIN 索引分区的速度。

### 合并工作线程如何运行

1. 工作线程每 `merge_interval_secs`（默认值：60 秒）轮询一次
2. 对于每个谓词，它检查 `delta 行数 >= merge_threshold`
3. 如果是，它创建一个新的主表：`(旧主 − 墓碑) UNION ALL delta`
4. 它交换视图指向新主，然后在 `merge_retention_seconds` 后删除旧主
5. 如果 `auto_analyze` 开启，则在新主上运行 ANALYZE

### 针对写入密集型工作负载的调优

降低合并阈值和间歇，以保持 delta 表较小：

```ini
pg_ripple.merge_threshold = 5000
pg_ripple.merge_interval_secs = 30
pg_ripple.latch_trigger_threshold = 5000
```

这提供更新鲜的读取，但由于更频繁的合并而增加 I/O。

### 针对读取密集型工作负载的调优

提高阈值以在合并之前批处理更多写入：

```ini
pg_ripple.merge_threshold = 50000
pg_ripple.merge_interval_secs = 120
```

这减少了合并 I/O 开销，但意味着查询扫描更大的 delta 表。

### 监控合并活动

```sql
-- 合并工作线程是否正在运行？
SELECT (pg_ripple.stats()->>'merge_worker_pid')::int AS pid;

-- 有多少未合并的 delta 行？
SELECT (pg_ripple.stats()->>'unmerged_delta_rows')::int AS delta_rows;
```

```admonish warning title="合并工作线程停滞"
如果 `unmerged_delta_rows` 持续增长，而 `merge_worker_pid` 不为零，则工作线程可能已卡住。检查 `pg_stat_activity` 中长时间运行的合并事务，并查找锁争用。如果工作线程空闲时间过长，`merge_watchdog_timeout` GUC（默认值：300 秒）会记录 WARNING。
```

---

## 只读副本

PostgreSQL 流复制为 SPARQL 查询提供水平读取扩展。

### 架构

```
┌────────────┐     WAL stream     ┌────────────┐
│  Primary   │ ──────────────────→ │  Replica 1 │ ← SPARQL reads
│  (writes)  │                     └────────────┘
│            │     WAL stream     ┌────────────┐
│            │ ──────────────────→ │  Replica 2 │ ← SPARQL reads
└────────────┘                     └────────────┘
```

### 设置只读副本

在主节点上：

```ini
# postgresql.conf
wal_level = replica
max_wal_senders = 5
wal_keep_size = 1GB
```

创建复制槽：

```sql
SELECT pg_create_physical_replication_slot('replica1');
```

在副本上：

```bash
pg_basebackup -h primary-host -D /var/lib/postgresql/18/main -R -S replica1 -P
```

启动副本 — 它将开始流式传输 WAL 并重放更改，包括所有 VP 表变更。

### 副本注意事项

```admonish note title="合并工作线程不在副本上运行"
后台合并工作线程仅在主节点上运行。副本通过 WAL 重放接收已合并的状态。这意味着副本始终具有一致的数据视图，无需任何额外开销。
```

- **SPARQL 查询在副本上的工作方式相同** — 查询引擎以相同方式读取 VP 表
- **字典缓存**每个实例独立 — 每个副本维护自己的 LRU 缓存
- **复制滞后**：在主节点上使用 `pg_stat_replication` 进行监控。在正常负载下，滞后应为亚秒级
- **热备冲突**：副本上长时间运行的 SPARQL 查询可能与 WAL 重放冲突。适当地设置 `max_standby_streaming_delay`：

```ini
# 在副本上
max_standby_streaming_delay = 30s
hot_standby_feedback = on
```

---

## 连接池

对于具有许多并发 SPARQL 客户端的工作负载，使用连接池：

```admonish tip title="PgBouncer 与 pg_ripple"
pg_ripple 使用会话级 GUC 参数（例如 `pg_ripple.inference_mode`）。如果您使用 PgBouncer，请在 **会话**池模式下配置它，而不是事务模式。事务模式池化在事务之间重置 GUC，这可能导致意外行为。
```

```ini
# pgbouncer.ini
[databases]
mydb = host=127.0.0.1 port=5432 dbname=mydb

[pgbouncer]
pool_mode = session
max_client_conn = 200
default_pool_size = 20
```

---

## 扩展限制和诚实的边界

| 维度 | 当前能力 | 限制 |
|---|---|---|
| 每个实例的三元组数 | 没有保留的当前版本规模结果 | 受磁盘、内存和工作负载限制 |
| 并发 SPARQL 查询 | 没有保留的当前版本并发性结果 | 受连接数、CPU、内存和查询形状限制 |
| 写入吞吐量 | 有界的 100,000 三元组制作者可用 | 单主 PostgreSQL 架构 |
| 只读副本 | PostgreSQL 流复制 | 必须使用目标拓扑进行限定 |
| 跨节点分片 | 实验性 Citus 集成 | 当前 CI 仅验证非 Citus 降级路径 |
| 多主写入 | **不支持** | PostgreSQL 限制 |
| 联邦 | 支持（SERVICE 子句） | 远程端点增加延迟 |

```admonish tip title="使用 Citus 进行水平分片"
pg_ripple 可以按主语 ID 分发 VP 表，并且可以为绑定主语查询注释分片修剪。在目标拓扑上运行多节点可靠性、恢复和负载测试之前，请将此路径视为实验性的。请参阅 [Citus 集成](citus-integration.md)。
```

---

## 容量规划

### 存储估计

| 组件 | 每个三元组（约） |
|---|---|
| VP 表行（s、o、g、i、source） | ~40 字节 |
| VP 索引（双 B-tree） | ~80 字节 |
| 字典条目（每个唯一项） | ~120 字节 |
| HTAP 开销（delta + 墓碑表） | 主动写入期间约为 VP 大小的 20% |

**示例**：100M 三元组具有 20M 唯一项 ≈ 12 GB（VP）+ 2.4 GB（字典）+ 开销 ≈ **~20 GB** 总计。

### 内存估计

| 组件 | 大小调整 |
|---|---|
| `shared_buffers` | RAM 的 25% |
| `dictionary_cache_size` | 唯一项的 10% |
| `work_mem` | 64MB–512MB 取决于查询复杂度 |
| 操作系统页缓存 | 剩余 RAM |

```admonish tip title="从小处开始，测量，扩展"
使用保守设置进行部署，加载数据，并运行代表性查询。在添加硬件之前，使用 `pg_ripple.stats()` 和 PostgreSQL 的 `pg_stat_user_tables` 来识别瓶颈。
```

---

## 序列耗尽（`statement_id_seq`） {#sequence-exhaustion}

**(L15-11，v0.97.0)**

pg_ripple 存储的每个三元组都从 `_pg_ripple.statement_id_seq` 序列接收一个全局唯一的**语句标识符（SID）**。该序列是标准的 PostgreSQL `BIGINT` 序列，意味着它可以从 1 计数到 9,223,372,036,854,775,807（大约 9.2 × 10¹⁸）。

### 距离耗尽还有多长时间？

| 插入速率 | 耗尽年数 |
|---|---|
| 1,000 三元组/秒 | ~292,000 年 |
| 1,000,000 三元组/秒 | ~292 年 |
| 1,000,000,000 三元组/秒（理论最大值） | ~0.3 年 |

在实际工作负载下，该序列永远不会耗尽。但是，如果您达到此限制，PostgreSQL 会引发：

```
ERROR:  nextval: reached maximum value of sequence "_pg_ripple.statement_id_seq" (9223372036854775807)
```

### 检查剩余容量

使用 `pg_ripple.sid_exhaustion_years()` 检查还剩多少年：

```sql
SELECT pg_ripple.sid_exhaustion_years();
-- 返回：291723.6（按当前插入速率剩余的年数）
```

如果已分配的 SID 少于 100,000，则该函数返回 `NULL`（在低计数下速率估计不可靠）。

您也可以直接查询序列：

```sql
SELECT
    last_value                                  AS current_sid,
    9223372036854775807 - last_value            AS sids_remaining,
    (9223372036854775807 - last_value)::numeric
        / NULLIF((SELECT count(*) FROM _pg_ripple.vp_rare), 0) AS approx_years_at_current_rate
FROM _pg_ripple.statement_id_seq;
```

### 恢复程序（如果发生耗尽）

```admonish warning title="破坏数据的程序"
重置序列需要清除所有 VP 表。仅在开发环境中执行此操作，或在完全转储/恢复周期（重新建立 SID 唯一性）后执行。
```

1. **转储所有数据**使用 `pg_ripple.export_turtle()` 或 `pg_dump`。
2. **截断所有 VP 表**：
   ```sql
   SELECT pg_ripple.truncate_all_triples();  -- 仅开发！
   ```
3. **重置序列**：
   ```sql
   ALTER SEQUENCE _pg_ripple.statement_id_seq RESTART WITH 1;
   ```
4. **从转储中重新加载数据**。

在生产中，正确的长期解决方案是使用 `CYCLE` 序列，pg_ripple 当前不支持（添加 `CYCLE` 将允许 SID 重用，这可能导致时序查询的正确性问题）。如果您预计将在部署范围内达到此限制，请在支持票证中跟踪此问题。