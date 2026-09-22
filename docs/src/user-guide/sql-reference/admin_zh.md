# 管理函数

pg_ripple v0.6.0 在 `pg_ripple` schema 中引入了一组管理和监控函数，用于 HTAP 维护、变更数据捕获和统计。

---

## compact()

```sql
pg_ripple.compact() → bigint
```

触发所有 HTAP delta 表同步合并到它们对应的主表中。在合并完成之前阻塞。

**返回**：合并后所有主表中当前的行总数。

**使用场景**：
- 大型批量加载后，在开始读取密集型查询之前调用 `compact()` 以将 delta 刷新到主表
- 在维护窗口中主动减少 delta 大小
- 在测试中模拟已完成的合并周期

```sql
SELECT pg_ripple.compact();
-- 1500000
```

> **注意**：对于后台（非阻塞）合并，请改用合并 worker。`compact()` 是一项前台操作，在表交换期间持有排他锁。

---

## stats()

```sql
pg_ripple.stats() → jsonb
```

返回带有扩展范围统计信息的 JSONB 对象。字段：

| 字段 | 类型 | 描述 |
|---|---|---|
| `total_triples` | integer | 所有 VP 表和 `vp_rare` 中的三元组总数 |
| `dedicated_predicates` | integer | 具有自己 VP 表的谓词数 |
| `htap_predicates` | integer | 使用 delta/main 分割的谓词数 |
| `rare_triples` | integer | 存储在共享 `vp_rare` 表中的三元组 |
| `unmerged_delta_rows` | integer | 尚未合并的所有 delta 表中的行 — 如果未设置 `shared_preload_libraries`，则为 `-1` |
| `merge_worker_pid` | integer | 后台合并 worker 的 PID — 如果未运行则为 `0` |

```sql
SELECT pg_ripple.stats();
-- {
--   "total_triples": 1500000,
--   "dedicated_predicates": 42,
--   "htap_predicates": 42,
--   "rare_triples": 1234,
--   "unmerged_delta_rows": 8742,
--   "merge_worker_pid": 12345
-- }
```

监视随时间变化的 `unmerged_delta_rows`。如果它无限增长，合并 worker 可能被阻塞或配置错误。

---

## htap_migrate_predicate(pred_id)

```sql
pg_ripple.htap_migrate_predicate(pred_id bigint) → void
```

将现有的平面 VP 表（v0.6.0 之前创建）迁移到 delta/main 分区分割。由 `pg_ripple--0.5.1--0.6.0.sql` 迁移脚本自动调用。

**参数**：`pred_id` — 谓词的字典整数 ID。

```sql
-- 首先查找谓词 ID
SELECT id FROM _pg_ripple.predicates p
JOIN _pg_ripple.dictionary d ON d.id = p.id
WHERE d.value = 'https://schema.org/name';

-- 然后迁移
SELECT pg_ripple.htap_migrate_predicate(12345678);
```

---

## subscribe(pattern, channel)

```sql
pg_ripple.subscribe(pattern text, channel text) → bigint
```

注册 CDC（变更数据捕获）订阅。每当在 VP delta 表中插入或删除匹配 `pattern` 的三元组时，在 `channel` 上触发 `pg_notify`。

**参数**：
- `pattern` — 谓词 IRI（例如 `'<https://schema.org/name>'`）或 `'*'` 订阅所有谓词
- `channel` — 用于发送通知的 PostgreSQL NOTIFY 通道的名称

**返回**：订阅 ID（整数）。

```sql
-- 订阅 schema:name 谓词上的所有更改
SELECT pg_ripple.subscribe('<https://schema.org/name>', 'name_changes');

-- 在另一个会话中，监听通知
LISTEN name_changes;

-- 插入三元组以触发通知
SELECT pg_ripple.insert_triple(
    '<https://example.org/Alice>',
    '<https://schema.org/name>',
    '"Alice"'
);
-- NOTIFY name_changes, '{"op":"INSERT","s":...,"p":...,"o":...}'
```

通知有效负载是一个 JSON 对象，其中包含字段 `op`（`"INSERT"` 或 `"DELETE"`）、`s`、`p`、`o`（N-Triples 编码）和 `g`（图 ID）。

---

## unsubscribe(channel)

```sql
pg_ripple.unsubscribe(channel text) → bigint
```

删除给定通道的所有 CDC 订阅。

