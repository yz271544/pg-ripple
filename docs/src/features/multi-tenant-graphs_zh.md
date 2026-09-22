# 多租户图

多租户图存储是任何数据库中最棘手的模式之一。您有一个逻辑产品，但有许多客户，每个客户的数据必须对其他人**不可见**、**有大小界限**并且**可测量**用于计费和容量规划。pg_ripple 通过三个可组合的构建块解决了这个问题：

| 构建块 | 它提供的内容 |
|---|---|
| **命名图** | 逻辑分区 —— 每个租户住在自己的图 IRI 中 |
| **图上的行级安全性** | 租户角色仅看到自己的图；由 PostgreSQL 本身强制执行 |
| **租户配额** | 每个租户的可配置三元组计数上限，在插入时强制执行 |

这些共同为您提供了由 PostgreSQL 权限系统支持的*真正*的租户隔离 —— 没有应用程序级过滤，没有 SQL 注入绕过墙的风险。

---

## 何时使用此功能

- 每个客户都有自己的知识图谱的 SaaS 产品。
- 每个项目都被沙盒化的研究平台。
- 一个多团队数据平台，您希望共享实例而不共享数据。

如果您只需要*标记的*分区而不需要隔离（例如，"该图来自 PubMed，该图来自 Crossref"），则仅使用纯命名图就足够了。本页上的功能涉及**强制**隔离。

---

## 模型

```
   ┌──────────────────────────────────────────────────────────────┐
   │                    pg_ripple 实例                              │
   │                                                               │
   │   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
   │   │  图 A         │  │  图 B         │  │  图 C         │      │
   │   │  (acme.com)  │  │  (globex)    │  │  (shared)    │      │
   │   │  500 K trip. │  │  120 K trip. │  │  20 K trip.  │      │
   │   │  quota: 1M   │  │  quota: 250K │  │  quota: ∞    │      │
   │   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
   │          │                 │                 │              │
   │          └────► role tenant_acme   ◄─────────┤              │
   │                  role tenant_globex ◄────────┤              │
   │                                              │              │
   └──────────────────────────────────────────────┴──────────────┘
                                                  │
                                          everyone reads "shared"
```

- **图 IRI** 是隔离单位：三元组在插入时使用其图进行标记。
- **角色** 是访问控制单位：PostgreSQL 角色拥有图的授权。
- **租户** 是配额单位：每个已注册为租户的命名图都有一个三元组计数上限。

---

## 第 1 步 —— 注册租户

`create_tenant()` 将命名图注册为租户并为其分配三元组计数配额。注册后，每次插入该图都会根据配额进行检查；超额插入引发 `PT530`。

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

视图构建在触发器维护的计数器之上，因此它是 O(1) —— 可以安全地从计费作业调用。

---

## 第 2 步 —— 将图授予角色

`grant_graph(role, graph_iri)` 是图的 `GRANT … ON … TO …` 的类似物。它注册一个每角色可见性规则，pg_ripple 在每个 SPARQL 查询中强制执行。

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

`tenant_acme` 运行的 SPARQL 查询现在仅看到 `acme` 和 `shared` 中的三元组。没有应用程序级过滤器可以忘记；该规则在存储层强制执行。

---

## 第 3 步 —— （可选）将每个更改归因

将租户与[审计日志](temporal-and-provenance.md#audit-log)配对以获得计费级归因：

```sql
SET pg_ripevent.audit_log_enabled = on;

-- 过去 24 小时内每个租户的写入量：
SELECT role, count(*) AS updates
FROM _pg_ripple.audit_log
WHERE ts > now() - interval '24 hours'
GROUP BY role
ORDER BY updates DESC;
```

对于每租户*读取*量，捕获 `pg_stat_statements` 数据；pg_ripple 透明地与其集成。

---

## 操作注意事项

### 配额强制执行

强制配额的触发器与插入在同一事务中运行。如果批量加载将超过配额，**整个加载都将回滚** —— 没有部分提交。请相应地规划您的加载器：如果您希望接近配额，则分块加载，或通过 `tenant_stats()` 预先检查。

### 驱逐

配额是上限，而不是 LRU。当租户填满时，pg_ripple 不会自动驱逐旧三元组。要缩小租户：删除三元组（或调用 `clear_graph()`），计数器立即更新。

### 重命名或拆分租户

`rename_tenant(old_iri, new_iri)` 在单个事务中更新注册并重新标记每个三元组。请谨慎使用 —— 它触及图中的每个三元组。

### 备份和恢复

租户是纯 PostgreSQL 对象。`pg_dump --schema=pg_ripple --schema=_pg_ripple` 捕获所有内容。要为法律保留或迁移导出单个租户：

```sql
COPY (SELECT * FROM pg_ripple.export_quads_for_graph('https://example.org/tenants/acme'))
TO '/tmp/acme.nq';
```

---

## 失败模式和陷阱

1. **忘记授予共享图。** 没有对您的参考词汇表的读访问权的租户将看到类型作为不透明的 IRI。始终将 `https://example.org/shared`（或您的等效项）授予每个租户。
2. **授予默认图。** 默认图（图 ID `0`）*不*按租户划分。不要将租户数据放入默认图；它将对所有人可见。
3. **对所有租户使用单一角色。** 配额和 RLS 附加到角色。在租户之间共享角色会同时击败两者。
4. **预防性配额与反应性配额。** PT530 在*提交之前*引发，而不是之后。长时间运行的批量加载应拆分为批，并在每批之后检查 `tenant_stats()`。

---

## 另请参阅

- [操作 → 安全](../operations/security.md) — 更深入的 PostgreSQL RLS 背景。
- [时间与出处](temporal-and-provenance.md) — 用于租户归因的审计日志捕获。
- [Cookbook：使用 PROV-O 和时间查询的审计跟踪](../cookbook/audit-trail.md)

## 进一步阅读

- [博客：多租户知识图谱](https://github.com/trickle-labs/pg-ripple/blob/main/blog/multi-tenant-knowledge-graphs.md) — SaaS 部署的隔离、配额和 RLS 模式