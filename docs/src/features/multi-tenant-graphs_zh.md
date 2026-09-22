# 多租户图

多租户图存储是任何数据库中最棘手的模式之一。您有一个逻辑产品，但有许多客户，每个客户的数据必须对其他人**不可见**、**有界**（大小限制）且**可测量**（用于计费和容量规划）。pg_ripple 通过三个可组合的构建块来解决这个问题：

| 构建块 | 为您提供什么 |
|---|---|
| **命名图** | 逻辑分区 — 每个租户生活在自己的图 IRI 中 |
| **图上的行级安全性** | 租户角色只能看到自己的图；由 PostgreSQL 本身强制执行 |
| **租户配额** | 每个租户可配置的三元组计数上限，在插入时强制执行

这些共同提供了由 PostgreSQL 权限系统支持的**真正**的租户隔离 — 没有应用级过滤，没有 SQL 注入绕过隔离的风险。

---

## 何时使用此功能

- 一种 SaaS 产品，每个客户都有自己的知识图。
- 一个研究平台，每个项目都被沙盒化。
- 一个多团队数据平台，您希望在共享实例时不共享数据。

如果您只需要*带标签的*分区而无需隔离（例如，"这个图来自 PubMed，这个来自 Crossref"），普通命名图就够了。本页面的功能是关于**强制**隔离。

---

## 模型

```
   ┌──────────────────────────────────────────────────────────────┐
   │                    pg_ripple 实例                             │
   │                                                               │
   │   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
   │   │  图 A        │  │  图 B        │  │  图 C        │      │
   │   │  (acme.com)  │  │  (globex)    │  │  (shared)    │      │
   │   │  500 K 三元  │  │  120 K 三元  │  │  20 K 三元   │      │
   │   │  配额：1M    │  │  配额：250K  │  │  配额：∞    │      │
   │   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
   │          │                 │                 │              │
   │          └────► 角色 tenant_acme   ◄─────────┤              │
   │                  角色 tenant_globex ◄────────┘              │
   │                                              │              │
   └──────────────────────────────────────────────┴──────────────┘
                                                  │
                                          所有人读取 "shared"
```

- **图 IRI** 是隔离单位：三元组在插入时带有它们的图标签。
- **角色** 是访问控制的单位：PostgreSQL 角色拥有图的权限。
- **租户** 是配额的单位：每个已注册为租户的命名图都带有一个三元组计数上限。

---

## 第 1 步 — 注册租户

`create_tenant()` 将一个命名图注册为租户并为其分配一个三元组计数配额。注册后，每次插入到该图都会根据配额进行检查；超出配额的插入会引发 `PT530`。

```sql
SELECT pg_ripple.create_tenant(
    graph_iri    := 'https://example.org/tenants/acme',
    triple_quota := 1_000_000
);

SELECT pg_ripple.create_tenant(
    graph_iri    := 'https://example.org/tenants/globex',
    triple_quota := 250_000
);
```

您可以随时列出当前使用情况：

```sql
SELECT graph_iri, triple_count, triple_quota,
       round(100.0 * triple_count / triple_quota, 1) AS pct_used
FROM pg_ripple.tenant_stats()
ORDER BY pct_used DESC;
```

该视图构建在触发器维护的计数器上，因此它是 O(1) — 可以安全地从计费数据库作业调用。

---

## 第 2 步 — 将图授权给角色

`grant_graph(role, graph_iri)` 是 `GRANT … ON … TO …` 对于图的类比。它注册一个每角色可见性规则，pg_ripple 在每个 SPARQL 查询中强制执行。

