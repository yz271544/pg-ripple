# §2.8 API 和集成

## 是什么和为什么

pg_ripple 的 SQL 函数功能强大，但大多数应用程序不直接与 PostgreSQL 对话。**pg_ripple_http** 配套服务通过 HTTP 公开符合 W3C 的 SPARQL 协议端点，因此任何 SPARQL 客户端、编程语言或工具都可以查询您的知识图。

本章涵盖：

- **pg_ripple_http**：独立 SPARQL 端点服务。
- **应用程序代码示例**：Python、JavaScript 和 Java。
- **SPARQL 联合**：从 pg_ripple 内部查询远程 SPARQL 端点。
- **缓存策略**：计划缓存、连接池和结果缓存。

---

## 它是如何工作的

### pg_ripple_http 架构

`pg_ripple_http` 是一个独立的 Rust 二进制文件（不是 PostgreSQL 扩展），它：

1. 通过 **deadpool** 连接池连接到 PostgreSQL。
2. 通过 HTTP GET/POST（W3C SPARQL 协议）接收 SPARQL 查询。
3. 通过 SQL 调用 `pg_ripple.sparql()`、`pg_ripple.sparql_construct()` 等。
4. 以标准格式返回结果：SPARQL Results JSON/XML、Turtle、N-Triples、JSON-LD。
5. 公开一个 `/rag` 端点用于 AI 检索。

### 支持的端点

| 方法 | 路径 | 内容类型 | 描述 |
|---|---|---|---|
| GET | `/sparql?query=...` | Accept 标头 | 通过 URL 参数的 SPARQL 查询 |
| POST | `/sparql` | `application/sparql-query` | 请求正文中的 SPARQL 查询 |
| POST | `/sparql/bindings` | `application/json` | 具有类型化初始绑定的只读 SPARQL 查询 |
| POST | `/sparql` | `application/x-www-form-urlencoded` | 作为表单参数的 SPARQL 查询 |
| POST | `/sparql` | `application/sparql-update` | 请求正文中的 SPARQL Update |
| POST | `/rag` | `application/json` | RAG 检索端点 |
| GET | `/health` | `application/json` | 健康检查 |
| GET | `/metrics` | `text/plain` | Prometheus 指标 |

### 响应格式（内容协商）

| Accept 标头 | 格式 |
|---|---|
| `application/sparql-results+json` | SPARQL Results JSON（SELECT/ASK 的默认值）|
| `application/sparql-results+xml` | SPARQL Results XML |
| `text/csv` | CSV |
| `text/tab-separated-values` | TSV |
| `text/turtle` | Turtle（用于 CONSTRUCT/DESCRIBE）|
| `application/n-triples` | N-Triples（用于 CONSTRUCT/DESCRIBE）|
| `application/ld+json` | JSON-LD（用于 CONSTRUCT/DESCRIBE）|

---

## 详细的示例

### 启动 pg_ripple_http

```bash
# 设置环境变量
export PG_RIPPLE_DATABASE_URL="postgresql://user:pass@localhost:5432/mydb"
export PG_RIPPLE_LISTEN="0.0.0.0:8080"
export PG_RIPPLE_AUTH_TOKEN="my-secret-token"  # 可选

# 启动服务器
pg_ripple_http
```

通过环境变量进行配置：

| 变量 | 默认值 | 描述 |
|---|---|---|
| `PG_RIPPLE_DATABASE_URL` | `postgresql://localhost/postgres` | PostgreSQL 连接字符串 |
| `PG_RIPPLE_LISTEN` | `127.0.0.1:8080` | 侦听地址和端口 |
| `PG_RIPPLE_AUTH_TOKEN` | （无）| 用于身份验证的 Bearer 令牌 |
| `PG_RIPPLE_POOL_SIZE` | `10` | 连接池大小 |
| `PG_RIPPLE_RATE_LIMIT` | `100` | 每个 IP 的每秒请求数 |
| `PG_RIPPLE_CORS_ORIGIN` | `*` | CORS 允许的来源 |

### 通过 curl 查询

**通过 GET 的 SPARQL SELECT：**

```bash
curl -G http://localhost:8080/sparql \
  --data-urlencode 'query=PREFIX dct: <http://purl.org/dc/terms/> SELECT ?paper ?title WHERE { ?paper dct:title ?title } LIMIT 10' \
  -H "Accept: application/sparql-results+json"
```

