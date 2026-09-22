# 数据建模

## 何时使用 RDF-star 与引用化

**引用化**（传统 RDF）将三元组表示为具有四个属性的资源（`rdf:subject`、`rdf:predicate`、`rdf:object`、`rdf:type rdf:Statement`）。它需要每个带注释的陈述额外 4 个三元组，并产生冗长的查询模式。

**RDF-star** 直接使用带引号的三元组 `<< s p o >>` 作为主语或宾语：

```
# RDF-star（紧凑）
<< <ex:alice> <ex:knows> <ex:bob> >> <ex:since> "2023-01-01"^^xsd:date .

# 引用化（冗长 — 多出 4 个三元组）
<ex:stmt1> rdf:type      rdf:Statement ;
           rdf:subject   <ex:alice> ;
           rdf:predicate <ex:knows> ;
           rdf:object    <ex:bob> ;
           <ex:since>    "2023-01-01"^^xsd:date .
```

对边注释（来源、置信度、时间范围）使用 RDF-star。仅出于与不支持 RDF-star 的存储库的旧版兼容性而使用引用化。

## 用于分区的命名图

命名图是一种轻量级方式，可以按来源、时间或主题对数据进行分区，而无需更改三元组结构：

```sql
-- 将来源数据加载到单独的图中
SELECT pg_ripple.load_nquads('
<ex:alice> <ex:knows> <ex:bob> <ex:source1> .
<ex:alice> <ex:knows> <ex:bob> <ex:source2> .
');

-- 在特定图中查询
SELECT * FROM pg_ripple.sparql('
  SELECT ?s ?p ?o WHERE {
    GRAPH <ex:source1> { ?s ?p ?o }
  }
');
```

## 空白节点

空白节点对于匿名中间资源很有用——不需要全局唯一 IRI 的节点。常见用途：

- **列表编码**：每个列表元素都是一个带有 `rdf:first` 和 `rdf:rest` 谓词的空白节点
- **结构化值**：具有多个方面的测量（值、单位、不确定性）
- **中间连接节点**：无需引用化的 n 元关系

**陷阱**：空白节点具有加载作用域身份。两次单独的 `load_ntriples()` 调用中的 `_:b0` 会获得两个不同的字典 ID。如果需要稳定的跨加载空白节点，请使用基于 IRI 的标识符。

## 主语位置与宾语位置的带引号三元组

pg_ripple 同时支持：

```
# 宾语位置：将陈述注释为宾语
<ex:carol> <ex:asserted> << <ex:alice> <ex:knows> <ex:bob> >> .

# 主语位置：陈述具有属性
<< <ex:alice> <ex:knows> <ex:bob> >> <ex:since> "2023"^^xsd:gYear .
```

两者都通过字典中的"encode_triple()`存储，可以通过"decode_triple()`或"get_statement()`检索。

## 通过 RDF-star 实现 LPG 风格的边属性

RDF-star 可以干净地映射到 LPG 边属性：

| LPG 概念 | RDF-star 编码 |
|---|---|
| 节点 `alice` | `<ex:alice>` |
| 边 `alice --[KNOWS]--> bob` | `<ex:alice> <ex:knows> <ex:bob>` |
| 边属性 `since = 2023` | `<< <ex:alice> <ex:knows> <ex:bob> >> <ex:since> "2023"^^xsd:gYear` |
| 节点属性 `name = "Alice"` | `<ex:alice> <ex:name> "Alice"` |

这使得 pg_ripple 一旦添加了 Cypher/GQL 查询层（v0.13.0）便自然成为 LPG 数据的后端。

## 互操作格式指南（v0.9.0）

为您正在集成的工具或上下文选择合适的序列化格式：

| 工具 / 上下文 | 推荐格式 | pg_ripple 函数 |
|---|---|---|
| PROTégé / OWL 本体 | RDF/XML | `load_rdfxml()` |
| 关联数据平台（LDP）REST API | JSON-LD | `export_jsonld()` / `sparql_construct_jsonld()` |
| 命令行管道、流式处理 | N-Triples 或 N-Quads | `export_ntriples()` / `export_nquads()` |
| 人类可读文件、Git 存储 | Turtle | `export_turtle()` / `sparql_construct_turtle()` |
| 大型图导出（节省内存） | 流式 Turtle | `export_turtle_stream()` |
| 用于 API 的 SPARQL 查询结果 | JSON-LD CONSTRUCT | `sparql_construct_jsonld()` |

