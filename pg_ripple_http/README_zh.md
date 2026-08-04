# pg_ripple_http

一个独立的 HTTP 服务，为 [pg_ripple](../README.md) 暴露 [W3C SPARQL 1.1 协议](https://www.w3.org/TR/sparql11-protocol/)端点。任何标准的 SPARQL 客户端 —— YASGUI、SPARQLWrapper、Jena，甚至是最朴素的 `curl` —— 都无需 PostgreSQL 驱动即可查询 pg_ripple。

## 构建

```bash
cargo build --release -p pg_ripple_http
```

生成的二进制文件位于 `target/release/pg_ripple_http`。

**环境要求：** Rust 1.88+，以及一个已安装 `pg_ripple` 扩展的 PostgreSQL 18 实例。

## 运行

```bash
./target/release/pg_ripple_http
```

启动时，服务会连接 PostgreSQL，验证 pg_ripple 可用，并输出连接详情日志：

```
INFO pg_ripple_http: connected to postgresql://localhost/postgres (port 7878), triple store contains 12345 triples
INFO pg_ripple_http: pg_ripple_http listening on http://0.0.0.0:7878
```

## 配置

所有配置均通过环境变量完成：

| 变量 | 默认值 | 说明 |
|---|---|---|
| `PG_RIPPLE_HTTP_PG_URL` | `postgresql://localhost/postgres` | PostgreSQL 连接 URL |
| `PG_RIPPLE_HTTP_PORT` | `7878` | HTTP 监听端口 |
| `PG_RIPPLE_HTTP_POOL_SIZE` | `16` | 数据库连接池大小 |
| `PG_RIPPLE_HTTP_AUTH_TOKEN` | (未设置) | 若设置，则每个请求必须携带 `Authorization: Bearer *** |
| `PG_RIPPLE_HTTP_AUTH_REALM` | `pg_ripple` | `WWW-Authenticate` 响应头中 `Bearer realm=` 字段的值（L16-06，v0.117.0） |
| `PG_RIPPLE_HTTP_METRICS_TOKEN` | (未设置) | 若设置，则 `GET /metrics` 需要 `Authorization: Bearer *** (M16-22) |
| `PG_RIPPLE_HTTP_RATE_LIMIT` | `0` | 每个客户端 IP 每秒最大请求数（0 表示禁用） |
| `PG_RIPPLE_HTTP_CORS_ORIGINS` | `*` | 允许的来源，逗号分隔；或 `*` 表示全部允许 |

示例：

```bash
export PG_RIPPLE_HTTP_PG_URL="postgresql://user:***@db-host:5432/mydb"
export PG_RIPPLE_HTTP_PORT=8080
export PG_RIPPLE_HTTP_AUTH_TOKEN="my-secret-token"
./target/release/pg_ripple_http
```

## 端点

### `GET /health`（存活探针）

当进程存活 **且** 数据库可达时返回 `200 OK`。
用于 Kubernetes **存活探针**（livenessProbe）—— 非 200 响应意味着容器应当被重启。

```bash
curl http://localhost:7878/health
```

### `GET /ready`（就绪探针）

当服务器完成启动并成功从连接池获取到连接后返回 `200 OK`，表示已准备好对外提供服务。
用于 Kubernetes **就绪探针**（readinessProbe）—— 非 200 响应会将 Pod 从服务端点中移除，直到恢复为止。

```bash
curl http://localhost:7878/ready
```

> **M16-09（v0.115.0）：** Kubernetes `livenessProbe` 应指向 `/health`，
> 而 `readinessProbe` 应指向 `/ready`。从 v0.115.0 起，对两者都使用
> `exec: pg_isready` 的做法已被弃用。

### `GET /metrics`

兼容 Prometheus 的指标端点。

```bash
curl http://localhost:7878/metrics
# 可选 Bearer Token 保护（M16-22，v0.115.0）：
curl -H "Authorization: Bearer $PG_RI...OKEN" http://localhost:7878/metrics
```

> **安全（M16-22，v0.115.0）：** 设置 `PG_RIPPLE_HTTP_METRICS_TOKEN` 后，
> `/metrics` 端点会要求携带 Bearer Token。未携带有效 Token 的请求将收到
> `401 Unauthorized` 以及 `WWW-Authenticate: Bearer` 质询。
> 即使不设置该 Token，也应通过反向代理的 ACL 限制指标端口仅允许
> Prometheus 抓取器的 IP 访问。
>
> 详见 [Security → Metrics Port Isolation](../docs/src/operations/security.md)。

### `GET /sparql?query=…`

通过 URL 参数执行 SPARQL 查询。

```bash
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
```

### `POST /sparql`

通过请求体执行 SPARQL 查询或更新。

| Content-Type | 请求体 |
|---|---|
| `application/sparql-query` | 原始 SPARQL SELECT/ASK/CONSTRUCT/DESCRIBE |
| `application/sparql-update` | 原始 SPARQL INSERT/DELETE |
| `application/x-www-form-urlencoded` | `query=…` 或 `update=…` |

```bash
# SELECT
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-query" \
  -d "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"

# Update
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-update" \
  -d 'INSERT DATA { <http://example.org/alice> <http://example.org/name> "Alice" }'
```

## 内容协商

设置 `Accept` 头来控制响应格式：

| Accept | 格式 |
|---|---|
| `application/sparql-results+json` *(SELECT/ASK 的默认格式)* | SPARQL Results JSON |
| `application/sparql-results+xml` | SPARQL Results XML |
| `text/csv` | CSV |
| `text/tab-separated-values` | TSV |
| `text/turtle` *(CONSTRUCT/DESCRIBE 的默认格式)* | Turtle |
| `application/n-triples` | N-Triples |
| `application/ld+json` | JSON-LD |

```bash
curl -G http://localhost:7878/sparql \
  -H "Accept: text/csv" \
  --data-urlencode "query=SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5"
```

## 身份认证

如果设置了 `PG_RIPPLE_HTTP_AUTH_TOKEN`，每个请求都必须携带该 Token：

```bash
curl -G http://localhost:7878/sparql \
  -H "Authorization: Bearer ***" \
  --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 5"
```

同时支持 `Authorization: Bearer ***` 与 `Authorization: Basic ***` 两种形式。

## Docker Compose

根目录下的 `docker-compose.yml` 会同时启动 PostgreSQL 与 `pg_ripple_http`：

```bash
docker compose up
```

服务列表：

| 服务 | 端口 | 说明 |
|---|---|---|
| `postgres` | 5432 | PostgreSQL 18 + pg_ripple |
| `sparql` | 7878 | SPARQL HTTP 端点 |

---

## Datalog API

自 v0.39.0 起，`pg_ripple_http` 同样暴露一个 `/datalog` REST 命名空间，
允许任意 HTTP 客户端管理 Datalog 规则集、触发推理、运行目标导向的查询、
检查完整性约束以及查看监控统计 —— 无需 PostgreSQL 驱动。

所有 Datalog 端点接收与返回 `application/json`（除非另有说明）。规则文本使用 `text/x-datalog` 或 `text/plain`。

### 身份认证

同一个 `PG_RIPPLE_HTTP_AUTH_TOKEN` Bearer Token 同时覆盖所有 `/datalog/*` 端点。
此外，也可以设置 `PG_RIPPLE_HTTP_DATALOG_WRITE_TOKEN`，以便将变更类端点
（`POST`、`PUT`、`DELETE`）限定为单独一个 Token，而读取类端点
（推理触发、监控、GET）仍由主 Token 覆盖。

### Phase 1 — 规则管理

#### `POST /datalog/rules/{rule_set}`

从 Datalog 文本加载规则。请求体：`text/x-datalog`。

```bash
curl -X POST http://localhost:7878/datalog/rules/my-ontology \
  -H "Content-Type: text/x-datalog" \
  -d 'ancestor(?x, ?y) :- parent(?x, ?y).
ancestor(?x, ?z) :- parent(?x, ?y), ancestor(?y, ?z).'
# → {"rule_set": "my-ontology", "rules_loaded": 2}
```

#### `POST /datalog/rules/{rule_set}/builtin`

加载内置规则集（`rdfs`、`owl-rl`）。

```bash
curl -X POST http://localhost:7878/datalog/rules/rdfs/builtin
# → {"rule_set": "rdfs", "rules_loaded": 13}
```

#### `GET /datalog/rules`

列出所有规则集及其规则。

```bash
curl http://localhost:7878/datalog/rules
# → [{"id": 1, "rule_set": "my-ontology", "rule_text": "…", "active": true}, …]
```

#### `DELETE /datalog/rules/{rule_set}`

删除某个规则集中的全部规则。

```bash
curl -X DELETE http://localhost:7878/datalog/rules/my-ontology
# → {"deleted": 2}
```

#### `POST /datalog/rules/{rule_set}/add`

向已有规则集追加单条规则。请求体：`text/x-datalog`。

```bash
curl -X POST http://localhost:7878/datalog/rules/my-ontology/add \
  -H "Content-Type: text/x-datalog" \
  -d 'sibling(?x, ?y) :- parent(?p, ?x), parent(?p, ?y).'
# → {"rule_set": "my-ontology", "rule_id": 3}
```

#### `DELETE /datalog/rules/{rule_set}/{rule_id}`

按 ID 移除单条规则（会触发 DRed 回溯）。

```bash
curl -X DELETE http://localhost:7878/datalog/rules/my-ontology/3
# → {"removed": 1}
```

#### `PUT /datalog/rules/{rule_set}/enable`

激活某个规则集。

```bash
curl -X PUT http://localhost:7878/datalog/rules/my-ontology/enable
# → {"rule_set": "my-ontology", "enabled": true}
```

#### `PUT /datalog/rules/{rule_set}/disable`

停用某个规则集。

```bash
curl -X PUT http://localhost:7878/datalog/rules/my-ontology/disable
# → {"rule_set": "my-ontology", "enabled": false}
```

### Phase 2 — 推理

#### `POST /datalog/infer/{rule_set}`

物化导出三元组（半朴素求值）。

```bash
curl -X POST http://localhost:7878/datalog/infer/my-ontology
# → {"derived": 42}
```

#### `POST /datalog/infer/{rule_set}/stats`

执行推理，并返回每个分层的详细统计信息。

```bash
curl -X POST http://localhost:7878/datalog/infer/my-ontology/stats
# → {"derived": 42, "iterations": 3, "eliminated_rules": 0, "parallel_groups": 2, …}
```

#### `POST /datalog/infer/{rule_set}/agg`

支持聚合的推理（Datalog^agg）。

```bash
curl -X POST http://localhost:7878/datalog/infer/my-ontology/agg
# → {"derived": 12}
```

#### `POST /datalog/infer/{rule_set}/wfs`

基于 Well-Founded Semantics（三值）的推理。

```bash
curl -X POST http://localhost:7878/datalog/infer/my-ontology/wfs
# → {"derived": 8}
```

#### `POST /datalog/infer/{rule_set}/demand`

基于需求变换（目标导向）的推理。请求体：JSON。

```bash
curl -X POST http://localhost:7878/datalog/infer/my-ontology/demand \
  -H "Content-Type: application/json" \
  -d '{"demands": [{"predicate": "ancestor", "bound": [0]}]}'
# → {"derived": 12, "iterations": 2, "demand_predicates": ["ancestor_bf"]}
```

#### `POST /datalog/infer/{rule_set}/lattice`

基于格（Lattice）的推理（Datalog^L）。请求体：JSON。

```bash
curl -X POST http://localhost:7878/datalog/infer/my-ontology/lattice \
  -H "Content-Type: application/json" \
  -d '{"lattice": "min"}'
# → {"derived": 5}
```

### Phase 3 — 查询与约束

#### `POST /datalog/query/{rule_set}`

通过 Magic Sets 进行目标导向查询。请求体：Datalog 目标文本。

```bash
curl -X POST http://localhost:7878/datalog/query/my-ontology \
  -H "Content-Type: text/x-datalog" \
  -d 'ancestor(ex:alice, ?y).'
# → {"derived": 5, "iterations": 2, "matching": [{"y": "http://example.org/bob"}, …]}
```

#### `GET /datalog/constraints`

检查所有约束规则，返回违规情况。

```bash
curl http://localhost:7878/datalog/constraints
# → [{"rule": "no_self_parent", "violated": false}, …]
```

#### `GET /datalog/constraints/{rule_set}`

检查指定规则集的约束。

```bash
curl http://localhost:7878/datalog/constraints/my-ontology
```

### Phase 4 — 管理与监控

#### `GET /datalog/stats/cache`

规则计划缓存统计。

```bash
curl http://localhost:7878/datalog/stats/cache
# → {"size": 12, "hits": 340, "misses": 8, …}
```

#### `GET /datalog/stats/tabling`

Tabling / 记忆化缓存统计。

```bash
curl http://localhost:7878/datalog/stats/tabling
# → {"entries": 100, "hit_rate": 0.82, …}
```

#### `GET /datalog/lattices`

列出已注册的格类型。

```bash
curl http://localhost:7878/datalog/lattices
# → [{"name": "min", "join_fn": "LEAST", "bottom": "Infinity"}, …]
```

#### `POST /datalog/lattices`

注册一个新的格类型。请求体：JSON。

```bash
curl -X POST http://localhost:7878/datalog/lattices \
  -H "Content-Type: application/json" \
  -d '{"name": "my_min", "join_fn": "my_schema.my_min", "bottom": "9999"}'
# → {"created": "my_min"}
```

#### `GET /datalog/views`

列出所有 Datalog 物化视图。

```bash
curl http://localhost:7878/datalog/views
# → [{"name": "ancestor_view", "goal": "ancestor(?x, ?y).", …}, …]
```

#### `POST /datalog/views`

创建一个 Datalog 物化视图。请求体：JSON。

```bash
curl -X POST http://localhost:7878/datalog/views \
  -H "Content-Type: application/json" \
  -d '{"name": "ancestor_view", "goal": "ancestor(?x, ?y).", "rule_set": "my-ontology"}'
# → {"created": "ancestor_view"}
```

#### `DELETE /datalog/views/{name}`

删除一个 Datalog 物化视图。

```bash
curl -X DELETE http://localhost:7878/datalog/views/ancestor_view
# → {"dropped": "ancestor_view"}
```

### 错误码

| HTTP 状态码 | `error` 字段 | 触发条件 |
|---|---|---|
| `400` | `datalog_parse_error` | Datalog 规则文本格式错误 |
| `400` | `datalog_goal_error` | 无效的目标模式 |
| `400` | `invalid_request` | 缺少请求体、Content-Type 错误、`rule_id` 非数值 |
| `404` | `rule_set_not_found` | 在不存在的规则集上执行推理 / 删除 |
| `401` | — | 缺少或无效的 `Authorization` Token |
| `503` | `service_unavailable` | 连接池已耗尽 |

所有错误响应都包含一个 `trace_id` 字段，便于日志关联。

## 读副本路由（v0.120.0）

`pg_ripple_http` 可以将只读 SPARQL 查询路由到 PostgreSQL 备机副本，
从而降低主库负载。

### 配置

将 `PG_RIPPLE_HTTP_REPLICA_DSN` 环境变量设置为指向读副本的 PostgreSQL DSN：

```bash
export PG_RIPPLE_HTTP_REPLICA_DSN="host=replica.db.example.com dbname=ripple user=app password=..."
```

当设置该变量后，会为该副本单独创建一个连接池。

### 使用

在任何 SPARQL SELECT、CONSTRUCT 或 ASK 请求上添加 `?replica=ok`，
即可将该请求路由到副本：

```bash
# GET
curl "http://localhost:7878/sparql?query=SELECT+%2A+WHERE+%7B+%3Fs+%3Fp+%3Fo+%7D+LIMIT+10&replica=ok"

# POST (application/sparql-query)
curl -X POST "http://localhost:7878/sparql?replica=ok" \
  -H "Content-Type: application/sparql-query" \
  -d "SELECT * WHERE { ?s ?p ?o } LIMIT 10"

```
