# 比较：pg_ripple 与替代方案

公正的并排比较。选择适合您的团队和约束的技术，而不是博客最响亮的那种。

---

## 一目了然

|  | **pg_ripple** | 普通 PostgreSQL | Virtuoso / Blazegraph / GraphDB | Neo4j / Memgraph | Pinecone / Weaviate / Qdrant |
|---|---|---|---|---|---|
| 部署 | PostgreSQL 扩展 | 您已运行的 RDBMS | 独立（JVM/原生） | 独立 | 独立 / SaaS |
| 查询语言 | SPARQL 1.1 | SQL | SPARQL 1.1 | Cypher / GQL | 专有 |
| 数据模型 | RDF（三元组） | 关系 | RDF（三元组） | 属性图 | 向量 + 元数据 |
| 验证 | SHACL（完整 Core + SPARQL） | CHECK / 触发器 | 各异 | 约束 | 无 |
| 推理 | Datalog（RDFS、OWL RL/EL/QL、格、WFS、magic sets） | 手动 SQL 或无 | RDFS / OWL（各异） | 无内置 | 无 |
| 向量搜索 | pgvector + KGE | pgvector | 插件或无 | 有限 | 原生，仅此目的 |
| 混合图 + 向量 | 一个 SQL 查询 | 手动连接 | 插件解决方法 | 有限 | 不可能 |
| ACID | 完整 PostgreSQL ACID | 完整 ACID | 各异（通常是最终一致性） | ACID | 各异 |
| 备份 / 恢复 | `pg_dump` / `pg_restore` | 标准 | 自定义工具 | 自定义工具 | 自定义工具 |
| HTTP / SPARQL 协议 | `pg_ripple_http` | 无 | 内置 | 内置（通过 HTTP 的 Cypher） | REST |
| 联邦 | SPARQL `SERVICE` + 向量联邦 | 无 | SPARQL `SERVICE` | 无 | 一些 |
| 运维专业知识 | 转移 PostgreSQL DBA 技能 | PostgreSQL DBA | 专业三元组存储运维 | 专业图运维 | 供应商特定 |
| 标准证据 | 324 / 324 锁定 W3C SPARQL 案例；14 / 14 LUBM 查询；OWL 语料库源检查 | n/a | 各异 | n/a | n/a |

---

## pg_ripple 是显而易见的选择的情况

- 您**已经运营 PostgreSQL**，并希望避免运行第二个数据库。
- 您的数据具有**丰富的互联关系** —— 本体、供应链、组织层次结构、引用网络。
- 您需要 **SPARQL 1.1** 以与 W3C 标准工具互操作。
- 您需要**根据形式规则验证数据质量**（SHACL）。
- 您需要**从现有数据派生新事实**（Datalog、OWL RL）。
- 您希望将**图遍历与向量相似度**相结合，用于 RAG、推荐或记录链接。
- 您需要涵盖图数据、向量数据和普通关系数据的**事务保证**。

---

## pg_ripple *不是*正确答案的情况

| 情况 | 更好的选择 |
|---|---|
| 今日需要多节点生产规模 | 具有生产证据的分布式存储；pg_ripple Citus 支持是实验性的 |
| 现有的 Cypher / GQL 代码库，不打算学习 SPARQL | Neo4j / Memgraph |
| 纯向量搜索，无图遍历 | 仅 pgvector，或者如果需要托管服务则使用 Pinecone/Qdrant |
| 仅追加事件流上的流式图分析 | Apache Flink + 图库 |
| 您不在任何地方运行 PostgreSQL，也没有计划运行 | 选择与您工作区天然兼容的工具 |
| 仅限 SQL，对 RDF 过敏 | 通过精心设计的 schema 设计的普通 PostgreSQL |

---

## 一个具体的比较：用于 RAG 的混合检索

这是当今大多数团队关心的比较。

|  | pg_ripple | 仅向量 DB | 仅图 DB |
|---|---|---|---|
| 自由文本问题 → 相似实体 | 原生 | 原生 | 手动 |
| 多跳关系遍历 | 原生（SPARQL 属性路径） | 不可能 | 原生（Cypher） |
| 组合混合查询 | 一次 SQL 调用 | 应用中的胶水代码 | 应用中的胶水代码 |
| 三元组 + 嵌入的原子写入 | 是（一个事务） | 否 | 否 |
| 审计 + 来源 | PROV-O + 审计日志 | 无 | 自定义 |
| 多租户隔离 | 图 RLS + 配额 | 按命名空间、按层 | 按数据库 |
| 运维足迹 | 一个 PostgreSQL | 两个系统 | 两个系统 |
| 您不再需要的额外向量存储的成本 | $0 | $$$$ | $$$$ |

主导权衡：当您的*唯一*任务是"查找相似段落"时，向量 DB 更简单。一旦您还需要精确的关系、出处、多跳推理或事务一致性，缝合两个系统的成本很快就会超过运行 pg_ripple 的成本。

---

## 另请参阅

- [何时使用 pg_ripple](when-to-use.md)
- [架构一览](architecture-glance.md)
- [性能和一致性结果](performance-results.md)