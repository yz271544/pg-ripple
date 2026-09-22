# pg_ripple / pg_ripple_http 兼容性矩阵

`pg_ripple`（PostgreSQL 扩展）和 `pg_ripple_http`（独立 HTTP 配套服务）是独立进行版本控制的。本页记录哪些扩展版本与哪些 HTTP 配套版本兼容，以及该组合适用哪些保证。

## 版本控制策略

- **pg_ripple** 遵循与扩展功能和 PostgreSQL 目录更改相关的语义版本控制。
- **pg_ripple_http** 遵循其自己的版本系列（当前为 `0.x.y`），因为它是一个独立二进制文件，具有自己的发布节奏。HTTP 配套版本号跟踪它所测试的**最低** pg_ripple 扩展版本。

给定的 `pg_ripple_http` 版本与扩展版本范围 `[tested_with, next_major)` 兼容。如果已安装的扩展版本超出其已知兼容范围，则 HTTP 配套在启动时记录警告。

## 兼容性表

| pg_ripple_http 版本 | pg_ripple 扩展范围 | 注释 |
|------------------------|---------------------------|-------|
| 0.136.x | ≥ 0.135.0 | 已冻结 v1 查询契约的加固候选；外部审计证据单独跟踪 |
| 0.135.x | ≥ 0.135.0 | 类型化查询绑定、受治理的前缀模式和稳定的 v1 查询清单 |
| 0.133.x | ≥ 0.132.0 | 弹性健康 API 和恢复资格（v0.133.0）|
| 0.134.x | ≥ 0.134.0 | 性能、规模和真正的流式资格（v0.134.0）|
| 0.132.x | ≥ 0.131.0 | 一致性证据、功能和路由真实性检查、有界谓词扩展和键集分页导出 |
| 0.131.x | ≥ 0.130.0 | 默认安全的 HTTP 配置、中心路由授权、PostgreSQL TLS、出站策略和可信代理强制 |
| 0.130.x | ≥ 0.129.0 | 安装/迁移完整性：独立迁移图、schema 指纹和经过验证的 JSON 回写配置 API |
| 0.129.x | ≥ 0.128.0 | JSON 回写和变更完整性；异步队列覆盖和类型化直接回写 |
| 0.128.x | ≥ 0.127.0 | JSON 映射关系回写 API；破坏性更改：需要 v0.128 JSON 回写目录和队列 schema |
| 0.127.x | ≥ 0.126.0 | pg_tide 中继桥迁移；破坏性更改：通过 `tide.outbox_publish` 和 `relay_set_*_v2` API 使用命名 outbox 事件 |
| 0.126.x | ≥ 0.125.0 | 联邦凭证状态和时间快照 API；破坏性更改：需要 v0.126 HTTP 路由和凭证目录添加 |
| 0.125.x | ≥ 0.124.0 | 时间图快照和差异 API；添加新指标：v0.125 snapshot catalog |
| 0.124.x | ≥ 0.123.0 | 规则库可观察性和基准历史 API；破坏性更改：需要 v0.124 规则库和基准目录添加 |
| 0.123.x | ≥ 0.122.0 | A17 可观察性和文档：副本池 Prometheus 指标（OBS-M-01）、规则库流延迟/错误计数器（OBS-M-02）、`bench_workload_result()` SQL 包装器（ERG-L-01）|
| 0.122.x | ≥ 0.121.0 | A17 神模块分解和测试覆盖率闭合（H17-02）；WatDiv 正确性门控；v0.119.0–v0.120.0 功能的 pg_regress 覆盖率 |
| 0.121.x | ≥ 0.120.0 | A17 安全加固（H17-01 SSRF、SEC-M-03 CGNAT/多播、BUG-M-01 静默降级、OBS-L-01 变更日志）；规则库操作的变更日志 |
| 0.120.x | ≥ 0.119.0 | 功能 11：规则库联邦（通过 Arrow Flight 发布/订阅）；功能 12：只读副本路由（`?replica=ok`）；`compat_check()` JSON schema |
| 0.119.x | ≥ 0.118.0 | 功能 6：联邦断路器 Prometheus 指标；功能 7：`bench_workload()` BSBM/WatDiv/PageRank/PPRL 配置文件；Allen 的区间关系 |
| 0.118.x | ≥ 0.117.0 | 功能 4：诊断快照包（`/admin/diagnostic-snapshot`）；功能 5：租户配额强制；多租户知识图 |
| 0.117.x | ≥ 0.116.0 | 功能 3：PPRL Bloom 过滤器编码；神经符号实体解析；LLM 解释缓存；证明树生成 |
| 0.116.x | ≥ 0.115.0 | A16 补救弧线：ER 阶段延迟（M16-03）、owl:sameAs 断言计数器、贝叶斯传播延迟、冲突检测计数器；咨询生命周期策略（M16-21）|
| 0.115.x | ≥ 0.114.0 | A15 补救：`er_stage_duration_seconds` 直方图标签（M16-03）；PPRL/LLM/证明树可观察性；GNN 链接预测集成 |
| 0.114.x | ≥ 0.113.0 | A15 高/中：`bidi_relay_dropped_total` 计数器（H15-03）；合并/层/SHACL/CDC Prometheus 指标（M15-19）；WatDiv 100 模板套件 |
| 0.113.x | ≥ 0.112.0 | UNNEST-数组批量加载路径；Citus 对象分片修剪；直接 `COPY` 批量加载；每图 RLS；`explain_inference()` |
| 0.93.x | ≥ 0.92.0 | pg_tide 集成：`pg_tide_available()` SQL 函数；BIDI 中继注释更新到 pg_tide API |
| 0.92.x | ≥ 0.88.0 | A14 低抛光：PageRank 边界、阻尼指南、SERVICE SILENT TLS、SOURCE_DATE_EPOCH、一致性徽章 CI |
| 0.91.x | ≥ 0.88.0 | OBS-01 PageRank IVM Prometheus 指标；HTTP-03 中间件提取；HTTP-04 EXPLAIN 行估计；API-05 `explain_pagerank_json()`；CDC 水印批量加载 GUC |
| 0.90.x | ≥ 0.88.0 | A14 中等正确性/性能：PageRank 流式传输、收敛范数 GUC、咨询锁、模块拆分 |
| 0.89.x | ≥ 0.88.0 | A14 高补救：变更处理程序上的 `check_auth_write`、GUC 名称审计、速率限制 100 req/s 默认值 |
| 0.88.x | ≥ 0.87.0 | 添加 `pagerank_scores`、`centrality_scores`、`pagerank_dirty_edges`；需要 `COMPATIBLE_EXTENSION_MIN = "0.87.0"` |
| 0.87.x | ≥ 0.86.0 | 添加 `_pg_ripple.confidence`、`shacl_score_log`；概率 Datalog、模糊 SPARQL 过滤器、软 SHACL 评分 |
| 0.86.x | ≥ 0.85.0 | D13-01（v0.86.0）：SSE 流式传输、优雅关闭、新的 Prometheus 指标（O13-02）、Arrow 413 防护（S13-08）、CORS 计数器（S13-03）、结构化 JSON 日志（O13-04）|
| 0.85.x | ≥ 0.84.0 | COMPATIBLE_EXTENSION_MIN 提升到 0.84.0（S13-05）；严格兼容模式（PG_RIPPLE_HTTP_STRICT_COMPAT）；schema.rs/federation.rs 拆分；encode_batch GUC API |
| 0.84.x | ≥ 0.83.0 | 在 /openapi 提供的 OpenAPI 规范；每查询类型指标（METRICS-LABELS-01）；/health/ready 深度检查（HEALTH-DEEP-01）；HTTP 中的 SPARQL Update |
| 0.83.x | ≥ 0.82.0 | CDC outbox 桥触发器、JSON-LD 事件序列化器、词汇模板、pg-trickle 检测 |
| 0.82.x | ≥ 0.80.0 | SBOM（cargo-cyclonedx）、结构化错误主体（HTTP-ERR-01）、Arrow 票证 nonce 缓存（FLIGHT-NONCE-01）、每 IP 限制器 |
| 0.81.x | ≥ 0.80.0 | SHACL-SPARQL 约束、COPY rdf FROM、RAG 加固、CDC 生命周期事件、OpenAPI 规范 |
| 0.80.x | ≥ 0.79.0 | SQL 注入参数化（SQL-INJ-01）、/explorer 身份验证门控（EXPLORER-AUTH-01）|
| 0.76.x | ≥ 0.79.0 | COMPAT-MIN-01（v0.80.0）：需要 `sparql_update_cursor()`（v0.76.0）和 `feature_status()` wcoj/shacl_sparql 条目（v0.79.0）；`/explorer` 现在需要身份验证（EXPLORER-AUTH-01）|
| 0.75.x | ≥ 0.78.0 | 双向集成操作（v0.78.0）、SPARQL 订阅 SSE（v0.73.0）|
| 0.74.x | ≥ 0.77.0 | 双向集成原语（v0.77.0）、SPARQL 1.2 跟踪（v0.73.0）|
| 0.73.x | ≥ 0.76.0 | Fuzz 加固（v0.76.0）、CONTRIBUTING.md（v0.73.0）、Helm chart SHA 钉 |
| 0.16.x | ≥ 0.70.0 | 第一个具有 `Body::from_stream` Arrow Flight 的版本；兼容性检查添加（v0.71.0 COMPAT-01）|
| 0.15.x | 0.66.0 – 0.69.x | Arrow Flight 安全性（FLIGHT-SEC-02）、SPARQL 游标流式传输（STREAM-01）|
| 0.14.x | 0.63.0 – 0.65.x | CONSTRUCT 回写规则（v0.63.0）、Datalog REST API（v0.39.0）|
| 0.13.x | 0.57.0 – 0.62.x | OWL 2 EL/QL 配置文件、KGE 嵌入、可视化图浏览器 |
| 0.12.x | 0.51.0 – 0.56.x | 非 root 容器、HTTP 流式传输、OTLP 跟踪 |
| 0.11.x | 0.40.0 – 0.50.x | SPARQL 游标、解释/可观察性、OpenTelemetry |
| 0.10.x | 0.38.0 – 0.39.x | 模块重构、所有 27 个 Datalog SQL 函数 |
| 0.9.x | 0.33.0 – 0.37.x | 文档站点重建、并行 Datalog、HTAP 稳定性 |
| ≤ 0.8.x | 0.15.0 – 0.32.x | HTTP 端点、批量加载、基本 SPARQL |


