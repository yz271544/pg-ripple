# 总结

[什么是 pg_ripple？](landing.md)

---

# 评估

- [何时使用 pg_ripple](evaluate/when-to-use.md)
- [架构概览](evaluate/architecture-glance.md)
- [与替代方案的对比](evaluate/comparison.md)
- [性能与符合性结果](evaluate/performance-results.md)
- [v0.132.0 浸泡测试证据](benchmarks/soak-0.132.0.md)

---

# 入门

- [安装](getting-started/installation.md)
- [Hello World — 五分钟演练](getting-started/hello-world.md)
- [引导式教程 — 在 30 分钟内构建知识图谱](getting-started/tutorial.md)
- [关键概念 — PostgreSQL 用户的 RDF](getting-started/key-concepts.md)
- [演练场 — 无需安装即可试用](user-guide/playground.md)

---

# 功能深度探索

- [存储知识](features/storing-knowledge.md)
- [加载数据](features/loading-data.md)
- [使用 SPARQL 查询](features/querying-with-sparql.md)
- [验证数据质量（SHACL）](features/validating-data-quality.md)
- [推理和推断（Datalog）](features/reasoning-and-inference.md)
- [导出和共享](features/exporting-and-sharing.md)
- [实时视图和订阅](features/live-views-and-subscriptions.md)
- [地理空间（GeoSPARQL）](features/geospatial.md)
- [全文搜索](features/full-text-search.md)
- [时间性和出处](features/temporal-and-provenance.md)
- [多租户图](features/multi-tenant-graphs.md)
- [OWL 2 Profiles（RL / EL / QL）](features/owl-profiles.md)
- [SHACL-SPARQL 规则](features/shacl-sparql-rules.md)
- [R2RML — 关系型到 RDF](features/r2rml.md)
- [格 Datalog — 何时及为何](features/lattice-datalog.md)
- [高级推理：WCOJ、DRed 和 Tabling](features/advanced-inference.md)
- [图分析（PageRank）](features/pagerank.md)
- [概率推理](features/uncertain-knowledge.md)
- [CDC 订阅](features/cdc-subscriptions.md)
- [实时 SPARQL 订阅](features/live-subscriptions.md)
- [JSON 映射（双向 JSON-LD）](features/json-mapping.md)
- [Cypher / LPG → RDF 映射](features/lpg-mapping.md)

---

# AI、RAG & 记录链接

- [AI 概述 — 决策树](features/ai-overview.md)
- [向量和混合搜索](features/vector-and-hybrid-search.md)
- [RAG 管道 — `rag_context()`](user-guide/rag-pipeline.md)
- [自然语言到 SPARQL](features/nl-to-sparql.md)
- [知识图谱嵌入（KGE）](features/knowledge-graph-embeddings.md)
- [记录链接和实体解析](features/record-linkage.md)
- [GraphRAG（Microsoft）](features/graphrag.md)
- [向量联邦](user-guide/vector-federation.md)
- [AI 代理集成（LangChain、LlamaIndex）](features/ai-agent-integration.md)

---

# 用例手册

- [RDF 索引](cookbook/index.md)
- [来自关系目录的知识图谱](cookbook/relational-to-rdf.md)
- [基于知识图谱的聊天机器人](cookbook/grounded-chatbot.md)
- [去重客户记录](cookbook/dedupe-customers.md)
- [隐私保护记录链接（PPRL）](cookbook/pprl.md)
- [使用 PROV-O 和时间查询的审计跟踪](cookbook/audit-trail.md)
- [CDC → 通过 JSON-LD Outbox 发送至 Kafka](cookbook/cdc-to-kafka.md)
- [软约束的概率规则](cookbook/probabilistic-rules.md)
- [规则库](cookbook/rule-libraries.md)
- [SPARQL 修复工作流](cookbook/sparql-repair.md)
- [本体映射和对齐](cookbook/ontology-mapping.md)
- [LLM 工作流 — NL 到知识图谱答案](cookbook/llm-workflow.md)
- [与 Wikidata 和 DBpedia 的联邦](cookbook/federation-wikidata.md)
- [SHACL + Datalog 数据质量管道](cookbook/shacl-datalog-quality.md)
- [SKOS 词库管理](cookbook/skos-thesaurus.md)
- [常用词汇包：DCTERMS、Schema.org、FOAF](cookbook/common-vocabularies.md)
- [从 Neo4j 迁移](cookbook/migrate-from-neo4j.md)
- [从头开始构建 RAG 管道](cookbook/rag-pipeline-from-scratch.md)
- [规则库联邦](cookbook/rule-federation.md)

