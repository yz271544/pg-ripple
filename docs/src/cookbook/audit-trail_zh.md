# Cookbook：通过 JSON-LD Outbox 实现 CDC → Kafka

**目标。** 将结构化图变更事件流推送到 Kafka（或 NATS、RabbitMQ、AWS SNS — 任何事件总线所在的地方），无需轮询数据库，并使用下游消费者可以根据与数据库使用的相同 SHACL 形状进行验证的 JSON-LD 负载。

**为什么使用 pg_ripple。** 结合 CDC 订阅（推送，无需轮询）、JSON-LD 框架（schema 形状的负载）和事务性 outbox 模式（无事件丢失）。

**首次结果时间。** 约 30 分钟（大部分是连接 Kafka producer）。

---

## 架构

```
   三元组 INSERT/DELETE
            │
            ▼
   ┌────────────────────────┐
   │ CDC 订阅               │  pg_ripple.create_subscription('out_persons',
   │ 过滤器：SHACL 形状     │       filter_shape := '<…/PersonShape>')
   └─────────┬──────────────┘
             │ NOTIFY  ──── JSON  ─────┐
             ▼                          │
   ┌────────────────────────┐           │
   │ outbox 表              │           │  或者，一个小型
   │ INSERT 触发器写入       │           │  Rust/Python 监听器
   │ JSON-LD 框架负载       │           │
   └─────────┬──────────────┘           │
             ▼                          ▼
   ┌────────────────────────┐  ┌────────────────────────┐
   │ Debezium / outbox      │  │ asyncpg LISTENer       │
   │ 连接器 → Kafka         │  │ → Kafka producer       │
   └────────────────────────┘  └────────────────────────┘
```

两种有效的实现：

- **Outbox + Debezium**（推荐用于至少一次持久性）。
- **直接 LISTEN**（推荐用于低延迟、fire-and-forget 流）。

本食谱展示两者。

---

## 第 1 步 — 定义变更形状

最简单的负载是*整个更新后的实体*采用 JSON-LD 框架形状，以便下游消费者看到一个自包含的文档。一次定义框架：

```sql
SELECT pg_ripple.register_jsonld_frame('person_event', $JSON$
{
  "@context": {
    "name":  "http://xmlns.com/foaf/0.0.1/name",
    "email": "http://xmlns.com/foaf/0.0.1/mbox",
    "knows": { "@id": "http://xmlns.com/foaf/0.0.1/knows", "@type": "@id" }
  },
  "@type": "http://xmlns.com/foaf/0.0.1/Person"
}
$JSON$);
```

## 第 2 步 — 创建 CDC 订阅

```sql
-- 可选：创建 SHACL 形状，以便订阅仅对有效的 Persons 触发。
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix foaf: <http://xmlns.com/foaf/0.0.1/> .

<https://shapes.example.org/PersonShape> a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    sh:property [ sh:path foaf:name ; sh:minCount 1 ] .
$TTL$);

SELECT pg_ripple.create_subscription(
    'persons_out',
    filter_sparql := 'SELECT ?s ?p ?o WHERE { ?s a <http://xmlns.com/foaf/0.0.1/Person> ; ?p ?o }'
);
```

## 第 3a 步 — 直接 LISTEN 生产者

最低延迟路径：一个小型 LISTENer 读取 NOTIFY 流并直接生成到 Kafka。

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
        # 可选地将变更的实体重新框架化为 JSON-LD。
        await kafka.send_and_wait("graph.persons", raw.encode())

asyncio.run(main())
```

CDC 负载已包括 `op`、`s`、`p`、`o`、`g` — 请参阅 [CDC 订阅](../features/live-views-and-subscriptions.md#cdc-subscriptions)。

对于完整实体负载（而不是每个三元组的事件），在 LISTENer 内调用 `sparql_construct_jsonld()` 以在变更时获取框架化实体。

## 第 3b 步 — Outbox + Debezium

对于至少一次持久性，您希望事件在*表*中，由 Debezium 复制。添加一个 outbox 触发器，在写入时具体化 JSON-LD 负载：

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

将 Debezium 指向 `outbox` 表；将其配置为在发布后删除行。该模式是规范的，也正是 Debezium 的 "outbox event router" SMT 设计的用途。

## 第 4 步 — 验证消费者

过滤 CDC 订阅的相同 SHACL 形状可以作为合约传递给下游消费者。读取 JSON-LD 负载的消费者使用任何标准 SHACL 库（Python `pyshacl`、Java `topbraid-shacl` 等）对其进行验证。如果合约更改，只有一个文档更改 — `<https://shapes.example.org/PersonShape>` 定义。

---

## 失败模式及其处理方法

| 失败 | 直接 LISTEN | Outbox + Debezium |
|---|---|---|
| 订阅者崩溃 | 事件丢失 | 事件持久化，重启时重放 |
| NOTIFY 队列溢出 | 事件被丢弃 | Outbox 增长；反压处理 |
| 消费者慢 | 生产者背压 | Outbox 增长；清理滞后 |
| Schema 偏差 | 消费者解析垃圾 | Outbox + SHACL 在发布前捕获它 |

outbox 路径有更多代码；作为回报，您可以获得大多数生产事件总线期望的持久性保证。

---

## 另请参阅

- [实时视图和订阅](../features/live-views-and-subscriptions.md)
- [导出和共享 — JSON-LD 框架](../features/exporting-and-sharing.md)
- [API 和集成 — `pg_ripple_http`](../features/apis-and-integration.md)