# 双向运维 Runbook

> v0.78.0 — BIDIOPS-DOC-01

本 runbook 涵盖 v0.77.0 和 v0.78.0 中引入的双向集成（bidi）子系统的日常运维。它假定读者已完成 [pg-tide 中继指南](./pg-tide-relay.md)。

---

## 队列排空过程

当中继出现故障或 pg_tide 传递停滞时：

1. **检查队列深度** 针对每个订阅：

   ```sql
   SELECT subscription_name, outbox_depth, outbox_oldest_age,
          dead_letter_count, pg_tide_paused
   FROM pg_ripple.bidi_status();
   ```

2. **暂停传递** 如果您需要维护窗口：

   ```sql
   SELECT tide.relay_disable('<pipeline_name>');
   ```

   暂停时 `bidi_status()` 将显示 `pg_tide_paused = true`。新的 outbox 事件继续累积。

3. **解决中继问题**，然后恢复：

   ```sql
   SELECT tide.relay_enable('<pipeline_name>');
   ```

4. **监控排空进度** 直到 `outbox_depth` 返回 0：

   ```sql
   SELECT subscription_name, outbox_depth FROM pg_ripple.bidi_status();
   ```

5. **检查死信事件** 如果在暂停期间积累了任何事件：

   ```sql
   SELECT * FROM pg_ripple.list_dead_letters('<subscription_name>');
   ```

---

## 令牌轮换过程

应定期（例如每 90 天）或在怀疑泄露后立即轮换每个订阅的 Bearer 令牌。

1. **在旧令牌仍处于活动状态时注册新令牌**：

   ```sql
   SELECT pg_ripple.register_subscription_token(
       '<subscription_name>',
       ARRAY['linkback','divergence','abandon'],
       'crm-relay-2026-Q3'
   );
   ```

   安全地存储返回的原始令牌（它仅显示一次）。

2. **将新令牌分发** 给中继。允许短暂的重叠期（≥ 令牌缓存 TTL，60 秒）。

3. **撤销旧令牌** 一旦 `last_used_at` 确认中继流量已切换：

   ```sql
   SELECT pg_ripple.revoke_subscription_token(<old_token_hash_bytea>);
   ```

4. **验证** 只有新令牌处于活动状态：

   ```sql
   SELECT label, scopes, last_used_at, revoked_at
   FROM pg_ripple.list_subscription_tokens('<subscription_name>');
   ```

---

## 脱敏策略指南

