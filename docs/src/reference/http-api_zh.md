# HTTP API 参考

`pg_ripple_http` 是一个独立的 Rust 二进制文件，通过 HTTP 公开完整的 pg_ripple 功能集。它支持 W3C SPARQL 1.1 协议、通过 Server-Sent Events 的实时订阅流、Datalog REST 操作、PageRank、概率推理、Arrow Flight 批量导出和管理端点。

启动服务：

```bash
PG_RIPPLE_HTTP_PG_URL=postgresql://localhost/mydb \
PG_RIPPLE_HTTP_PORT=7878 \
PG_RIPPLE_HTTP_AUTH_TOKEN=mysecret \
pg_ripple_http
```

---

## 身份验证

所有受保护的端点都需要 Bearer 令牌。读取、写入、管理和指标令牌可以使用相应的环境变量分离；当专用令牌未设置时，该类使用主令牌。

```
Authorization: Bearer <token>
```

运行状况端点是公开的。除非配置了 `PG_RIPPLE_HTTP_METRICS_TOKEN`，否则指标是公开的。所有其他端点都受其访问类保护。

---

## 配置

| 环境变量 | 默认值 | 描述 |
|---|---|---|
| `PG_RIPPLE_HTTP_PG_URL` | `postgresql://localhost/postgres` | PostgreSQL 连接 URL |
| `PG_RIPPLE_HTTP_MODE` | `production` | `development` 或 `production` |
| `PG_RIPPLE_HTTP_BIND` | `127.0.0.1:7878` | TCP 监听地址 |
| `PG_RIPPLE_HTTP_PORT` | `7878` | 在 bind 未设置时使用的 TCP 监听端口 |
| `PG_RIPPLE_HTTP_POOL_SIZE` | `16` | PostgreSQL 连接池大小 |
| `PG_RIPPLE_HTTP_AUTH_TOKEN[_FILE]` | *(无)* | 读取 bearer 令牌 |
| `PG_RIPPLE_HTTP_WRITE_TOKEN[_FILE]` | *(回退到 auth token)* | 写入 bearer 令牌 |
| `PG_RIPPLE_HTTP_ADMIN_TOKEN[_FILE]` | *(回退到 auth token)* | 管理 bearer 令牌 |
| `PG_RIPPLE_HTTP_METRICS_TOKEN[_FILE]` | *(无)* | 可选的指标 bearer 令牌 |
| `PG_RIPPLE_HTTP_ALLOW_UNAUTHENTICATED` | `0` | 仅用于开发的显式身份验证绕过 |
| `PG_RIPPLE_HTTP_PG_PASSWORD[_FILE]` | *(未设置)* | PostgreSQL 密码或装载的密钥文件 |
| `PG_RIPPLE_HTTP_PG_SSLMODE` | `disable` | `disable`、`require`、`verify-ca` 或 `verify-full` |
| `PG_RIPPLE_HTTP_PG_CA_FILE` | *(未设置)* | `verify-ca` 和 `verify-full` 所需的 CA 文件 |
| `PG_RIPPLE_HTTP_RATE_LIMIT` | `100` | 每个 IP 的速率限制（req/s；0 = 无限制） |
| `PG_RIPPLE_HTTP_CORS_ORIGINS` | `''` | 逗号分隔的允许的 CORS 源；`*` 启用宽容 CORS |
| `PG_RIPPLE_HTTP_MAX_BODY_BYTES` | `10485760` | 最大请求主体大小（10 MiB） |
| `PG_RIPPLE_HTTP_SKIP_COMPAT_CHECK` | *(未设置)* | 设置为 `1` 以跳过扩展版本检查 |
| `PG_RIPPLE_HTTP_STRICT_COMPAT` | *(未设置)* | 设置为 `1` 以在扩展版本不兼容时启动失败 |
| `PG_RIPPLE_HTTP_METRICS_TOKEN` | *(无)* | `/metrics` 的可选 bearer 令牌 |
| `PG_RIPPLE_HTTP_AUTH_REALM` | `pg_ripple` | 用于 401 响应的 `WWW-Authenticate` 域 |
| `PG_RIPPLE_HTTP_REPLICA_DSN` | *(无)* | 可选的只读副本 PostgreSQL DSN |
| `PG_RIPPLE_HTTP_TRUST_PROXY` | *(无)* | 用于转发标头的受信任上游代理 IP/CIDR 列表 |
| `PG_RIPPLE_HTTP_CA_BUNDLE` | *(系统根)* | 用于出站 HTTPS 客户端的额外 CA 包 |
| `PG_RIPPLE_HTTP_PIN_FINGERPRINTS` | *(无)* | 可选的固定 TLS 证书指纹 |
| `PG_RIPPLE_HTTP_SHUTDOWN_TIMEOUT_SECS` | `30` | 优雅关闭超时 |
| `PG_RIPPLE_HTTP_QUERY_TIMEOUT_MS` | `300000` | 默认流式查询截止时间 |
| `PG_RIPPLE_HTTP_QUERY_TIMEOUT_MAX_MS` | `900000` | 最大流式查询截止时间 |
| `PG_RIPPLE_HTTP_STREAM_IDLE_TIMEOUT_MS` | `60000` | 流式行之间的最大等待时间 |
| `PG_RIPPLE_HTTP_STREAM_CHUNK_BYTES` | `65536` | 最大合并响应块 |
| `PG_RIPPLE_HTTP_STREAM_MAX_ROW_BYTES` | `1048576` | 最大编码行大小 |
| `ARROW_FLIGHT_SECRET` | *(无)* | Arrow Flight 票据的 HMAC 密钥 |
| `ARROW_UNSIGNED_TICKETS_ALLOWED` | `false` | 允许用于本地开发的未签名 Arrow 票据 |
| `ARROW_NONCE_CACHE_MAX` | `10000` | 重放保护 nonce 缓存大小 |

