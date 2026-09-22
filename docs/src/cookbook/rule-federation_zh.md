# 规则库联邦食谱

> 在 v0.120.0 中添加

当您已有版本化的规则库并希望跨 pg_ripple 实例共享时，请使用本食谱。有关完整演练，包括两个实例的设置和监控示例，请参阅[规则库联邦指南](../guides/rule-library-federation.md)。

## 何时使用本食谱

规则库联邦适合维护共享 Datalog 或 SHACL 规则集的团队，例如 RDFS 推理、客户去重或合规性检查，并需要在所有环境中一致地安装相同的规则。

## 最小流程

1. 在源实例上安装或创建库。
2. 使用 HTTP 配套服务提供其流的 URL 发布库。
3. 在目标实例上记录订阅。
4. 要求目标 HTTP 配套服务提供程序获取并安装远程流。

```sql
-- 源实例：发布已安装的库。
SELECT pg_ripple.publish_rule_library(
  'rdfs-entailment',
  'https://instance-a.example.com/rule-libraries/rdfs-entailment/stream'
);

-- 目标实例：记录订阅意图。
SELECT pg_ripple.subscribe_rule_library(
  'https://instance-a.example.com/rule-libraries/rdfs-entailment/stream',
  'rdfs-entailment'
);
```

```bash
# 目标 HTTP 配套服务：获取并安装流。
curl -X POST https://instance-b.example.com/rule-libraries/rdfs-entailment/subscribe \
  -H "Authorization: Bearer $PG_RIPPLE_HTTP_AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"source_uri":"https://instance-a.example.com/rule-libraries/rdfs-entailment/stream"}'
```

## 注意

- `publish_rule_library()` 和 `subscribe_rule_library()` 是目录操作；它们不启动后台守护进程。
- `POST /rule-libraries/{name}/subscribe` 执行远程获取并将规则安装到目标实例。
- 当前的 HTTP 获取器不会将单独的 Authorization 头转发到源流。对于受保护的源流，请使用受信任的内部路由或从经过身份验证的客户端获取。

## 另请参阅

- [规则库联邦指南](../guides/rule-library-federation.md)
- [SQL API: publish_rule_library](../reference/sql-api.md#publish_rule_library)
- [SQL API: subscribe_rule_library](../reference/sql-api.md#subscribe_rule_library)
- [HTTP API 参考](../reference/http-api.md#complete-endpoint-reference)