# 向量嵌入与混合搜索

**状态**：自 v0.48.0（VEC-01）起可用  
**需要**：[`pgvector`](https://github.com/pgvector/pgvector) 扩展（必需）。自动嵌入生成需要 OpenAI 兼容的嵌入 API；手动 `float4[]` 插入无需它。  
**SQL**：`pg_ripple.store_embedding()`、`pg_ripple.vector_search()`、`pg_ripple.hybrid_search()`  
**HTTP**：`POST /sparql`（在 SPARQL SELECT 内支持 `pg:similar()`）  
**降级**：没有 pgvector，所有嵌入存储和 `pg:similar()` 查询都会在查询时失败。

---

向量搜索和图查询回答不同的问题。向量搜索擅长 *"看起来像 X 的事物"*；图查询擅长 *"与 X 处于此精确关系的事物"*。现实世界的问题通常需要两者 —— *"语义上类似于布洛芬的处方，由心脏病学队列的患者服用"*。pg_ripple 在一个查询中同时执行两者。

本章是实用参考。它涵盖：

- 将向量嵌入与 RDF 实体一起存储。
- 使用 `pgvector` 构建 HNSW 索引。
- 运行纯相似性、纯 SPARQL 和混合（RRF）搜索。
- `pg:similar()` SPARQL 函数 —— SPARQL 模式内的向量搜索。
- 联邦到外部向量存储（Weaviate、Qdrant、Pinecone、远程 pgvector）。

要进行更高级别的决策制定，请从 [AI 概述](ai-overview.md) 开始。
有关端到端 RAG 管道，请参阅 [RAG 管道](../user-guide/rag-pipeline.md)。

---

## 设置

```sql
-- 1. pgvector 是必需的。
CREATE EXTENSION IF NOT EXISTS vector;

-- 2. 将 pg_ripple 指向 OpenAI 兼容的嵌入 API。
ALTER SYSTEM SET pg_ripple.embedding_api_url      = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.embedding_api_key_env  = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.embedding_model        = 'text-embedding-3-small';
ALTER SYSTEM SET pg_ripple.embedding_dimensions   = 1536;
SELECT pg_reload_conf();
```

API 密钥在调用时从指定的环境变量读取，**永远不**存储在数据库中。

---

## 嵌入实体

每个具有 `rdfs:label`（或 `skos:prefLabel` 或任何配置的标签谓词）的实体都可以被嵌入：

```sql
-- 批量嵌入每个已标记的实体。
SELECT pg_ripple.embed_entities();

-- 或者仅嵌入一个命名图。
SELECT pg_ripple.embed_entities('https://example.org/products');

-- 手动存储预计算的向量。
SELECT pg_ripple.store_embedding(
    iri    := '<https://example.org/aspirin>',
    vec    := '[...1536 floats...]'::vector,
    model  := 'text-embedding-3-small'
);
```

设置 `pg_ripple.use_graph_context = on` 以使用每个实体的 1 跳图邻域丰富其嵌入输入 —— 这显着提高了标签本身模糊的实体的召回率（公司"Apple"与水果"apple"）。

设置 `pg_ripple.auto_embed = on` 以将任何新插入的已标记实体加入后台嵌入队列。合并工作线程排出队列。

---

## 三种搜索模式

### 1. 纯相似性

```sql
SELECT entity_iri, similarity
FROM pg_ripple.similar_entities('headache medication', k := 10);
```

等效于您针对 `_pg_ripple.embeddings` 手动编写的余弦距离 HNSW 查找，但为您处理了文本到向量的转换。

### 2. 纯 SPARQL

```sql
SELECT * FROM pg_ripple.sparql($$
    SELECT ?drug WHERE {
        ?drug a <https://example.org/Drug> ;
              <https://example.org/treats> <https://example.org/headache> .
    }
$$);
```

### 3. 混合（倒数排名融合）

混合搜索返回（a）SPARQL 查询的 top-k 匹配和（b）向量查询的 top-k 匹配的*融合*排名，使用倒数排名融合。这同时捕获精确的关系匹配*和*模糊的语义邻居。

```sql
SELECT entity_iri, score
FROM pg_ripple.hybrid_search(
    sparql := 'SELECT ?d WHERE { ?d a <https://example.org/Drug> }',
    text   := 'headache medication',
    k      := 10,
    alpha  := 0.5  -- 0.0 = 纯 SPARQL，1.0 = 纯向量
);
```

`alpha` 是给予向量排名的相对权重。通过检查进行调优；0.5 是合理的默认值。

---

## `pg:similar()` —— SPARQL **内部**的向量搜索

`pg:similar()` SPARQL 函数返回实体和自由文本查询之间的余弦相似度。它可以在 `BIND`、`FILTER` 和 `ORDER BY` 中调用。

```sparql
PREFIX pg: <http://pg-ripple.io/fn/>

SELECT ?drug ?score WHERE {
    ?drug a <https://example.org/Drug> .
    BIND(pg:similar(?drug, "anti-inflammatory") AS ?score)
    FILTER(?score > 0.7)
}
ORDER BY DESC(?score)
LIMIT 20
```

这是最具表现力的形式 —— 图约束和相似性分数位于同一查询计划中，无需客户端后处理。

---

## 向量联邦 —— Weaviate、Qdrant、Pinecone、远程 pgvector

如果您已经运营向量存储并且不想迁移，请将其注册为**向量联邦端点**。`hybrid_search()` 将混合来自本地 pg_ripple *和*远程服务的结果。

```sql
SELECT pg_ripple.register_vector_endpoint(
    url      := 'https://qdrant.internal:6333',
    api_type := 'qdrant'
);

SELECT pg_ripple.register_vector_endpoint('https://weaviate.internal:8080', 'weaviate');
SELECT pg_ripple.register_vector_endpoint('https://my-index.pinecone.io', 'pinecone');
```

支持的 `api_type` 值：`pgvector`、`weaviate`、`qdrant`、`pinecone`。端点存储在 `_pg_ripple.vector_endpoints` 中，`register_vector_endpoint()` 是幂等的。

联邦 `hybrid_search()` 调用自动扇出，收集每个端点的 top-k 结果，并重新融合它们。

---

## GUC 一览

| GUC | 默认值 | 目的 |
|---|---|---|
| `pg_ripple.pgvector_enabled` | `on` | 主开关。当关闭时，向量函数发出 WARNING 并返回零行。 |
| `pg_ripple.embedding_api_url` | 空 | OpenAI 兼容端点基础 URL |
| `pg_ripple.embedding_api_key_env` | `PG_RIPPLE_EMBEDDING_API_KEY` | API 密钥的环境变量名称 |
| `pg_ripple.embedding_model` | `text-embedding-3-small` | 默认模型 |
| `pg_ripple.embedding_dimensions` | `1536` | 向量大小 |
| `pg_ripple.embedding_batch_size` | `100` | `embed_entities()` 的 API 批大小 |
| `pg_ripple.use_graph_context` | `off` | 在嵌入输入中包括 1 跳邻居 |
| `pg_ripple.auto_embed` | `off` | 后台嵌入新插入的实体 |

---

## 调优 HNSW

pg_ripple 构建的 HNSW 索引是 `_pg_ripple.embeddings(vector vector_cosine_ops)` 上的标准 pgvector 索引。调优它就像调优任何 pgvector 索引一样：

| 设置 | 默认值 | 效果 |
|---|---|---|
| `m`（构建） | 16 | 越高 = 召回率越好，内存越多 |
| `ef_construction`（构建） | 64 | 越高 = 召回率越好，构建越慢 |
| `hnsw.ef_search`（查询） | 40 | 越高 = 召回率越好，查询越慢 |

有关这些旋钮的测量基准，请参阅[向量索引权衡](../reference/vector-index-tradeoffs.md)。

---

## 优雅降级

pg_ripple 中的每个向量函数在调用时检查 pgvector。如果缺少 pgvector，或 `pgvector_enabled = off`，该函数会发出一个 `WARNING` 并返回零行。您的应用程序代码可以在没有 pgvector 的环境（CI、开发）中调用向量函数而不会崩溃。

---

## 另请参阅

- [AI 概述](ai-overview.md)
- [RAG 管道](../user-guide/rag-pipeline.md)
- [知识图谱嵌入](knowledge-graph-embeddings.md) — 图结构嵌入，与文本嵌入互补。
- [向量索引权衡](../reference/vector-index-tradeoffs.md)
- [嵌入函数参考](../reference/embedding-functions.md)

## 进一步阅读

- [博客：向量 + SPARQL 混合搜索](https://github.com/trickle-labs/pg-ripple/blob/main/blog/vector-sparql-hybrid-search.md) — 将图遍历与相似性搜索结合