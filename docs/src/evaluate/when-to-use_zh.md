# 何时使用 pg_ripple

pg_ripple 是一个 PostgreSQL 扩展，可将您的数据库转变为知识图谱存储。此页面可帮助您确定它是否适合您的架构。

## 决策流程图

按顺序问自己这些问题：

1. **您是否已经运行 PostgreSQL？** 如果是，则 pg_ripple 与数据存储的零额外基础设施集成。如果您运行的是不同的数据库，请评估迁移成本。
2. **您需要建模复杂关系吗？** 如果您的数据主要是表格型的且连接很少，则标准 SQL 可能更简单。如果您有深度嵌套、多对多或分层关系，则图模型会有所帮助。
3. **您需要标准查询语言吗？** SPARQL 是具有广泛工具支持的 W3C 标准。如果您更喜欢属性图查询语言（Cypher/GQL），请考虑 Neo4j 或 Amazon Neptune。
4. **您需要推理或验证吗？** pg_ripple 包括 SHACL 验证和 Datalog 推理。像 Virtuoso 或 Blazegraph 这样的独立三元组存储可能不包括。
5. **您需要 LLM 提示的图上下文吗？** pg_ripple 在单个查询中将 SPARQL 图遍历与 pgvector 相似度搜索相结合 —— 这是纯向量数据库无法做到的。

## 比较矩阵

| 条件 | pg_ripple | 普通 SQL | Virtuoso / Blazegraph | Neo4j | 纯向量 DB |
|---|---|---|---|---|---|
| 部署 | PostgreSQL 扩展 | 任何 RDBMS | 独立 JVM | 独立 | 独立 |
| 查询语言 | SPARQL 1.1 | SQL | SPARQL 1.1 | Cypher | 专有 |
| 数据模型 | RDF（三元组） | 关系 | RDF（三元组） | 属性图 | 向量 + 元数据 |
| Schema 验证 | SHACL | CHECK / 触发器 | 各异 | Varies | 无 |
| 推理 | Datalog（RDFS、OWL RL） | 手动 SQL | RDFS / OWL（各异） | 无内置 | 无 |
| 向量搜索 | pgvector 集成 | pgvector | 非内置 | 有限 | 原生 |
| 混合图+向量 | 是（单个查询） | 手动连接 | 否 | 否 | 否 |
| HTTP API | pg_ripple_http | 构建您自己的 | 内置 | 内置 | 内置 |
| 事务 | 完整 PostgreSQL ACID | 完整 ACID | 各异 | ACID | 各异 |
| 备份/恢复 | pg_dump/pg_restore | 标准 | 自定义工具 | 自定义工具 | 自定义工具 |
| 运维复杂性 | 低（PostgreSQL） | 低 | 中–高 | 中 | 中 |

## pg_ripple 是合适选择的情况

- 您已经运营 PostgreSQL，并希望避免管理单独的图数据库
- 您的数据具有丰富的互联关系（本体、目录、供应链）
- 您需要 SPARQL 1.1 合规性以与 W3C 标准工具互操作
- 您需要根据形式规则验证数据质量（SHACL）
- 您需要从现有数据派生新事实（Datalog 推理、OWL RL、RDFS）
- 您希望将图遍历与向量相似度相结合用于 RAG 管道
- 您需要对图数据进行完整 ACID 事务

## pg_ripple 不是最佳选择的情况

- **没有项目特定验证运行的大型或分布式数据集**：存储库不包含当前生产规模结果。基准测试您的固定版本或选择具有目标规模证据的分布式系统。
- **具有 Cypher/GQL 的属性图**：如果您的团队已经使用 Cypher 和 Neo4j，迁移到 SPARQL 具有学习曲线。pg_ripple 说 SPARQL，不说 Cypher。
- **纯向量搜索工作负载**：如果您只需要没有图遍历的近似最近邻搜索，则单独的 pgvector 更简单。
- **实时流式图**：pg_ripple 在事务中处理数据，而不是连续流。对于流式图分析，请考虑使用图库的 Apache Flink。
- **您的堆栈中没有 PostgreSQL**：如果您运行 MySQL、MongoDB 或托管 NoSQL 服务，并且没有计划采用 PostgreSQL，则仅为 pg_ripple 引入它会增加运维开销。

## AI/LLM 比较：何时图上下文优于扁平向量检索？

图增强检索在以下情况下有帮助：

- 查询需要**多跳推理** —— "查找 Alice 的合作者的合作者的论文"不能仅通过向量相似度来回答
- **实体去重**很重要 —— `owl:sameAs` 规范化确保同一实体不会使用不同的 IRI 多次嵌入
- 需要**结构化输出** —— JSON-LD 框架生成令牌高效、结构化的上下文，这是扁平 top-k 结果无法提供的
- **出处**很重要 —— 图遍历可以追溯某个事实为何相关，而不仅仅是它相似

纯向量搜索（Qdrant、Weaviate、仅 pgvector）在以下情况下就足够了：

- 查询是简单的"查找相似文档"，没有关系约束
- 您的语料库是没有实体级结构的非结构化文本
- 延迟要求在数百万向量时为亚毫秒级

## 下一步

- **安装**（../getting-started/installation.md）— 让 pg_ripple 跑起来
- **Hello World**（../getting-started/hello-world.md）— 在五分钟内加载和查询数据