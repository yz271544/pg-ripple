# GUC 参考

所有 pg_ripple 配置参数都使用 `ALTER SYSTEM SET`、`SET`（会话级别）或在 `postgresql.conf` 中设置。在 `ALTER SYSTEM` 之后使用 `SELECT pg_reload_conf()` 重新加载。

---

## 通用参数

### `pg_ripple.max_path_depth`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10` |
| 范围 | 1–100 |

SPARQL 属性路径（`*`、`+`）的最大递归深度。对于深度嵌套的图增加；对于更严格的资源限制降低。

---

### `pg_ripple.property_path_max_depth` *(已弃用)*

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `64` |
| 范围 | 1–100 000 |
| 状态 | 自 v0.38.0 起 **已弃用** — 改用 `max_path_depth` |

`max_path_depth` 的遗留别名。设置此 GUC 仍然有效，但会发出弃用通知。它将在未来的主要版本中删除。

---

### `pg_ripple.federation_timeout`

| | |
|---|---|
| 类型 | Integer (毫秒) |
| 默认值 | `5000` |

出站 SPARQL 联邦请求的超时。

---

### `pg_ripple.export_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000` |

当 SPARQL 游标将结果流式传输回调用者时每页获取的行数。这控制了 Rust 端的峰值内存：同时从字典 ID 解码并保留在内存中的结果行最多为 `export_batch_size`。增加以获得更高的吞吐量，代价是更多内存；降低以获得更严格的内存预算。

另请参阅：`arrow_batch_size`（Arrow IPC 批处理）独立运行 — 两个 GUC 管理不同的输出路径，可以单独调整。

---

### `pg_ripple.arrow_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000` |
| 最小值 | `1` |

在 Arrow Flight 批量导出期间打包到每个 Arrow IPC `RecordBatch` 中的行数。较大的值会产生更少的 IPC 帧和更低的每帧开销；较小的值允许使用者更快地开始处理结果。

**与 `export_batch_size` 的交互**：`arrow_batch_size` 控制 Arrow Flight 响应主体内的 IPC 批处理粒度。`export_batch_size` 控制从 PostgreSQL 每页获取多少 SPARQL 游标行。这两个参数在独立的代码路径上运行，可以单独调整。

```sql
-- 为高带宽 Arrow 批量导出进行调整
SET pg_ripple.arrow_batch_size = 5000;
```

---

### `pg_ripple.max_predicate_union_branches`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `500` |
| 范围 | 10–10 000 |

当变量谓词 SPARQL 模式扩展为 `UNION ALL` 时允许的最大 VP 表分支数。超过此限制的查询将失败并显示 `PT0601`；使用命名谓词或有意提高限制。

---

### `pg_ripple.vp_promotion_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10000` |
| 最小值 | `100` |

VP 表提升的 COPY 阶段批处理大小：当稀有谓词三元组计数超过 `vp_promotion_threshold` 时，提升工作线程以该大小的批次将行从 `_pg_ripple.vp_rare` 复制到新的专用 VP 表中。较大的批次减少提升时间，但代价是每个批次的 WAL 记录较大；较小的批次减少 WAL 压力并允许其他事务在批次之间进行。

**与其他批处理大小 GUC 的交互**：VP 提升在后台工作线程中运行，与 SPARQL 游标流式传输或 Arrow IPC 导出无关。更改此 GUC 仅影响提升吞吐量，不影响查询性能。

```sql
-- 在大型提升期间减少 WAL 压力
SET pg_ripple.vp_promotion_batch_size = 1000;
```

---

## 嵌入 / 向量参数（v0.27.0+）

这些 GUC 控制 v0.27.0 中引入的 pgvector 集成。当 pgvector 不存在时，所有嵌入函数都会优雅降级。

---

### `pg_ripple.pgvector_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |

所有向量嵌入路径的主开关。设置为 `off` 以禁用嵌入存储、相似度搜索和 SPARQL `pg:similar()`，而不卸载 pgvector。用于临时禁用该功能。

```sql
-- 在会话级别为批量加载禁用
SET pg_ripple.pgvector_enabled = off;
```

---

### `pg_ripple.embedding_api_url`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |

OpenAI 兼容嵌入 API 的基础 URL。扩展在发出请求时将 `/embeddings` 附加到此 URL。

```sql
ALTER SYSTEM SET pg_ripple.embedding_api_url = 'https://api.openai.com/v1';
-- 对于 Ollama（本地）：
ALTER SYSTEM SET pg_ripple.embedding_api_url = 'http://localhost:11434/v1';
```

---

### `pg_ripple.embedding_api_key`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |

在嵌入 API 请求中作为 `Authorization: Bearer <key>` 发送的 Bearer 令牌。对于不需要身份验证的本地模型，设置为任何非空字符串（例如，`'local'`）。

> **安全性：** 避免在 `postgresql.conf` 中存储 API 密钥。使用 `ALTER SYSTEM` 并限制 `pg_hba.conf` 访问，或通过应用程序代码中的会话级别 `SET` 注入密钥。

---

### `pg_ripple.embedding_model`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |

在嵌入 API 请求的 `"model"` 字段中传递的模型名称。

```sql
ALTER SYSTEM SET pg_ripple.embedding_model = 'text-embedding-3-small';
-- 或对于 Ollama：
ALTER SYSTEM SET pg_ripple.embedding_model = 'nomic-embed-text';
```

---

### `pg_ripple.embedding_dimensions`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1536` |
| 范围 | 1–65535 |

嵌入模型的预期输出维度。必须与模型的输出长度匹配。常见值：

| 模型 | 维度 |
|---|---|
| `text-embedding-3-small` | 1536 |
| `text-embedding-3-large` | 3072 |
| `text-embedding-ada-002` | 1536 |
| `nomic-embed-text`（Ollama） | 768 |

---

### `pg_ripple.embedding_index_type`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无 — pgvector 存在时为 HNSW)* |
| 值 | `hnsw`、`ivfflat` |

`_pg_ripple.embeddings` 表的索引类型。HNSW 是默认值，建议用于大多数工作负载。IVFFlat 使用更少的内存，但需要 `lists` 参数调整。

---

### `pg_ripple.embedding_precision`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无 — 全 float4 精度)* |
| 值 | *(未设置)*、`half`、`binary` |

嵌入向量的存储精度。以准确性为代价减少磁盘/内存使用：

| 值 | pgvector 类型 | 注释 |
|---|---|---|
| *(未设置)* | `vector(N)` | 全 32 位浮点；最高准确性 |
| `half` | `halfvec(N)` | 16 位浮点；~50% 存储减少 |
| `binary` | `bit(N)` | 1 位量化；~97% 存储减少，准确性较低 |

> **注意：** 在存储数据后更改精度需要重新运行迁移或手动更改列类型并重新嵌入。

---

## v0.37.0：墓碑 GC 和错误安全

### `pg_ripple.tombstone_gc_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `sighup`（共享：需要服务器信号，而不是每个会话） |

当为 `on` 时，如果剩余墓碑计数超过 `tombstone_gc_threshold × main_row_count`，pg_ripple 在每个合并周期后自动对谓词的墓碑表发出 `VACUUM ANALYZE`。设置为 `off` 以禁用自动墓碑清理（在手动管理 VACUUM 时有用）。

### `pg_ripple.tombstone_gc_threshold`

| | |
|---|---|
| 类型 | String (十进制) |
| 默认值 | `0.05` (5%) |
| 范围 | `0.0` – `1.0` |
| 上下文 | `sighup` |

