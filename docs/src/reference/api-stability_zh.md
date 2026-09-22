# API 稳定性

此页面定义自 v0.20.0 起生效的 pg_ripple 公共 API 的稳定性合约。

---

## 稳定性层级

pg_ripple 函数和表分为三个层级：

| 层级 | schema 前缀 | 合约 |
|---|---|---|
| **稳定** | `pg_ripple.*` | 在 1.x 发布系列中不会以任何不兼容的方式更改。 |
| **内部** | `_pg_ripple.*` | 私有实现细节。可能在任何次要版本上更改。不要在应用程序代码中依赖它。 |
| **实验性** | 在源代码中标记为 `-- EXPERIMENTAL` | 可能会在没有通知的情况下更改。 |

---

## 稳定的公共 API（`pg_ripple.*`）

以下函数自 v0.20.0 起为**稳定的**，将通过 v1.x 在没有不兼容更改的情况下进行维护：

### 三元组存储核心

| 函数 | 签名 | 注释 |
|---|---|---|
| `insert_triple` | `(s text, p text, o text) → bigint` | 返回语句 ID（SID）。 |
| `delete_triple` | `(s text, p text, o text) → bigint` | 返回已删除的行数。 |
| `find_triples` | `(s text, p text, o text) → setof jsonb` | NULL = 通配符。 |
| `triple_count` | `() → bigint` | 默认图中的总显式三元组。 |
| `load_ntriples` | `(data text) → bigint` | 从 N-Triples 字符串批量加载。 |
| `load_turtle` | `(data text) → bigint` | 从 Turtle 字符串批量加载。 |
| `load_rdf_xml` | `(data text) → bigint` | 从 RDF/XML 字符串批量加载。 |

### SPARQL

| 函数 | 签名 | 注释 |
|---|---|---|
| `sparql` | `(query text) → setof jsonb` | SPARQL 1.1 SELECT。每行返回一个 JSONB 对象。 |
| `sparql_ask` | `(query text) → boolean` | SPARQL 1.1 ASK。 |
| `sparql_construct` | `(query text) → setof jsonb` | SPARQL 1.1 CONSTRUCT。 |
| `sparql_describe` | `(query text, strategy text) → setof jsonb` | SPARQL 1.1 DESCRIBE。 |
| `sparql_update` | `(update text) → bigint` | SPARQL 1.1 Update。返回受影响的三元组计数。 |
| `sparql_explain` | `(query text, analyze boolean) → text` | 查询计划/执行跟踪。 |

### 字典

| 函数 | 签名 | 注释 |
|---|---|---|
| `encode_term` | `(value text, kind smallint) → bigint` | 将术语编码为字典 ID。 |
| `decode_id` | `(id bigint) → text` | 将字典 ID 解码为其字符串形式。 |
| `encode_triple` | `(s text, p text, o text) → bigint` | 将引用三元组编码为字典 ID。 |
| `decode_triple` | `(id bigint) → jsonb` | 将引用三元组 ID 解码为 `{s, p, o}`。 |
| `dictionary_stats` | `() → jsonb` | 字典表的统计信息。 |
| `vacuum_dictionary` | `() → bigint` | 删除孤立的字典条目。 |

### SHACL 验证

| 函数 | 签名 | 注释 |
|---|---|---|
| `load_shacl` | `(shapes text) → bigint` | 解析和存储 SHACL 形状图。 |
| `validate` | `() → jsonb` | 运行离线验证。返回 `{conforms, violations}`。 |
| `list_shapes` | `() → setof record` | 枚举已存储的形状。 |
| `drop_shape` | `(shape_iri text) → bigint` | 删除形状。 |
| `process_validation_queue` | `() → bigint` | 处理异步验证队列。 |
| `dead_letter_queue` | `() → setof record` | 验证处理失败的项目。 |

### Datalog 推理

| 函数 | 签名 | 注释 |
|---|---|---|
| `load_rules` | `(rules text, rule_set text DEFAULT 'custom') → bigint` | 解析和存储 Datalog 规则集。 |
| `load_rules_builtin` | `(name text) → bigint` | 加载内置规则集（`'rdfs'` 或 `'owl-rl'`）。 |
| `add_rule` | `(rule_set text, rule_text text) → bigint` | 添加单个规则；返回新规则目录 ID。 |
| `remove_rule` | `(rule_id bigint) → bigint` | 按 ID 删除规则；返回撤回的三元组。 |
| `drop_rules` | `(rule_set text) → bigint` | 删除命名规则集中的所有规则。 |
| `infer` | `(rule_set text DEFAULT 'custom') → bigint` | 具体化推理；返回三元组计数。 |
| `infer_with_stats` | `(rule_set text DEFAULT 'custom') → jsonb` | 半朴素推理与统计信息。 |
| `infer_goal` | `(rule_set text, goal text) → jsonb` | 目标导向的 magic-sets 推理。 |
| `retract_inferred` | `(rule_set text) → bigint` | 删除规则集的所有具体化三元组。 |
| `list_rules` | `() → jsonb` | 列出所有已存储的规则及其元数据。 |
| `list_rule_sets` | `() → table` | 列出所有命名规则集及其规则计数。 |