**返回**：已删除的订阅数。

```sql
SELECT pg_ripple.unsubscribe('name_changes');
-- 1
```

---

## subject_predicates(subject_id) / object_predicates(object_id)

```sql
pg_ripple.subject_predicates(subject_id bigint) → bigint[]
pg_ripple.object_predicates(object_id  bigint) → bigint[]
```

返回给定主语（或对象）至少有一个三元组的谓词 ID 的排序数组。由合并 worker 填充的 `_pg_ripple.subject_patterns` 和 `_pg_ripple.object_patterns` 索引支持。

如果主语/对象尚未建立索引（在第一次合并之前），则返回 `NULL`。

```sql
-- 查找 Alice 使用的所有谓词
SELECT pg_ripple.subject_predicates(
    pg_ripple.encode_term('https://example.org/Alice', 0)
);
```

---

## predicate_stats（视图）

```sql
SELECT * FROM pg_ripple.predicate_stats;
```

方便视图，覆盖 `_pg_ripple.predicates` 和 `_pg_ripple.dictionary`：

| 列 | 描述 |
|---|---|
| `predicate_iri` | 谓词的完整 IRI |
| `triple_count` | 三元组总数（跨 delta + main） |
| `storage` | `'dedicated'`（自己的 VP 表）或 `'rare'`（`vp_rare`） |

```sql
-- 按三元组数排名的前 10 个谓词
SELECT predicate_iri, triple_count, storage
FROM pg_ripple.predicate_stats
ORDER BY triple_count DESC
LIMIT 10;
```

---

## deduplicate_predicate(p_iri TEXT) → BIGINT（v0.7.0）

删除单个谓词的重复 `(s, o, g)` 行，保留具有最低 SID 的行（最早断言）。返回已删除的行数。

- **Delta 表**（`vp_{id}_delta`）：物理删除重复行 — 保留每个分组的最小 SID 行。
- **主表**（`vp_{id}_main`）：为除最小 SID 重复项之外的所有行插入墓碑行，立即从查询中屏蔽重复项；它们将在下一个合并周期中物理删除。
- **vp_rare**：物理删除重复行（保留最小 SID）。
- 在去重后对所有修改过的表运行 ANALYZE。

```sql
-- 删除特定谓词的重复项
SELECT pg_ripple.deduplicate_predicate('<https://schema.org/name>');

-- 返回：已删除的行数
```

**典型用法**：在可能包含重复三元组的批量加载之后调用一次。

---

## vacuum() → bigint（v0.14.0）

```sql
pg_ripple.vacuum() → bigint
```

对所有 HTAP VP 表强制执行完全 delta→main 合并，然后对每个 VP 表（delta、main、tombstones）和 `vp_ripple` 运行 `ANALYZE`。

**返回**：已分析的 VP 表组数。

```sql
SELECT pg_ripple.vacuum();
-- 42
```

> **注意**：`ANALYZE` 更新规划器统计信息。PostgreSQL 的 `VACUUM` 本身无法在事务块内运行；如果需要死元组回收，请单独调用它。

**获取的锁级别（ADMIN-LOCK-01，v0.82.0）：**
- `ANALYZE` 在每个 VP 表上获取一个简短的 `ShareUpdateExclusiveLock`。并发读取和写入不被阻塞。
- delta→main 合并在最后一次交换期间获取 `SET LOCAL lock_timeout`（可通过 `pg_ripple.merge_lock_timeout_ms` 配置，默认 5 秒）以获取 VP 表上的 `ShareRowExclusiveLock`。

---

## reindex() → bigint（v0.14.0）

```sql
pg_ripple.reindex() → bigint
```

使用 `REINDEX TABLE` 重建每个 VP 表（delta 和 main）和 `vp_ripple` 上的所有索引。在大型批量删除后或从索引损坏中恢复时运行。

**返回**：已重新索引的 VP 表组数。

```sql
SELECT pg_ripple.reindex();
-- 42
```

**获取的锁级别（ADMIN-LOCK-01，v0.82.0）：**
- `REINDEX TABLE` 在每个 VP 表上获取一个 `AccessExclusiveLock`，持续重建期间。该表上的所有并发读取和写入将被阻塞，直到重新索引完成。
- 为了最小化影响，`reindex()` 一次处理一个 VP 表。对于具有许多谓词的数据库，请考虑在维护窗口期间运行。