---

# API 和集成

- [API 和集成概述](features/apis-and-integration.md)
- [SPARQL 查询调试器 — `EXPLAIN SPARQL`](user-guide/explain-sparql.md)
- [规则库联邦指南](guides/rule-library-federation.md)

---

# 运维

- [架构概述](operations/architecture.md)
- [生产就绪清单](operations/production-checklist.md)
- [部署模型](operations/deployment.md)
- [Docker（开箱即用）](operations/docker.md)
- [Kubernetes 和 Helm](operations/kubernetes.md)
- [CloudNativePG](operations/cloudnativepg.md)
- [高可用性](operations/high-availability.md)
- [逻辑复制](operations/replication.md)
- [CDC 运维](operations/cdc.md)
- [Citus + pg-trickle 集成](operations/citus-integration.md)
- [pg-tide 中继：中心辐射](operations/pg-tide-relay.md)
- [pg-trickle 中继（旧版重定向）](operations/pg-trickle-relay.md)
- [Bidi 运维 Runbook](operations/bidi-runbook.md)
- [Bidi 生产清单](operations/bidi-production-checklist.md)
- [配置](operations/configuration.md)
- [调优](operations/tuning.md)
- [监控和可观察性](operations/monitoring.md)
- [OpenTelemetry 可观察性指南](operations/observability-otel.md)
- [性能调优](operations/performance.md)
- [并行合并工作线程](operations/merge-workers.md)
- [备份和灾难恢复](operations/backup-recovery.md)
- [安全升级](operations/upgrading.md)
- [扩展](operations/scaling.md)
- [读副本](operations/read-replicas.md)
- [故障排除](operations/troubleshooting.md)
- [安全](operations/security.md)
- [兼容性矩阵](operations/compatibility.md)
- [HTTP 伴随服务运维](operations/http-companion.md)

---

# 最佳实践

- [索引](user-guide/best-practices/index.md)
- [批量加载](user-guide/best-practices/bulk-loading.md)
- [数据建模](user-guide/best-practices/data-modeling.md)
- [SPARQL 模式](user-guide/best-practices/sparql-patterns.md)
- [SPARQL 性能](user-guide/best-practices/sparql-performance.md)
- [SHACL 模式](user-guide/best-practices/shacl-patterns.md)
- [Datalog 优化](user-guide/best-practices/datalog-optimization.md)
- [更新模式](user-guide/best-practices/update-patterns.md)
- [联邦性能](user-guide/best-practices/federation-performance.md)
- [查询规划](user-guide/performance/query-planning.md)

---

# 参考

