# 限制和配额

此页面记录 pg_ripple 中的硬限制、默认配额和可调整上限。此处列出的所有 GUC 参数均可调整；有关完整详细信息，请参阅 [GUC 参考](guc-reference.md)。

---

## 查询限制

| 限制 | 默认值 | GUC | 错误代码 |
|---|---|---|---|
| SPARQL 最大结果行数 | 无限制 | `pg_ripple.sparql_max_rows` | PT640 |
| SPARQL 溢出操作 | `truncate` | `pg_ripple.sparql_overflow_action` | — |
| 最大代数树深度 | 256 | `pg_ripple.sparql_max_algebra_depth` | PT440 |
| 每个查询的最大三元组模式数 | 4096 | `pg_ripple.sparql_max_triple_patterns` | PT440 |
| 最大 DESCRIBE CBD 深度 | 16 | `pg_ripple.describe_max_depth` | — |
| 模糊匹配输入长度 | 4096 字符 | `pg_ripple.fuzzy_max_input_length` | PT0308 |
| all-nodes 谓词展开上限 | 500 个谓词 | `pg_ripple.all_nodes_predicate_limit` | — |

## 导出限制

| 限制 | 默认值 | GUC | 错误代码 |
|---|---|---|---|
| 导出函数最大行数（Turtle/N-Triples/JSON-LD） | 无限制 | `pg_ripple.export_max_rows` | PT642 |
| Arrow Flight 批大小 | 1000 行/批 | `pg_ripple.arrow_batch_size` | — |

## 推导限制

| 限制 | 默认值 | GUC | 错误代码 |
|---|---|---|---|
| Datalog 最大深度 | 无限制 | `pg_ripple.datalog_max_depth` | — |
| Datalog 最大派生事实数 | 无限制 | `pg_ripple.datalog_max_derived` | — |
| 良好基础语义最大轮数 | 100 | `pg_ripple.wfs_max_iterations` | PT520 |
| SHACL 规则最大迭代次数 | 100 | `pg_ripple.shacl_rule_max_iterations` | PT301 |
| Lattice 推导最大迭代次数 | 1000 | `pg_ripple.lattice_max_iterations` | PT540 |
| 概率 Datalog 最大迭代次数 | 100 | `pg_ripple.prob_datalog_max_iterations` | — |
| SHACL 分数日志保留 | 30 天 | `pg_ripple.shacl_score_log_retention_days` | — |

## 联邦限制

| 限制 | 默认值 | GUC | 错误代码 |
|---|---|---|---|
| 每个 SERVICE 超时 | 30 秒 | `pg_ripple.federation_timeout` | PT214 |
| 连接超时 | 10 秒 | `pg_ripple.federation_connect_timeout_secs` | PT214 |
| 每个 SERVICE 调用的最大行数 | 10,000 | `pg_ripple.federation_max_results` | — |
| 每个端点的最大响应主体 | 100 MiB | `pg_ripple.federation_max_response_bytes` | PT215 |
| 部分恢复最大字节 | 64 KiB | `pg_ripple.federation_partial_recovery_max_bytes` | — |
| 熔断器阈值 | 5 次失败 | `pg_ripple.federation_circuit_breaker_threshold` | PT217 |
| 并行 SERVICE 工作线程数 | 4 | `pg_ripple.federation_parallel_max` | — |

## PageRank 限制

| 限制 | 默认值 | GUC | 错误代码 |
|---|---|---|---|
| 最大 PageRank 迭代次数 | 100 | `pg_ripple.pagerank_max_iterations` | — |
| 每次调用的最大种子 IRI 数 | 1024 | `pg_ripple.pagerank_max_seeds` | PT0411 |
| 收敛检查范数 | L1 | `pg_ripple.pagerank_convergence_norm` | — |

## 存储限制

| 限制 | 默认值 | GUC | 备注 |
|---|---|---|---|
| VP 提升阈值 | 1,000 个三元组 | `pg_ripple.vp_promotion_threshold` | 低于阈值：存储在 `vp_rare` |
| 合并批大小 | 1,000,000 行 | `pg_ripple.merge_batch_size` | 每合并 INSERT…SELECT |
| 合并围栏锁超时 | 5,000 毫秒 | `pg_ripple.merge_lock_timeout_ms` | — |
| CDC 水位批大小 | 100 个事件 | `pg_ripple.cdc_watermark_batch_size` | — |
| VP 提升批大小 | 10,000 行 | `pg_ripple.vp_promotion_batch_size` | — |
| Bidi 中继最大在飞操作数 | 1,000 ops | `pg_ripple.bidi_relay_max_inflight` | Drop-oldest 策略 |
| 字典分析阈值 | 10,000 词项 | `pg_ripple.dict_vacuum_threshold` | 编码后 `ANALYZE`；保留 GUC 名称以保持兼容性 |

