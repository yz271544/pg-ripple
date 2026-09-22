# 向量搜索参考

此页面是 pg_ripple 向量 + SPARQL 混合搜索的参考。

## 概述

pg_ripple 与 pgvector 集成，以提供对 RDF 实体的语义相似度搜索。实体嵌入存储在 `_pg_ripple.embeddings` 中，并带有 HNSW 或 IVFFlat 索引。`pg:similar()` SPARQL 函数查询向量索引并按余弦相似度返回排序结果。

混合检索将向量相似度排序与 SPARQL 图约束相结合，支持如下查询："查找与 X 语义相似且也满足 SHACL 形状 Y 的实体"。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE '%vector%' OR feature_name LIKE '%embed%' OR feature_name LIKE '%kge%';
```

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.embed_entities(graph_iri TEXT, model TEXT) → BIGINT` | 批量嵌入图中的所有实体 |
| `pg_ripple.similar_entities(iri TEXT, k INT, model TEXT) → SETOF TEXT` | 通过嵌入查找 k 个最近邻实体 |
| `pg_ripple.suggest_sameas(iri TEXT, k INT) → SETOF TEXT` | 通过余弦相似度建议 `owl:sameAs` 候选者 |

## SPARQL `pg:similar()` 函数

在 SPARQL 查询中使用 `pg:similar()` 扩展函数进行内联向量搜索：

```sparql
PREFIX pg: <http://pg_ripple.io/fn/>
SELECT ?entity ?score WHERE {
  ?entity pg:similar("machine learning", 10) ?score .
  ?entity a <http://example.org/Paper> .
}
ORDER BY DESC(?score)
```

## 嵌入模型

嵌入通过配置的 LLM 嵌入端点生成。每个实体-模型对在 `_pg_ripple.embeddings` 中存储一次。增量嵌入工作线程在后台运行，并在插入新实体时对其进行嵌入。

## 知识图谱嵌入 (KGE)

图结构嵌入（TransE、RotatE）由 `src/kge.rs` 计算，并与文本嵌入一起存储。KGE 嵌入捕获结构关系模式，并补充基于文本的语义相似度。

## 索引配置

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.vector_index_type` | `'hnsw'` | 索引类型：`hnsw` 或 `ivfflat` |
| `pg_ripple.hnsw_m` | `16` | HNSW M 参数 |
| `pg_ripple.hnsw_ef_construction` | `64` | HNSW ef_construction 参数 |
| `pg_ripple.vector_dimensions` | `1536` | 嵌入向量维度 |

## 相关页面

- [嵌入函数](embedding-functions.md)
- [GraphRAG 参考](graphrag.md)
- [向量索引权衡](vector-index-tradeoffs.md)
- [功能状态分类](feature-status-taxonomy.md)