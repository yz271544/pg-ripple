# 用例 Cookbook

**功能深入探讨**章节下的章节解释了*每个 pg_ripple 功能的作用*。本 cookbook 中的配方解释了*链接在一起的功能可以做什么*。每个配方都是一个独立的故事：一个真实的目标、分步骤的 SQL 以及需要注意的权衡。

如果您正在评估 pg_ripple，请从这里开始 —— 这些模式决定了该技术是否适合您的问题。

| 配方 | 您构建的内容 | 使用的功能 |
|---|---|---|
| [从关系目录构建知识图谱](relational-to-rdf.md) | 从现有 PostgreSQL 表生成的可查询 RDF 图，经过验证并保持同步 | R2RML、SHACL、命名图 |
| [基于知识图谱的聊天机器人](grounded-chatbot.md) | 使用您的图作为权威上下文回答问题的 LLM 应用程序 | RAG 管道、NL→SPARQL、JSON-LD 框架 |
| [跨系统对客户记录进行去重](dedupe-customers.md) | 安全、可审计的记录链接管道，合并来自两个或多个来源的客户行 | KGE、suggest_sameas、SHACL 硬规则、owl:sameAs |
| [使用 PROV-O 和时序查询的审计跟踪](audit-trail.md) | 可被监管机构接受的链，显示系统何时以及为什么告诉用户某些信息 | PROV-O、审计日志、point_in_time、RDF-star |
| [通过 JSON-LD outbox 的 CDC → Kafka](cdc-to-kafka.md) | 准备好推送到 Kafka、NatS 或任何事件总线的结构化图变更事件流 | CDC 订阅、JSON-LD 框架、事务性 outbox |
| [用于软约束的概率规则](probabilistic-rules.md) | 传播置信度值（而不仅仅是事实）的评分规则集 | 网格 Datalog、RDF-star 置信度三元组 |
| [SPARQL 修复工作流](sparql-repair.md) | 使用 LLM 修复无法解析或未返回结果的查询的迭代循环 | sparql_from_nl、explain_sparql、错误目录 |
| [本体映射与对齐](ontology-mapping.md) | 使用 KGE 和 SHACL 将外部词汇提升到本地 schema 的管道 | KGE、suggest_sameas、OWL profiles |