---

## 完整端点参考

`Read` 和 `Write` 表示当配置了 `PG_RIPPLE_HTTP_AUTH_TOKEN` 时，端点调用 HTTP 配套服务的读取或写入身份验证检查。`None` 表示该端点有意未经过身份验证。

| 方法 | 路径 | 身份验证 | 描述 |
|---|---|---|---|
| `GET` | `/sparql` | Read | 通过 URL 参数的 SPARQL 1.1 查询 |
| `POST` | `/sparql` | Write | 通过请求主体的 SPARQL 1.1 查询/更新 |
| `POST` | `/sparql/bindings` | Read | 具有类型化初始绑定的 SPARQL 查询 |
| `POST` | `/sparql/stream` | Read | 显式流式 SPARQL 兼容性别名 |
| `POST` | `/rag` | Read | RAG 检索 / NL 到 SPARQL |
| `GET` | `/health` | None | 活跃度探测 |
| `GET` | `/ready` | None | Kubernetes 就绪性探测 |
| `GET` | `/health/ready` | None | 深度扩展就绪性探测 |
| `GET` | `/metrics` | None 或指标令牌 | Prometheus 指标 |
| `GET` | `/metrics/extension` | Metrics | 扩展内部 Prometheus 指标 |
| `GET` | `/void` | Read | VoID 数据集描述 |
| `GET` | `/service` | Read | SPARQL 服务描述 |
| `GET` | `/openapi.yaml` | Read | OpenAPI 3.1 规范 |
| `GET` | `/explorer` | Admin | 基于 Web 的 SPARQL 资源管理器 UI |
| `GET` | `/admin/bench-history` | Admin | 来自 `_pg_ripple.bench_history` 的最近基准测试历史 |
| `GET` | `/admin/diagnostic-snapshot` | Admin | 具有架构、GUC、版本和指标数据的诊断包 |
| `POST` | `/flight/do_get` | Read | Arrow Flight 批量导出 |
| `GET` | `/subscribe/{subscription_id}` | Read | 实时 SPARQL 订阅 SSE 流 |
| `GET` | `/datalog/rules` | Read | 列出 Datalog 规则集 |
| `POST/DELETE` | `/datalog/rules/{rule_set}` | Write | 加载或删除规则集 |
| `POST` | `/datalog/rules/{rule_set}/builtin` | Write | 为规则集加载内置规则 |
| `POST` | `/datalog/rules/{rule_set}/add` | Write | 将规则添加到规则集 |
| `PUT` | `/datalog/rules/{rule_set}/enable` | Write | 启用规则集 |
| `PUT` | `/datalog/rules/{rule_set}/disable` | Write | 禁用规则集 |
| `DELETE` | `/datalog/rules/{rule_set}/{rule_id}` | Write | 删除特定规则 |
| `POST` | `/datalog/infer/{rule_set}` | Write | 运行正向链接推导 |
| `POST` | `/datalog/infer/{rule_set}/stats` | Write | 运行推导并返回统计信息 |
| `POST` | `/datalog/infer/{rule_set}/agg` | Write | 使用聚合运行推导 |
| `POST` | `/datalog/infer/{rule_set}/wfs` | Write | 运行良好基础语义推导 |
| `POST` | `/datalog/infer/{rule_set}/demand` | Write | 目标导向需求推导 |
| `POST` | `/datalog/infer/{rule_set}/lattice` | Write | 基于 lattice 的推导 |
| `POST` | `/datalog/query/{rule_set}` | Read | 目标导向 Datalog 查询 |
| `GET` | `/datalog/constraints` | Read | 检查所有约束规则 |
| `GET` | `/datalog/constraints/{rule_set}` | Read | 检查一个规则集的约束 |
| `GET` | `/datalog/stats/cache` | Admin | 规则计划缓存统计信息 |
| `GET` | `/datalog/stats/tabling` | Admin | 列表化缓存统计信息 |
| `GET/POST` | `/datalog/lattices` | Read/Admin | 列出或创建 lattice 结构 |
| `GET/POST` | `/datalog/views` | Read/Admin | 列出或创建 Datalog 支持的视图 |
| `DELETE` | `/datalog/views/{name}` | Admin | 删除 Datalog 支持的视图 |
| `POST` | `/pagerank/run` | Write | 启动 PageRank 计算 |
| `GET` | `/pagerank/status` | Read | PageRank 计算状态 |
| `GET` | `/pagerank/results` | Read | 检索 PageRank 分数 |
| `GET` | `/pagerank/export` | Read | 导出 PageRank 分数 |
| `GET` | `/pagerank/explain/{node_iri}` | Read | 解释节点的 PageRank 分数 |
| `GET` | `/pagerank/queue-stats` | Read | PageRank 队列统计信息 |
| `POST` | `/pagerank/vacuum-dirty` | Write | 清理陈旧的 PageRank 行 |
| `POST` | `/centrality/run` | Write | 计算中心性指标 |
| `GET` | `/centrality/results` | Read | 检索中心性结果 |
| `POST` | `/pagerank/find-duplicates` | Write | 查找近似重复的节点 |
| `POST` | `/confidence/load` | Write | 加载具有置信度分数的三元组 |
| `GET` | `/confidence/shacl-score` | Read | 获取 SHACL 软验证分数 |
| `GET` | `/confidence/shacl-report` | Read | 获取评分的 SHACL 验证报告 |
| `POST` | `/confidence/vacuum` | Write | 清理陈旧的置信度分行 |
| `POST` | `/confidence/update` | Write | 运行贝叶斯置信度更新 |
| `POST` | `/confidence/bulk-update` | Write | 批量置信度更新 |
| `POST` | `/explain` | Read | 查询或结果的自然语言解释 |
| `GET` | `/explain` | Read | 解释端点元数据/查询表单 |
| `POST` | `/hypothetical` | Read | 针对假设事实的 what-if 推理 |
| `GET` | `/rule-conflicts/{ruleset}` | Read | 检测规则集中的规则冲突 |
| `GET` | `/rule-libraries` | Read | 列出规则库 |
| `GET` | `/rule-libraries/{name}/stream` | Read | 流式传输已发布的规则库 |
| `POST` | `/rule-libraries/{name}/subscribe` | Write | 订阅远程规则库流 |
| `POST` | `/rules/draft` | Read | 从自然语言指导起草规则 |
| `POST` | `/rules/validate` | Read | 验证起草的规则 |
| `GET` | `/rules/{id}/explain` | Read | 按 ID 解释规则 |
| `GET/POST` | `/temporal/mark` | Read/Write | 列出或标记时间谓词 |
| `POST` | `/temporal/point_in_time` | Write | 设置时间点时间上下文 |
| `GET` | `/temporal/facts` | Read | 列出时间事实 |
| `GET` | `/temporal/graphs/{iri}/snapshot` | Read | 具体化时间点图快照 |
| `GET` | `/temporal/graphs/{iri}/diff` | Read | 在两个时间戳之间区分命名图 |
| `POST` | `/pprl/bloom_encode` | Write | 隐私保护 Bloom 编码 |
| `POST` | `/pprl/dice_similarity` | Write | 编码值的 Dice 相似度 |
| `POST` | `/dp/noisy_count` | Write | 差分隐私噪声计数 |
| `POST` | `/dp/noisy_histogram` | Write | 差分隐私噪声直方图 |
| `GET` | `/dp/budget/{dataset}/{principal}` | Read | 隐私预算状态 |
| `POST` | `/entity-resolution/resolve` | Write | 运行实体解析 |
| `POST` | `/entity-resolution/evaluate` | Write | 评估实体解析输出 |
| `POST` | `/entity-resolution/monitoring/enable` | Write | 启用实体解析监控 |
| `POST` | `/entity-resolution/monitoring/disable` | Write | 禁用实体解析监控 |
| `GET` | `/proof-tree/{subject}/{predicate}/{object}` | Read | 解释三元组的派生/证明树 |
| `GET/POST` | `/tenants` | Read/Write | 列出或创建租户 |
| `GET/DELETE` | `/tenants/{name}` | Read/Write | 获取或删除租户 |
| `GET/POST` | `/tenants/{name}/quota` | Read/Write | 获取或更新租户配额 |
| `GET` | `/federation/{endpoint}/auth-status` | Write | 每个端点的联邦凭据状态 |
| `POST` | `/json-mapping/{name}/writeback` | Write | 同步 JSON 映射关系回写 |
| `GET` | `/json-mapping/{name}/writeback/status` | Read | JSON 映射回写队列状态 |
| `GET` | `/json-mapping/{name}/writeback/config` | Read | 已验证的 JSON 映射回写配置 |

