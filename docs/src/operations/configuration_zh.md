# 配置和调优

pg_ripple 通过 PostgreSQL GUC（Grand Unified Configuration）参数公开其配置。所有参数使用 `pg_ripple.` 前缀，可以在 `postgresql.conf` 中、通过 `ALTER SYSTEM` 或每个会话使用 `SET` 进行设置。

```admonish info title="重新启动要求"
标记为 **Postmaster** 的参数需要重新启动 PostgreSQL。标记为 **SIGHUP** 的参数可以使用 `SELECT pg_reload_conf()` 重新加载。所有其他参数都可以使用 `SET` 进行每个会话的更改。
```

---

## 存储参数

控制三元组在 VP 表和稀有谓词合并表中的存储方式。

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `vp_promotion_threshold` | int | `1000` | 10 – 10,000,000 | Userset | 谓词获得专用 VP 表之前的最小三元组数。低于此值，三元组将进入 `vp_rare`。|
| `named_graph_optimized` | bool | `off` | — | Userset | 为每个 VP 表添加 `(g, s, o)` 索引。加速 `GRAPH` 查询，但增加写入开销。|
| `default_graph` | text | `''` | 任何 IRI | Userset | 插入时未指定 `g` 时用作默认图的 IRI。|
| `dedup_on_merge` | bool | `off` | — | Userset | 当开启时，合并工作线程对 `(s, o, g)` 行进行去重，保留最低的 SID。|

---

## HTAP / 合并工作线程参数

控制 delta/main 分割和后台合并行为。这些仅在通过 `shared_preload_libraries` 加载 pg_ripple 时生效。

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `merge_threshold` | int | `10000` | 1 – 2,147,483,647 | SIGHUP | 触发谓词合并的 Delta 行数。较低 = 读取更新鲜，更多 I/O。|
| `merge_interval_secs` | int | `60` | 1 – 3600 | SIGHUP | 合并工作线程轮询周期之间的最大秒数。|
| `merge_retention_seconds` | int | `60` | 0 – 86,400 | SIGHUP | 合并后保留旧主表的秒数，然后删除。|
| `latch_trigger_threshold` | int | `10000` | 1 – 2,147,483,643 | SIGHUP | 立即触发合并工作线程锁存器之前批处理中写入的行数。|
| `merge_watchdog_timeout` | int | `300` | 10 – 86,400 | SIGHUP | 记录 WARNING 之前合并工作线程不活动的秒数。|
| `worker_database` | text | `'postgres'` | — | SIGHUP | 后台合并工作线程连接的数据库。|
| `auto_analyze` | bool | `on` | — | SIGHUP | 在每个合并周期后对 VP 主表运行 ANALYZE。|

---

## 查询引擎参数

调整 SPARQL 到 SQL 转换和执行。

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `plan_cache_size` | int | `256` | 0 – 65,536 | Userset | 每个后端缓存的 SPARQL→SQL 翻译数。0 禁用缓存。|
| `max_path_depth` | int | `100` | 0 – 10,000 | Userset | 属性路径查询（`+`、`*`）的最大递归深度。0 = 无限制。|
| `property_path_max_depth` | int | `64` | 1 – 100,000 | Userset | 备用属性路径深度限制（v0.24.0）。|
| `describe_strategy` | text | `'cbd'` | `'cbd'`、`'sd'`、`'simple'` | Userset | DESCRIBE 算法：Concise Bounded Description、Symmetric CBD 或简单一跳。|
| `bgp_reorder` | bool | `on` | — | Userset | 在 SQL 生成之前按估计选择性重新排序 BGP 三元组模式。|
| `parallel_query_min_joins` | int | `3` | 1 – 100 | Userset | 启用并行查询工作线程之前的最小 VP 表连接数。|
| `sparql_strict` | bool | `on` | — | Userset | 当开启时，不支持的 FILTER 函数会引发错误；当关闭时，它们会被静默删除。|
| `export_batch_size` | int | `10000` | 100 – 1,000,000 | Userset | 流式导出期间每个游标批次的三元组数。|

---

## 推理 / Datalog 参数