## 字典和编码

| 限制 | 默认值 | GUC | 备注 |
|---|---|---|---|
| 字典 LRU 缓存大小 | 65,536 条 | `pg_ripple.dictionary_cache_size` | XXH3-128 哈希映射 |
| 共享内存预算 | 64 MiB | `pg_ripple.cache_budget_mb` | `postmaster` 作用域 |

## HTTP API 限制

| 限制 | 默认值 | 配置 | 备注 |
|---|---|---|---|
| 最大请求主体大小 | 10 MiB | `PG_RIPPLE_HTTP_MAX_BODY_BYTES` | 适用于 SPARQL UPDATE 主体 |
| 速率限制 | 无限制 | `PG_RIPPLE_HTTP_RATE_LIMIT` | 按源 IP，req/s |
| Arrow Flight 票据到期 | 3,600 秒 | `pg_ripple.arrow_flight_expiry_secs` | 签名 HMAC 票据 |
| 连接池大小 | 16 | `PG_RIPPLE_HTTP_POOL_SIZE` | Postgres 连接 |
| 流式查询超时 | 300,000 毫秒 | `PG_RIPPLE_HTTP_QUERY_TIMEOUT_MS` | 上限为 `PG_RIPPLE_HTTP_QUERY_TIMEOUT_MAX_MS` |
| 流式空闲超时 | 60,000 毫秒 | `PG_RIPPLE_HTTP_STREAM_IDLE_TIMEOUT_MS` | 行之间的最大等待时间 |
| 流式块大小 | 65,536 字节 | `PG_RIPPLE_HTTP_STREAM_CHUNK_BYTES` | 有界响应合并 |
| 流式行大小 | 1 MiB | `PG_RIPPLE_HTTP_STREAM_MAX_ROW_BYTES` | 编码行上限 |

## 审计和保留

| 限制 | 默认值 | GUC | 备注 |
|---|---|---|---|
| 事件审计保留 | 90 天 | `pg_ripple.audit_retention_days` | `_pg_ripple.event_audit` |
| SHACL 分数日志保留 | 30 天 | `pg_ripple.shacl_score_log_retention_days` | `_pg_ripple.shacl_score_log` |
| CDC 槽空闲超时 | 3,600 秒 | `pg_ripple.cdc_slot_idle_timeout_seconds` | 孤立槽清理 |
| VACUUM 字典批大小 | 200 条 | `pg_ripple.vacuum_dict_batch_size` | `vacuum_dictionary()` |

---

## 硬限制（不可配置）

这些限制已内嵌到实现中：

| 约束 | 值 | 备注 |
|---|---|---|
| 字典哈希空间 | 2^64（XXH3-128） | 实际碰撞概率可忽略不计 |
| 最大 SID（语句 ID） | 2^63 − 1（i64） | PostgreSQL 序列最大值 |
| 最大命名图 ID | 2^63 − 1（i64） | 同一序列命名空间 |
| 最大谓词 ID | 2^63 − 1（i64） | 同一序列命名空间 |
| SPARQL 1.1 规范合规性 | 完整 | SELECT, CONSTRUCT, DESCRIBE, ASK, UPDATE, LOAD, CLEAR, DROP, ADD, MOVE, COPY |
| PostgreSQL 目标 | 仅 18.x | pgrx 0.18，不支持较旧的 PG |

---

## 建议的生产限制

对于 32 核 / 128 GB RAM 服务器上的 100 GB RDF 数据集：

```ini
# postgresql.conf 覆盖
pg_ripple.dictionary_cache_size     = 1000000
pg_ripple.cache_budget_mb           = 512
pg_ripple.sparql_max_rows           = 100000
pg_ripple.export_max_rows           = 500000
pg_ripple.merge_threshold           = 50000
pg_ripple.merge_workers             = 4
pg_ripple.datalog_parallel_workers  = 8
pg_ripple.federation_timeout        = 60
pg_ripple.federation_parallel_max   = 8
pg_ripple.pagerank_max_iterations   = 200
```

有关完整调优指南，请参阅[性能调优](../operations/tuning.md)。