触发合并周期后自动 `VACUUM` 的墓碑与主行比率。当剩余墓碑计数除以新主表行数超过此值时，会在墓碑表上调度 `VACUUM ANALYZE`。

较低的值（例如 `0.01`）更积极地触发 VACUUM；较高的值（例如 `0.20`）在清理之前允许更多的墓碑膨胀。

---

## v0.37.0：GUC 验证器规则

以下字符串枚举 GUC 现在在 `SET` 时拒绝无效值并报错。以前，无效值在执行路径检查它们之前会被默默忽略。

| GUC | 有效值 |
|---|---|
| `pg_ripple.inference_mode` | `off`、`on_demand`、`materialized` |
| `pg_ripple.enforce_constraints` | `off`、`warn`、`error` |
| `pg_ripple.rule_graph_scope` | `default`、`all` |
| `pg_ripple.shacl_mode` | `off`、`sync`、`async` |
| `pg_ripple.describe_strategy` | `cbd`、`scbd`、`simple` |

**`pg_ripple.rls_bypass` 范围更改（v0.37.0）**：当通过 `shared_preload_libraries` 加载 pg_ripple 时，此 GUC 现在在 `PGC_POSTMASTER` 范围注册。这可以防止会话使用 `SET LOCAL pg_ripple.rls_bypass = on` 绕过图级 RLS。

---

## v0.42.0：并行合并工作线程

### `pg_ripple.merge_workers`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1` |
| 范围 | `1` – `16` |
| 上下文 | `postmaster`（仅启动；在 `postgresql.conf` 中设置） |

后台合并工作线程进程的数量。每个工作线程拥有 VP 谓词的不相交轮询切片。工作线程使用 `pg_advisory_lock` 来防止冲突；空闲工作线程从过载的同级窃取工作。对于具有许多不同谓词（> 50）的工作负载，增加此值会有所帮助。

---

## v0.42.0：基于成本的联邦规划器

### `pg_ripple.federation_planner_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |

当为 `on` 时，pg_ripple 使用从远程 SPARQL 端点收集的 VoID 统计信息，按升序估计成本对 SERVICE 执行顺序进行排序。当为 `off` 时，SERVICE 子句按文档顺序执行。

### `pg_ripple.federation_stats_ttl_secs`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `3600` (1 小时) |
| 范围 | `0` – `86400` |
| 上下文 | `userset` |

远程端点的缓存 VoID 统计信息被视为陈旧之前的秒数。将 `0` 设置为禁用缓存（每次查询重新获取）。

### `pg_ripple.federation_parallel_max`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `4` |
| 范围 | `1` – `64` |
| 上下文 | `userset` |

pg_ripple 在单个查询中并发执行的最大远程 SERVICE 子句数。设置为 `1` 以禁用并行 SERVICE 执行。

### `pg_ripple.federation_parallel_timeout`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `60` (秒) |
| 范围 | `1` – `3600` |
| 上下文 | `userset` |

执行并行 SERVICE 子句时每个端点的超时。在此限制内不响应的端点返回空结果集（带有 WARNING）。不影响顺序 SERVICE 执行。

### `pg_ripple.federation_inline_max_rows`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10000` |
| 范围 | `1` – `1000000` |
| 上下文 | `userset` |

传递给远程 SERVICE 子句的 VALUES 绑定表中的最大行数。当来自本地图的结果集超过此限制时，pg_ripple 会自动将绑定假脱机到临时表中（记录 PT620 INFO）并以较小的批次向远程端点发出多个较小的请求。如果远程端点强制执行查询复杂性限制，请设置为较低的值。

### `pg_ripple.federation_allow_private`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `superuser` |

> **安全关键 GUC** — 只有超级用户可以设置此参数。

当为 `off`（默认值）时，`register_endpoint()` 拒绝主机名解析为 loopback 地址（`127.0.0.0/8`）、link-local 地址（`169.254.0.0/16`）、任何 RFC-1918 私有范围（`10/8`、`172.16/12`、`192.168/16`）或 IPv6 等效地址的端点。这可以防止通过恶意 SPARQL SERVICE 调用进行服务器端请求伪造（SSRF）。

仅在受控环境中将 `on` 远程端点是受信任的内部服务（例如，Docker 网络中的本地 Fuseki 实例）。

---

## v0.42.0：owl:sameAs 安全性

### `pg_ripple.sameas_max_cluster_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100000` |
| 范围 | `0` – `2147483647` |
| 上下文 | `userset` |

在规范化被跳过并显示 PT550 WARNING 之前，单个 `owl:sameAs` 等价集群中的最大实体数。超过此限制的单个集群通常是数据质量问题（例如，错误声明的 `owl:sameAs owl:Thing`）。设置为 `0` 以禁用检查（无限制）。

---

## v0.46.0：TopN 下推和 Datalog 序列批处理

### `pg_ripple.topn_pushdown`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |

当为 `on`（默认值）时，包含 `ORDER BY` 和 `LIMIT N`（没有 `OFFSET > 0` 也没有 `DISTINCT`）的 SPARQL SELECT 查询将 SQL 发出为 `… ORDER BY … LIMIT N`，而不是获取所有行并在解码后丢弃。

设置 `off` 以全局禁用优化 — 例如，在调试期间怀疑 TopN 下推产生不正确的结果时。

`sparql_explain()` 输出包括一个 `"topn_applied": true/false` 键，指示下推是否应用于特定查询。

### `pg_ripple.datalog_sequence_batch`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10000` |
| 范围 | `100` – `1000000` |
| 上下文 | `userset` |

每个并行 Datalog 工作线程每批次保留的 SID（statement-ID）范围。在启动 N 个并行层工作线程之前，协调器原子地将全局 `_pg_ripple.statement_id_seq` 序列前进 `N * datalog_sequence_batch`，然后为每个工作线程分配一个独占子范围。工作线程使用预先计算的 SID 插入三元组，而不接触共享序列，从而消除争用。

如果并行推导工作线程经常在序列上发生冲突，则增加此值。当推导产生的三元组少于每个批次的预期时，降低它以减少未使用的 SID 间隙。

---

## v0.48.0 GUC

### `pg_ripple.federation_max_response_bytes`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `104857600` (100 MiB) |
| 范围 | `-1`（已禁用） – `2147483647` |
| 上下文 | `userset` |

联邦（SERVICE）响应主体的最大允许大小（以字节为单位）。当远程 SPARQL 端点返回大于此值的 JSON 响应时，pg_ripple 引发错误代码 **PT543** 并中止查询。设置为 `-1` 以禁用限制（不建议用于具有不受信任的联邦端点的部署）。

```sql
-- 允许单个查询最多 500 MiB 响应
SET pg_ripple.federation_max_response_bytes = 524288000;

-- 禁用限制（仅限受信任的内部网络）
SET pg_ripple.federation_max_response_bytes = -1;
```

---

## v0.47.0：经过验证的字符串 GUC

以下六个字符串值 GUC 现在在 SET 时拒绝无效值（以前接受无效值并在运行时默默忽略）。

### `pg_ripple.federation_on_error`

| | |
|---|---|
| 类型 | String |
| 默认值 | `warning` |
| 有效值 | `warning`、`error`、`empty` |
| 上下文 | `userset` |

控制当 SERVICE 调用完全失败时的行为。`warning` 发出 PT610 WARNING 并返回该端点的空绑定集。`error` 引发错误并中止查询。`empty` 默默返回该端点的零行。

### `pg_ripple.federation_on_partial`

