# 三元组 CRUD

## insert_triple

```sql
pg_ripple.insert_triple(
    subject   TEXT,
    predicate TEXT,
    object    TEXT,
    graph     TEXT DEFAULT NULL
) RETURNS BIGINT
```

插入单个三元组并返回其全局唯一的**语句标识符**（SID）。如果 `graph` 为 `NULL`，则将三元组插入默认图（ID 0）。

如果谓词的不同三元组少于 `pg_ripple.vp_promotion_threshold`（默认值：1000），则将其存储在共享的 `_pg_ripple.vp_rare` 表中；否则它将获得自己的 VP 表 `_pg_ripple.vp_{predicate_id}`。

**示例：**

```sql
SELECT pg_ripple.insert_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
);
-- 返回：1  (SID)
```

---

## delete_triple

```sql
pg_ripple.delete_triple(
    subject   TEXT,
    predicate TEXT,
    object    TEXT,
    graph     TEXT DEFAULT NULL
) RETURNS BIGINT
```

删除匹配给定（主语、谓词、对象、图）模式的所有三元组，其中 `NULL` 是通配符。返回已删除的三元组数量。

**示例：**

```sql
-- 删除特定三元组
SELECT pg_ripple.delete_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
);

-- 删除具有给定主语的所有三元组
SELECT pg_ripple.delete_triple('<https://example.org/alice>', NULL, NULL);
```

---

## find_triples

```sql
pg_ripple.find_triples(
    subject   TEXT DEFAULT NULL,
    predicate TEXT DEFAULT NULL,
    object    TEXT DEFAULT NULL,
    graph     TEXT DEFAULT NULL
) RETURNS TABLE(subject TEXT, predicate TEXT, object TEXT, graph_id BIGINT)
```

返回匹配模式的所有三元组。`NULL` 是任何位置的通配符。

**示例：**

```sql
-- 按主语搜索
SELECT * FROM pg_ripple.find_triples('<https://example.org/alice>', NULL, NULL);

-- 按谓词搜索
SELECT * FROM pg_ripple.find_triples(NULL, '<https://example.org/knows>', NULL);

-- 查找精确的三元组
SELECT * FROM pg_ripple.find_triples(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
);
```

---

## triple_count

```sql
pg_ripple.triple_count() RETURNS BIGINT
```

返回所有图和所有 VP 表（包括专用的和 `vp_rare`）的三元组总数。

```sql
SELECT pg_ripple.triple_count();
-- 对于空存储返回 0
```