# SPARQL 协议（HTTP 端点）

pg_ripple v0.15.0 附带一个伴生 HTTP 服务（`pg_ripple_http`），它实现了 [W3C SPARQL 1.1 协议](https://www.w3.org/TR/sparql11-protocol/)。任何标准的 SPARQL 客户端 — YASGUI、Protégé、SPARQLWrapper、Jena 或纯 `curl` — 都可以在无需特定于驱动程序的配置的情况下查询 pg_ripple。

---

## 架构

`pg_ripple_http` 是一个独立的 Rust 二进制文件，使用 [axum](https://github.com/tokio-rs/axum) 和 [tokio](https://tokio.rs/) 构建。它通过 [deadpool-postgres](https://crates.io/crates/deadpool-postgres) 连接到 PostgreSQL，将 HTTP 请求转换为对 `pg_ripple.sparql()`、`sparql_ask()`、`sparql_construct()`、`sparql_describe()` 和 `sparql_update()` 的调用，然后根据请求的内容类型格式化结果。

```
┌────────────┐    HTTP     ┌──────────────────┐    SQL/SPI    ┌────────────┐
│   Client   │ ──────────► │  pg_ripple_http   │ ────────────► │ PostgreSQL │
│  (YASGUI,  │ ◄─────────── │  (axum + tokio)   │ ◄──────────── │ + pg_ripple│
│  curl, …)  │   JSON/XML  └──────────────────┘               └────────────┘
└────────────┘
```

---

## Docker Compose 快速入门

运行 PostgreSQL 和 HTTP 端点的最简单方法：

```bash
docker compose up
```

这将启动两个容器：

| 服务 | 端口 | 描述 |
|---------|------|-------------|
| `postgres` | 5432 | 已安装 pg_ripple 的 PostgreSQL 18 |
| `sparql` | 7878 | SPARQL 查询的 HTTP 端点 |

运行后：

```bash
# 健康检查
curl http://localhost:7878/health

# 运行 SPARQL 查询
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
```

---

## 端点

### GET /sparql

通过 URL 参数查询。用于简单查询和基于浏览器的工具。

```
GET /sparql?query=SELECT+?s+?p+?o+WHERE+{+?s+?p+?o+}+LIMIT+10
```

### POST /sparql

通过请求体查询。支持两种内容类型：

| Content-Type | 主体格式 |
|---|---|
| `application/sparql-query` | 原始 SPARQL 查询文本 |
| `application/x-www-form-urlencoded` | `query=...`（URL 编码） |

```bash
# 原始 SPARQL 主体
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-query" \
  -d "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"

# 表单编码
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "query=SELECT+?s+?p+?o+WHERE+{+?s+?p+?o+}+LIMIT+10"
```

### POST /sparql（更新）

SPARQL 更新操作使用 `Content-Type: application/sparql-update`：

```bash
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-update" \
  -d "INSERT DATA { <http://example.org/alice> <http://example.org/name> \"Alice\" }"
```

### GET /health

当服务就绪时返回 `200 OK`。用于负载均衡器健康检查。

### GET /metrics

Prometheus 兼容的指标：

| 指标 | 描述 |
|--------|-------------|
| `pg_ripple_query_count` | 已处理的 SPARQL 查询总数 |
| `pg_ripple_error_count` | 查询错误总数 |
| `pg_ripple_query_duration_seconds_total` | 累计查询执行时间 |

```bash
curl http://localhost:7878/metrics
```

---

## 内容协商

设置 `Accept` 标头以选择响应格式：

| Accept 标头 | 格式 | 适用于 |
|---|---|---|
| `application/sparql-results+json` | SPARQL 结果 JSON | JavaScript 应用程序、YASGUI |
| `application/sparql-results+xml` | SPARQL 结果 XML | Java/Jena 客户端 |
| `text/csv` | CSV | 电子表格、pandas |
| `text/tab-separated-values` | TSV | CLI 管道 |
| `text/turtle` | Turtle | CONSTRUCT/DESCRIBE 结果 |
| `application/n-triples` | N-Triples | 流式管道 |
| `application/ld+json` | JSON-LD | REST API、关联数据平台 |

如果未设置 `Accept` 标头，则 SELECT/ASK 默认为 `application/sparql-results+json`，CONSTRUCT/DESCRIBE 默认为 `text/turtle`。

```bash
# 将结果作为 CSV 获取
curl -G http://localhost:7878/sparql \
  -H "Accept: text/csv" \
  --data-urlencode "query=SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5"

# 将 CONSTRUCT 结果作为 JSON-LD 获取
curl -G http://localhost:7878/sparql \
  -H "Accept: application/ld+json" \
  --data-urlencode "query=CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o } LIMIT 5"
```

---

## 身份验证

`pg_ripple_http` 支持 Bearer 令牌和 HTTP Basic 身份验证。通过环境变量进行配置：

| 变量 | 描述 |
|----------|-------------|
| `PG_RIPPLE_HTTP_AUTH_TOKEN` | 如果设置，所有请求必须包含 `Authorization: Bearer <token>` 或 `Authorization: Basic <token>` |

未设置时，禁用身份验证。

```bash
# 使用 Bearer 令牌身份验证启动
PG_RIPPLE_HTTP_AUTH_TOKEN=my-secret-token ./pg_ripple_http

# 使用身份验证进行查询
curl -G http://localhost:7878/sparql \
  -H "Authorization: Bearer my-secret-token" \
  --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 5"
```

---

## CORS

默认情况下通过 `tower-http` 启用跨源资源共享标头，允许基于浏览器的工具（如 YASGUI）直接查询端点。

---

## 配置

所有配置都通过环境变量进行：

| 变量 | 默认值 | 描述 |
|----------|---------|-------------|
| `PG_RIPPLE_HTTP_PG_URL` | `postgresql://localhost/postgres` | 完整的 PostgreSQL 连接 URL |
| `PG_RIPPLE_HTTP_PORT` | `7878` | HTTP 监听端口 |
| `PG_RIPPLE_HTTP_POOL_SIZE` | `16` | 池中的最大连接数 |
| `PG_RIPPLE_HTTP_AUTH_TOKEN` | （未设置） | Bearer/Basic 身份验证令牌 |
| `PG_RIPPLE_HTTP_RATE_LIMIT` | `0` | 每个客户端 IP 的最大请求数/秒（0 = 禁用） |
| `PG_RIPPLE_HTTP_CORS_ORIGINS` | `*` | 允许的 CORS 来源（以逗号分隔） |

示例：

```bash
export PG_RIPPLE_HTTP_PG_URL="postgresql://user:password@db-host:5432/mydb"
export PG_RIPPLE_HTTP_PORT=8080
export PG_RIPPLE_HTTP_POOL_SIZE=32
export PG_RIPPLE_HTTP_AUTH_TOKEN="my-secret-token"
./pg_ripple_http
```

---

## 客户端示例

### Python (SPARQLWrapper)

```python
from SPARQLWrapper import SPARQLWrapper, JSON

sparql = SPARQLWrapper("http://localhost:7878/sparql")
sparql.setQuery("SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10")
sparql.setReturnFormat(JSON)
results = sparql.query().convert()

for row in results["results"]["bindings"]:
    print(row["s"]["value"], row["p"]["value"], row["o"]["value"])
```

### JavaScript（fetch）

```javascript
const response = await fetch('http://localhost:7878/sparql', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/sparql-query',
    'Accept': 'application/sparql-results+json'
  },
  body: 'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10'
});
const data = await response.json();
```

### curl

```bash
# SELECT
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=SELECT ?s WHERE { ?s a <http://xmlns.com/foaf/0.1/Person> }"

# ASK
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=ASK { <http://example.org/alice> ?p ?o }"

# CONSTRUCT 作为 Turtle
curl -G http://localhost:7878/sparql \
  -H "Accept: text/turtle" \
  --data-urlencode "query=CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o } LIMIT 100"
```

### Apache Jena (Java)

```java
QueryExecution qe = QueryExecution.service("http://localhost:7878/sparql")
    .query("SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10")
    .build();
ResultSet rs = qe.execSelect();
ResultSetFormatter.out(rs);
```