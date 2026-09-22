# 常见问题解答

## 概述

### 为什么使用 VP 表而不是一个大三元组表？

具有 1 亿个三元组的单个 `(s, p, o, g)` 表需要一个 B-tree 索引，该索引在任何有用的谓词特定查询中涉及所有四列。每个查询都必须扫描所有谓词的行，无论过滤器如何。

垂直分区（每个谓词一个表）意味着对 `<ex:knows>` 三元组的查询只扫描 `vp_{knows_id}` 表 — 通常只是总数据的一小部分。`(s, o)` 和 `(o, s)` 上的两个 B-tree 索引小且对缓存友好。SPARQL 星形模式（相同主语、多个谓词）成为小表之间的简单多路连接。

### 为什么要使用 PostgreSQL 18？

pg_ripple 在 `WITH RECURSIVE` CTE 中使用 `CYCLE` 子句进行属性路径查询中的基于哈希的循环检测。`CYCLE` 子句在 PostgreSQL 14 中引入，但基于哈希的变体（相对于基于数组的）在 PG 17/18 中首次变得高性能。PG 18 也是 pgrx 0.18 稳定支持的第一个版本。

### pg_ripple 是否与 LPG 工具兼容？

尚未兼容。Cypher/GQL 兼容性层在 1.0 后路线图上。VP 存储结构在架构上与 LPG 对齐 — 每个 VP 表都是一种属性边类型 — 因此映射将是自然的。

### pg_ripple 支持哪些 RDF 格式？

**导入（加载）：**
- N-Triples 和 N-Triples-star（`load_ntriples`）
- N-Quads（`load_nquads`）
- Turtle 和 Turtle-star（`load_turtle`）
- TriG（`load_trig`）
- RDF/XML（`load_rdfxml`，v0.9.0）

**导出：**
- N-Triples（`export_ntriples`）
- N-Quads（`export_nquads`）
- Turtle（`export_turtle`，v0.9.0）— 包括 Turtle-star 用于 RDF-star 数据
- JSON-LD 扩展形式（`export_jsonld`，v0.9.0）
- 大型图的流式 Turtle 或 JSON-LD（`export_turtle_stream`、`export_jsonld_stream`，v0.9.0）

SPARQL CONSTRUCT 和 DESCRIBE 结果可以通过 `sparql_construct_turtle`、`sparql_construct_jsonld`、`sparql_describe_turtle` 和 `sparql_describe_jsonld`（v0.9.0）直接序列化为 Turtle 或 JSON-LD。

### 我可以将 pg_ripple 与 JSON-LD 一起用于 REST API 吗？

可以。使用 `export_jsonld()` 或 `sparql_construct_jsonld()` 生成 JSON-LD 响应：

```sql
-- 完整图为 JSON-LD
SELECT pg_ripple.export_jsonld('https://myapp.example.org/graph/users');

-- 由 SPARQL 驱动的选择为 JSON-LD
SELECT pg_ripple.sparql_construct_jsonld('
  CONSTRUCT { ?s ?p ?o }
  WHERE { ?s a <https://schema.org/Person> ; ?p ?o }
');
```

输出是扩展形式的 JSON-LD — 每个主语都是一个数组条目，具有 IRI 键和类型化值数组。

---

## SPARQL

### 支持哪些 SPARQL 1.1 功能？

截至 v0.19.0，实现了完整的 SPARQL 1.1 规范：

**查询形式：** SELECT、ASK、CONSTRUCT、DESCRIBE

**图模式：** BGP、OPTIONAL (LeftJoin)、UNION、MINUS、FILTER、BIND、VALUES、通过 `GRAPH` 的命名图

**属性路径：** `+`、`*`、`?`、`/`（序列）、`|`（替代）、`^`（逆）

**聚合：** GROUP BY、HAVING、COUNT、SUM、AVG、MIN、MAX、GROUP_CONCAT

**修饰符：** DISTINCT、ORDER BY、LIMIT、OFFSET、子查询

**更新：** INSERT DATA、DELETE DATA、DELETE/INSERT WHERE、LOAD、CLEAR、DROP、CREATE、COPY、MOVE、ADD

**联邦：** `SERVICE <url> { … }` 与 SSRF 白名单、`SERVICE SILENT`、连接池、结果缓存、自适应超时、批 SERVICE 检测

### pg_ripple 是否支持 SPARQL 1.1 属性路径？

是的，截至 v0.5.0。支持所有标准路径运算符：`+`、`*`、`?`、`/`（序列）、`|`（替代）、`^`（逆）。否定属性集 `!(p1|p2)` 通过 `vp_rare` 部分支持。

属性路径查询使用 PostgreSQL 18 的 `CYCLE` 子句编译为 `WITH RECURSIVE` CTE，以进行基于哈希的循环检测。

### 属性路径的最大遍历深度是多少？

