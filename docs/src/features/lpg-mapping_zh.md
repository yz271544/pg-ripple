# Cypher / LPG → RDF 映射

如果您来自 Neo4j、Memgraph 或任何其他属性图数据库，您已经有了数据的心智模型。本页介绍**属性图概念如何映射到 RDF**，如何将属性图 schema 加载到 pg_ripple，以及如何在 SPARQL 中查询它。

您的图无需任何更改。映射是机械的并且是可逆的。

---

## 两个模型，一个想法

**标记属性图（LPG）** 和 **RDF** 都将知识表示为图。术语不同：

| 属性图 | RDF / pg_ripple |
|---|---|
| 节点 | 主语（IRI 或空节点）|
| 节点标签 | `rdf:type` 三元组 |
| 节点属性键 + 值 | 谓词 + 对象三元组 |
| 关系类型 | 谓词 IRI |
| 关系属性 | RDF-star 注释 `<< s p o >> key value` |
| `id()` 内部标识符 | IRI（您选择命名方案）|
| 命名图 | 命名图（相同概念）|
| 无等效项 | 数据类型注释的字面量（`"42"^^xsd:integer`）|

唯一的实质性区别是**关系属性**（LPG 中*边上的*属性）。在 RDF 中，这些成为 **RDF-star 引用三元组**。

---

## IRI 的命名方案

LPG 节点具有内部整数 ID。您需要将它们转换为 IRI。一个简单而稳健的方案：

```
https://example.org/node/{label}/{id}
```

例如，Neo4j 节点 `(:Person {id: 42, name: "Alice"})` 变为：

```
<https://example.org/node/Person/42>  rdf:type  <https://example.org/vocab/Person> .
<https://example.org/node/Person/42>  <https://example.org/vocab/name>  "Alice" .
```

如果您的节点具有域内有意义的唯一键（电子邮件、UUID、slug），请使用它而不是内部 ID — 这使得 IRI 在重新导入时保持稳定。

---

## 将 Cypher schema 转换为 RDF

### 节点和节点属性

**Cypher：**
```cypher
CREATE (:Person {name: "Alice", age: 30, active: true})
```

**RDF（Turtle）：**
```turtle
@prefix ex:  <https://example.org/node/Person/> .
@prefix voc: <https://example.org/vocab/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:alice  a              voc:Person ;
          voc:name       "Alice" ;
          voc:age        30 ;
          voc:active     true .
```

### 没有属性的关系

**Cypher：**
```cypher
MATCH (a:Person {name: "Alice"}), (b:Person {name: "Bob"})
CREATE (a)-[:KNOWS]->(b)
```

**RDF：**
```turtle
ex:alice  voc:KNOWS  ex:bob .
```

### *带*属性的关系（RDF-star）

**Cypher：**
```cypher
CREATE (a)-[:KNOWS {since: 2020, strength: 0.9}]->(b)
```

**RDF-star（Turtle-star）：**
```turtle
ex:alice  voc:KNOWS  ex:bob .

<< ex:alice  voc:KNOWS  ex:bob >>
    voc:since     2020 ;
    voc:strength  "0.9"^^xsd:decimal .
```

将其存储在 pg_ripple 中：

```sql
SELECT pg_ripple.load_turtle($TTL$
@prefix ex:  <https://example.org/node/Person/> .
@prefix voc: <https://example.org/vocab/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:alice  voc:KNOWS  ex:bob .
<< ex:alice  voc:KNOWS  ex:bob >>
    voc:since    "2020"^^xsd:integer ;
    voc:strength "0.9"^^xsd:decimal .
$TTL$);
```

---

## 将常见的 Cypher 查询转换为 SPARQL

### 按属性查找节点

```cypher
MATCH (p:Person {name: "Alice"}) RETURN p.age
```
```sparql
PREFIX voc: <https://example.org/vocab/>
SELECT ?age WHERE {
    ?p  a       voc:Person ;
        voc:name "Alice" ;
        voc:age  ?age .
}
```

### 遍历关系

