# §2.2 加载数据

## 是什么和为什么

将数据加载到 pg_ripple 是构建知识图的第一步。pg_ripple 支持所有主要 RDF 序列化格式，并提供针对不同场景调整的三种加载模式：内联字符串加载、服务器端文件加载和单三元组插入。

选择正确的格式和加载模式很重要。1000 万三元组的数据集通过 `insert_triple()` 在循环中加载需要数小时；通过 `load_ntriples_file()` 从服务器端 N-Triples 文件加载的相同数据集在几分钟内完成。

---

## 它是如何工作的

### 支持的格式

| 格式 | 函数（字符串）| 函数（文件）| 命名图 | 注释 |
|---|---|---|---|---|
| **Turtle** | `load_turtle()` | `load_turtle_file()` | 否（使用 `load_turtle_into_graph()`）| 人类可读；支持前缀、RDF-star |
| **N-Triples** | `load_ntriples()` | `load_ntriples_file()` | 否（使用 `load_ntriples_into_graph()`）| 每行一个三元组；解析最快 |
| **N-Quads** | `load_nquads()` | `load_nquads_file()` | 是（内联）| N-Triples + 第四图列 |
| **TriG** | `load_trig()` | `load_trig_file()` | 是（内联）| Turtle + 命名图块 |
| **RDF/XML** | `load_rdfxml()` | `load_rdfxml_file()` | 否（使用 `load_rdfxml_into_graph()`）| 旧版 XML 格式；广泛支持 |

### 三种加载模式

**模式 1：字符串加载** — 将 RDF 文本作为 SQL 字符串参数传递。最适合中小型数据集（最多几 MB）和交互式使用：

```sql
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
ex:paper/1 ex:title "Hello World" .
');
```

**模式 2：服务器端文件加载** — 从 PostgreSQL 服务器的文件系统读取。最适合大型数据集。需要的超级用户权限：

```sql
SELECT pg_ripple.load_turtle_file('/data/papers.ttl');
```

**模式 3：单三元组插入** — 一次插入一个三元组。最适合从应用程序代码进行实时摄取：

```sql
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/1>',
    '<https://example.org/title>',
    '"Hello World"'
);
```

### 加载管道

无论格式如何，每个加载器都遵循相同的内部管道：

1. **解析** — 将 RDF 序列化反序列化为 (subject, predicate, object, graph) 四元组。
2. **编码** — 使用批量 `ON CONFLICT DO NOTHING ... RETURNING` 将每个 IRI、空节点和字面量字典编码为 `BIGINT` ID。
3. **路由** — 在 `_pg_ripple.predicates` 中查找谓词以找到目标 VP 表（或 `vp_rare`）。
4. **插入** — 将编码的 `(s, o, g)` 行批量插入到适当的 VP delta 表中。

```admonish tip
字符串加载器在单个事务中处理整个输入。如果使用 `strict = true` 时任何三元组无法解析，则整个加载将回滚。使用 `strict = false`（默认），将跳过格式错误的三元组并发出 WARNING。
```

---

## 详细的示例

### 加载 Turtle

手写数据最常见的格式：

```sql
SELECT pg_ripple.load_turtle('
@prefix ex:    <https://example.org/> .
@prefix dct:   <http://purl.org/dc/terms/> .
@prefix foaf:  <http://xmlns.com/foaf/0.1/> .
@prefix bibo:  <http://purl.org/ontology/bibo/> .
@prefix schema: <https://schema.org/> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .

ex:paper/42 a bibo:AcademicArticle ;
    dct:title "Knowledge Graphs in Practice"@en ;
    dct:creator ex:person/alice, ex:person/bob ;
    dct:date "2024-03-15"^^xsd:date ;
    bibo:citedBy ex:paper/99 ;
    schema:keywords "knowledge graph", "RDF", "SPARQL" .

ex:paper/99 a bibo:AcademicArticle ;
    dct:title "Graph Neural Networks for Entity Resolution" ;
    dct:creator ex:person/carol .

ex:person/alice foaf:name "Alice Johnson" ;
    schema:affiliation ex:institution/mit .

ex:person/bob foaf:name "Bob Smith" ;
    schema:affiliation ex:institution/stanford .

ex:person/carol foaf:name "Carol Williams" ;
    schema:affiliation ex:institution/mit .

ex:institution/mit foaf:name "Massachusetts Institute of Technology" .
ex:institution/stanford foaf:name "Stanford University" .
');
```

