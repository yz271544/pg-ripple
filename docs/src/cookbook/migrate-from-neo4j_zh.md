# 从 Neo4j 迁移到 pg_ripple

本配方引导您将 Neo4j 属性图迁移到 pg_ripple 的 RDF 三元组存储。您将导出 Neo4j 数据，将其转换为 RDF，加载到 pg_ripple 中，并验证迁移。

## 先决条件

- Neo4j 5.x（Community 或 Enterprise）
- pg_ripple v0.90.0 或更高版本
- Python 3.11+ 及 `neo4j` 和 `rdflib` 包

```bash
pip install neo4j rdflib
```

---

## 第 1 步：导出 Neo4j 图

使用 Neo4j Cypher shell 或 APOC 导出节点和关系。

### 选项 A：APOC 导出到 CSV

```cypher
-- 导出节点
CALL apoc.export.csv.query(
  "MATCH (n) RETURN id(n) as id, labels(n) as labels, properties(n) as props",
  "/tmp/nodes.csv", {quotes: "always"}
)

-- 导出关系
CALL apoc.export.csv.query(
  "MATCH (a)-[r]->(b) RETURN id(a) as from, type(r) as type, properties(r) as props, id(b) as to",
  "/tmp/rels.csv", {quotes: "always"}
)
```

### 选项 B：Python 直接导出

```python
from neo4j import GraphDatabase

driver = GraphDatabase.driver("bolt://localhost:7687", auth=("neo4j", "password"))

def export_nodes(session):
    return session.run("MATCH (n) RETURN id(n) as id, labels(n) as labels, properties(n) as props").data()

def export_rels(session):
    return session.run(
        "MATCH (a)-[r]->(b) RETURN id(a) as from, type(r) as rel, properties(r) as props, id(b) as to"
    ).data()

with driver.session() as s:
    nodes = export_nodes(s)
    rels = export_rels(s)
```

---

## 第 2 步：转换为 RDF（Turtle）

将 Neo4j 属性图概念映射到 RDF：

| Neo4j | RDF |
|---|---|
| 带标签 `Person` 的节点 | `?node rdf:type ex:Person` |
| 节点属性 `name = "Alice"` | `?node ex:name "Alice"` |
| 关系 `KNOWS` | `?from ex:knows ?to` |
| 节点 ID | `ex:node_{id}` IRI |
| 关系属性 | RDF-star 引用三元组 `<<ex:from ex:knows ex:to>> ex:since "2021"` |

```python
from rdflib import Graph, URIRef, Literal, Namespace, RDF
import json

EX = Namespace("https://example.org/")
g = Graph()

# 绑定前缀
g.bind("ex", EX)
g.bind("rdf", RDF)

# 转换节点
for node in nodes:
    node_uri = EX[f"node_{node['id']}"]
    # 类型断言
    for label in node["labels"]:
        g.add((node_uri, RDF.type, EX[label]))
    # 属性
    props = json.loads(node["props"]) if isinstance(node["props"], str) else node["props"]
    for key, val in props.items():
        g.add((node_uri, EX[key], Literal(val)))

# 转换关系
for rel in rels:
    from_uri = EX[f"node_{rel['from']}"]
    to_uri = EX[f"node_{rel['to']}"]
    pred_uri = EX[rel["rel"].lower()]
    g.add((from_uri, pred_uri, to_uri))

# 导出为 Turtle
turtle_data = g.serialize(format="turtle")
with open("/tmp/neo4j_export.ttl", "w") as f:
    f.write(turtle_data)
print(f"Exported {len(g)} triples to neo4j_export.ttl")
```

---

## 第 3 步：加载到 pg_ripple

```sql
-- 为迁移的数据创建一个命名图
SELECT pg_ripple.insert_triple(
  'https://example.org/neo4j_import',
  'rdf:type',
  'pg_ripple:Graph'
);

-- 加载 Turtle 文件
SELECT pg_ripple.load_turtle(
  pg_read_file('/tmp/neo4j_export.ttl'),
  'https://example.org/neo4j_import'
);

-- 验证加载
SELECT COUNT(*) FROM pg_ripple.sparql(
  'SELECT (COUNT(*) AS ?count) WHERE { GRAPH <https://example.org/neo4j_import> { ?s ?p ?o } }'
);
```