| | |
|---|---|
| 类型 | String |
| 默认值 | `empty` |
| 有效值 | `empty`、`use` |
| 上下文 | `userset` |

控制当 SERVICE 响应流在传输过程中被中断时的行为（例如，远程端点断开连接）。`empty` 丢弃部分结果并返回零行。`use` 保留错误前接收到的行。

### `pg_ripple.sparql_overflow_action`

| | |
|---|---|
| 类型 | String |
| 默认值 | `warn` |
| 有效值 | `warn`、`error` |
| 上下文 | `userset` |

当 SPARQL SELECT 结果集超过 `sparql_max_rows` 时采取的操作（`OFFSET` = `0`）。`warn` 截断结果集并发出 PT601 WARNING。`error` 引发错误。

### `pg_ripple.tracing_exporter`

| | |
|---|---|
| 类型 | String |
| 默认值 | `stdout` |
| 有效值 | `stdout`、`otlp` |
| 上下文 | `sighup` |

OpenTelemetry 跟踪导出器。`stdout` 将 span 写入 PostgreSQL 日志（无网络开销）。`otlp` 通过 OTLP gRPC 协议将 span 发送到由 `OTEL_EXPORTER_OTLP_ENDPOINT` 环境变量指定的端点。

### `pg_ripple.embedding_index_type`

| | |
|---|---|
| 类型 | String |
| 默认值 | `hnsw` |
| 有效值 | `hnsw`、`ivfflat` |
| 上下文 | `userset` |

`_pg_ripple.embeddings` 表的索引类型。`hnsw` 构建 HNSW 图索引；`ivfflat` 构建 IVFFlat 索引。在嵌入已被索引后更改此设置需要 `REINDEX TABLE _pg_ripple.embeddings`。

### `pg_ripple.embedding_precision`

| | |
|---|---|
| 类型 | String |
| 默认值 | `single` |
| 有效值 | `single`、`half`、`binary` |
| 上下文 | `userset` |

嵌入向量的存储精度。`single` 使用 pgvector `vector(N)` 类型；`half` 使用 `halfvec(N)`；`binary` 使用 `bit(N)`。

---

## AI 和 LLM 集成参数（v0.49.0）

### `pg_ripple.llm_endpoint`

| | |
|---|---|
| 类型 | String |
| 默认值 | `''`（空 — 已禁用） |
| 上下文 | `userset` |

`sparql_from_nl()` 使用的 OpenAI 兼容 `/v1/chat/completions` API 的基础 URL。为空时，调用 `sparql_from_nl()` 会立即引发 PT700。设置为 `'mock'` 以使用内置测试模拟而不使用真正的 LLM。示例：`https://api.openai.com/v1`、`http://localhost:11434/v1`（Ollama）。

> **注意（H16-04，v0.112.0）**：此 GUC 在 pg_ripple 扩展本身内是无操作的。HTTP 配套服务 `pg_ripple_http` 在为 `/rules/{id}/explain` 端点服务时读取此 GUC（通过数据库连接）以处理 Datalog 规则解释的 LLM 富化。`sparql_from_nl()` 等直接扩展函数也遵循此 GUC。不要将此 GUC 设置为原始 API 密钥 — 使用 `pg_ripple.llm_api_key_env` 进行安全密钥管理。

---

### `pg_ripple.llm_model`

| | |
|---|---|
| 类型 | String |
| 默认值 | `gpt-4o` |
| 上下文 | `userset` |

在聊天完成请求主体的 `model` 字段中传递的 LLM 模型标识符。支持的值取决于端点 — 例如 `gpt-4o`、`gpt-4-turbo`、`llama3`、`mistral`。

---

### `pg_ripple.llm_api_key_env`

| | |
|---|---|
| 类型 | String |
| 默认值 | `PG_RIPPLE_LLM_API_KEY` |
| 上下文 | `userset` |

`sparql_from_nl()` 在调用时从中读取 Bearer API 密钥的环境变量名称。值必须匹配 `^[A-Z_][A-Z0-9_]*$`；原始值被拒绝。仅对于 0.131 之前的迁移，超级用户可以设置 `pg_ripple.llm_api_key_env_allow_raw = on`。密钥永远不会存储在数据库中或在 `pg_settings` 中可见。

### `pg_ripple.llm_api_key_env_allow_raw`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `suset` |

用于从 0.131 之前的安装迁移原始 API 密钥值的临时超级用户专用逃生舱。迁移后保持 `off`。

---

### `pg_ripple.llm_include_shapes`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |

当为 `on` 时，`sparql_from_nl()` 发送的 LLM 提示包括活动 SHACL 形状的摘要作为其他模式上下文。当形状非常大或 LLM 上下文窗口有限时禁用。

---

## v0.54.0 GUC — 高可用性和逻辑复制

### `pg_ripple.replication_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |

当为 `on` 时，启动 `logical_apply_worker` 后台工作线程，该线程订阅 `pg_ripple_pub` 发布并将传入的 N-Triples 批次应用于本地存储。在主服务器上需要 `wal_level = logical`。

需要服务器重启（或 SIGHUP）才能生效。

---

### `pg_ripple.replication_conflict_strategy`

| | |
|---|---|
| 类型 | String |
| 默认值 | `last_writer_wins` |
| 上下文 | `sighup` |

当传入三元组的 `(s, p, g)` 已存在于本地存储中但具有不同的对象或 SID 时，逻辑应用工作线程使用的冲突解决策略。

支持的值：

| 值 | 行为 |
|-------|-----------|
| `last_writer_wins` | 保留具有最高 Statement ID（SID）的行。这是默认值，符合最终一致性语义。 |

---

### `pg_ripple.strict_dictionary`（D13-02，v0.86.0）

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |

当为 `on` 时，所有字典查找都将失败并显示 `PT400` 错误，而不是为未知的 IRI 和字面值默默插入新字典条目。这对于未知术语表示数据质量问题的只读副本和验证管道很有用。

当为 `off`（默认值）时，未知术语在首次访问时插入到字典中（惰性编码）。这适用于大多数写入工作负载。

---

### `pg_ripple.plan_cache_capacity`（D13-02，v0.86.0）

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1024` |
| 最小值 / 最大值 | `1` / `32768` |
| 上下文 | `sighup` |

在进程内 LRU 计划缓存中保留的已编译 SPARQL 查询计划的最大数量。每个条目存储 SQL 字符串、投影变量列表和解码的类型元数据。缓存命中率可通过 `pg_ripple.explain_sparql(query, 'plan_cache_stats')` 或 `pg_ripple_plan_cache_hit_ratio` Prometheus 指标查看。

---

### `pg_ripple.cdc_slot_cleanup_timeout_ms`（D13-02，v0.86.0）

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `5000` |
| 最小值 / 最大值 | `100` / `300000` |
| 上下文 | `sighup` |

在扩展卸载期间或调用 `pg_ripple.cleanup_cdc_slot()` 函数时 CDC 复制槽清理的超时（以毫秒为单位）。如果槽处于活动状态（连接了订阅者），清理将等待这么多毫秒，然后返回错误。此方案的崩溃恢复测试在 `tests/crash_recovery/cdc_slot_cleanup_during_kill.sh` 中。

---

## Datalog 推理参数

### `pg_ripple.inference_mode`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'on_demand'` |
| 有效值 | `off`、`on_demand`、`materialized` |
| 上下文 | `userset` |

控制何时评估 Datalog 规则。

