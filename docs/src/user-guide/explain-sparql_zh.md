# SPARQL 查询调试器 — EXPLAIN SPARQL

pg_ripple v0.50.0 扩展了 `pg_ripple.explain_sparql()`，提供了交互式查询调试器模式（`analyze := true`），将代数树、生成的 SQL、计划缓存状态和每个操作符的行计数公开为结构化的 JSONB 文档。

---

## 函数签名

```sql
pg_ripple.explain_sparql(
    query       TEXT,
    analyze     BOOL DEFAULT FALSE
) RETURNS JSONB
```

---

## 输出 Schema

| 键 | 类型 | 描述 |
|-----|------|-------------|
| `algebra` | text | spargebra 代数树（字符串表示） |
| `sql` | text | 发送到 PostgreSQL 的生成的 SQL |
| `plan` | text | PostgreSQL EXPLAIN 输出（JSON 格式） |
| `cache_status` | text | `"hit"`、`"miss"` 或 `"bypass"` |
| `cache_hit` | bool | `cache_status = "hit"` 的旧别名 |
| `actual_rows` | array | 每个操作符的实际行计数（仅当 `analyze = true` 时） |
| `topn_applied` | bool | 是否应用了 TopN 下推优化 |
| `encode_calls` | int | 翻译期间的字典编码调用次数 |

### `cache_status` 值

| 值 | 含义 |
|-------|---------|
| `"hit"` | SQL 是从后端计划缓存中提供 |
| `"miss"` | SQL 是新编译的；查询尚未缓存 |
| `"bypass"` | 计划缓存已禁用（`pg_ripple.plan_cache_size = 0`） |

### `actual_rows`

仅在 `analyze := true` 时存在。包含从 PostgreSQL `EXPLAIN ANALYZE` JSON 输出中提取的实际行计数扁平数组 — 每个计划节点一个整数，按计划树顺序排列。

---

## 示例

### 基本解释（不执行）

```sql
SELECT pg_ripple.explain_sparql(
    'SELECT ?name WHERE { ?s <http://schema.org/name> ?name }',
    false
) AS explain_out;
```

### 交互式调试器（带行计数）

```sql
WITH result AS (
    SELECT pg_ripple.explain_sparql(
        'SELECT ?name WHERE { ?s <http://schema.org/name> ?name }',
        true
    ) AS j
)
SELECT
    j->>'cache_status'  AS cache_status,
    j->>'sql'           AS generated_sql,
    j->'actual_rows'    AS actual_rows_per_operator
FROM result;
```

### 所有四种查询类型

```sql
-- SELECT
SELECT pg_ripple.explain_sparql('SELECT * WHERE { ?s ?p ?o } LIMIT 5', true);

-- ASK
SELECT pg_ripple.explain_sparql('ASK { <http://example.org/a> ?p ?o }', true);

-- CONSTRUCT
SELECT pg_ripple.explain_sparql(
    'CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o } LIMIT 5', true);

-- DESCRIBE（返回代数 + 合成的 SQL 存根；无错误）
SELECT pg_ripple.explain_sparql('DESCRIBE <http://example.org/a>', false);
```

---

## 解读代数树

`algebra` 字段是内部 `spargebra` 代数树的字符串表示。它显示 SPARQL 引擎如何在 SQL 生成之前解析和构建您的查询。常见节点：

- `Project` — 变量投影
- `Filter` — FILTER 表达式
- `Join` / `LeftJoin` — 三元组模式连接 / OPTIONAL
- `BGP` — 基本图模式（三元组模式列表）
- `Union` — UNION 子句
- `Extend` — BIND 表达式

---

## 使用 EXPLAIN ANALYZE 调优

使用 `actual_rows` 比较估计与实际行计数：

```sql
WITH debug AS (
    SELECT pg_ripple.explain_sparql(
        'SELECT ?s WHERE { ?s <http://schema.org/type> <http://schema.org/Person> }',
        true
    ) AS j
)
SELECT
    jsonb_array_length(j->'actual_rows') AS num_plan_nodes,
    j->'actual_rows'                      AS row_counts
FROM debug;
```

如果行估计与实际相差很大，请在 VP 表上运行 `ANALYZE`：

```sql
ANALYZE _pg_ripple.vp_rare;
```

---

## 配置

| GUC | 默认值 | 对 explain 的影响 |
|-----|---------|-------------------|
| `pg_ripple.plan_cache_size` | `256` | 设置为 `0` 强制 `cache_status = "bypass"` |
| `pg_ripple.max_path_depth` | `10` | 影响属性路径 SQL 深度；更改缓存键 |