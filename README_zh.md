# pg-ripple

[![CI](https://github.com/trickle-labs/pg-ripple/actions/workflows/ci.yml/badge.svg)](https://github.com/trickle-labs/pg-ripple/actions/workflows/ci.yml)
[![Release](https://github.com/trickle-labs/pg-ripple/actions/workflows/release.yml/badge.svg)](https://github.com/trickle-labs/pg-ripple/actions/workflows/release.yml)
[![Roadmap](https://img.shields.io/badge/Roadmap-view-informational)](ROADMAP.md)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![PostgreSQL 18](https://img.shields.io/badge/PostgreSQL-18-blue?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![pgrx 0.18](https://img.shields.io/badge/pgrx-0.18-orange)](https://github.com/pgcentralfoundation/pgrx)
[![WatDiv correctness](https://img.shields.io/badge/WatDiv-100%25%20correct-brightgreen)](docs/src/reference/watdiv-results.md)
[![LUBM conformance](https://img.shields.io/badge/LUBM-14%2F14%20pass-brightgreen)](docs/src/reference/lubm-results.md)
[![Jena conformance](https://img.shields.io/badge/Jena-%E2%89%A595%25%20pass-brightgreen)](docs/src/reference/jena-results.md)
[![OWL 2 RL conformance](https://img.shields.io/badge/OWL%202%20RL-%E2%89%A595%25%20pass-brightgreen)](docs/src/reference/owl2rl-results.md)
[![PPRL](https://img.shields.io/badge/PPRL-CLK%20%2B%20DP-purple)](docs/src/cookbook/pprl.md)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/trickle-labs/pg-ripple)

**内置于 PostgreSQL 的知识图谱引擎。**

pg_ripple 是一个 PostgreSQL 18 扩展，它将你的数据库转变为知识图谱存储。你可以将数据建模为一个互联事实的网络——实体、关系和属性——然后在同一数据库中查询、验证和推理这些连接。

无需独立的图数据库。无需数据管道。无需额外的基础设施。

> **不了解知识图谱？** 可以把知识图谱想象成一种更智能、更互联的数据存储方式。不是存储表中的行，而是存储事实：*Alice 认识 Bob*，*Bob 在 Acme Corp 工作*，*Acme Corp 位于奥斯陆*。然后你可以提出跨越多跳的问题：*"Alice 的扩展职业网络中都有谁？"* —— 这类问题在 SQL 中很难处理，但在图中却很自然。

---

## 当前已实现的功能（v0.126.0）

pg_ripple 通过了 **100% 的 W3C SPARQL 1.1、SHACL Core 和 OWL 2 RL 合规测试套件** —— 这是知识图谱系统正确性的行业基准。经过 126 个版本的迭代，它涵盖了以下全部功能集。

| 你可以做什么 | 如何实现 |
|---|---|
| **导入知识** | 以标准格式加载数据：Turtle、N-Triples、N-Quads、TriG 或 RDF/XML —— 来自文件、内联文本或远程 URL。命名图让你将事实组织成逻辑组（例如每个数据源或主题一个图）。 |
| **使用 SPARQL 查询** | 使用 SPARQL 1.1 提出复杂问题 —— 这是链接数据的 W3C 标准查询语言（类似于 SQL，但为图设计）。追踪关系链、应用过滤器、聚合结果、跨多个图查询。完全符合 W3C 规范。可配置的 DoS 限制（`pg_ripple.sparql_max_algebra_depth`、`pg_ripple.sparql_max_triple_patterns`）在解析时拒绝格式错误的深度查询。 |
| **AI 和 LLM 集成** | 将向量嵌入与图事实一起存储。在一个查询中结合语义相似性搜索（*"查找与 X 相似的实体"*）和 SPARQL 图遍历。内置 RAG 管道为语言模型提示词检索图上下文化的上下文。使用 `sparql_construct_jsonld()` 配合 JSON-LD 帧，从 SPARQL CONSTRUCT 查询生成结构化的、token 高效的系统提示词。 |
| **Microsoft GraphRAG** | 以 GraphRAG 的 BYOG（Bring Your Own Graph）Parquet 格式导出实体和关系。用 Datalog 规则丰富图。用 SHACL 验证导出质量。通过单个 SQL 调用将知识图谱连接到微软的 GraphRAG 管道。 |
| **验证数据质量** | 用 SHACL 定义质量规则：*"每个人必须恰好有一个名字"*，*"年龄必须是正整数"*。违规在插入时被捕获（即时反馈）或在后台检查。完整的 SHACL Core 合规性，包括 `sh:equals`、`sh:disjoint` 和复杂的属性路径遍历（反向、选择、顺序、零或多、一或多）。违规报告包含解码后的 focus-node IRI，便于调试。 |
| **自动推理新事实** | 编写 Datalog 规则从已有知识推导出结论 —— *"如果 Alice 管理 Bob 且 Bob 管理 Carol，则 Alice 间接管理 Carol"*。包含对标准 RDFS 和 OWL 推理的内置支持。目标导向模式（`infer_goal()`）和需求过滤模式（`infer_demand()`）仅推导出与查询相关的事实，在大型程序上减少 50–90% 的推理工作。`owl:sameAs` 实体规范化在推理前自动应用，因此等价实体被视为一个。良基语义（`infer_wfs()`）处理具有相互否定的不可分层程序。表（tabling）缓存重复的推理子目标（2–5 倍加速）。并行地层评估并发运行独立的规则组。最坏情况最优连接加速循环图查询。增量删除（DRed）在删除后保持派生谓词一致，无需完全重算。 |
| **流式查询和检查** | 使用 `sparql_cursor()` 通过 PostgreSQL portal API 分页流式传输大型结果集 —— 峰值内存由 `pg_ripple.export_batch_size` 限制，而非整个结果集。通过 `sparql_csv()` / `sparql_tsv()` 导出 W3C CSV 或 TSV 格式的结果。使用 `explain_sparql()` 和 `explain_datalog()` 检查查询计划和规则编译。向 `explain_sparql()` 传递 `citus := true` 以获取 Citus 分片修剪部分，显示查询被修剪到哪个分片以及避免了多少行。`streaming_metrics()` 返回游标页面、Arrow 批处理和 ticket 拒绝的实时原子计数器。支持 OpenTelemetry 跨度追踪，配置 OTLP 端点（`pg_ripple.tracing_otlp_endpoint`）。 |
| **实时变更通知** | 通过 PostgreSQL NOTIFY 或服务器发送事件（SSE）订阅图变更。`create_subscription(name, filter_sparql)` 在匹配的三元组变更时在 `pg_ripple_cdc_{name}` 通道上触发 `pg_notify`。`subscribe_sparql(id, query, graph_iri)` 注册一个 SPARQL 查询订阅，在每次图写入后重新执行，并通过 `pg_ripple_http` 中的 `GET /subscribe/{id}` 将结果作为 SSE 流式传输。`unsubscribe_sparql(id)` 移除订阅；`list_sparql_subscriptions()` 枚举活动订阅。CDC 生命周期事件（`pg_ripple.cdc_lifecycle_events`）记录订阅活动。 |
| **导出和共享** | 将图导出为 Turtle、N-Triples、JSON-LD 或 RDF/XML。使用 JSON-LD 帧生成适合 REST API 或 LLM 提示词的嵌套文档。`export_jsonld_node(iri)` 返回给定主语的所有三元组作为 JSON-LD 文档。`json_ld_load(document, default_graph)` 一次调用摄取多图 JSON-LD 文档。`COPY rdf FROM` 通过 PostgreSQL 的 COPY 协议直接加载批量 RDF 文件。`pg_ripple_http` 通过 HMAC-SHA256 签名 ticket（含 nonce 重放保护）的 Arrow IPC 批量导出：通过 `POST /flight/do_get` 的二进制 IPC 流。 |
| **标准 HTTP 端点** | 配套的 `pg_ripple_http` 服务通过 HTTP/HTTPS 暴露 W3C SPARQL 协议端点。支持 JSON、XML、CSV、Turtle 和 JSON-LD 响应；身份验证；Prometheus 指标（`/metrics`）；扩展级指标 via `/metrics/extension`（三元组计数、活动图、GUC 设置）；Docker Compose 便于部署；完整的 OpenAPI 3.1 规范；以及 Arrow/Flight 批量导出端点。 |
| **查询远程图服务** | 使用 SPARQL `SERVICE` 关键字将外部 SPARQL 端点作为单个查询的一部分查询 —— 你本地数据和远程公共数据集在一个请求中。包含连接池、结果缓存、安全超时和断路器（`pg_ripple.federation_circuit_breaker_threshold`）来停止重试失败的端点。 |
| **使用 Citus 水平扩展** | 启用 `pg_ripple.citus_sharding_enabled` 将 VP 表分布到 Citus 工作节点。有界主语 SPARQL 模式自动被修剪到正确的分片（10–100 倍加速）。`citus_rebalance()` 发出 NOTIFY 信号，以便 pg-trickle 在重新平衡期间暂停 CDC。`citus_rebalance_progress()` 报告实时分片移动状态。 |
| **时序 RDF 查询** | `point_in_time(ts TIMESTAMPTZ)` 将当前会话中的所有 SPARQL 查询限制为在给定时间戳存在的三元组 —— 实现时间点查询、审计追踪和时序连接，无需模式更改。 |
| **PROV-O 数据溯源** | 启用 `pg_ripple.prov_enabled` 自动为每个批量加载操作记录 W3C PROV-O `prov:Activity` + `prov:Entity` 三元组。`prov_stats()` 汇总加载历史。 |
| **地理空间查询** | GeoSPARQL 1.1：通过 `geof:within`、`geof:intersects` 和 `geof:distance` 过滤；计算 `geof:buffer`、`geof:convexHull`、`geof:envelope`。几何值存储为 WKT 字面量，通过 PostGIS 处理。 |
| **OWL 2 EL/QL 推理配置文件** | 激活 `load_rules_builtin('owl-el')` 或 `load_rules_builtin('owl-ql')` 以使用特定配置文件的推理。OWL 2 QL 在翻译时重写 SPARQL BGP 用于 DL-Lite 本体。通过 `pg_ripple.owl_profile` 控制。 |
| **知识图谱嵌入** | 启用 `pg_ripple.kge_enabled` 训练存储在 `_pg_ripple.kge_embeddings` 中的 TransE 或 RotatE 实体嵌入，并带有 HNSW 索引。使用 `find_alignments()` 通过余弦相似性提出跨图 `owl:sameAs` 候选。 |
| **SPARQL 审计日志** | 启用 `pg_ripple.audit_log_enabled` 将所有 SPARQL UPDATE 操作（角色、事务 ID、查询文本）记录到 `_pg_ripple.audit_log`。`purge_audit_log(before)` 清理旧条目。 |
| **多租户图隔离** | `create_tenant()` 使用三元组计数配额注册命名图。触发器在插入时强制执行配额；`tenant_stats()` 报告每个租户的用量。 |
| **SPARQL-DL OWL 公理查询** | `sparql_dl_subclasses(IRI)` 和 `sparql_dl_superclasses(IRI)` 将 OWL 词汇 BGP（`owl:subClassOf`、`owl:equivalentClass`、`owl:disjointWith`）直接路由到 VP 表 T-Box 数据 —— 无需单独的索引。 |
| **SHACL-SPARQL 规则** | SHACL 高级特性：`sh:SPARQLRule` 和 `sh:SPARQLConstraint` 作为原生 SPARQL 查询针对 VP 存储进行评估，支持无法用纯属性路径 SHACL 表达的复杂跨形状验证。 |
| **JSON↔RDF 映射注册表** | 使用 `register_json_mapping(name, context_jsonb, shape_iri)` 注册命名的双向 JSON↔RDF 映射。`ingest_json(mapping, document)` 使用存储的上下文将 JSON 文档转换为 RDF 三元组；`export_json_node(mapping, iri)` 将图节点转换回 JSON。与可选 SHACL 形状的映射不一致记录在 `_pg_ripple.json_mapping_warnings` 中。 |
| **R2RML 直接映射** | `pg_ripple.r2rml_load(mapping_ttl)` 应用 R2RML 映射文档，将同一数据库中的关系表转换为 RDF 三元组，直接插入 VP 存储。 |
| **图分析（PageRank）** | 通过 `pg_ripple.pagerank_run()` 的 Datalog 原生迭代 PageRank。支持主题敏感、个性化、置信度加权和时间衰减变体。通过 IVM 脏边队列增量刷新。四种中心性度量（介数、接近度、度、Katz）。`pg:pagerank()` SPARQL 函数。通过 `explain_pagerank()` 的分数解释树。基于草图的近似 top-N。SHACL 感知排名。标准格式导出（CSV、Turtle、N-Triples、JSON-LD）。 |
| **概率推理** | Datalog 规则上的 `@weight(FLOAT)` 注解用于带噪或置信度传播的概率推理。`pg:confidence()`、`pg:fuzzy_match()`、`pg:token_set_ratio()` SPARQL 函数。软 SHACL 评分（`shacl_score()`）。置信度加权批量加载。PROV-O 源信任传播。具有良基收敛保证的循环概率程序。 |
| **贝叶斯置信度更新** | `update_confidence(subject, predicate, object, evidence)` 以赔率形式应用贝叶斯定理进行动态信念修正。`bulk_update_confidence()` 批量摄取 CSV 或 JSON-L 更新。只能追加的 `_pg_ripple.evidence_log` 记录每次更新及完整审计追踪。下游置信度传播沿推导 DAG 向上游走，深度可配置。可配置策略：`'bayesian'`、`'noisy-or'` 或 `'manual'`。 |
| **时序知识图谱** | 通过 `mark_temporal()` 将谓词注册为时序。一级 `_pg_ripple.temporal_facts` 存储跟踪 `(s, p, o, valid_from, valid_to)`。`insert_triple_temporal()` 以快照或版本化模式插入带时间戳的事实。Datalog 时序过滤器：`AFTER`、`BEFORE`、`DURING`。顺序运算符：`temporal_within()`、`temporal_sequence()`、`temporal_consecutive()`。`sh:validFor` SHACL 约束。CDC 自动录制连接到 `insert_triple()`。`retract_triple_temporal()` 关闭开放区间。 |
| **证明树和正当性** | 启用 `pg_ripple.record_derivations` 将每个 Datalog 派生事实的溯源记录到 `_pg_ripple.derivations`。`justify(subject, predicate, object)` 返回带有循环保护（深度限制 64）的完整反向链接证明树作为 JSONB。`vacuum_derivations()` 清理孤并行。证明树在 DRed 删除中能正常存活。 |
| **自然语言解释** | `explain_inference(subject, predicate, object)` 返回关于事实为何被派生的人类可读叙述 —— 通过配置的 LLM 端点或确定性后备渲染器。`explain_inference_jsonb()` 返回组合的证明树和叙述，供程序化消费者使用。结果缓存在 `_pg_ripple.explanation_cache` 中，TTL 可配置。`vacuum_explanation_cache()` 清除过期条目。 |
| **假设性（what-if）推理** | `hypothetical_inference(hypotheses, rules)` 对投机性图修改运行 Datalog 推理，但不持久化更改。返回 `{"derived": [...], "retracted": [...]}`。所有更改都包装在 PostgreSQL 内部子事务中 —— VP 表永远不会被修改。`hypothetical_max_assertions` GUC 限制资源使用。`pg_ripple_http` 中的 REST 端点 `POST /hypothetical`。 |
| **冲突检测** | `rule_conflicts(ruleset, mode)` 检测矛盾的 Datalog 规则。静态模式：结构性分析（相同头部的对立值、规则对 SHACL 约束）。运行时模式：扫描 `_pg_ripple.derivations` 查找已派生的矛盾。`block_on_conflict` GUC 在发现冲突时停止 `infer()`。`rule_conflict_check_on_load` GUC 在加载时启用自动静态分析。`pg_ripple_http` 中的 REST 端点 `GET /rule-conflicts/{ruleset}`。 |
| **领域规则库** | `install_rule_library(source)` 从 URL 或本地 Turtle 文件安装版本化的 Datalog 规则集和 SHACL 形状。库携带元数据（`dcterms:title`、`owl:versionInfo`、许可证 IRI）和依赖声明。内置 SSRF 保护、许可证检查和拓扑依赖解析。`upgrade_rule_library()` / `uninstall_rule_library()` 管理生命周期。`list_rule_libraries()` 枚举已安装的库。`pg_ripple_http` 中的 REST 端点 `GET /rule-libraries`。 |
| **引导式规则创作** | `validate_rule(rule)` 执行静态分析 —— 检测语法错误、未绑定头部变量、不安全否定和分层循环。`draft_rule_from_nl(description)` 调用配置的 LLM 将自然语言描述翻译成排序的 Datalog 候选。`suggest_rules(graph_iri)` 扫描谓词共现模式以提出候选规则。`pg_ripple_http` 中的 REST 端点 `POST /rules/draft` 和 `POST /rules/validate`。 |
| **神经符号记录链接（NS-RL）** | 六个 SPARQL 和 Datalog 字符串相似性内置函数：`pg:trigram_similarity()`、`pg:levenshtein()`、`pg:levenshtein_less_equal()`、`pg:soundex()`、`pg:metaphone()`、`pg:jaro_winkler()`。三个内置 ER 阻塞模板（`email`、`postal_name`、`name_prefix`）。`resolve_entities(source_graph, target_graph)` 编排五阶段管道：符号阻塞、基于嵌入的候选生成、SHACL 验证门控、`owl:sameAs` 规范化、RDF-star 溯源注解。干运行模式返回 JSON 摘要而不写入。 |
| **NS-RL 评估和监控** | `evaluate_resolution(gold_graph)` 针对黄金标准计算成对（精确率/召回率/F1）、阻塞（pairs_completeness/reduction_ratio/F-PQ）和 B³ 聚类指标。`enable_er_monitoring()` 创建三个实时流表：未解析实体、聚类大小和解析仪表板。`explain_rule(rule_id)` 用自然语言叙述规则。`owl:sameAs` 异常检测将触发 PT550 的断言记录到 `_pg_ripple.sameas_anomaly_log`。Magellan ER 基准 CI 门控（Abt-Buy F1 ≥ 0.78，DBLP-ACM F1 ≥ 0.90）。 |
| **隐私保护记录链接（PPRL）** | `bloom_encode(value, key, hash_count, length)` 使用 HMAC-SHA-256 生成 CLK Bloom 过滤器编码 —— 使实体匹配能够在不暴露原始 PII 的情况下进行。`dice_similarity(a, b)` 计算两个 Bloom 过滤器十六进制位向量的 Dice 系数。`pg:dice_similarity(?a, ?b)` SPARQL FILTER 函数和 Datalog 内置谓词，用于跨图隐私安全实体匹配。`dp_noisy_count(query, epsilon)` 和 `dp_noisy_histogram(query, key_col, count_col, epsilon)` 为聚合查询添加拉普拉斯差分隐私噪声，用于隐私保护分析。 |
| **时序图快照** | `graph_at(graph_iri, snapshot_time)` 从 `_pg_ripple.temporal_facts` 实现时间点命名图快照，注册一个确定性 `urn:snapshot:…` IRI，可直接在 `GRAPH <iri> { … }` SPARQL 查询中使用。`graph_diff(graph_iri, from_ts, to_ts)` 返回两个时间戳之间的 `'added'`/`'removed'` 增量行，用于符合审计的变更日志和增量导出。快照在 `pg_ripple.snapshot_retention_days`（默认 30 天）后自动垃圾回收。HTTP 端点 `GET /temporal/graphs/{iri}/snapshot` 和 `GET /temporal/graphs/{iri}/diff` 暴露这两个操作。Prometheus gauge `pg_ripple_graph_snapshots_total` 跟踪实时快照计数。 |
| **每个端点的联邦凭证** | `set_federation_credential(endpoint_iri, auth_type, token)` 为任何联邦端点注册或原子替换加密的 Bearer 或 API 密钥凭证；令牌通过 `pgcrypto.pgp_sym_encrypt` 静态存储，永远不会以明文形式返回。`rotate_federation_credential(endpoint_iri, new_token)` 替换令牌并原子记录轮换时间戳。`federation_credential_audit()` 返回操作元数据（令牌年龄、上次使用）而不暴露秘密。联邦执行器在 SSRF 验证后自动注入正确的 HTTP 头 —— 现有 SPARQL 查询无需更改。HTTP 端点 `GET /federation/{endpoint}/auth-status` 暴露凭证状态用于监控。 |
| **实时自动更新视图** | 将 SPARQL 查询定义为视图；pg_ripple（配合可选的 `pg_trickle` 配套扩展）使视图在数据变更时自动保持最新。 |
| **访问控制** | 命名图具有由 PostgreSQL 内置权限系统支持的行级安全性。每个图可以授予特定的数据库角色，就像表一样。当配置 `pg_ripple.read_replica_dsn` 时，读取副本路由自动将读取查询发送到副本。 |
| **全文搜索** | 使用 PostgreSQL 的快速全文搜索索引搜索字面量值（名称、描述、笔记）的文本。 |

以下是使用 pg_ripple 从 SQL 操作的示例：

```sql
CREATE EXTENSION pg_ripple;

-- 导入 Turtle 文件（RDF 知识图谱的标准文本格式）
SELECT pg_ripple.load_turtle(pg_read_file('/data/people.ttl'));

-- 使用属性路径查询：查找 Alice 通过 "knows" 能到达的所有人
-- （自动追踪 Alice→Bob→Carol→… 链）
SELECT * FROM pg_ripple.sparql('
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name WHERE {
    <http://example.org/Alice> foaf:knows+ ?person .
    ?person foaf:name ?name .
  }
');

-- 强制执行 SHACL 约束：每个 Person 必须恰好有一个名字
SELECT pg_ripple.load_shacl('
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  <http://example.org/PersonShape> a sh:NodeShape ;
    sh:targetClass <http://example.org/Person> ;
    sh:property [ sh:path foaf:name ; sh:minCount 1 ; sh:maxCount 1 ] .
');

-- 将整个图导出为 Turtle
SELECT pg_ripple.export_turtle();

-- SPARQL CONSTRUCT → JSON-LD 用于 REST API
SELECT pg_ripple.sparql_construct_jsonld('
  CONSTRUCT { ?s ?p ?o } WHERE { ?s a <http://schema.org/Person> ; ?p ?o }
');

-- 加载 RDFS 蕴含规则并运行推理
-- 此后，如果 :Dog 是 :Animal 的子类，且 :Rex 是 Dog，
-- 则 SPARQL 在查询 Animal 时也会返回 :Rex
SELECT pg_ripple.load_rules_builtin('rdfs');
SELECT pg_ripple.infer('rdfs');

-- 编写自定义规则（传递性管理链）
SELECT pg_ripple.load_rules(
  '?x ex:indirectManager ?z :- ?x ex:manager ?z .
   ?x ex:indirectManager ?z :- ?x ex:manager ?y, ?y ex:indirectManager ?z .',
  'org_rules'
);
SELECT pg_ripple.infer('org_rules');

-- ── AI / LLM 集成 ──────────────────────────────────────────────

-- 混合检索：图模式 + 矢量相似性在一个查询中
-- 查找与某主题语义相似、由合著者撰写的论文
SELECT * FROM pg_ripple.sparql('
  PREFIX ex: <http://example.org/>
  PREFIX pg:  <http://pg-ripple.io/fn/>
  SELECT ?paper ?title ?score WHERE {
    <http://example.org/Alice> ex:coAuthor+ ?colleague .
    ?colleague ex:authored ?paper .
    ?paper ex:title ?title .
    BIND(pg:similar(?paper, "graph neural networks") AS ?score)
    FILTER(?score > 0.75)
  }
  ORDER BY DESC(?score)
');

-- 为 LLM 生成结构化 JSON-LD 系统提示词
-- 帧将输出塑造成你提示词模板完全期望的 JSON 形状
SELECT pg_ripple.sparql_construct_jsonld(
  'CONSTRUCT { ?s ex:name ?name ; ex:role ?role ; ex:manages ?report }
   WHERE   { ?s a ex:Person ; ex:name ?name ; ex:role ?role .
             OPTIONAL { ?s ex:manages ?report } }',
  -- JSON-LD 帧：生成嵌套 {"name":..., "manages":[...]} 对象
  '{"@type": "ex:Person", "ex:manages": {}}'
);

-- 图上下文化的 RAG 检索
-- 返回一个 JSONB 上下文块，可直接用作 LLM 系统提示词
SELECT pg_ripple.rag_retrieve(
  query_embedding  => ai.embed('Who manages the Oslo team?'),
  graph_patterns   => ARRAY['?s ex:locatedIn ex:Oslo', '?s ex:role ?role'],
  top_k            => 10
);
```

---

## AI 和 LLM 用例

pg_ripple 是需要结构化、可解释上下文的 AI 应用的天然选择 —— 而不仅仅是一袋向量。以下是三个具体场景。

### 知识增强的 RAG

纯向量搜索找到*相似*的文档，但丢失了它们之间的*关系*。pg_ripple 让你同时使用两者：SPARQL 图模式通过关系选择实体（*"Alice 合著者在过去两年撰写的论文"*），然后用向量相似性过滤器（`pg:similar()`）按语义接近度排序。倒数排名融合合并两个结果列表。发送给 LLM 的检索上下文比平面 top-k 向量搜索更精确、更可解释。

### 嵌入前的实体解析

企业数据有重复：`"Alice Smith"`、`"A. Smith"` 和 `"alice.smith@example.com"` 可能都指同一个人。pg_ripple 的 `owl:sameAs` 实体规范化在推理或嵌入之前将这些折叠成单个规范实体。当 LLM 询问 Alice 时，它得到统一视图 —— 而不是三个矛盾的碎片。

### 通过 JSON-LD 帧构建结构化提示词

Token 预算很重要。`sparql_construct_jsonld()` 接受 SPARQL CONSTRUCT 查询和 JSON-LD 帧 —— 一个描述你想要的精确 JSON 形状的模板 —— 并生成紧凑的、结构化的提示词上下文，没有冗余三元组、没有平面转储、不需要后处理。帧定义包含哪些属性、以什么顺序、以及如何嵌套它们。输出直接插入系统提示词。

---

## 我们要去往何方

通往 v1.0.0 的道路上只剩下一个版本。

v0.91.0–v0.111.0 开发周期增加了深度推理能力：证明树和正当性基础设施（v0.100.0）、通过 LLM 或确定性后备的自然语言派生事实解释（v0.101.0）、what-if 假设性推理（v0.102.0）、Datalog 冲突检测（v0.103.0）、版本化领域规则库（v0.104.0）、LLM 支持的 NL 到 Datalog 翻译的引导式规则创作（v0.105.0）、带 AFTER/BEFORE/DURING 运算符和 CDC 集成的一级时序事实存储（v0.106.0–v0.107.0）、带证据日志和推导 DAG 传播的贝叶斯置信度更新（v0.108.0）、带六个字符串相似性内置函数和五阶段 `resolve_entities()` 管道的神经符号记录链接（v0.109.0）、带实时 ER 监控流表和规则可解释性的 NS-RL 评估工具（v0.110.0）、以及通过 CLK Bloom 过滤器编码和差分隐私聚合的隐私保护记录链接（v0.111.0）。`pg_ripple.feature_status()` 中的每一行都显示 `implemented`。

v0.112.0–v0.126.0 周期专注于加固、正确性和操作完整性：安全审计修复，包括带注解的 unsafe 块和实体解析中连线的 SHACL 验证门控（v0.112.0）、通过默认开启的 COPY 路径实现 5–10 倍批量加载吞吐量（v0.113.0）、HTTP API 对等和模块分解（v0.114.0–v0.116.0）、时序 Allen 区间关系和隐私预算注册表（v0.118.0）、OWL `propertyChainAxiom` n 跳链、SERVICE 断路器和模式感知 NL→SPARQL 翻译（v0.119.0）、PageRank 解释树、管理诊断快照、租户配额 API 和规则库联邦（v0.120.0）、可观察性加固和测试覆盖率闭合（v0.121.0–v0.123.0）、SPARQL 属性路径笛卡尔积错误的精确修复（该错误曾导致复合路径结果膨胀）（v0.124.0）、通过 `graph_at()` / `graph_diff()` 实现时间点命名图快照和 diff API（v0.125.0）、以及加密的每个端点联邦凭证及自动头注入和原子轮换（v0.126.0）。

### v1.0.0 — 生产发布

最后一个里程碑：完整的 API 和文档冻结、长期支持承诺以及公开的生产档案。所有合规套件（SPARQL 1.1、SHACL Core、OWL 2 RL、LUBM）仍然是必需的门槛；性能回归 CI 和发布证据仪表板是强制性的工件。

---

## 为什么选择 pg_ripple？

大多数 RDF 三元组存储是独立系统 —— 单独的进程、单独的存储、单独的管理。pg_ripple 采取了不同的方法：它将三元组存储*放入* PostgreSQL。

这意味着你将获得：

- **一个数据库** 同时处理关系数据和知识图谱
- **PostgreSQL 的完整工具箱** —— MVCC、WAL 复制、`pg_dump`/`pg_restore`、`EXPLAIN`、监控、连接池 —— 开箱即用
- **无需数据移动** —— 你的 RDF 数据与现有表共存；SPARQL 查询可以与 SQL 在同一事务中共存
- **熟悉的操作** —— 任何了解 PostgreSQL 的 DBA 都可以操作 pg_ripple

### 与其他方案对比

> **注意**：下表中标记为 "Yes" 的 pg_ripple 功能在 v0.1.0–v0.126.0 中实现。W3C SPARQL 1.1 查询、更新、SHACL Core 和 OWL 2 RL 合规性为 100%。竞品能力反映公开记录的功能集。

| 能力 | pg_ripple | Blazegraph | Virtuoso | Apache Fuseki |
|---|---|---|---|---|
| 运行在 PostgreSQL 内部 | Yes | No | No | No |
| SPARQL 1.1 查询 | Yes | Yes | Yes | Yes |
| SPARQL 1.1 更新 | Yes | Yes | Yes | Yes |
| SHACL 验证 | Yes（同步 + 异步） | No | No | Plugin |
| Datalog 推理（RDFS、OWL RL） | Yes | No | Limited | Partial |
| 增量 SPARQL 视图（IVM） | Yes（通过 pg_trickle） | No | No | No |
| RDF-star / RDF 1.2 | Yes | No | No | Yes |
| 时序 RDF 查询 | Yes | No | Limited | No |
| 时序顺序模式（WITHIN、SEQUENCE） | Yes | No | No | No |
| 证明树和正当性 | Yes | No | No | No |
| 假设性（what-if）推理 | Yes | No | No | No |
| 贝叶斯置信度更新 | Yes | No | No | No |
| 神经符号记录链接（NS-RL） | Yes | No | No | No |
| 隐私保护记录链接（PPRL） | Yes | No | No | No |
| 差分隐私聚合 | Yes | No | No | No |
| 时序图快照 | Yes | No | No | No |
| 加密联邦凭证 | Yes | No | No | No |
| 水平分片（Citus） | Yes | No | No | No |
| SPARQL 联邦 | Yes | No | Yes | Yes |
| 命名图访问控制 | Yes（PostgreSQL RLS） | No | ACL | Apache Shiro |
| 全文搜索 | Yes（PostgreSQL GIN） | Yes | Yes | Yes |
| 备份和复制 | PostgreSQL WAL | Custom | Custom | Custom |
| 语言 | Rust | Java | C | Java |

---

## 架构

pg_ripple 从零开始为在 PostgreSQL 内部实现高性能而构建。

> 下图显示了内部管道：查询以 SPARQL 文本形式输入，经过优化，翻译成 SQL，针对存储层执行 —— 全部在单个 PostgreSQL 会话内完成。

```
 SPARQL Query / Update                   HTTP API
        │                                   │
        ▼                                   ▼
 ┌─────────────────┐              ┌──────────────────┐
 │  SPARQL Parser   │              │  pg_ripple_http   │
 │  (spargebra)     │              │  (Rust binary)    │
 └────────┬────────┘              └────────┬─────────┘
          │                                │
          ▼                                │
 ┌─────────────────┐                       │
 │  Algebra         │◄──────────────────────┘
 │  Optimizer       │
 │  · Self-join     │
 │    elimination   │
 │  · Filter        │
 │    pushdown      │
 │  · SHACL hints   │
 └────────┬────────┘
          │
          ▼
 ┌─────────────────┐    ┌──────────────────┐
 │  SQL Generator   │───▶│  PostgreSQL       │
 │  (integer joins) │    │  Executor (SPI)   │
 └─────────────────┘    └────────┬─────────┘
                                 │
                    ┌────────────┴────────────┐
                    │                         │
              ┌─────▼─────┐           ┌───────▼──────┐
              │ VP Tables  │           │  Dictionary   │
              │ (per-      │           │  (XXH3-128    │
              │ predicate) │           │   → i64)      │
              │            │           │              │
              │ Delta      │           │  Sharded LRU │
              │ (writes)   │           │  Cache (shmem)│
              │ Main       │           └──────────────┘
              │ (reads)    │
              └────────────┘
```

### 数据如何存储

- **一切使用紧凑 ID**：每个值 —— URI、标签、字面量 —— 都被分配一个短整数 ID。内部连接使用这些整数而非原始字符串，这保持了存储小且查询快。
- **每种关系类型一个表**：关于 `worksAt`、`knows`、`birthDate` 等事实存储在单独的表中。仅询问 `worksAt` 的查询只扫描该表，而非整个数据集。
- **读写分离车道**：新数据进入快速"增量"区域；后台工作进程不断将其移动到优化的"主"区域。大量插入工作负载和复杂查询永远不会相互拖慢。

### 性能目标

| 操作 | 目标 | 规模化情况下 |
|---|---|---|
| 批量加载 | >100,000 facts/sec | 批量导入，延迟索引 |
| 事务性插入 | >10,000 facts/sec | 增量分区，异步验证 |
| 简单查询 | <5 ms | 1000 万 facts |
| 多跳查询（5 个模式） | <20 ms | 1000 万 facts |
| 深度路径遍历（深度 10） | <100 ms | 1000 万 facts |
| 字典查找（缓存命中） | <1 μs | 分片的内存缓存 |

---

## 技术栈

| 组件 | 技术 |
|---|---|
| 语言 | Rust（2024 版） |
| PostgreSQL 绑定 | [pgrx](https://github.com/pgcentralfoundation/pgrx) 0.18 |
| PostgreSQL 版本 | 18.x |
| SPARQL 解析器 | [spargebra](https://crates.io/crates/spargebra) — W3C 合规的 SPARQL 1.1 代数 |
| SPARQL 优化器 | [sparopt](https://crates.io/crates/sparopt) — 第一轮代数优化器（过滤器下推、常量折叠） |
| RDF 解析器 | [rio_turtle](https://crates.io/crates/rio_turtle)、[rio_xml](https://crates.io/crates/rio_xml) — Turtle、N-Triples、RDF/XML；[oxttl](https://crates.io/crates/oxttl) / [oxrdf](https://crates.io/crates/oxrdf) — RDF-star / Turtle-star |
| 哈希 | [xxhash-rust](https://crates.io/crates/xxhash-rust)（XXH3-128）—— 用于字典去重的快速非加密哈希 |
| 序列化 | [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json) —— SHACL 报告、SPARQL 结果、配置 |
| HTTP 服务器 | [axum](https://crates.io/crates/axum)（构建在 [tokio](https://tokio.rs/) 之上）—— SPARQL 协议 HTTP 端点（`pg_ripple_http` 二进制文件） |
| PG 客户端（HTTP 服务） | [tokio-postgres](https://crates.io/crates/tokio-postgres) + [deadpool-postgres](https://crates.io/crates/deadpool-postgres) —— HTTP 服务到 PostgreSQL 的异步连接池 |
| HTTP 客户端（联邦） | [ureq](https://crates.io/crates/ureq) 2.12 —— 到远程 SPARQL 端点的出站调用（`SERVICE` 关键字）；每个后端会话的连接池 `Agent` |
| IVM / 流表 | [pg_trickle](https://github.com/trickle-labs/pg-trickle)（*可选配套扩展*）—— 增量 SPARQL 视图、ExtVP、实时统计 |
| 字典缓存 | [lru](https://crates.io/crates/lru) —— 后端本地 LRU 缓存（v0.1.0–v0.5.1）；在 v0.6.0 中替换为分片的共享内存映射 |
| 错误处理 | [thiserror](https://crates.io/crates/thiserror) —— 带 PT 错误代码常量（PT001–PT799）的类型化错误枚举 |
| 测试 | pgrx `#[pg_test]`、`cargo pgrx regress`、[proptest](https://crates.io/crates/proptest)、[cargo-fuzz](https://crates.io/crates/cargo-fuzz) |

---

## pg_trickle 依赖矩阵

[pg_trickle](https://github.com/trickle-labs/pg-trickle) 是一个可选的配套扩展。
下表显示哪些 pg_ripple 功能需要 pg_trickle，哪些可以独立运行。

| 功能 | 独立运行 | 需要 pg_trickle |
|---|---|---|
| SPARQL SELECT / ASK / CONSTRUCT / DESCRIBE | ✓ | — |
| SPARQL UPDATE（INSERT/DELETE/CLEAR/LOAD） | ✓ | — |
| 属性路径（ZeroOrMorePath、InversePath、…） | ✓ | — |
| 联邦（SERVICE、并行、基于成本） | ✓ | — |
| SHACL 验证（Core + SPARQL 约束） | ✓ | — |
| Datalog 规则（RDFS/OWL RL、 半朴素、魔法集） | ✓ | — |
| HTAP 合并工作进程 | ✓ | — |
| 批量加载（Turtle / N-Triples / RDF-XML） | ✓ | — |
| GeoSPARQL 1.1（geof:distance、ST_DWithin、…） | ✓ | — |
| 全文搜索（RDF-FTS） | ✓ | — |
| 向量 + SPARQL 混合搜索 | ✓ | — |
| pg_dump / pg_restore 往返 | ✓ | — |
| CDC 订阅（三元组变更上的 NOTIFY） | ✓ | — |
| 增量 SPARQL 视图（IVM） | — | ✓ 必需 |
| ExtVP 物化统计 | — | ✓ 必需 |
| 实时自动更新 CONSTRUCT 视图 | — | ✓ 必需 |
| CDC 期间的 Citus 重新平衡暂停/恢复 | ✓（NOTIFY 信号） | ✓（暂停/恢复逻辑） |
| 联邦的读取副本路由 | ✓ | — |

---

## 快速开始

### 前置要求

- PostgreSQL 18
- Rust 稳定工具链（pg_ripple 是一个编译型扩展）
- [pgrx](https://github.com/pgcentralfoundation/pgrx) 0.18

### 构建和安装

```bash
git clone https://github.com/trickle-labs/pg-ripple.git
cd pg-ripple

# 为 PostgreSQL 18 初始化 pgrx
cargo pgrx init --pg18 $(which pg_config)

# 运行测试
cargo pgrx test pg18

# 安装到本地 PostgreSQL
cargo pgrx install --pg-config $(which pg_config)
```

### 启用扩展

```sql
CREATE EXTENSION pg_ripple;
```

---

## 质量与测试

pg_ripple 按生产级标准构建：

- **W3C 合规性** —— 官方 SPARQL 1.1 查询、SPARQL 1.1 更新和 SHACL Core 测试套件 100% 通过率（约 3000 个测试，并行运行，2 分钟内完成）
- **Apache Jena 测试套件** —— ~1000 个额外测试，涵盖 XSD 数字提升、时区感知日期/时间、空白节点作用域和所有 SPARQL 字符串函数
- **WatDiv 基准** —— 所有 100 个 WatDiv 查询模板（星型、链型、雪花型、复杂型）在 1000 万三元组数据集上针对 ±0.1% 行数基线验证正确性
- **LUBM 合规套件** —— 所有 14 个规范 LUBM 查询在合成大学 OWL 本体上通过；包含 Datalog 验证子套件，确认 `infer('owl-rl')` 产生正确的超类型蕴含（v0.44.0）
- **W3C OWL 2 RL 合规套件** —— W3C OWL 2 RL 测试清单（蕴含性、一致性和不一致性测试）在 CI 中运行；**在 v0.51.0 达到 100% 通过率（66/66）** —— CI 中的阻塞门控（v0.51.0）
- **基于属性的测试** —— `proptest` 套件断言代数不变量：SPARQL 代数往返产生字节相同的 SQL，字典编码/解码对于 10,000 个随机不同项始终稳定且无冲突，JSON-LD 帧保留所有匹配的 IRI（v0.51.0）
- **广泛的测试套件** —— 截至 v0.126.0，300+ pg_regress 测试和基于属性（`proptest`）套件覆盖每个 SQL 暴露函数、每个功能和每个边缘情况
- **安全测试** —— 抵抗注入攻击、格式错误的输入和资源耗尽
- **模糊测试** —— 联邦结果解码器、查询管道和 URL 主机解析器持续进行模糊测试（每晚，每目标 120 秒）；来自远程 SERVICE 端点的任意 XML/JSON 不能导致崩溃或 panic（v0.51.0）
- **性能回归 CI** —— BSBM 基准（100 万三元组产品数据集，12 个探索查询）和自动化吞吐量基准在性能下降超过 10% 时使构建失败（v0.51.0）
- **安全 CI** —— `cargo audit --deny warnings` 在每个拉取请求上运行；SBOM（CycloneDX）在每个版本上生成并附加；GitHub Actions 引用固定到完整 SHA；Docker 发布镜像通过 Trivy 扫描，使用不可变摘要
- **稳定性** —— 72 小时浸泡测试，包含已发布工件（内存趋势、合并延迟、查询 p50/p95/p99、错误计数）、内存泄漏检测和崩溃恢复测试（v0.67.0）
- **升级和备份验收** —— 所有支持的 0.x 版本的迁移链、`pg_dump`/restore 往返和回滚指导在 CI 中测试（v0.67.0）
- **公开基准基线** —— BSBM、WatDiv、LUBM、批量 N-Triples/Turtle 加载、HTAP 合并吞吐量、构造规则增量维护、Datalog DRed、向量混合搜索、Arrow IPC 导出、Citus fan-out、bidi relay 吞吐量和 Magellan ER 基准（Abt-Buy、DBLP-ACM）随硬件、数据集大小和原始输出发布；基线刷新到 v0.126.0

---

## 贡献

欢迎贡献、反馈和设计讨论。在提交拉取请求之前请先开 issue 讨论。

---

## 许可证

Apache License 2.0 —— 详见 [LICENSE](LICENSE)。