| 值 | 行为 |
|---|---|
| `off` | 不执行推导。`infer()` 是无操作。 |
| `on_demand` | 在调用 `infer()` 时明确运行推导。 |
| `materialized` | 推导结果保持具体化并增量更新。 |

---

### `pg_ripple.enforce_constraints`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'warn'` |
| 有效值 | `off`、`warn`、`error` |
| 上下文 | `userset` |

控制如何强制执行 Datalog 约束规则（`:- body.`）。

| 值 | 行为 |
|---|---|
| `off` | 忽略约束违规。 |
| `warn` | 违规产生 PostgreSQL WARNING。 |
| `error` | 违规引发 PT404 并中止事务。 |

---

### `pg_ripple.rule_graph_scope`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'all'` |
| 有效值 | `default`、`all` |
| 上下文 | `userset` |

控制当 Datalog 规则主体原子没有显式图注释时搜索哪些图。

| 值 | 行为 |
|---|---|
| `default` | 未限定范围的原子仅匹配默认图（g = 0）。 |
| `all` | 未限定范围的原子匹配任何命名图中的三元组。 |

---

### `pg_ripple.magic_sets`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.29.0 |

magic-set 转换的主开关 — 目标导向推导，重写规则以避免派生无关事实。启用 `infer_goal()` 以执行目标导向的需求驱动求值。禁用以调试不正确的 magic-set 重写。

---

### `pg_ripple.datalog_cost_reorder`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.29.0 |

当为 `on` 时，在 SQL 编译之前按升序估计的 VP 表基数对 Datalog 规则主体原子进行排序。这将最具选择性的连接放在首位，从而减少中间结果大小。

---

### `pg_ripple.datalog_antijoin_threshold`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000` |
| 上下文 | `userset` |
| 自 | v0.29.0 |

否定主体原子（NOT EXISTS）使用反连接的最小 VP 表行计数。低于此阈值时，使用相关子查询。

---

### `pg_ripple.delta_index_threshold`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `500` |
| 上下文 | `userset` |
| 自 | v0.29.0 |

创建 B-tree 索引之前的最小半朴素 delta 临时表行计数。对于小型 delta 集，顺序扫描更快。

---

### `pg_ripple.rule_plan_cache`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.30.0 |

Datalog 规则计划缓存的主开关。当为 `on` 时，每个规则集的已编译 SQL 被缓存在内存中并在 `infer()` 调用之间重用，从而避免重复编译。

---

### `pg_ripple.rule_plan_cache_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `64` |
| 上下文 | `userset` |
| 自 | v0.30.0 |

在计划缓存中保留其已编译 SQL 的最大规则集数。

---

### `pg_ripple.sameas_reasoning`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.31.0 |

`owl:sameAs` 实体规范化的主开关。当为 `on` 时，通过 `owl:sameAs` 链接的实体在推导和查询求值之前合并到其规范代表。

---

### `pg_ripple.demand_transform`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.31.0 |

需求转换的主开关 — 一种重写规则以将绑定从查询目标传播到规则求值中的技术，从而减少搜索空间。

---

### `pg_ripple.wfs_max_iterations`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 上下文 | `userset` |
| 自 | v0.32.0 |

良好基础语义（`infer_wfs()`）的交替不动点轮次的安全上限。当达到上限时，会发出 PT520 WARNING 并返回部分结果。

---

### `pg_ripple.tabling`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.32.0 |

子句包含列表化缓存的主开关。列表化记忆中间查询结果，消除递归 SPARQL 和 Datalog 查询中相同子目标的冗余重新求值。

---

### `pg_ripple.tabling_ttl`

| | |
|---|---|
| 类型 | Integer (秒) |
| 默认值 | `300` |
| 上下文 | `userset` |
| 自 | v0.32.0 |

列表化缓存条目的生存时间（以秒为单位）。条目将在此间隔后被逐出。设置为 `0` 以无限期保留条目（直到会话结束）。

---

### `pg_ripple.datalog_max_depth`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `0` (无限制) |
| 上下文 | `userset` |
| 自 | v0.34.0 |

有界深度 Datalog 不动点终止的最大深度。当为 `0` 时，不动点一直运行直到收敛。对于大型图上的递归规则，设置一个界限（例如 `10`）以限制计算时间。

---

### `pg_ripple.dred_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.34.0 |

Delete-Rederive（DRed）算法的主开关。当为 `on` 时，删除基础三元组仅重新派生受影响的派生三元组的最小集合，而不是重新计算所有内容。设置 `off` 以始终使用完全重新计算。

---

### `pg_ripple.dred_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000` |
| 上下文 | `userset` |
| 自 | v0.34.0 |

每个 DRed 事务处理的已删除基础三元组的最大数量。

---

### `pg_ripple.datalog_parallel_workers`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `4` |
| 范围 | `1`–`64` |
| 上下文 | `userset` |
| 自 | v0.35.0 |

Datalog 层求值的最大并行后台工作线程数。独立的层分布在工作线程之间以进行并发求值。

---

### `pg_ripple.datalog_parallel_threshold`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10000` |
| 上下文 | `userset` |
| 自 | v0.35.0 |

在应用并行组分析之前，层中所有规则的最小估计总行计数。小层按顺序求值。

---

### `pg_ripple.lattice_max_iterations`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000` |
| 上下文 | `userset` |
| 自 | v0.36.0 |

基于 lattice 的 Datalog 推导（`infer_lattice()`）的最大不动点迭代。有关收敛失败，请参阅 PT540。

---

### `pg_ripple.datalog_max_derived`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `0` (无限制) |
| 上下文 | `userset` |
| 自 | v0.40.0 |

单个 `infer()` 调用产生的最大派生事实数。当达到限制时，推导停止并返回部分结果。在开发中用作安全保护很有用。

---

### `pg_ripple.owl_profile`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'RL'` |
| 有效值 | `RL`、`EL`、`QL`、`off` |
| 上下文 | `userset` |
| 自 | v0.57.0 |

活动的 OWL 2 推理配置文件。选择由 `load_rules_builtin('owl')` 加载的内置 OWL 规则集。

| 值 | 配置文件 |
|---|---|
| `RL` | OWL 2 RL — 完整规则集，在数据大小上是多项式的 |
| `EL` | OWL 2 EL — 针对大型类层次结构（TBox 重型本体）进行了优化 |
| `QL` | OWL 2 QL — 查询重写模式，最小具体化 |
| `off` | 未加载内置 OWL 规则 |

---

### `pg_ripple.probabilistic_datalog`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.57.0 |

启用具有规则置信度权重（`@weight` 注释）的实验性概率 Datalog。请参阅[概率推理](../features/uncertain-knowledge.md)。

---

### `pg_ripple.datalog_citus_dispatch`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.62.0 |

当为 `on` 时，将 Datalog 层迭代 `INSERT…SELECT` 语句包装在 `run_command_on_all_nodes()` 中，以跨 Citus 工作线程进行分布式执行。需要 `pg_ripple.citus_sharding_enabled = on`。

---

### `pg_ripple.prob_datalog_cyclic`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.87.0 |

允许在循环规则集上进行概率求值。循环概率程序需要近似不动点求值；必须显式启用。

---

### `pg_ripple.prob_datalog_max_iterations`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 上下文 | `userset` |
| 自 | v0.87.0 |

当 `prob_datalog_cyclic = on` 时的最大半朴素推导轮次。在此限制之后，发出 WARNING 并返回部分结果。

---

### `pg_ripple.prob_datalog_convergence_delta`

| | |
|---|---|
| 类型 | Float |
| 默认值 | `0.001` |
| 上下文 | `userset` |
| 自 | v0.87.0 |

