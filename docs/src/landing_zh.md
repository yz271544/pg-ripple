# 什么是 pg_ripple？

**pg_ripple** 将您的 PostgreSQL 数据库转变为知识图谱存储 — 并转变为需要*可验证、结构化、可追溯*答案而非虚构答案的 AI 应用程序的基础。

您可以构建一个聊天机器人，从您自己的知识库回答问题，跨系统对客户记录进行重复数据消除，使用形式化的规则验证数据质量，运行 OWL 推理，并在您已运营的 PostgreSQL 内部回答 SPARQL 查询。

```sql
-- 针对您的知识图谱提出自然语言问题。
SELECT pg_ripple.rag_context('Which drugs interact with metformin?', k := 8);
-- 返回准备传递给 LLM 的结构化上下文。

-- 或直接在 SPARQL 中查询。
SELECT * FROM pg_ripple.sparql($$
  PREFIX ex: <https://example.org/>
  SELECT ?name WHERE {
    ex:alice <http://xmlns.com/foaf/0.1/knows>+ ?person .
    ?person  <http://xmlns.com/foaf/0.1/name>   ?name .
  }
$$);
-- 通过任意数量的跳数跟踪 foaf:knows 关系。
```

---

## pg_ripple 解决的三个问题

### 1. 知道自己所知的 AI — 并且能够证明它

LLM 响应捏造事实，因为模型没有可靠的真相来源。`rag_context()` 在单个 SQL 调用中为任何问题检索结构化图上下文，将 LLM 的答案建立在您的数据基础上。每个检索到的事实都链接回其源三元组、PROV-O 出处和审计日志条目。

→ [AI 概述和决策树](features/ai-overview.md) | [聊天机器人配方](cookbook/grounded-chatbot.md)

### 2. 数据库速度的记录链接和重复数据消除

跨 CRM 和 ERP 对客户记录进行重复数据消除，对齐外部本体，或合并研究实体提及 — 所有操作都在一个 PostgreSQL 事务中完成。知识图谱嵌入生成候选者；SHACL 硬规则阻止不安全的合并；`owl:sameAs` 规范化使合并后的视图对每个下游查询透明。

→ [记录链接](features/record-linkage.md) | [重复数据消除配方](cookbook/dedupe-customers.md)

### 3. 一个事务中的知识图谱 + 关系型 + 向量

没有单独的图存储，没有单独的向量索引，没有单独的 schema 注册表。一个 `pg_dump` 捕获所有内容。一个 ACID 事务同时跨越三元组写入、向量索引更新和普通表更新。

→ [架构概览](evaluate/architecture-glance.md) | [与替代方案的对比](evaluate/comparison.md)

---

## 关键能力

| 能力 | 它做什么 |
|---|---|
| **SPARQL 1.1** | W3C 标准图查询，包含必需的 324 案例套件 |
| **SHACL 验证** | 定义和强制执行数据质量规则 — 在插入时拒绝错误数据 |
| **Datalog 推理** | 从 RDFS、OWL RL/EL/QL 或自定义规则派生新事实 |
| **向量 + 图混合** | SPARQL 遍历与 pgvector HNSW 相似性搜索结合 |
| **RAG 管道** | `rag_context()` — 一个调用检索结构化 LLM 上下文 |
| **NL → SPARQL** | `sparql_from_nl()` — 从自然语言问题自动生成 SPARQL |
| **记录链接** | KGE 嵌入 + SHACL 网关 + `owl:sameAs` 规范化 |
| **GraphRAG** | Microsoft GraphRAG 摄取、丰富、验证、导出管道 |
| **联邦** | 与 Wikidata、DBpedia 和其他 SPARQL 端点以及本地数据一起查询 |
| **R2RML** | 从任何现有 PostgreSQL schema 生成 RDF 图 |
| **JSON-LD 框架** | 导出嵌套 JSON 文档，形状符合您的 API 合约 |
| **SPARQL 协议** | 通过 `pg_ripple_http` 的标准 HTTP 端点 |

### 关键数据