```sql
CREATE ROLE tenant_acme   LOGIN PASSWORD 'pw_a';
CREATE ROLE tenant_globex LOGIN PASSWORD 'pw_b';

GRANT USAGE ON SCHEMA pg_ripple, _pg_ripple TO tenant_acme, tenant_globex;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA pg_ripple TO tenant_acme, tenant_globex;

SELECT pg_ripple.grant_graph('tenant_acme',   'https://example.org/tenants/acme');
SELECT pg_ripple.grant_graph('tenant_globex', 'https://example.org/tenants/globex');

-- 两个租户共享对公共参考图的访问。
SELECT pg_ripple.grant_graph('tenant_acme',   'https://example.org/shared');
SELECT pg_ripple.grant_graph('tenant_globex', 'https://example.org/shared');
```

由 `tenant_acme` 运行的 SPARQL 查询现在只能看到 `acme` 和 `shared` 中的三元组。没有应用级过滤器可以忘记；规则在存储层强制执行。

---

## 第 3 步 — （可选）归属每次更改

将租户与[审计日志](temporal-and-provenance.md#audit-log)配对，实现计费级别的归属：

```sql
SET pg_ripple.audit_log_enabled = on;

-- 过去 24 小时内每租户的写后/写入量：
SELECT role, count(*) AS updates
FROM _pg_ripple.audit_log
WHERE ts > now() - interval '24 hours'
GROUP BY role
ORDER BY updates DESC;
```

对于每租户的*读取*量，捕获 `pg_stat_statements` 数据；pg_ripple 透明地与它集成。

---

## 运维问题

### 配额强制执行

强制配额的触发器在与插入相同的事务中运行。如果批量加载将超出配额，**整个加载将回滚** — 没有部分提交。相应地规划您的加载器：如果您预期接近配额，则分块加载，或通过 `tenant_stats()` 预检查。

### 驱逐

触发器触发配额。检查的是检查的是超出配额后会回滚，而不是滞后检查。当租户占满时，pg_ripple 不会自动驱逐旧三元组。要缩小租户：删除三元组（或调用 `clear_graph()`），计数器会立即更新。

### 重命名或拆分租户

`rename_tenant(old_iri, new_iri)` 在单个事务中更新注册并重新标记每个三元组。谨慎使用 — 它会触及图中的每个三元组。

### 备份和恢复

租户是纯 PostgreSQL 对象。`pg_dump --schema=pg_ripple --schema=_pg_ripple` 捕获所有内容。要导出单个租户以进行法律保留或迁移：

```sql
COPY (SELECT * FROM pg_ripple.export_quads_for_graph('https://example.org/tenants/acme'))
TO '/tmp/acme.nq';
```

---

## 失败模式和陷阱

1. **忘记授权共享图。** 没有对参考词汇表的读取权限的租户将把类型视为不透明的 IRI。始终将 `https://example.org/shared`（或您的等价物）授权给每个租户。
2. **授权默认图。** 默认图（图 ID `0`）是*不*租户范围的。不要将租户数据放入默认图；它将对所有人可见。
3. **对所有租户使用单个角色。** 配额和 RLS 附加到角色上。在租户之间共享角色会同时击败两者。
4. **主动配额 vs 被动配额。** PT530 在*提交之前*引发，而不是在*之后*。长时间运行的批量加载应拆分为批次，并在每批后检查 `tenant_stats()`。

---

## 另请参阅

- [运维 → 安全](../operations/security.md) — 更深入的 PostgreSQL RLS 背景。
- [时间与来源](temporal-and-provenance.md) — 用于租户归属的审计日志捕获。
- [Cookbook：使用 PROV-O 和时间查询的审计跟踪](../cookbook/audit-trail.md)

## 进一步阅读

- [博客：多租户知识图](https://github.com/trickle-labs/pg-ripple/blob/main/blog/multi-tenant-knowledge-graphs.md) — SaaS 部署的隔离、配额和 RLS 模式

## 另请参阅 — 操作员

- [运维 → 安全](../operations/security.md) — RLS、命名图隔离、Bearer 身份验证
- [运维 → 高可用性](../operations/high-availability.md) — 多租户部署的故障转移、读副本