该函数返回加载的三元组数：

```sql
-- 返回：15
```

### 加载 N-Triples

N-Triples 是每行一个三元组，没有缩写 — 非常适合机器生成的数据：

```sql
SELECT pg_ripple.load_ntriples('
<https://example.org/paper/42> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://purl.org/ontology/bibo/AcademicArticle> .
<https://example.org/paper/42> <http://purl.org/dc/terms/title> "Knowledge Graphs in Practice" .
<https://example.org/paper/42> <http://purl.org/dc/terms/creator> <https://example.org/person/alice> .
<https://example.org/paper/42> <http://purl.org/dc/terms/creator> <https://example.org/person/bob> .
');
```

### 加载 N-Quads（带命名图）

N-Quads 使用第四个字段扩展 N-Triples 以包含图 IRI：

```sql
SELECT pg_ripple.load_nquads('
<https://example.org/paper/42> <http://purl.org/dc/terms/title> "Knowledge Graphs in Practice" <https://example.org/graph/pubmed> .
<https://example.org/paper/99> <http://purl.org/dc/terms/title> "Graph Neural Networks" <https://example.org/graph/arxiv> .
<https://example.org/paper/42> <http://purl.org/dc/terms/creator> <https://example.org/person/alice> <https://example.org/graph/pubmed> .
');
```

### 加载 TriG（带命名图的 Turtle）

TriG 在 `GRAPH { }` 部分中包装 Turtle 块：

```sql
SELECT pg_ripple.load_trig('
@prefix ex:  <https://example.org/> .
@prefix dct: <http://purl.org/dc/terms/> .
@prefix bibo: <http://purl.org/ontology/bibo/> .

GRAPH ex:graph/pubmed {
    ex:paper/100 a bibo:AcademicArticle ;
        dct:title "Drug Interaction Networks" ;
        dct:creator ex:person/dave .
}

GRAPH ex:graph/arxiv {
    ex:paper/200 a bibo:AcademicArticle ;
        dct:title "Transformer Architectures for NLP" ;
        dct:creator ex:person/eve .
}
');
```

### 加载 RDF/XML

RDF 的原始 XML 序列化 — 在较旧的数据集和 OWL 本体中很常见：

```sql
SELECT pg_ripple.load_rdfxml('
<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:dct="http://purl.org/dc/terms/"
         xmlns:bibo="http://purl.org/ontology/bibo/">
  <bibo:AcademicArticle rdf:about="https://example.org/paper/42">
    <dct:title>Knowledge Graphs in Practice</dct:title>
    <dct:creator rdf:resource="https://example.org/person/alice"/>
  </bibo:AcademicArticle>
</rdf:RDF>
');
```

### 从服务器端文件加载

对于大型数据集，服务器端文件加载避免通过 SQL 协议传输数据：

```sql
-- 加载大型 N-Triples 转储（需要超级用户）
SELECT pg_ripple.load_ntriples_file('/data/exports/papers.nt');

-- 使用严格解析加载 Turtle（任何错误时中止）
SELECT pg_ripple.load_turtle_file('/data/exports/ontology.ttl', true);

-- 加载到特定的命名图中
SELECT pg_ripple.load_turtle_file_into_graph(
    '/data/exports/pubmed.ttl',
    'https://example.org/graph/pubmed'
);
```

```admonish warning
文件加载函数从 **PostgreSQL 服务器的** 文件系统读取，而不是从客户端读取。路径必须可被 `postgres` 操作系统用户访问。这些是出于安全原因需要超级用户权限。
```

### 加载 Turtle-Star（RDF-Star）

pg_ripple 的 Turtle 解析器原生支持 RDF-star 引用三元组：

