# 规则库联邦

本指南将逐步演示在 pg_ripple 实例（实例 A）上发布规则库，并从第二个实例（实例 B）订阅该规则库的完整端到端示例。

## 先决条件

- 两个运行中的 pg_ripple 0.120.0+ 实例，均配备 pg_ripple_http
- 两个 HTTP 配套服务能够通过 HTTPS 互相访问
- 已为对受保护流端点的读取访问配置 `PG_RIPPLE_HTTP_AUTH_TOKEN`。如果订阅实例上设置了 `PG_RIPPLE_HTTP_DATALOG_WRITE_TOKEN`，则在订阅调用中使用它；否则使用主身份验证令牌
- 源流必须可由订阅 HTTP 配套服务访问。当前的订阅处理程序会直接获取 `source_uri`，而不会转发单独的源授权头，因此在需要源身份验证时，通过可信的内部网络或网关保护私有流

## 第 1 步 —— 在实例 A 上安装规则库

```sql
-- 连接到实例 A
\c ripple_a

-- 安装 RDFS 推理规则集
SELECT pg_ripple.install_rule_library(
    'rdfs-entailment',
    '1.0.0',
    'RDFS entailment rules: rdfs:subClassOf, rdfs:subPropertyOf, rdfs:domain, rdfs:range'
);

-- 添加核心 RDFS 规则
SELECT pg_ripple.add_rule(
    'rdfs-entailment',
    '?x rdf:type ?C :- ?x rdf:type ?D, ?D rdfs:subClassOf ?C .'
);
SELECT pg_ripple.add_rule(
    'rdfs-entailment',
    '?x ?q ?y :- ?x ?p ?y, ?p rdfs:subPropertyOf ?q .'
);
```

## 第 2 步 —— 在实例 A 上发布规则库

```sql
-- 通过 SQL 函数发布（记录到联邦目录中）
SELECT pg_ripple.publish_rule_library(
    'rdfs-entailment',
    'https://instance-a.example.com/rule-libraries/rdfs-entailment/stream'
);
```

HTTP 配套服务立即开始在以下位置提供该流：
`GET /rule-libraries/rdfs-entailment/stream`。

## 第 3 步 —— 从实例 B 订阅

```sql
-- 连接到实例 B
\c ripple_b

-- 记录订阅意图。SQL 函数会验证 URI 和目录状态，
-- 但实际的获取/安装步骤由 HTTP 配套服务执行。
SELECT pg_ripple.subscribe_rule_library(
    'https://instance-a.example.com/rule-libraries/rdfs-entailment/stream',
    'rdfs-entailment'
);
-- (void —— 出错时抛出 PT046x)
```

或者，使用实例 B 配套服务的 HTTP API：

```bash
curl -X POST https://instance-b.example.com/rule-libraries/rdfs-entailment/subscribe \
  -H "Authorization: Bearer $PG_RIPPLE_HTTP_AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"source_uri": "https://instance-a.example.com/rule-libraries/rdfs-entailment/stream"}'
```

## 第 4 步 —— 验证共享推理

在实例 B 上加载一些三元组，并验证使用订阅的规则触发 RDFS 推理：

```sql
\c ripple_b

-- 加载一个小型本体片段
SELECT pg_ripple.load_turtle('
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:   <https://example.org/> .

ex:GraduateStudent rdfs:subClassOf ex:Student .
ex:alice            a ex:GraduateStudent .
');

-- 启用 Datalog 推理
SELECT pg_ripple.enable_datalog('rdfs-entailment');

-- 验证推理：alice 现在应该被推断为 Student
SELECT pg_ripple.sparql_query('
  SELECT ?type WHERE {
    <https://example.org/alice> a ?type .
  }
');
-- 预期：行包括 ex:GraduateStudent（声明的）以及 ex:Student（推断的）
```

## 第 5 步 —— 使用 Prometheus 监控

执行第 3 步后，实例 B 的 HTTP 配套服务上将填充以下指标：

```
# 实例 A 上的流持续时间（累计服务流请求的秒数）
pg_ripple_rule_library_stream_duration_seconds 0.012345

# 实例 B 上的订阅错误（成功订阅应为 0）
pg_ripple_rule_library_subscribe_errors_total 0
```

为失败的订阅添加告警规则：

```yaml
- alert: RuleLibrarySubscribeFailed
  expr: increase(pg_ripple_rule_library_subscribe_errors_total[5m]) > 0
  labels:
    severity: warning
  annotations:
    summary: "Rule library subscribe errors detected"
    description: "{{ $value }} subscribe errors in the last 5 minutes."
```

## 故障排除

| 症状 | 可能原因 | 解决方法 |
|---------|-------------|-----|
| `PT0466: SSRF blocked` | `source_uri` 解析为私有 IP | 使用公共 HTTPS 端点；私有地址被 SSRF 防护阻止 |
| `remote returned HTTP 401` | 源流需要订阅获取器未转发的身份验证 | 通过可信的内部路由公开源流，或从经过身份验证的客户端手动安装规则库 |
| `PT0467: catalog write failed` | 数据库权限不足 | 向应用角色授予对 `_pg_ripple` schema 的 USAGE 权限 |
| 未触发推理 | Datalog 引擎未启用 | 运行 `SELECT pg_ripple.enable_datalog('rdfs-entailment');` |

## 另请参阅

- [`publish_rule_library` API 参考](../reference/sql-api.md#publish_rule_library)
- [`subscribe_rule_library` API 参考](../reference/sql-api.md#subscribe_rule_library)
- [规则库 Prometheus 指标](../operations/read-replicas.md)
- [博客：规则库联邦](https://github.com/trickle-labs/pg-ripple/blob/main/blog/rule-library-federation.md)