---

## SPARQL 端点

### `GET /sparql`

W3C SPARQL 1.1 协议查询端点。

**查询参数：**

| 参数 | 必需 | 描述 |
|---|---|---|
| `query` | 是（查询） | URL 编码的 SPARQL SELECT/CONSTRUCT/ASK/DESCRIBE 查询 |
| `update` | 是（更新） | URL 编码的 SPARQL UPDATE |
| `default-graph-uri` | 否 | 默认图 URI |
| `named-graph-uri` | 否 | 命名图 URI（可重复） |

**Accept 标头** 控制结果格式：

| Accept | 格式 |
|---|---|
| `application/sparql-results+json` | SPARQL JSON（默认） |
| `application/sparql-results+xml` | SPARQL XML |
| `text/turtle` | Turtle（用于 CONSTRUCT/DESCRIBE） |
| `application/n-triples` | N-Triples（用于 CONSTRUCT/DESCRIBE） |
| `application/ld+json` | JSON-LD（用于 CONSTRUCT/DESCRIBE） |

**示例：**

```bash
curl -G "http://localhost:7878/sparql" \
  --data-urlencode 'query=SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10' \
  -H "Accept: application/sparql-results+json"
```

---

### `POST /sparql`

接受作为 `application/x-www-form-urlencoded` 主体（表单发布）的查询或作为原始 `application/sparql-query` / `application/sparql-update` 主体的查询。

