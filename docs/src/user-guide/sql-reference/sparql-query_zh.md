# SPARQL 查询

pg_ripple 通过将 SPARQL 1.1 查询编译为在 PostgreSQL 引擎中原生运行的 SQL 来执行它们。生成的 SQL 通过 `sparql_explain()` 可见。

## sparql

```sql
pg_ripple.sparql(query TEXT) RETURNS TABLE(…)
```

执行 SPARQL SELECT 查询并将结果作为关系表返回。每个投影的变量都成为一个 `TEXT` 列。IRI 和空节点的值以 N-Triples 表示法返回，字面量的值以字面量表示法返回。

```sql
SELECT name, age
FROM pg_ripple.sparql('
  SELECT ?name ?age WHERE {
    ?person <https://example.org/name> ?name .
    OPTIONAL { ?person <https://example.org/age> ?age }
  }
');
```

> **注意**：返回表中的列名与 SPARQL 变量名匹配，没有 `?` 前缀。

---

## sparql_ask

```sql
pg_ripple.sparql_ask(query TEXT) RETURNS BOOLEAN
```

执行 SPARQL ASK 查询，如果至少存在一个解则返回 `true`。

```sql
SELECT pg_ripple.sparql_ask(
    'ASK { <https://example.org/alice> <https://example.org/knows> ?x }'
);
```

---

## sparql_explain

```sql
pg_ripple.sparql_explain(query TEXT, verbose BOOLEAN DEFAULT FALSE) RETURNS TEXT
```

返回为 SPARQL 查询生成的 SQL，但不执行它。用于调试和性能分析。

```sql
SELECT pg_ripple.sparql_explain(
    'SELECT ?name WHERE { ?p <https://example.org/name> ?name }',
    false
);
```

---

## 支持的 SPARQL 1.1 功能

### SELECT

```sparql
SELECT ?s ?p WHERE { ?s ?p <https://example.org/bob> }
SELECT DISTINCT ?s WHERE { ?s <ex:knows> ?o }
SELECT ?s WHERE { ?s <ex:knows> ?o } ORDER BY ?s LIMIT 10 OFFSET 5
```

### FILTER

```sparql
SELECT ?name WHERE {
  ?p <ex:name> ?name .
  FILTER(STRLEN(?name) > 3)
}

SELECT ?age WHERE {
  ?p <ex:age> ?age .
  FILTER(?age >= 18 && ?age < 65)
}
```

### OPTIONAL (LeftJoin)

```sparql
SELECT ?person ?name ?email WHERE {
  ?person <ex:worksAt> <ex:acme> .
  OPTIONAL { ?person <ex:name> ?name }
  OPTIONAL { ?person <ex:email> ?email }
}
```

### UNION / MINUS

```sparql
-- UNION
SELECT ?contact WHERE {
  { ?alice <ex:knows> ?contact } UNION { ?bob <ex:knows> ?contact }
}

-- MINUS (反连接)
SELECT ?person WHERE {
  ?person <ex:worksAt> ?company .
  MINUS { ?person <ex:worksAt> <ex:acme> }
}
```

### 聚合和 GROUP BY

```sparql
SELECT ?company (COUNT(?person) AS ?headcount) WHERE {
  ?person <ex:worksAt> ?company
} GROUP BY ?company HAVING (COUNT(?person) >= 2)
```

支持的聚合函数：`COUNT`、`SUM`、`AVG`、`MIN`、`MAX`、`GROUP_CONCAT`。

### 子查询

```sparql
SELECT ?company ?headcount WHERE {
  {
    SELECT ?company (COUNT(?p) AS ?headcount) WHERE {
      ?p <ex:worksAt> ?company
    } GROUP BY ?company
  }
  FILTER(?headcount >= 2)
}
```

### BIND / VALUES

```sparql
-- BIND
SELECT ?person ?label WHERE {
  ?person <ex:worksAt> ?company .
  BIND(<ex:employee> AS ?label)
}

-- VALUES (内联数据)
SELECT ?person ?company WHERE {
  VALUES ?person { <ex:alice> <ex:bob> }
  ?person <ex:worksAt> ?company
}
```

### 属性路径

