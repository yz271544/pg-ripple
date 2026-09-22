# 命名图

pg_ripple 支持 RDF 的命名图概念：每个三元组都属于**默认图**（内部 ID `0`）或由 IRI 标识的命名图。

命名图作为每个 VP 表行的一部分存储（`g BIGINT` 列）。不需要单独的图目录表 — 命名图的集合是从数据派生的。

## create_graph

```sql
pg_ripple.create_graph(graph_iri TEXT) RETURNS BIGINT
```

在字典中注册一个命名图并返回其 `BIGINT` ID。这只是信息性的——图是在插入三元组时隐式创建的。当您希望在加载数据之前获取图 ID 时，显式调用 `create_graph` 很有用。

```sql
SELECT pg_ripple.create_graph('<https://example.org/graph1>');
-- 返回该图 IRI 的字典 ID
```

## drop_graph

```sql
pg_ripple.drop_graph(graph_iri TEXT) RETURNS BIGINT
```

删除属于命名图的所有三元组。返回已删除的三元组数量。

```sql
SELECT pg_ripple.drop_graph('<https://example.org/graph1>');
```

> **警告**：这将永久删除图中的所有三元组。该操作是事务性的——如果需要试运行，请将其包装在 `BEGIN`/`ROLLBACK` 中。

## list_graphs

```sql
pg_ripple.list_graphs() RETURNS TABLE(graph_iri TEXT, triple_count BIGINT)
```

返回所有命名图以及每个图包含的三元组数量。排除默认图（ID 0）。

```sql
SELECT * FROM pg_ripple.list_graphs();
```

## 在 SPARQL 中查询命名图

使用 `GRAPH` 关键字将模式限制为特定图：

```sql
SELECT * FROM pg_ripple.sparql('
  SELECT ?s ?p ?o WHERE {
    GRAPH <https://example.org/graph1> { ?s ?p ?o }
  }
');
```

跨所有命名图查询：

```sql
SELECT * FROM pg_ripple.sparql('
  SELECT ?g ?s ?p ?o WHERE {
    GRAPH ?g { ?s ?p ?o }
  }
');
```

## 插入到命名图

```sql
-- 通过 insert_triple
SELECT pg_ripple.insert_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>',
    '<https://example.org/graph1>'  -- 命名图
);

-- 通过 bulk load（N-Quads — 第四列是图）
SELECT pg_ripple.load_nquads('
<https://example.org/alice> <https://example.org/knows> <https://example.org/bob> <https://example.org/graph1> .
');

-- 通过 TriG
SELECT pg_ripple.load_trig('
GRAPH <https://example.org/graph1> {
  <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> .
}
');
```

## 默认图

默认图的内部 ID 为 `0`。没有图参数插入的三元组将进入默认图。没有 `GRAPH` 子句的 SPARQL 模式与默认图匹配。

---

## 图感知操作（v0.15.0）

### find_triples_in_graph

```sql
pg_ripple.find_triples_in_graph(s TEXT, p TEXT, o TEXT, graph TEXT) → SETOF RECORD
```

在特定命名图内模式匹配三元组。与 `find_triples()` 具有相同的通配符规则 — 对任何位置传递 `NULL`。

```sql
SELECT * FROM pg_ripple.find_triples_in_graph(
    NULL, NULL, NULL,
    '<https://example.org/graph1>'
);
```

### triple_count_in_graph

```sql
pg_ripple.triple_count_in_graph(graph_iri TEXT) → BIGINT
```

返回特定命名图中的三元组数量。

```sql
SELECT pg_ripple.triple_count_in_graph('<https://example.org/graph1>');
-- 42
```

### delete_triple_from_graph

```sql
pg_ripple.delete_triple_from_graph(s TEXT, p TEXT, o TEXT, graph_iri TEXT) → BIGINT
```

从特定命名图中删除单个三元组。返回已删除的行数。

```sql
SELECT pg_ripple.delete_triple_from_graph(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>',
    '<https://example.org/graph1>'
);
```

### clear_graph

```sql
pg_ripple.clear_graph(graph_iri TEXT) → BIGINT
```

从命名图中删除所有三元组而不取消注册它。返回已删除的三元组数量。与 `drop_graph()` 不同的是，图的 IRI 保留在字典中——当您计划将数据重新加载到同一个图时很有用。

```sql
SELECT pg_ripple.clear_graph('<https://example.org/graph1>');
```

> **提示**：对"截断并重新加载"工作流使用 `clear_graph()`。当您希望永久删除图时使用 `drop_graph()`。