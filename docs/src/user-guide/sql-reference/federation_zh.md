# SPARQL 联邦

SPARQL 联邦允许单个查询将 pg_ripple 的数据与存储在外部 SPARQL 端点上的数据组合起来。使用 `SERVICE` 关键字将查询的一部分委托给远程端点。

## 快速入门

```sql
-- 1. 注册远程端点（SSRF 保护所需）
SELECT pg_ripple.register_endpoint('https://query.wikidata.org/sparql');

-- 2. 跨本地和远程数据进行查询
SELECT result->>'local_s' AS local_subject,
       result->>'remote_o' AS remote_label
FROM pg_ripple.sparql($$
  SELECT ?local_s ?remote_o WHERE {
    ?local_s <https://example.org/sameAs> ?wikidata_item .
    SERVICE <https://query.wikidata.org/sparql> {
      ?wikidata_item <http://www.w3.org/2000/01/rdf-schema#label> ?remote_o .
      FILTER(LANG(?remote_o) = "en")
    }
  }
$$);
```

## SERVICE 子句语法

```sparql
SERVICE <endpoint-url> { ... graph pattern ... }
SERVICE SILENT <endpoint-url> { ... }
SERVICE ?var { ... }  -- 可变端点（需要 VALUES 绑定）
```

- **`SERVICE <url> { … }`** — 在远程 SPARQL 端点执行内部模式。如果调用失败，则引发 ERROR（除非 `federation_on_error = 'empty'`）。
- **`SERVICE SILENT <url> { … }`** — 相同，但在失败时静默返回空结果。仍然记录 WARNING。
- **`SERVICE ?var { … }` 与 `VALUES`** — 将端点 URL 绑定到变量，允许动态调度。

## 端点注册

只允许白名单上的端点。调用未注册的 URL 会引发错误 — 这可以防止服务器侧请求伪造（SSRF）攻击。

### `pg_ripple.register_endpoint(url, local_view_name, complexity)`

注册远程 SPARQL 端点。

| 参数 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `url` | `TEXT` | — | 端点的完整 URL（例如 `https://dbpedia.org/sparql`） |
| `local_view_name` | `TEXT` | `NULL` | 用于预先从该端点物化数据的本地 SPARQL 视图流表的可选名称。设置后，针对此 URL 的 `SERVICE` 调用将被重写为扫描本地表而不是进行 HTTP 调用。 |
| `complexity` | `TEXT` | `'normal'` | 端点速度提示：`'fast'`、`'normal'` 或 `'slow'`。当多个 `SERVICE` 子句针对不同端点时，影响查询计划顺序。 |

```sql
-- 注册纯远程端点
SELECT pg_ripple.register_endpoint('https://dbpedia.org/sparql');

-- 注册时使用本地视图覆盖（SERVICE 变为本地扫描）
SELECT pg_ripple.register_endpoint(
    'https://internal-kb.example.com/sparql',
    'my_local_view_stream'
);

-- 注册已知快速的端点
SELECT pg_ripple.register_endpoint(
    'https://fast-endpoint.example.com/sparql',
    NULL,
    'fast'
);
```

### `pg_ripple.remove_endpoint(url)`

永久从白名单中删除端点。

```sql
SELECT pg_ripple.remove_endpoint('https://dbpedia.org/sparql');
```

### `pg_ripple.disable_endpoint(url)`

临时禁用端点而不删除它。通过再次调用 `register_endpoint()` 重新启用。

```sql
SELECT pg_ripple.disable_endpoint('https://slow-endpoint.example.com/sparql');
-- 稍后：
SELECT pg_ripple.register_endpoint('https://slow-endpoint.example.com/sparql');
```

### `pg_ripple.list_endpoints()`

列出所有已注册的端点。

```sql
SELECT * FROM pg_ripple.list_endpoints();
```

返回：`(url TEXT, enabled BOOLEAN, local_view_name TEXT, complexity TEXT)`。

## 配置 GUC

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.federation_timeout` | `30` | 每个 SERVICE 调用的挂钟超时（秒）。 |
| `pg_ripple.federation_max_results` | `10000` | 单个远程调用接受的最大行数。额外的行将被静默丢弃。 |
| `pg_ripple.federation_on_error` | `'warning'` | 失败时的行为：`'warning'`（发出 WARNING，返回空结果），`'error'`（引发 ERROR），`'empty'`（静默空结果）。 |

```sql
-- 为延迟敏感的查询收紧超时
SET pg_ripple.federation_timeout = 5;

