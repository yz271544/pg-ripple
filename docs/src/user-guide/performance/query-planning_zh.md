# 查询规划

pg_ripple 在执行之前将 SPARQL 代数转换为 PostgreSQL SQL。
本页介绍如何构建计划以及如何调优它。

## 计划缓存

每个转换后的计划都缓存在每个后端的 LRU 缓存中，键为**代数摘要**（规范化 SPARQL IR 的 XXH3-128，加上 `max_path_depth` 和 `bgp_reorder` 的当前值）。这意味着：

- 同一查询的空白变体和前缀别名变体共享一个缓存条目。
- 更改 `SET pg_ripple.bgp_reorder = off` 会导致缓存未命中并触发重新翻译。
- 缓存在每个后端是局部的，并在连接关闭时清除。

使用以下方式检查缓存健康状况：

```sql
SELECT * FROM pg_ripple.plan_cache_stats();
-- 返回：(hit_count, miss_count, current_size, capacity)
```

重置计数器而不需要重新连接：

```sql
SELECT pg_ripple.reset_plan_cache();
```

## BGP 重排

默认情况下（`pg_ripple.bgp_reorder = on`），SPARQL 优化器对 BGP 中的三元组模式进行排列，以基于每个谓词的三元组计数最小化中间结果大小。禁用以强制从左到右求值顺序：

```sql
SET pg_ripple.bgp_reorder = off;
```

## 谓词目录

谓词目录（`storage/catalog.rs`）缓存谓词 → VP 表 OID 映射，以消除每个谓词每个查询的一次 SPI 查找。对于 10 个原子的 BGP，这将与字典相关的 SPI 开销从 10 减少到 1。

```sql
-- 在架构更改或形状更新后失效：
SELECT pg_ripple.invalidate_catalog_cache();
```

目录缓存在默认情况下启用。引入了调试：

```sql
SET pg_ripple.predicate_cache_enabled = off;
```

## SHACL 驱动的 SQL 提示

加载 SHACL 形状后，SQL 生成器从 `_pg_ripple.shape_hints` 读取每个谓词的提示，以生成更高效的 SQL：

| SHACL 约束 | SQL 优化 |
|---|---|
| `sh:maxCount 1` | 省略 `DISTINCT` — 每个主语最多一个绑定 |
| `sh:minCount 1` | 对 OPTIONAL 使用 `INNER JOIN` 而非 `LEFT JOIN` |

加载形状并验证是否写入了提示：

```sql
SELECT pg_ripple.load_shacl($$ @prefix sh: <...> . @prefix ex: <...> . ex:MyShape a sh:NodeShape ; sh:targetClass ex:Thing ; sh:property [ sh:path ex:name ; sh:maxCount 1 ] . $$);

SELECT * FROM _pg_ripple.shape_hints LIMIT 10;
```

## EXPLAIN SPARQL 查询

使用 `sparql_explain` 查看为查询生成的 SQL：

```sql
SELECT pg_ripple.sparql_explain(
  'SELECT * WHERE { ?s <http://schema.org/name> ?name }',
  analyze := false
);
```

传递 `analyze := true` 以包括实际运行时统计信息（运行查询）。

## 属性路径深度

递归属性路径（`*`、`+`）使用 `WITH RECURSIVE … CYCLE`，具有可配置的深度限制：

```sql
SET pg_ripple.max_path_depth = 32;   -- 默认：64
```

> **注意：** `property_path_max_depth` 是 `max_path_depth` 的已弃用别名，将在未来的版本中删除。

## 用于查询规划的有用 GUC

| GUC | 默认值 | 效果 |
|-----|---------|--------|
| `pg_ripple.bgp_reorder` | `on` | 按选择性重排三元组模式 |
| `pg_ripple.max_path_depth` | `64` | `*` / `+` 路径的最大递归深度 |
| `pg_ripple.predicate_cache_enabled` | `on` | 缓存谓词 → VP 表 OID |
| `pg_ripple.plan_cache_size` | `256` | 每个后端计划缓存的 LRU 容量 |