### Protégé → RDF/XML

Protégé 默认以 OWL/RDF/XML 格式保存本体。直接加载它们：

```sql
-- 将文件读入 PostgreSQL（仅限超级用户）
SELECT pg_ripple.load_rdfxml(pg_read_file('/data/ontology.owl'));
```

### 关联数据平台 → JSON-LD

基于 LDP 构建的 REST API 通常提供 JSON-LD。使用 `export_jsonld()` 获取当前状态：

```sql
SELECT pg_ripple.export_jsonld('https://myapp.example.org/graph/users');
```

对于 SPARQL 驱动的响应：

```sql
SELECT pg_ripple.sparql_construct_jsonld('
  CONSTRUCT { ?s ?p ?o }
  WHERE     { ?s a <https://schema.org/Person> ; ?p ?o }
');
```

### CLI / shell 管道 → N-Triples 或 N-Quads

使用 `rapper`、`riot`、`rdfpipe` 或命令行的 `awk`/`grep` 进行处理：

```bash
psql -c "COPY (SELECT pg_ripple.export_ntriples()) TO STDOUT" > snapshot.nt
```

对于多图导出：

```bash
psql -c "COPY (SELECT pg_ripple.export_nquads(NULL)) TO STDOUT" > snapshot.nq
```

---

## 用于 REST API 的 JSON-LD 框架（v0.17.0）

### Frame-First API 设计

构建以 pg_ripple 为后端的 REST API 时，请先于编写应用程序代码设计 JSON-LD 框架。框架定义了 API 响应的确切形状，`export_jsonld_framed()` 自动生成优化的 SPARQL CONSTRUCT 查询。

```sql
-- "公司及其员工"API 端点的框架。
SELECT pg_ripple.export_jsonld_framed('{
    "@context": {"schema": "https://schema.org/"},
    "@type": "https://schema.org/Organization",
    "https://schema.org/name": {},
    "https://schema.org/employee": {
        "https://schema.org/name": {},
        "https://schema.org/email": {}
    }
}'::jsonb);
```

仅扫描 3 个 VP 表（`rdf:type`、`schema:name`、`schema:employee`）——而不是大型图中的全部 10,000 个表。

### 使用 `jsonld_frame_to_sparql` 进行检查

在部署框架驱动的 API 端点之前，请使用 `jsonld_frame_to_sparql()` 查看生成的 SPARQL 查询，以验证框架是否符合预期。

```sql
-- 检查框架生成的 SPARQL 查询。
SELECT pg_ripple.jsonld_frame_to_sparql('{
    "@type": "https://schema.org/Organization",
    "https://schema.org/name": {},
    "https://schema.org/employee": {
        "https://schema.org/name": {}
    }
}'::jsonb);
```

这对于以下情况尤其有用：
- **意外广泛的扫描**：如果生成的 SPARQL 似乎在扫描太多 VP 表，请考虑缩小框架或使用更具体的选择性查询。
- **命名图作用域**：传递 `graph` IRI 以将 CONSTRUCT 限制为单个命名图，进一步降低扫描开销。
- **重复调用**：对相同框架的重复调用受益于 SPARQL 计划缓存。

### 框架视图与物化视图

对于高吞吐量场景，使用 `create_framing_view()` 而不是每次都调用 `export_jsonld_framed()`：

| 使用场景 | 推荐方法 |
|---|---|
| 一次性 API 请求 | `export_jsonld_framed()` |
| 实时仪表板（高读取吞吐量） | 使用 `DIFFERENTIAL` 调度的 `create_framing_view()` |
| 约束监控（违规检测） | 使用 `IMMEDIATE` 刷新的 `create_framing_view()` |
| 数据仓库的大型导出 | 使用 `FULL` + 长调度的 `create_framing_view()` |