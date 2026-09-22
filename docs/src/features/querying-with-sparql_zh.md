# §2.3 使用 SPARQL 查询

## 是什么和为什么

SPARQL 是 RDF 数据的 W3C 标准查询语言 — 知识图世界的 SQL。pg_ripple 在后台将 SPARQL 查询转换为优化的 PostgreSQL SQL，因此您可以获得 SPARQL 的表达能力和成熟关系引擎的性能。

为什么使用 SPARQL 而不是对 VP 表的原始 SQL？

- **图模式匹配**：自然地查找路径、循环和子图形状。
- **属性路径**：使用 `+`、`*`、`?` 遍历可变长度关系。
- **联邦**：查询远程 SPARQL 端点以及本地数据。
- **标准合规性**：查询可在三元组中保持一致。
- **更新支持**：`INSERT DATA` 和 `DELETE DATA` 用于编程修改。

pg_ripple 支持所有四种 SPARQL 查询形式（SELECT、CONSTRUCT、DESCRIBE、ASK）和 SPARQL Update（`INSERT DATA`、`DELETE DATA`、`DELETE/INSERT WHERE`）。

---

## 它是如何工作的

### SPARQL 管道

1. **解析** — `spargebra` 将 SPARQL 文本解析为代数树。
2. **优化** — `sparopt` 应用代数优化（过滤器下推、连接重排序）。
3. **转换** — pg_ripple 的 SQL 生成器将代数转换为 PostgreSQL SQL，仅使用整数 VP 表连接。
4. **缓存** — 计划缓存按 SPARQL 文本哈希存储转换后的 SQL。
5. **执行** — SPI 执行 SQL；结果从整数 ID 批量解码回 IRI 和字面量。
6. **返回** — 每个结果行作为 JSONB 对象返回。

### 关键函数

| 函数 | 目的 |
|---|---|
| `sparql(query)` | 执行 SELECT 或 ASK；返回 JSONB 行 |
| `sparql_ask(query)` | 执行 ASK；返回布尔值 |
| `sparql_construct(query)` | 执行 CONSTRUCT；返回三元组 JSONB 行 |
| `sparql_construct_turtle(query)` | CONSTRUCT → Turtle 文本 |
| `sparql_construct_jsonld(query)` | CONSTRUCT → JSON-LD JSONB |
| `sparql_describe(query)` | 带 CBD 的 DESCRIBE；返回三元组 JSONB 行 |
| `sparql_describe_turtle(query)` | DESCRIBE → Turtle 文本 |
| `sparql_update(query)` | INSERT DATA / DELETE DATA；返回受影响计数 |
| `sparql_explain(query, analyze)` | 显示生成的 SQL 或 EXPLAIN ANALYZE 输出 |
| `explain_sparql(query, format)` | 带有 SQL、文本、JSON 或代数输出的扩展解释 |

---

## 详细的示例

所有示例假定已加载来自 [§2.1](../features/storing-knowledge.md) 和 [§2.2](../features/loading-data.md) 的书目数据集。

### 基本三元组模式

查找所有论文及其标题：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>

SELECT ?paper ?title
WHERE {
    ?paper a bibo:AcademicArticle .
    ?paper dct:title ?title .
}
');
```

每行都是一个 JSONB 对象，例如 `{"paper": "<https://example.org/paper/42>", "title": "\"Knowledge Graphs in Practice\""}`。

### 过滤结果

查找 2023 年之后发布的论文：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>
PREFIX xsd:  <http://www.w3.org/2001/XMLSchema#>

SELECT ?paper ?title ?date
WHERE {
    ?paper a bibo:AcademicArticle ;
           dct:title ?title ;
           dct:date ?date .
    FILTER (?date > "2023-01-01"^^xsd:date)
}
');
```

### OPTIONAL 模式

包括作者，即使他们没有附属机构：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX foaf:   <http://xmlns.com/foaf/0.1/>
PREFIX schema: <https://schema.org/>