---

## vacuum_dictionary() → bigint（v0.14.0）

```sql
pg_ripple.vacuum_dictionary() → bigint
```

删除任何 VP 表不再引用的字典条目。孤立条目在批量删除后累积。

使用咨询事务锁（键 `0x7269706c`）防止并发运行。在正常操作期间运行是安全的 — 可能会错过最近孤立的条目，这些条目将在下次运行时清理。

**返回**：已删除的字典条目数。

```sql
SELECT pg_ripple.vacuum_dictionary();
-- 128
```

**典型用法**：在批量删除后或 `drop_graph()` 之后定期运行。

---

## dictionary_stats() → jsonb（v0.14.0）

```sql
pg_ripple.dictionary_stats() → jsonb
```

返回有关字典和缓存配置的详细指标。

| 字段 | 描述 |
|---|---|
| `total_entries` | 字典中的总行数 |
| `hot_entries` | 未记录的热字典缓存中的行 |
| `cache_capacity` | 共享内存编码缓存容量（条目） |
| `cache_budget_mb` | 配置的缓存预算上限（MB） |
| `shmem_ready` | 共享内存是否已初始化 |

```sql
SELECT pg_ripple.dictionary_stats();
-- {
--   "total_entries":   450000,
--   "hot_entries":     1024,
--   "cache_capacity":  4096,
--   "cache_budget_mb": 64,
--   "shmem_ready":     true
-- }
```

---

## enable_graph_rls() → boolean（v0.14.0）

```sql
pg_ripple.enable_graph_rls() → boolean
```

使用 `g` 列和 `_pg_ripple.graph_access` 映射表激活 `_pg_ripple.vp_ripple` 上的行级安全策略。默认图（g = 0）始终可访问。命名图需要明确的授权。

成功时返回 `true`。

```sql
SELECT pg_ripple.enable_graph_rls();
-- true
```

---

## grant_graph(role, graph, permission)（v0.14.0）

```sql
pg_ripple.grant_graph(role text, graph text, permission text) → void
```

将 `permission`（`'read'`、`'write'` 或 `'admin'`）授予 PostgreSQL 角色上的命名图。

```sql
SELECT pg_ripple.grant_graph('app_user', '<https://example.org/graph1>', 'read');
SELECT pg_ripple.grant_graph('admin_user', '<https://example.org/graph1>', 'admin');
```

> **注意**：`grant_graph_permission(role, graph, permission)` 是 `grant_graph()` 的旧别名，保留以兼容。新代码请使用 `grant_graph()`。

---

## revoke_graph(role, graph [, permission])（v0.14.0）

```sql
pg_ripple.revoke_graph(role text, graph text, permission text DEFAULT NULL) → void
```

从角色撤销命名图上的权限。为 `permission` 传递 `NULL`（或省略）以撤销该角色在该图上的所有权限。

```sql
-- 撤销特定权限
SELECT pg_ripple.revoke_graph('app_user', '<https://example.org/graph1>', 'read');

-- 撤销所有权限
SELECT pg_ripple.revoke_graph('app_user', '<https://example.org/graph1>');
```

> **注意**：`revoke_graph_permission(role, graph, permission)` 是 `revoke_graph()` 的旧别名，保留以兼容。新代码请使用 `revoke_graph()`。

---

## list_graph_access() → jsonb（v0.14.0）

```sql
pg_ripple.list_graph_access() → jsonb
```

返回所有图访问控制条目作为 JSONB 数组。每个元素都有 `role`、`graph`（解码的 IRI）和 `permission`。

```sql
SELECT * FROM jsonb_array_elements(pg_ripple.list_graph_access());
```

---

## schema_summary() → jsonb（v0.14.0）

```sql
pg_ripple.schema_summary() → jsonb
```

返回类→属性→基数实时摘要作为 JSONB 数组。当调用 `enable_schema_summary()` 时（需要 pg_trickle），从物化的 `_pg_ripple.inferred_schema` 流表中读取。

每个元素：`{"class": "...", "property": "...", "cardinality": N}`。

```sql
SELECT * FROM jsonb_array_elements(pg_ripple.schema_summary());
```

---

## enable_schema_summary() → boolean（v0.14.0）

```sql
pg_ripple.enable_schema_summary() → boolean
```

