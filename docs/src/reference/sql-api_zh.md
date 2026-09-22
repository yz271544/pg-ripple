# SQL API 参考

精选的 pg_ripple SQL 函数，包含完整参数说明、返回架构和可复制粘贴的示例。有关所有 157 个函数的完整字母顺序列表，请参阅 [sql-functions.md](sql-functions.md)。

---

## 兼容性和诊断

### `compat_check`

返回一个 JSON 对象，描述已安装的扩展版本及其与 HTTP 配套服务的兼容性。

```sql
pg_ripple.compat_check() → TEXT
```

**返回** — 一个 JSON 字符串，包含以下键：

| 键 | 类型 | 描述 |
|-----|------|-------------|
| `extension_version` | `STRING` | 已安装的 pg_ripple 扩展版本，例如 `"0.136.0"` |
| `http_min_version` | `STRING` | 此扩展所需的最低 pg_ripple_http 版本，例如 `"0.135.0"` |
| `compatible` | `BOOL` | 当运行的 HTTP 配套服务满足 `http_min_version` 时为 `true` |

**示例返回值：**

```json
{
  "extension_version": "0.136.0",
  "http_min_version": "0.135.0",
  "compatible": true
}
```

**示例用法：**

```sql
SELECT pg_ripple.compat_check();
-- {"extension_version":"0.136.0","http_min_version":"0.135.0","compatible":true}

-- 解析为 JSON：
SELECT
  (pg_ripple.compat_check()::jsonb) ->> 'extension_version' AS ext_version,
  (pg_ripple.compat_check()::jsonb) ->> 'compatible'        AS compatible;
```

---

## 基准测试

### `bench_workload`

针对本地三元组存储运行基准测试工作负载配置文件，并将结果记录在 `_pg_ripple.bench_history` 中。

```sql
pg_ripple.bench_workload(
    profile TEXT DEFAULT 'bsbm'
) → BIGINT
```

**参数：**

| 参数 | 类型 | 默认值 | 描述 |
|-----------|------|---------|-------------|
| `profile` | `TEXT` | `'bsbm'` | 要运行的基准测试配置文件。之一：`'bsbm'`、`'watdiv'`、`'pagerank'`、`'pprl'` |

**返回** — 新插入的 `_pg_ripple.bench_history` 行的 `run_id`。

```sql
-- 运行 BSBM 基准测试：
SELECT pg_ripple.bench_workload('bsbm');

-- 运行 WatDiv 基准测试：
SELECT pg_ripple.bench_workload('watdiv');
```

### `bench_workload_result`

返回给定配置文件的最新基准测试运行。`_pg_ripple.bench_history` 的便捷包装 — 无需原始 SQL。

```sql
pg_ripple.bench_workload_result(
    profile TEXT DEFAULT 'bsbm'
) → TABLE (
    run_id              BIGINT,
    profile             TEXT,
    started_at          TIMESTAMPTZ,
    duration_ms         BIGINT,
    queries_per_second  FLOAT8,
    triples_processed   BIGINT
)
```

**参数：**

| 参数 | 类型 | 默认值 | 描述 |
|-----------|------|---------|-------------|
| `profile` | `TEXT` | `'bsbm'` | 要筛选的配置文件名；必须与先前传递给 `bench_workload()` 的值匹配 |

**示例用法：**

```sql
-- 运行基准测试并立即检索结果：
SELECT pg_ripple.bench_workload('bsbm');
SELECT * FROM pg_ripple.bench_workload_result('bsbm');

-- 输出：
--  run_id | profile | started_at               | duration_ms | queries_per_second | triples_processed
--  -------+---------+--------------------------+-------------+--------------------+------------------
--       1 | bsbm    | 2026-05-19 12:00:00+00   |         342 |             175.44 |             50000
```

---

## 规则库联邦

### `publish_rule_library`

发布已安装的规则库，以便可以通过 HTTP 配套服务的规则库流端点从远程 pg_ripple 实例订阅它。

```sql
pg_ripple.publish_rule_library(
    name         TEXT,
    endpoint_uri TEXT
) → VOID
```

**参数：**

| 参数 | 类型 | 描述 |
|-----------|------|-------------|
| `name` | `TEXT` | 已安装规则库的名称（1–64 个字母数字/连字符/下划线字符） |
| `endpoint_uri` | `TEXT` | 将提供规则流的完整 HTTP URI，例如 `'https://host/rule-libraries/my-lib/stream'` |

**错误** — 对于无效的名称/URI、缺失的库或目录写入失败，会引发 `PT046x` 错误代码。

```sql
-- 在此实例上发布 "rdfs-base" 库：
SELECT pg_ripple.publish_rule_library(
    'rdfs-base',
    'https://instance-a.example.com/rule-libraries/rdfs-base/stream'
);
```

### `subscribe_rule_library`

记录订阅远程规则库流的意图。源 URI 必须通过 SSRF 策略检查。SQL 函数不执行出站网络 I/O；获取和安装远程规则由 HTTP 配套服务的 `POST /rule-libraries/{name}/subscribe` 端点执行。

```sql
pg_ripple.subscribe_rule_library(
    source_uri TEXT,
    name       TEXT
) → VOID
```

**参数：**