由 `pg_ripple.max_path_depth` GUC（默认值：100）控制。将其设置得较低以防止在密集图上的失控查询：

```sql
SET pg_ripple.max_path_depth = 10;
```

### 为什么我的 FILTER 与数字不匹配？

SPARQL FILTER 对数字字面值的比较（`FILTER(?age >= 18)`）要求字面值使用 XSD 数字类型进行类型化：

```
"18"^^<http://www.w3.org/2001/XMLSchema#integer>
```

像 `"18"` 这样的纯字符串字面值将作为字符串进行比较。插入数字数据时使用类型化字面值，或在 FILTER 表达式中进行转换。

---

## 数据建模

### 命名图和空白节点之间有什么区别？

**命名图** 是由 IRI 标识的三元组集合。它用于按来源、时间或主题对数据进行分区。您可以跨所有命名图查询、在特定图内查询，或计算每个图的三元组数。

**空白节点** 是没有全局 IRI 标识的资源 — 它仅在文档加载范围内具有标识。空白节点用于匿名资源（例如结构中的中间节点），不需要稳定的标识符。

### 什么是 RDF-star 引号三元组？

引号三元组 `<< s p o >>` 是可以出现在另一个三元组的主语或宾语位置的三元组。它支持关于三元组的*声明* — 用于来源（`<< alice knows bob >> :assertedBy :carol`）、时间注释和置信度分数很有用。

pg_ripple 将引号三元组存储为 `kind = 5` 的字典条目。请参阅 [RDF-star](../user-guide/sql-reference/rdf-star.md) 了解详细信息。

---

## 性能

### 批量加载有多快？

在具有 NVMe SSD 的现代服务器上，`load_ntriples()` 每秒处理大约 50,000–150,000 个三元组（单连接、默认设置）。性能取决于谓词多样性（更多唯一谓词 → 创建更多 VP 表）、硬件和 PostgreSQL 配置。

### 何时应使用 SPARQL 与 `find_triples`？

`find_triples()` 仅匹配单个 `(s, p, o, g)` 模式 — 它等效于恰好一个三元组模式的 SPARQL BGP。对于单模式查找使用它。

对于更复杂的内容使用 `sparql()`：多模式连接、OPTIONAL、FILTER、聚合、属性路径，或当您想要 SPARQL 的变量绑定模型的便利性时。

---

## HTAP 和运维（v0.6.0）

### pg_ripple 是否需要 shared_preload_libraries？

对于完整的 HTAP 功能（后台合并工作线程、闩锁触发钩子、共享内存统计），您必须将 pg_ripple 添加到 `shared_preload_libraries`：

```ini
shared_preload_libraries = 'pg_ripple'
```

没有它，扩展仍然可以读取和写入 — 但所有写入都保留在 delta 表中，并且永远不会自动合并到 main 中。具有大 delta 的谓词上的查询将比预期更慢。

请参阅[部署前检查表](../operations/deployment.md)了解完整的设置顺序。

### `compact()` 和合并工作线程之间有什么区别？

| | `compact()` | 合并工作线程 |
|---|---|---|
| **触发器** | 手动 SQL 调用 | 自动（闩锁触发或计时器） |
| **阻塞调用者** | 是 | 否 — 在后台运行 |
| **何时使用** | 维护窗口、测试 | 生产持续运行 |

两者产生相同的结果：delta 行被移动到 main，清除墓碑，并构建新的 BRIN 索引。

### 如何知道合并工作线程是否跟得上？

```sql
-- 检查未合并的行数
SELECT pg_ripple.stats() -> 'unmerged_delta_rows';

-- 随时间观察
SELECT now(), (pg_ripple.stats() -> 'unmerged_delta_rows')::int AS lag
FROM generate_series(1, 10) g,
     pg_sleep(5) AS _s
WHERE true;  -- 在循环中手动运行此操作
```

健康的部署显示 `unmerged_delta_rows` 在写入期间上升，在合并后下降。如果它仅上升，则工作线程落后 — 降低 `merge_threshold` 或增加服务器 I/O 容量。

### 我可以实时订阅三元组更改吗？

可以。CDC（Change Data Capture）从 v0.6.0 开始通过 PostgreSQL `NOTIFY` 提供：

```sql
-- 订阅特定谓词
SELECT pg_ripple.subscribe('<https://schema.org/name>', 'name_changes');

-- 在另一个会话中
LISTEN name_changes;

-- 插入或删除三元组时通知到达
SELECT pg_ripple.insert_triple(
    '<https://example.org/Alice>',
    '<https://schema.org/name>',
    '"Alice"'
);
```