### 管理

| 函数 | 签名 | 描述 |
|---|---|---|
| `vacuum` | `() → bigint` | 删除陈旧的墓碑和孤立的增量行。 |
| `reindex` | `() → bigint` | 重建 VP 表索引。 |
| `promote_rare_predicates` | `() → bigint` | 将谓词从 `vp_rare` 提升到专用 VP 表。 |
| `trigger_merge` | `() → void` | 通知后台合并工作线程立即运行。 |
| `get_statement` | `(sid bigint) → jsonb` | 通过其语句 ID 检索三元组。 |

### 命名图

| 函数 | 签名 | 描述 |
|---|---|---|
| `insert_triple_graph` | `(s text, p text, o text, g text) → bigint` | 插入到命名图中。 |
| `delete_triple_graph` | `(s text, p text, o text, g text) → bigint` | 从命名图中删除。 |
| `find_triples_graph` | `(s text, p text, o text, g text) → setof jsonb` | 查询命名图。 |

### 图 RLS

| 函数 | 签名 | 描述 |
|---|---|---|
| `enable_graph_rls` | `() → void` | 启用图级行级安全性。 |
| `grant_graph` | `(graph_iri text, role text) → void` | 为角色授予对图的 RLS SELECT 访问权限。 |
| `revoke_graph` | `(graph_iri text, role text) → void` | 撤销角色的 RLS 访问权限。 |
| `grant_graph_access` | `(graph_iri text, role text, privilege text DEFAULT 'SELECT') → void` | 以明确的权限级别授予。 |
| `revoke_graph_access` | `(graph_iri text, role text) → void` | 撤销访问权限（详细）。 |

### 全文搜索

| 函数 | 签名 | 描述 |
|---|---|---|
| `fts_index` | `(predicate text) → bigint` | 将谓词添加到 FTS 索引。 |
| `fts_search` | `(query text) → setof jsonb` | 在索引字面值上进行全文搜索。 |

### 导出和序列化

| 函数 | 签名 | 描述 |
|---|---|---|
| `export_ntriples` | `() → text` | 将所有三元组导出为 N-Triples。 |
| `export_turtle` | `() → text` | 导出为 Turtle。 |
| `export_jsonld` | `() → jsonb` | 导出为 JSON-LD。 |

### 计划缓存

| 函数 | 签名 | 描述 |
|---|---|---|
| `plan_cache_stats` | `() → jsonb` | SPARQL 计划缓存的命中/未命中统计信息。 |
| `plan_cache_reset` | `() → void` | 刷新计划缓存。 |

---

## 内部 schema（`_pg_ripple.*`）

`_pg_ripple` schema 中的表、序列和函数是**私有实现细节**。它们可以在任何次要版本上被重命名、重组或删除。

不要编写直接查询 `_pg_ripple.*` 表的应用程序代码。请改用 `pg_ripple.*` API。

内部对象的示例（非穷尽）：

- `_pg_ripple.dictionary` — 术语编码表
- `_pg_ripple.predicates` — VP 表目录
- `_pg_ripple.vp_rare` — 罕见谓词合并表
- `_pg_ripple.vp_{id}_main`、`_pg_ripple.vp_{id}_delta`、`_pg_ripple.vp_{id}_tombstones` — HTAP 分区表
- `_pg_ripple.statement_id_seq` — 全局 SID 序列
- `_pg_ripple.shacl_shapes`、`_pg_ripple.validation_queue`、`_pg_ripple.dead_letter_queue` — SHACL 内部

---

## GUC 稳定性

以下 GUC 是稳定公共 API 的一部分：

| GUC | 默认 | 类型 |
|---|---|---|
| `pg_ripple.dictionary_cache_size` | `65536` | integer |
| `pg_ripple.vp_promotion_threshold` | `1000` | integer |
| `pg_ripple.merge_threshold` | `10000` | integer |
| `pg_ripple.shacl_mode` | `'async'` | enum: `'async'`、`'sync'`、`'off'` |
| `pg_ripple.enforce_constraints` | `'warn'` | enum: `'error'`、`'warn'`、`'off'` |
| `pg_ripple.rls_bypass` | `off` | boolean |
| `pg_ripple.federation_timeout_ms` | `5000` | integer |
| `pg_ripple.enable_plan_cache` | `on` | boolean |

内部 GUC（以 `pg_ripple._` 开头的名称）可能会在没有通知的情况下更改。

---

## 升级兼容性

pg_ripple 为每个次要版本提供 SQL 迁移脚本（`sql/pg_ripple--X.Y.Z--X.Y.(Z+1).sql`）。升级始终可以通过以下方式完成：

```sql
ALTER EXTENSION pg_ripple UPDATE;
```

对于稳定的 API 更改，从不需要数据迁移。如果将来的版本修改了 `_pg_ripple.*` schema，则迁移脚本会自动处理它。