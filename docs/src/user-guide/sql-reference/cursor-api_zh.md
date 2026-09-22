# 流式游标 API

在处理数百万个三元组时，在一次调用中物化整个 SPARQL 结果集可能会耗尽内存或达到语句级别的行限制。流式游标 API 通过 PostgreSQL `SETOF` 函数返回批量结果。

---

## 函数

### `sparql_cursor(query TEXT) RETURNS SETOF JSONB`（v0.40.0）

将 SPARQL `SELECT` 或 `ASK` 查询的输出作为 JSONB 绑定行序列流式传输。

```sql
SELECT * FROM pg_ripple.sparql_cursor($$
    SELECT ?s ?label WHERE { ?s <https://schema.org/name> ?label }
$$);
```

等效于 `sparql()`，但避免了完全物化。每行是一个 JSON 对象，每个投影的变量都有一个键。

### `sparql_cursor_turtle(query TEXT) RETURNS SETOF TEXT`（v0.40.0）

将 SPARQL `CONSTRUCT` 查询的输出作为 Turtle 文本块流式传输。

```sql
COPY (
    SELECT result FROM pg_ripple.sparql_cursor_turtle($$
        CONSTRUCT { ?s <https://schema.org/name> ?name }
        WHERE    { ?s <https://schema.org/name> ?name }
    $$)
) TO '/tmp/dump.ttl';
```

每个返回的 `TEXT` 值都是一个独立、自包含的 Turtle 序列化批次（最多 1024 个三元组）。

### `sparql_cursor_jsonld(query TEXT) RETURNS SETOF TEXT`（v0.40.0）

将 SPARQL `CONSTRUCT` 查询的输出作为 JSON-LD 扩展形式的块流式传输。

```sql
SELECT string_agg(result, E'\n')
FROM pg_ripple.sparql_cursor_jsonld($$
    CONSTRUCT { ?s ?p ?o } WHERE { GRAPH <https://my.graph/> { ?s ?p ?o } }
END);
```

### `sparql_stream_metadata(query TEXT)` 和类型化绑定（v0.134.0）

内部的 `_pg_ripple.sparql_stream_metadata()` 和 `_pg_ripple.sparql_stream_bindings()` 函数支持 HTTP 流式传输路径。它们在第一行之前公开投影变量并返回 SPARQL JSON 项描述符，保留 IRI、空白节点、数据类型、语言标记和未绑定变量。它们是内部实现函数，不是稳定公共 SQL API 的一部分。

---

## 溢出控制 GUC

| GUC | 默认值 | 描述 |
|-----|---------|-------------|
| `pg_ripple.sparql_max_rows` | `0`（无限制） | `sparql()` 和 `sparql_cursor()` 返回的最大行数。超过时，行为由 `sparql_overflow_action` 控制。 |
| `pg_ripple.export_max_rows` | `0`（无限制） | Turtle/N-Triples/JSON-LD 导出函数返回的最大行数。超过时，发出 PT642 WARNING 并截断结果。 |
| `pg_ripple.sparql_overflow_action` | `''`（warn） | `'warn'` — 发出 PT640 WARNING 并截断。`'error'` — 引发 PT640 ERROR。 |
| `pg_ripple.datalog_max_derived` | `0`（无限制） | 单个 `infer()` 调用生成的最大派生事实数。超过时，发出 PT641 WARNING。 |

**示例 — 将大型导出限制为 50,000 行：**

```sql
SET pg_ripple.export_max_rows = 50000;
SELECT string_agg(result, E'\n')
FROM pg_ripple.sparql_cursor_turtle('CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }');
```

**示例 — 在溢出时出错而非静默截断：**

```sql
SET pg_ripple.sparql_max_rows = 100000;
SET pg_ripple.sparql_overflow_action = 'error';
SELECT * FROM pg_ripple.sparql_cursor('SELECT ?s WHERE { ?s ?p ?o }');
```

---

## 性能说明

- 每轮迭代处理 1024 行；内存占用为 `O(batch_size)`。
- 对于非常大的 CONSTRUCT 导出，请考虑使用 `COPY ... TO` 与 `sparql_cursor_turtle` 以避免在客户端中缓冲。
- 游标函数调用与 `sparql()` 相同的 SPARQL→SQL 管道；计划缓存命中适用。

---

## 另请参阅

- [Explain API](explain.md) — 内省 SPARQL 查询计划
- [Observability](../../operations/monitoring.md) — 跟踪和缓存统计信息
- [Error Reference](../../reference/error-catalog.md) — PT640、PT641、PT642