- [SQL 函数参考](reference/sql-functions.md)
- [SQL API 参考](reference/sql-api.md)
- [SQL 参考索引](user-guide/sql-reference/index.md)
- [三元组 CRUD](user-guide/sql-reference/triple-crud.md)
- [批量加载](user-guide/sql-reference/bulk-load.md)
- [SPARQL 查询](user-guide/sql-reference/sparql-query.md)
- [SPARQL 更新](user-guide/sql-reference/sparql-update.md)
- [Datalog](user-guide/sql-reference/datalog.md)
- [SHACL](user-guide/sql-reference/shacl.md)
- [RDF-star](user-guide/sql-reference/rdf-star.md)
- [命名图](user-guide/sql-reference/named-graphs.md)
- [联邦](user-guide/sql-reference/federation.md)
- [全文搜索](user-guide/sql-reference/fts.md)
- [游标 API](user-guide/sql-reference/cursor-api.md)
- [视图](user-guide/sql-reference/views.md)
- [框架视图](user-guide/sql-reference/framing-views.md)
- [序列化](user-guide/sql-reference/serialization.md)
- [HTTP 端点](user-guide/sql-reference/http-endpoint.md)
- [字典](user-guide/sql-reference/dictionary.md)
- [前缀](user-guide/sql-reference/prefix.md)
- [管理](user-guide/sql-reference/admin.md)
- [解释](user-guide/sql-reference/explain.md)
- [架构（内部）](reference/architecture.md)
- [子系统依赖图](architecture.md)
- [GUC 参考](reference/guc-reference.md)
- [GUC](reference/gucs.md)
- [已弃用的 GUC](reference/deprecated-gucs.md)
- [功能状态分类](reference/feature-status-taxonomy.md)
- [计划缓存](reference/plan-cache.md)
- [审计日志](reference/audit-log.md)
- [嵌入函数](reference/embedding-functions.md)
- [GraphRAG 函数](reference/graphrag-functions.md)
- [GraphRAG 本体](reference/graphrag-ontology.md)
- [GeoSPARQL 函数](reference/geosparql.md)
- [GeoSPARQL 函数索引](reference/geosparql-functions.md)
- [基于格的 Datalog](reference/lattice-datalog.md)
- [HTTP API 参考](reference/http-api.md)
- [限制和配额](reference/limits.md)
- [近似聚合（HLL）](reference/approximate-aggregates.md)
- [Arrow Flight 参考](reference/arrow-flight.md)
- [Citus SERVICE 分片修剪](reference/citus-service-pruning.md)
- [API 稳定性保证](reference/api-stability.md)
- [SPARQL 参考](reference/sparql.md)
- [Datalog 参考](reference/datalog.md)
- [SHACL 参考](reference/shacl.md)
- [存储参考](reference/storage.md)
- [CONSTRUCT 写回规则](reference/construct-rules.md)
- [IVM 边界：CWB vs. PageRank](reference/ivm.md)
- [联邦参考](reference/federation.md)
- [CDC 参考](reference/cdc.md)
- [GraphRAG 参考](reference/graphrag.md)
- [可观察性参考](reference/observability.md)
- [查询优化](reference/query-optimization.md)
- [向量搜索参考](reference/vector-search.md)
- [开发参考](reference/development.md)
- [SPARQL 符合性矩阵](reference/sparql-compliance.md)
- [SPARQL 1.2 属性路径状态](reference/sparql12-status.md)
- [W3C 符合性](reference/w3c-conformance.md)
- [符合性趋势](reference/conformance-trends.md)
- [Jena 结果](reference/jena-results.md)
- [OWL 2 RL 结果](reference/owl2rl-results.md)
- [WatDiv 结果](reference/watdiv-results.md)
- [LUBM 结果](reference/lubm-results.md)
- [向量索引权衡](reference/vector-index-tradeoffs.md)
- [运行符合性测试](reference/running-conformance-tests.md)
- [错误代码](reference/error-codes.md)
- [错误目录](reference/error-catalog.md)
- [可扩展性](reference/scalability.md)
- [FAQ](reference/faq.md)
- [可选功能降级](reference/degradation.md)
- [词汇表](reference/glossary.md)
- [发布说明](reference/changelog.md)
- [路线图](reference/roadmap.md)

---

# 研究

- [研究索引](research/index.md)
- [PostgreSQL 三元组存储深度探索](research/postgresql-deepdive.md)

---

# 贡献

- [如何贡献](reference/contributing.md)
- [发布流程](reference/release-process.md)

---

# 博客

- [博客索引](blog/README.md)