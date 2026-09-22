# §2.1 存储知识

## 是什么和为什么

pg_ripple 将数据存储为 **RDF 三元组** — 用于表示知识的 W3C 标准。每个事实都是一个三部分语句：**主语**、**谓词**和**对象**。这种结构表面上很简单，但功能强大到足以对任何领域建模——从书目记录和生物医学本体到企业知识图。

为什么使用三元组而不是表格？

- **无模式的演变**：添加新的谓词无需 ALTER TABLE。
- **自然链接**：每个实体都是一个 IRI — 跨数据集的链接是免费的。
- **基于标准**：SPARQL、SHACL、OWL 和数千个公共词汇表开箱即用。
- **可溯源**：RDF-star 允许您使用置信度分数、来源和时间戳注释各个事实。

pg_ripple 使用 **Vertical Partitioning (VP)** 在 PostgreSQL 中存储三元组 — 每个谓词一个内部表，所有值都字典编码为 `BIGINT`。您永远不会看到这些机制；您通过 `insert_triple()`、`load_turtle()` 和 SPARQL 进行交互。

---

## 它是如何工作的

### 三元组模型

每个 RDF 三元组的形式为：

```
<subject>  <predicate>  <object> .
```

- **主语**：您正在描述的事物（始终是 IRI 或空节点）。
- **谓词**：关系或属性（始终是 IRI）。
- **对象**：值 — IRI（另一个实体）、字面量（字符串、数字、日期）或空节点。

```admonish note
IRI（国际化资源标识符）看起来像 URL，但它们是标识符，不一定是 Web 地址。`<https://example.org/paper/42>` 标识一篇论文 — 它不需要解析为网页。
```

### 命名图

三元组可以分组到**命名图**中 — 由 IRI 标识的逻辑分区。这对于：

- 跟踪出处："这些三元组来自 PubMed"
- 多租户：每个客户一个图
- 推理输出：派生的三元组进入单独的图

pg_ripple 使用图 ID `0` 作为**默认图**（没有显式图的三元组）。命名图通过字典编码获得正整数 ID。

### 空节点

空节点是匿名标识符 — 它们表示"某物存在"而不赋予全局 IRI。pg_ripple 使用 `_:` 前缀编码空节点：

```sql
SELECT pg_ripple.insert_triple(
    '_:review1',
    '<https://schema.org/author>',
    '<https://example.org/person/alice>'
);
```

```admonish warning
空节点是**文档作用域**的。两个单独的 `load_turtle()` 调用都使用 `_:x` 将创建两个不同的内部标识符。如果您需要稳定的跨文档身份，请使用 IRI。
```

### RDF-Star（引用三元组）

RDF-star 允许您对其他语句进行**陈述**。这对于出处、置信度分数和时间注释至关重要。

引用三元组包装 `<< subject predicate object >>`，可以作为主语或对象出现在另一个三元组中：

```sql
-- "Paper42 由 Alice 创作这一事实的置信度为 0.95"
SELECT pg_ripple.insert_triple(
    '<< <https://example.org/paper/42> <https://purl.org/dc/terms/creator> <https://example.org/person/alice> >>',
    '<https://example.org/confidence>',
    '"0.95"^^<http://www.w3.org/2001/XMLSchema#decimal>'
);
```

### 字典编码

每个 IRI、空节点和字面量在存储之前通过 XXH3-128 哈希映射到 `BIGINT` (i64)。VP 表仅包含整数 — 这使得连接快速且存储紧凑。您无需考虑编码；pg_ripple 透明地处理它。

### HTAP 存储架构

pg_ripple 使用混合事务/分析处理 (HTAP) 分割同时提供快速写入和分析读取：

```mermaid
flowchart LR
    subgraph Write Path
        A[INSERT triple] --> B[vp_{id}_delta\nheap + B-tree]
    end
    subgraph Read Path
        C[SPARQL query] --> D["(main EXCEPT tombstones)\nUNION ALL delta"]
    end
    subgraph Background Merge Worker
        E[vp_{id}_main\nBRIN index] --> F[Fresh vp_{id}_main]
        B --> F
        G[廈_event {vp_{id}_tombstones}] --> F
    end
    B --> D
    E --> D
    G --> D
    F --> E
