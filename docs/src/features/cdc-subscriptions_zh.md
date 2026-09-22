# CDC 订阅

**状态**：自 v0.42.0 起可用（CDC-01）  
**要求**：[`pg_ripple_http`](../reference/http-api.md) 用于 Server-Sent Events 传递。PostgreSQL `LISTEN/NOTIFY` 在没有它的情况下工作。  
**SQL**：`pg_ripple.subscribe_cdc()`、`pg_ripple.unsubscribe_cdc()`、`pg_ripple.list_cdc_subscriptions()`  
**HTTP**：`GET /subscribe/{id}`（Server-Sent Events，通过 `pg_ripple_http`）  
**另请参阅**：[实时 SPARQL 订阅](live-subscriptions.md) · [实时视图](live-views-and-subscriptions.md)  

---

## 概述

**变更数据捕获（CDC）订阅**允许您的应用程序订阅 RDF 三元组更改的实时流 — 由 SPARQL 模式或 SHACL 形状过滤 — 无需轮询数据库。

当匹配的 Triple 被插入或删除时，pg_ripple 在命名通道上发送 PostgreSQL `NOTIFY` 消息。侦听器接收描述更改的 JSON 负载。`pg_ripple_http` 配套服务将这些订阅公开为 Server-Sent Events（SSE）流，用于 Web 和流式应用程序。

```mermaid
    participant App
    participant pg_ripple_http
    participant PostgreSQL
    participant pg_ripple_ext as pg_ripple extension

    App->>pg_ripple_http: GET /subscribe/{id}
    pg_ripple_http->>PostgreSQL: LISTEN pg_ripple_sub_{id}
    Note over App,pg_ripple_http: SSE 连接建立

    loop Triple Changes
        App->>PostgreSQL: INSERT triple（通过 SPARQL 或 SQL）
        pg_ripple_ext->>PostgreSQL: NOTIFY pg_ripple_sub_{id}, payload
        PostgreSQL->>pg_ripple_http: async notification
        pg_ripple_http->>App: event: row\ndata: {...}
    end
```



## 创建订阅

```sql
-- 订阅所有三元组更改。
SELECT pg_ripple.create_subscription('my_feed');

-- 使用 SPARQL 模式过滤器订阅。
SELECT pg_ripple.create_subscription(
    'person_changes',
    filter_sparql := 'SELECT ?s ?p ?o WHERE { ?s a <https://schema.org/Person> ; ?p ?o }'
);

-- 使用 SHACL 形状过滤器订阅。
SELECT pg_ripple.create_subscription(
    'shape_violations',
    filter_shape := '<https://shapes.example.org/PersonShape>'
);
```

### 参数

| 参数 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `name` | `TEXT` | 必需 | 唯一订阅名称（字母数字 + `_`/`-`，最多 63 个字符）|
| `filter_sparql` | `TEXT` | `NULL` | 可选的 SPARQL SELECT 模式；仅发布匹配的三元组 |
| `filter_shape` | `TEXT` | `NULL` | 可选的 SHACL 形状 IRI；仅发布违反形状的三元组 |

如果创建则返回 `TRUE`，如果同名订阅已存在则返回 `FALSE`。

## 侦听更改

```sql
-- 开始侦听。
LISTEN pg_ripple_cdc_my_feed;

-- 插入三元组。
SELECT pg_ripple.insert_triple(
    '<https://ex.org/alice>',
    '<https://schema.org/name>',
    '"Alice"'
);

-- 在您的应用程序中，通过 pg_notify/asyncpg/etc. 接收通知。
```

### 通知负载

每个通知都携带 JSON 负载：

```json
{
  "op": "add",
  "s": "<https://ex.org/alice>",
  "p": "<https://schema.org/name>",
  "o": "\"Alice\"",
  "g": ""
}
```

| 字段 | 值 |
|---|---|
| `op` | `"add"` 用于 INSERT，`"remove"` 用于 DELETE |
| `s` | 主语 — N-Triples 格式的 IRI 或空节点 |
| `p` | 谓词 — N-Triples 格式的 IRI |
| `o` | 对象 — N-Triples 格式的字面量或 IRI |
| `g` | 命名图 IRI，或默认图为空字符串 |

## 列出订阅

```sql
SELECT name, filter_sparql IS NOT NULL AS has_filter, created_at
FROM pg_ripple.list_subscriptions()
ORDER BY created_at;
```

## 删除订阅

```sql
-- 如果删除则返回 TRUE，如果未找到则返回 FALSE。
SELECT pg_ripple.drop_subscription('my_feed');
```

## 通过 pg_ripple_http 访问 WebSocket

当 `pg_ripple_http` 配套服务正在运行时，订阅可作为 WebSocket 端点访问：

```
ws://<host>:8080/ws/subscriptions/{name}
```

该服务通过 `Accept` 标头支持内容协商：
- `application/json`（默认）— JSON 负载
- `text/turtle` — Turtle 序列化更改通知
- `application/ld+json` — JSON-LD 更改通知

## 集成模式

### GraphRAG 管道

```python
import asyncpg

async def watch_entity_changes():
    conn = await asyncpg.connect(dsn)
    await conn.execute("LISTEN pg_ripple_cdc_entity_changes")

    async for notification in conn.listen("pg_ripple_cdc_entity_changes"):
        payload = json.loads(notification.payload)
        # 更改时重新嵌入实体。
        await update_embedding(payload["s"])
```

### 实时仪表板

```javascript
const ws = new WebSocket("ws://localhost:8080/ws/subscriptions/dashboard_feed");
ws.onmessage = (event) => {
  const change = JSON.parse(event.data);
  updateDashboard(change.op, change.s, change.p, change.o);
};
```

## 底层表

| 表 | Description |
|---|---|
| `_pg_ripple.subscriptions` | 命名订阅注册表 |
| `_pg_ripple.cdc_subscriptions` | 低级谓词模式订阅（v0.6.0 旧版 API）|

## 相关函数

| Function | Description |
|---|---|
| `pg_ripple.create_subscription(name, filter_sparql, filter_shape)` | 创建命名订阅 |
| `pg_ripple.drop_subscription(name)` | 删除命名订阅 |
| `pg_ripple.list_subscriptions()` | 列出所有命名订阅 |
| `pg_ripple.subscribe(pattern, channel)` | 低级订阅（v0.6.0 API）|
| `pg_ripple.unsubscribe(channel)` | 删除低级订阅 |

## 进一步阅读

- [博客：CDC 和知识图](https://github.com/trickle-labs/pg-ripple/blob/main/blog/cdc-knowledge-graphs.md) — 从您的知识图进行实时事件流