循环概率 Datalog 的提前退出阈值。当所有原子的最大置信度 delta 降至该值以下时，迭代停止。

---

## SPARQL 查询引擎参数

### `pg_ripple.bgp_reorder`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.13.0 |

根据 `pg_stats` 选择性估计启用 BGP（基本图模式）连接重排序。最具选择性的三元组模式放在连接顺序的首位。

---

### `pg_ripple.parallel_query_min_joins`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `3` |
| 上下文 | `userset` |
| 自 | v0.13.0 |

在为生成的 SQL 启用 PostgreSQL 并行查询工作线程之前，SPARQL 查询中的最小 VP 表连接数。

---

### `pg_ripple.sparql_strict`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.21.0 |

当为 `on` 时，对不受支持的 SPARQL 内置函数引发 `ERRCODE_FEATURE_NOT_SUPPORTED`。当为 `off` 时，不受支持的函数默默求值为 UNDEF。

---

### `pg_ripple.wcoj_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.36.0 |

最坏情况最优连接（WCOJ / Leapfrog Triejoin）优化的主开关。当为 `on` 时，超过 `wcoj_min_tables` 的循环连接模式（例如三角形、团）使用 WCOJ 执行器求值，而不是 PostgreSQL 的哈希连接路径。

---

### `pg_ripple.wcoj_min_tables`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `3` |
| 上下文 | `userset` |
| 自 | v0.36.0 |

在应用 WCOJ 分析之前的最少 VP 表连接数。

---

### `pg_ripple.wcoj_min_cardinality`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `0` |
| 上下文 | `userset` |
| 自 | v0.79.0 |

使用 WCOJ 执行器之前的最小 VP 表基数。低于此阈值时，使用标准 SQL 哈希连接路径。

---

### `pg_ripple.sparql_max_rows`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `0` (无限制) |
| 上下文 | `userset` |
| 自 | v0.40.0 |

SPARQL SELECT 或 CONSTRUCT 查询返回的最大行数。当超过时，行为由 `sparql_overflow_action` 控制。设置为 `0` 表示无限制。

---

### `pg_ripple.sparql_overflow_action`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'truncate'` |
| 有效值 | `truncate`、`error` |
| 上下文 | `userset` |
| 自 | v0.40.0 |

当超过 `sparql_max_rows` 时采取的操作。`truncate` 返回带有 PT640 通知的前 N 行；`error` 将 PT640 引发为错误。

---

### `pg_ripple.sparql_max_algebra_depth`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `256` |
| 上下文 | `userset` |
| 自 | v0.51.0 |

SPARQL 查询的最大允许代数树深度。超过此限制的查询引发 PT440。

---

### `pg_ripple.sparql_max_triple_patterns`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `4096` |
| 上下文 | `userset` |
| 自 | v0.51.0 |

单个 SPARQL 查询中的最大三元组模式数。超过此限制的查询引发 PT440。

---

### `pg_ripple.describe_strategy`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'cbd'` |
| 有效值 | `cbd`、`scbd`、`simple` |
| 上下文 | `userset` |

SPARQL DESCRIBE 查询的算法。

| 值 | 描述 |
|---|---|
| `cbd` | 简洁有界描述 — 所有以资源为主语的三元组，加上空白节点闭包 |
| `scbd` | 对称 CBD — 添加资源为宾语的三元组 |
| `simple` | 仅返回直接主语三元组，没有空白节点闭包 |

---

### `pg_ripple.strict_sparql_filters`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.81.0 |

当为 `on` 时，FILTER 表达式中的未知内置函数名将引发错误（PT422），而不是求值为 UNDEF。

---

### `pg_ripple.fuzzy_max_input_length`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `4096` |
| 范围 | `1`–`65536` |
| 上下文 | `userset` |
| 自 | v0.89.0 |

`pg:fuzzy_match()` 和 `pg:token_set_ratio()` 的最大输入字符串长度。比此限制更长的参数引发 PT0308。防止算法复杂性攻击。

---

### `pg_ripple.all_nodes_predicate_limit`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `500` |
| 上下文 | `userset` |
| 自 | v0.82.0 |

通配符属性路径扩展（`*` 或 `+` 无显式谓词）中使用的最大谓词数。当架构具有更多谓词时，仅按三元组计数展开 top-N。

---

## 存储和 HTAP 参数

### `pg_ripple.vp_promotion_threshold` *(别名：vp_promotion_threshold)*

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000` |
| 上下文 | `sighup` |

谓词获得专用 VP 表之前的最小三元组计数。低于此阈值的谓词存储在合并的 `vp_rare` 表中。

---

### `pg_ripple.merge_threshold`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10000` |
| 上下文 | `sighup` |

合并工作线程触发该谓词的合并周期之前的 delta 表中的最小行计数。

---

### `pg_ripple.merge_interval_secs`

| | |
|---|---|
| 类型 | Integer (秒) |
| 默认值 | `60` |
| 上下文 | `sighup` |

合并工作线程轮询间隔之间的最大秒数。即使 delta 表低于阈值，合并工作线程也会以此频率检查工作。

---

### `pg_ripple.merge_retention_seconds`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `60` |
| 上下文 | `sighup` |

合并后保留旧主表的秒数，然后删除它。允许对旧主表的长时间运行读取优雅完成。

---

### `pg_ripple.latch_trigger_threshold`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10000` |
| 上下文 | `sighup` |

在触发（闩锁）合并工作线程以提前唤醒之前，一个批次中写入的三元组数。

---

### `pg_ripple.worker_database`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |
| 上下文 | `postmaster` |

后台合并工作线程连接到的数据库的名称。必须与安装 `pg_ripple` 扩展的数据库匹配。使用 `shared_preload_libraries` 时需要。

---

### `pg_ripple.dedup_on_merge`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |

当为 `on` 时，HTAP 合并使用 `DISTINCT ON` 对 `(s, o, g)` 行进行去重，保留具有最低 SID 的行。对于可能插入重复的写入工作负载很有用。

---

### `pg_ripple.cache_budget_mb`

| | |
|---|---|
| 类型 | Integer (MiB) |
| 默认值 | `64` |
| 上下文 | `postmaster` |

字典编码缓存的共享内存预算上限（以兆字节为单位）。必须与 `dictionary_cache_size` 同步增加。

---

### `pg_ripple.auto_analyze`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `sighup` |

当为 `on` 时，后台合并工作线程在成功合并周期后立即对每个 VP 主表运行 `ANALYZE`，保持规划器统计信息是最新的。

---

### `pg_ripple.named_graph_optimized`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `postmaster` |

当为 `on` 时，为每个专用 VP 表添加 `(g, s, o)` B-tree 索引以加快命名图范围查询。默认情况下禁用以避免索引膨胀。如果大多数查询使用 `GRAPH` 模式，则启用。

---

### `pg_ripple.normalize_iris`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `sighup` |
| 自 | v0.55.0 |

当为 `on` 时，在字典编码之前将 IRI 字符串规范化为 NFC Unicode。确保仅在 Unicode 规范化形式上不同的语义等效 IRI 映射到同一字典条目。

---

### `pg_ripple.rls_bypass`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `postmaster`（需要 `shared_preload_libraries`） |
| 访问 | 仅超级用户 |
| 自 | v0.37.0 |

绕过图级行级安全策略的超级用户覆盖。在 `PGC_POSTMASTER` 范围注册 — 会话级别 `SET` 被拒绝，防止权限提升。

---

### `pg_ripple.predicate_cache_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.38.0 |

