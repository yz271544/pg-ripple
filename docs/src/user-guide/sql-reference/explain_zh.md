# Explain API

pg_ripple 公开两个 `explain_sparql` 重载和 `explain_datalog` 用于内省查询计划。

---

## `explain_sparql(query, format)` — 文本输出（v0.23.0）

返回 SPARQL 查询计划的人类可读或结构化文本表示。

```sql
pg_ripple.explain_sparql(query TEXT, format TEXT DEFAULT 'text') RETURNS TEXT
```

**格式选项：**

| 格式 | 描述 |
|--------|-------------|
| `'text'`（默认） | 对生成的 SQL 运行 `EXPLAIN` 并以文本形式返回计划 |
| `'json'` | 运行 `EXPLAIN FORMAT JSON` 并以 JSON 文本形式返回计划 |
| `'sql'` | 返回生成的 SQL 而不运行 `EXPLAIN` |
| `'sparql_algebra'` | 返回已解析的 SPARQL 代数树（Debug 格式） |

**示例：**

```sql
SELECT pg_ripple.explain_sparql(
    'SELECT ?s ?p ?o WHERE { ?s ?p ?o }',
    'sql'
);
```

---

## `explain_sparql(query, analyze)` — JSONB 输出（v0.40.0）

返回包含完整解释管道的机器可读 JSONB 文档。

```sql
pg_ripple.explain_sparql(query TEXT, analyze BOOLEAN DEFAULT false) RETURNS JSONB
```

**返回结构：**

```json
{
  "algebra":   "<spargebra debug output>",
  "sql":       "<generated SQL>",
  "plan":      "<EXPLAIN [ANALYZE] output>",
  "cache_hit": true,
  "encode_calls": 0
}
```

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `algebra` | string | 以 Rust Debug 格式解析的 SPARQL 代数树 |
| `sql` | string | 将要执行的生成的 SQL |
| `plan` | string | PostgreSQL `EXPLAIN [ANALYZE]` 输出 |
| `cache_hit` | boolean | 已编译计划是否从计划缓存中提供 |
| `encode_calls` | number | 字典编码器调用次数（使用计划缓存时为 0） |

当 `analyze = true` 时，运行 `EXPLAIN ANALYZE`，计划包括实际时序。

**示例：**

```sql
SELECT pg_ripple.explain_sparql(
    'SELECT ?name WHERE { ?s <https://schema.org/name> ?name }',
    false
) ->> 'sql';
```

---

## `explain_datalog(rule_set_name)` — JSONB 输出（v0.40.0）

返回命名 Datalog 规则集的 JSONB 内省文档。

```sql
pg_ripple.explain_datalog(rule_set_name TEXT) RETURNS JSONB
```

**返回结构：**

```json
{
  "strata": [["rule1", "rule2"], ["rule3"]],
  "rules": ["head(?x, ?y) :- body(?x, ?y) ."],
  "sql_per_rule": ["INSERT INTO ... SELECT ..."],
  "last_run_stats": [{"rule_set": "...", "derived_count": 42, "elapsed_ms": 3}]
}
```

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `strata` | 数组的数组 | 分层结果 — 每个内部数组是一个层 |
| `rules` | 字符串数组 | 存储在 `_pg_ripple.rules` 中的规则文本 |
| `sql_per_rule` | 字符串数组 | 按相同顺序为每个规则编译的 SQL |
| `last_run_stats` | 对象数组 | 来自最近 `infer()` 运行的统计信息（来自 `_pg_ripple.inference_stats`） |

当规则集不存在时返回 `{"strata": [], "rules": [], "sql_per_rule": [], "last_run_stats": []}`。

**示例：**

```sql
SELECT jsonb_pretty(pg_ripple.explain_datalog('my_rules'));
```

---

## 另请参阅

- [Streaming Cursor API](cursor-api.md) — 流式传输大型结果集
- [Observability](../../operations/monitoring.md) — 跟踪和缓存统计信息
- [GUC Reference](../../reference/guc-reference.md) — `pg_ripple.sparql_plan_cache`、`pg_ripple.rule_plan_cache`