**表单主体参数：** 与 `GET /sparql` 查询参数相同。

**示例（SELECT）：**

```bash
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-query" \
  -d 'SELECT ?label WHERE { <https://example.org/Alice> rdfs:label ?label }'
```

**示例（UPDATE，需要身份验证）：**

```bash
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-update" \
  -H "Authorization: Bearer $TOKEN" \
  -d 'INSERT DATA { <:Alice> rdfs:label "Alice" }'
```

---

### `POST /sparql/bindings`

使用单行类型化初始绑定执行只读 SELECT、ASK、CONSTRUCT 或 DESCRIBE 查询。请求主体为 JSON；绑定值使用 W3C SPARQL Results JSON 术语形状并支持 `uri` 和 `literal` 术语。

```json
{
  "query": "SELECT ?s WHERE { ?s <https://example.org/email> ?email }",
  "bindings": {
    "email": {"type": "literal", "value": "alice@example.test"}
  },
  "prefix_mode": "strict",
  "timeout_ms": 1000
}
```

使用 `prefix_mode: "registered"` 从托管注册表解析未声明的前缀。查询本地声明优先于注册表。绑定负载在流式传输开始之前进行验证；SPARQL Update、空白节点、RDF-star 和多行绑定被拒绝。

响应使用与 `POST /sparql/stream` 相同的内容协商和流式传输格式。