```sql
SELECT pg_ripple.load_turtle('
@prefix ex:  <https://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<< ex:paper/42 ex:cites ex:paper/99 >> ex:confidence "0.92"^^xsd:decimal .
<< ex:paper/42 ex:cites ex:paper/99 >> ex:source ex:system/citation-extractor .
');
```

### 加载到命名图

将数据加载到没有 TriG/N-Quads 格式的特定图中：

```sql
-- 首先创建图（可选 — 加载时自动创建）
SELECT pg_ripple.create_graph('https://example.org/graph/2024');

-- 将 Turtle 加载到命名图中
SELECT pg_ripple.load_turtle_into_graph('
@prefix ex:  <https://example.org/> .
@prefix dct: <http://purl.org/dc/terms/> .
@prefix bibo: <http://purl.org/ontology/bibo/> .

ex:paper/300 a bibo:AcademicArticle ;
    dct:title "New Findings in Graph Theory" ;
    dct:creator ex:person/frank .
', 'https://example.org/graph/2024');
```

### 使用 SPARQL Update 进行加载

SPARQL `INSERT DATA` 是添加三元组的另一种方式：

```sql
SELECT pg_ripple.sparql_update('
PREFIX ex:   <https://example.org/>
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>

INSERT DATA {
    ex:paper/500 a bibo:AcademicArticle ;
        dct:title "SPARQL Performance Tuning" ;
        dct:creator ex:person/alice .
}
');
```

---

## 常见模式

### 模式：ETL 管道

典型的 ETL 管道分阶段加载数据：

```sql
-- 第 1 步：加载本体
SELECT pg_ripple.load_turtle_file('/data/ontology.ttl');

-- 第 2 步：加载参考数据
SELECT pg_ripple.load_ntriples_file('/data/institutions.nt');

-- 第 3 步：加载主数据集
SELECT pg_ripple.load_ntriples_file('/data/papers.nt');

-- 第 4 步：将补充数据加载到命名图中
SELECT pg_ripple.load_nquads_file('/data/citations.nq');

-- 第 5 步：更新统计信息
SELECT pg_ripple.vacuum();

-- 第 6 步：验证加载
SELECT pg_ripple.triple_count();
SELECT pg_ripple.stats();
```

### 模式：增量加载

对于流式数据摄取，请在应用程序代码中使用 `insert_triple()`：

```sql
-- 应用程序在事件到达时插入三元组
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/new123>',
    '<http://purl.org/dc/terms/title>',
    '"Just Published: A New Study"'
);

-- 定期压缩 HTAP 表
SELECT pg_ripple.compact();
```

### 模式：严格与宽松解析

```sql
-- 宽松（默认）：跳过错误的三元组，发出 WARNING
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
ex:good ex:rel ex:target .
ex:bad ex:rel "unclosed literal .
ex:also_good ex:rel ex:other .
', false);
-- 返回：2（跳过了错误的三元组）

-- 严格：任何解析错误时中止
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
ex:good ex:rel ex:target .
ex:bad ex:rel "unclosed literal .
', true);
-- ERROR：第 4 行的 Turtle 解析错误
```

### 模式：加载 OWL 本体

```sql
-- 从文件扩展名自动检测格式（.ttl、.nt、.xml、.rdf、.owl）
SELECT pg_ripple.load_owl_ontology('/data/ontologies/foaf.rdf');

-- 或者显式作为 RDF/XML 加载
SELECT pg_ripple.load_rdfxml_file('/data/ontologies/dublin_core.rdf');
```

---

## 性能和权衡

### 按加载模式的吞吐量

| 模式 | 相对成本 | 案例 |
|---|---|---|
| `insert_triple()` | 每个三元组开销最高 | 实时摄取、单三元组更新 |
| `load_turtle()` / `load_ntriples()` | 批量 | 最多几 MB 的交互式批量加载 |
| `load_ntriples_file()` | 流式传输，通常解析器开销最低 | 大型服务器端文件 |
| `load_turtle_file()` | 使用 Turtle 解析的流式传输 | 大型服务器端 Turtle 文件 |