**通过 POST（正文）的 SPARQL SELECT：**

```bash
curl -X POST http://localhost:8080/sparql \
  -H "Content-Type: application/sparql-query" \
  -H "Accept: application/sparql-results+json" \
  -d 'PREFIX dct: <http://purl.org/dc/terms/>
      PREFIX bibo: <http://purl.org/ontology/bibo/>
      SELECT ?paper ?title
      WHERE {
          ?paper a bibo:AcademicArticle ;
                 dct:title ?title .
      }
      ORDER BY ?title
      LIMIT 20'
```

**作为 Turtle 的 SPARQL CONSTRUCT：**

```bash
curl -X POST http://localhost:8080/sparql \
  -H "Content-Type: application/sparql-query" \
  -H "Accept: text/turtle" \
  -d 'PREFIX dct: <http://purl.org/dc/terms/>
      PREFIX ex: <https://example.org/>
      CONSTRUCT { ?paper ex:hasTitle ?title }
      WHERE { ?paper dct:title ?title }'
```

**作为 JSON-LD 的 SPARQL CONSTRUCT：**

```bash
curl -X POST http://localhost:8080/sparql \
  -H "Content-Type: application/sparql-query" \
  -H "Accept: application/ld+json" \
  -d 'PREFIX dct: <http://purl.org/dc/terms/>
      PREFIX ex: <https://example.org/>
      CONSTRUCT { ?paper ex:hasTitle ?title }
      WHERE { ?paper dct:title ?title }'
```

**SPARQL Update：**

```sql
curl -X POST http://localhost:8080/sparql \
  -H "Content-Type: application/sparql-update" \
  -d 'PREFIX ex: <https://example.org/>
      PREFIX dct: <http://purl.org/dc/terms/>
      INSERT DATA {
          ex:paper/new1 dct:title "A New Discovery" .
      }'
```

**RAG 端点：**

```bash
curl -X POST http://localhost:8080/rag \
  -H "Content-Type: application/json" \
  -d '{
    "question": "What papers discuss knowledge graphs?",
    "sparql_filter": "?entity a <http://purl.org/ontology/bibo/AcademicArticle> .",
    "k": 5,
    "output_format": "jsonld"
  }'
```

RAG 响应包括一个 `context` 字段，其中包含为 LLM 提示预格式化的文本：

```json
{
  "results": [
    {
      "entity_iri": "https://example.org/paper/42",
      "label": "Knowledge Graphs in Practice",
      "context_json": {"@type": ["AcademicArticle"], "...": "..."},
      "distance": 0.12
    }
  ],
  "context": "Knowledge Graphs in Practice (AcademicArticle): A comprehensive survey..."
}
```

**身份验证**（当设置了 `PG_RIPPLE_AUTH_TOKEN` 时）：

```bash
curl -X POST http://localhost:8080/sparql \
  -H "Authorization: Bearer my-secret-token" \
  -H "Content-Type: application/sparql-query" \
  -d 'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5'
```

### Python 与 psycopg2

通过 SQL 直接从 Python 查询 pg_ripple：

```python
import json
import psycopg2

conn = psycopg2.connect("dbname=mydb user=postgres")
cur = conn.cursor()

# 执行 SPARQL 查询
cur.execute("""
    SELECT * FROM pg_ripple.sparql(%s)
""", ("""
    PREFIX dct:  <http://purl.org/dc/terms/>
    PREFIX bibo: <http://purl.org/ontology/bibo/>
    
    SELECT ?paper ?title ?author
    WHERE {
        ?paper a bibo:AcademicArticle ;
               dct:title ?title ;
               dct:creator ?author .
    }
    ORDER BY ?title
    LIMIT 20
""",))

for row in cur.fetchall():
    result = json.loads(row[0])
    print(f"Paper: {result['paper']}")
    print(f"Title: {result['title']}")
    print(f"Author: {result['author']}")
    print()

# 加载 Turtle 数据
cur.execute("""
    SELECT pg_ripple.load_turtle(%s)
""", ("""
    @prefix ex: <https://example.org/> .
    @prefix dct: <http://purl.org/dc/terms/> .
    ex:paper/new dct:title "Loaded from Python" .
""",))
conn.commit()

# 导出为 JSON-LD
cur.execute("SELECT pg_ripple.export_jsonld()")
jsonld = json.loads(cur.fetchone()[0])
print(json.dumps(jsonld, indent=2))

cur.close()
conn.close()
```

