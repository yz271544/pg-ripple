# §2.6 导出和共享

## 是什么和为什么

pg_ripple 中的数据需要**流出** — 流向其他系统、流向用于归档的文件、流向 LLM 的 RAG 管道，或通过 Parquet 文件流向 Microsoft 的 GraphRAG 框架。pg_ripple 支持所有标准 RDF 序列化格式以及用于 API 就绪输出的 JSON-LD 框架和用于 GraphRAG 的 BYOG（自带图）Parquet 导出。

本章是所有导出功能的**规范参考**，包括 GraphRAG BYOG 管道。其他章节交叉引用此处以进行 GraphRAG 详细信息。

---

## 它是如何工作的

### 导出格式

| 格式 | 函数 | 流式变体 | 命名图支持 |
|---|---|---|---|
| **N-Triples** | `export_ntriples()` | — | 每图或默认 |
| **N-Quads** | `export_nquads()` | — | 是（所有图）|
| **Turtle** | `export_turtle()` | `export_turtle_stream()` | 每图或默认 |
| **JSON-LD** | `export_jsonld()` | `export_jsonld_stream()` | 每图或默认 |
| **JSON-LD Framed** | `export_jsonld_framed()` | `export_jsonld_framed_stream()` | 每图或默认 |
| **SPARQL CONSTRUCT → Turtle** | `sparql_construct_turtle()` | — | 通过查询 |
| **SPARQL CONSTRUCT → JSON-LD** | `sparql_construct_jsonld()` | — | 通过查询 |
| **SPARQL DESCRIBE → Turtle** | `sparql_describe_turtle()` | — | 通过查询 |
| **SPARQL DESCRIBE → JSON-LD** | `sparql_describe_jsonld()` | — | 通过查询 |
| **Parquet（GraphRAG 实体）** | `export_graphrag_entities()` | — | 每图 |
| **Parquet（GraphRAG 关系）** | `export_graphrag_relationships()` | — | 每图 |
| **Parquet（GraphRAG 文本单元）** | `export_graphrag_text_units()` | — | 每图 |

### 流式导出

对于大型图，流式导出每个三元组（或每个主语，JSON-LD）返回一行，避免缓冲内存中的整个文档：

```sql
-- 一次流式传输 Turtle 一行
SELECT * FROM pg_ripple.export_turtle_stream();

-- 一次流式传输 JSON-LD 一个主语（NDJSON）
SELECT * FROM pg_ripple.export_jsonld_stream();
```

### JSON-LD 框架

JSON-LD 框架将扁平 RDF 重新整形为嵌套的、对应用程序友好的 JSON。**frame**是指定所需结构的 JSON 模板：

1. pg_ripple 将框架转换为 SPARQL CONSTRUCT 查询。
2. CONSTRUCT 查询针对三元组存储执行。
3. W3C 嵌入算法根据框架嵌套匹配的节点。
4. 结果使用框架的 `@context` 进行压缩。

---

## 详细的示例

### 导出为 N-Triples

最简单的格式 — 每行一个三元组：

```sql
-- 导出默认图
SELECT pg_ripple.export_ntriples(NULL);
```

输出：

```
<https://example.org/paper/42> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://purl.org/ontology/bibo/AcademicArticle> .
<https://example.org/paper/42> <http://purl.org/dc/terms/title> "Knowledge Graphs in Practice" .
<https://example.org/paper/42> <http://purl.org/dc/terms/creator> <https://example.org/person/alice> .
```

导出特定的命名图：

```sql
SELECT pg_ripple.export_ntriples('https://example.org/graph/pubmed');
```

### 导出为 N-Quads

N-Quads 包括每个三元组的图 IRI：

```sql
-- 导出所有图（传递 NULL）
SELECT pg_ripple.export_nquads(NULL);
```

### 导出为 Turtle

使用前缀声明的紧凑、可读输出：

```sql
SELECT pg_ripple.export_turtle();
```

输出：

```turtle
@prefix ex: <https://example.org/> .
@prefix dct: <http://purl.org/dc/terms/> .
@prefix bibo: <http://purl.org/ontology/bibo/> .

ex:paper/42 a bibo:AcademicArticle ;
    dct:title "Knowledge Graphs in Practice" ;
    dct:creator ex:person/alice, ex:person/bob .
```

### 导出为 JSON-LD

```sql
SELECT pg_ripple.export_jsonld();
```

返回扩展节点对象的 JSONB 数组：

