# AI、RAG 和 LLM 集成 — 概述

pg_ripple 是少数几个在**单个事务**中将知识图、向量搜索和大型语言模型结合在一起的 PostgreSQL 扩展之一。本页是该功能的前门。它解释*每个 AI 功能的作用*、*何时使用哪个*以及*接下来要阅读哪个深入页面*。

如果您只有五分钟，请阅读本页。如果您有一个小时，请遵循下面的链接到各个章节。

---

## 为什么选择 pg_ripple 用于 AI 工作负载？

现代检索管道落入三个陷阱之一：

1. **纯向量搜索**擅长模糊相似性（"看起来像 X 的事物"），但无法回答遵循精确关系的问题（"哪些药物与我的病人正在服用的药物相互作用？"）。
2. **纯图查询**精确捕获关系，但无法将自由文本问题匹配到正确的实体。
3. **将两者缝合在一起的管道**（向量数据库 + 图数据库 + 粘合代码）在系统之间泄漏数据，受同步滞后影响，并且无法原子回滚。

pg_ripple 消除了第三个陷阱。向量存储在 `pgvector` 中，三元组存储在垂直分区表中，并且单个 SQL 事务可以同时更新两者。本页上的每个 AI 功能都建立在此基础之上。

---

## 五个 AI 功能一览

| 功能 | 它回答的问题 | 接下来阅读 |
|---|---|---|
| **混合搜索** | "查找看起来像 *X* **并且**满足此图模式的实体。" | [向量和混合搜索](vector-and-hybrid-search.md) |
| **RAG 管道** | "构建一个上下文块，我可以放入 LLM 提示中。" | [RAG 管道](../user-guide/rag-pipeline.md) |
| **自然语言 → SPARQL** | "将此英语问题转换为我可以运行的 SPARQL 查询。" | [NL → SPARQL](nl-to-sparql.md) |
| **知识图嵌入（KGE）** | "从*图结构本身*学习实体向量，而不是从文本。" | [知识图嵌入](knowledge-graph-embeddings.md) |
| **记录链接 / 实体解析** | "查找引用同一现实事物的实体对并安全地合并它们。" | [记录链接](record-linkage.md) |

这些功能是可组合的。一个典型的生产管道同时使用其中三个或四个 — 请参阅[用例 Cookbook](../cookbook/index.md) 获取端到端配方。

---

## 决策树

使用此树为新项目选择正确的功能。

```
您需要在数据上回答自由文本问题吗？
├─ 否  → 您可能不需要任何 AI 功能。使用普通 SPARQL。
└─ 是 → 该问题命名了特定实体，还是描述了一个？
         ├─ 命名它    → 普通 SPARQL 最快。（例如"显示 Alice 的论文"）
         └─ 描述它 → 您需要检索。继续。
                          │
                          答案是一个事实，还是一段上下文？
                          ├─ 事实      → 使用 NL → SPARQL。
                          └─ 上下文   → 使用 RAG 管道（rag_context()）。
                                         │
                                         您的查询是否需要在相似性之上具有精确关系？
                                         ├─ 否  → 混合搜索可能就足够了。
                                         └─ 是 → 在一个查询中组合 SPARQL + similar()。

您是否有来自多个可能重叠的来源的实体？
                               └─ 是 → 您需要记录链接。从 KGE 开始进行候选生成，
                                        然后使用 SHACL 作为硬规则，然后 suggest_sameas + apply。

您现有的嵌入是否仅捕获文本而忽略关系？
└─ 是 → 训练知识图嵌入（TransE / RotatE）并将它们用于实体对齐、推荐或链接预测。
```

---

## 各部分如何组合在一起

```
                ┌─────────────────────────────────────┐
                │            应用程序                  │
                └──────────────┬──────────────────────┘
                               │ SQL
                ┌──────────────┴──────────────────────┐
                │            pg_ripple                │
                │                                     │
   文本查询 →   │  rag_context()                      │ ← LLM 提示
                │     ├─ 嵌入问题（HTTP）             │
                │     ├─ HNSW 向量召回  ───────────────┼─→ pgvector
                │     ├─ SPARQL 图扩展 ────────────────┼─→ VP 表
                │     └─ 组装 JSON-LD                 │
                │                                     │
   文本查询 →   │  sparql_from_nl()                   │ ← SPARQL 字符串
                │     ├─ 构建 VoID + SHACL 上下文     │
                │     ├─ LLM /v1/chat/completions     │
                │     └─ 解析 + 验证（spargebra）     │
                │                                     │
   批处理运行 → │  embed_entities()  ─────────────────┼─→ pgvector
   批处理运行 → │  kge_train()       ─────────────────┼─→ kge_embeddings
   批处理运行 → │  suggest_sameas()  ─────────────────┼─→ owl:sameAs 三元组
                └─────────────────────────────────────┘
                               │
                ┌──────────────┴──────────────────────┐
                │   PostgreSQL 事务边界                │
                └─────────────────────────────────────┘
```

虚线框内的所有内容都在单个 PostgreSQL 事务中运行。如果出现任何故障，整个管道都会回滚 — 没有半更新的向量存储需要清理。

---

## 先决条件

| 要求 | 需要 |
|---|---|
| `CREATE EXTENSION vector;`（pgvector）| 除 NL → SPARQL 之外的所有功能 |
| `pg_ripple.embedding_api_url` 已配置 | RAG、embed_entities、通过文本输入的混合搜索 |
| `pg_ripple.llm_endpoint` 已配置 | NL → SPARQL，`rag_context()` 的可选第二阶段 |
| 环境变量中的 API 密钥 | LLM 和嵌入端点 — 密钥**永远不**存储在数据库中 |
| `pg_ripple.kge_enabled = on` | KGE 训练和 `find_alignments()` |

当它们的依赖项缺失时，所有 AI 功能都会优雅降级 — 它们发出 `WARNING` 并返回零行而不是引发 `ERROR`。您可以将*使用*这些功能的代码部署到没有配置 LLM 端点的 CI 环境中。

---

## 安全注意事项

- **API 密钥永远不存储在 PostgreSQL 中。** 配置环境变量的*名称*（例如 `pg_ripple.llm_api_key_env = 'OPENAI_API_KEY'`），扩展在调用时读取密钥。
- **出站 HTTP 调用需要列入白名单的端点。** LLM 端点（通过 `llm_endpoint` / `embedding_api_url` 注册）和联合 SPARQL 服务（通过 `register_endpoint()` 注册）都会在每次调用时根据白名单进行检查。这可以防止服务器端请求伪造（SSRF）。
- **提示中的 PII。** `rag_context()` 和 `sparql_from_nl()` 将图摘录发送到配置的 LLM。使用命名图行级安全性（请参阅[多租户图](multi-tenant-graphs.md)）将租户数据从您不控制的提示中排除。

---

## 接下来去哪里

- **想在五分钟内试用？** [Cookbook：基于知识图的聊天机器人](../cookbook/grounded-chatbot.md)
- **已有知识图并想添加 RAG？** [RAG 管道](../user-guide/rag-pipeline.md)
- **合并来自多个系统的客户数据？** [记录链接](record-linkage.md)
- **在图上构建推荐引擎？** [知识图嵌入](knowledge-graph-embeddings.md)
- **需要 OpenAI 结构化输出的结构化提示？** [导出和共享 — JSON-LD 框架](exporting-and-sharing.md)