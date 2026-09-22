# 故障排除

常见问题、原因和分步解决方案的运行手册。每个条目都遵循以下模式：**症状 → 原因 → 诊断 → 修复**。

---

## 1. SPARQL 查询返回零行

**症状**：应该返回结果的 SPARQL 查询返回空集。

**原因**：最常见的原因是使用与字典不匹配的未编码 IRI 进行查询，或者查询了错误的图。

**诊断**：

```sql
-- 检查三元组是否存在
SELECT pg_ripple.stats();

-- 验证 IRI 在字典中
SELECT id FROM _pg_ripple.dictionary WHERE value = 'http://example.org/MyResource';

-- 检查默认图与命名图
SELECT pg_ripple.sparql($$
  SELECT ?g (COUNT(*) AS ?n) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g
$$);
```

**修复**：确保查询使用与存储中完全相同的 IRI（区分大小写，没有尾随斜杠差异）。如果数据已加载到命名图中，请使用 `GRAPH` 或 `FROM` 子句。

---

## 2. 合并工作线程未运行

**症状**：`pg_ripple.stats()` 显示 `merge_worker_pid: 0`。Delta 行累积。

**原因**：`pg_ripple` 不在 `shared_preload_libraries` 中，或 `worker_database` 指向错误的数据库。

**诊断**：

```sql
SHOW shared_preload_libraries;
SHOW pg_ripple.worker_database;
```

**修复**：

```ini
# postgresql.conf
shared_preload_libraries = 'pg_ripple'
pg_ripple.worker_database = 'mydb'
```

重新启动 PostgreSQL。使用以下命令验证：

```sql
SELECT (pg_ripple.stats()->>'merge_worker_pid')::int;
```

---

## 3. 慢速查询 — 无界属性路径

**症状**：具有 `*` 或 `+` 属性路径的查询需要数分钟或永远无法完成。

**原因**：属性路径查询编译为 `WITH RECURSIVE` CTE。在大型、高度连接的图上，递归探索巨大的搜索空间。

**诊断**：

```sql
SHOW pg_ripple.max_path_depth;
-- 检查生成的 SQL
SET pg_ripple.plan_cache_size = 0;  -- 禁用缓存以查看新计划
EXPLAIN (ANALYZE, BUFFERS) <来自日志的生成 SQL>;
```

**修复**：限制递归深度：

```sql
SET pg_ripple.max_path_depth = 10;
```

或重写查询以使用有界路径（`{1,5}`）而不是 `*`/`+`。

---

## 4. SHACL 验证未触发

**症状**：违反 SHACL 形状的数据被插入而没有错误。

**原因**：SHACL 强制执行默认为异步，或形状未加载。

**诊断**：

```sql
-- 检查加载的形状
SELECT pg_ripple.sparql($$
  SELECT ?shape WHERE { ?shape a <http://www.w3.org/ns/shacl#NodeShape> }
$$);

-- 检查强制模式
SHOW pg_ripple.enforce_constraints;
```

**修复**：将强制模式设置为 `'error'` 以进行同步验证：

```sql
SET pg_ripple.enforce_constraints = 'error';
```

如果需要，重新加载形状：

```sql
SELECT pg_ripple.load_shapes('<shapes-graph-iri>');
```

---

## 5. Datalog 推理不产生结果

**症状**：`pg_ripple.infer()` 或 `pg_ripple.infer_goal()` 返回零个新三元组。

**原因**：规则未加载，推理模式为 `'off'`，或规则原子与任何数据都不匹配。

**诊断**：

```sql
SHOW pg_ripple.inference_mode;

-- 列出已加载的规则集
SELECT pg_ripple.list_rule_sets();

-- 使用简单规则测试
SELECT pg_ripple.load_rules('test', $$
  :Parent(?x, ?z) :- :Parent(?x, ?y), :Parent(?y, ?z).
$$);
SELECT pg_ripple.infer('test');
```

