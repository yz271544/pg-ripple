# 用例 Cookbook

**功能深度剖析**下的章节解释*每个 pg_ripple 功能做什么*。本 cookbook 中的食谱解释*您可以使用链接在一起的功能做什么*。每个食谱都是一个独立的故事：一个真实的目标，逐步的 SQL，以及要注意的权衡。

如果您正在评估 pg_ripple，请从这里开始 — 这些是决定技术是否适合您问题的模式。

| 食谱 | 您构建的内容 | 使用的功能 |
|---|---|---|
| [从关系目录生成知识图](relational-to-rdf.md) | 从现有 PostgreSQL 表生成的可查询 RDF 图，已验证并保持同步 | R2RML、SHACL、命名图 |
| [以知识图为依据的聊天机器人](grounded-chatbot.md) | 使用您的图作为权威上下文来回答问题的 LLM 应用程序 | RAG 管道、NL→SPARQL、JSON-LD 框架 |
| [跨系统去重客户记录](dedupe-customers.md) | 合并来自两个或多个来源的客户行的安全、可审计的记录链接管道 | KGE、suggest_sameas、SHACL 硬规则、owl:sameAs |
| [使用 PROV-O 和时间查询的审计跟踪](audit-trail.md) | 一个可监管显示的链，显示系统何时以及为什么告诉用户什么 | PROV-O、审计日志、point_in_time、RDF-star |
| [通过 JSON-LD outbox 的 CDC → Kafka](cdc-to-kafka.md) | 一系列结构化图变更事件，准备推送到 Kafka、NATS 或任何事件总线 | CDC 订阅、JSON-LD 框架、事务性 outbox |
| [软约束的概率规则](probabilistic-rules.md) | 传播置信值的评分规则集，不仅仅仅是事实 | Lattice Datalog、RDF-star 置信三元组 |
| [SPARQL 修复工作流](sparql-repair.md) | 一个迭代循环，使用 LLM 生成查询失败或返回无结果 | sparql_from_nl、explain_sparql、错误目录 |
| [本体映射和对齐](ontology-mapping.md) | 使用 KGE 和 SHACL 将外部词汇表提升到本地模式的管道 | KGE、suggest_sameas、OWL profiles |