控制 Datalog 推理引擎、magic sets 和规则缓存。

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `inference_mode` | text | `'off'` | `'off'`、`'on_demand'`、`'materialized'` | Userset | Datalog 推理模式。`'materialized'` 需要 pg_trickle。|
| `enforce_constraints` | text | `'off'` | `'off'`、`'warn'`、`'error'` | Userset | Datalog 约束规则检测到违规时的行为。|
| `rule_graph_scope` | text | `'default'` | `'default'`、`'all'` | Userset | 未限定范围的规则原子是对默认图还是所有图操作。|
| `magic_sets` | bool | `on` | — | Userset | 对 `infer_goal()` 中的目标导向推理使用 magic sets。|
| `datalog_cost_reorder` | bool | `on` | — | Userset | 在 SQL 编译之前按升序 VP 表基数对规则体原子进行排序。|
| `datalog_antijoin_threshold` | int | `1000` | 0 – 10,000,000 | Userset | NOT 原子使用 LEFT JOIN 反连接形式的最小 VP 表基数。|
| `delta_index_threshold` | int | `500` | 0 – 10,000,000 | Userset | 在创建 B-tree 索引之前的最小半朴素 delta 行数。|
| `demand_transform` | bool | `on` | — | Userset | 在指定多个目标模式时自动应用需求转换。|
| `sameas_reasoning` | bool | `on` | — | Userset | 在推理期间应用 `owl:sameAs` 规范化预处理。|
| `rule_plan_cache` | bool | `on` | — | Userset | 为每个规则集缓存已编译的 SQL。由 `drop_rules()` 和 `load_rules()` 使其失效。|
| `rule_plan_cache_size` | int | `64` | 1 – 4,096 | Userset | 计划缓存中的最大规则集数。|

---

## 良基语义 / Tabling 参数

控制 WFS 求值和 tabling 缓存（v0.32.0）。

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `wfs_max_iterations` | int | `100` | 1 – 10,000 | Userset | 每个 WFS 传递的交替不动点轮次的安全上限。如果未收敛则发出 PT520 WARNING。|
| `tabling` | bool | `on` | — | Userset | 在 `_pg_ripple.tabling_cache` 中缓存 `infer_wfs()` 和 SPARQL 结果。|
| `tabling_ttl` | int | `300` | 0 – 86,400 | Userset | tabling 缓存条目的 TTL（秒）。0 禁用基于 TTL 的过期。|

---

## SHACL 验证参数

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `shacl_mode` | text | `'off'` | `'off'`、`'sync'`、`'async'` | Userset | `'sync'` 内联拒绝违规；`'async'` 排队等待后台验证。|

---

## 联邦参数

通过 `SERVICE` 关键字控制远程 SPARQL 端点调用。

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `federation_timeout` | int | `30` | 1 – 3,600 | Userset | 每个 SERVICE 调用的挂钟超时（秒）。|
| `federation_max_results` | int | `10000` | 1 – 1,000,000 | Userset | 单个远程调用接受的最大行数。|
| `federation_on_error` | text | `'warning'` | `'warning'`、`'error'`、`'empty'` | Userset | SERVICE 调用失败时的行为。|
| `federation_pool_size` | int | `4` | 1 – 32 | Userset | 每个端点主机的空闲 HTTP 连接数。|
| `federation_cache_ttl` | int | `0` | 0 – 86,400 | Userset | 远程结果缓存 TTL（秒）。0 禁用缓存。|
| `federation_on_partial` | text | `'empty'` | `'empty'`、`'use'` | Userset | SERVICE 调用中途失败时的行为。|
| `federation_adaptive_timeout` | bool | `off` | — | Userset | 从 P95 延迟派生每个端点超时。|

---

## 共享内存参数（仅启动）

这些必须在 PostgreSQL 启动之前在 `postgresql.conf` 中设置。它们不能在运行时更改。

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `dictionary_cache_size` | int | `4096` | 0 – 1,000,000 | Postmaster | 共享内存编码缓存容量（条目数）。|
| `cache_budget` | int | `64` | 0 – 65,536 | Postmaster | 共享内存预算上限（MB）。批量加载在 90% 利用率时进行限流。|

```admonish warning title="启动 GUC 需要重新启动"
对 `dictionary_cache_size` 和 `cache_budget` 的更改需要完全重新启动 PostgreSQL。在部署到生产环境之前规划您的缓存大小。
```

---

## 安全参数

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `rls_bypass` | bool | `off` | — | Suset | 超级用户覆盖以绕过图级行级安全性。|

---

## 向量 / 嵌入参数

| 参数 | 类型 | 默认值 | 范围 | 上下文 | 描述 |
|---|---|---|---|---|---|
| `embedding_model` | text | `''` | — | Userset | 存储在 `_pg_ripple.embeddings` 中的模型名称标签。|
| `embedding_dimensions` | int | `1536` | 1 – 16,000 | Userset | 向量维数。必须与模型输出匹配。|
| `embedding_api_url` | text | `''` | — | Userset | OpenAI 兼容嵌入 API 的基础 URL。|
| `embedding_api_key` | text | `''` | — | Suset | API 密钥（仅超级用户，在 `pg_settings` 中屏蔽）。|
| `pgvector_enabled` | bool | `on` | — | Userset | 在不卸载的情况下禁用 pgvector 代码路径。|
| `embedding_index_type` | text | `'hnsw'` | `'hnsw'`、`'ivfflat'` | Userset | 嵌入表上的索引类型。|
| `embedding_precision` | text | `'single'` | `'single'`、`'half'`、`'binary'` | Userset | 嵌入向量的存储精度。|
| `auto_embed` | bool | `off` | — | Userset | 通过后台工作线程自动嵌入新实体。|
| `embedding_batch_size` | int | `100` | 1 – 10,000 | Userset | 每个后台工作线程批次排队的实体数。|