```json
[
  {
    "@id": "https://example.org/paper/42",
    "@type": ["http://purl.org/ontology/bibo/AcademicArticle"],
    "http://purl.org/dc/terms/title": [{"@value": "Knowledge Graphs in Practice"}],
    "http://purl.org/dc/terms/creator": [
      {"@id": "https://example.org/person/alice"},
      {"@id": "https://example.org/person/bob"}
    ]
  }
]
```

### JSON-LD 框架

将输出调整为应用程序预期的确切 JSON 结构：

```sql
SELECT pg_ripple.export_jsonld_framed('{
    "@context": {
        "dct": "http://purl.org/dc/terms/",
        "foaf": "http://xmlns.com/foaf/0.1/",
        "bibo": "http://purl.org/ontology/bibo/",
        "schema": "https://schema.org/",
        "title": "dct:title",
        "creator": "dct:creator",
        "name": "foaf:name",
        "affiliation": "schema:affiliation"
    },
    "@type": "bibo:AcademicArticle",
    "creator": {
        "name": {},
        "affiliation": {
            "name": {}
        }
    }
}'::jsonb);
```

返回嵌套的 JSON-LD：

```json
{
  "@context": {"dct": "http://purl.org/dc/terms/", "...": "..."},
  "@graph": [
    {
      "@type": "bibo:AcademicArticle",
      "title": "Knowledge Graphs in Practice",
      "creator": [
        {
          "name": "Alice Johnson",
          "affiliation": {
            "name": "Massachusetts Institute of Technology"
          }
        },
        {
          "name": "Bob Smith",
          "affiliation": {
            "name": "Stanford University"
          }
        }
      ]
    }
  ]
}
```

### 调试框架

查看生成出的生成而不执行：

```sql
SELECT pg_ripple.jsonld_frame_to_sparql('{
    "@context": {
        "dct": "http://purl.org/dc/terms/",
        "bibo": "http://purl.org/ontology/bibo/",
        "title": "dct:title"
    },
    "@type": "bibo:AcademicArticle",
    "title": {}
}'::jsonb);
```

### 基于 CONSTRUCT 的导出

使用 SPARQL CONTRACT 进行选择性的、转换后的导出：

```sql
-- 将引用图导出为 Turtle
SELECT pg_ripple.sparql_construct_turtle('
PREFIX bibo: <http://purl.org/ontology/bibo/>
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX ex:   <https://example.org/>

CONSTRUCT {
    ?paper ex:cites ?cited .
    ?paper dct:title ?title .
    ?cited dct:title ?citedTitle .
}
WHERE {
    ?paper bibo:cites ?cited ;
           dct:title ?title .
    ?cited dct:title ?citedTitle .
}
');

-- 与 REST API 的 JSON-LD 相同
SELECT pg_ripple.sparql_construct_jsonld('
PREFIX bibo: <http://purl.org/ontology/bibo/>
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX ex:   <https://example.org/>

CONSTRUCT {
    ?paper ex:cites ?cited .
    ?paper dct:title ?title .
}
WHERE {
    ?paper bibo:cites ?cited ;
           dct:title ?title .
}
');
```

### 基于 DESCRIBE 的导出

导出有关特定实体的所有内容：

```sql
-- 完整的 Turtle 描述
SELECT pg_ripple.sparql_describe_turtle('
DESCRIBE <https://example.org/paper/42>
');

-- 对称 CBD（包括传入链接）
SELECT pg_ripple.sparql_describe_turtle(
    'DESCRIBE <https://example.org/person/alice>',
    'scbd'
);

-- 作为 JSON-LD
SELECT pg_ripple.sparql_describe_jsonld(
    'DESCRIBE <https://example.org/paper/42>'
);
```

---

## GraphRAG BYOG 管道

pg_ripple 是 Microsoft GraphRAG 的**自带图**（BYOG）数据的规范源。该管道使用三个导出函数生成与 GraphRAG 引入格式兼容的 Parquet 文件。

```admonish note
这是规范的 GraphRAG 章节点。所有其他提及 GraphRAG 的文档都应交叉引用此部分。
```

### 第 1 步：建模实体和关系

GraphRAG 要求使用 `gr:` 前缀建模的实体、关系和文本单元。加载 GraphRAG 本体：

