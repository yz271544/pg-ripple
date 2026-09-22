# 引导式教程 — 在 30 分钟内构建知识图谱

本教程从 [Hello World 演练](hello-world.md)结束的地方开始。您将构建一个书目知识图谱，其中包含论文、作者、机构和引用 — 然后对其进行验证、推理并将其导出为 JSON-LD。

教程分为四个独立的片段。每个片段用时不到 10 分钟，并留下一个可用的、越来越丰富的知识图谱。您可以在任何片段之后停止。

```admonish note
本教程使用学术书目数据集。模式 — 实体关系、类型化字面量、命名图、推理、验证 — 同样适用于产品目录、供应链、组织层次结构或任何具有互连数据的领域。
```

## 先决条件

pg_ripple 已安装，您已连接到已创建扩展的 PostgreSQL 数据库。请参阅[安装](installation.md)。

---

## 片段 1：加载和探索（10 分钟）

```admonish tip title="您将学到的内容"
- 如何注册前缀并将 Turtle 数据加载到 pg_ripple
- 如何使用过滤器、聚合和图遍历编写 SPARQL 查询
- 数据如何组织为三元组（主语–谓词–对象）
```

### 注册前缀

```sql
SELECT pg_ripple.register_prefix('bib', 'http://example.org/bib/');
SELECT pg_ripple.register_prefix('foaf', 'http://xmlns.com/foaf/0.1/');
SELECT pg_ripple.register_prefix('dc', 'http://purl.org/dc/elements/1.1/');
SELECT pg_ripple.register_prefix('dcterms', 'http://purl.org/dc/terms/');
SELECT pg_ripple.register_prefix('schema', 'http://schema.org/');
SELECT pg_ripple.register_prefix('skos', 'http://www.w3.org/2004/02/skos/core#');
```

### 加载书目数据集

```sql
SELECT pg_ripple.load_turtle('
@prefix bib:     <http://example.org/bib/> .
@prefix foaf:    <http://xmlns.com/foaf/0.1/> .
@prefix dc:      <http://purl.org/dc/elements/1.1/> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix schema:  <http://schema.org/> .
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix skos:    <http://www.w3.org/2004/02/skos/core#> .

bib:mit       a schema:Organization ; schema:name "MIT" .
bib:stanford  a schema:Organization ; schema:name "Stanford University" .
bib:oxford    a schema:Organization ; schema:name "University of Oxford" .

bib:alice     a foaf:Person ; foaf:name "Alice Chen" ;
              schema:affiliation bib:mit .
bib:bob       a foaf:Person ; foaf:name "Bob Smith" ;
              schema:affiliation bib:stanford .
bib:carol     a foaf:Person ; foaf:name "Carol Martinez" ;
              schema:affiliation bib:oxford .

bib:paper1    a schema:ScholarlyArticle ;
              dc:title "Knowledge Graphs in Practice" ;
              dc:creator bib:alice ; dc:creator bib:bob ;
              dcterms:issued "2024-01-15"^^xsd:date ;
              schema:about <http://example.org/bib/kg> .

bib:paper2    a schema:ScholarlyArticle ;
              dc:title "Efficient SPARQL Query Processing" ;
              dc:creator bib:bob ; dc:creator bib:carol ;
              dcterms:issued "2024-03-22"^^xsd:date .

bib:paper3    a schema:ScholarlyArticle ;
              dc:title "Graph-Enhanced Retrieval for LLMs" ;
              dc:creator bib:alice ;
              dcterms:issued "2024-06-10"^^xsd:date .

bib:paper2    dcterms:references bib:paper1 .
bib:paper3    dcterms:references bib:paper1 .
bib:paper3    dcterms:references bib:paper2 .

bib:alice foaf:knows bib:bob .
bib:bob   foaf:knows bib:carol .
');
```

### 探索：查找 Alice 撰写的所有论文

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX dc: <http://purl.org/dc/elements/1.1/>
  PREFIX bib: <http://example.org/bib/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?title WHERE {
    ?paper dc:creator bib:alice .
    ?paper dc:title ?title .
  }
');
```

### 探索：引用链

查找引用 Alice 所撰写论文的论文：

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX dc: <http://purl.org/dc/elements/1.1/>
  PREFIX dcterms: <http://purl.org/dc/terms/>
  PREFIX bib: <http://example.org/bib/>
  SELECT ?citingTitle ?citedTitle WHERE {
    ?citing dcterms:references ?cited .
    ?cited dc:creator bib:alice .
    ?citing dc:title ?citingTitle .
    ?cited dc:title ?citedTitle .
  }
');
```

### 探索：每个作者的论文数

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX dc: <http://purl.org/dc/elements/1.1/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name (COUNT(?paper) AS ?papers) WHERE {
    ?paper dc:creator ?author .
    ?author foaf:name ?name .
  }
  GROUP BY ?name
  ORDER BY DESC(?papers)