| 参数 | 类型 | 描述 |
|-----------|------|-------------|
| `source_uri` | `TEXT` | 远程流端点的完整 HTTP URI |
| `name` | `TEXT` | 用于已订阅规则库的本地名称 |

**错误** — 对于 SSRF 阻止的 URI、无效的名称或目录写入失败，会引发 `PT046x` 错误代码。

```sql
-- 记录对在实例 A 上发布的规则库的订阅意图：
SELECT pg_ripple.subscribe_rule_library(
    'https://instance-a.example.com/rule-libraries/rdfs-base/stream',
    'rdfs-base'
);
```

---

## Allen 区间关系

所有七个 Allen 区间关系都可用作 SQL 函数和 SPARQL 扩展函数（`pg:before`、`pg:meets` 等）。

每个函数接受四个 `TIMESTAMPTZ` 参数，表示两个区间的开始和结束：`(a_start, a_end, b_start, b_end)`。

| SQL 函数 | SPARQL IRI | 关系 |
|---|---|---|
| `pg_ripple.allen_before` | `pg:before` | A 在 B 开始之前严格结束 |
| `pg_ripple.allen_meets` | `pg:meets` | A 在 B 开始时准确结束 |
| `pg_ripple.allen_overlaps` | `pg:overlaps` | A 在 B 之前开始并且它们重叠 |
| `pg_ripple.allen_during` | `pg:during` | A 完全在 B 内 |
| `pg_ripple.allen_finishes` | `pg:finishes` | A 与 B 同时结束 |
| `pg_ripple.allen_starts` | `pg:starts` | A 与 B 同时开始 |
| `pg_ripple.allen_equals` | `pg:equals` | A 和 B 具有相同的边界 |

### `allen_before`

```sql
pg_ripple.allen_before(
    a_start TIMESTAMPTZ,
    a_end   TIMESTAMPTZ,
    b_start TIMESTAMPTZ,
    b_end   TIMESTAMPTZ
) → BOOLEAN
```

当区间 A 在区间 B 开始之前严格结束时返回 `true`（`a_end < b_start`）。

```sql
SELECT pg_ripple.allen_before(
    '2026-01-01'::timestamptz, '2026-01-15'::timestamptz,
    '2026-02-01'::timestamptz, '2026-02-28'::timestamptz
);  -- true
```

### `allen_meets`

```sql
pg_ripple.allen_meets(
    a_start TIMESTAMPTZ,
    a_end   TIMESTAMPTZ,
    b_start TIMESTAMPTZ,
    b_end   TIMESTAMPTZ
) → BOOLEAN
```

当区间 A 在 B 开始的同一时刻结束时返回 `true`（`a_end = b_start`）。

### `allen_overlaps`

```sql
pg_ripple.allen_overlaps(
    a_start TIMESTAMPTZ,
    a_end   TIMESTAMPTZ,
    b_start TIMESTAMPTZ,
    b_end   TIMESTAMPTZ
) → BOOLEAN
```

当 A 在 B 之前开始并且区间重叠但彼此不包含时返回 `true`（`a_start < b_start AND a_end > b_start AND a_end < b_end`）。

### `allen_during`

```sql
pg_ripple.allen_during(
    a_start TIMESTAMPTZ,
    a_end   TIMESTAMPTZ,
    b_start TIMESTAMPTZ,
    b_end   TIMESTAMPTZ
) → BOOLEAN
```

当区间 A 完全在区间 B 内时返回 `true`（`b_start < a_start AND a_end < b_end`）。

### `allen_finishes`

```sql
pg_ripple.allen_finishes(
    a_start TIMESTAMPTZ,
    a_end   TIMESTAMPTZ,
    b_start TIMESTAMPTZ,
    b_end   TIMESTAMPTZ
) → BOOLEAN
```

当 A 与 B 同时结束但开始较晚时返回 `true`（`a_start > b_start AND a_end = b_end`）。

### `allen_starts`

```sql
pg_ripple.allen_starts(
    a_start TIMESTAMPTZ,
    a_end   TIMESTAMPTZ,
    b_start TIMESTAMPTZ,
    b_end   TIMESTAMPTZ
) → BOOLEAN
```

当 A 与 B 同时开始但结束较早时返回 `true`（`a_start = b_start AND a_end < b_end`）。

### `allen_equals`

```sql
pg_ripple.allen_equals(
    a_start TIMESTAMPTZ,
    a_end   TIMESTAMPTZ,
    b_start TIMESTAMPTZ,
    b_end   TIMESTAMPTZ
) → BOOLEAN
```

当两个区间相同时返回 `true`（`a_start = b_start AND a_end = b_end`）。

**使用区间关系的 SPARQL 示例：**

```sparql
PREFIX pg: <https://pgrdf.io/fn/>
PREFIX ex: <https://example.org/>

SELECT ?event ?label WHERE {
  ?event ex:startTime ?s ;
         ex:endTime   ?e ;
         ex:label     ?label .
  FILTER(pg:before(?s, ?e,
                   "2026-06-01T00:00:00Z"^^xsd:dateTime,
                   "2026-12-31T23:59:59Z"^^xsd:dateTime))
}
```

---

## 另请参阅

- [完整 SQL 函数列表](sql-functions.md)
- [GUC 参考](guc-reference.md)
- [HTTP API 参考](http-api.md)