-- 在任何 SERVICE 失败时引发错误
SET pg_ripple.federation_on_error = 'error';
```

## 使用 VALUES 的可变端点

```sparql
SELECT ?s ?label WHERE {
  VALUES ?endpoint {
    <https://query.wikidata.org/sparql>
    <https://dbpedia.org/sparql>
  }
  SERVICE ?endpoint {
    ?s <http://www.w3.org/2000/01/rdf-schema#label> ?label
    FILTER(LANG(?label) = "en")
  }
}
```

两个端点都必须注册。两个端点的结果被组合并通过 `SELECT DISTINCT` 去重。

## 本地视图重写

当 `SERVICE` 端点设置了 `local_view_name` 时，pg_ripple 将 `SERVICE` 子句重写为直接扫描预先物化的流表：

- **无 HTTP 调用**：零网络延迟。
- **PostgreSQL 规划器优化**：本地扫描参与完整查询计划。
- **准确的统计信息**：对流表的 `ANALYZE` 为规划器提供基数信息。

使用 `create_sparql_view()` 设置（请参阅[视图](views.md)），然后使用视图名称注册该端点：

```sql
-- 创建由流表支持的 SPARQL 视图
SELECT pg_ripple.create_sparql_view(
    'eu_companies',
    'SELECT ?company ?name WHERE { ?company <https://eu.example.org/name> ?name }',
    'manual'
);

-- 使用本地视图作为覆盖注册远程端点
SELECT pg_ripple.register_endpoint(
    'https://eu-kb.example.com/sparql',
    '_pg_ripple.eu_companies'  -- 流表名称
);
```

## 基于健康状况的端点跳过

pg_ripple 在 `_pg_ripple.federation_health` 中跟踪每个 SERVICE 调用的成功/失败。如果已注册的端点在过去 5 分钟内的成功率低于 10%，则执行器自动跳过它（发出 WARNING），而不是等待完整的超时。这可防止单个慢速端点阻塞整个查询。

```sql
-- 检查最近的健康状况
SELECT url,
       COUNT(*) AS total_probes,
       AVG(CASE WHEN success THEN 1.0 ELSE 0.0 END) AS success_rate,
       AVG(latency_ms) AS avg_latency_ms
FROM _pg_ripple.federation_health
WHERE probed_at >= now() - INTERVAL '5 minutes'
GROUP BY url;
```

## SSRF 保护

pg_ripple 强制执行严格的白名单：只能联系通过 `register_endpoint()` 注册的端点。任何针对未注册 URL 的 `SERVICE` 子句都会引发：

```
ERROR: federation endpoint not registered: http://internal-host/sparql;
       use pg_ripple.register_endpoint() to allow it
```

这可以防止查询被用作探测内部网络服务的媒介。

## 并行性

在 PostgreSQL 会话（SPI 上下文）内，单个查询中的多个 `SERVICE` 子句**按顺序执行**，以避免 HTTP I/O 和 SPI 事务之间的冲突。pg_ripple_http sidecar 进程可以通过其异步运行时并行执行联邦调用；对性能至关重要的联邦工作负载应使用 HTTP 接口。

---

## v0.19.0：性能改进

### 连接池

每个后端的线程本地 `ureq::Agent` 跨会话中的 `SERVICE` 调用重用 TCP 和 TLS 会话。以前每次调用都打开并丢弃新的 TCP 连接。

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.federation_pool_size` | 4 | 池中每个端点保持的空闲连接（1–32） |

```sql
-- 为具有许多端点的延迟敏感型工作负载使用更大的池
SET pg_ripple.federation_pool_size = 16;
```

### 使用 TTL 的结果缓存

当 `pg_ripple.federation_cache_ttl > 0` 时，成功的远程结果存储在 `_pg_ripple.federation_cache` 中。TTL 窗口内对相同端点 URL 和 SPARQL 文本的重复调用完全跳过 HTTP 调用。

缓存键是 `(url, XXH3-64(sparql_text))。过期行由合并后台工作线程在每个轮询周期上清除。

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.federation_cache_ttl` | 0 | 缓存 TTL（秒）；0 = 禁用（0–86400） |

```sql
-- 将 Wikidata 标签结果缓存 10 分钟
SET pg_ripple.federation_cache_ttl = 600;

-- 检查缓存
SELECT url, query_hash, cached_at, expires_at
FROM _pg_ripple.federation_cache
ORDER BY cached_at DESC;

-- 手动清除缓存
DELETE FROM _pg_ripple.federation_cache;
```