');
```

---

## 片段 2：验证（10 分钟）

```admonish tip title "您将学到的内容"
- 如何使用 SHACL shape 定义数据质量规则
- 如何验证您的知识图谱并捕获约束违规
- SHACL shape 如何充当图数据的 CHECK 约束
```

SHACL（形状约束语言）允许您定义数据质量规则。您将创建一个 shape，要求每个 `ScholarlyArticle` 都有一个标题和至少一个创建者。

### 加载 SHACL shape

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:     <http://www.w3.org/ns/shacl#> .
@prefix schema: <http://schema.org/> .
@prefix dc:     <http://purl.org/dc/elements/1.1/> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .

<http://example.org/shapes/ArticleShape>
  a sh:NodeShape ;
  sh:targetClass schema:ScholarlyArticle ;
  sh:property [
    sh:path dc:title ;
    sh:minCount 1 ;
    sh:maxCount 1 ;
    sh:datatype xsd:string ;
    sh:message "Every article must have exactly one title" ;
  ] ;
  sh:property [
    sh:path dc:creator ;
    sh:minCount 1 ;
    sh:message "Every article must have at least one creator" ;
  ] .
');
```

### 验证数据集

```sql
SELECT pg_ripple.validate();
```

结果是 JSONB 验证报告。如果所有文章都符合，报告显示零个违规。插入一篇错误文章以查看验证捕获：

```sql
SELECT pg_ripple.insert_triple(
  'http://example.org/bib/bad_paper',
  'http://www.w3.org/1999/02/22-rdf-syntax-ns#type',
  'http://schema.org/ScholarlyArticle'
);

SELECT pg_ripple.validate();
```

报告现在显示一个违规：文章没有标题和没有创建者。

---

## 片段 3：推理（10 分钟）

```admonish tip title "您将学到的内容"
- 如何编写从现有数据派生新事实的 Datalog 规则
- 传递推理的工作原理（如果 A 连接到 B 并且 B 连接到 C，那么 A 连接到 C）
- 推理与 SQL 物化视图的比较
```

Datalog 规则允许您派生新事实。您将编写一个规则来推断传递的合著关系：如果 Alice 与 Bob 合著了论文，而 Bob 与 Carol 合著了论文，那么 Alice 和 Carol 是间接连接的。

```sql
SELECT pg_ripple.load_rules('
  coauthor(?a, ?b) :- <http://purl.org/dc/elements/1.1/creator>(?paper, ?a),
                      <http://purl.org/dc/elements/1.1/creator>(?paper, ?b),
                      ?a != ?b.
  connected(?a, ?b) :- coauthor(?a, ?b).
  connected(?a, ?b) :- connected(?a, ?c), coauthor(?c, ?b), ?a != ?b.
', 'coauthorship');
```

### 运行推理

```sql
SELECT pg_ripple.infer('coauthorship');
```

这返回派生出的新事实数。

### 查询派生事实

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX bib: <http://example.org/bib/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name WHERE {
    bib:alice <http://example.org/bib/connected> ?person .
    ?person foaf:name ?name .
  }
');
```

Alice 现在连接到 Bob（在 paper1 上的直接合著者）、Carol（通过 Bob 在 paper2 上）以及通过传递链可能连接到其他人。

---

## 片段 4：导出（10 分钟）

```admonish tip title "您将学到的内容"
- 如何以 Turtle 和 JSON-LD 格式导出您的知识图谱
- SPARQL CONSTRUCT 查询如何为 API 使用者塑造输出
- JSON-LD 框架如何生成嵌套的、API 就绪的 JSON 文档
```

将您的知识图谱导出为 JSON-LD，使用框架模板为 API 塑造格式。

### 导出为 Turtle

```sql
SELECT pg_ripple.export_turtle();
```

这以人类可读的 Turtle 格式返回所有三元组。

### 使用框架导出为 JSON-LD

```sql
SELECT pg_ripple.sparql_construct_jsonld('
  PREFIX dc: <http://purl.org/dc/elements/1.1/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  PREFIX schema: <http://schema.org/>
  CONSTRUCT {
    ?paper dc:title ?title .
    ?paper dc:creator ?author .
    ?author foaf:name ?name .
    ?author schema:affiliation ?org .
    ?org schema:name ?orgName .
  }
  WHERE {
    ?paper a schema:ScholarlyArticle .
    ?paper dc:title ?title .
    ?paper dc:creator ?author .
    ?author foaf:name ?name .
    OPTIONAL {
      ?author schema:affiliation ?org .
      ?org schema:name ?orgName .
    }
  }
');
```

结果是嵌套的 JSON-LD 文档，其中包含论文、其作者和机构隶属关系 — 准备从 REST API 提供服务。

---

## 您构建了什么

在 30 分钟内，您创建了一个具有以下内容的知识图谱：

- **结构化数据** — 论文、作者、机构和引用作为 RDF 三元组
- **质量规则** — SHACL shape 用于捕获不完整的文章
- **派生知识** — Datalog 规则用于推断传递的合著关系
- **API 就绪的导出** — 为下游消费者塑造的 JSON-LD 输出

## 下一步

- [存储知识](../features/storing-knowledge.md) — 数据建模深度探索
- [使用 SPARQL 查询](../features/querying-with-sparql.md) — 完整的查询语言
- [验证数据质量](../features/validating-data-quality.md) — 高级 SHACL 模式
- [推理和推理](../features/reasoning-and-inference.md) — Datalog、RDFS、OWL RL