启用后端本地谓词 OID 缓存，该缓存存储从谓词 ID 到 VP 表 OID 的映射。禁用以调试目录查找问题。

---

### `pg_ripple.cdc_bridge_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.52.0 |

CDC → pg_tide outbox 桥接工作线程的主开关。当为 `on` 时，三元组插入和删除被序列化为 JSON-LD 事件并发布到配置的 pg_tide outbox。

---

### `pg_ripple.cdc_bridge_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 上下文 | `sighup` |
| 自 | v0.52.0 |

在刷新到 outbox 表之前批处理的最大 CDC 通知数。

---

### `pg_ripple.cdc_bridge_flush_ms`

| | |
|---|---|
| 类型 | Integer (毫秒) |
| 默认值 | `200` |
| 上下文 | `sighup` |
| 自 | v0.52.0 |

CDC 桥接工作线程刷新周期之间的最大毫秒数。

---

### `pg_ripple.cdc_bridge_outbox_table`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |
| 上下文 | `sighup` |
| 自 | v0.52.0 |

CDC 桥接工作线程将 JSON-LD 事件发布到的 pg_tide outbox 的传统名称。当 `cdc_bridge_enabled = on` 时，outbox 必须已通过 `tide.outbox_create(...)` 存在。

---

### `pg_ripple.trickle_integration`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.52.0 |

中继桥接集成功能的传统主开关。当为 `off` 时，即使安装了 pg_tide，CDC 桥接代码路径也被禁用。分别使用 `pg_ripple.pg_trickle_available()` 进行 IVM 支持的实时视图。

---

### `pg_ripple.replication_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.54.0 |

启用 RDF 逻辑复制使用者工作线程。当为 `on` 时，工作线程订阅逻辑复制槽并从主服务器应用三元组更改。

---

### `pg_ripple.prov_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.58.0 |

当为 `on` 时，为所有引入操作发出 PROV-O 来源三元组，将每个三元组链接到其 `prov:Activity`、`prov:Agent` 和时间戳。

---

### `pg_ripple.citus_sharding_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `postmaster` |
| 自 | v0.58.0 |

启用 VP 表的 Citus 水平分片。当为 `on` 时，使用 `REPLICA IDENTITY FULL` 创建新的 VP 表并通过 `create_distributed_table(s)` 分发。

---

### `pg_ripple.citus_trickle_compat`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.58.0 |

当为 `on` 时，`create_distributed_table` 对传统 CDC/IVM 兼容性使用 `colocate_with = 'none'`。防止跨分片墓碑删除。

---

### `pg_ripple.approx_distinct`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.68.0 |

当为 `on` 时，在 `hll` 扩展可用时，通过 Citus HLL（`hll_add_agg`）路由 `COUNT(DISTINCT …)` 聚合。在分布式 VP 表上提供近似但高度可扩展的不同计数。

---

### `pg_ripple.citus_service_pruning`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.68.0 |

当为 `on` 时，SPARQL 联邦翻译器重写面向 Citus 工作线程的 SERVICE 子查询以包括分片约束注释，从而修剪不相关的分片。

---

### `pg_ripple.columnar_threshold`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `-1` (已禁用) |
| 上下文 | `sighup` |
| 自 | v0.57.0 |

HTAP 合并将 `vp_{id}_main` 从堆转换为列式存储（通过 `pg_columnar`）的三元组计数阈值。`-1` 禁用列式转换。

---

### `pg_ripple.adaptive_indexing_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.57.0 |

基于查询访问模式启用自动自适应索引创建。当为 `on` 时，规划器为经常查询的谓词组合在 VP 表上创建其他索引。

---

### `pg_ripple.arrow_flight_secret`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无 — 票据未签名)* |
| 上下文 | `sighup` |
| 访问 | 仅超级用户 |
| 自 | v0.66.0 |

用于签署 Arrow Flight 导出票据的 HMAC-SHA256 密钥。在生产中，设置为长随机值。空字符串 = 未签名的票据（默认情况下被 `pg_ripple_http` 拒绝）。

```sql
ALTER SYSTEM SET pg_ripple.arrow_flight_secret = 'your-long-random-secret-here';
SELECT pg_reload_conf();
```

---

### `pg_ripple.arrow_flight_expiry_secs`

| | |
|---|---|
| 类型 | Integer (秒) |
| 默认值 | `3600` |
| 上下文 | `sighup` |
| 自 | v0.66.0 |

Arrow Flight 票据的有效期。超过这么多秒的票据被拒绝。

---

### `pg_ripple.arrow_unsigned_tickets_allowed`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 访问 | 仅超级用户 |
| 自 | v0.67.0 |

当为 `on` 时，接受未签名的 Arrow Flight 票据。仅用于本地开发 — 切勿在生产中启用。

---

### `pg_ripple.merge_lock_timeout_ms`

| | |
|---|---|
| 类型 | Integer (毫秒) |
| 默认值 | `5000` |
| 上下文 | `sighup` |
| 自 | v0.82.0 |

合并工作线程的围栏锁超时。如果在此间隔内未获得锁，则合并周期被跳过。

---

### `pg_ripple.merge_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000000` |
| 范围 | `100`–`100000000` |
| 上下文 | `sighup` |
| 自 | v0.82.0 |

单个合并 `INSERT…SELECT` 批次中处理的最大行数。向下调整以减少事务持续时间，代价是更多合并传递。

---

### `pg_ripple.describe_max_depth`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `16` |
| 范围 | `1`–`256` |
| 上下文 | `userset` |
| 自 | v0.85.0 |

`DESCRIBE` CBD 遍历的最大递归深度。防止循环或深度嵌套图上的失控递归。

---

### `pg_ripple.bulk_load_use_copy`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.94.0 |

当为 `on` 时，批量加载器对字典编码的三元组流插入使用 `COPY ... FROM STDIN BINARY`，而不是批量 INSERT。可以提高非常大加载的吞吐量。

---

### `pg_ripple.cdc_watermark_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 上下文 | `sighup` |
| 自 | v0.91.0 |

在刷新 LSN 水位之前累积的 CDC 事件数。减少每个事件的写放大。

---

### `pg_ripple.bidi_relay_max_inflight`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1000` |
| 范围 | `1`–`100000` |
| 上下文 | `userset` |
| 自 | v0.94.0 |

每个进程的最大并发进行中双向中继操作数。当达到限制时，新的中继分派调用将被丢弃（丢弃最旧），并且 `pg_ripple_bidi_relay_dropped_total` 增加。

---

## 联邦参数

### `pg_ripple.federation_timeout`

| | |
|---|---|
| 类型 | Integer (秒) |
| 默认值 | `30` |
| 上下文 | `userset` |

每个 SERVICE 调用的挂钟超时。在此间隔内不响应的端点返回空结果（带有 PT214 WARNING）。

---

### `pg_ripple.federation_max_results`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `10000` |
| 上下文 | `userset` |

从单个远程 SERVICE 调用接受的最大行数。

---

### `pg_ripple.federation_on_error`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'empty'` |
| 有效值 | `empty`、`error` |
| 上下文 | `userset` |

当 SERVICE 调用完全失败时的行为。`empty` 返回空结果集；`error` 引发异常。

---

### `pg_ripple.federation_pool_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `4` |
| 上下文 | `sighup` |
| 自 | v0.19.0 |

每个远程联邦端点保留的空闲 HTTP 连接数（连接池）。

---

### `pg_ripple.federation_cache_ttl`

