# JSON 映射

pg_ripple 的 **JSON 映射**功能提供 JSON 负载和 RDF 知识图之间的双向桥梁。使用 `register_json_mapping()` 一次注册一个命名 JSON-LD 上下文，然后将其用于导入和导出。

## 注册

```sql
SELECT pg_ripple.register_json_mapping(
    'contacts',
    '{"contact_id": "http://schema.org/identifier",
      "full_name":  "http://schema.org/name",
      "email_addr": "http://schema.org/email"}'::jsonb
);
```

参数：

| 参数 | 描述 |
|---|---|
| `name` | 唯一的映射名称 |
| `context` | JSON-LD `@context` 对象，将 JSON 键映射到 RDF 谓词 IRI |
| `shape_iri` | 用于一致性验证的可选 SHACL 形状 |
| `default_graph_iri` | 已引入三元组的默认命名图 |
| `timestamp_path` | 根时间戳字段的 JSONPath（差异模式）|
| `timestamp_predicate` | 每个三元组更改时间戳的 RDF 谓词 |
| `iri_template` | 带有 `{id}` 占位符的 IRI 模板 |
| `iri_match_pattern` | 用于晚期绑定 IRI 重写的前缀或正则表达式 |

## 引入（JSON → RDF）

```sql
SELECT pg_ripple.ingest_json(
    '{"contact_id": "c001", "full_name": "Alice Smith"}'::jsonb,
    'https://example.com/contacts/c001',
    'contacts'
);
```

模式：`'append'`（默认）、`'upsert'`、`'diff'`。

## 导出（RDF → JSON）

```sql
SELECT pg_ripple.export_json_node(subject_id, 'contacts');
```

## 关系回写（v0.128.0）

v0.128.0 添加了将 RDF 图更改回写到原始关系表的功能 — 完成完整的往返。

### 配置

通过经过验证的公共 API 配置关系目标。这将检查关系、键列、冲突策略、权限和目标 PostgreSQL 类型，然后原子地缓存类型转换。对 `_pg_ripple` 的直接更新不是公共 API 的一部分。

```sql
SELECT pg_ripple.configure_json_writeback(
    'contacts', 'public', 'contacts', ARRAY['contact_id'], 'replace'
);
```

检查解析的目标、缓存的列类型、启用状态和队列运行状况：

```sql
SELECT * FROM pg_ripple.writeback_inspect('contacts');
```

### 冲突策略

| 策略 | 行为 |
|---|---|
| `'replace'`（默认）| `ON CONFLICT (key_cols) DO UPDATE SET …` |
| `'skip'` | `ON CONFLICT DO NOTHING` — 返回 0 行 |
| `'error'` | 在冲突时引发 `PT0551` |

### 直接回写

```sql
-- 将单个主语回写到关系表。
SELECT pg_ripple.writeback_json_row('contacts', 'https://example.com/contacts/c001');

-- 从关系表中删除主语。
SELECT pg_ripple.writeback_json_row_delete('contacts', 'https://example.com/contacts/c001');
```

### 基于触发器的自动化

启用 VP delta 触发器以进行自动异步回写：

```sql
SELECT pg_ripple.enable_json_writeback('contacts');
```

后台合并工作线程按批处理排空队列（由 `pg_ripple.json_writeback_batch_size` 控制，默认值为 100）。

监控队列状态：

```sql
SELECT * FROM pg_ripple.json_writeback_status();
--  mapping_name | pending | errors | last_error | last_processed_at
```

### HTTP 回写

HTTP 配套服务为不直接调用 SQL 的应用程序公开相同的回写路径：

```bash
curl -X POST http://localhost:7878/json-mapping/contacts/writeback \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"subject_iri":"https://example.com/contacts/c001"}'
```

成功的同步回写返回：

```json
{"rows_affected": 1}
```

一个映射的队列状态也可通过 HTTP 获得：

```bash
curl http://localhost:7878/json-mapping/contacts/writeback/status \
    -H "Authorization: Bearer $TOKEN"
```

状态响应反映所选映射的 `json_writeback_status()`：

```json
{
    "mapping_name": "contacts",
    "pending": 0,
    "errors": 0,
    "last_error": null,
    "last_processed_at": null
}
```

经过验证的回写配置和队列运行状况也可在以下位置获得：

```bash
curl http://localhost:7878/json-mapping/contacts/writeback/config \
    -H "Authorization: Bearer $TOKEN"
```

禁用触发器：

```sql
SELECT pg_ripple.disable_json_writeback('contacts');
```

`enable_json_writeback()` 和 `disable_json_writeback()` 都是幂等的。

### 错误代码

| 代码 | 消息 |
|---|---|
| `PT0550` | `json mapping writeback target not configured` — `writeback_table` 为 NULL 或 `writeback_key_columns` 为空 |
| `PT0551` | `json mapping writeback conflict` — 使用策略 `'error'` 检测到冲突 |
| `PT0552` | `json mapping writeback key column(s) have no asserted value` — 配置的 `writeback_key_columns` 谓词从未为该主语断言 |

## 另请参阅

- [JSON-LD 反向映射博客文章](https://github.com/trickle-labs/pg-ripple/blob/main/blog/json-ld-reverse-mapping.md)
- [JSON 回写，以及使其实际工作所需的努力](https://github.com/trickle-labs/pg-ripple/blob/main/blog/json-writeback-correctness.md)
- [GUC 参考：json_writeback_batch_size](../reference/guc-reference.md#pg_ripplejson_writeback_batch_size)
- [HTTP API 参考](../reference/http-api.md#json-mapping-writeback)
- [R2RML 用于复杂 ETL](r2rml.md)