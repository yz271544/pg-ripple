# GraphRAG 参考

此页面是 pg_ripple 的 GraphRAG（检索增强生成）集成的参考。

## 概述

pg_ripple 提供将 RDF 知识图谱检索与大语言模型（LLM）生成相结合的 GraphRAG 功能。该系统使用基于 pgvector 的语义搜索、SPARQL 查询生成、RAG 上下文组装和实体对齐来支持基于知识图谱的问答。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE '%rag%' OR feature_name LIKE 'graphrag%';
```

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.rag_context(question TEXT, k INT) → TEXT` | 为问题检索 top-k 相关图上下文 |
| `pg_ripple.sparql_from_nl(question TEXT) → TEXT` | 从自然语言生成 SPARQL（需要 LLM 端点） |
| `pg_ripple.suggest_sameas(iri TEXT, k INT) → SETOF TEXT` | 通过嵌入相似度建议 `owl:sameAs` 候选项 |
| `pg_ripple.graphrag_export(graph_iri TEXT, format TEXT) → TEXT` | 以 GraphRAG 兼容格式（Parquet 或 JSON-LD）导出图 |

## RAG 管道

1. **嵌入**：通过配置的 LLM 嵌入端点嵌入主语 IRI。
2. **检索**：给定一个问题，使用 HNSW/IVFFlat 向量搜索（`pg_ripple.rag_context()`）将其嵌入与实体嵌入进行比较。
3. **上下文组装**：通过 SPARQL（`DESCRIBE` 或自定义查询模板）将 top-k 实体扩展到它们的三元组。
4. **生成**：将组装的上下文传递给 LLM 以生成答案。

## Microsoft GraphRAG 集成

`graphrag_export()` 产生与 Microsoft GraphRAG 兼容的输出：
- **Parquet 格式**：具有嵌入的实体和关系表
- **JSON-LD 格式**：上下文注释的图节点

## LLM 配置

通过 GUC 设置 LLM 端点参数：

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.llm_endpoint` | `''` | LLM API 的基础 URL（OpenAI 兼容） |
| `pg_ripple.llm_api_key` | `''` | API 密钥（使用密钥，而不是明文） |
| `pg_ripple.llm_model` | `'gpt-4o'` | 用于生成的模型名称 |
| `pg_ripple.embedding_model` | `'text-embedding-3-small'` | 用于嵌入的模型 |

## 相关页面

- [向量搜索参考](vector-search.md)
- [GraphRAG 函数](graphrag-functions.md)
- [GraphRAG 本体](graphrag-ontology.md)
- [功能状态分类](feature-status-taxonomy.md)