| | |
|---|---|
| 类型 | Integer (秒) |
| 默认值 | `0` (已禁用) |
| 上下文 | `userset` |
| 自 | v0.19.0 |

缓存 SERVICE 结果的 TTL。当为 `0` 时，联邦结果不会被缓存。设置为正值以为幂等查询启用结果缓存。

---

### `pg_ripple.federation_adaptive_timeout`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.19.0 |

当为 `on` 时，根据观察到的 P95 延迟导出每个端点的有效超时，从而适应端点响应能力。

---

### `pg_ripple.federation_endpoint_policy`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'default-deny'` |
| 有效值 | `default-deny`、`allowlist`、`open` |
| 上下文 | `sighup` |
| 自 | v0.55.0 |

联邦端点的网络安全策略。

| 值 | 行为 |
|---|---|
| `default-deny` | 阻止 RFC-1918 私有、loopback 和 link-local 地址（SSRF 保护） |
| `allowlist` | 仅允许在 `federation_allowed_endpoints` 中列出的端点 |
| `open` | 允许所有端点 — 仅用于开发，切勿在生产中使用 |

---

### `pg_ripple.federation_allowed_endpoints`

| | |
|---|---|
| 类型 | String (逗号分隔的 URL) |
| 默认值 | *(无)* |
| 上下文 | `sighup` |
| 自 | v0.55.0 |

允许的联邦端点 URL 的逗号分隔列表。仅当 `federation_endpoint_policy = 'allowlist'` 时才查询。

---

### `pg_ripple.federation_circuit_breaker_threshold`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `5` |
| 上下文 | `userset` |
| 自 | v0.56.0 |

在为远程端点打开断路器之前的连续失败次数。一旦触发，端点将被绕过 `federation_circuit_breaker_reset_seconds` 秒。

---

### `pg_ripple.federation_circuit_breaker_reset_seconds`

| | |
|---|---|
| 类型 | Integer (秒) |
| 默认值 | `60` |
| 上下文 | `userset` |
| 自 | v0.56.0 |

直到触发的断路器半开并允许重试探测的秒数。

---

### `pg_ripple.federation_connect_timeout_secs`

| | |
|---|---|
| 类型 | Integer (秒) |
| 默认值 | `10` |
| 上下文 | `userset` |
| 自 | v0.96.0 |

联邦 SERVICE 端点的 TCP/TLS 连接超时。与 `federation_timeout`（涵盖查询主体）分开。如果端点未在此窗口内接受连接，则立即拒绝请求。

---

### `pg_ripple.federation_allow_unregistered_service_endpoints`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.98.0 |

当为 `off`（默认值）时，针对未在 `pg_ripple.federation_endpoints` 中注册的端点执行 SERVICE 子句会引发 PT-SSRF-01。仅用于开发或测试时设置为 `on`。

---

## 可观测性参数

### `pg_ripple.tracing_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.40.0 |

OpenTelemetry 分布式跟踪的主开关。当为 `on` 时，SPARQL 和 Datalog 查询执行将 span 发出到配置的导出器。

---

### `pg_ripple.tracing_exporter`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'stdout'` |
| 有效值 | `stdout`、`otlp` |
| 上下文 | `sighup` |
| 自 | v0.40.0 |

OpenTelemetry 导出器后端。使用 `tracing_otlp_endpoint` 的 `otlp` 将 span 发送到 OTLP 收集器（Jaeger、Tempo 等）。

---

### `pg_ripple.tracing_otlp_endpoint`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |
| 上下文 | `sighup` |
| 自 | v0.51.0 |

用于 span 导出的 OTLP 收集器端点 URL。示例：`http://localhost:4317`。

---

### `pg_ripple.audit_log_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.56.0 |

启用 SPARQL 写操作审计日志记录到 `_pg_ripple.audit_log`。记录所有 INSERT/DELETE/UPDATE 操作以及时间戳、查询文本和角色。

---

### `pg_ripple.audit_retention_days`

| | |
|---|---|
| 类型 | Integer (天) |
| 默认值 | `90` |
| 上下文 | `sighup` |
| 自 | v0.78.0 |

`_pg_ripple.event_audit` 行的保留期。后台工作线程扫描每小时修剪超过这么多天的行一次。设置为 `0` 以禁用自动修剪。

---

### `pg_ripple.export_max_rows`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `0` (无限制) |
| 上下文 | `userset` |
| 自 | v0.40.0 |

导出函数（`export_turtle()`、`export_ntriples()`、`export_jsonld()`）返回的最大行数。截断的导出发出 PT642。

---

## PageRank 参数

### `pg_ripple.pagerank_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.88.0 |

Datalog 原生 PageRank 引擎的主开关。必须为 `on` 才能使用 `pagerank_run()`、`centrality_run()` 和相关函数。

---

### `pg_ripple.pagerank_damping`

| | |
|---|---|
| 类型 | Float |
| 默认值 | `0.85` |
| 范围 | `0.0`–`1.0` |
| 上下文 | `userset` |
| 自 | v0.88.0 |

PageRank 阻尼因子（传送概率 = 1 − 阻尼）。标准值为 0.85（与 Google 原始论文一致）。

---

### `pg_ripple.pagerank_max_iterations`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 上下文 | `userset` |
| 自 | v0.88.0 |

最大 PageRank 迭代次数。在达到收敛时提前停止。

---

### `pg_ripple.pagerank_convergence_delta`

| | |
|---|---|
| 类型 | Float |
| 默认值 | `0.0001` |
| 上下文 | `userset` |
| 自 | v0.88.0 |

PageRank 的收敛阈值。当所有节点的最大分数变化降至该值以下时，迭代停止。

---

### `pg_ripple.pagerank_convergence_norm`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'l1'` |
| 有效值 | `l1`、`l2`、`linf` |
| 上下文 | `userset` |
| 自 | v0.90.0 |

PageRank 迭代的收敛范数。`l1` 匹配 NetworkX；`l2` 匹配 igraph；`linf` 最保守。

---

### `pg_ripple.pagerank_partition`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.88.0 |

启用图分区并行 PageRank 计算。当为 `on` 时，分区数自动调整为 `min(num_cpus, count(named_graphs))`。设置 `off` 以使用单分区（调试）模式。

---

### `pg_ripple.pagerank_incremental`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.88.0 |

启用 pg-trickle 增量 K 跳刷新。当为 `on` 时，在每次写入后，仅重新评估已更改节点的 K 跳邻域，而不是完全重新计算。

---

### `pg_ripple.pagerank_khop_limit`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `30` |
| 上下文 | `userset` |
| 自 | v0.88.0 |

增量 PageRank 更新的最大 K 跳传播深度。

---

### `pg_ripple.pagerank_rules`

| | |
|---|---|
| 类型 | String (逗号分隔的 IRI) |
| 默认值 | *(空 — 所有对象值谓词)* |
| 上下文 | `userset` |
| 自 | v0.88.0 |

要包含在 PageRank 图中的边谓词的逗号分隔 IRI 列表。空字符串表示所有对象值谓词。

---

### `pg_ripple.pagerank_confidence_weighted`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.88.0 |

当为 `on` 时，在 PageRank 计算期间将边权重乘以 `_pg_ripple.confidence` 中的置信度分数。较高置信度的边对 PageRank 传播的贡献更大。

---

### `pg_ripple.pagerank_full_recompute_threshold`

| | |
|---|---|
| 类型 | Float |
| 默认值 | `0.01` |
| 范围 | `0.0`–`1.0` |
| 上下文 | `userset` |
| 自 | v0.90.0 |