```

- **Delta 表** (`vp_{id}_delta`)：通过堆 + B-tree 接收所有新写入。
- **Main 表** (`vp_{id}_main`)：历史 BRIN 索引分区，针对读取进行了优化。
- **Tombstones** (`vp_{id}_tombstones`)：已删除的主驻留三元组被记录在这里。
- **合并工作线程**：定期将 main + delta（减去墓碑）合并到一个新的 main 中，然后丢弃旧的 delta。

读取始终看到完整的、一致的视图：`(main EXCEPT tombstones) UNION ALL delta`。

---

## 详细的示例

本章中的示例使用书目数据集：论文、作者、机构、期刊和引用。

### 设置前缀

注册命名空间前缀，以便 SPARQL 查询可读：

```sql
SELECT pg_ripple.register_prefix('ex',    'https://example.org/');
SELECT pg_ripple.register_prefix('dct',   'http://purl.org/dc/terms/');
SELECT pg_ripple.register_prefix('foaf',  'http://xmlns.com/foaf/0.1/');
SELECT pg_ripple.register_prefix('bibo',  'http://purl.org/ontology/bibo/');
SELECT pg_ripple.register_prefix('schema','https://schema.org/');
SELECT pg_ripple.register_prefix('xsd',   'http://www.w3.org/2001/XMLSchema#');
```

### 插入单个三元组

```sql
-- 创建一篇论文
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>',
    '<http://purl.org/ontology/bibo/AcademicArticle>'
);

-- 添加标题
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/title>',
    '"Knowledge Graphs in Practice"'
);

-- 添加作者
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/creator>',
    '<https://example.org/person/alice>'
);

-- 作者元数据
SELECT pg_ripple.insert_triple(
    '<https://example.org/person/alice>',
    '<http://xmlns.com/foaf/0.1/name>',
    '"Alice Johnson"'
);

SELECT pg_ripple.insert_triple(
    '<https://example.org/person/alice>',
    '<https://schema.org/affiliation>',
    '<https://example.org/institution/mit>'
);

