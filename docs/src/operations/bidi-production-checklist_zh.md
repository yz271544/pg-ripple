# 双向生产清单

> v0.78.0 — BIDIOPS-DOC-01

在将双向集成投入生产之前，请使用此清单。每个项目链接到相关的 runbook 部分。

---

## 1. 扩展版本

- [ ] `SELECT pg_ripple.version()` 返回 `0.78.0` 或更高版本。
- [ ] `pg_ripple.control` 的 `default_version` 与安装的版本匹配。

## 2. 模式完整性

运行此查询 — 期望 **7 行**（v0.78.0 中每个必需的表一行）：

```sql
SELECT tablename FROM pg_tables
WHERE schemaname = '_pg_ripple'
  AND tablename IN (
      'event_dead_letters','subscription_schema_changes',
      'subscription_tokens','admin_tokens','event_audit',
      'reconciliation_queue','pending_linkbacks'
  )
ORDER BY tablename;
```

## 3. 订阅

- [ ] 每个订阅都通过 `pg_ripple.create_subscription(...)` 注册。
- [ ] `overflow_policy` 已配置（`drop` 或 `dead_letter` — 永远不能为 `NULL`）。
- [ ] `max_queue_depth` 设置为合适的值（推荐：10,000–100,000）。
- [ ] `dead_letter_after` 设置为适当的重试次数（推荐：3–10）。
- [ ] Frame 包括所有 PII 谓词的 `"@redact": true`。

## 4. 令牌

- [ ] 至少一个通过 `pg_ripple.register_subscription_token(...)` 注册的每个订阅的令牌。
- [ ] 所有令牌都使用**最小特权范围**（不超过中继要求的范围）。
- [ ] 令牌轮换计划已记录（推荐：90 天轮换）。
- [ ] 管理员令牌在 `_pg_ripple.admin_tokens` 中注册（如果启用了 HTTP 管理员 API）。

## 5. 审计日志

- [ ] `pg_ripple.audit_log_enabled` 为 `on`。
- [ ] 已设置 `pg_ripple.audit_retention`（默认：90 天；根据合规要求进行调整）。
- [ ] 确认 `purge_event_audit()` 每天运行（通过 `pg_cron` 或等效的调度程序）。

## 6. 健康监控

- [ ] `pg_ripple.bidi_health()` 被告警系统轮询（推荐：每 60 秒）。
- [ ] 当 `status = 'failing'` 和 `status = 'degraded'` 持续超过 15 分钟时设置告警。
- [ ] `pg_ripple.bidi_status()` 包含在仪表板中（Grafana / Prometheus 导出）。

## 7. 死信分类

- [ ] 至少每周审查一次 `event_dead_letters`。
- [ ] 当 `dead_letter_count > 0` 持续超过 1 小时时设置告警。
- [ ] 重新排队/删除程序的 runbook 从 on-call 指南链接。

## 8. 协调

- [ ] 通过 `bidi_status()` 监控 `reconciliation_open`。
- [ ] On-call 程序记录了每个解决操作（`accept_external`、`force_internal`、`merge_via_owl_sameAs`、`dead_letter`）。

## 9. 模式演进

- [ ] 任何计划的 frame、IRI 模板或 exclude-graph 更改都通过[模式演进推出手册](./bidi-runbook.md#schema-evolution-rollout-playbook)进行。
- [ ] 每次更改后审查 `subscription_schema_changes` 表。

## 10. 混沌测试

- [ ] 在每次主要发布之前，`tests/stress/bidi_chaos.sh` 在生产实例上通过。
- [ ] 确认 Linkback 幂等性（重新调用 `abandon_linkback` ≥ 2× 无错误）。

## 11. pg_tide 中继

- [ ] pg_tide 版本与兼容性矩阵匹配（请参阅 [compatibility.md](./compatibility.md)）。
- [ ] `SELECT pg_ripple.relay_available();` 在中继数据库中返回 `true`。
- [ ] 中继在 `outbox_oldest_age` SLA 内确认行（推荐：正常负载下 ≤ 1 分钟）。
- [ ] 监控中继 TLS 证书到期。

## 12. 备份和恢复

- [ ] `_pg_ripple.event_dead_letters`、`event_audit`、`subscription_tokens`、`reconciliation_queue` 包含在备份目标中。
- [ ] 时间点恢复演练已验证这些表是可恢复的。

---

## 签署

| 项目 | 所有者 | 日期 |
|---|---|---|
| 扩展版本已验证 | | |
| 模式完整性已验证 | | |
| 订阅已配置 | | |
| 令牌已注册 | | |
| 审计日志处于活动状态 | | |
| 健康监控处于活动状态 | | |
| 死信分类程序已记录 | | |
| 混沌测试通过 | | |
| pg_tide 中继已确认 | | |
| 备份已验证 | | |