---

### `POST /sparql/stream`

显式流式传输兼容性别名。结果使用自然的 HTTP 反压从 PostgreSQL 行流传送。

**请求：** 与 `POST /sparql` 相同。

**响应 Content-Type：** `application/sparql-results+json`、`text/csv`、`text/tab-separated-values` 或 `application/n-triples`，由 `Accept` 选择。JSON 是完整的 SPARQL Results JSON 文档，而不是 JSON-Lines 或 SSE。XML、Turtle 和 JSON-LD 在 `/sparql` 上保持缓冲。

**示例：**

```bash
curl -X POST http://localhost:7878/sparql/stream \
  -H "Content-Type: application/sparql-query" \
  -H "Accept: application/sparql-results+json" \
  -d 'SELECT ?s ?p ?o WHERE { ?s ?p ?o }' \
  --no-buffer
```

---

## RAG 端点

### `POST /rag`

执行混合 SPARQL + 向量相似度 RAG 检索查询。该端点嵌入问题，通过余弦相似度查找最近的 RDF 实体，并返回具有适合用作 LLM 提示的纯文本上下文字符串的结构化结果。

**Content-Type：** `application/json`

**请求主体：**

```json
{
  "question": "what treats headaches?",
  "sparql_filter": "?entity a <https://pharma.example/Drug>",
  "k": 5,
  "model": "text-embedding-3-small",
  "output_format": "jsonb"
}
```

| 字段 | 类型 | 必需 | 默认值 | 描述 |
|---|---|---|---|---|
| `question` | string | **是** | — | 自然语言问题 |
| `sparql_filter` | string | 否 | `null` | 用于过滤候选项的 SPARQL WHERE 片段 |
| `k` | integer | 否 | `5` | 最近邻数 |
| `model` | string | 否 | *(GUC)* | 覆盖 `pg_ripple.embedding_model` |
| `output_format` | string | 否 | `"jsonb"` | `"jsonb"` 或 `"jsonld"` |

**响应（200 OK）：**

```json
{
  "results": [
    {
      "entity_iri": "https://pharma.example/aspirin",
      "label": "aspirin",
      "context_json": {
        "label": "aspirin",
        "types": ["Drug", "NSAID"],
        "properties": [{"predicate": "treats", "object": "headache"}],
        "contextText": "aspirin. Type: NSAID, Drug."
      },
      "distance": 0.12
    }
  ],
  "context": "aspirin. Type: NSAID, Drug.\n\nibuprofen. Type: Drug."
}
```

`context` 字段是一个串联的纯文本摘要，可用作 LLM 系统提示。

---

## JSON 映射回写

### `POST /json-mapping/{name}/writeback`

将一个 RDF 主语同步写回到为命名 JSON 映射配置的关系表。这会调用 `pg_ripple.writeback_json_row(name, subject_iri)`，并在启用身份验证时需要写入身份验证。

**Content-Type：** `application/json`

```json
{
  "subject_iri": "https://example.com/contacts/c001"
}
```

**响应（200 OK）：**

```json
{
  "rows_affected": 1
}
```

**错误映射：**

| 状态 | 错误 | 原因 |
|---|---|---|
| `422` | `writeback_target_not_configured` | 映射没有回写表或键列（`PT0550`） |
| `409` | `writeback_conflict` | 冲突策略是 `error` 并且存在冲突行（`PT0551`） |
| `422` | `writeback_key_column_missing` | 配置的键列谓词对于此主语没有值（`PT0552`） |

### `GET /json-mapping/{name}/writeback/status`

返回一个映射的队列深度、错误计数和最后处理时间戳。这是 `pg_ripple.json_writeback_status()` 的过滤 HTTP 视图，并在启用身份验证时需要读取身份验证。

**响应（200 OK）：**

