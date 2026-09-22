# 嵌入函数参考

这些函数在 pg_ripple 中实现向量嵌入存储和相似度搜索。所有函数都需要 pgvector 扩展才能执行实际工作；没有它，它们会优雅降级（WARNING + 空结果）。

---

## `pg_ripple.store_embedding`

```sql
pg_ripple.store_embedding(
    entity_iri TEXT,
    embedding  FLOAT8[],
    model      TEXT DEFAULT NULL
) RETURNS VOID
```

将给定实体 IRI 的浮点向量上插入到 `_pg_ripple.embeddings`。如果实体已有嵌入，则会替换它。

**参数：**

| 参数 | 描述 |
|---|---|
| `entity_iri` | 实体的完整 IRI（必须存在于三元组存储字典中） |
| `embedding` | Float8 数组；长度必须匹配 `pg_ripple.embedding_dimensions` |
| `model` | 与向量一起存储的模型名称标签；默认为 `pg_ripple.embedding_model` |

**返回：** `VOID`

**错误代码：** PT602（维度不匹配），PT603（未安装 pgvector）

**示例：**

```sql
SELECT pg_ripple.store_embedding(
    'https://pharma.example/aspirin',
    ARRAY[0.12, -0.34, 0.56, ...]::float8[]
);
```

---

## `pg_ripple.similar_entities`

```sql
pg_ripple.similar_entities(
    query_text TEXT,
    k          INT  DEFAULT 10,
    model      TEXT DEFAULT NULL
) RETURNS TABLE(entity_id BIGINT, entity_iri TEXT, score FLOAT8)
```

通过配置的嵌入 API 嵌入 `query_text`，并返回具有最小余弦距离的 *k* 个实体。

**参数：**

| 参数 | 描述 |
|---|---|
| `query_text` | 要嵌入并用作查询的自由格式文本 |
| `k` | 要返回的最近邻数（限制为 1–1000） |
| `model` | 从 `pg_ripple.embedding_model` 覆盖模型 |

**返回：** 按升序余弦距离排序的 `(entity_id, entity_iri, score)` 表。

**错误代码：** PT601（未配置 API URL），PT603（未安装 pgvector）

**示例：**

```sql
SELECT entity_iri, score
FROM pg_ripple.similar_entities('anti-inflammatory drugs', k := 5)
ORDER BY score;
```

---

## `pg_ripple.embed_entities`

```sql
pg_ripple.embed_entities(
    graph_iri  TEXT DEFAULT '',
    model      TEXT DEFAULT NULL,
    batch_size INT  DEFAULT 100
) RETURNS BIGINT
```

批量嵌入具有 `rdfs:label`（或 `skos:prefLabel`）但尚无嵌入的实体。按批次调用嵌入 API 并通过 `store_embedding` 存储结果。

**参数：**

| 参数 | 描述 |
|---|---|
| `graph_iri` | 要扫描的命名图；空字符串 = 默认图 |
| `model` | 要使用的嵌入模型（回退到 GUC） |
| `batch_size` | 每个 API 调用要嵌入的实体数（1–500） |

**返回：** 成功嵌入的实体数。

**错误代码：** PT601（未配置 API URL），PT603（未安装 pgvector）

**示例：**

```sql
-- 嵌入默认图中的所有实体
SELECT pg_ripple.embed_entities() AS embedded_count;

-- 嵌入特定命名图中的实体
SELECT pg_ripple.embed_entities(
    graph_iri  := 'https://myapp.org/graphs/products',
    batch_size := 200
);
```

---

## `pg_ripple.refresh_embeddings`

```sql
pg_ripple.refresh_embeddings(
    graph_iri TEXT    DEFAULT '',
    model     TEXT    DEFAULT NULL,
    force     BOOLEAN DEFAULT FALSE
) RETURNS BIGINT
```

重新嵌入其嵌入陈旧的（标签已更改）或缺失的实体。当 `force := TRUE` 时，无论陈旧与否，重新嵌入所有实体。

**参数：**

| 参数 | 描述 |
|---|---|
| `graph_iri` | 要扫描的命名图 |
| `model` | 要使用的嵌入模型 |
| `force` | 当 TRUE 时，重新嵌入所有内容；当 FALSE（默认）时，仅重新嵌入陈旧条目 |

**返回：** 重新嵌入的实体数。

**错误代码：** PT601（未配置 API URL），PT603（未安装 pgvector），PT606（非强制时未找到陈旧嵌入）

**示例：**