**修复**：确保 `inference_mode` 为 `'on_demand'` 或 `'materialized'`，规则已加载，并且规则原子中的谓词与您数据的实际 IRI 完全匹配。

---

## 6. 启动时共享内存错误

**症状**：PostgreSQL 无法启动，出现 `could not create shared memory segment`，或 pg_ripple 记录 `insufficient shared memory`。

**原因**：`pg_ripple.dictionary_cache_size` 对于系统的共享内存限制来说太大。

**诊断**：

```bash
# 检查系统共享内存限制
sysctl kern.sysv.shmmax  # macOS
sysctl kernel.shmmax      # Linux
```

**修复**：减少 `dictionary_cache_size` 或增加操作系统共享内存限制：

```bash
# Linux
sudo sysctl -w kernel.shmmax=17179869184  # 16GB
sudo sysctl -w kernel.shmall=4194304

# macOS
sudo sysctl -w kern.sysv.shmmax=17179869184
```

```admonish tip title="Docker 环境"
在 Docker 中，在您的 `docker run` 命令中设置 `--shm-size=2g`（或更大）。
```

---

## 7. 高字典缓存驱逐压力

**症状**：`pg_ripple.stats()` 中的 `encode_cache_evictions` 很高；缓存命中率降至 90% 以下。

**原因**：IRI/字面量的工作集超过缓存容量。

**诊断**：

```sql
SELECT
  s->>'encode_cache_capacity' AS capacity,
  s->>'encode_cache_utilization_pct' AS util_pct,
  s->>'encode_cache_evictions' AS evictions,
  ROUND(
    (s->>'encode_cache_hits')::numeric /
    NULLIF((s->>'encode_cache_hits')::numeric + (s->>'encode_cache_misses')::numeric, 0),
    4
  ) AS hit_rate
FROM pg_ripple.stats() s;
```

**修复**：在 `postgresql.conf` 中增加 `dictionary_cache_size` 并重新启动：

```ini
pg_ripple.dictionary_cache_size = 131072  -- 将默认值加倍
```

---

## 8. 联邦查询超时

**症状**：具有 `SERVICE` 子句的查询挂起或返回超时错误。

**原因**：远程 SPARQL 端点无法访问、速度慢或返回意外格式。

**诊断**：

```bash
# 直接测试远程端点
curl -s -H "Accept: application/sparql-results+json" \
  "https://remote.example.org/sparql?query=SELECT+*+WHERE+{?s+?p+?o}+LIMIT+1"
```

**修复**：

- 验证到远程端点的网络连接
- 增加联邦超时：

```sql
SET pg_ripple.federation_timeout = 60;  -- 秒
```

- 检查远程端点是否支持所需的结果格式（SPARQL JSON 结果）

---

## 9. pg_ripple_http 无响应

**症状**：HTTP SPARQL 端点返回连接被拒绝或 502 错误。

**原因**：`pg_ripple_http` 配套服务未运行，或无法连接到 PostgreSQL。

**诊断**：

```bash
# 检查进程是否正在运行
ps aux | grep pg_ripple_http

# 检查服务日志
journalctl -u pg_ripple_http --since "10 minutes ago"

# 直接测试 PostgreSQL 连接
psql -h localhost -p 5432 -U pg_ripple_http -d mydb -c "SELECT 1"
```

**修复**：

- 启动或重新启动服务
- 验证 `pg_ripple_http` 配置中的连接字符串
- 检查 `pg_hba.conf` 是否允许来自 HTTP 服务的连接

---

## 10. VP 表膨胀

**症状**：磁盘使用量增长快于预期；`pg_size_pretty(pg_total_relation_size('_pg_ripple.vp_12345'))` 远大于三元组计数所暗示的大小。

**原因**：频繁的删除和重新插入而不进行合并周期，或自动清理跟不上。

**诊断**：