### Python 与 SPARQLWrapper

使用标准 SPARQLWrapper 库查询 pg_ripple_http 端点：

```python
from SPARQLWrapper import SPARQLWrapper, JSON, TURTLE

# 指向 pg_ripple_http 端点
sparql = SPARQLWrapper("http://localhost:8080/sparql")

# SELECT 查询
sparql.setQuery("""
    PREFIX dct:  <http://purl.org/dc/terms/>
    PREFIX bibo: <http://purl.org/ontology/bibo/>
    
    SELECT ?paper ?title
    WHERE {
        ?paper a bibo:AcademicArticle ;
               dct:title ?title .
    }
    LIMIT 10
""")
sparql.setReturnFormat(JSON)
results = sparql.query().convert()

for binding in results["results"]["bindings"]:
    print(f"{binding['paper']['value']}: {binding['title']['value']}")

# CONSTRUCT 查询作为 Turtle
sparql.setQuery("""
    PREFIX dct: <http://purl.org/dc/terms/>
    PREFIX ex:  <https://example.org/>
    
    CONSTRUCT { ?paper ex:hasTitle ?title }
    WHERE { ?paper dct:title ?title }
""")
sparql.setReturnFormat(TURTLE)
turtle_output = sparql.query().convert()
print(turtle_output.decode("utf-8"))
```

### JavaScript 与 pg

从 Node.js 直接查询 pg_ripple：

```javascript
const { Client } = require('pg');

async function main() {
    const client = new Client({ connectionString: 'postgresql://localhost/mydb' });
    await client.connect();

    // SPARQL SELECT
    const { rows } = await client.query(
        `SELECT * FROM pg_ripple.sparql($1)`,
        [`
            PREFIX dct:  <http://purl.org/dc/terms/>
            PREFIX bibo: <http://purl.org/ontology/bibo/>
            
            SELECT ?paper ?title
            WHERE {
                ?paper a bibo:AcademicArticle ;
                       dct:title ?title .
            }
            LIMIT 10
        `]
    );

    for (const row of rows) {
        const result = row.result;
        console.log(`Paper: ${result.paper}, Title: ${result.title}`);
    }

    // 加载 Turtle
    const loadResult = await client.query(
        `SELECT pg_ripple.load_turtle($1)`,
        [`
            @prefix ex: <https://example.org/> .
            @prefix dct: <http://purl.org/dc/terms/> .
            ex:paper/fromjs dct:title "Loaded from JavaScript" .
        `]
    );
    console.log(`Loaded: ${loadResult.rows[0].load_turtle} triples`);

    // 导出 JSON-LD
    const jsonldResult = await client.query(`SELECT pg_ripple.export_jsonld()`);
    console.log(JSON.stringify(jsonldResult.rows[0].export_jsonld, null, 2));

    await client.end();
}

main().catch(console.error);
```

### JavaScript 与 fetch（HTTP 端点）

```javascript
async function sparqlQuery(query) {
    const response = await fetch('http://localhost:8080/sparql', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/sparql-query',
            'Accept': 'application/sparql-results+json',
        },
        body: query,
    });
    return response.json();
}

const results = await sparqlQuery(`
    PREFIX dct: <http://purl.org/dc/terms/>
    SELECT ?paper ?title
    WHERE { ?paper dct:title ?title }
    LIMIT 10
`);

for (const binding of results.results.bindings) {
    console.log(`${binding.paper.value}: ${binding.title.value}`);
}
```

### Java 与 JDBC