## 启动版本检查

从 `pg_ripple_http` 0.16.0 开始，HTTP 配套服务在启动时执行兼容性检查。它查询已安装的扩展版本并将其与其已知兼容范围进行比较。如果扩展早于最低支持版本：

- **记录警告**：配套服务启动但记录明显的警告。
- `GET /ready` 端点返回 HTTP 503 以及 `{"compatible": false, ...}`，如果扩展低于硬最低版本条件。

可以使用 `PG_RIPPLE_HTTP_SKIP_COMPAT_CHECK=1` 禁用此检查，前提是有意将较旧的扩展与较新的配套配对。

> **⚠ 生产警告（COMPAT-DOC-01 / MF-R）：**
> `PG_RIPPLE_HTTP_SKIP_COMPAT_CHECK=1` 仅**用于测试和开发**，在这些场景中您**有意**需要运行不匹配的配对（例如，针对较旧扩展的集成测试）。**不要在生产环境中设置此选项。**跳过检查允许 HTTP 配套服务为不兼容的扩展提供服务，这可能导致静默数据损坏、意外错误或安全漏洞，当针对较旧扩展使用新的 SQL 函数时。如果您需要在生产中使兼容性警告静音，请升级扩展或配套服务到兼容的版本配对。

## 独立版本控制的基本原理

