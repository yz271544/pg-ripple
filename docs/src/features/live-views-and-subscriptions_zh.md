# 实时视图和订阅

**状态**：实时视图自 v0.12.0 起可用；CDC 订阅自 v0.42.0 起可用  
**另请参阅**：[实时 SPARQL 订阅](live-subscriptions.md) · [CDC 订阅](cdc-subscriptions.md)  

---

两个功能涵盖了 pg_ripple 的 *push* 方面 — 在数据**更改时**获取数据，而不是轮询它。

| 功能 | 适用于 | 支持 |
|---|---|---|
| **物化 SPARQL / Datalog 视图** | 始终新鲜的仪表板、反规范化表 | [pg_trickle](https://github.com/trickle-labs/pg-trickle) |
| **CDC 订阅** | 将事件流式传输到应用程序、Kafka、WebSocket 客户端 | PostgreSQL `LISTEN/NOTIFY` |

如果您想要*当前真实情况*的快照，请使用视图。如果您想要*自上次查看以来发生了什么变化*的流，请使用 CDC。

---

## 物化 SPARQL 和 Datalog 视图

SPARQL 视图将 SPARQL `SELECT`（或 Datalog 目标）编译为 pg_trickle 流表，该表随三元组的更改增量维护。视图始终反映最新数据，而无需您自己运行查询。

> 需要可选的 [`pg_trickle`](https://github.com/trickle-labs/pg-trickle) 扩展。pg_ripple 可以在没有它的情况下加载和运行；视图函数在调用时检测其缺失并返回带有安装提示的明确错误。

```sql
-- 在使用前检查可用性。
SELECT pg_ripple.pg_trickle_available();   -- true / false

-- 创建所有人及其姓名的视图，每秒刷新一次。
-- 流表存储 BIGINT 字典 ID 以确保 IVM 的正确性。
SELECT pg_ripple.create_sparql_view(
    name     := 'people_names',
    sparql   := 'SELECT ?p ?name WHERE { ?p <http://xmlns.com/foaf/0.1/name> ?name }',
    schedule := '1s',
    decode   := true   -- 还会创建具有 TEXT 列的 pg_ripple.people_names_decoded
);

-- 查询原始 BIGINT 视图。
SELECT * FROM pg_ripple.people_names;

-- 或者使用自动创建的已解码伴随视图以获得人类可读的 TEXT 输出。
SELECT * FROM pg_ripple.people_names_decoded;

-- 使用完后删除（如果存在也会删除 _decoded 伴随视图）。
SELECT pg_ripple.drop_sparql_view('people_names');
```

`decode` 标志控制是否在流表之上创建 `_{name}_decoded` 伴随 VIEW。流表本身**始终存储原始 `BIGINT` 字典 ID** — 这保持了 pg_trickle 的增量视图维护（IVM）正确运行，因为 IVM 使用底层 VP 表的整数列来区分行。当 `decode = true` 时，会在流表旁边创建一个名为 `{name}_decoded` 的薄 SQL VIEW；它在读取时执行字典查找并公开 `TEXT` 列。这是 `create_construct_view` 使用的相同模式。

您可以从 Datalog 目标构建相同类型的视图：

```sql
SELECT pg_ripple.create_datalog_view(
    name     := 'indirect_managers',
    goal     := '?x <https://example.org/indirectManager> ?y',
    schedule := '5s'
);
```

这些视图与物化推理管道集成 — 在 `infer()`、`retract_rule()` 和 `clear_graph()` 之后视图保持正确。

---

## CDC 订阅

每次插入或删除与 SPARQL 过滤器匹配的三元组时，订阅都会在命名 PostgreSQL `NOTIFY` 通道上发布 JSON 消息。侦听器无需轮询即可接收更改。

> 自 v0.42.0 起可用。

### 创建

```sql
-- 订阅所有更改。
SELECT pg_ripple.create_subscription('all_changes');

-- 使用 SPARQL 模式过滤器订阅。
SELECT pg_ripple.create_subscription(
    'person_changes',
    filter_sparql := 'SELECT ?s ?p ?o WHERE { ?s a <https://schema.org/Person> ; ?p ?o }'
);

-- 使用 SHACL 形状过滤器订阅 — 仅发布*违反*形状的三元组。
SELECT pg_ripple.create_subscription(
    'shape_violations',
    filter_shape := '<https://shapes.example.org/PersonShape>'
);
```

### 侦听

```sql
LISTEN pg_ripple_cdc_person_changes;
```

在您的应用程序（Python、Node.js、Go 等）中连接，发出相同的 `LISTEN`，然后读取通知流：

```python
import psycopg
import json

with psycopg.connect("...") as conn:
    conn.set_isolation_level(0)  # AUTOCOMMIT
    with conn.cursor() as cur:
        cur.execute("LISTEN pg_ripple_cdc_person_changes;")
    for notify in conn.notify():
        event = json.loads(notify.payload)
        print(event["op"], event["s"], event["p"], event["o"])
```

### 负载

```json
{
  "op": "add",
  "s": "<https://example.org/alice>",
  "p": "<https://schema.org/name>",
  "o": "\"Alice\"",
  "g": ""
}
```

| 字段 | 含义 |
|---|---|
| `op` | `"add"` 或 `"remove"` |
| `s` / `p` / `o` | N-Triples 格式的主语 / 谓词 / 对象 |
| `g` | 命名图 IRI，或默认图为空字符串 |

### 管理

```sql
SELECT name, has_filter, created_at FROM pg_ripple.list_subscriptions();
SELECT pg_ripple.drop_subscription('person_changes');
```

### 通过 `pg_ripple_http` 的 WebSocket 访问

当 [`pg_ripple_http`](apis-and-integration.md) 配套服务运行时，每个订阅都会自动公开为 WebSocket 端点：

```
ws://<host>:8080/ws/subscriptions/{name}
```

HTTP 服务处理背压、重连和身份验证 — 您将浏览器端 EventSource 或服务器端流使用者指向该 URL。

### 生命周期遥测

`_pg_ripple.cdc_lifecycle_events` 表记录每次订阅创建 / 删除 / 错误，并带有时间戳。可用于在生产中针对已删除的订阅发出警报。

---

## 在视图和订阅之间选择

| 如果您想要… | 使用 |
|---|---|
| 始终显示最新聚合 / 投影的表 | **视图** |
| 每次更改的推送通知以驱动外部系统 | **订阅** |
| 浏览器的 WebSocket 流 | **订阅** + `pg_ripple_http` |
| 派生事实的反规范化缓存 | **Datalog 视图** |
| 在更改时触发 Kafka / SQS / SNS 消息 | **订阅** + outbox 工作线程 |

两者可以组合：构建一个视图来了解*当前真实情况*，构建一个订阅来了解*刚刚发生了什么变化*，并让您的应用程序根据用例进行选择。

---

## 另请参阅

- [API 和集成](apis-and-integration.md) — `pg_ripple_http` 和 WebSocket 访问。
- [Cookbook：通过 JSON-LD outbox 的 CDC→Kafka](../cookbook/cdc-to-kafka.md)

## 进一步阅读

- [博客：CONSTRUCT 视图 — 实时转换](https://github.com/trickle-labs/pg-ripple/blob/main/blog/construct-views-live-transformations.md) — SPARQL 视图的增量物化
- [博客：使用 pg-trickle 集成进行 IVM](https://github.com/trickle-labs/pg-ripple/blob/main/blog/ivm-pg-trickle-integration.md) — pg_ripple 如何使视图保持最新