- 在订阅的 JSON-LD frame 中使用 `"@redact": true` 标记 PII 和包含秘密的谓词。
- 脱敏谓词在标准 outbox 中呈现为 `{"@redacted": true}`。
- 对于需要明文的合规性管道，请配置一个单独的订阅到未脱敏的 outbox 表，并仅向提升中继的凭据授予访问权限（标准 PostgreSQL GRANT）。
- frame 是**唯一**的脱敏真相来源 — 没有额外的允许列表。
- 请参阅下面的[脱敏模式](#pattern-redaction-with-elevated-subscription)。

---

## 模式演进推出手册

当您需要更改订阅 frame、IRI 模板或排除列表时：

1. **Frame 更改**（添加/删除谓词）：

   ```sql
   SELECT pg_ripple.alter_subscription(
       '<name>',
       frame_change_policy => 'new_events_only'
   );
   -- 然后单独更新订阅的 frame 列。
   ```

   已排队的 outbox 行使用旧 frame 排空；新事件使用更新的 frame。

2. **IRI 模板或匹配模式更改**：

   ```sql
   SELECT pg_ripple.alter_subscription(
       '<name>',
       iri_change_policy => 'new_events_only'
   );
   ```

   已经渲染的 outbox IRI 不会被追溯重写。
   对于损坏的模板，请暂停 pg_tide 中继管道，如果需要则删除/重新排队受影响的行，
   并在 `_pg_ripple.subscription_schema_changes` 中记录该操作。

3. **Exclude-graphs 更改**：

   ```sql
   SELECT pg_ripple.alter_subscription(
       '<name>',
       exclude_change_policy => 'new_events_only'
   );
   ```

   所有更改都自动记录在 `_pg_ripple.subscription_schema_changes` 中：

   ```sql
   SELECT * FROM _pg_ripple.subscription_schema_changes
   WHERE subscription_name = '<name>'
   ORDER BY changed_at DESC;
   ```

---

## 协调手册

当中继报告 CAS 分歧（实际值与 base 不同时）：

1. **将分歧入队**（中继自动调用此操作）：

   ```sql
   SELECT pg_ripple.reconciliation_enqueue(
       '<event_id>'::uuid,
       '{"ex:phone": {"actual": "+1-555-0100", "base": "+1-555-0200", "after": "+1-555-0300"}}'::jsonb
   );
   ```

2. **拉取下一项** 进行审查：

   ```sql
   SELECT * FROM pg_ripple.reconciliation_next('<subscription_name>');
   ```

3. **选择解决操作**：

   | 操作 | 何时使用 |
   |---|---|
   | `accept_external` | 外部系统是权威；摄取其实际值 |
   | `force_internal` | 中心是权威；无条件重新发出事件 |
   | `merge_via_owl_sameAs` | 分歧揭示了重复主体；声明 `owl:sameAs` |
   | `dead_letter` | 现在无法解决；移至死信以供以后审查 |

   ```sql
   SELECT pg_ripple.reconciliation_resolve(
       <reconciliation_id>,
       'accept_external',
       'Confirmed with CRM team: external value is correct'
   );
   ```

4. **监控未处理项**：

   ```sql
   SELECT subscription_name, reconciliation_open
   FROM pg_ripple.bidi_status()
   WHERE reconciliation_open > 0;
   ```

---

## 混沌测试解释

双向混沌测试（`tests/stress/bidi_chaos.sh`）运行以下冒烟场景：

1. **abandon_linkback 幂等性** — 调用两次不应出错。
2. **purge_event_audit 零行安全性** — 表为空时无错误。
3. **协调入队/解决循环** — 端到端往返。
4. **bidi_health 有效状态** — 始终返回以下之一：`healthy|degraded|paused|failing`。
5. **令牌注册/撤销** — 注册，然后撤销。

如果任何测试失败，脚本将以非零代码退出并识别失败的案例。

---

## 模式：每个订阅的身份验证

四令牌部署示例（两个中继 × 两个范围集）：

```sql
-- CRM 中继：完整的 bidi 范围。
SELECT pg_ripple.register_subscription_token(
    'crm_relay',
    ARRAY['linkback','divergence','abandon','outbox_read'],
    'crm-relay-prod'
);

-- ERP 中继：仅 outbox-read。
SELECT pg_ripple.register_subscription_token(
    'erp_relay',
    ARRAY['outbox_read'],
    'erp-relay-prod'
);

-- 操作员管理员令牌：在 _pg_ripple.admin_tokens 中注册（此处未显示）。
```

HTTP 端点强制范围检查：`POST /subscriptions/crm_relay/events/{id}/linkback` 需要 `crm_relay` 订阅的 `linkback` 范围。

---

## 模式：使用提升订阅进行脱敏 {#pattern-redaction-with-elevated-subscription}

标准中继（脱敏，普通消费者）：

```json
{
  "@context": { "ex": "https://example.com/ns#" },
  "@type": "ex:Contact",
  "ex:name": {},
  "ex:phone": { "@redact": true },
  "ex:email": {},
  "ex:taxId": { "@redact": true }
}
```

合规中继（未脱敏，提升访问）：

- 注册一个单独的订阅 `crm_relay_audit` 指向 `crm_relay_outbox_unredacted`。
- 该 frame 对所有谓词省略 `"@redact": true`。
- 仅向合规中继的数据库角色授予对 `crm_relay_outbox_unredacted` 的 `SELECT` 权限。
- 为 `crm_relay_audit` 注册一个具有 `outbox_read` 范围的单独令牌。

当启用可选的发出审计时（由 `pg_ripple.audit_log_enabled` 控制），
`event_audit` 记录未脱敏 outbox 中桥接写入器行的 `action = 'emit_unredacted'`。
