# AI、RAG 和 LLM 集成 —— 概述

pg_ripple 是少数几个在**单个事务中**将**知识图谱、向量搜索和大语言模型结合在一起**的 PostgreSQL 扩展之一。本页是通往该功能的前门。它解释了*每个 AI 功能做什么*、*何时使用哪一个*以及*接下来阅读哪个深入页面*。

如果您只有五分钟时间，请阅读此页。如果您有一个小时，请遵循下面的章节链接。

---

## 为什么选择 pg_ripple 用于 AI 工作负载？

现代检索管道会陷入以下三个陷阱之一：

1. **纯向量搜索**擅长模糊相似性（"看起来像 X 的事物"），但无法回答遵循精确关系的问题（"哪些药物与我的患者正在服用的药物相互作用？"）。
2. **纯图查询**可以精确捕获关系，但无法将自由文本问题匹配到正确的实体。
3. **将两者拼接在一起的管道**（向量数据库 + 图数据库 + 胶水代码）在系统之间泄漏数据，存在同步滞后，并且无法原子回滚。

pg_ripple 消除了第三个陷阱。向量存储在 `pgvector` 中，三元组存储在垂直分区表中，一个 SQL 事务可以同时更新两者。此页面上的每个 AI 功能都构建在该基础之上。

---

## 五个 AI 功能一览

| 功能 | 它回答的问题 | 接下来阅读 |
|---|---|---|
| **混合搜索** | "查找看起来像 *X* **且**满足此图模式的实体。" | [向量与混合搜索](vector-and-hybrid-search.md) |
| **RAG 管道** | "构建一个可以放入 LLM 提示的上下文块。" | [RAG 管道](../user-guide/rag-pipeline.md) |
| **自然语言 → SPARQL** | "将此英语问题翻译成我可以运行的 SPARQL 查询。" | [NL → SPARQL](nl-to-sparql.md) |
| **知识图谱嵌入（KGE）** | "从*图结构本身*（而不是文本）学习实体向量。" | [知识图谱嵌入](knowledge-graph-embeddings.md) |
| **记录链接 / 实体解析** | "查找引用同一现实世界事物的实体对并安全地合并它们。" | [记录链接](record-linkage.md) |

这些功能可以组合使用。典型的生产管道会一起使用三到四个 —— 请参阅[用例 Cookbook](../cookbook/index.md) 获取端到端配方。

---

## 决策树

使用此树为新项目选择正确的功能。

```
您需要根据数据回答自由文本问题吗？
├─ 否  → 您可能不需要任何 AI 功能。使用普通 SPARQL。
└─ 是 → 问题命名特定实体还是描述一个？
         ├─ 命名它  → 普通 SPARQL 最快。（例如"显示 Alice 的论文"）
         └─ 描述它  → 您需要检索。继续。
                          │
                          答案是一个事实还是一段上下文？
                          ├─ 事实   → 使用 NL → SPARQL。
                          └─ 上下文 → 使用 RAG 管道 (rag_context())。
                                         │
                                         您的查询在相似性之上需要精确关系吗？
                                         ├─ 否  → 混合搜索可能就足够了。
                                         └─ 是 → 在一个查询中组合 SPARQL + similar()。

您是否有来自多个来源的可能重叠的实体？
└─ 是 → 您需要记录链接。从 KGE 开始进行候选生成，
         然后使用 SHACL 作为硬规则，然后是 suggest_sameas + apply。

您现有的嵌入是否仅捕获文本而忽略关系？
└─ 是 → 训练知识图谱嵌入（TransE / RotatE）并将它们用于
         实体对齐、推荐或链接预测。
```

---

## 这些部分如何组合在一起

```
                ┌─────────────────────────────────────┐
                │            应用程序                   │
                └──────────────┬──────────────────────┘
                               │ SQL
                ┌──────────────┴──────────────────────┐
                │            pg_ripple                │
                │                                     │
   文本查询 →  │  rag_context()                      │ ← LLM 提示
                │     ├─ 嵌入问题（HTTP）              │
                │     ├─ HNSW 向量召回  ──────────────┼─→ pgvector
                │     ├─ SPARQL 图扩展  ──────────────┼─→ VP 表
                │     └─ 组装 JSON-LD                 │
                │                                     │
   文本查询 →  │  sparql_from_nl()                   │ ← SPARQL 字符串
                │     ├─ 构建 VoID + SHACL 上下文       │
                │     ├─ LLM /v1/chat/completions     │
                │     └─ 解析 + 验证（spargebra）       │
                │                                     │
   批处理运行 → │  embed_entities()  ─────────────────┼─→ pgvector
   批处理运行 → │  kge_train()       ─────────────────┼─→ kge_embeddings
   批处理运行 → │  suggest_sameas()  ─────────────────┼─→ owl:sameAs 三元组
                └─────────────────────────────────────┘
                               │
                ┌──────────────┴──────────────────────┐
                │   PostgreSQL 事务边界                  │
                └─────────────────────────────────────┘
```

虚线框内的所有内容都在单个 PostgreSQL 事务内运行。如果任何操作失败，整个管道都会回滚 —— 没有需要清理的半更新向量存储。

---

## 先决条件

| 要求 | 需要者 |
|---|---|
| `CREATE EXTENSION vector;`（pgvector） | 除 NL → SPARQL 之外的所有功能 |
| 配置 `pg_ripple.embedding_api_url` | RAG、embed_entities、通过文本输入的混合搜索 |
| 配置 `pg_ripple.llm_endpoint` | NL → SPARQL、`rag_context()` 的可选第二阶段 |
| 环境变量中的 API 密钥 | LLM 和嵌入端点 —— 密钥**永远不**存储在数据库中 |
| `pg_ripple.kge_enabled = on` | KGE 训练和 `find_alignments()` |

当缺少依赖项时，所有 AI 功能都会优雅降级 —— 它们发出 `WARNING` 并返回零行，而不是引发 `ERROR`。您可以将*使用*这些功能的代码部署到未配置 LLM 端点的 CI 环境中。

---

## 安全说明

- **API 密钥永远不存储在 PostgreSQL 中。** 配置环境变量的*名称*（例如 `pg_ripple.llm_api_key_env = 'OPENAI_API_KEY'`），扩展在调用时读取密钥。
- **出站 HTTP 调用需要列入白名单的端点。** LLM 端点（通过 `llm_endpoint` / `embedding_api_url` 注册）和联邦 SPARQL 服务（通过 `register_endpoint()` 注册）在每次调用时都会根据白名单进行检查。这可以防止服务器端请求伪造（SSRF）。
- **提示中的 PII。** `rag_context()` 和 `sparql_from_nl()` 将图摘录发送到配置的 LLM。使用命名图行级安全性（请参阅[多租户图](multi-tenant-graphs.md)）将租户数据排除在您无法控制的提示之外。

---

## 下一步

- **想在五分钟内尝试？** [Cookbook：基于知识图谱的聊天机器人](../cookbook/grounded-chatbot.md)
- **已有知识图谱并想添加 RAG？** [RAG 管道](../user-guide/rag-pipeline.md)
- **合并来自多个系统的客户数据？** [记录链接](record-linkage.md)
- **在图上构建推荐引擎？** [知识图谱嵌入](knowledge-graph-embeddings.md)
- **需要用于 OpenAI 结构化输出的结构化提示？** [导出与共享 —— JSON-LD 框架](exporting-and-sharing.md)