---

## 快速入门配置

### 小型数据集（< 1M 三元组）

适用于开发、原型设计或小知识图谱：

```ini
# postgresql.conf
shared_preload_libraries = 'pg_ripple'

# 字典缓存 — 小占用空间
pg_ripple.dictionary_cache_size = 8192
pg_ripple.cache_budget = 16

# 合并工作线程 — 尽早合并以获得更新鲜的读取
pg_ripple.merge_threshold = 5000
pg_ripple.merge_interval_secs = 30

# 查询引擎
pg_ripple.plan_cache_size = 64
pg_ripple.max_path_depth = 50
```

### 中型数据集（1M – 100M 三元组）

具有中等查询复杂度的生产工作负载：

```ini
# postgresql.conf
shared_preload_libraries = 'pg_ripple'

# 字典缓存 — 更大的缓存以获得更好的命中率
pg_ripple.dictionary_cache_size = 131072
pg_ripple.cache_budget = 128

# 合并工作线程 — 平衡新鲜度和 I/O
pg_ripple.merge_threshold = 50000
pg_ripple.merge_interval_secs = 60
pg_ripple.latch_trigger_threshold = 20000
pg_ripple.auto_analyze = on

# 查询引擎 — 更大的计划缓存以支持多样化的查询
pg_ripple.plan_cache_size = 512
pg_ripple.max_path_depth = 100
pg_ripple.bgp_reorder = on

# 推理（如果使用）
pg_ripple.inference_mode = 'on_demand'
pg_ripple.magic_sets = on
```

### 大型数据集（> 100M 三元组）

高查询负载的高吞吐量生产环境：

```ini
# postgresql.conf
shared_preload_libraries = 'pg_ripple'

# 字典缓存 — 最大化缓存覆盖范围
pg_ripple.dictionary_cache_size = 500000
pg_ripple.cache_budget = 512

# 合并工作线程 — 批量更大的合并，减少抖动
pg_ripple.merge_threshold = 200000
pg_ripple.merge_interval_secs = 120
pg_ripple.latch_trigger_threshold = 100000
pg_ripple.merge_retention_seconds = 120
pg_ripple.auto_analyze = on

# 查询引擎 — 大计划缓存，多元查询
pg_ripple.plan_cache_size = 2048
pg_ripple.max_path_depth = 200
pg_ripple.bgp_reorder = on
pg_ripple.parallel_query_min_joins = 2

# 命名图优化（如果大量使用 GRAPH）
pg_ripple.named_graph_optimized = on

# 推理
pg_ripple.inference_mode = 'on_demand'
pg_ripple.magic_sets = on
pg_ripple.rule_plan_cache = on
pg_ripple.rule_plan_cache_size = 256

# 用于重复推理模式的 Tabling 缓存
pg_ripple.tabling = on
pg_ripple.tabling_ttl = 600

# 联邦（如果使用）
pg_ripple.federation_timeout = 60
pg_ripple.federation_pool_size = 8
pg_ripple.federation_cache_ttl = 300
```

```admonish tip title="PostgreSQL 调优"
不要忘记同时调整 PostgreSQL 本身。对于三元组存储工作负载的关键 PostgreSQL 参数：
- `shared_buffers` = RAM 的 25%
- `effective_cache_size` = RAM 的 75%
- `work_mem` = 64MB–256MB（用于复杂 JOIN）
- `maintenance_work_mem` = 512MB–1GB（用于合并 ANALYZE）
- `random_page_cost` = 1.1（如果使用 SSD）
- `max_parallel_workers_per_gather` = 4
```

## 不确定知识引擎 GUC（v0.87.0）

| GUC | 类型 | 默认值 | 描述 |
|-----|------|---------|-------------|
| `pg_ripple.probabilistic_datalog` | bool | `off` | 启用 @weight 规则注释 |
| `pg_ripple.prob_datalog_cyclic` | bool | `off` | 允许对循环规则集进行近似求值 |
| `pg_ripple.prob_datalog_max_iterations` | int | `100` | 最大半朴素迭代次数 |
| `pg_ripple.prob_datalog_convergence_delta` | float8 | `0.001` | 提前退出收敛阈值 |
| `pg_ripple.prob_datalog_cyclic_strict` | bool | `off` | 将不收敛提升为 ERROR（PT0307）|
| `pg_ripple.default_fuzzy_threshold` | float8 | `0.7` | 默认模糊匹配阈值 |
| `pg_ripple.prov_confidence` | bool | `off` | 启用 PROV-O pg:sourceTrust 传播 |
| `pg_ripple.export_confidence` | bool | `off` | 在 Turtle 导出中包含 RDF-star 注释 |
| `pg_ripple.cwb_confidence_propagation` | string | （空）| 用于 CWB 信任传播的 CONSTRUCT 规则名称 |