```admonish note
N-Triples 始终比 Turtle 快，因为它不需要前缀展开或缩写处理。要在大型数据集上获得最大吞吐量，首先转换为 N-Triples：`rapper -i turtle -o ntriples data.ttl > data.nt`
```

### 格式选择指南

| 场景 | 推荐格式 |
|---|---|
| 手写数据 | Turtle（可读，支持前缀）|
| 机器生成的导出 | N-Triples（解析最快，每行一个三元组）|
| 具有命名图的数据 | N-Quads 或 TriG |
| 旧版 XML 数据集 | RDF/XML |
| 最大加载速度 | N-Triples 通过 `load_ntriples_file()` |

### 加载后分析

加载大量数据后，更新 PostgreSQL 规划器统计信息：

```sql
-- 对所有 VP 表运行 ANALYZE
SELECT pg_ripple.vacuum();
```

这确保查询规划器具有准确的行计数估计以进行连接排序。

### 批量大小注意事项

对于基于字符串的加载器，整个输入在单个事务中处理。非常大的字符串（数百 MB）可能会导致内存压力。对于超过 50 MB 的数据集，首选基于文件的加载：

```sql
-- 不要使用巨大的字符串字面量：
-- SELECT pg_ripple.load_ntriples('... 1 亿行 ...');

-- 使用文件加载：
SELECT pg_ripple.load_ntriples_file('/data/huge_dataset.nt');
```

---

## 陷阱和调试

### 空节点作用域

每次 `load_turtle()` 调用都会创建一个新的空节点作用域。使用不同名字的两次单独调用 `_:x` 产生两个不同的内部 ID：

```sql
-- 调用 1：`_:`x 映射到内部 ID 12345
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
_:x ex:name "Alice" .
ex:paper/1 ex:author _:x .
');

-- 调用 2：`_:`x 映射到内部 ID 67890（不同！）
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
_:x ex:name "Bob" .
ex:paper/2 ex:author _:x .
');
```

如果需要在加载之间使用相同的匿名节点，请改用稳定的 IRI：

```sql
SELECT pg_ripple.insert_triple(
    '<https://example.org/anon/shared-node>',
    '<https://example.org/name>',
    '"Shared Entity"'
);
```

### 字符编码

所有加载器都期望 UTF-8 输入。非 UTF-8 数据会导致解析错误：

```sql
-- 如果您的文件是 Latin-1，请先转换：
-- iconv -f ISO-8859-1 -t UTF-8 data.nt > data_utf8.nt
SELECT pg_ripple.load_ntriples_file('/data/data_utf8.nt');
```

### 验证加载的数据

加载后，使用 `find_triples()` 或 `triple_count()` 进行验证：

```sql
-- 检查总三元组
SELECT pg_ripple.triple_count();

-- 检查特定三元组
SELECT * FROM pg_ripple.find_triples(
    '<https://example.org/paper/42>', NULL, NULL
);

-- 检查每谓词统计信息
SELECT pg_ripple.stats();
```

### 文件路径错误

文件加载器从服务器文件系统读取。常见错误：

```sql
-- 错误：无法打开文件 "/data/papers.nt"：没有此类文件或目录
-- 修复：确保文件存在并且可被 postgres 操作系统用户读取

-- 错误：函数 load_turtle_file 权限被拒绝
-- 修复：文件加载器需要超级用户；为非超级用户使用字符串加载器
```

### 重复处理

加载相同的数据两次不会创建重复项 — VP 表使用 `ON CONFLICT DO NOTHING`：

```sql
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
ex:a ex:rel ex:b .
');
-- 返回：1

SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
ex:a ex:rel ex:b .
');
-- 返回：0（已存在）
```

---

## 后续步骤

- **[§2.1 存储知识](../features/storing-knowledge.md)** — 了解三元组模型和命名图。
- **[§2.3 使用 SPARQL 查询](../features/querying-with-sparql.md)** — 查询您加载的数据。
- **[§2.6 导出和共享](../features/exporting-and-sharing.md)** — 以各种格式导出数据。