SELECT ?paper ?authorName ?instName
WHERE {
    ?paper dct:creator ?author .
    ?author foaf:name ?authorName .
    OPTIONAL {
        ?author schema:affiliation ?inst .
        ?inst foaf:name ?instName .
    }
}
');
```

### UNION

查找是论文或人的实体：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX bibo: <http://purl.org/ontology/bibo/>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
PREFIX dct:  <http://purl.org/dc/terms/>

SELECT ?entity ?label
WHERE {
    {
        ?entity a bibo:AcademicArticle .
        ?entity dct:title ?label .
    }
    UNION
    {
        ?entity a foaf:Person .
        ?entity foaf:name ?label .
    }
}
');
```

### MINUS

查找没有引用的论文：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX bibo: <http://purl.org/ontology/bibo/>
PREFIX dct:  <http://purl.org/dc/terms/>

SELECT ?paper ?title
WHERE {
    ?paper a bibo:AcademicArticle ;
           dct:title ?title .
    MINUS {
        ?paper bibo:citedBy ?other .
    }
}
');
```

### 聚合

按机构计算论文数量：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX schema: <https://schema.org/>
PREFIX foaf:   <http://xmlns.com/foaf/0.1/>

SELECT ?instName (COUNT(DISTINCT ?paper) AS ?paperCount)
WHERE {
    ?paper dct:creator ?author .
    ?author schema:affiliation ?inst .
    ?inst foaf:name ?instName .
}
GROUP BY ?instName
ORDER BY DESC(?paperCount)
');
```

### 子查询

查找最高产的作者及其所有论文：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>

SELECT ?authorName ?paper ?title
WHERE {
    {
        SELECT ?author (COUNT(?p) AS ?count)
        WHERE {
            ?p dct:creator ?author .
        }
        GROUP BY ?author
        ORDER BY DESC(?count)
        LIMIT 1
    }
    ?author foaf:name ?authorName .
    ?paper dct:creator ?author ;
           dct:title ?title .
}
');
```

### 属性路径

属性路径允许您遍历可变长度关系。

**传递闭包（`+`）** — 通过子类层次结构查找实体所属的所有类：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?entity ?superClass
WHERE {
    ?entity rdf:type/rdfs:subClassOf+ ?superClass .
}
');
```

**零或多个（`*`）** — 包括起始节点：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?class ?ancestor
WHERE {
    ?class rdfs:subClassOf* ?ancestor .
}
');
```

**可选步骤（`?`）** — 零或一跳：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX schema: <https://schema.org/>
PREFIX foaf:   <http://xmlns.com/foaf/0.1/>

SELECT ?person ?nameOrInst
WHERE {
    ?person schema:affiliation? ?target .
    ?target foaf:name ?nameOrInst .
}
');
```

**序列路径（`/`）** — 链接属性：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX schema: <https://schema.org/>
PREFIX foaf:   <http://xmlns.com/foaf/0.1/>

SELECT ?paper ?instName
WHERE {
    ?paper dct:creator/schema:affiliation/foaf:name ?instName .
}
');
```

**替代路径（`|`）** — 匹配任一属性：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX schema: <https://schema.org/>

SELECT ?entity ?label
WHERE {
    ?entity (dct:title | schema:name) ?label .
}
');
```

**逆路径（`^`）** — 反向遍历：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct: <http://purl.org/dc/terms/>

SELECT ?author ?paper
WHERE {
    ?author ^dct:creator ?paper .
}
');
```

### GRAPH 模式

查询特定命名图中的数据：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct: <http://purl.org/dc/terms/>

SELECT ?paper ?title ?graph
WHERE {
    GRAPH ?graph {
        ?paper dct:title ?title .
    }
}
');
```

查询特定的命名图：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct: <http://purl.org/dc/terms/>

