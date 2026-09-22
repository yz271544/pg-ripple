# 序列化与导出

pg_ripple（v0.9.0）支持将 RDF 数据导出到 Turtle、JSON-LD、N-Triples 和 N-Quads 格式，并从 RDF/XML 导入。SPARQL CONSTRUCT 和 DESCRIBE 查询也可以直接以 Turtle 或 JSON-LD 形式返回结果。

---

## 导入

### load_rdfxml

```sql
pg_ripple.load_rdfxml(data TEXT) RETURNS BIGINT
```

从字符串解析 [RDF/XML](https://www.w3.org/TR/rdf-syntax-grammar/) 数据并将所有三元组存储在默认图中。返回加载的数据组数。

RDF/XML 是原始的 W3C 标准 RDF 序列化格式，由许多本体编辑器（例如 [Protégé](https://protege.stanford.edu/)）生成。

```sql
SELECT pg_ripple.load_rdfxml('<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="https://example.org/">
  <rdf:Description rdf:about="https://example.org/alice">
    <ex:name>Alice</ex:name>
    <ex:knows rdf:resource="https://example.org/bob"/>
  </rdf:Description>
</rdf:RDF>');
-- 返回 2
```

**注意：** RDF/XML 不支持命名图；所有三元组都加载到默认图中。

---

## 导出

### export_turtle

```sql
pg_ripple.export_turtle(graph TEXT DEFAULT NULL) RETURNS TEXT
```

将三元组导出为 [Turtle](https://www.w3.org/TR/turtle/) 文档。三元组按主语分组，并以紧凑的 Turtle 块发出。前缀注册表中的所有前缀声明都作为 `@prefix` 行包含在内。

RDF-star 引用三元组以 Turtle-star `<< s p o >>` 表示法序列化。

```sql
-- 导出默认图
SELECT pg_ripple.export_turtle();

-- 导出命名图
SELECT pg_ripple.export_turtle('https://example.org/my-graph');
```

**输出示例：**

```turtle
@prefix ex: <https://example.org/> .

<https://example.org/alice>
    <https://example.org/knows> <https://example.org/bob> ;
    <https://example.org/name> "Alice" .
```

### export_jsonld

```sql
pg_ripple.export_jsonld(graph TEXT DEFAULT NULL) RETURNS JSONB
```

将三元组导出为 [JSON-LD](https://www.w3.org/TR/json-ld11/) 展开形式文档。每个主语成为一个数组条目，包含其所有谓词和对象。

```sql
SELECT pg_ripple.export_jsonld();
-- 返回：[{"@id": "https://example.org/alice", "https://example.org/name": [{"@value": "Alice"}], ...}]
```

JSON-LD 非常适合在 REST API 和关联数据平台（LDP）上下文中使用。

### export_ntriples

```sql
pg_ripple.export_ntriples(graph TEXT DEFAULT NULL) RETURNS TEXT
```

将三元组导出为 [N-Triples](https://www.w3.org/TR/n-triples/) 文本（每行一个三元组）。

### export_nquads

```sql
pg_ripple.export_nquads(graph TEXT DEFAULT NULL) RETURNS TEXT
```

将四元组导出为 [N-Quads](https://www.w3.org/TR/n-quads/) 文本。传递 `NULL` 以导出所有图。

---

## 流式导出

对于大型图，请使用返回 `SETOF TEXT` 的流式变体 — 每行一条记录。这样可以避免在内存中构建完整文档。

### export_turtle_stream

```sql
pg_ripple.export_turtle_stream(graph TEXT DEFAULT NULL) RETURNS SETOF TEXT
```

首先产出 `@prefix` 声明，然后每行一个扁平 Turtle 三元组。

```sql
COPY (SELECT line FROM pg_ripple.export_turtle_stream()) TO '/tmp/output.ttl';
```

### export_jsonld_stream

```sql
pg_ripple.export_jsonld_stream(graph TEXT DEFAULT NULL) RETURNS SETOF TEXT
```

每个主语产生一行 NDJSON。每行都是一个完整的 JSON 对象。

```sql
COPY (SELECT line FROM pg_ripple.export_jsonld_stream()) TO '/tmp/output.ndjson';
```

---

## SPARQL CONSTRUCT 和 DESCRIBE 输出格式

默认情况下，`sparql_construct()` 和 `sparql_describe()` 返回 JSONB 行。v0.9.0 格式特定的变体将相同的三元组直接作为 Turtle 或 JSON-LD 返回。

### sparql_construct_turtle

```sql
pg_ripple.sparql_construct_turtle(query TEXT) RETURNS TEXT
```

执行 SPARQL CONSTRUCT 查询并以 Turtle 文档形式返回结果。RDF-star 引用三元组使用 Turtle-star `<< s p o >>` 表示法。

```sql
SELECT pg_ripple.sparql_construct_turtle('
  CONSTRUCT { ?s <https://schema.org/knows> ?o }
  WHERE     { ?s <https://schema.org/knows> ?o }
');
```

### sparql_construct_jsonld

```sql
pg_ripple.sparql_construct_jsonld(query TEXT) RETURNS JSONB
```

执行 SPARQL CONSTRUCT 查询并以 JSON-LD 展开形式数组返回结果。

```sql
SELECT pg_ripple.sparql_construct_jsonld('
  CONSTRUCT { ?s ?p ?o }
  WHERE     { ?s ?p ?o }
  LIMIT 100
');
```

### sparql_describe_turtle

```sql
pg_ripple.sparql_describe_turtle(query TEXT, strategy TEXT DEFAULT 'cbd') RETURNS TEXT
```

执行 SPARQL DESCRIBE 查询并将描述作为 Turtle 文本返回。`strategy` 可以是 `'cbd'`（Concise Bounded Description，默认）、`'scbd'`（Symmetric CBD）或 `'simple'`。

### sparql_describe_jsonld

```sql
pg_ripple.sparql_describe_jsonld(query TEXT, strategy TEXT DEFAULT 'cbd') RETURNS JSONB
```

执行 SPARQL DESCRIBE 查询并将描述作为 JSON-LD 返回。

---

## RDF-star 序列化

所有导出函数都透明地处理 RDF-star 引用三元组：

- **N-Triples / N-Quads**：使用 `<< s p o >>` 表示法（N-Triples-star / N-Quads-star）
- **Turtle**：使用 `<< s p o >>` 表示法（Turtle-star）
- **JSON-LD**：引用三元组表示为 `{"@value": "<< s p o >>", "@type": "rdf:Statement"}`

---

## 格式指南

| 格式 | 导入 | 导出 | 命名图 | RDF-star |
|--------|--------|--------|-------------|---------|
| N-Triples | `load_ntriples` | `export_ntriples` | 否 | 是（N-Triples-star）|
| N-Quads | `load_nquads` | `export_nquads` | 是 | 否 |
| Turtle | `load_turtle` | `export_turtle` | 否 | 是（Turtle-star）|
| TriG | `load_trig` | — | 是 | 否 |
| RDF/XML | `load_rdfxml` | — | 否 | 否 |
| JSON-LD | — | `export_jsonld` | 否 | 部分 |

**提示：** 对于 Protégé 本体，使用 RDF/XML；对于 REST API，使用 JSON-LD；对于人类可读文件，使用 Turtle。

---

## JSON-LD 框架（v0.17.0）

JSON-LD 框架允许您将 RDF 图数据重塑为适合 REST API 或应用程序的特定树结构。您提供一个*框架*（一个 JSON 模板），pg_ripple 返回一个干净嵌套的 JSON-LD 文档。

### `export_jsonld_framed`

```sql
pg_ripple.export_jsonld_framed(
    frame     JSONB,
    graph     TEXT    DEFAULT NULL,   -- NULL = 合并图；IRI = 命名图
    embed     TEXT    DEFAULT '@once', -- @once | @always | @never
    explicit  BOOLEAN DEFAULT FALSE,  -- 省略不在框架中的属性
    ordered   BOOLEAN DEFAULT FALSE   -- 按字典顺序对输出键排序
) RETURNS JSONB
```

将框架翻译为 SPARQL CONSTRUCT 查询，执行它，使用 W3C 嵌入算法，使用框架的 `@context` 压缩 IRI，并返回框架化的 JSON-LD 文档。

```sql
-- 选择所有 Person 节点及其名称。
SELECT pg_ripple.export_jsonld_framed('{
    "@context": {"schema": "https://schema.org/"},
    "@type": "https://schema.org/Person",
    "https://schema.org/name": {}
}'::jsonb);
```

### `jsonld_frame_to_sparql`

```sql
pg_ripple.jsonld_frame_to_sparql(
    frame   JSONB,
    graph   TEXT DEFAULT NULL
) RETURNS TEXT
```

将框架转换为其 SPARQL CONSTRUCT 查询字符串而不执行它。使用此函数检查或调试生成的查询。

```sql
SELECT pg_ripple.jsonld_frame_to_sparql(
    '{"@type": "https://schema.org/Person", "https://schema.org/name": {}}'::jsonb
);
```

### `jsonld_frame`

```sql
pg_ripple.jsonld_frame(
    input     JSONB,
    frame     JSONB,
    embed     TEXT    DEFAULT '@once',
    explicit  BOOLEAN DEFAULT FALSE,
    ordered   BOOLEAN DEFAULT FALSE
) RETURNS JSONB
```

通用框架原语：将 W3C 嵌入算法应用于任何已展开的 JSON-LD JSONB 值，不一定来自 pg_ripple 存储。用于框架化通过其他方式获得的 SPARQL CONSTRUCT 结果。

### `export_jsonld_framed_stream`

```sql
pg_ripple.export_jsonld_framed_stream(
    frame   JSONB,
    graph   TEXT DEFAULT NULL
) RETURNS SETOF TEXT
```

流式变体：每个匹配的根节点返回一行 NDJSON，避免在内存中缓冲大型文档。

```sql
-- 每个匹配的 Company 流式输出一行。
SELECT line FROM pg_ripple.export_jsonld_framed_stream(
    '{"@type": "https://example.org/Company"}'::jsonb
);
```

### 框架语法入门

框架是一个 JSON 对象，其结构反映了所需的输出形状：

| 框架构造 | 含义 |
|---|---|
| `"@type": "ex:Foo"` | 选择 RDF 类型为 `ex:Foo` 的节点 |
| `"ex:prop": {}` | 包含 `ex:prop`；匹配任何值（通配符）|
| `"ex:prop": []` | 选择**缺少** `ex:prop` 的节点 |
| `"ex:prop": { ... nested frame ... }` | 递归地嵌入所引用的节点 |
| `"@reverse": { "ex:memberOf": {} }` | 收集其 `ex:memberOf` 指向此处的所有主语 |
| `"@id": "http://ex.org/Alice"` | 限制为特定主语 IRI |
| `"@requireAll": true` | 所有列出的属性都是必需的（无 OPTIONAL JOIN）|

### `@embed` / `@explicit` / `@omitDefault` / `@requireAll` 标志

| 标志 | 默认值 | 描述 |
|---|---|---|
| `@embed` | `@once` | `@once` — 嵌入每个节点一次，对重复使用 `{"@id":"..."}` 引用；`@always` — 始终嵌入；`@never` — 始终使用引用 |
| `@explicit` | `false` | 当为 `true` 时，从输出节点中省略框架中未列出的属性 |
| `@omitDefault` | `false` | 当为 `true` 时，省略不存在的属性而不是替换为 `@default` |
| `@requireAll` | `false` | 当为 `true` 时，将 OPTIONAL JOIN 转换为 INNER JOIN；只有具有所有列出属性的节点才匹配 |

### 命名图作用域

传递 `graph` 以将 CONSTRUCT 限制为特定命名图：

```sql
SELECT pg_ripple.export_jsonld_framed(
    '{"@type": "https://schema.org/Person"}'::jsonb,
    'https://example.org/graph1'
);
```

### 支持的框架特性

| 特性 | 支持 |
|---|---|
| `@type` 匹配（单个或数组）| ✓ |
| `@id` 匹配（单个或数组）| ✓ |
| 属性通配符 `{}` | ✓ |
| 不存在属性模式 `[]` | ✓ |
| `@reverse` 属性 | ✓ |
| `@embed`：`@once` / `@always` / `@never` | ✓ |
| `@explicit` 包含标志 | ✓ |
| `@omitDefault` 标志 | ✓ |
| `@default` 值 | ✓ |
| `@requireAll` 标志 | ✓ |
| `@context` 压缩 | ✓ |
| 命名图 `@graph` 作用域 | ✓ |
| 值模式匹配（值对象中的 `@value` / `@language` / `@type`）| ✗ （推迟）|