对于大型导出（数百万个三元组），请改用批量加载器：

```sql
-- 通过 COPY 流式传输以获得最大吞吐量
SELECT pg_ripple.bulk_load_turtle_file(
  '/tmp/neo4j_export.ttl',
  'https://example.org/neo4j_import'
);
```

---

## 第 4 步：为 Cypher 风格查询定义 SPARQL 视图

将常见的 Neo4j 查询模式映射到 SPARQL：

### Neo4j：最短路径

```cypher
MATCH p = shortestPath((a:Person {name: "Alice"})-[:KNOWS*]-(b:Person {name: "Bob"}))
RETURN p
```

```sparql
-- pg_ripple 等效（属性路径）
SELECT ?path WHERE {
  <https://example.org/Alice> ex:knows+ <https://example.org/Bob> .
}
```

### Neo4j：模式匹配

```cypher
MATCH (p:Person)-[:WORKS_AT]->(c:Company)<-[:WORKS_AT]-(q:Person)
WHERE p.name = "Alice"
RETURN q.name
```

```sparql
SELECT ?coworkerName WHERE {
  <https://example.org/Alice> ex:worksAt ?company .
  ?company rdf:type ex:Company .
  ?coworker ex:worksAt ?company .
  ?coworker ex:name ?coworkerName .
  FILTER(?coworker != <https://example.org/Alice>)
}
```

---

## 第 5 步：添加 SHACL 约束

将 Neo4j 唯一性和存在性约束重新创建为 SHACL 形状：

```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://example.org/> .

ex:PersonShape a sh:NodeShape ;
  sh:targetClass ex:Person ;
  sh:property [
    sh:path ex:email ;
    sh:maxCount 1 ;           -- Neo4j 唯一性约束等效
    sh:datatype xsd:string
  ] ;
  sh:property [
    sh:path ex:name ;
    sh:minCount 1 ;           -- NOT NULL 等效
    sh:datatype xsd:string
  ] .
```

```sql
SELECT pg_ripple.load_shacl($$
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix ex: <https://example.org/> .
  -- 在此处粘贴形状
$$);
```

---

## 第 6 步：验证和对账

```sql
-- 检查约束违规
SELECT pg_ripple.validate();

-- 查找没有必需属性的节点（替换 Neo4j NULL 检查）
SELECT binding->>'?node' AS node
FROM pg_ripple.sparql($$
  SELECT ?node WHERE {
    ?node rdf:type ex:Person .
    FILTER NOT EXISTS { ?node ex:name ?n }
  }
$$);

-- 验证计数与 Neo4j 匹配
SELECT binding->>'?count' AS triple_count
FROM pg_ripple.sparql('SELECT (COUNT(*) AS ?count) WHERE { ?s ?p ?o }');
```

---

## 映射参考

| Neo4j 概念 | RDF / pg_ripple |
|---|---|
| 节点标签 | `rdf:type` 断言 |
| 节点属性 | 谓词-对象对 |
| 关系类型 | 谓词 IRI |
| 关系属性 | RDF-star 引用三元组或具名化 |
| 节点 ID | IRI（推荐 UUID） |
| 唯一性约束 | `sh:maxCount 1` 形状 |
| 存在性约束 | `sh:minCount 1` 形状 |
| 属性上的索引 | VP 表自动索引 |
| Cypher `MATCH` | SPARQL `SELECT … WHERE { }` |
| Cypher `CREATE` | `pg_ripple.insert_triple()` |
| Cypher `MERGE` | 通过批量加载的 `INSERT … ON CONFLICT DO NOTHING` |
| Cypher `shortestPath` | `ex:pred+` 属性路径 |
| Cypher `CALL db.schema()` | `pg_ripple.list_predicates()` |

---

## 性能提示

1. **批量加载** 使用 `bulk_load_turtle_file()` 以避免单个插入的每次调用开销。
2. **设置 `pg_ripple.vp_promotion_threshold = 100`** 在迁移期间为更多谓词创建单独的 VP 表，然后在之后恢复默认值。
3. **运行 `VACUUM ANALYZE`** 在 `_pg_ripple.dictionary` 上进行大型加载后。
4. **使用 `pg_ripple.run_merge()`** 在重型分析查询之前将增量刷新到主分区。