```sql
SELECT pg_ripple.load_turtle('
@prefix gr:   <urn:graphrag:> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix ex:   <https://example.org/> .

# 实体：一篇论文
ex:paper/42 a gr:Entity ;
    gr:title "Knowledge Graphs in Practice" ;
    gr:type "AcademicArticle" ;
    gr:description "A comprehensive survey of knowledge graph technologies and applications." ;
    gr:frequency 5 ;
    gr:degree 3 .

# 实体：一位作者
ex:person/alice a gr:Entity ;
    gr:title "Alice Johnson" ;
    gr:type "Person" ;
    gr:description "Researcher at MIT specializing in knowledge representation." ;
    gr:frequency 8 ;
    gr:degree 5 .

# 关系
ex:rel/1 a gr:Relationship ;
    gr:source ex:paper/42 ;
    gr:target ex:person/alice ;
    gr:description "authored by" ;
    gr:weight "1.0"^^xsd:float ;
    gr:combinedDegree 8 .

# 文本单元
ex:text/1 a gr:TextUnit ;
    gr:text "This paper surveys knowledge graph technologies..." ;
    gr:nTokens 150 ;
    gr:documentId "doc-001" .
');
```

### 第 2 步：使用 Datalog 规则丰富

使用 Datalog 规则派生其他 GraphRAG 元数据：

```sql
SELECT pg_ripple.load_rules('
# 从三元组计数派生实体频率
?e gr:frequency ?count :-
    ?e rdf:type gr:Entity ,
    COUNT(?t WHERE ?t ?anyPred ?e) = ?count .

# 派生关系组合度数
?r gr:combinedDegree ?deg :-
    ?r rdf:type gr:Relationship ,
    ?r gr:source ?s ,
    ?r gr:target ?t ,
    COUNT(?p1 WHERE ?s ?p1 ?_) = ?sDeg ,
    COUNT(?p2 WHERE ?t ?p2 ?_) = ?tDeg .
', 'graphrag-enrichment');

SELECT pg_ripple.infer_agg('graphrag-enrichment');
```

### 第 3 步：使用 SHACL 验证

在导出之前确保数据质量：

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix gr:   <urn:graphrag:> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

<urn:graphrag:EntityShape> a sh:NodeShape ;
    sh:targetClass gr:Entity ;
    sh:property [
        sh:path gr:title ;
        sh:minCount 1 ;
        sh:datatype xsd:string ;
    ] ;
    sh:property [
        sh:path gr:type ;
        sh:minCount 1 ;
    ] .