| 指标 | 值 |
|---|---|
| 当前性能工件 | 有界 100,000 三元组合成工作负载 |
| BSBM 执行门槛 | 适配的固定装置，约 100 万三元组 |
| W3C SPARQL 1.1 | 324 / 324 锁定案例通过 |
| LUBM | 14 / 14 必需查询通过 |
| W3C OWL 2 RL | 仅来源获取；无当前通过率 |
| PostgreSQL 版本 | 18 |

---

## 架构概览

```
┌─────────────────────────────────────────────────┐
│                  PostgreSQL 18                   │
│  ┌───────────────────────────────────────────┐  │
│  │              pg_ripple extension           │  │
│  │  ┌─────────┐  ┌────────┐  ┌───────────┐  │  │
│  │  │Dictionary│  │ SPARQL │  │  Datalog   │  │  │
│  │  │ Encoder  │  │ Engine │  │  Engine    │  │  │
│  │  └────┬─────┘  └───┬────┘  └─────┬─────┘  │  │
│  │       │             │             │         │  │
│  │  ┌────┴─────────────┴─────────────┴─────┐  │  │
│  │  │     VP Tables (one per predicate)     │  │  │
│  │  │   HTAP: delta + main + merge worker   │  │  │
│  │  └──────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
         ▲                          ▲
         │ SQL                      │ HTTP
    Application              pg_ripple_http
```

每个 IRI、字面量和空节点由字典编码器映射为紧凑的整数 ID。数据存储在垂直分区（VP）表中 — 每个唯一谓词一个表 — 仅使用整数连接以实现快速查询执行。HTAP 架构分离读写路径，以便大量加载不会阻塞查询。

---

## 从这里开始 — 选择您的路径

| 您的角色 | 从这里开始 | 然后探索 | 更深入 |
|---|---|---|---|
| **PostgreSQL DBA** — 您了解 PostgreSQL，对 RDF 新手 | [安装](getting-started/installation.md) → [Hello World](getting-started/hello-world.md) | [关键概念（PG 用户的 RDF）](getting-started/key-concepts.md) | [运维](operations/configuration.md)、[调优](operations/tuning.md) |
| **数据 / AI 工程师** — 构建 RAG 或知识管道 | [AI 概述](features/ai-overview.md) → [基于知识图谱的聊天机器人](cookbook/grounded-chatbot.md) | [向量和混合搜索](features/vector-and-hybrid-search.md) | [GraphRAG](features/graphrag.md)、[NL 到 SPARQL](features/nl-to-sparql.md) |
| **语义网 / RDF 工程师** — 您了解 SPARQL 和 OWL | [SPARQL 功能](features/querying-with-sparql.md) → [Datalog](features/reasoning-and-inference.md) | [SHACL 验证](features/validating-data-quality.md) | [联邦](features/apis-and-integration.md)、[OWL Profiles](features/owl-profiles.md) |

---

## 下一步

- **评估中？** [何时使用 pg_ripple](evaluate/when-to-use.md)、[与替代方案的对比](evaluate/comparison.md)和[性能结果](evaluate/performance-results.md)包含您所需的内容。
- **准备安装？** 从[安装](getting-started/installation.md)开始，然后[五分钟 Hello World](getting-started/hello-world.md)。
- **构建 RAG 或 AI 应用程序？** 阅读 [AI 概述](features/ai-overview.md)和[基于知识图谱的聊天机器人配方](cookbook/grounded-chatbot.md)。
- **从关系数据库或 Neo4j 迁移？** 参阅 [R2RML](features/r2rml.md)（关系型 → RDF）和 [LPG/Cypher 映射](features/lpg-mapping.md)。
- **正在对记录进行重复数据消除？** 参阅[记录链接](features/record-linkage.md)和[重复数据消除配方](cookbook/dedupe-customers.md)。
- **想要完整的图景？** [引导式教程](getting-started/tutorial.md)在 30 分钟内从加载数据到推理。
- **想要贡献？** 参阅[贡献](reference/contributing.md)。