SELECT ?paper ?title
WHERE {
    GRAPH <https://example.org/graph/pubmed> {
        ?paper dct:title ?title .
    }
}
');
```

### ASK 查询

检查某些内容是否存在：

```sql
SELECT pg_ripple.sparql_ask('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>

ASK {
    ?paper a bibo:AcademicArticle ;
           dct:title "Knowledge Graphs in Practice" .
}
');
-- 返回：true
```

### CONSTRUCT 查询

从查询结果构建新三元组：

```sql
SELECT * FROM pg_ripple.sparql_construct('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX schema: <https://schema.org/>
PREFIX foaf:   <http://xmlns.com/foaf/0.1/>
PREFIX ex:     <https://example.org/>

CONSTRUCT {
    ?author ex:worksOn ?paper .
    ?paper ex:authoredAt ?inst .
}
WHERE {
    ?paper dct:creator ?author .
    ?author schema:affiliation ?inst .
}
');
```

以 Turtle 形式获取 CONSTRUCT 结果：

```sql
SELECT pg_ripple.sparql_construct_turtle('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX foaf:   <http://xmlns.com/foaf/0.1/>
PREFIX ex:     <https://example.org/>

CONSTRUCT {
    ?author ex:wrote ?paper .
}
WHERE {
    ?paper dct:creator ?author .
}
');
```

以 JSON-LD 形式获取 CONSTRUCT 结果：

```sql
SELECT pg_ripple.sparql_construct_jsonld('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
PREFIX ex:   <https://example.org/>

CONSTRUCT {
    ?author ex:wrote ?paper .
}
WHERE {
    ?paper dct:creator ?author .
}
');
```

### DESCRIBE 查询

使用简洁有界描述获取实体的所有内容：

```sql
SELECT * FROM pg_ripple.sparql_describe('
DESCRIBE <https://example.org/paper/42>
');
```

以 Turtle 形式获取描述：

```sql
SELECT * FROM pg_ripple.sparql_describe_turtle('
DESCRIBE <https://example.org/person/alice>
');
```

选择 describe 策略：

```sql
-- 对称 CBD：包括实体也是对象的三元组
SELECT * FROM pg_ripple.sparql_describe(
    'DESCRIBE <https://example.org/person/alice>',
    'scbd'
);
```

### SPARQL Update

插入新三元组：

```sql
SELECT pg_ripple.sparql_update('
PREFIX ex:  <https://example.org/>
PREFIX dct: <http://purl.org/dc/terms/>

INSERT DATA {
    ex:paper/600 a <http://purl.org/ontology/bibo/AcademicArticle> ;
        dct:title "Emerging Trends in Knowledge Graphs" ;
        dct:creator ex:person/alice .
}
');
-- 返回：3
```

删除特定三元组：

```sql
SELECT pg_ripple.sparql_update('
PREFIX ex:  <https://example.org/>
PREFIX dct: <http://purl.org/dc/terms/>

DELETE DATA {
    ex:paper/600 dct:title "Emerging Trends in Knowledge Graphs" .
}
');
-- 返回：1
```

### 使用 EXPLAIN 进行查询调试

在不执行的情况下查看生成的 SQL：

```sql
SELECT pg_ripple.sparql_explain('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>

SELECT ?paper ?title
WHERE {
    ?paper a bibo:AcademicArticle ;
           dct:title ?title .
}
', false);
```

运行 EXPLAIN ANALYZE 查看执行时间：

```sql
SELECT pg_ripple.sparql_explain('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>

SELECT ?paper ?title
WHERE {
    ?paper a bibo:AcademicArticle ;
           dct:title ?title .
}
', true);
```

使用带有格式选项的扩展解释：

```sql
-- 仅显示生成的 SQL
SELECT pg_ripple.explain_sparql('
PREFIX dct: <http://purl.org/dc/terms/>
SELECT ?paper ?title
WHERE { ?paper dct:title ?title }
', 'sql');

-- 以 JSON 形式显示 EXPLAIN ANALYZE（用于编程消费）
SELECT pg_ripple.explain_sparql('
PREFIX dct: <http://purl.org/dc/terms/>
SELECT ?paper ?title
WHERE { ?paper dct:title ?title }
', 'json');

-- 显示 spargebra 代数树
SELECT pg_ripple.explain_sparql('
PREFIX dct: <http://purl.org/dc/terms/>
SELECT ?paper ?title
WHERE { ?paper dct:title ?title }
', 'sparql_algebra');
```

---

## 常见模式

### 模式：星型查询（同一主语上的多个谓词）

优化器检测星型模式并将它们折叠为高效的多路连接：

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX schema: <https://schema.org/>
PREFIX bibo:   <http://purl.org/ontology/bibo/>

SELECT ?paper ?title ?date
WHERE {
    ?paper a bibo:AcademicArticle ;
           dct:title ?title ;
           dct:date ?date ;
           schema:keywords ?kw .
    FILTER (CONTAINS(?kw, "knowledge"))
}
');
```

### 模式：使用 FILTER EXISTS 进行存在性检查

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:  <http://purl.org/dc/terms/>
PREFIX bibo: <http://purl.org/ontology/bibo/>

SELECT ?paper ?title
WHERE {
    ?paper a bibo:AcademicArticle ;
           dct:title ?title .
    FILTER EXISTS {
        ?paper bibo:citedBy ?other .
    }
}
');
```

### 模式：用于参数化查询的 VALUES 子句

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct: <http://purl.org/dc/terms/>

SELECT ?paper ?title
WHERE {
    VALUES ?paper {
        <https://example.org/paper/42>
        <https://example.org/paper/99>
    }
    ?paper dct:title ?title .
}
');
```

### 模式：BIND 和计算值

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct: <http://purl.org/dc/terms/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?paper ?title ?yearLabel
WHERE {
    ?paper dct:title ?title ;
           dct:date ?date .
    BIND(YEAR(?date) AS ?year)
    BIND(CONCAT("Published in ", STR(?year)) AS ?yearLabel)
}
');
```

---

## 性能和权衡

### 计划缓存

pg_ripple 按 SPARQL 查询哈希缓存转换后的 SQL。重复查询跳过解析和转换步骤：

```sql
-- 检查缓存统计信息
SELECT pg_ripple.plan_cache_stats();
-- 返回：{"hits": 42, "misses": 5, "size": 5, "capacity": 128, "hit_rate": 0.89}

-- 重置缓存（例如，在架构更改之后）
SELECT pg_ripple.plan_cache_reset();
```

### 过滤器下推

对绑定常量的 SPARQL FILTER 在 SQL 生成之前被编码为整数。这意味着数据库比较的是整数，而不是字符串：

```sql
-- 这个 FILTER 作为整数比较下推：
SELECT * FROM pg_ripple.sparql('
PREFIX dct: <http://purl.org/dc/terms/>
SELECT ?paper WHERE {
    ?paper dct:creator <https://example.org/person/alice> .
}
');
```

### 属性路径深度限制

递归属性路径（`+`、`*`）编译为 `WITH RECURSIVE ... CYCLE`。GUC `pg_ripple.max_path_depth`（默认值：50）防止失控的递归：

```sql
-- 为深度层次结构增加深度
SET pg_ripple.max_path_depth = 100;
```

```admonish warning
在循环图上将 `max_path_depth` 设置得很高会导致查询缓慢。pg_ripple 使用 PostgreSQL 18 的 `CYCLE` 子句进行基于哈希的循环检测，但宽图仍然会累积许多中间行。
```

### 全文搜索集成

为特定谓词的快速文本搜索创建 GIN 索引：

```sql
-- 为全文搜索索引 dct:title 谓词
SELECT pg_ripple.fts_index('<http://purl.org/dc/terms/title>');

-- 然后对 dct:title 对象的 CONTAINS() 和 REGEX() 过滤器使用 GIN 索引
SELECT * FROM pg_ripple.sparql('
PREFIX dct: <http://purl.org/dc/terms/>
SELECT ?paper ?title
WHERE {
    ?paper dct:title ?title .
    FILTER (CONTAINS(?title, "Knowledge"))
}
');
```

或使用直接全文搜索函数：

```sql
SELECT * FROM pg_ripple.fts_search(
    'knowledge & graph',
    '<http://purl.org/dc/terms/title>'
);
```

---

## 陷阱和调试

### SPARQL 语法错误

pg_ripple 使用 `spargebra` 解析器，它提供精确的错误消息：

```sql
SELECT * FROM pg_ripple.sparql('
SELECT ?x WHERE { ?x ?p }
');
-- ERROR: SPARQL parse error: Expected '.' or '}' at line 2
```

在运行之前检查查询是否编译：

```sql
SELECT pg_ripple.explain_sparql('
PREFIX dct: <http://purl.org/dc/terms/>
SELECT ?paper WHERE { ?paper dct:title ?title }
', 'sql');
```

### 预期有结果但没有结果

常见原因：

1. **缺少尖括号**：SPARQL 中的 `dct:title` 需要 PREFIX 声明。没有它，解析器将其视为相对 IRI。
2. **错误的字面量格式**：`"42"` 是字符串，而不是数字。使用 `"42"^^xsd:integer`。
3. **区分大小写**：IRI 区分大小写。`<https://Example.org/X>` 和 `<https://example.org/x>` 是不同的。

通过检查存储的内容进行调试：

```sql
-- 检查谓词是否存在
SELECT * FROM pg_ripple.find_triples(
    NULL, '<http://purl.org/dc/terms/title>', NULL
);
```

### 查询缓慢

1. 使用 `sparql_explain()` 检查生成的 SQL。
2. 在大对 VP 表上查找顺序扫描 — 运行 `pg_ripple.vacuum()` 以更新统计信息。
3. 对于属性路径，检查 `max_path_depth` — 如果查询正在探索太多路径，则降低它。
4. 检查计划缓存命中率 — 低命中率意味着许多唯一查询被重复解析。

```sql
-- 第 1 步：查看执行计划
SELECT pg_ripple.sparql_explain('
PREFIX dct: <http://purl.org/dc/terms/>
SELECT ?paper ?title
WHERE { ?paper dct:title ?title }
', true);

-- 第 2 步：更新统计信息
SELECT pg_ripple.vacuum();

-- 第 3 步：检查计划缓存
SELECT pg_ripple.plan_cache_stats();
```

### SPARQL Update 限制

`sparql_update()` 支持 `INSERT DATA` 和 `DELETE DATA`（仅基础三元组）。也支持带有变量的基于模式的 `DELETE/INSERT WHERE`，用于灵活的图修改。对于程序化单三元组删除，请使用 `delete_triple()`。

---

## 后续步骤

- **[§2.4 验证数据质量](../features/validating-data-quality.md)** — 使用 SHACL 对数据强制约束。
- **[§2.5 推理和推理](../features/reasoning-and-inference.md)** — 使用 Datalog 规则派生新事实。
- **[§2.8 API 和集成](../features/apis-and-integration.md)** — 通过 HTTP 端点从应用程序代码访问 SPARQL。

## 进一步阅读

- [博客：SPARQL 到 SQL 的转换](https://github.com/trickle-labs/pg-ripple/blob/main/blog/sparql-to-sql-translation.md) — pg_ripple 如何将 SPARQL 编译为优化的 PostgreSQL SQL
- [博客：属性路径和递归 CTE](https://github.com/trickle-labs/pg-ripple/blob/main/blog/property-paths-recursive-ctes.md) — `*` 和 `+` 路径运算符的实现
- [博客：解释 SPARQL 查询计划](https://github.com/trickle-labs/pg-ripple/blob/main/blog/explain-sparql-query-plans.md) — 理解 SPARQL 查询调试器