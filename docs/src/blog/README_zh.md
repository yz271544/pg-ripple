# pg_ripple 博客

> **注意**：博客文章使用 AI 辅助（GitHub Copilot / Claude）撰写，作为探索针对小众系统工程主题的 LLM 生成技术写作的一种方式。技术内容已通过准确性审查，但请将文章视为草稿，而非经过正式审查的文档。

完整博客位于存储库的 [`blog/`](https://github.com/trickle-labs/pg-ripple/tree/main/blog) 目录下。
各个文章链接如下。

---

## 核心概念与架构

| 文章 | 摘要 |
|------|---------|
| [为什么在 PostgreSQL 中使用 RDF？](https://github.com/trickle-labs/pg-ripple/blob/main/blog/why-rdf-in-postgresql.md) | 在数据已有的位置构建三元组存储的论点 —— 无 ETL 管道、无独立集群、无阻抗失配。 |
| [垂直分区：每个谓词一张表](https://github.com/trickle-labs/pg-ripple/blob/main/blog/vertical-partitioning-explained.md) | 深入了解 pg_ripple 的 VP 存储模型以及它如何改变选择性查询计划。 |
| [一切都是整数](https://github.com/trickle-labs/pg-ripple/blob/main/blog/dictionary-encoding-integer-joins.md) | 使用 XXH3-128 进行字典编码：为什么字符串比较在三元组存储中是性能缺陷。 |
| [SPARQL 如何成为 PostgreSQL 查询计划](https://github.com/trickle-labs/pg-ripple/blob/main/blog/sparql-to-sql-translation.md) | 从 SPARQL 文本到 `spargebra` 代数，再到 SQL，再到 SPI 执行的翻译管道。 |

## 存储与性能

| 文章 | 摘要 |
|------|---------|
| [三元组的 HTAP：同时进行读取和写入](https://github.com/trickle-labs/pg-ripple/blob/main/blog/htap-reads-and-writes.md) | delta/main/tombstone 拆分让 pg_ripple 在无锁的情况下处理并发的 OLTP 写入和分析型 SPARQL 查询。 |
| [Leapfrog Triejoin：当三角形查询遇上最优连接](https://github.com/trickle-labs/pg-ripple/blob/main/blog/leapfrog-triejoin.md) | 最坏情况最优连接如何被编译到 PostgreSQL 中以处理循环查询。 |
| [属性路径只是递归 CTE](https://github.com/trickle-labs/pg-ripple/blob/main/blog/property-paths-recursive-ctes.md) | SPARQL 属性路径使用 PostgreSQL 18 基于哈希的循环检测编译为 `WITH RECURSIVE … CYCLE`。 |

## 推理与演绎

| 文章 | 摘要 |
|------|---------|
| [PostgreSQL 内部的 Datalog](https://github.com/trickle-labs/pg-ripple/blob/main/blog/datalog-inside-postgresql.md) | 从规则自动派生事实 —— RDFS、OWL RL、传递闭包，全部作为 SQL 运行。 |
| [Magic Sets：问一个问题，只推断出所需](https://github.com/trickle-labs/pg-ripple/blob/main/blog/magic-sets-goal-directed.md) | 目标导向推理：200 万个推断三元组与 47 个之间的区别。 |
| [没有爆炸的 `owl:sameAs`](https://github.com/trickle-labs/pg-ripple/blob/main/blog/owl-sameas-entity-resolution.md) | 在查询时使用基于 `owl:sameAs` 链的并查集进行实体规范化。 |
| [四个内置规则集](https://github.com/trickle-labs/pg-ripple/blob/main/blog/builtin-reasoning-rules-explained.md) | RDFS、OWL RL、OWL EL 和 OWL QL 的实际作用 —— 每个规则都附有示例进行解释。 |
| [良基语义](https://github.com/trickle-labs/pg-ripple/blob/main/blog/well-founded-semantics.md) | 为什么分层否定并不总是足够的，以及良基语义如何处理递归否定。 |
| [OWL 属性链公理](https://github.com/trickle-labs/pg-ripple/blob/main/blog/owl-property-chain-axiom.md) | 将 `owl:propertyChainAxiom` 编译为递归 SQL 连接。 |

## 数据质量与验证

| 文章 | 摘要 |
|------|---------|
| [SHACL：面向无 schema 的 schema 验证](https://github.com/trickle-labs/pg-ripple/blob/main/blog/shacl-data-quality.md) | pg_ripple 如何将 SHACL 形状编译为 DDL 约束和异步验证管道。 |

## AI、RAG 与向量搜索

| 文章 | 摘要 |
|------|---------|
| [向量 + SPARQL 混合搜索](https://github.com/trickle-labs/pg-ripple/blob/main/blog/vector-sparql-hybrid-search.md) | 将 pgvector 近似最近邻搜索与 SPARQL 图遍历相结合。 |
| [自然语言到 SPARQL](https://github.com/trickle-labs/pg-ripple/blob/main/blog/natural-language-to-sparql.md) | 由 LLM 驱动的 NL→SPARQL 翻译，使用少样本提示。 |
| [GraphRAG 知识导出](https://github.com/trickle-labs/pg-ripple/blob/main/blog/graphrag-knowledge-export.md) | 为 Microsoft 的 GraphRAG 管道导出 pg_ripple 图。 |
| [神经-符号实体解析](https://github.com/trickle-labs/pg-ripple/blob/main/blog/neuro-symbolic-entity-resolution.md) | 将 ML 嵌入与 Datalog 规则相结合进行记录链接。 |

## 集成与运维

| 文章 | 摘要 |
|------|---------|
| [CDC → 知识图谱](https://github.com/trickle-labs/pg-ripple/blob/main/blog/cdc-knowledge-graphs.md) | 通过逻辑复制将关系变更事件流式传输到 RDF 图。 |
| [Citus 分片剪枝与 SPARQL](https://github.com/trickle-labs/pg-ripple/blob/main/blog/citus-shard-pruning-sparql.md) | `SERVICE` 子句如何将联邦 SPARQL 查询分发到正确的分片。 |
| [使用 pg-trickle 集成的 IVM](https://github.com/trickle-labs/pg-ripple/blob/main/blog/ivm-pg-trickle-integration.md) | 通过 CDC 流进行增量视图维护。 |
| [语义中心：pg-tide 中继](https://github.com/trickle-labs/pg-ripple/blob/main/blog/semantic-hub-trickle-relay.md) | 用于多实例 RDF 同步的星型拓扑。 |
| [GDPR 被遗忘权](https://github.com/trickle-labs/pg-ripple/blob/main/blog/gdpr-right-to-erasure.md) | 实施 GDPR 第 17 条 —— 级联三元组删除并跟踪出处。 |
| [多租户知识图谱](https://github.com/trickle-labs/pg-ripple/blob/main/blog/multi-tenant-knowledge-graphs.md) | 命名图隔离、行级安全性和每租户字典。 |

## 高级功能

| 文章 | 摘要 |
|------|---------|
| [CONSTRUCT 视图：实时转换](https://github.com/trickle-labs/pg-ripple/blob/main/blog/construct-views-live-transformations.md) | SPARQL CONSTRUCT 规则作为物化视图，支持自动 delta 传播。 |
| [JSON-LD 框架与嵌套 JSON](https://github.com/trickle-labs/pg-ripple/blob/main/blog/json-ld-framing-nested-json.md) | 按需将扁平的三元组存储转换为结构化的嵌套 JSON 文档。 |
| [JSON-LD 反向映射](https://github.com/trickle-labs/pg-ripple/blob/main/blog/json-ld-reverse-mapping.md) | 通过 JSON-LD context 将 RDF 变更写回到关系表。 |
| [R2RML：关系型到图](https://github.com/trickle-labs/pg-ripple/blob/main/blog/r2rml-relational-to-graph.md) | 使用 W3C R2RML 标准将关系表映射到 RDF。 |
| [RDF-star：关于语句的语句](https://github.com/trickle-labs/pg-ripple/blob/main/blog/rdf-star-statements-about-statements.md) | 为单个三元组添加出处、置信度和时间元数据的注释。 |
| [时序图快照](https://github.com/trickle-labs/pg-ripple/blob/main/blog/temporal-graph-snapshots.md) | 用于审计跟踪和时间旅行查询的时点命名图。 |
| [时序时间旅行查询](https://github.com/trickle-labs/pg-ripple/blob/main/blog/temporal-time-travel-queries.md) | 查询知识图谱的历史状态。 |
| [Allen's 区间关系](https://github.com/trickle-labs/pg-ripple/blob/main/blog/allen-interval-relations.md) | 在 Datalog 中使用 Allen 的十三个区间关系进行时间推理。 |
| [GeoSPARQL + PostGIS](https://github.com/trickle-labs/pg-ripple/blob/main/blog/geosparql-postgis-spatial.md) | 使用 GeoSPARQL 1.1 扩展对 RDF 几何文字进行空间查询。 |
| [SKOS 知识组织](https://github.com/trickle-labs/pg-ripple/blob/main/blog/skos-knowledge-organization.md) | 在 pg_ripple 中使用 SKOS 管理叙词表、概念体系和分类法。 |
| [使用 PageRank 进行图分析](https://github.com/trickle-labs/pg-ripple/blob/main/blog/pagerank.md) | 具有增量视图维护、WCOJ 和 Prometheus 仪表盘的 Datalog 原生 PageRank。 |
| [概率 Datalog](https://github.com/trickle-labs/pg-ripple/blob/main/blog/probabilistic-datalog.md) | 不确定知识：noisy-OR、置信度传播和软约束。 |
| [使用 PROV-O 进行出处跟踪](https://github.com/trickle-labs/pg-ripple/blob/main/blog/provenance-tracking-prov-o.md) | 使用 W3C PROV-O 本体记录数据血缘。 |
| [本体映射与对齐](https://github.com/trickle-labs/pg-ripple/blob/main/blog/ontology-mapping-alignment.md) | 通过 `owl:equivalentClass` 和 SPARQL CONSTRUCT 规则桥接异构词汇表。 |
| [SPARQL 联邦：本地 + 远程](https://github.com/trickle-labs/pg-ripple/blob/main/blog/sparql-federation-local-remote.md) | 在单个查询中混合本地 VP 表扫描和远程 SPARQL 端点。 |
| [EXPLAIN SPARQL：查询计划](https://github.com/trickle-labs/pg-ripple/blob/main/blog/explain-sparql-query-plans.md) | 阅读 pg_ripple 的查询计划输出以诊断慢速 SPARQL 查询。 |
| [规则库联邦](https://github.com/trickle-labs/pg-ripple/blob/main/blog/rule-library-federation.md) | 跨 pg_ripple 实例发布和订阅共享的 Datalog 规则库。 |
| [联邦断路器](https://github.com/trickle-labs/pg-ripple/blob/main/blog/federation-circuit-breaker.md) | 具有断路器、超时和后备图的弹性联邦查询。 |
| [DCTERMS、Schema.org 和 FOAF 捆绑包](https://github.com/trickle-labs/pg-ripple/blob/main/blog/dcterms-schema-foaf-bundles.md) | 通用本体的内置词汇捆绑包。 |
| [不确定知识](https://github.com/trickle-labs/pg-ripple/blob/main/blog/uncertain-knowledge.md) | PostgreSQL 中的概率 Datalog：表示和查询不确定事实。 |