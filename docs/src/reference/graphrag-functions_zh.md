# GraphRAG 导出函数

这些函数将 GraphRAG 知识图谱数据从 pg_ripple 导出到与 Microsoft GraphRAG 管道兼容的 Parquet 文件。

所有函数都需要 **superuser** 并返回写入的行数。

---

## `export_graphrag_entities`

```sql
pg_ripple.export_graphrag_entities(
    graph_iri  TEXT,   -- 命名图 IRI，或 '' 表示默认图
    output_path TEXT   -- 输出 .parquet 文件的绝对路径
) RETURNS BIGINT
```

导出所有 `gr:Entity` 实例。

**输出列：**

| 列 | 类型 | 描述 |
|---|---|---|
| `id` | `BYTE_ARRAY` | 实体 IRI |
| `title` | `BYTE_ARRAY` | `gr:title` 值 |
| `type` | `BYTE_ARRAY` | `gr:type` 值 |
| `description` | `BYTE_ARRAY` | `gr:description` 值 |
| `text_unit_ids` | `BYTE_ARRAY` | JSON 数组占位符（始终为 `"[]"`） |
| `frequency` | `INT64` | `gr:frequency` 值，默认 0 |
| `degree` | `INT64` | `gr:degree` 值，默认 0 |

**示例：**

```sql
SELECT pg_ripple.export_graphrag_entities(
    'https://myapp.org/graphs/graphrag',
    '/var/data/entities.parquet'
);
```

---

## `export_graphrag_relationships`

```sql
pg_ripple.export_graphrag_relationships(
    graph_iri  TEXT,
    output_path TEXT
) RETURNS BIGINT
```

导出所有 `gr:Relationship` 实例。

**输出列：**

| 列 | 类型 | 描述 |
|---|---|---|
| `id` | `BYTE_ARRAY` | 关系 IRI |
| `source` | `BYTE_ARRAY` | `gr:source` 实体 IRI |
| `target` | `BYTE_ARRAY` | `gr:target` 实体 IRI |
| `description` | `BYTE_ARRAY` | `gr:description` 值 |
| `weight` | `DOUBLE` | `gr:weight` 值，默认 0.0 |
| `combined_degree` | `INT64` | 占位符，始终为 0 |
| `text_unit_ids` | `BYTE_ARRAY` | JSON 数组占位符 |

---

## `export_graphrag_text_units`

```sql
pg_ripple.export_graphrag_text_units(
    graph_iri  TEXT,
    output_path TEXT
) RETURNS BIGINT
```

导出所有 `gr:TextUnit` 实例。

**输出列：**

| 列 | 类型 | 描述 |
|---|---|---|
| `id` | `BYTE_ARRAY` | 文本单元 IRI |
| `text` | `BYTE_ARRAY` | `gr:text` 值 |
| `n_tokens` | `INT64` | `gr:tokenCount` 值，默认 0 |
| `document_id` | `BYTE_ARRAY` | `gr:documentId` 值 |
| `entity_ids` | `BYTE_ARRAY` | JSON 数组占位符 |
| `relationship_ids` | `BYTE_ARRAY` | JSON 数组占位符 |

---

## 注意事项

- **图 IRI：** 传递 `''`（空字符串）以查询默认图（所有没有命名图分配的三元组）。传递完整 IRI 以限制为命名图。
- **路径安全性：** 输出路径不能包含 `..` 组件或空字节。目录必须已经存在。
- **Parquet 编码：** 使用 Snappy 压缩。必填字段的列为 `REQUIRED BYTE_ARRAY`，可选字段的列为 `OPTIONAL BYTE_ARRAY / INT64 / DOUBLE`。
- **需要超级用户：** 因为该函数写入文件系统。

## 另请参阅

- [GraphRAG 用户指南](../features/graphrag.md)
- [GraphRAG 本体](graphrag-ontology.md)
- [GraphRAG 富化](../features/graphrag.md)