```sql
-- 检查死元组
SELECT relname, n_dead_tup, n_live_tup,
       last_autovacuum, last_autoanalyze
FROM pg_stat_user_tables
WHERE schemaname = '_pg_ripple'
ORDER BY n_dead_tup DESC
LIMIT 10;
```

**修复**：

```sql
-- 在膨胀的表上强制 vacuum
VACUUM (VERBOSE) _pg_ripple.vp_12345_main;

-- 积极回收空间
VACUUM (FULL) _pg_ripple.vp_12345_main;
```

为 VP 表调整自动清理：

```sql
ALTER TABLE _pg_ripple.vp_12345_delta
  SET (autovacuum_vacuum_scale_factor = 0.01);
```

---

## 11. 批量加载比预期慢

**症状**：`pg_ripple.load_turtle()` 或 `pg_ripple.load_ntriples()` 的运行速度远低于文档记录的 50K–200K 三元组/秒。

**原因**：小批量、同步提交开销或 `work_mem` 不足。

**诊断**：

```sql
SHOW synchronous_commit;
SHOW work_mem;
SHOW maintenance_work_mem;
```

**修复**：

```sql
-- 为批量加载禁用同步提交
SET synchronous_commit = off;

-- 增加工作内存
SET work_mem = '256MB';
SET maintenance_work_mem = '2GB';

-- 使用批量加载函数
SELECT pg_ripple.load_turtle_file('/path/to/data.ttl');
```

```admonish warning title="synchronous_commit = off"
禁用同步提交会在崩溃时丢失最后几个事务的风险。仅对可以重新运行的批量加载使用此设置。
```

---

## 12. RDF-Star 解析错误

**症状**：加载 RDF-star 数据失败，出现 `unexpected token` 或 `invalid quoted triple`。

**原因**：输入文件使用 RDF-star 语法（`<<>>`），但解析器未处于 RDF-star 模式，或语法格式错误。

**诊断**：检查报告行号附近文件的语法问题。常见问题：

- 嵌套的 `<<>>` 没有适当的空白
- 引用三元组内的字面量对象缺少数据类型
- 在 N-Triples 文件中使用 Turtle-star 语法（或反之亦然）

**修复**：验证文件使用正确的格式。对于 Turtle-star：

```turtle
<<:Alice :knows :Bob>> :since "2024"^^xsd:gYear .
```

对于 N-Triples-star，每个项都必须完全限定 — 没有前缀。

---

## 13. SHACL 验证队列积压

**症状**：`pg_ripple.validation_queue_depth()` 返回大量数字；验证结果延迟。

**原因**：高写入吞吐量产生的验证速度快于异步验证器处理它们的速度。

**诊断**：

```sql
SELECT pg_ripple.validation_queue_depth();
SELECT pg_ripple.stats();
```

**修复**：

- 增加验证工作线程的处理能力（如果适用）
- 在低流量期间临时切换到同步验证：

```sql
SET pg_ripple.enforce_constraints = 'error';
```

- 减少写入批量大小以给验证器赶上时间

---

## 14. 计划缓存抖动

**症状**：SPARQL 查询延迟不一致。首次执行查询模式很慢，但后续运行很快 — 然后又变慢。

**原因**：计划缓存（`pg_ripple.plan_cache_size`）对于不同查询模式的数量来说太小。计划被驱逐并反复重新编译。

**诊断**：

```sql
SHOW pg_ripple.plan_cache_size;

-- 估计工作负载中不同查询模式的数量
-- （需要应用程序级日志记录）
```

**修复**：

```sql
-- 增加计划缓存
SET pg_ripple.plan_cache_size = 1024;
```

如果不同模式的数量超过任何合理的缓存大小，请考虑对查询进行参数化以减少模式多样性。

---

## 15. "relation _pg_ripple.vp_XXXXX does not exist"

**症状**：SPARQL 查询因特定 VP 表的"关系不存在"错误而失败。

**原因**：谓词目录引用已删除或从未创建的 VP 表。这可能发生在不完整的迁移或手动 DDL 之后。