订阅存储在 `_pg_ripple.cdc_subscriptions` 中，并在重新连接后保留（但必须在服务器重启后重新注册）。请参阅[管理](../user-guide/sql-reference/admin.md#subscribepattern-channel)参考了解详细信息。

### 为什么我的查询看不到最近插入的三元组？

如果您插入了三元组并立即使用 SPARQL 查询，结果应包括这些三元组 — delta 表始终与主表一起查询。

如果三元组缺失，请检查：
1. 三元组已提交（不在未提交的事务内）
2. 正在查询正确的图（默认图与命名图）
3. 使用了正确的谓词 IRI 拼写

### HTTP 端点 URL 是什么？

`pg_ripple_http` 配套服务默认监听 `http://localhost:7878/sparql`。使用 `PG_RIPPLE_HTTP_PORT` 配置端口。该 URL 根据 W3C SPARQL 1.1 协议接受 GET 和 POST SPARQL 请求。

### 如何将 SPARQL 工具连接到 pg_ripple？

在 PostgreSQL 实例旁边启动 `pg_ripple_http`。将任何 SPARQL 客户端（YASGUI、Protege、SPARQLWrapper、Jena）指向 `http://localhost:7878/sparql`。端点支持标准内容协商（`Accept: application/sparql-results+json`、`text/turtle` 等）。

### 我可以在 Docker 中运行 pg_ripple_http 吗？

可以。Docker 镜像捆绑了 PostgreSQL 和 `pg_ripple_http`。使用提供的 `docker-compose.yml` 运行 `docker compose up` 来启动两个服务。SPARQL 端点默认暴露在 7878 端口。

---

## JSON-LD 框架（v0.17.0）

### 什么是 JSON-LD 框架，它与纯 JSON-LD 导出有何不同？

纯 JSON-LD 导出（`export_jsonld`）将图中的每个三元组序列化为平面节点对象列表。JSON-LD 框架允许您使用框架文档指定所需的输出*形状* — 选择哪些类型、包括哪些属性以及如何嵌套相关节点。结果是嵌套的、结构化的 JSON-LD 文档，适合直接从 REST API 提供。

性能上的关键区别：框架仅读取框架触及的 VP 表。针对具有 10,000 个谓词的图上的 3 个谓词的框架读取 3 个 VP 表，而不是 10,000 个。

### 支持哪些 W3C 框架功能？

pg_ripple v0.17.0 支持：`@type` 匹配、`@id` 匹配、属性通配符 `{}`、缺席属性模式 `[]`、`@reverse`、`@embed`（`@once`/`@always`/`@never`）、`@explicit`、`@omitDefault`、`@default`、`@requireAll`、`@context` 压缩、命名图 `@graph` 范围和 `@omitGraph`。

值模式匹配（值对象内的 `@value`/`@language`/`@type`）推迟到未来的版本。

### 什么是值模式匹配，为什么会被推迟？

值模式匹配将允许像 `{"ex:name": {"@language": "en"}}` 这样的框架仅选择英语名称字面值。正确实现此功能需要全图扫描以查找匹配的字面值 — 无法使用 VP 表连接模型高效完成。它被推迟到有针对性的字面值索引可用时。

### 框架视图和 SPARQL 视图之间有什么区别？

SPARQL 视图（`create_sparql_view`）将原始 SPARQL SELECT 结果存储为流表中的整数 ID 列。框架视图（`create_framing_view`）通过 CONSTRUCT 结果运行完整的嵌入和压缩管道，因此流表中的每行包含一个随时可以服务的嵌套 JSON-LD 文档，而不是原始投影值。

当您需要结果绑定的低级访问时使用 SPARQL 视图；当您希望 API 随时可以服务的嵌套 JSON-LD 时使用框架视图。

---

## 向量联邦（v0.28.0）

### 向量联邦如何工作？

使用 `pg_ripple.register_vector_endpoint(url, api_type)` 注册外部端点后，pg_ripple 可以将相似度查询路由到 Weaviate、Qdrant、Pinecone 或远程 pgvector 实例。结果使用 `hybrid_search()` 中的倒数排名融合与本地三元组存储数据合并。

### 使用向量联邦时如何防止 SSRF 攻击？

pg_ripple 不限制可以注册哪些 URL。您应该使用网络策略（例如 Kubernetes NetworkPolicy、AWS 安全组）来限制 PostgreSQL 服务器可以访问的外部主机。仅注册属于您基础架构中受信任的向量服务的端点。

### 为什么我的联邦查询超时？

默认超时为 5000 毫秒。使用以下命令增加它：

```sql
SET pg_ripple.vector_federation_timeout_ms = 30000;
```

或通过 `ALTER SYSTEM SET pg_ripple.vector_federation_timeout_ms = 30000; SELECT pg_reload_conf();` 全局配置它。

### 如何配置远程端点的 API 密钥？

pg_ripple 不为外部向量服务存储 API 密钥。如果服务支持，请在端点 URL 中传递 API 密钥，或在调用端点之前通过应用程序层中的环境变量进行配置。