HTTP 配套服务作为预构建的二进制文件分发。扩展升级（`ALTER EXTENSION pg_ripple UPDATE`）在数据库内应用，不需要重新构建或重新部署配套服务。这意味着：

1. 单个 `pg_ripple_http` 二进制可以在其兼容范围内为多个扩展版本提供服务。
2. 仅扩展的更改（新 SQL 函数、GUC、性能改进）不需要配套更新。
3. 破坏性 API 更改（新必需请求字段、移除的端点）确实需要配套更新。

## 升级过程

1. 首先升级扩展：`ALTER EXTENSION pg_ripple UPDATE TO 'X.Y.Z';`
2. 如果还需要配套升级，请重新启动或重新部署 `pg_ripple_http`。
3. 通过 `GET /ready` 验证兼容性 — 正确配对时返回 `{"compatible": true}`。

另请参阅：[Arrow Flight 参考](../reference/arrow-flight.md)、[HTTP API](../reference/http-api.md)。

## GA 之前升级支持策略

在 v1.0.0 之前，CI 运行完整的、历史的顺序迁移链。发布资格还测试从最近的六个已发布预 GA 版本工件的直接升级。在 v1.0.0 之后，每个次要版本支持从两个先前次要版本的直接升级，以及从仍保留在迁移图中的较旧版本的顺序升级。

