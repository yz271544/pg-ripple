# SQL 参考

本节记录 pg_ripple 导出的所有 SQL 函数和视图。

## 约定

**项格式** — 所有 IRI、空节点和字面量参数使用 N-Triples 表示法：

| 类型 | 格式 | 示例 |
|---|---|---|
| IRI | `<…>` | `<https://example.org/alice>` |
| 空节点 | `_:…` | `_:b0` |
| 纯字面量 | `"…"` | `"Alice"` |
| 带语言标签的字面量 | `"…"@lang` | `"Alice"@en` |
| 类型化字面量 | `"…"^^<type>` | `"30"^^<http://www.w3.org/2001/XMLSchema#integer>` |

**NULL 通配符** — 在 find/delete 函数中，`NULL` 匹配该位置的任意值。

**Schema** — 所有公共函数位于 `pg_ripple` schema。内部表位于 `_pg_ripple`。

## 函数索引

| 函数 | 节 |
|---|---|
| `insert_triple` | [三元组 CRUD](triple-crud.md) |
| `delete_triple` | [三元组 CRUD](triple-crud.md) |
| `find_triples` | [三元组 CRUD](triple-crud.md) |
| `triple_count` | [三元组 CRUD](triple-crud.md) |
| `load_ntriples` | [批量加载](bulk-load.md) |
| `load_ntriples_file` | [批量加载](bulk-load.md) |
| `load_turtle` | [批量加载](bulk-load.md) |
| `load_turtle_file` | [批量加载](bulk-load.md) |
| `load_nquads` | [批量加载](bulk-load.md) |
| `load_nquads_file` | [批量加载](bulk-load.md) |
| `load_trig` | [批量加载](bulk-load.md) |
| `load_trig_file` | [批量加载](bulk-load.md) |
| `load_rdfxml` | [批量加载](bulk-load.md) |
| `load_rdfxml_file` | [批量加载](bulk-load.md) |
| `load_ntriples_into_graph` | [批量加载](bulk-load.md) |
| `load_turtle_into_graph` | [批量加载](bulk-load.md) |
| `load_rdfxml_into_graph` | [批量加载](bulk-load.md) |
| `load_ntriples_file_into_graph` | [批量加载](bulk-load.md) |
| `load_turtle_file_into_graph` | [批量加载](bulk-load.md) |
| `load_rdfxml_file_into_graph` | [批量加载](bulk-load.md) |
| `create_graph` | [命名图](named-graphs.md) |
| `drop_graph` | [命名图](named-graphs.md) |
| `list_graphs` | [命名图](named-graphs.md) |
| `find_triples_in_graph` | [命名图](named-graphs.md) |
| `triple_count_in_graph` | [命名图](named-graphs.md) |
| `delete_triple_from_graph` | [命名图](named-graphs.md) |
| `clear_graph` | [命名图](named-graphs.md) |
| `sparql` | [SPARQL 查询](sparql-query.md) |
| `sparql_ask` | [SPARQL 查询](sparql-query.md) |
| `sparql_explain` | [SPARQL 查询](sparql-query.md) |
| `sparql_construct` | [SPARQL 查询](sparql-query.md) |
| `sparql_describe` | [SPARQL 查询](sparql-query.md) |
| `sparql_update` | [SPARQL 更新](sparql-update.md) |
| `fts_index` | [全文搜索](fts.md) |
| `fts_search` | [全文搜索](fts.md) |
| `encode_triple` | [RDF-star](rdf-star.md) |
| `decode_triple` | [RDF-star](rdf-star.md) |
| `get_statement` | [RDF-star](rdf-star.md) |
| `encode_term` | [字典](dictionary.md) |
| `decode_id` | [字典](dictionary.md) |
| `decode_id_full` | [字典](dictionary.md) |
| `lookup_iri` | [字典](dictionary.md) |
| `register_prefix` | [前缀注册表](prefix.md) |
| `prefixes` | [前缀注册表](prefix.md) |
| `export_ntriples` | [序列化与导出](serialization.md) |
| `export_nquads` | [序列化与导出](serialization.md) |
| `export_turtle` | [序列化与导出](serialization.md) |
| `export_turtle_stream` | [序列化与导出](serialization.md) |
| `export_jsonld` | [序列化与导出](serialization.md) |
| `export_jsonld_stream` | [序列化与导出](serialization.md) |
| `sparql_construct_turtle` | [序列化与导出](serialization.md) |
| `sparql_construct_jsonld` | [序列化与导出](serialization.md) |
| `sparql_describe_turtle` | [序列化与导出](serialization.md) |
| `sparql_describe_jsonld` | [序列化与导出](serialization.md) |
| `load_shacl` | [SHACL 验证](shacl.md) |
| `validate` | [SHACL 验证](shacl.md) |
| `list_shapes` | [SHACL 验证](shacl.md) |
| `drop_shape` | [SHACL 验证](shacl.md) |
| `process_validation_queue` | [SHACL 验证](shacl.md) |
| `validation_queue_length` | [SHACL 验证](shacl.md) |
| `dead_letter_count` | [SHACL 验证](shacl.md) |
| `dead_letter_queue` | [SHACL 验证](shacl.md) |
| `drain_dead_letter_queue` | [SHACL 验证](shacl.md) |
| `enable_shacl_monitors` | [SHACL 验证](shacl.md) |
| `load_rules` | [Datalog 推理](datalog.md) |
| `load_rules_builtin` | [Datalog 推理](datalog.md) |
| `infer` | [Datalog 推理](datalog.md) |
| `check_constraints` | [Datalog 推理](datalog.md) |
| `list_rules` | [Datalog 推理](datalog.md) |
| `drop_rules` | [Datalog 推理](datalog.md) |
| `enable_rule_set` | [Datalog 推理](datalog.md) |
| `disable_rule_set` | [Datalog 推理](datalog.md) |
| `prewarm_dictionary_hot` | [Datalog 推理](datalog.md) |
| `create_sparql_view` | [物化视图](views.md) |
| `drop_sparql_view` | [物化视图](views.md) |
| `list_sparql_views` | [物化视图](views.md) |
| `create_datalog_view` | [物化视图](views.md) |
| `create_datalog_view_from_rule_set` | [物化视图](views.md) |
| `drop_datalog_view` | [物化视图](views.md) |
| `list_datalog_views` | [物化视图](views.md) |
| `create_extvp` | [物化视图](views.md) |
| `drop_extvp` | [物化视图](views.md) |
| `list_extvp` | [物化视图](views.md) |
| `pg_trickle_available` | [物化视图](views.md) |
| `compact` | [管理](admin.md) |
| `stats` | [管理](admin.md) |
| `subscribe` | [管理](admin.md) |
| `unsubscribe` | [管理](admin.md) |
| `htap_migrate_predicate` | [管理](admin.md) |
| `subject_predicates` | [管理](admin.md) |
| `object_predicates` | [管理](admin.md) |
| `deduplicate_predicate` | [管理](admin.md) |
| `deduplicate_all` | [管理](admin.md) |
| `vacuum` | [管理](admin.md) |
| `reindex` | [管理](admin.md) |
| `vacuum_dictionary` | [管理](admin.md) |
| `dictionary_stats` | [管理](admin.md) |
| `enable_graph_rls` | [管理](admin.md) |
| `grant_graph` | [管理](admin.md) |
| `revoke_graph` | [管理](admin.md) |
| `list_graph_access` | [管理](admin.md) |
| `schema_summary` | [管理](admin.md) |
| `enable_schema_summary` | [管理](admin.md) |
| `plan_cache_stats` | [管理](admin.md) |
| `plan_cache_reset` | [管理](admin.md) |