**诊断**：

```sql
-- 检查谓词目录
SELECT id, table_oid, triple_count
FROM _pg_ripple.predicates
WHERE id = XXXXX;

-- 验证表是否存在
SELECT oid FROM pg_class WHERE oid = (
  SELECT table_oid FROM _pg_ripple.predicates WHERE id = XXXXX
);
```

**修复**：

```sql
-- 为谓词重建 VP 表
SELECT pg_ripple.reindex_predicate(XXXXX);
```

如果数据丢失，应删除谓词条目：

```sql
DELETE FROM _pg_ripple.predicates WHERE id = XXXXX;
```

```admonish danger title="手动编辑目录"
直接修改 `_pg_ripple.predicates` 会绕过完整性检查。仅在确认 VP 表确实丢失后，才将作为最后的手段执行此操作。
```

---

## 16. "permission denied for schema _pg_ripple"

**症状**：非超级用户连接在运行 SPARQL 查询时收到权限错误。

**原因**：用户没有 `_pg_ripple` 和 `pg_ripple` schema 上的 `USAGE`。

**修复**：

```sql
GRANT USAGE ON SCHEMA pg_ripple TO myuser;
GRANT USAGE ON SCHEMA _pg_ripple TO myuser;
GRANT SELECT ON ALL TABLES IN SCHEMA _pg_ripple TO myuser;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA pg_ripple TO myuser;
```

---

## 通用诊断命令

任何故障排除会话的一组快速参考命令：

```sql
-- 扩展健康状况
SELECT pg_ripple.canary();
SELECT pg_ripple.stats();

-- PostgreSQL 活动
SELECT pid, state, query, wait_event_type, wait_event
FROM pg_stat_activity
WHERE datname = current_database();

-- 锁争用
SELECT * FROM pg_locks WHERE NOT granted;

-- _pg_ripple 中的表大小
SELECT relname, pg_size_pretty(pg_total_relation_size(oid))
FROM pg_class
WHERE relnamespace = '_pg_ripple'::regnamespace
ORDER BY pg_total_relation_size(oid) DESC
LIMIT 20;

-- GUC 设置
SELECT name, setting, source
FROM pg_settings
WHERE name LIKE 'pg_ripple.%'
ORDER BY name;
```

---

## 合并后丢失删除（v0.37.0+）

**症状**：在后台合并周期完成后，已删除的三元组仍出现在查询结果中。

**原因**：在 v0.37.0 之前，合并工作线程在 delta→主交换期间未持有每谓词咨询锁。在构建 `main_new` 之后但在墓碑表行记录的截断之前到达的 `DELETE` 将在同一截断中删除其墓碑，从而使三元组在新主中保持活动状态。

**检测**：

```sql
-- 使用 diagnostic_report 检查系统健康状况
SELECT key, value FROM pg_ripple.diagnostic_report()
WHERE key IN ('schema_version', 'merge_backlog_rows');
```

如果 `schema_version` 早于 `0.37.0`，请升级以获取修复。

**修复**：

1. 升级到 v0.37.0 或更高版本：
   ```sql
   ALTER EXTENSION pg_ripple UPDATE TO '0.37.0';
   ```

2. 验证修复是否处于活动状态 — `diagnostic_report()` 报告正确的版本：
   ```sql
   SELECT value FROM pg_ripple.diagnostic_report() WHERE key = 'schema_version';
   -- 应返回：0.37.0
   ```

3. 升级后，合并工作线程在 delta→主交换之前获取 `pg_advisory_xact_lock(pred_id)`（独占），删除路径在插入墓碑之前获取 `pg_advisory_xact_lock_shared(pred_id)`。这两种锁模式不兼容，保证序列化。

**影响**：低 — 需要在合并周期期间出现不凑巧的时机窗口。大多数部署在实践中不会观察到丢失的删除，但关键正确性的工作负载应升级。