```json
{
  "mapping_name": "contacts",
  "pending": 0,
  "errors": 0,
  "last_error": null,
  "last_processed_at": null
}
```

### `GET /json-mapping/{name}/writeback/config`

返回一个映射的已验证关系目标和回写健康状况。响应镜像 `pg_ripple.writeback_inspect()` 并需要读取身份验证。

**响应（200 OK）：**

```json
{
  "target_schema": "public",
  "target_table": "contacts",
  "key_columns": ["contact_id"],
  "conflict_policy": "replace",
  "writeback_enabled": false,
  "trigger_count": 0,
  "queue_depth": 0
}
```

---

## Arrow Flight 批量导出

### `POST /flight/do_get`

将 SPARQL 查询结果作为 [Apache Arrow IPC](https://arrow.apache.org/docs/format/Columnar.html) 记录流批量导出。适用于高吞吐量提取管道（ DuckDB、pandas、Spark）。

**Content-Type：** `application/json`

**请求主体：**

```json
{
  "ticket": "eyJhbGciOiJIUzI1NiJ9...",
  "query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"
}
```

| 字段 | 类型 | 必需 | 描述 |
|---|---|---|---|
| `ticket` | string | **是** | 由 `pg_ripple.arrow_flight_ticket(query)` 发布的 HMAC 签名 JWT 票据 |
| `query` | string | 否 | 内联查询（仅当 `pg_ripple.arrow_unsigned_tickets_allowed = on` 时） |

**响应：** `application/vnd.apache.arrow.stream` — Arrow IPC 流。

**示例（Python）：**

```python
import requests, pyarrow as pa

# 1. 从 PostgreSQL 获取已签名的票据
ticket = pg.execute("SELECT pg_ripple.arrow_flight_ticket('SELECT ?s ?p ?o WHERE { ?s ?p ?o }')")

# 2. 流式传输 Arrow IPC 导出
r = requests.post("http://localhost:7878/flight/do_get",
    json={"ticket": ticket},
    headers={"Authorization": f"Bearer {token}"},
    stream=True)

reader = pa.ipc.open_stream(r.raw)
table = reader.read_all()
print(table.to_pandas())
```

---

## 流式订阅

### `GET /subscribe/{subscription_id}`

通过 Server-Sent Events 订阅实时 SPARQL 结果流。`subscription_id` 由 `pg_ripple.subscribe(query)` 返回。

**响应 Content-Type：** `text/event-stream`

**事件类型：**

| 事件 | 负载 | 描述 |
|---|---|---|
| `row` | JSON 对象 | 新结果行 |
| `retract` | JSON 对象 | 撤回的行（三元组已删除） |
| `heartbeat` | `{}` | 每 30 秒保持活动状态 |
| `error` | `{"message": "..."}` | 订阅错误 |

**示例：**

```bash
# 在 PostgreSQL 中创建订阅
psql -c "SELECT pg_ripple.subscribe('SELECT ?s ?p ?o WHERE { ?s ?p ?o }')"
-- 返回：sub_abc123

# 连接 SSE 流
curl -N "http://localhost:7878/subscribe/sub_abc123" \
  -H "Accept: text/event-stream"
```

---

## Datalog REST API

### `GET /datalog/rules`

列出所有已注册的 Datalog 规则集。

**响应（200 OK）：**

```json
[
  {"rule_set": "rdfs_closure", "rule_count": 13, "enabled": true},
  {"rule_set": "custom_rules", "rule_count": 5, "enabled": true}
]
```

---

### `POST /datalog/rules/{rule_set}/add`

将 Datalog 规则添加到规则集。

**Content-Type：** `application/json`

```json
{
  "rule": "ancestor(?x, ?z) :- ancestor(?x, ?y), parent(?y, ?z).",
  "enabled": true
}
```

**响应（200 OK）：**

```json
{"rule_id": "rule_42", "rule_set": "custom_rules"}
```

---

### `POST /datalog/infer/{rule_set}`

为规则集运行正向链接推导。

**Content-Type：** `application/json`

```json
{
  "graph": "https://example.org/graph1",
  "max_iterations": 100,
  "dry_run": false
}
```

**响应（200 OK）：**

```json
{
  "derived_count": 847,
  "iterations": 3,
  "elapsed_ms": 42,
  "stratification_order": ["base_rules", "closure_rules"]
}
```

---

### `POST /datalog/infer/{rule_set}/wfs`

运行良好基础语义推导。对于带有否定的递归规则，返回三值结果（true / false / undefined）。

---

### `POST /datalog/infer/{rule_set}/demand`

目标导向的 magic-sets 推导。仅派生与特定查询目标相关的事实。

**Content-Type：** `application/json`

```json
{
  "goal": "ancestor(<:Alice>, ?z)",
  "graph": "https://example.org/graph1"
}
```

---

### `POST /datalog/query/{rule_set}`

在不对存储进行修改的情况下对物化规则集运行单个 Datalog 目标查询。

**Content-Type：** `application/json`

```json
{
  "goal": "ancestor(<:Alice>, ?z)",
  "limit": 100
}
```

**响应（200 OK）：**

```json
{
  "results": [
    {"?z": "<https://example.org/Bob>"},
    {"?z": "<https://example.org/Carol>"}
  ],
  "total": 2
}
```

---

### `GET /datalog/constraints`

检查所有规则集中的所有约束规则。

**响应（200 OK）：**

```json
{
  "violations": [
    {
      "rule_set": "integrity_rules",
      "constraint": "uniqueness_violation",
      "violating_bindings": [{"?x": "<:entity1>"}]
    }
  ],
  "total_violations": 1
}
```

---

### `GET /datalog/stats/cache`

返回规则计划缓存命中/未命中统计信息。

---

### `GET /datalog/stats/tabling`

返回列表化缓存占用和驱逐统计信息。

---

## PageRank API

### `POST /pagerank/run`

启动 PageRank 计算作业。

**Content-Type：** `application/json`

```json
{
  "graph": "https://example.org/graph1",
  "edge_predicates": ["schema:knows", "schema:worksFor"],
  "damping": 0.85,
  "max_iterations": 100,
  "convergence_delta": 0.0001
}
```

**响应（200 OK）：**

```json
{
  "job_id": "pr_20250503_001",
  "status": "running",
  "started_at": "2025-05-03T12:00:00Z"
}
```

---

### `GET /pagerank/status`

轮询最近 PageRank 作业的状态。

**响应：**

```json
{
  "job_id": "pr_20250503_001",
  "status": "completed",
  "iterations": 47,
  "elapsed_ms": 1204,
  "node_count": 50000,
  "edge_count": 250000
}
```

---

### `GET /pagerank/export`

将 PageRank 分数导出为 JSON 或 CSV。

**查询参数：**

| 参数 | 默认值 | 描述 |
|---|---|---|
| `format` | `json` | `json` 或 `csv` |
| `limit` | `1000` | 最大节点数 |
| `offset` | `0` | 分页偏移 |
| `graph` | *(所有)* | 按命名图过滤 |

**响应（200 OK，JSON）：**

```json
{
  "scores": [
    {"iri": "https://example.org/Alice", "score": 0.0423, "rank": 1},
    {"iri": "https://example.org/Bob", "score": 0.0381, "rank": 2}
  ],
  "total": 50000
}
```

---

### `GET /pagerank/explain/{node_iri}`

解释特定节点的 PageRank 分数，显示贡献最大的入边。

**路径参数：** `node_iri` — URL 编码的 IRI。

**响应：**

```json
{
  "iri": "https://example.org/Alice",
  "score": 0.0423,
  "rank": 1,
  "top_contributors": [
    {"from_iri": "https://example.org/Bob", "edge_weight": 0.012},
    {"from_ri": "https://example.org/Carol", "edge_weight": 0.009}
  ]
}
```

---

### `POST /centrality/run`

计算中心性指标（度、中介性、Katz）。

**Content-Type：** `application/json`

```json
{
  "graph": "https://example.org/graph1",
  "metrics": ["degree", "katz"],
  "katz_alpha": 0.01
}
```

---

### `POST /pagerank/find-duplicates`

通过 PageRank 分数相似度和图结构查找近似重复的节点。

---

## 置信度 / 概率 API

### `POST /confidence/load`

加载具有关联置信度分数的三元组。

**Content-Type：** `application/json`

```json
{
  "triples": [
    {
      "subject": "https://example.org/Alice",
      "predicate": "rdf:type",
      "object": "schema:Person",
      "confidence": 0.95,
      "graph": "https://example.org/g1"
    }
  ]
}
```

**响应（200 OK）：**

```json
{"inserted": 1, "updated": 0}
```

---

### `GET /confidence/shacl-score`

获取形状的 SHACL 软验证置信度分数。

**查询参数：**

| 参数 | 默认值 | 描述 |
|---|---|---|
| `shape_iri` | *(所有)* | 按特定形状过滤 |
| `min_score` | `0.0` | 最低分数阈值 |

---

### `GET /confidence/shacl-report`

获取每个约束的具有置信度分数的完整 SHACL 验证报告。

---

### `POST /confidence/vacuum`

从 `_pg_ripple.confidence` 中清理陈旧的置信度分行。

---

## 管理端点

### `GET /health`

当服务正在运行且数据库连接池正常时返回 `{"status": "ok"}`。

**响应：** `200 OK` — `{"status": "ok"}`
**响应：** `503 Service Unavailable` — 数据库不可访问。

---

### `GET /health/ready`

Kubernetes 兼容的就绪性探测。检查 PostgreSQL 连接池是否至少有一个可用连接。

---

### `GET /metrics`

Prometheus 格式的指标。无需身份验证。

**关键指标：**

| 指标 | 类型 | 描述 |
|---|---|---|
| `pg_ripple_http_requests_total` | 计数器 | 按方法、路径、状态划分的 HTTP 请求总数 |
| `pg_ripple_http_request_duration_seconds` | 直方图 | 请求延迟 |
| `pg_ripple_sparql_queries_total` | 计数器 | 按类型划分的 SPARQL 查询 |
| `pg_ripple_sparql_query_duration_seconds` | 直方图 | SPARQL 查询延迟 |
| `pg_ripple_federation_calls_total` | 计数器 | 按端点划分的 SERVICE 子句调用 |
| `pg_ripple_datalog_infer_duration_seconds` | 直方图 | 推导运行延迟 |

### `GET /metrics/extension`

直接从 PostgreSQL 中的 pg_ripple 扩展源获取的扩展内部 Prometheus 指标。

---

### `GET /void`

返回 VoID（Vocabulary of Interlinked Datasets）数据集描述，总结 RDF 存储内容：三元组计数、谓词频率和类分布。

**Accept：** `text/turtle` 或 `application/ld+json`

---

### `GET /service`

返回描述端点功能的 SPARQL 1.1 服务描述文档。

**Accept：** `text/turtle` 或 `application/ld+json`

---

### `GET /openapi.yaml`

返回 `pg_ripple_http` API 的完整 OpenAPI 3.1 规范。

---

### `GET /explorer`

基于 Web 的 SPARQL 资源管理器 UI — 一个嵌入式查询编辑器，具有语法突出显示、结果表和端点配置。

---

## 错误响应

所有错误响应都使用一致的 JSON 信封：

```json
{
  "error": "PT440",
  "message": "query exceeds maximum algebra depth (256)",
  "hint": "Simplify the query or raise pg_ripple.sparql_max_algebra_depth"
}
```

| HTTP 状态 | 含义 |
|---|---|
| 400 | 无效的请求主体或 SPARQL 语法错误 |
| 401 | Bearer 令牌缺失或无效 |
| 403 | 此角色不允许执行此操作 |
| 404 | 未找到资源 |
| 422 | 语义错误（SHACL 违规、约束错误） |
| 429 | 超过速率限制 |
| 500 | 内部错误（检查 PostgreSQL 日志） |
| 503 | 数据库连接不可用 |

---

## 安全性

- Bearer 令牌身份验证控制变更端点
- SSRF 保护：联邦 SERVICE 端点针对 `federation_endpoint_policy` 进行检查
- SQL 注入预防：所有查询都使用参数化的 PostgreSQL SPI
- Arrow Flight 票据是 HMAC-SHA256 签名的；设置 `pg_ripple.arrow_flight_secret`
- 速率限制：按 IP，可通过 `PG_RIPPLE_HTTP_RATE_LIMIT` 配置
- HTTPS 终止应由反向代理处理（nginx、Caddy、Traefik）
- 切勿在 URL 或日志中公开 `PG_RIPPLE_HTTP_AUTH_TOKEN`