-- 机构元数据
SELECT pg_ripple.insert_triple(
    '<https://example.org/institution/mit>',
    '<http://xmlns.com/foaf/0.1/name>',
    '"Massachusetts Institute of Technology"'
);
```

### 使用 Turtle 加载完整数据集

对于批量数据，Turtle 格式更自然：

```sql
SELECT pg_ripple.load_turtle('
@prefix ex:     <https://example.org/> .
@prefix dct:    <http://purl.org/dc/terms/> .
@prefix foaf:   <http://xmlns.com/foaf/0.1/> .
@prefix bibo:   <http://purl.org/ontology/bibo/> .
@prefix schema: <https://schema.org/> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .

ex:paper/42 a bibo:AcademicArticle ;
    dct:title "Knowledge Graphs in Practice" ;
    dct:creator ex:person/alice, ex:person/bob ;
    dct:date "2024-03-15"^^xsd:date ;
    bibo:citedBy ex:paper/99 ;
    schema:keywords "knowledge graph", "RDF", "SPARQL" .

ex:paper/99 a bibo:AcademicArticle ;
    dct:title "Graph Neural Networks for Entity Resolution" ;
    dct:creator ex:person/carol ;
    bibo:cites ex:paper/42 .

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

### 使用命名图

将不同来源的三元组存储在单独的图中：

```sql
-- 为不同的数据源创建命名图
SELECT pg_ripple.create_graph('https://example.org/graph/pubmed');
SELECT pg_ripple.create_graph('https://example.org/graph/arxiv');

-- 将 PubMed 数据加载到其图中
SELECT pg_ripple.load_turtle_into_graph('
@prefix ex:   <https://example.org/> .
@prefix dct:  <http://purl.org/dc/terms/> .
@prefix bibo: <http://purl.org/ontology/bibo/> .

ex:paper/100 a bibo:AcademicArticle ;
    dct:title "Drug Interaction Networks" ;
    dct:creator ex:person/dave .
', 'https://example.org/graph/pubmed');

-- 将 arXiv 数据加载到其图中
SELECT pg_ripple.load_turtle_into_graph('
@prefix ex:   <https://example.org/> .
@prefix dct:  <http://purl.org/dc/terms/> .
@prefix bibo: <http://purl.org/ontology/bibo/> .

ex:paper/200 a bibo:AcademicArticle ;
    dct:title "Transformer Architectures for NLP" ;
    dct:creator ex:person/eve .
', 'https://example.org/graph/arxiv');

-- 列出所有命名图
SELECT * FROM pg_ripple.list_graphs();
```

### 用于出处和置信度的 RDF-Star

使用出处元数据注释引用：

```sql
-- 记录 Paper 42 引用 Paper 99（基本事实）
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/ontology/bibo/cites>',
    '<https://example.org/paper/99>'
);

-- 使用置信度分数注释此引用
SELECT pg_ripple.insert_triple(
    '<< <https://example.org/paper/42> <http://purl.org/ontology/bibo/cites> <https://example.org/paper/99> >>',
    '<https://example.org/confidence>',
    '"0.92"^^<http://www.w3.org/2001/XMLSchema#decimal>'
);

-- 记录谁断言了这个引用
SELECT pg_ripple.insert_triple(
    '<< <https://example.org/paper/42> <http://purl.org/ontology/bibo/cites> <https://example.org/paper/99> >>',
    '<http://purl.org/dc/terms/source>',
    '<https://example.org/system/citation-extractor>'
);
```

### 将关系 schema 转换为 RDF

假设您有一个带有表 `papers`、`authors` 和 `affiliations` 的关系数据库：

| papers.id | papers.title | papers.year |
|---|---|---|
| 42 | Knowledge Graphs in Practice | 2024 |

| authors.id | authors.name | authors.institution_id |
|---|---|---|
| 1 | Alice Johnson | 10 |

映射模式：

1. **每一行成为主语 IRI**：`<https://example.org/paper/{id}>`
2. **每一列成为谓词**：使用标准词汇表（Dublin Core、Schema.org、FOAF）
3. **外键成为对象 IRI**：`authors.institution_id = 10` → `<https://example.org/institution/10>`
4. **标量值成为字面量**：`papers.title` → `"Knowledge Graphs in Practice"`

```sql
-- 从 papers 表中获取的行 → 三元组
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>',
    '<http://purl.org/ontology/bibo/AcademicArticle>'
);
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/title>',
    '"Knowledge Graphs in Practice"'
);
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/date>',
    '"2024"^^<http://www.w3.org/2001/XMLSchema#gYear>'
);

-- 外键 → IRI 链接
SELECT pg_ripple.insert_triple(
    '<https://example.org/person/1>',
    '<https://schema.org/affiliation>',
    '<https://example.org/institution/10>'
);
```

---

## 常见模式

### 模式：类型层次结构

使用 `rdf:type` 和 `rdfs:subClassOf` 创建类型层次结构：

```sql
SELECT pg_ripple.load_turtle('
@prefix ex:   <https://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix bibo: <http://purl.org/ontology/bibo/> .

bibo:AcademicArticle rdfs:subClassOf bibo:Article .
bibo:Article rdfs:subClassOf bibo:Document .

ex:paper/42 a bibo:AcademicArticle .
');
```

启用 RDFS 推理（请参阅 [§2.5](../features/reasoning-and-inference.md)）后，pg_ripple 可以自动派生 `ex:paper/42` 也是 `bibo:Article` 和 `bibo:Document`。

### 模式：多值属性

与关系列不同，RDF 谓词自然是多值的：

```sql
-- 一篇论文可以有多个作者 — 只需插入多个三元组
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/creator>',
    '<https://example.org/person/alice>'
);
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/creator>',
    '<https://example.org/person/bob>'
);
```

### 模式：类型化和带语言标签的字符串字面量

```sql
-- 类型化字面量（日期）
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/date>',
    '"2024-03-15"^^<http://www.w3.org/2001/XMLSchema#date>'
);

-- 带语言标签的字符串
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/title>',
    '"Knowledge Graphs in Practice"@en'
);
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/title>',
    '"Wissensgraphen in der Praxis"@de'
);
```

### 模式：使用 RDF-Star 与命名图进行重新化

跟踪谁说了什么的两种方法：

**RDF-star** — 注释各个三元组：

```sql
SELECT pg_ripple.insert_triple(
    '<< <https://example.org/paper/42> <http://purl.org/dc/terms/creator> <https://example.org/person/alice> >>',
    '<http://purl.org/dc/terms/source>',
    '<https://example.org/dataset/pubmed>'
);
```

**命名图** — 按来源分组三元组：

```sql
SELECT pg_ripple.load_turtle_into_graph('
@prefix ex:  <https://example.org/> .
@prefix dct: <http://purl.org/dc/terms/> .
ex:paper/42 dct:creator ex:person/alice .
', 'https://example.org/dataset/pubmed');
```

当同一实体的不同三元组具有不同的出处时，请使用 RDF-star。当整个批次共享同一来源时，请使用命名图。

---

## 性能和权衡

| 方法 | 插入率 | 查询灵活性 | 存储开销 |
|---|---|---|---|
| `insert_triple()` | ~5,000 三元组/秒 | 完整 | 最高（每次调用的开销）|
| `load_turtle()` | ~50,000 三元组/秒 | 完整 | 低（批量字典编码）|
| `load_turtle_file()` | ~100,000 三元组/秒 | 完整 | 最低（服务器端流式传输）|

- **字典缓存**：频繁使用的 IRI（谓词、常见类型）保留在共享内存 LRU 缓存中。使用 `SELECT pg_ripple.cache_stats()` 检查命中率。
- **VP 表提升**：少于 1,000 个三元组的谓词共享 `vp_rare` 合并表。一旦谓词超过阈值，它将获得自己的专用 VP 表，具有双 B 树索引。
- **命名图开销**：`g` 列为每个三元组添加 8 个字节。如果您不需要命名图，使用默认图（默认）可以避免图 ID 查找的成本。

```admonish tip
在大批量加载之后，对内部表运行 `ANALYZE` 以更新 PostgreSQL 规划器统计信息：
```sql
SELECT pg_ripple.vacuum();
```
```

---

## 陷阱和调试

**IRI 格式**：所有 IRI 必须在函数调用中用尖括号（`<...>`）括起来。忘记括号是最常见的错误：

```sql
-- 错误：将被视为普通字面量
SELECT pg_ripple.insert_triple(
    'https://example.org/paper/42',
    'http://purl.org/dc/terms/title',
    '"Hello"'
);

-- 正确：IRI 用尖括号括起来
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/42>',
    '<http://purl.org/dc/terms/title>',
    '"Hello"'
);
```

**空节点作用域**：来自单独 `load_turtle()` 调用的空节点是独立的。两次使用 `_:x` 的调用创建两个不同的实体。

**字面量引用**：字面量必须在单引号 SQL 字符串中用双引号括起来。类型化字面量使用 `^^<datatype>` 后缀：

```sql
-- 普通字符串
'"Hello"'

-- 类型化整数
'"42"^^<http://www.w3.org/2001/XMLSchema#integer>'

-- 带语言标签的字符串
'"Bonjour"@fr'
```

**检查存储的内容**：使用 `find_triples()` 和通配符检查数据：

```sql
-- 关于 Paper 42 的所有三元组
SELECT * FROM pg_ripple.find_triples(
    '<https://example.org/paper/42>', NULL, NULL
);

-- 使用 dct:creator 谓词的所有三元组
SELECT * FROM pg_ripple.find_triples(
    NULL, '<http://purl.org/dc/terms/creator>', NULL
);

-- 总三元组数
SELECT pg_ripple.triple_count();
```

**重复三元组**：插入相同的 (s, p, o, g) 两次是幂等的 — 第二次插入返回现有的 SID。使用 `deduplicate_all()` 清理历史重复项。

---

## 后续步骤

- **[§2.2 加载数据](../features/loading-data.md)** — 使用性能调整批量加载所有 RDF 格式。
- **[§2.3 使用 SPARQL 查询](../features/querying-with-sparql.md)** — 查询您存储的三元组。
- **[§2.4 验证数据质量](../features/validating-data-quality.md)** — 使用 SHACL 强制执行 schema 约束。

## 进一步阅读

- [博客：RDF-star — 关于语句的语句](https://github.com/trickle-labs/pg-ripple/blob/main/blog/rdf-star-statements-about-statements.md) — 三元组上的边属性和出处
- [博客：Vertical Partitioning 解释](https://github.com/trickle-labs/pg-ripple/blob/main/blog/vertical-partitioning-explained.md) — 为什么 pg_ripple 为每个谓词存储一个表