# Cookbook：通过 JSON-LD Outbox 的 CDC → Kafka

**目标。** 将结构化图变更事件流推送到 Kafka（或 NATS、RabbitMQ、AWS SNS —— 您的事件总线所在的任何地方），无需轮询数据库，并使用下游消费者可以使用与数据库相同的 SHACL 形状进行验证的 JSON-LD 有效负载。

**为什么选择 pg_ripple。** 结合 CDC 订阅（推送，无轮询）、JSON-LD 框架（schema 形状的有效负载）和事务性 outbox 模式（无丢失事件）。

**核心结果的时间。** ~30 分钟（大部分用于连接 Kafka 生产者）。

---

## 架构

```
   triple INSERT/DELETE
            │
            ▼
   ┌────────────────────────┐
   │ CDC subscription       │  pg_ripple.create_subscription('out_persons',
   │ filter: SHACL shape    │       filter_shape := '<…/PersonShape>')
   └─────────┬──────────────┘
             │ NOTIFY  ──── JSON  ─────┐
             ▼                          │
   ┌────────────────────────┐           │
   │ outbox table           │           │  alternatively, a small
   │ INSERT trigger writes  │           │  Rust/Python LISTENer
   │ JSON-LD framed payload │           │
   └─────────┬──────────────┘           │
             ▼                          ▼
   ┌────────────────────────┐  ┌────────────────────────┐
   │ Debezium / outbox      │  │ asyncpg LISTENer       │
   │ connector → Kafka      │  │ → Kafka producer       │
   └────────────────────────┘  └────────────────────────┘
```

两种有效的实现：

- **Outbox + Debezium**（推荐用于至少一次持久性）。
- **直接 LISTEN**（推荐用于低延迟、即用即弃流）。

该配方显示了两者。

---

## 第 1 步 —— 定义变更形状

最简单的有效负载是 JSON-LD 框架形状的*整个更新实体*，因此下游消费者看到一个自包含的文档。一次定义 frame：

```sql
SELECT pg_ripple.register_jsonld_frame('person_event', $JSON$
{
  "@context": {
    "name":  "http://xmlns.com/foaf/0.1/name",
    "email": "http://xmlns.com/foaf/0.1/mbox",
    "knows": { "@id": "http://xmlns.com/foaf/0.1/knows", "@type": "@id" }
  },
  "@type": "http://xmlns.com/foaf/0.1/Person"
}
$JSON$);
```

## 第 2 步 —— 创建 CDC 订阅

```sql
-- 可选：创建 SHACL 形状，以便订阅仅对有效的 Person 触发。
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<https://shapes.example.org/PersonShape> a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    sh:property [ sh:path foaf:name ; sh:minCount 1 ] .
$TTL$);

SELECT pg_ripple.create_subscription(
    'persons_out',
    filter_sparql := 'SELECT ?s ?p ?o WHERE { ?s a <http://xmlns.com/foaf/0.1/Person> ; ?p ?o }'
);
```

## 第 3a 步 —— 直接 LISTEN 生产者

最低延迟路径：一个微小的 LISTENer 读取 NOTIFY 流并直接生成到 Kafka。

```python
import asyncio, json, asyncpg
from aiokafka import AIOKafkaProducer

async def main():
    conn  = await asyncpg.connect("postgresql://…")
    kafka = AIOKafkaProducer(bootstrap_servers="kafka:9092")
    await kafka.start()
    queue: asyncio.Queue = asyncio.Queue()

    def callback(_conn, _pid, _channel, payload):
        queue.put_nowait(payload)

    await conn.add_listener("pg_ripple_cdc_persons_out", callback)

    while True:
        raw = await queue.get()
        evt = json.loads(raw)
        # 可选地，将更改的实体重新框架化为 JSON-LD。
        await kafka.send_and_wait("graph.persons", raw.encode())

asyncio.run(main())
```

CDC 有效负载已经包括 `op`、`s`、`p`、`o`、`g` —— 请参阅 [CDC 订阅](../features/live-views-and-subscriptions.md#cdc-subscriptions)。

对于完整实体有效负载（而不是每三元组事件），在 LISTENer 内调用 `sparql_construct_jsonld()` 在更改时获取框架化的实体。

## 第 3b 步 —— Outbox + Debezium

对于至少一次持久性，您希望事件位于*表*中，由 Debezium 复制。添加一个 outbox 触发器，在写入时具体化 JSON-LD 有效负载：

```sql
CREATE TABLE outbox (
    id          BIGSERIAL PRIMARY KEY,
    aggregate   TEXT NOT NULL,
    event_type  TEXT NOT NULL,
    payload     JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION enqueue_person_event() RETURNS trigger AS $$
DECLARE
    framed JSONB;
BEGIN
    framed := pg_ripple.sparql_construct_jsonld(
        format(
            'CONSTRUCT { %s ?p ?o } WHERE { %s ?p ?o }',
            NEW.s, NEW.s
        ),
        frame_name := 'person_event'
    );
    INSERT INTO outbox (aggregate, event_type, payload)
    VALUES (NEW.s, 'PersonChanged', framed);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- pg_ripple.cdc_events 是由每个订阅写入的追赶表。
CREATE TRIGGER outbox_persons
AFTER INSERT ON _pg_ripple.cdc_events
FOR EACH ROW
WHEN (NEW.subscription = 'persons_out')
EXECUTE FUNCTION enqueue_person_event();
```

将 Debezium 指向 `outbox` 表；将其配置为在发布后删除行。该模式是规范的，并且正是 Debezium 的 "outbox event router" SMT 设计的。

## 第 4 步 —— 验证消费者

与过滤 CDC 订阅相同的 SHACL 形状可以作为合同运送给下游消费者。读取 JSON-LD 有效负载的消费者可以使用任何标准 SHACL 库（Python `pyshacl`、Java `topbraid-shacl` 等）对其进行验证。如果合同更改，则只有一个文档更改 —— `<https://shapes.example.org/PersonShape>` 定义。

---

## 失败模式及其处理方式

| 失败 | 直接 LISTEN | Outbox + Debezium |
|---|---|---|
| 订阅者崩溃 | 事件丢失 | 事件持久化，重启时重放 |
| NOTIFY 队列溢出 | 事件被丢弃 | 空数组增长；处理背压 |
| 消费者慢 | 生产者背压 | 空数组增长；清理滞后 |
| Schema 偏移 | 消费者解析垃圾 | Outbox + SHACL 在发布前捕获它 |

Outbox 路径有更多代码；作为回报，您可以获得大多数生产事件总线期望的持久性保证。

---

## 另请参阅

- [实时视图和订阅](../features/live-views-and-subscriptions.md)
- [导出和共享 —— JSON-LD 框架](../features/exporting-and-sharing.md)
- [API 和集成 —— `pg_ripple_http`](../features/apis-and-integration.md)