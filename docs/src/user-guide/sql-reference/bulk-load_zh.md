# 批量加载

pg_ripple 提供八个批量加载函数，涵盖四种 RDF 序列化格式的内联字符串和文件变体。

## load_ntriples

```sql
pg_ripple.load_ntriples(data TEXT) RETURNS BIGINT
```

从字符串解析并加载 N-Triples 数据。返回加载的三元组数。

```sql
SELECT pg_ripple.load_ntriples('
<https://example.org/alice> <https://example.org/knows> <https://example.org/bob> .
<https://example.org/bob>   <https://example.org/name>  "Bob" .
');
-- 返回 2
```

## load_ntriples_file

```sql
pg_ripple.load_ntriples_file(path TEXT) RETURNS BIGINT
```

从服务器端文件加载 N-Triples 数据。路径必须可被 PostgreSQL 服务器进程读取。

```sql
SELECT pg_ripple.load_ntriples_file('/data/dataset.nt');
```

## load_turtle / load_turtle_file

```sql
pg_ripple.load_turtle(data TEXT) RETURNS BIGINT
pg_ripple.load_turtle_file(path TEXT) RETURNS BIGINT
```

解析并加载 [Turtle](https://www.w3.org/TR/turtle/) 数据。Turtle 支持前缀声明（`@prefix`）和相关三元组的简洁语法。

```sql
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .

ex:alice ex:knows ex:bob ;
         ex:name  "Alice" .
');
```

## load_nquads / load_nquads_file

```sql
pg_ripple.load_nquads(data TEXT) RETURNS BIGINT
pg_ripple.load_nquads_file(path TEXT) RETURNS BIGINT
```

加载 [N-Quads](https://www.w3.org/TR/n-quads/) 数据。N-Quads 使用可选的第四个组件（命名图）扩展 N-Triples。没有图组件的三元组进入默认图。

```sql
SELECT pg_ripple.load_nquads('
<https://example.org/alice> <https://example.org/knows> <https://example.org/bob> <https://example.org/graph1> .
<https://example.org/bob>   <https://example.org/name>  "Bob" .
');
```

## load_trig / load_trig_file

```sql
pg_ripple.load_trig(data TEXT) RETURNS BIGINT
pg_ripple.load_trig_file(path TEXT) RETURNS BIGINT
```

加载 [TriG](https://www.w3.org/TR/trig/) 数据。TriG 是 Turtle 的图感知扩展 — 三元组可以在命名 `GRAPH { }` 块内分组。

```sql
SELECT pg_ripple.load_trig('
@prefix ex: <https://example.org/> .

GRAPH ex:graph1 {
    ex:alice ex:knows ex:bob .
}

ex:alice ex:name "Alice" .
');
```

## 图感知加载器（v0.15.0）

从 v0.15.0 开始，每种格式都有一个图感知变体，可将三元组直接加载到命名图中。还提供文件变体（仅限超级用户）。

### 内联加载器

```sql
pg_ripple.load_ntriples_into_graph(data TEXT, graph_iri TEXT) RETURNS BIGINT
pg_ripple.load_turtle_into_graph(data TEXT, graph_iri TEXT) RETURNS BIGINT
pg_ripple.load_rdfxml_into_graph(data TEXT, graph_iri TEXT) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_turtle_into_graph('
@prefix ex: <https://example.org/> .
ex:alice ex:knows ex:bob .
', '<https://example.org/graph1>');
-- 返回：1
```

### 文件加载器

```sql
pg_ripple.load_ntriples_file_into_graph(path TEXT, graph_iri TEXT) RETURNS BIGINT
pg_ripple.load_turtle_file_into_graph(path TEXT, graph_iri TEXT) RETURNS BIGINT
pg_ripple.load_rdfxml_file_into_graph(path TEXT, graph_iri TEXT) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_ntriples_file_into_graph(
    '/data/people.nt',
    '<https://example.org/people>'
);
```

### RDF/XML 加载器（v0.9.0）

```sql
pg_ripple.load_rdfxml(data TEXT) RETURNS BIGINT
pg_ripple.load_rdfxml_file(path TEXT) RETURNS BIGINT
```

解析符合规范的 RDF/XML — 由 Protégé、OWL 编辑器和许多本体工具生成的格式。

```sql
SELECT pg_ripple.load_rdfxml('
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="https://example.org/">
  <rdf:Description rdf:about="https://example.org/alice">
    <ex:name>Alice</ex:name>
  </rdf:Description>
</rdf:RDF>
');
```

---

## 空白节点作用域

每次调用批量加载函数都是一个独立的**空白节点作用域**。一次 `load_ntriples()` 调用中的 `_:b0` 与后续调用中的 `_:b0` 具有不同的字典 ID。这符合 SPARQL/RDF 空白节点作用域规则，并防止在文档加载之间意外合并空白节点。

## 批量加载后的 VP 提升

加载大型数据集后，某些谓词可能超过 `pg_ripple.vp_promotion_threshold` 并自动从 `vp_rare` 提升到专用 VP 表。此提升是自动触发的。

在任何大型加载之后，运行 `ANALYZE` 以帮助 PostgreSQL 规划器选择高效的连接策略：

```sql
ANALYZE _pg_ripple.vp_rare;
```

或使用服务器端函数（v0.6.0+ 中可用）一次性分析所有 VP 表：

```sql
SELECT pg_ripple.vacuum_analyze();
```

---

## 严格模式（v0.25.0）

所有批量加载函数都接受一个可选的 `strict BOOLEAN DEFAULT false` 参数。

| 模式 | 对格式错误的三元组的处理 |
|---|---|
| `strict = false`（默认） | 格式错误的三元组发出 `WARNING` 并被跳过；其余三元组已提交 |
| `strict = true` | 任何解析错误都会中止整个加载并显示 `ERROR`；事务回滚 |

### Examples

```sql
-- 宽松模式（默认）：跳过错误的三元组，其他三元组已提交
SELECT pg_ripple.load_ntriples(
    '<https://example.org/s> <https://example.org/p> <https://example.org/o> .' || chr(10) ||
    'this is malformed' || chr(10)
);
-- WARNING: skipping malformed triple on line 2
-- 结果：1

-- 严格模式：错误的三元组导致整个加载回滚
SELECT pg_ripple.load_ntriples(
    '<https://example.org/s> <https://example.org/p> <https://example.org/o> .' || chr(10) ||
    'this is malformed' || chr(10),
    strict => true
);
-- ERROR: malformed triple on line 2: [...]
-- 来自此调用的所有三元组都已回滚
```

### 何时使用严格模式

- **数据管道**：使用 `strict = true` 尽早检测上游数据质量问题。
- **明确交互式探索**：使用 `strict = false`（默认）加载部分数据并进行调查。
- **迁移**：使用 `strict = true` 以确保所有数据已加载，然后使用 `strict = false` 进行恢复。

---

## 文件路径安全性（v0.25.0）

`load_*_file()` 变体（`load_turtle_file`、`load_ntriples_file` 等）使用 `realpath()` 解析符号链接，并在读取之前验证规范路径是否位于 PostgreSQL 数据目录中。这可以防止基于符号链接的路径遍历攻击：

```sql
-- 被拒绝：超出数据目录
SELECT pg_ripple.load_turtle_file('/etc/passwd');
-- ERROR: permission denied: "/etc/passwd" is outside the database cluster directory
```

只能加载 `current_setting('data_directory')` 内的文件。