## pg_tide / pg_trickle 扩展兼容性

pg_ripple 与两个配套 PostgreSQL 扩展集成：

| 扩展 | 目的 | 需要用于 |
|-----------|---------|--------------|
| pg_trickle ≥ 0.46.0 | 增量物化视图维护（仅 IVM）| SPARQL 视图、Datalog 视图、CONSTRUCT/DESCRIBE/ASK 视图、ExtVP |
| pg_tide ≥ 0.33.0 | 中继、outbox 和 inbox 子系统 | 双向中继（BIDI-OUTBOX-01、BIDI-INBOX-01）、中心辐射型集成 |

> **架构更改（pg_ripple v0.93.0 / pg_trickle v0.46.0）**：pg_trickle v0.46.0 将中继、outbox 和 inbox 子系统（约 6,150 行 Rust LOC + 约 2,500 行 SQL LOC）提取到新的独立 `pg_tide` 扩展（`trickle-labs/pg-tide`）。从 v0.93.0 开始，pg_ripple 将 pg_tide 识别为推荐的中继传输层。

### pg_ripple + pg_trickle 兼容性

| pg_ripple 版本 | pg_trickle 版本 | IVM 状态 |
|-------------------|--------------------|------------|
| ≥ 0.112.0 | ≥ 0.57.0 | ✅ 仅 IVM（pg_tide 中的中继功能）|
| ≥ 0.93.0 | ≥ 0.46.0 | ✅ 仅 IVM（pg_tide 中的中继功能）|
| ≥ 0.93.0 | ≤ 0.45.0 | ⚠ IVM 工作；中继功能需要手动迁移到 pg_tide |
| 0.52.0 – 0.92.0 | 任何 | ✅ 完整中继+IVM（提取前）|

### pg_ripple + pg_tide 兼容性

| pg_ripple 版本 | pg_tide 版本 | 中继状态 |
|-------------------|-----------------|--------------|
| ≥ 0.127.0 | ≥ 0.33.0 | ✅ 通过 `tide.outbox_publish()` 和稳定的 `relay_set_*_v2` 配置 API 进行命名 outbox 发布，完整中继支持 |
| 0.93.0 – 0.126.x | ≥ 0.4.0 | ✅ 中继 API 检测和文档；CDC 桥用户应升级到 0.127.0 以获得当前的命名 outbox 发布语义 |
| ≥ 0.93.0 | 0.1.0 – 0.3.x | ✅ 核心中继支持（tide.* API，较旧的功能集）|
| ≥ 0.93.0 | 未安装 | ⚠ 核心 pg_ripple + IVM 工作；双向中继不可用 |
| < 0.93.0 | 任何 | 不支持 pg_tide（对中继使用 pg_trickle ≤ 0.45.0）|

### 推荐堆栈（pg_ripple ≥ 0.127.0）

```sql
CREATE EXTENSION pg_tide;      -- 中继、outbox、inbox（trickle-labs/pg-tide ≥ 0.33.0）
CREATE EXTENSION pg_trickle;   -- IVM（trickle-labs/pg-trickle ≥ 0.46.0）
CREATE EXTENSION pg_ripple;    -- RDF 三元组存储（≥ 0.127.0）
```

调用 `pg_ripple.pg_tide_available()` 验证运行时是否安装了 pg_tide。
调用 `pg_ripple.pg_trickle_available()` 验证运行时是否安装了 pg_trickle。