```sparql
-- OneOrMore (+)：传递闭包
SELECT ?target WHERE { <ex:alice> <ex:knows>+ ?target }

-- ZeroOrMore (*)：包含恒等的传递闭包
SELECT ?target WHERE { <ex:alice> <ex:follows>* ?target }

-- ZeroOrOne (?)：直接或恒等
SELECT ?target WHERE { <ex:alice> <ex:follows>? ?target }

-- 序列 (/)
SELECT ?target WHERE { <ex:alice> <ex:knows>/<ex:knows> ?target }

-- 选择 (|)
SELECT ?target WHERE { <ex:alice> (<ex:knows>|<ex:follows>) ?target }

-- 反向 (^)
SELECT ?who WHERE { ?who ^<ex:knows> <ex:bob> }
```

属性路径编译为带 PG18 `CYCLE` 子句的 PostgreSQL `WITH RECURSIVE` CTE，用于基于哈希的循环检测。参阅 [max_path_depth](../../operations/configuration.md#max_path_depth) 限制遍历深度。

### 命名图

```sparql
SELECT ?s ?p ?o WHERE {
  GRAPH <https://example.org/graph1> { ?s ?p ?o }
}
```

### ASK

```sparql
ASK { <ex:alice> <ex:knows> <ex:bob> }
```

---

## 计划缓存

已编译的 SPARQL→SQL 计划缓存在每个后端的 LRU 缓存中（可通过 [`pg_ripple.plan_cache_size`](../../operations/configuration.md#plan_cache_size) 配置）。重复的相同查询会跳过重新编译。

缓存键包括查询文本和 `pg_ripple.max_path_depth` 的当前值。更改 GUC 会使缓存的路径查询计划失效。

---

## sparql_construct

```sql
pg_ripple.sparql_construct(query TEXT) RETURNS SETOF JSONB
```

执行 SPARQL CONSTRUCT 查询并将构造的三元组作为 JSONB 对象返回。每个结果行有三个键：`"s"`（主语）、`"p"`（谓词）和 `"o"`（对象），都使用 N-Triples 表示法。

### 显式模板形式

```sql
SELECT *
FROM pg_ripple.sparql_construct('
    CONSTRUCT { ?b <https://example.org/knownBy> ?a }
    WHERE { ?a <https://example.org/knows> ?b }
');
-- 返回：{"s": "<https://...bob>", "p": "<https://...knownBy>", "o": "<https://...alice>"}
```

### CONSTRUCT WHERE（裸形式）

`CONSTRUCT WHERE` 简写直接返回匹配的三元组：

```sql
SELECT *
FROM pg_ripple.sparql_construct('
    CONSTRUCT WHERE { <https://example.org/alice> <https://example.org/knows> ?o }
');
```

---

## sparql_describe

```sql
pg_ripple.sparql_describe(query TEXT, strategy TEXT DEFAULT current_setting('pg_ripple.describe_strategy'))
    RETURNS SETOF JSONB
```

执行 SPARQL DESCRIBE 查询并将命名资源的描述作为 JSONB 三元组 `{s, p, o}` 返回。

```sql
-- 描述单个资源（CBD 算法）
SELECT *
FROM pg_ripple.sparql_describe(
    'DESCRIBE <https://example.org/alice>'
);

-- 描述所有人（由 WHERE 模式标识的资源）
SELECT *
FROM pg_ripple.sparql_describe(
    'DESCRIBE ?person WHERE { ?person a <https://example.org/Person> }'
);
```

### describe_strategy GUC

`pg_ripple.describe_strategy`（默认值：`'cbd'`）设置默认的展开算法：

| 值 | 算法 | 描述 |
|---|---|---|
| `'cbd'` | Concise Bounded Description | 所有出向弧；递归展开空节点对象 |
| `'scbd'` | Symmetric CBD | CBD + 命名资源的所有入向弧 |
| `'simple'` | 简单描述 | 仅出向弧；不递归空节点 |

```sql
-- 在此会话中使用 SCBD
SET pg_ripple.describe_strategy = 'scbd';

SELECT * FROM pg_ripple.sparql_describe('DESCRIBE <https://example.org/alice>');
```

您也可以将策略作为第二个参数传递给 `sparql_describe`：

```sql
SELECT * FROM pg_ripple.sparql_describe(
    'DESCRIBE <https://example.org/alice>',
    'scbd'
);
```

> **注意**：CONSTRUCT 和 DESCRIBE 在 v0.5.1 中返回 JSONB。Turtle 和 JSON-LD 序列化输出计划在 v0.9.0 中提供。

---

## HTTP 协议端点

pg_ripple 包含一个伴生 HTTP 服务（`pg_ripple_http`），它实现了 W3C SPARQL 1.1 协议，允许标准 SPARQL 客户端在没有任何特定于 pg_ripple 的驱动程序的情况下连接。

### 启动 HTTP 服务

```bash
export PG_RIPPLE_HTTP_PG_URL="postgresql://user:pass@localhost/mydb"
export PG_RIPPLE_HTTP_PORT=7878
pg_ripple_http
```

### 配置

| 环境变量 | 默认值 | 描述 |
|---|---|---|
| `PG_RIPPLE_HTTP_PG_URL` | `postgresql://localhost/postgres` | PostgreSQL 连接字符串 |
| `PG_RIPPLE_HTTP_PORT` | `7878` | HTTP 监听端口 |
| `PG_RIPPLE_HTTP_POOL_SIZE` | `16` | 连接池大小 |
| `PG_RIPPLE_HTTP_AUTH_TOKEN` | *(无)* | Bearer/Basic 认证令牌 |
| `PG_RIPPLE_HTTP_CORS_ORIGINS` | `*` | 逗号分隔的 CORS 来源 |
| `PG_RIPPLE_HTTP_RATE_LIMIT` | `0` | 速率限制（0 = 无限制）|

### SPARQL 1.1 协议一致性

`/sparql` 处的端点支持所有标准请求形式：

- `GET /sparql?query=...`（URL 编码查询）
- `POST /sparql`，`Content-Type: application/sparql-query`
- `POST /sparql`，`Content-Type: application/sparql-update`
- `POST /sparql`，`Content-Type: application/x-www-form-urlencoded`（`query=...` 或 `update=...`）

### Accept 标头格式

| Accept 标头 | 用于 | MIME 类型 |
|---|---|---|
| `application/sparql-results+json` | SELECT, ASK（默认）| JSON 结果 |
| `application/sparql-results+xml` | SELECT, ASK | XML 结果 |
| `text/csv` | SELECT | CSV |
| `text/tab-separated-values` | SELECT | TSV |
| `text/turtle` | CONSTRUCT, DESCRIBE（默认）| Turtle |
| `application/n-triples` | CONSTRUCT, DESCRIBE | N-Triples |
| `application/ld+json` | CONSTRUCT, DESCRIBE | JSON-LD |

对于真正的增量传递，SELECT/ASK 请求 `application/sparql-results+json`、`text/csv` 或 `text/tab-separated-values`，CONSTRUCT/DESCRIBE 请求 `application/n-triples`。`/sparql` 端点及其 `POST /sparql/stream` 兼容性别名使用相同的管道。流式响应保留类型化的 RDF 项并应用 PostgreSQL 背压；XML、Turtle 和 JSON-LD 仍保持缓冲。

### 示例

```bash
# SELECT 查询
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"

# 带 JSON 结果的 ASK 查询
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=ASK { <http://example.org/alice> ?p ?o }"

# 带 Turtle 输出的 CONSTRUCT 查询
curl -H "Accept: text/turtle" -G http://localhost:7878/sparql \
  --data-urlencode "query=CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o } LIMIT 10"

# 通过 POST 的 SPARQL 更新
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-update" \
  -d "INSERT DATA { <http://example.org/s> <http://example.org/p> \"value\" }"

# 健康检查
curl http://localhost:7878/health

# Prometheus 指标
curl http://localhost:7878/metrics
```

### Docker

使用 Docker Compose 一起运行带有 pg_ripple 的 PostgreSQL 和 HTTP 端点：

```bash
docker compose up -d
curl http://localhost:7878/health
```

### 连接 SPARQL 工具

`/sparql` 端点与标准 SPARQL 工具兼容：

- **YASGUI**：将端点 URL 设置为 `http://localhost:7878/sparql`
- **Python SPARQLWrapper**：`sparql = SPARQLWrapper("http://localhost:7878/sparql")`
- **Apache Jena**：`QueryExecutionFactory.sparqlService("http://localhost:7878/sparql", query)`
- **Protege**：添加 SPARQL 选项卡，将端点设置为 `http://localhost:7878/sparql`