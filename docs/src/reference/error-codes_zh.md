# 错误代码注册表

> **A13-03 (v0.86.0)**：此页面是 pg_ripple 中使用的所有 `PT` 错误代码的权威注册表。每条生产错误路径**必须**引用此列表中的代码。CI 在生产错误路径上强制使用 `PT` 代码。

pg_ripple 使用 **PT001–PT799**（扩展）和 **PT400–PT503**（HTTP 配套）范围内的结构化错误代码。错误消息遵循 PostgreSQL 约定：首单词小写，无尾随句点。

另请参阅：[错误消息目录](error-catalog.md) 按子系统查看完整的错误消息列表。

---

## 代码范围

| 范围 | 子系统 |
|---|---|
| PT001–PT099 | 字典编码 |
| PT100–PT199 | VP 存储 |
| PT200–PT299 | SPARQL 查询引擎 |
| PT300–PT399 | Datalog 推导 |
| PT400–PT499 | 输入验证 / HTTP |
| PT500–PT599 | 内部执行 |
| PT600–PT699 | SHACL 验证 |
| PT700–PT799 | 外部服务（LLM、联邦） |

---

## 不确定性知识（v0.87.0）— PT0301–PT0307

| 代码 | 功能 | 条件 |
|---|---|---|
| PT0301 | 不确定性知识 | 模糊 SPARQL 输入超过 `pg_ripple.fuzzy_max_input_length` 个字符 |
| PT0302 | 不确定性知识 | 未安装 `pg_trgm` 扩展 |
| PT0303 | 不确定性知识 | 无效的置信度值（NaN、Inf 或超出 `[0.0, 1.0]`） |
| PT0304 | 不确定性知识 | `pg:confidence()` 在三个参数都未绑定时被调用 |
| PT0305 | 不确定性知识 | `pg:confidence()` 在 `SERVICE` 子句内使用（不支持） |
| PT0306 | 不确定性知识 | SHACL 分数日志表 `_pg_ripple.shacl_score_log` 不存在 |
| PT0307 | 不确定性知识 | 置信度批量加载器：文件路径超出允许的目录 |

---

## PageRank（v0.88.0）— PT0401–PT0423

| 代码 | 功能 | 条件 |
|---|---|---|
| PT0401 | PageRank | 阻尼因子无效 — 必须在 `(0, 1)` 开区间内 |
| PT0402 | PageRank | `max_iterations` 必须是正整数 |
| PT0403 | PageRank | `topic` 标签超过 `pg_ripple.pagerank_max_topic_length` 个字符 |
| PT0404 | PageRank | `seed_nodes` 列表超过 `pg_ripple.pagerank_max_seeds` 限制 |
| PT0405 | PageRank | `edge_predicates` 列表为空（至少需要一个） |
| PT0406 | PageRank | `convergence_threshold` 必须为正且有限 |
| PT0407 | PageRank | 未找到 pagerank 分数 — 首先运行 `pagerank_run()` |
| PT0408 | PageRank | `pagerank_scores` 表不存在 — 首先运行 `pagerank_run()` |
| PT0409 | PageRank | `pagerank_dirty_edges` 表不存在 |
| PT0410 | PageRank | 导出格式不受支持 — 有效值：`turtle`、`jsonld`、`csv`、`ntriples` |
| PT0411 | PageRank | 中心性指标不受支持 — 有效值：`betweenness`、`closeness`、`eigenvector`、`katz` |
| PT0412 | PageRank | `katz_beta`（衰减因子）必须为正 |
| PT0413 | PageRank | `explain_pagerank` `top_k` 必须是正整数 |
| PT0414 | PageRank | PageRank 运行中止 — 在 `max_iterations` 内未达到收敛 |
| PT0415 | PageRank | 此主题的并发 PageRank 运行已在进行中 |
| PT0416 | PageRank | 导出中的 IRI 转义错误（`pagerank_scores` 中格式错误的 IRI） |
| PT0417 | PageRank | SPARQL 查询中的 `pg:pagerank()` 触发按需运行；超时 |
| PT0418 | PageRank | 中介中心性至少需要 3 个节点 |
| PT0419 | PageRank | `pagerank_partition` 值超出范围 `[1, 1024]` |
| PT0420 | PageRank | 增量刷新的 `k_hop_depth` 超出范围 `[1, 20]` |
| PT0421 | PageRank | 请求了置信度加权 PageRank 但不存在置信度分数 |
| PT0422 | PageRank | 时间衰减 `half_life_days` 必须为正 |
| PT0423 | PageRank | `federation_minimum_confidence` 超出 `[0.0, 1.0]` |

---

## HTTP 配套 PT 代码

| 代码 | HTTP 状态 | 含义 | 来源 |
|---|---|---|---|
| PT400 | 400 | 查询参数缺失或格式错误 | `routing/sparql_handlers.rs` |
| PT400_SPARQL_PARSE | 400 | SPARQL 语法错误 — 解析失败 | `routing/sparql_handlers.rs`、`spi_bridge.rs` |
| PT401 | 401 | 未授权 — Bearer 令牌缺失或无效 | `common.rs` |
| PT403 | 403 | 禁止 — 路径超出允许的目录 | `bulk_load.rs` |
| PT404 | 413 | 请求主体超过最大允许大小 | `routing/sparql_handlers.rs` |
| PT413 | 413 | Arrow Flight 导出结果过大 | `arrow_encode.rs` |
| PT503 | 503 | 数据库连接不可用 | `common.rs`、`stream.rs` |

---

## 扩展 PT 代码（精选）

| 代码 | 消息 | 子系统 |
|---|---|---|
| PT001 | dictionary encode failed: hash collision detected | 字典 |
| PT002 | dictionary decode failed: id not found | 字典 |
| PT003 | invalid term kind: expected 0/1/2 | 字典 |
| PT008 | malformed IRI: `<detail>` | 字典 |
| PT400 | SPARQL parse error: `<detail>` | SPARQL |
| PT403 | file path outside allowed directory | 批量加载 |
| PT501 | deprecated GUC: use `<replacement>` | 存储 GUC |
| PT512 | strict_dictionary: unknown dictionary id | 字典 |
| PT600 | SHACL constraint violation: `<detail>` | SHACL |
| PT700 | LLM endpoint unreachable or returned HTTP error | LLM |

---

## CI 强制

CI 任务 `check-pt-codes`（在 v0.86.0 中添加）扫描生产代码中的所有 `pgrx::error!` 和 `tracing::error!` 调用点，以验证每个调用点引用以下位置的 PT 代码：

- 错误消息正文中（例如，`"... (PT400)"`），或
- 错误代码参数中（例如，`json_error("PT400", ..., StatusCode::BAD_REQUEST)`）。

使用 `"internal: <description> — please report"` 格式的纯内部错误被豁免。

要添加新的错误代码，请**首先**更新此文件，然后在实现中引用该代码。