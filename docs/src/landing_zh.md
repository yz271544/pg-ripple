# 什么是 pg_ripple？

**pg_ripple** 将您的 PostgreSQL 数据库转变为知识图谱存储 —— 并成为需要*可验证、结构化、可追溯*答案而非虚构答案的 AI 应用程序的基础。

您可以构建从您自己的知识库回答问题的聊天机器人、跨系统对客户记录进行去重、使用形式规则验证数据质量、运行 OWL 推理，并在您已经运营的 PostgreSQL 中回答 SPARQL 查询。

```sql
-- 针对您的知识图谱询问自然语言问题。
SELECT pg_ripple.rag_context('Which drugs interact with metformin?', k := 8);
-- 返回准备好传递给 LLM 的结构化上下文。

-- 或者直接用 SPARQL 查询。
SELECT * FROM pg_ripple.sparql($$
  PREFIX ex: <https://example.org/>
  SELECT ?name WHERE {
    ex:alice <http://xmlns.com/foaf/0.1/knows>+ ?person .
    ?person  <http://xmlns.com/foaf/0.1/name>   ?name .
  }
$$);
-- 沿着 foaf:knows 关系跨越任意跳数。
```

---

## pg_ripple 解决的三个问题

### 1. 知道其所知并能证明的 AI

LLM 响应编造事实，因为模型没有可靠的事实来源。`rag_context()` 在单个 SQL 调用中为任何问题检索结构化图上下文，将 LLM 的回答扎根于您的数据中。每个检索到的事实都链接回其源三元组、PROV-O 出处和审计日志条目。

→ [AI 概述与决策树](features/ai-overview.md) | [聊天机器人配方](cookbook/grounded-chatbot.md)

### 2. 数据库速度的记录链接和去重

在 CRM 和 ERP 之间对客户记录进行去重、对齐外部标本体，或合并研究实体提及 —— 全部在一个 PostgreSQL 事务中完成。知识图谱嵌入生成候选；SHACL 硬规则阻止不安全的合并；`owl:sameAs` 规范化使合并视图对每个下游查询透明。

→ [记录链接](features/record-linkage.md) | [去重配方](cookbook/dedupe-customers.md)

### 3. 一个事务中的知识图谱 + 关系型 + 向量

没有独立的图存储、没有独立的向量索引、没有独立的 schema 注册表。一个 `pg_dump` 就可以捕获所有内容。一个 ACID 事务同时跨越三元组写入、向量索引更新和普通表更新。

→ [架构一览](evaluate/architecture-glance.md) | [与替代方案的比较](evaluate/comparison.md)

---

## 主要功能

| 功能 | 作用 |
|---|---|
| **SPARQL 1.1** | W3C 标准图查询，具有 324 个必需用例的套件 |
| **SHACL 验证** | 定义并强制执行数据质量规则 —— 在插入时拒绝坏数据 |
| **Datalog 推理** | 从 RDFS、OWL RL/EL/QL 或自定义规则推导新事实 |
| **向量 + 图混合** | SPARQL 遍历与 pgvector HNSW 相似度搜索相结合 |
| **RAG 管道** | `rag_context()` —— 一个调用即可检索结构化 LLM 上下文 |
| **NL → SPARQL** | `sparql_from_nl()` —— 从自然语言问题自动生成 SPARQL |
| **记录链接** | KGE 嵌入 + SHACL 门控 + `owl:sameAs` 规范化 |
| **GraphRAG** | Microsoft GraphRAG 摄取、增强、验证、导出管道 |
| **联邦** | 在本地数据旁边查询 Wikidata、DBpedia 和其他 SPARQL 端点 |
| **R2RML** | 从任何现有 PostgreSQL schema 生成 RDF 图 |
| **JSON-LD 框架** | 导出根据您的 API 契约成形的嵌套 JSON 文档 |
| **SPARQL 协议** | 通过 `pg_ripple_http` 的标准 HTTP 端点 |

### 关键数字

| 指标 | 数值 |
|---|---|
| 当前性能工件 | 有界 100,000 三元组合成工作负载 |
| BSBM 执行门控 | 适配的装置，约一百万个三元组 |
| W3C SPARQL 1.1 | 324 / 324 锁定用例通过 |
| LUBM | 14 / 14 必需查询通过 |
| W3C OWL 2 RL | 仅源采集；无当前通过率 |
| PostgreSQL 版本 | 18 |

---

## 架构一览

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

每个 IRI、字面量和空白节点都由字典编码器映射到一个紧凑的整数 ID。数据存储在垂直分区 (VP) 表中 —— 每个谓词一张表 —— 只使用整数连接以实现快速查询执行。HTAP 架构分离了读和写路径，因此繁重的负载不会阻塞查询。

---

## 从这里开始 —— 选择您的路径

| 您的角色 | 从这里开始 | 然后探索 | 更深入 |
|---|---|---|---|
| **PostgreSQL DBA** —— 您了解 PostgreSQL，但 RDF 是新的 | [安装](getting-started/installation.md) → [Hello World](getting-started/hello-world.md) | [关键概念（面向 PG 用户的 RDF）](getting-started/key-concepts.md) | [运维](operations/configuration.md)，[调优](operations/tuning.md) |
| **数据 / AI 工程师** —— 构建 RAG 或知识管道 | [AI 概述](features/ai-overview.md) → [基于知识的聊天机器人](cookbook/grounded-chatbot.md) | [向量 + 混合搜索](features/vector-and-hybrid-search.md) | [GraphRAG](features/graphrag.md)，[NL-to-SPARQL](features/nl-to-sparql.md) |
| **语义 Web / RDF 工程师** —— 您了解 SPARQL 和 OWL | [SPARQL 功能](features/querying-with-sparql.md) → [Datalog](features/reasoning-and-inference.md) | [SHACL 验证](features/validating-data-quality.md) | [联邦](features/apis-and-integration.md)，[OWL Profiles](features/owl-profiles.md) |

---

## 下一步

- **评估？** [何时使用 pg_ripple](evaluate/when-to-use.md)、[与替代方案的比较](evaluate/comparison.md) 和 [性能结果](evaluate/performance-results.md) 提供了您所需的内容。
- **准备安装？** 从 [安装](getting-started/installation.md) 开始，然后 [五分钟 Hello World](getting-started/hello-world.md)。
- **正在构建 RAG 或 AI 应用程序？** 阅读 [AI 概述](features/ai-overview.md) 和 [基于知识的聊天机器人配方](cookbook/grounded-chatbot.md)。
- **从关系数据库或 Neo4j 迁移？** 请参阅 [R2RML](features/r2rml.md)（关系型 → RDF）和 [LPG/Cypher 映射](features/lpg-mapping.md)。
- **正在去重记录？** 请参阅 [记录链接](features/record-linkage.md) 和 [去重配方](cookbook/dedupe-customers.md)。
- **想要完整的图景？** [引导式教程](getting-started/tutorial.md) 在 30 分钟内带您从加载数据到推理。
- **想要贡献？** 请参阅 [贡献](reference/contributing.md)。