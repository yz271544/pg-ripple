# 字典

字典使用 XXH3-128 哈希将每个 IRI、空白节点和字面量映射到 `BIGINT` ID。VP 表仅存储 `BIGINT` 值 — 数据表中不显示原始字符串。

这些函数适用于需要直接与字典交互的 API 用户。大多数应用程序应该使用替代策略的高级 SPARQL 和 CRUD 函数。

## encode_term

```sql
pg_ripple.encode_term(term TEXT) RETURNS BIGINT
```

编码 IRI、空白节点或字面量（以 N-Triples 形式），并返回其字典 ID。如果术语尚未在字典中，则插入它。

```sql
SELECT pg_ripple.encode_term('<https://example.org/alice>');
-- 返回 BIGINT ID

SELECT pg_ripple.encode_term('"Alice"@en');
-- 返回语言标记字面量的 BIGINT ID
```

## decode_id

```sql
pg_ripple.decode_id(id BIGINT) RETURNS TEXT
```

查找字典 ID 并以 N-Triples 形式返回原始术语。

```sql
SELECT pg_ripple.decode_id(1);
-- 返回：'<https://example.org/alice>'
```

## decode_id_full（v0.15.0）

```sql
pg_ripple.decode_id_full(id BIGINT) RETURNS JSONB
```

返回带有详细类型信息的结构化 JSONB 对象。对于调试和内省，比 `decode_id()` 更有信息量。

| 字段 | 描述 |
|-------|-------------|
| `kind` | 术语类型：`"iri"`、`"bnode"`、`"literal"`、`"default_graph"`、`"quoted_triple"` |
| `value` | 术语值（IRI 字符串、字面量文本、空白节点标签） |
| `datatype` | XSD 数据类型 IRI（对于类型化字面量）或 `null` |
| `language` | 语言标记（对于语言标记字面量）或 `null` |

```sql
SELECT pg_ripple.decode_id_full(42);
-- {"kind": "literal", "value": "Alice", "datatype": null, "language": "en"}

SELECT pg_ripple.decode_id_full(1);
-- {"kind": "iri", "value": "https://example.org/alice", "datatype": null, "language": null}
```

## lookup_iri（v0.15.0）

```sql
pg_ripple.lookup_iri(iri TEXT) RETURNS BIGINT
```

检查 IRI 是否存在于字典中，并返回其 ID。如果 IRI 从未编码，则返回 `NULL`。与 `encode_term()` 不同，这从不插入 — 它是只读查找。

```sql
SELECT pg_ripple.lookup_iri('<https://example.org/alice>');
-- 返回：1（或 NULL，如果在字典中）
```

用于在查询之前检查资源是否存在：

```sql
DO $$
BEGIN
  IF pg_ripple.lookup_iri('<https://example.org/alice>') IS NOT NULL THEN
    RAISE NOTICE 'Alice 存在于存储中';
  END IF;
END $$;
```

## 内部表：_pg_ripple.dictionary

```sql
TABLE _pg_ripple.dictionary (
    id    BIGINT PRIMARY KEY,
    kind  SMALLINT NOT NULL,  -- 1=IRI, 2=BNode, 3=Literal, 4=Default, 5=QuotedTriple
    value TEXT NOT NULL,
    hash_hi BIGINT,
    hash_lo BIGINT,
    qt_s BIGINT,  -- 对于 kind=5：主语分量
    qt_p BIGINT,  -- 对于 kind=5：谓词分量
    qt_o BIGINT   -- 对于 kind=5：宾语分量
)
```

`kind` 列编码术语类型：

| Kind | 含义 |
|---|---|
| 1 | IRI |
| 2 | 空白节点 |
| 3 | 字面量 |
| 4 | 默认图（ID 0） |
| 5 | 带引号的三元组（RDF-star） |