触发完全重新计算而不是增量刷新的陈旧 `pagerank_scores` 行的比例。

---

### `pg_ripple.pagerank_max_seeds`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `1024` |
| 范围 | `1`–`1048576` |
| 上下文 | `userset` |
| 自 | v0.89.0 |

`pagerank_run(..., seed_iris)` 接受的最大种子 IRI 数。比这更长的数组引发 PT0411。

---

### `pg_ripple.pagerank_katz_alpha`

| | |
|---|---|
| 类型 | Float |
| 默认值 | `0.01` |
| 上下文 | `userset` |
| 自 | v0.89.0 |

通过 `centrality_run()` 进行 Katz 中心性计算的衰减因子。

---

## LLM / AI 参数

### `pg_ripple.llm_endpoint`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |
| 上下文 | `userset` |
| 自 | v0.49.0 |

用于通过 `sparql_from_nl()` 进行自然语言 → SPARQL 生成的 LLM API 基础 URL。必须是 OpenAI 兼容的聊天完成端点。

```sql
ALTER SYSTEM SET pg_ripple.llm_endpoint = 'https://api.openai.com/v1';
-- 或对于 Ollama：
ALTER SYSTEM SET pg_ripple.llm_endpoint = 'http://localhost:11434/v1';
```

---

### `pg_ripple.llm_model`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |
| 上下文 | `userset` |
| 自 | v0.49.0 |

用于 NL → SPARQL 生成的 LLM 模型标识符。示例：`'gpt-4o'`、`'llama3'`。

---

### `pg_ripple.llm_api_key_env`

| | |
|---|---|
| 类型 | String |
| 默认值 | *(无)* |
| 上下文 | `sighup` |
| 自 | v0.49.0 |

保存 LLM API 密钥的操作系统环境变量的名称。扩展在查询时从此环境变量读取密钥。

---

### `pg_ripple.llm_include_shapes`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `on` |
| 上下文 | `userset` |
| 自 | v0.49.0 |

当为 `on` 时，活动 SHACL 形状作为语义上下文包含在发送给 LLM 的提示中，用于 NL → SPARQL 生成。

---

### `pg_ripple.kge_enabled`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.57.0 |

启用知识图谱嵌入（KGE）后台工作线程。当为 `on` 时，工作线程在图结构上训练 TransE/RotatE 嵌入。

---

### `pg_ripple.kge_model`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'transe'` |
| 有效值 | `transe`、`rotate` |
| 上下文 | `sighup` |
| 自 | v0.57.0 |

知识图谱嵌入模型。`transe` 更快；`rotate` 处理关系组合更好。

---

### `pg_ripple.auto_embed`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `sighup` |
| 自 | v0.28.0 |

基于触发器的新字典条目自动嵌入的主开关。当为 `on` 时，新加载的实体被排队等待后台嵌入。

---

### `pg_ripple.embedding_batch_size`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 上下文 | `sighup` |
| 自 | v0.28.0 |

每个后台工作线程批次出队和嵌入的实体数。

---

### `pg_ripple.use_graph_context`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.28.0 |

当为 `on` 时，在将每个实体的 RDF 邻域传递给嵌入模型之前对其进行序列化。以更长的输入字符串为代价产生图上下文嵌入。

---

### `pg_ripple.vector_federation_timeout_ms`

| | |
|---|---|
| 类型 | Integer (毫秒) |
| 默认值 | `5000` |
| 上下文 | `userset` |
| 自 | v0.28.0 |

对外部向量服务端点调用的 HTTP 超时。

---

## SHACL 参数

### `pg_ripple.shacl_mode`

| | |
|---|---|
| 类型 | Enum string |
| 默认值 | `'off'` |
| 有效值 | `off`、`sync`、`async` |
| 上下文 | `userset` |

SHACL 验证模式。

| 值 | 行为 |
|---|---|
| `off` | SHACL 强制已禁用 — `validate()` 仍然按需工作 |
| `sync` | 在三元组插入期间内联捕获违规；违规的三元组被拒绝并显示 PT301 |
| `async` | 违规排队等待后台验证；插入不会被阻止 |

---

### `pg_ripple.shacl_rule_max_iterations`

| | |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 上下文 | `userset` |
| 自 | v0.79.0 |

每个验证周期的 `sh:SPARQLRule` 求值的最大不动点迭代。当规则相互触发时，防止无限循环。

---

### `pg_ripple.shacl_rule_cwb`

| | |
|---|---|
| 类型 | Boolean |
| 默认值 | `off` |
| 上下文 | `userset` |
| 自 | v0.79.0 |

当为 `on` 时，其目标图与现有 CONSTRUCT 回写管道匹配的 `sh:SPARQLRule` 规则被注册为 CWB 规则，而不是独立的 SPARQL 求值。

---

## 快速参考表

为方便起见，以下是按最常见调整需求分组的所有参数的摘要：

### 性能调优

| 参数 | 默认值 | 用途 |
|---|---|---|
| `dictionary_cache_size` | `65536` | 内存中编码/解码 LRU 缓存大小 |
| `merge_threshold` | `10000` | 合并工作线程触发之前的行数 |
| `merge_workers` | `1` | 并行合并后台工作线程 |
| `export_batch_size` | `10000` | SPARQL 游标批处理大小 |
| `wcoj_enabled` | `on` | 循环模式的最坏情况最优连接 |
| `bgp_reorder` | `on` | 按选择性的连接重排序 |
| `topn_pushdown` | `on` | `ORDER BY … LIMIT N` 优化 |
| `arrow_batch_size` | `1000` | Arrow IPC 记录批处理大小 |
| `auto_analyze` | `on` | 合并后 `ANALYZE` |

### 安全性

| 参数 | 默认值 | 用途 |
|---|---|---|
| `federation_endpoint_policy` | `default-deny` | SERVICE 子句的 SSRF 保护 |
| `federation_allow_private` | `off` | 阻止私有 IP 联邦目标 |
| `rls_bypass` | `off` | 超级用户图 RLS 绕过 |
| `arrow_flight_secret` | *(无)* | Arrow Flight 票据签名 |
| `arrow_unsigned_tickets_allowed` | `off` | 仅用于开发的未签名票据 |

### 推导调优

| 参数 | 默认值 | 用途 |
|---|---|---|
| `inference_mode` | `on_demand` | 何时运行推导 |
| `magic_sets` | `on` | 目标导向的需求求值 |
| `dred_enabled` | `on` | 增量删除重新派生 |
| `datalog_parallel_workers` | `4` | 并行层求值 |
| `owl_profile` | `RL` | OWL 2 规则集选择 |
| `wfs_max_iterations` | `100` | 良好基础语义上限 |

### JSON 映射回写（v0.128.0）

| 参数 | 默认值 | 用途 |
|---|---|---|
| `json_writeback_batch_size` | `100` | 每个后台合并工作线程周期排空的队列行 |

---

## `pg_ripple.json_writeback_batch_size`

控制后台合并工作线程每个周期处理多少 `_pg_ripple.json_writeback_queue` 行。

| 属性 | 值 |
|---|---|
| 类型 | Integer |
| 默认值 | `100` |
| 范围 | `0`–`10000` |
| 上下文 | `suset` |
| 自 | v0.128.0 |

将 `0` 设置为禁用自动后台排空（行将累积，直到手动处理或通过直接调用 `pg_ripple.writeback_json_row()` 处理）。

较高的值以更长的工作线程事务为代价提高回写吞吐量。默认值 100 适用于大多数工作负载。