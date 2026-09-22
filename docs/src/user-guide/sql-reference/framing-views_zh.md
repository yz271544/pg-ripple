# 框架视图（v0.17.0）

框架视图将 JSON-LD 框架与 pg_trickle 相结合，以创建实时的、增量维护的流表。框架视图将您的框架一次性转换为 SPARQL CONSTRUCT 查询，并将其注册到 pg_trickle；每当插入或删除三元组时，仅重新扫描框架引用的 VP 表。

> **需要 pg_trickle。** 调用 `pg_ripple.pg_trickle_available()` 进行检查。所有函数在调用时都会在缺少 pg_trickle 时引发描述性错误；扩展加载永远不会失败。

## `create_framing_view`

```sql
pg_ripple.create_framing_view(
    name          TEXT,
    frame         JSONB,
    schedule      TEXT    DEFAULT '5s',
    decode        BOOLEAN DEFAULT FALSE,
    output_format TEXT    DEFAULT 'jsonld'
) RETURNS void
```

创建 pg_trickle 流表 `pg_ripple.framing_view_{name}`，其 schema 为：

```
subject_id   BIGINT       -- 字典编码的主语 IRI
frame_tree   JSONB        -- 此根节点的完全嵌入 + 压缩的 JSON-LD
refreshed_at TIMESTAMPTZ
```

当 `decode = TRUE` 时，还会创建一个伴随视图 `pg_ripple.framing_view_{name}_decoded`。它将 `subject_id` 解码为人类可读的 IRI 字符串。

```sql
-- 创建每 10 秒刷新一次的实时公司目录。
SELECT pg_ripple.create_framing_view(
    'companies',
    '{
        "@context": {"schema": "https://schema.org/"},
        "@type": "https://schema.org/Organization",
        "https://schema.org/name": {},
        "https://schema.org/employee": {
            "https://schema.org/name": {}
        }
    }'::jsonb,
    '10s',
    TRUE
);

-- 像常规表一样查询它。
SELECT subject_id, frame_tree FROM pg_ripple.framing_view_companies;
```

## `drop_framing_view`

```sql
pg_ripple.drop_framing_view(name TEXT) RETURNS BOOLEAN
```

删除流表、其可选的解码视图和目录条目。

```sql
SELECT pg_ripple.drop_framing_view('companies');
```

## `list_framing_views`

```sql
pg_ripple.list_framing_views() RETURNS JSONB
```

返回所有已注册的框架视图作为 JSONB 数组，按创建时间排序。每个条目包括 `name`、`frame`、`schedule`、`output_format`、`decode` 和 `created_at`。

```sql
SELECT pg_ripple.list_framing_views();
```

## 刷新模式选择

根据您的用例选择刷新模式：

| 刷新模式 | 使用场景 |
|---|---|
| `IMMEDIATE` | 约束风格的框架：任何匹配的节点都是违规（例如缺少合规官的公司）。在与 DML 相同的事务中触发。 |
| `DIFFERENTIAL` + schedule | 仪表板/ API 用例：仅重新处理已更改的主语。适合每 10 秒刷新一次的公司目录。 |
| `FULL` + 长 schedule | 数据仓库的大型全图框架导出。深度嵌套或 `@always` 嵌入是安全的。 |

## 解码选项

`decode = TRUE` 选项创建一个瘦调用函数，它调用 `pg_ripple.decode_iri(subject_id)` 将主语 IRI 公开为人类可读的字符串。流表本身存储整数 ID 以最小化变更数据捕获（CDC）表面。

```sql
-- 查询解码视图（需要在创建时设置 decode = TRUE）。
SELECT subject_iri, frame_tree FROM pg_ripple.framing_view_companies_decoded;
```

## 目录表

所有框架视图都记录在 `_pg_ripple.framing_views` 中：

| 列 | 类型 | 描述 |
|---|---|---|
| `name` | VARCHAR | 视图名称（主键） |
| `frame` | JSONB | 原始框架文档 |
| `generated_construct` | TEXT | pg_trinkle 使用的 SPARQL CONSTRUCT 字符串 |
| `schedule` | TEXT | pg_trinkle 刷新计划 |
| `output_format` | TEXT | `jsonld`、`ndjson` 或 `turtle` |
| `decode` | BOOLEAN | 是否创建了解码视图 |
| `created_at` | TIMESTAMPTZ | 创建时间戳 |

## pg_trickle 依赖

`create_framing_view()` 和 `drop_framing_view()` 在调用时检查 pg_trickle。如果缺失，它们会引发：

```
ERROR: pg_trickle is required for framing views — install pg_trickle and add it to
shared_preload_libraries, then retry
```

由于缺少 pg_trickle，扩展加载永远不会失败。有关安装说明，请参阅 [pg_trickle](https://github.com/trickle-labs/pg-trickle)。