```sql
-- 仅刷新陈旧嵌入
SELECT pg_ripple.refresh_embeddings();

-- 强制完全重新嵌入
SELECT pg_ripple.refresh_embeddings(force := TRUE);
```

---

## 内部表

### `_pg_ripple.embeddings`

存储实体嵌入。安装 pgvector 时：

| 列 | 类型 | 描述 |
|---|---|---|
| `entity_id` | `BIGINT` | 实体 IRI 的字典整数 ID |
| `embedding` | `vector(N)` | 浮点向量，维度 = `pg_ripple.embedding_dimensions` |
| `model` | `TEXT` | 用于生成此嵌入的模型 |
| `updated_at` | `TIMESTAMPTZ` | 上次存储此嵌入的时间 |

`embedding` 上的 HNSW 索引支持近似最近邻搜索。

当 pgvector **不存在**时，`embedding` 列为 `BYTEA`，所有相似度函数返回空结果。

---

## SPARQL 集成

`pg:similar()` 函数可从 SPARQL `BIND` 表达式调用。有关用法，请参阅[混合搜索](../features/vector-and-hybrid-search.md)。

**函数 IRI：** `http://pg-ripple.org/functions/similar`

**签名（SPARQL）：**

```sparql
pg:similar(?entity_variable, "query text", k_integer)
```

返回数字余弦距离，或者在 pgvector 不存在时返回 SPARQL 未绑定（NULL）。

---

## v0.28.0 函数

### `pg_ripple.hybrid_search`

```sql
pg_ripple.hybrid_search(
    sparql_query TEXT,
    query_text   TEXT,
    k            INT     DEFAULT 10,
    alpha        FLOAT8  DEFAULT 0.5,
    model        TEXT    DEFAULT NULL
) RETURNS TABLE (
    entity_id   BIGINT,
    entity_iri  TEXT,
    rrf_score   FLOAT8,
    sparql_rank INT,
    vector_rank INT
)
```

使用倒数排名融合（RRF）将 SPARQL 候选集与向量 k-NN 结果融合。`alpha = 0.0` 是仅向量，`1.0` 是仅 SPARQL，`0.5` 是相等的权重。

**错误代码：** PT603（未安装 pgvector；返回空结果和 WARNING）

---

### `pg_ripple.rag_retrieve`

```sql
pg_ripple.rag_retrieve(
    question      TEXT,
    sparql_filter TEXT    DEFAULT NULL,
    k             INT     DEFAULT 5,
    model         TEXT    DEFAULT NULL,
    output_format TEXT    DEFAULT 'jsonb'
) RETURNS TABLE (
    entity_iri   TEXT,
    label        TEXT,
    context_json JSONB,
    distance     FLOAT8
)
```

端到端 RAG 检索：向量搜索 → 可选的 SPARQL 过滤器 → 上下文化 → 结构化输出。有关完整文档，请参阅[RAG 检索](../user-guide/rag-pipeline.md)。

---

### `pg_ripple.contextualize_entity`

```sql
pg_ripple.contextualize_entity(
    entity_iri   TEXT,
    depth        INT DEFAULT 1,
    max_neighbors INT DEFAULT 20
) RETURNS TEXT
```

将实体的标签、RDF 类型和邻居标签序列化为适合嵌入的纯文本。示例输出：`"aspirin. Type: NSAID, Drug. Related: headache, fever, inflammation"`。

---

### `pg_ripple.list_embedding_models`

```sql
pg_ripple.list_embedding_models()
RETURNS TABLE (model TEXT, entity_count BIGINT, dimensions INT)
```

枚举 `_pg_ripple.embeddings` 中的所有嵌入模型及其实体计数和向量维度。

---

### `pg_ripple.add_embedding_triples`

```sql
pg_ripple.add_embedding_triples() RETURNS BIGINT
```

为所有具有嵌入的实体具体化 `<entity> <http://pg-ripple.org/functions/hasEmbedding> "true"^^xsd:boolean` 三元组。与 SHACL `sh:minCount 1` 一起使用以验证嵌入完整性。

---

### `pg_ripple.register_vector_endpoint`

```sql
pg_ripple.register_vector_endpoint(
    url      TEXT,
    api_type TEXT
) RETURNS VOID
```

注册用于联邦的外部向量服务。有关完整文档，请参阅[向量联邦](../user-guide/vector-federation.md)。

`api_type` 必须是以下之一：`pgvector`、`weaviate`、`qdrant`、`pinecone`。无效的类型发出 WARNING **不**持久化端点。

**错误代码：** PT607（无效的 api_type）