**何时使用缓存：**
- 不经常更新的参考数据集（Wikidata 标签、DBpedia 类别、受控词汇表）。
- 同一子模式被评估多次的查询（例如在循环内或从应用程序重复调用 SPARQL）。

**何时不使用缓存：**
- 实时事件流、传感器数据或任何新鲜度重要的端点。
- 返回大型变量结果集的端点（高缓存未命中率、高存储成本）。

### 端点复杂性提示

使用性能提示注册端点以指导多端点查询排序。快速端点首先执行，从而实现更早的故障检测和更低的总挂钟时间。

```sql
-- 使用提示注册
SELECT pg_ripple.register_endpoint(
    'https://fast-kb.example.com/sparql',
    NULL,        -- local_view_name
    'fast'       -- complexity: 'fast', 'normal' 或 'slow'
);

-- 注册后更新
SELECT pg_ripple.set_endpoint_complexity('https://slow-kb.example.com/sparql', 'slow');

-- 查看所有具有复杂性的端点
SELECT url, enabled, complexity FROM pg_ripple.list_endpoints();
```

### 变量投影重写

pg_ripple 现在不是将 `SELECT * WHERE { … }` 发送到远程端点，而是发送显式的 `SELECT ?v1 ?v2 … WHERE { … }`，列出出现在内部模式中的变量。这：

- 当远程支持投影下推时减少数据传输。
- 为缓存键匹配生成稳定的、确定的查询文本。
- 使检查发送的 SPARQL 更容易（失败时在 WARNING 消息中可见）。

### 部分结果处理

当 `pg_ripple.federation_on_partial = 'use'` 时，连接中途丢失使用接收到的任何行，而不是完全丢弃它们。WARNING 命名端点、接收到的行数和错误。

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.federation_on_partial` | `'empty'` | `'empty'` = 全部丢弃，`'use'` = 保留部分行 |

```sql
SET pg_ripple.federation_on_partial = 'use';
```

### 自适应超时

当 `pg_ripple.federation_adaptive_timeout = on` 时，有效的每个端点超时源自 `_pg_ripple.federation_health` 中观察到的 `max(1s, p95_latency_ms × 3 / 1000)`。快速端点获得更紧凑的超时；慢速端点获得更多缓冲。如果没有可用的健康数据，则回退到 `pg_ripple.federation_timeout`。

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.federation_adaptive_timeout` | `off` | 从 P95 健康数据派生超时 |

```sql
SET pg_ripple.federation_adaptive_timeout = on;
```

### 批量 SERVICE 调用

当单个查询包含两个或更多 `SERVICE` 子句针对**相同**的已注册端点且具有**独立**的内部模式（无共享变量）时，pg_ripple 将它们合并为单个 HTTP 请求：

```sparql
SELECT * WHERE {
  SERVICE <https://kb.example.com/sparql> { ?s <ex:label> ?label }
  SERVICE <https://kb.example.com/sparql> { ?s <ex:type>  ?type  }
  # ^ 一个 HTTP 请求：SELECT * WHERE { { ?s <ex:label> ?label } UNION { ?s <ex:type> ?type } }
}
```

这将从同一端点提取多个独立属性的查询的 HTTP 往返次数减半。

### GUC 参考（v0.19.0 新增）

| GUC | 类型 | 默认值 | 范围 | 描述 |
|---|---|---|---|---|
| `pg_ripple.federation_pool_size` | INT | 4 | 1–32 | 线程本地池中每个端点的空闲连接数 |
| `pg_ripple.federation_cache_ttl` | INT | 0 | 0–86400 | 结果缓存 TTL（秒）（0 = 禁用） |
| `pg_ripple.federation_on_partial` | STRING | `'empty'` | `'empty'`、`'use'` | SERVICE 在传递行后失败时的行为 |
| `pg_ripple.federation_adaptive_timeout` | BOOL | off | — | 从 P95 健康延迟派生超时 |

## 限制

- 运行时**无 bind-join 下推**：完整的内部模式在没有预先绑定已知变量的情况下发送到远程端点。
- **仅 SPARQL results+JSON**：直接 SPI 路径尚不支持 XML 响应格式。
- **无流式传输**：远程结果在字典编码之前完全缓存在内存中。大型结果集应使用 `federation_max_results` 来限制内存使用。