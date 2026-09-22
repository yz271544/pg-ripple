# pg_ripple GUC 参考

有关完整参考，请参阅 [参考 GUC 文档](src/reference/guc-reference.md)。

`pg_ripple.llm_api_key_env` 仅接受匹配 `^[A-Z_][A-Z0-9_]*$` 的环境变量名称。原始值被拒绝。在 0.131 之前的迁移期间，超级用户可以临时设置 `pg_ripple.llm_api_key_env_allow_raw = on`。

> **版本**：v0.116.0
> 所有参数都以 `pg_ripple.` 为前缀，可通过 `SET`、`postgresql.conf` 或 `ALTER DATABASE/ROLE SET` 配置。

## 类别

- [存储与 VP 表](#存储与-vp-表)
- [字典与缓存](#字典与缓存)
- [SPARQL 引擎](#sparql-引擎)
- [Datalog 与推理](#datalog-与推理)
- [证明树与规则解释](#证明树与规则解释)  *(v0.116.0)*
- [SHACL 验证](#shacl-验证)
- [owl:sameAs 与实体解析](#owlsameas-与实体解析)
- [ER 监控](#er-监控)  *(v0.116.0)*
- [联邦](#联邦)
- [CDC 与复制](#cdc-与复制)
- [双向中继](#双向中继)  *(v0.116.0)*
- [概率与贝叶斯推理](#概率与贝叶斯推理)  *(v0.116.0)*
- [LLM / 嵌入与 RAG](#llm--嵌入与-rag)
- [PageRank 与中心性](#pagerank-与中心性)
- [可观测性与追踪](#可观测性与追踪)
- [安全与访问控制](#安全与访问控制)
- [HTTP 配套服务（Arrow Flight）](#http-配套服务arrow-flight)
- [Citus 集成](#citus-集成)
- [后台工作线程](#后台工作线程)

---

## 存储与 VP 表

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `vp_promotion_threshold` | integer | 1000 | Suset | 罕见谓词提升到自己的 VP 表之前的最小三元组数 |
| `vp_promotion_batch_size` | integer | 10000 | Suset | 批量 VP 提升的批量大小 |
| `columnar_threshold` | integer | -1 | Suset | 主分区使用列式存储的行数阈值（-1 = 禁用） |
| `adaptive_indexing_enabled` | bool | off | Suset | 允许合并工作线程动态创建辅助索引 |
| `dedup_on_merge` | bool | off | Suset | 在 HTAP 合并期间去重三元组 |
| `tombstone_gc_enabled` | bool | on | Suset | 启用后台 GC 墓碑行 |
| `tombstone_gc_threshold` | integer | 10000 | Suset | 触发 GC 周期的墓碑数 |
| `tombstone_retention_seconds` | integer | 0 | Suset | GC 资格前保留墓碑的秒数（0 = 立即） |
| `delta_index_threshold` | integer | 5000 | Suset | 触发自动索引创建的 delta 分区行数 |
| `dict_vacuum_threshold` | integer | 10000 | Suset | 自动字典 `ANALYZE` 之前的新词项阈值 |
| `vacuum_dict_batch_size` | integer | 200 | Suset | 增量字典 vacuum 的批量大小 |
| `auto_analyze` | bool | on | Suset | 在合并后启用对 VP 表的自动 ANALYZE |
| `stats_refresh_interval_seconds` | integer | 300 | Suset | 后台统计刷新周期之间的秒数 |
| `stats_scan_limit` | integer | 1000 | Suset | 统计刷新期间每个 VP 表采样的最大行数 |
| `bulk_load_use_copy` | bool | on | Suset | 对批量 RDF 加载使用 COPY 而不是 INSERT |
| `export_batch_size` | integer | 10000 | Userset | 序列化 RDF 导出时每批的行数 |
| `export_max_rows` | integer | 1000000 | Userset | 导出函数返回的行数硬上限 |
| `export_confidence` | real | 0.0 | Userset | 导出中包含的三元组的最小置信度分数 |

---

## 字典与缓存

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `dictionary_cache_size` | integer | 65536 | Postmaster | IRI/literal → i64 缓存的每进程 LRU 容量 |
| `dictionary_tier_threshold` | integer | 1000000 | Suset | 字典切换到分层存储的条目数阈值 |
| `strict_dictionary` | bool | off | Userset | 拒绝插入未通过 RFC 3987 验证的 IRI |
| `predicate_cache_enabled` | bool | on | Suset | 在共享内存中缓存谓词 → VP 表 OID 查找 |
| `cache_budget` | integer | 256 | Suset | 为内存中谓词统计缓存保留的共享内存页 |
| `plan_cache_capacity` | integer | 512 | Suset | 计划缓存中保存的已编译 SPARQL 计划的最大数量 |
| `plan_cache_size` | integer | 8192 | Suset | SPARQL 计划缓存的最大字节大小（KB） |
| `rule_plan_cache` | bool | on | Suset | 启用 Datalog 规则 SQL 的计划级缓存 |
| `rule_plan_cache_size` | integer | 256 | Suset | 缓存的 Datalog 规则 SQL 计划的最大数量 |

---

## SPARQL 引擎

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `sparql_max_rows` | integer | 10000 | Userset | 任何 SPARQL SELECT 返回的最大结果行数 |
| `sparql_max_algebra_depth` | integer | 64 | Userset | 查询被拒绝之前的 SPARQL 代数树的最大深度 |
| `sparql_max_triple_patterns` | integer | 256 | Userset | 单个 BGP 中的最大三元组模式数 |
| `sparql_overflow_action` | string | `'error'` | Userset | 行限制溢出时的操作：`'error'`、`'truncate'` 或 `'warn'` |
| `sparql_strict` | bool | off | Userset | 拒绝引用未定义前缀或未知谓词的查询 |
| `strict_sparql_filters` | bool | off | Userset | 在类型错误的 FILTER 参数上引发错误（而不是返回 NULL） |
| `bgp_reorder` | bool | on | Userset | 启用基本图模式连接顺序优化 |
| `star_join_collapse` | bool | on | Userset | 将星形模式（相同主语）折叠到单个 VP 扫描中 |
| `topn_pushdown` | bool | on | Userset | 将 `LIMIT` 下推到 VP 扫描以用于 ORDER BY 查询 |
| `parallel_query_min_joins` | integer | 4 | Userset | 考虑并行扫描之前 BGP 中的最小连接数 |
| `all_nodes_predicate_limit` | integer | 100 | Userset | `ALL_PREDICATES()` 表达式中扫描的最大谓词数 |
| `approx_distinct` | bool | off | Userset | 对 `COUNT(DISTINCT ...)` 使用 HyperLogLog 近似 |
| `describe_strategy` | string | `'cbd'` | Userset | DESCRIBE 形式：`'cbd'`（简洁有界描述）或 `'symcbd'` |
| `describe_form` | string | `'turtle'` | Userset | DESCRIBE 响应的序列化格式 |
| `describe_max_depth` | integer | 16 | Userset | 对称 CBD DESCRIBE 的最大跳数深度 |
| `default_graph` | string | `''` | Userset | 未指定 GRAPH 子句时使用的命名图 IRI（空 = 默认图） |
| `use_graph_context` | bool | on | Userset | 在 SPARQL 结果绑定中包含图列 |

---

## Datalog 与推理

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `datalog_max_depth` | integer | 64 | Userset | Datalog 不动点评估的最大递归深度 |
| `datalog_max_derived` | integer | 1000000 | Userset | 推理运行中止之前最大派生三元组数 |
| `datalog_parallel_workers` | integer | 4 | Suset | 用于层评估的并行工作线程数 |
| `datalog_parallel_threshold` | integer | 10000 | Suset | 触发并行层评估的最小三元组数 |
| `datalog_sequence_batch` | integer | 1000 | Suset | 顺序 Datalog 层评估的批量大小 |
| `datalog_antijoin_threshold` | integer | 100 | Userset | 反连接使用哈希策略时估计的右侧行数下限 |
| `datalog_cost_reorder` | bool | on | Userset | 为 Datalog 规则体启用基于成本的连接重新排序 |
| `datalog_cost_bound_s_divisor` | real | 10.0 | Userset | 应用于主语绑定基数估计的除数 |
| `datalog_cost_bound_so_divisor` | real | 100.0 | Userset | 应用于主语+对象绑定基数估计的除数 |
| `datalog_citus_dispatch` | bool | off | Suset | 将 Datalog SQL 分发到所有 Citus 工作线程分片 |
| `wcoj_enabled` | bool | on | Userset | 启用最坏情况最优连接（leapfrog triejoin） |
| `wcoj_min_tables` | integer | 3 | Userset | 考虑 WCOJ 之前的最小连接表数 |
| `wcoj_min_cardinality` | integer | 1000 | Userset | 考虑 WCOJ 之前的最小估计基数 |
| `tabling` | bool | on | Userset | 为递归 Datalog 启用表查询（记忆化） |
| `tabling_ttl` | integer | 3600 | Userset | 表格结果被认为过期的秒数 |
| `dred_enabled` | bool | on | Suset | 启用增量回溯的 Delete–Rederive (DReD) |
| `dred_batch_size` | integer | 1000 | Suset | DReD 回溯周期的批量大小 |
| `wfs_max_iterations` | integer | 100 | Userset | 良基语义计算的最大迭代次数 |
| `demand_transform` | bool | on | Userset | 对递归规则应用 magic-sets 需求转换 |
| `rule_graph_scope` | string | `'default'` | Userset | 规则在哪些图中触发：`'default'` 或 `'all'` |
| `rule_conflict_check_on_load` | bool | on | Suset | 在加载规则时检查冲突的规则头部 |
| `record_derivations` | bool | off | Userset | 将派生出处写入 `_pg_ripple.derivations` |
| `strict_goal_validation` | bool | off | Userset | 拒绝引用未注册谓词的 Datalog 目标原子 |
| `suggest_rules_max_candidates` | integer | 20 | Userset | `suggest_rules()` 返回的最大候选规则数 |

---

## 证明树与规则解释

> **v0.116.0 中新增**（M16-07、M16-19）

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `proof_tree_max_depth` | integer | 64 | Userset | 由 `justify()`/`build_proof_tree()` 构建的证明树的最大深度。超过此限制会发出 `PT0480` 警告并截断树 |
| `proof_tree_max_nodes` | integer | 10000 | Userset | 证明树所有分支上的最大总节点数。超过此限制会发出 `PT0481` 警告并停止扩展 |
| `rule_explanation_cache_max_entries` | integer | 1000 | Userset | `explain_rule()` 结果的每进程 LRU 缓存容量。设置为 0 可禁用 |

---

## SHACL 验证

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `shacl_mode` | string | `'warn'` | Userset | SHACL 验证模式：`'warn'`、`'error'` 或 `'off'` |
| `shacl_rule_cwb` | bool | on | Suset | 对 SHACL 规则评估应用封闭世界行为 |
| `shacl_rule_max_iterations` | integer | 100 | Suset | SHACL 规则不动点迭代的最大次数 |
| `shacl_score_log_retention_days` | integer | 30 | Suset | 保留 SHACL 验证分数历史记录的天数 |
| `enforce_constraints` | bool | on | Userset | 在 INSERT 时为 SHACL 约束违规引发错误 |

---

## owl:sameAs 与实体解析

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `sameas_reasoning` | bool | on | Userset | 启用 owl:sameAs 规范化和聚类合并 |
| `sameas_max_cluster_size` | integer | 10000 | Suset | 在 owl:sameAs 合并被阻止之前的最大聚类大小 |
| `sameas_apply_rate_limit` | integer | 100 | Suset | 每秒最大 owl:sameAs 合并数 |
| `record_sameas_anomalies` | bool | on | Suset | 将异常 owl:sameAs 对记录到 `_pg_ripple.sameas_anomalies` |
| `sameas_anomaly_log_retention` | integer | 30 | Suset | 保留 owl:sameAs 异常日志条目的天数 |
| `default_fuzzy_threshold` | real | 0.85 | Userset | `fuzzy_match()` 的默认相似度阈值 |
| `string_similarity_extensions_ok` | bool | on | Userset | 允许字符串相似度 UDF 调用 `pg_trgm` / `fuzzystrmatch` |
| `bloom_max_input_length` | integer | 4096 | Userset | 传递给 Bloom 过滤器去重的字符串的最大字节长度 |

---

## ER 监控

> **v0.116.0 中新增**（M16-01）

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `er_monitoring_retention_days` | integer | 30 | Suset | 后台工作线程自动修剪之前，保留 ER 监控流表行的天数。范围：1–3650 |

---

## 联邦

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `federation_allowed_endpoints` | string | `''` | Suset | 服务端点 URL 前缀的逗号分隔允许列表 |
| `federation_allow_private` | bool | off | Suset | 允许对私有范围 IP 地址进行联邦 |
| `federation_adaptive_timeout` | bool | on | Userset | 根据观察到的延迟动态调整每端点超时 |
| `allow_unregistered_service_endpoints` | bool | off | Suset | 允许对未在 `federation_endpoints` 中列出的端点进行 SERVICE 调用 |
| `vector_federation_timeout_ms` | integer | 5000 | Userset | 远程向量搜索联邦调用的超时（毫秒） |

---

## CDC 与复制

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `cdc_bridge_enabled` | bool | off | Suset | 为下游消费者启用 CDC outbox 桥 |
| `cdc_bridge_batch_size` | integer | 100 | Suset | 在 pg_tide outbox 刷新之前批处理的 CDC 通知 |
| `cdc_bridge_flush_ms` | integer | 200 | Suset | CDC outbox 刷新之间的最大毫秒数 |
| `cdc_bridge_outbox_table` | string | `''` | Suset | CDC outbox 表的完全限定名 |
| `cdc_slot_idle_timeout_seconds` | integer | 3600 | Suset | 在孤立的 CDC slot 被丢弃之前的空闲超时（秒） |
| `cdc_watermark_batch_size` | integer | 100 | Suset | 每个周期处理的 CDC 水印行数 |
| `cdc_watermark_flush_interval_ms` | integer | 50 | Suset | 水印刷新间隔（毫秒） |
| `temporal_cdc_enabled` | bool | on | Suset | 为时序（有效时间）三元组版本启用 CDC 支持 |
| `temporal_data_model` | string | `'point'` | Suset | 时序数据模型：`'point'` 或 `'interval'` |
| `enable_temporal_operators` | bool | off | Userset | 启用时序 SPARQL 扩展（`VALID_TIME`、`AS OF`） |
| `replication_enabled` | bool | off | Suset | 在 pg_ripple 实例之间启用内置逻辑复制 |
| `replication_batch_size` | integer | 100 | Suset | 每个逻辑复制批次的三元组数 |
| `replication_batch_interval_ms` | integer | 500 | Suset | 复制批次刷新之间的间隔（毫秒） |
| `replication_conflict_strategy` | string | `'last_writer_wins'` | Suset | 逻辑应用冲突策略；当前工作线程使用 `'last_writer_wins'` |
| `read_replica_dsn` | string | `''` | Suset | 用于将只读查询定向到副本的连接字符串 |
| `trickle_integration` | bool | on | Userset | 旧版中继集成开关；设置为 `off` 时禁用 pg_tide CDC 桥 |

---

## 双向中继

> **v0.116.0 中新增**（M16-11）

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `bidi_relay_max_inflight` | integer | 1000 | Suset | 触发溢出处理之前双向中继通道中的最大消息数 |
| `bidi_relay_drop_policy` | string | `NULL` | Userset | 双向中继通道的溢出丢弃策略。`NULL` = 阻塞（默认），`'drop-oldest'` = 静默丢弃最旧的消息，`'drop-newest'` = 丢弃传入消息 |

---

## 概率与贝叶斯推理

> **`bayesian_propagation_max_depth` v0.116.0 中新增**（M16-20）

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `probabilistic_datalog` | bool | off | Userset | 启用概率 Datalog 评估（noisy-OR 模型） |
| `confidence_propagation_max_depth` | integer | 10 | Userset | 通过 `update_confidence()` 进行置信度传播的最大深度 |
| `bayesian_propagation_max_depth` | integer | 10 | Userset | `propagate_downstream()` 中贝叶斯下游传播的最大深度。仅替换贝叶斯路径的硬编码默认值 |
| `confidence_batch_size` | integer | 500 | Suset | 异步置信度管道中每批处理的三元组数 |
| `confidence_reprocessing_interval` | integer | 3600 | Suset | 计划置信度重新计算周期之间的秒数 |
| `confidence_update_strategy` | string | `'lazy'` | Suset | 置信度更新策略：`'lazy'` 或 `'eager'` |
| `conflict_confidence_penalty` | real | 0.1 | Userset | 应用于冲突三元组的置信度乘数 |
| `cwb_confidence_propagation` | bool | on | Suset | 在置信度传播期间应用 CWB（封闭世界偏差） |
| `prob_datalog_max_iterations` | integer | 100 | Userset | 概率 Datalog 的最大不动点迭代次数 |
| `prob_datalog_convergence_delta` | real | 0.001 | Userset | 概率 Datalog 不动点的收敛阈值 |
| `prob_datalog_cyclic` | bool | on | Userset | 允许概率 Datalog 程序中的循环依赖 |
| `prob_datalog_cyclic_strict` | bool | off | Userset | 在检测到循环概率规则时引发错误而不是警告 |
| `prov_enabled` | bool | off | Userset | 为所有三元组插入启用出处跟踪（PROV-O） |
| `prov_confidence` | real | 1.0 | Userset | 分配给没有显式置信度的三元组的默认置信度 |
| `evidence_log_retention` | integer | 30 | Suset | 保留出处证据日志条目的天数 |

---

## LLM / 嵌入与 RAG

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `embedding_model` | string | `''` | Userset | 嵌入模型标识符（例如 `'text-embedding-3-small'`） |
| `embedding_api_url` | string | `''` | Suset | 嵌入 API 端点的基础 URL |
| `embedding_api_key` | string | `''` | Suset | 嵌入服务的 API 密钥（存储在共享内存中，不在日志中） |
| `embedding_dimensions` | integer | 1536 | Userset | 嵌入向量维度 |
| `embedding_batch_size` | integer | 64 | Userset | 每个嵌入 API 请求发送的字面量数 |
| `embedding_index_type` | string | `'ivfflat'` | Suset | pgvector 索引类型：`'ivfflat'` 或 `'hnsw'` |
| `embedding_precision` | string | `'float4'` | Suset | 嵌入向量的存储精度：`'float4'` 或 `'float2'` |
| `pgvector_enabled` | bool | on | Suset | 启用 pgvector 集成以进行混合 SPARQL + 向量搜索 |
| `auto_embed` | bool | off | Userset | 在插入新字面量时自动嵌入它们 |

---

## PageRank 与中心性

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `pagerank_rules` | string | `'default'` | Userset | 用于 PageRank 权重计算的命名规则集 |
| `pagerank_wcoj_threshold` | integer | 10000 | Userset | 在 PageRank 中使用 WCOJ 之前的最小边数 |
| `pagerank_selective_threshold` | real | 0.01 | Userset | 选择性 PageRank 的最小个性化权重 |
| `pagerank_temp_threshold` | integer | 50000 | Suset | PageRank 使用临时表的行数阈值 |
| `pagerank_shacl_threshold` | real | 0.5 | Userset | 用于 SHACL 约束节点过滤的最小 PageRank 分数 |
| `pagerank_trickle_confidence_attenuation` | bool | on | Userset | 通过边置信度衰减增量 K 跳 PageRank delta |
| `pagerank_sketch_width` | integer | 2048 | Suset | 用于近似 PageRank 的 Count-Min sketch 的宽度 |
| `pagerank_sketch_depth` | integer | 5 | Suset | 用于近似 PageRank 的 Count-Min sketch 的深度 |

---

## 可观测性与追踪

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `tracing_enabled` | bool | off | Suset | 启用 OpenTelemetry 分布式追踪 |
| `tracing_exporter` | string | `'otlp'` | Suset | 追踪导出器后端：`'otlp'` 或 `'stdout'` |
| `tracing_otlp_endpoint` | string | `''` | Suset | OTLP 导出器端点 URL |
| `tracing_traceparent` | string | `''` | Userset | 当前会话的 W3C `traceparent` 头值 |
| `audit_log_enabled` | bool | off | Suset | 启用所有 SPARQL/Datalog 操作的审计日志记录 |
| `audit_retention` | integer | 90 | Suset | 保留审计日志条目的天数 |
| `explanation_cache_ttl` | integer | 3600 | Userset | 缓存的 `explain_rule()` 数据库条目被认为过期的秒数 |
| `rule_explanation_cache_ttl` | integer | 3600 | Userset | `explanation_cache_ttl` 的别名（规则解释的数据库端 TTL） |

---

## 安全与访问控制

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `rls_bypass` | bool | off | Userset | 允许超级用户绕过 VP 表上的 RLS 策略 |
| `copy_rdf_allowed_paths` | string | `''` | Suset | 允许用于基于 `COPY` 的 RDF 加载的逗号分隔路径前缀 |
| `block_on_conflict` | bool | off | Userset | 阻止与现有三元组冲突的插入（而不是 ON CONFLICT DO NOTHING） |

---

## HTTP 配套服务（Arrow Flight）

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `arrow_batch_size` | integer | 1000 | Userset | 批量导出中每个 Arrow record batch 的行数 |
| `arrow_flight_secret` | string | `''` | Suset | 用于签署 Arrow Flight 会话票证的 HMAC 密钥 |
| `arrow_flight_expiry_secs` | integer | 3600 | Suset | Arrow Flight 会话票证的过期时间（秒） |
| `arrow_unsigned_tickets_allowed` | bool | off | Suset | 接受没有 HMAC 签名的 Arrow Flight 票证（仅用于测试） |

---

## Citus 集成

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `citus_sharding_enabled` | bool | off | Suset | 启用 VP 表的 Citus 分布式分片 |
| `citus_service_pruning` | bool | off | Suset | 启用 SERVICE 查询的谓词级分片剪枝 |
| `citus_prune_carry_max` | integer | 1000 | Userset | 剪枝期间跨连接传递的最大分片绑定数 |
| `citus_trickle_compat` | bool | off | Userset | 使用 `colocate_with => 'none'` 的 Citus CDC/IVM 集成的旧版兼容模式 |

---

## 后台工作线程

| GUC | 类型 | 默认 | 上下文 | 描述 |
|-----|------|---------|---------|-------------|
| `worker_database` | string | `''` | Postmaster | HTAP 合并和管理后台工作线程的目标数据库名称 |

---

## v0.116.0 新增 GUC 摘要

以下 GUC 在 v0.116.0（A16 里程碑）中添加：

| GUC | 类别 | 工单 |
|-----|----------|--------|
| `er_monitoring_retention_days` | ER 监控 | M16-01 |
| `proof_tree_max_depth` | 证明树 | M16-07 |
| `proof_tree_max_nodes` | 证明树 | M16-07 |
| `rule_explanation_cache_max_entries` | 证明树 / 规则解释 | M16-19 |
| `bayesian_propagation_max_depth` | 概率推理 | M16-20 |
| `bidi_relay_drop_policy` | 双向中继 | M16-11 |

---

## 顾问生命周期策略

与 GUC 相关的安全顾问在 `audit.toml` 中跟踪，按季度审查计划进行：

1. **打开顾问** → 一个冲刺内分流。
2. **忽略带过期** → 添加 `[ignore]` 条目，`expires` 最多 12 个月。
3. **季度审查** → 当过期临近时，重新评估威胁模型。
4. **过期强制重新决策** → 过期条目被视为新发现。

有关完整顾问忽略列表，请参阅 [`audit.toml`](../audit.toml)。