```cypher
MATCH (a:Person {name: "Alice"})-[:KNOWS]->(b) RETURN b.name
```
```sparql
PREFIX ex:  <https://example.org/node/Person/>
PREFIX voc: <https://example.org/vocab/>
SELECT ?name WHERE {
    ex:alice  voc:KNOWS  ?b .
    ?b        voc:name   ?name .
}
```

### 多跳遍历（可变深度）

```cypher
MATCH (a:Person {name: "Alice"})-[:KNOWS*1..]->(b) RETURN b.name
```
```sparql
PREFIX ex:  <https://example.org/node/Person/>
PREFIX voc: <https://example.org/vocab/>
SELECT ?name WHERE {
    ex:alice  voc:KNOWS+  ?b .   # + = 一个或多个跳跃
    ?b        voc:name    ?name .
}
```

或者 `voc:KNOWS*` 表示零个或多个。

### 关系属性过滤器

```cypher
MATCH (a)-[r:KNOWS]->(b) WHERE r.strength > 0.8 RETURN a.name, b.name
```
```sparql
PREFIX voc: <https://example.org/vocab/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
SELECT ?aName ?bName WHERE {
    ?a voc:KNOWS ?b .
    << ?a voc:KNOWS ?b >> voc:strength ?str .
    FILTER(?str > "0.8"^^xsd:decimal)
    ?a voc:name ?aName .
    ?b voc:name ?bName .
}
```

### 聚合

```cypher
MATCH (p:Person)-[:KNOWS]->(friend) RETURN p.name, count(friend) AS friends
```
```sparql
PREFIX voc: <https://example.org/vocab/>
SELECT ?name (COUNT(?friend) AS ?friends) WHERE {
    ?p  a           voc:Person ;
        voc:name    ?name ;
        voc:KNOWS   ?friend .
}
GROUP BY ?name
ORDER BY DESC(?friends)
```

---

## 从 Neo4j 批量迁移

迁移 Neo4j 数据库的推荐路径：

1. **使用 `neo4j-admin dump` 导出** 或 APOC `export.graphml` 过程导出为 GraphML 或 CSV。
2. **转换为 Turtle** 使用一个小的 Python 脚本（或如果源是 JDBC 视图，则使用 R2RML）。
3. **加载到 pg_ripple** 使用 `pg_ripple.load_turtle_file()`。

CSV 导出的最小 Python 转换器：

```python
import csv, sys

BASE   = "https://example.org/node/"
VOCAB  = "https://example.org/vocab/"

# nodes.csv：nodeId、label、propKey1、propKey2、...
with open("nodes.csv") as f:
    for row in csv.DictReader(f):
        iri = f"<{BASE}{row['label']}/{row['nodeId']}>"
        print(f"{iri} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{VOCAB}{row['label']}> .")
        for k, v in row.items():
                if k not in ("nodeId", "label") and v:
                        print(f"{iri} <{VOCAB}{k}> {_literal(v)} .")
```

对于大型转储（> 1 亿个三元组），请使用通过 `load_ntriples_file()` 的 `COPY` 而不是 `load_turtle()` — N-Triples 批量流式传输，没有解析开销。

---

## 您相对于 LPG 获得了什么

一旦您的图位于 pg_ripple 中，您就可以使用大多数 LPG 数据库所没有的功能：

- **SHACL 验证** — 使用正式保证定义和强制图上的 schema。
- **OWL 推理** — 根据多个 schema 的 `owl:equivalentClass` 公理自动派生 `rdf:type` 断言。
- **联邦查询** — 在单个查询中将您的本地图与 Wikidata、DBpedia 或任何其他 SPARQL 端点连接起来。
- **向量 + 图混合** — 嵌入实体并运行 HNSW 相似性搜索，与 SPARQL 图遍历结合。
- **事务写入** — 在单个 PostgreSQL 事务中进行图写入、向量索引更新和关系表更新。

---

## 另请参阅

- [存储知识 — RDF-star](storing-knowledge.md) — 详细的关系属性。
- [使用 SPARQL 查询](querying-with-sparql.md) — 完整的 SPARQL 1.1 参考。
- [记录链接](record-linkage.md) — 如果迁移具有重叠实体的两个系统，则很有用。