```java
import java.sql.*;
import org.json.JSONObject;

public class PgRippleExample {
    public static void main(String[] args) throws Exception {
        Connection conn = DriverManager.getConnection(
            "jdbc:postgresql://localhost:5432/mydb", "postgres", "password"
        );

        // SPARQL SELECT
        PreparedStatement stmt = conn.prepareStatement(
            "SELECT * FROM pg_ripple.sparql(?)"
        );
        stmt.setString(1,
            "PREFIX dct: <http://purl.org/dc/terms/> " +
            "PREFIX bibo: <http://purl.org/ontology/bibo/> " +
            "SELECT ?paper ?title " +
            "WHERE { " +
            "    ?paper a bibo:AcademicArticle ; " +
            "           dct:title ?title ." +
            "} LIMIT 10"
        );

        ResultSet rs = stmt.executeQuery();
        while (rs.next()) {
            String jsonStr = rs.getString("result");
            JSONObject result = new JSONObject(jsonStr);
            System.out.println("Paper: " + result.getString("paper"));
            System.out.println("Title: " + result.getString("title"));
        }
        rs.close();
        stmt.close();

        // 加载 Turtle
        PreparedStatement loadStmt = conn.prepareStatement(
            "SELECT pg_ripple.load_turtle(?)"
        );
        loadStmt.setString(1,
            "@prefix ex: <https://example.org/> .\n" +
            "@prefix dct: <http://purl.org/dc/terms/> .\n" +
            "ex:paper/fromjava dct:title \"Loaded from Java\" .\n"
        );
        ResultSet loadRs = loadStmt.executeQuery();
        if (loadRs.next()) {
            System.out.println("Loaded: " + loadRs.getLong(1) + " triples");
        }
        loadRs.close();
        loadStmt.close();

        conn.close();
    }
}
```

---

## SPARQL 联合

pg_ripple 可以使用 `SERVICE` 关键字从 SPARQL 查询内部查询远程 SPARQL 端点。这使您可以将本地数据与 Wikidata 或 DBpedia 等远程数据集连接起来。

#### 查询远程 SPARQL 端点

```sql
SELECT * FROM pg_ripple.sparql('
PREFIX dct:    <http://purl.org/dc/terms/>
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX wd:     <http://www.wikidata.org/entity/>
PREFIX wdt:    <http://www.wikidata.org/prop/direct/>

SELECT ?paper ?title ?wikidataLabel
WHERE {
    ?paper dct:title ?title ;
           dct:subject ?topic .
    
    SERVICE <https://query.wikidata.org/sparql> {
        ?topic rdfs:label ?wikidataLabel .
        FILTER (LANG(?wikidataLabel) = "en")
    }
}
LIMIT 10
');
```

#### 向量联合

注册外部向量服务以进行联合相似性搜索（有关完整详细信息，请参阅[向量联合](../user-guide/vector-federation.md)）：

```sql
-- 注册 Qdrant 端点
SELECT pg_ripple.register_vector_endpoint(
    'https://qdrant.internal:6333',
    'qdrant'
);

-- 注册 Weaviate 端点
SELECT pg_ripple.register_vector_endpoint(
    'https://weaviate.internal:8080',
    'weaviate'
);
```

```admonish tip
联合查询会增加网络延迟。设置超时以防止慢速远程端点阻塞本地查询：
<sql>
SET pg_ripple.vector_federation_timeout_ms = 5000;
</sql>
```
```

---

## 常见模式

### 模式：连接池

对于高流量应用程序，在应用程序和 PostgreSQL 之间使用连接池（PgBouncer、pgcat）：

```
App → PgBouncer（端口 6432）→ PostgreSQL（端口 5432）
```

pg_ripple_http 在内部使用自己的连接池（可通过 `PG_RIPPLE_POOL_SIZE` 配置）。

### 模式：结果缓存

在应用程序级别缓存 SPARQL 结果以用于频繁重复的查询：

```python
import json
import hashlib
import redis
import psycopg2

cache = redis.Redis()

def cached_sparql(query, ttl=300):
    key = f"sparql:{hashlib.sha256(query.encode()).hexdigest()}"
    cached = cache.get(key)
    if cached:
        return json.loads(cached)

    conn = psycopg2.connect("dbname=mydb")
    cur = conn.cursor()
    cur.execute("SELECT * FROM pg_ripple.sparql(%s)", (query,))
    results = [json.loads(row[0]) for row in cur.fetchall()]
    cur.close()
    conn.close()

    cache.setex(key, ttl, json.dumps(results))
    return results