将 `_pg_ripple.inferred_schema` 创建为 pg_trickle 流表（每 30 秒刷新一次）用于 SPARQL IDE 自动完成。需要 pg_trickle。如果未安装 pg_trickle，则返回 `false` 和警告。

```sql
SELECT pg_ripple.enable_schema_summary();
-- true（或 false 和警告，如果缺少 pg_trickle）
```

---

## deduplicate_all() → bigint（v0.7.0）

```sql
pg_ripple.deduplicate_all() → bigint
```

跨所有谓词删除重复的 `(s, o, g)` 行，保留具有最低 SID 的行。返回已删除的重复行总数。

```sql
SELECT pg_ripple.deduplicate_all();
```

---

## dedup_on_merge (GUC)

| GUC | 类型 | 默认值 | 描述 |
|-----|------|---------|-------------|
| `pg_ripple.dedup_on_merge` | `boolean` | `off` | 当为 `on` 时，HTAP 合并 worker 在压缩期间使用 `DISTINCT ON` 对 `(s, o, g)` 行进行去重，保留最低 SID 行 |

```sql
-- 启用合并时去重
SET pg_ripple.dedup_on_merge = true;

-- 触发合并（去重在压缩期间原子发生）
SELECT pg_ripple.compact();
```

在合并之间，`(main EXCEPT tombstones) UNION ALL delta` 查询视图可能会观察到短暂的重复。对于大多数工作负载来说这是无害的。

---

## plan_cache_stats() → jsonb（v0.13.0）

```sql
pg_ripple.plan_cache_stats() → jsonb
```

以 JSONB 对象的形式返回有关 SPARQL 计划缓存的统计信息。使用此函数来监视缓存有效性并调整 `pg_ripple.plan_cache_size`。

| 字段 | 描述 |
|-------|-------------|
| `hits` | 自启动以来的缓存命中数 |
| `misses` | 缓存未命中（重新编译）次数 |
| `size` | 当前缓存的计划数 |
| `capacity` | 最大缓存容量 |

```sql
SELECT pg_ripple.plan_cache_stats();
-- {"hits": 1523, "misses": 42, "size": 38, "capacity": 128}
```

高未命中率（> 50%）表明要么有太多不同的查询形状，要么缓存太小。尝试增加 `pg_ripple.plan_cache_size` 或使用 `VALUES` 块对查询进行参数化。

---

## plan_cache_reset() → void（v0.13.0）

```sql
pg_ripple.plan_cache_reset() → void
```

逐出所有缓存的 SPARQL→SQL 计划并重置命中/未命中计数器。在架构更改、VP 提升之后或启用/禁用 `pg_ripple.bgp_reorder` 时很有用。

```sql
SELECT pg_ripple.plan_cache_reset();
```

---

## promote_rare_predicates() → bigint（v0.2.0）

```sql
pg_ripple.promote_rare_predicates() → bigint
```

扫描 `_pg_ripple.vp_ripple` 以查找其三元组数已超过 `pg_ripple.vp_promotion_threshold` 的谓词，并将每个谓词提升到专用 VP 表。返回已提升的谓词数。

```sql
SELECT pg_ripple.promote_rare_predicates();
-- 3
```

提升通常在插入期间自动进行。更改阈值或在延迟自动提升的批量加载后使用此选项。

---

## `_pg_ripple.merge_worker_status` 表（D13-02、v0.86.0）

由后台合并 worker 维护的内部监视表。

| 列 | 类型 | 描述 |
|---|---|---|
| `pid` | `INTEGER` | 上一个合并周期的后台 worker 进程的 PID |
| `last_merge_at` | `TIMESTAMPTZ` | 最近成功的合并周期的挂钟时间 |
| `last_merge_duration_ms` | `BIGINT` | 上一个合并周期的持续时间（毫秒） |
| `last_merge_rows` | `BIGINT` | 上一个合并周期提升的 delta 行数 |
| `total_merge_cycles` | `BIGINT` | 自 worker 启动以来的累计合并周期数 |
| `status` | `TEXT` | 当前 worker 状态：`idle`、`merging` 或 `error:<msg>` |

查询示例：

```sql
SELECT * FROM _pg_ripple.merge_worker_status;
```

`pg_ripple_merge_worker_delta_rows_pending` Prometheus 指标（v0.86.0 中添加）显示所有 VP 表中未合并的 delta 行数，在每个合并周期更新。