<urn:graphrag:RelationshipShape> a sh:NodeShape ;
    sh:targetClass gr:Relationship ;
    sh:property [
        sh:path gr:source ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path gr:target ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .
');

-- 在导出之前验证
SELECT pg_ripple.validate();
```

### 第 4 步：导出到 Parquet

```sql
-- 导出实体（需要超级用户）
SELECT pg_ripple.export_graphrag_entities('', '/data/graphrag/entities.parquet');

-- 导出关系
SELECT pg_ripple.export_graphrag_relationships('', '/data/graphrag/relationships.parquet');

-- 导出文本单元
SELECT pg_ripple.export_graphrag_text_units('', '/data/graphrag/text_units.parquet');
```

每个函数返回写入的行数。Parquet 文件直接与 `pyarrow.parquet.read_table()` 和 GraphRAG 的 BYOG 配置兼容：

```yaml
# GraphRAG settings.yaml
entity_table_path: /data/graphrag/entities.parquet
relationship_table_path: /data/graphrag/relationships.parquet
text_unit_table_path: /data/graphrag/text_units.parquet
```

### 第 5 步：从命名图导出

对于多租户或版本化导出：

```sql
-- 仅导出"production"图中的实体
SELECT pg_ripple.export_graphrag_entities(
    'https://example.org/graph/production',
    '/data/graphrag/prod_entities.parquet'
);

SELECT pg_ripple.export_graphrag_relationships(
    'https://example.org/graph/production',
    '/data/graphrag/prod_relationships.parquet'
);

SELECT pg_ripple.export_graphrag_text_units(
    'https://example.org/graph/production',
    '/data/graphrag/prod_text_units.parquet'
);
```

---

## 常见模式

### 模式：API 响应格式化

使用 JSON-LD 框架生成 API 就绪响应：

```sql
-- Papers 端点：带有作者的嵌套 JSON
SELECT pg_ripple.export_jsonld_framed('{
    "@context": {
        "title": "http://purl.org/dc/terms/title",
        "creator": "http://purl.org/dc/terms/creator",
        "name": "http://xmlns.com/foaf/0.1/name",
        "type": "@type"
    },
    "@type": "http://purl.org/ontology/bibo/AcademicArticle",
    "creator": { "name": {} }
}'::jsonb);
```

### 模式：通过 CONSTRUCT 视图安排导出

对于连续更新的导出，创建 CONSTRUCT 视图（需要 pg_trickle）：

```sql
SELECT pg_ripple.create_construct_view(
    'citation_graph',
    'PREFIX bibo: <http://purl.org/ontology/bibo/>
     PREFIX dct: <http://purl.org/dc/terms/>
     CONSTRUCT { ?p bibo:cites ?c . ?p dct:title ?t . }
     WHERE { ?p bibo:cites ?c ; dct:title ?t }',
    '30s',
    true
);

-- 视图每 30 秒自动刷新
SELECT * FROM pg_ripple.construct_view_citation_graph_decoded;
```

### 模式：流式导出到文件

对于大型图，将 `COPY` 与流式导出一起使用：

```sql
COPY (SELECT * FROM pg_ripple.export_turtle_stream())
TO '/data/export/full_graph.ttl';

COPY (SELECT * FROM pg_ripple.export_jsonld_stream())
TO '/data/export/full_graph.ndjson';
```

### 模式：使用 SPARQL 进行选择性导出

仅导出图的子集：

```sql
-- 仅导出 2024 年的论文
SELECT pg_ripple.sparql_construct_turtle('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>
PREFIX xsd:  <http://www.w3.org/2001/XMLSchema#>

CONSTRUCT { ?paper ?p ?o }
WHERE {
    ?paper a bibo:AcademicArticle ;
           dct:date ?date ;
           ?p ?o .
    FILTER (?date >= "2024-01-01"^^xsd:date)
}
');
```

---

## 性能和权衡

### 缓冲与流式导出

| 模式 | 内存使用 | 输出格式 | 适用于 |
|---|---|---|---|
| 缓冲（`export_turtle()`）| 整个图在内存中 | 完整文档 | 小型-中型图 |
| 流式（`export_turtle_stream()`）| 一次一个三元组 | 每行一个三元组 | 大型图（数百万个三元组）|

### Parquet 导出性能

GraphRAG Parquet 导出为每个实体类型扫描相关的 VP 表一次。结果取决于 `gr:Entity`、`gr:Relationship` 和 `gr:TextUnit` 节点的数量：

- 约 100K 实体：<5 秒
- 约 1M 实体：约 30 秒
- 写入路径需要超级用户（写入到服务器文件系统）

### JSON-LD 框架成本

框架涉及执行 SPARQL CONSTRUCT 查询，然后应用 W3C 嵌入算法。成本由 CONSTRUCT 查询主导；嵌入步骤与匹配节点的数量成线性关系。

```admonish tip
使用 `jsonld_frame_to_sparql()` 检查生成的 CONSTRUCT 查询，并在调用 `export_jsonld_framed()` 之前验证其效率。
```

---

## 陷阱和调试

### 空 Parquet 文件

如果 `export_graphrag_entities()` 返回 0，请检查您的数据是否使用正确的 `gr:` 前缀以及实体是否具有 `rdf:type gr:Entity`：

```sql
SELECT * FROM pg_ripple.find_triples(
    NULL,
    '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>',
    '<urn:graphrag:Entity>'
);
```

### 框架返回空结果

确保框架的 `@type` 与存储中的实际 `rdf:type` 值匹配。该类型必须是完整的 IRI，而不是前缀名称：

```sql
-- 检查存在的类型
SELECT * FROM pg_ripple.sparql('
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
SELECT DISTINCT ?type WHERE { ?x rdf:type ?type }
');
```

### 服务器端文件权限

Parquet 导出写入服务器文件系统。确保 `postgres` 操作系统用户对输出目录具有写权限：

```bash
sudo mkdir -p /data/graphrag
sudo chown postgres:postgres /data/graphrag
```

### 大型导出内存

对于具有数百万个三元组的图，缓冲导出（`export_turtle()`、`export_jsonld()`）可能会使用大量内存。切换到流式变体或将 `COPY ... TO` 与流式一起使用。

---

## 后续步骤

- **[§2.7 AI 检索和 GraphRAG](../features/graphrag.md)** — 向量嵌入和 RAG 检索管道。
- **[§2.8 API 和集成](../features/apis-and-integration.md)** — 通过 HTTP 端点提供导出的数据。
- **[§2.3 使用 SPARQL 查询](../features/querying-with-sparql.md)** — 用于选择性导出的 CONSTRUCT 和 DESCRIBE 查询。

## 进一步阅读

- [博客：用于嵌套 JSON 的 JSON-LD 框架](https://github.com/trickle-labs/pg-ripple/blob/main/blog/json-ld-framing-nested-json.md) — 将图数据整形为 API 就绪的 JSON 文档