```

### 模式：用于预计算结果的 SPARQL 视图

对于频繁运行的仪表板查询，创建 SPARQL 视图（需要 pg_trickle）：

```sql
-- 创建每个机构的论文计数的预计算视图
SELECT pg_ripple.create_sparql_view(
    'papers_by_institution',
    'PREFIX dct: <http://purl.org/dc/terms/>
     PREFIX schema: <https://schema.org/>
     PREFIX foaf: <http://xmlns.com/foaf/0.1/>
     SELECT ?inst ?instName (COUNT(DISTINCT ?paper) AS ?count)
     WHERE {
         ?paper dct:creator ?author .
         ?author schema:affiliation ?inst .
         ?inst foaf:name ?instName .
     }
     GROUP BY ?inst ?instName',
    '30s',
    true
);

-- 直接查询视图（即时，无需 SPARQL 解析）
SELECT * FROM pg_ripple.papers_by_institution;
```

### 模式：Prometheus 监控

pg_ripple_http 在 `/metrics` 公开 Prometheus 指标：

```bash
curl http://localhost:8080/metrics
```

指标包括：

- `pg_ripple_http_requests_total` — 按端点和状态的总请求数
- `pg_ripple_http_request_duration_seconds` — 请求延迟直方图
- `pg_ripple_http_active_connections` — 当前活动连接数

---

## 性能和权衡

### 直接 SQL 与 HTTP 端点

| 访问方法 | 延迟开销 | 适用于 |
|---|---|---|
| 直接 SQL（psycopg2、JDBC）| 无 | 服务器端应用程序、ETL |
| pg_ripple_http | 每个请求约 1-5 毫秒 | Web 应用程序、REST API、联合查询 |

### 连接池大小

经验法则：将池大小设置为 OLTP 工作负载的 `2 * CPU 核心数`。对于 SPARQL 密集型分析，`4 * CPU 核心数` 可能更好：

```bash
export PG_RIPPLE_POOL_SIZE=20
```

### 速率限制

pg_ripple_http 包括内置速率限制以防止滥用：

```bash
export PG_RIPPLE_RATE_LIMIT=100  # 每个 IP 的每秒请求数
```

对于面向公众的端点，与反向代理（nginx、Caddy）结合使用以获得额外的保护。

### CORS 配置

对于基于浏览器的应用程序：

```bash
export PG_RIPPLE_CORS_ORIGIN="https://myapp.example.com"
```

为开发设置为 `*`；在生产中限制为特定来源。

---

## 陷阱和调试

### 身份验证错误

如果设置了 `PG_RIPPLE_AUTH_TOKEN`，所有请求必须包括 `Authorization` 标头：

```
HTTP 401：缺少或无效的授权令牌
```

修复：在请求标头中包括 `Authorization: Bearer <token>`。

### 连接被拒绝

如果 pg_ripple_http 无法连接到 PostgreSQL：

```
错误：连接被拒绝（操作系统错误 61）
```

修复：检查 `PG_RIPPLE_DATABASE_URL` 并确保 PostgreSQL 正在运行并接受连接。

### 内容类型协商

如果您收到意外的响应格式，请检查 `Accept` 标头。pg_ripple_http 使用内容协商：

```bash
# 显式请求 JSON 结果
curl -H "Accept: application/sparql-results+json" ...

# 显式请求 CONSTRUCT 的 Turtle
curl -H "Accept: text/turtle" ...
```

### 联合超时

远程 SPARQL 端点可能很慢。如果联合查询超时：

```sql
-- 增加超时
SET pg_ripple.vector_federation_timeout_ms = 30000;
```

对于 SPARQL 联合（SERVICE 关键字），pg_ripple 对整体查询使用 PostgreSQL 的 `statement_timeout`：

```sql
SET statement_timeout = '60s';
```

### 健康检查

使用 `/health` 端点进行负载均衡器配置：

```bash
curl http://localhost:8080/health
# 返回：{"status": "ok", "pool_size": 10, "pool_available": 8}
```

---

## 后续步骤

- **[§2.3 使用 SPARQL 查询](../features/querying-with-sparql.md)** — 通过 API 发送的查询的 SPARQL 查询参考。
- **[§2.7 AI 检索和 GraphRAG](../features/graphrag.md)** — RAG 端点详细信息和 LLM 集成。
- **[§2.6 导出和共享](../features/exporting-and-sharing.md)** — HTTP 端点返回的导出格式。