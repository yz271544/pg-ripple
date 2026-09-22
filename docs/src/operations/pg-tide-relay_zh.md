# 双向 CRM ⇄ ERP 演练（v0.77.0）

> **自 v0.77.0 起可用**

v0.77.0 添加了通用双向集成原语，使得可以通过 pg_ripple 构建任意两个系统之间的双向中继。本演练使用 CRM ⇄ ERP 场景作为具体示例。

### 拓扑

```
  CRM（email/phone          ERP（税号/状态
   的真源）                   的真源）
       │                           │
       │  pg-tide 中继             │  pg-tide 中继
       ▼                           ▼
  <urn:source:crm>           <urn:source:erp>
       │                           │
       └──────────┬────────────────┘
                  ▼
           pg_ripple 中心
           （命名图）
                  │
          冲突策略
          （source_priority）
                  │
          已解析的投影
                  │
    ┌─────────────┴─────────────┐
    ▼                           ▼
  CRM outbox               ERP outbox
  (pg-tide)                (pg-tide)
```

### 第 1 步：使用默认图 IRI 注册映射

```sql
-- CRM 映射：联系人
SELECT pg_ripple.register_json_mapping(
    'crm_contact',
    '{"@context": {
        "ex:name":    "http://schema.org/name",
        "ex:email":   "http://schema.org/email",
        "ex:phone":   "http://schema.org/telephone"
    }}'::jsonb,
    default_graph_iri   => '<urn:source:crm>',
    timestamp_path      => '$.lastModified',
    iri_template        => 'https://crm.example.com/contacts/{id}'
);

-- ERP 映射：员工
SELECT pg_ripple.register_json_mapping(
    'erp_employee',
    '{"@context": {
        "ex:name":   "http://schema.org/name",
        "ex:taxId":  "http://schema.org/taxID",
        "ex:status": "http://schema.org/employmentType"
    }}'::jsonb,
    default_graph_iri   => '<urn:source:erp>',
    timestamp_path      => '$.updatedAt',
    iri_template        => 'https://erp.example.com/employees/{id}'
);
```

### 第 2 步：声明冲突解决策略

```sql
-- Email：CRM 是权威
SELECT pg_ripple.register_conflict_policy(
    'http://schema.org/email',
    'source_priority',
    '{"order": ["<urn:source:crm>", "<urn:source:erp>"]}'::jsonb
);

-- 名称：跨两个系统最后写入获胜
SELECT pg_ripple.register_conflict_policy(
    'http://schema.org/name',
    'latest_wins'
);

-- 税号：ERP 是唯一来源；拒绝任何 CRM 尝试
SELECT pg_ripple.register_conflict_policy(
    'http://schema.org/taxID',
    'reject_on_conflict'
);
```

### 第 3 步：摄取载荷

```sql
-- 摄取 CRM 联系人更新（diff 模式：派生每个三元组的时间戳）
SELECT pg_ripple.ingest_json(
    '{"ex:name": "Alice Smith", "ex:email": "alice@crm.example.com"}'::jsonb,
    'https://crm.example.com/contacts/42',
    'crm_contact',
    mode => 'diff'
);

-- 摄取匹配的 ERP 员工（upsert 模式：替换 sh:maxCount 1 字段）
SELECT pg_ripple.ingest_json(
    '{"ex:name": "Alice Smith", "ex:taxId": "123-45-6789"}'::jsonb,
    'https://erp.example.com/employees/E-999',
    'erp_employee',
    mode => 'upsert'
);
```

### 第 4 步：跨源实体链接（BIDI-REF-01）

ERP 确认 Alice 与 CRM 联系人 42 是同一个人后：

```sql
-- 记录链接回：CRM 中心 IRI ↔ ERP 员工 IRI
SELECT pg_ripple.record_linkback(
    'event-uuid-from-outbox'::uuid,
    target_iri => 'https://erp.example.com/employees/E-999'
);
```

这将 `<https://crm.example.com/contacts/42> owl:sameAs <https://erp.example.com/employees/E-999>` 写入目标图并刷新任何缓冲的事件。

### 第 5 步：循环安全订阅（BIDI-LOOP-01）

创建订阅时，使用 `exclude_graphs` 来防止回声循环：

```sql
-- CRM 订阅：排除 CRM 自己的图以防止循环
SELECT pg_ripple.create_subscription(
    'crm_contact_sync',
    outbox_table => 'pg_ripple_outbox.crm_contact_events'
    -- 注意：exclude_graphs 和 propagation_depth 在创建后通过
    -- _pg_ripple.subscriptions 表设置：
    -- UPDATE _pg_ripple.subscriptions
    --   SET exclude_graphs = ARRAY['<urn:source:crm>'],
    --       propagation_depth = 1
    -- WHERE name = 'crm_contact_sync';
);
```

### 第 6 步：观察每个图的指标（BIDI-OBS-01）

```sql
SELECT graph_iri, triple_count, last_write_at, conflicts_total
FROM pg_ripple.graph_stats()
ORDER BY triple_count DESC;
```

### 出站事件线协议格式（BIDI-WIRE-01）

每个发送到 outbox 表的事件遵循以下形状：

```json
{
  "version": "1.0",
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "subscription": "crm_contact_sync",
  "subject": "https://crm.example.com/contacts/42",
  "source_graph": "urn:source:crm",
  "after": {
    "http://schema.org/name": "Alice Smith",
    "http://schema.org/email": "alice@crm.example.com"
  },
  "base": {
    "http://schema.org/name": "Alice"
  },
  "subject_resolved": true
}
```

`base` 对象携带已更改谓词的先前值，在接收系统上启用比较和交换（CAS）安全性。请参阅
`assert_cas(event, actual)` 获取内置的 CAS 帮助程序。

完整的 JSON Schema 位于 `docs/src/operations/event-schema-v1.json`。

---

## 相关页面

- [CDC 运维](cdc.md)
- [Citus + pg-trickle 集成](citus-integration.md)
- [Cookbook：通过 JSON-LD Outbox 进行 CDC → Kafka](../cookbook/cdc-to-kafka.md)
- [推理和推断（Datalog）](../features/reasoning-and-inference.md)
- [验证数据质量（SHACL）](../features/validating-data-quality.md)
- [导出和共享](../features/exporting-and-sharing.md)
- [pg_tide 仓库](https://github.com/trickle-labs/pg-tide)
- [兼容性矩阵](compatibility.md)
