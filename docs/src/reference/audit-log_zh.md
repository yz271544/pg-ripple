# 审计日志

当 `pg_ripple.audit_log_enabled = on` 时，每个 SPARQL UPDATE — `INSERT DATA`、`DELETE DATA`、`INSERT { … } WHERE`、`DELETE { … } WHERE`、`LOAD`、`CLEAR`、`MOVE`、`COPY`、`ADD` — 都会被捕获到 `_pg_ripple.audit_log`。审计日志是合规性、调试和租户归属的主力。

有关审计故事（PROV-O、RDF-star、时间点重放）的高级视图，请参阅[时间和出处](../features/temporal-and-provenance.md)。

---

## Schema

```sql
CREATE TABLE _pg_ripple.audit_log (
    id          BIGSERIAL    PRIMARY KEY,
    ts          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    role        NAME         NOT NULL DEFAULT current_user,
    txid        BIGINT       NOT NULL DEFAULT txid_current(),
    operation   TEXT         NOT NULL,    -- 'INSERT DATA' | 'DELETE DATA' | …
    query       TEXT         NOT NULL,
    triple_delta INTEGER     NOT NULL     -- 三元组计数的净变化
);
```

如果在扩展创建时间设置 `pg_ripple.audit_log_partition_monthly = on`，则该表按月分区。

---

## 配置

| GUC | 默认 | 效果 |
|---|---|---|
| `pg_ripple.audit_log_enabled` | `off` | 主开关 |
| `pg_ripple.audit_log_payload_max` | `8192` | 截断超过这么多字符的捕获查询 |
| `pg_ripple.audit_log_partition_monthly` | `off` | 在扩展创建时创建每月子分区 |

根据该数据库的合规性姿态，按数据库切换 GUC。没有全局开关 — 同一实例上的不同租户可以具有不同的策略。

---

## 查询日志

```sql
-- 按角色查看最近活动。
SELECT role, count(*) AS ops, sum(triple_delta) AS net_triples
FROM   _pg_ripple.audit_log
WHERE  ts > now() - interval '24 hours'
GROUP  BY role
ORDER  BY ops DESC;

-- 事务中的所有删除。
SELECT id, ts, role, query
FROM   _pg_ripple.audit_log
WHERE  txid = 12345678
ORDER  BY id;

-- 查找引入特定三元组的 SPARQL UPDATE。
-- （与 point_in_time() 结合，重建更改历史记录。）
SELECT id, ts, role, query
FROM   _pg_ripple.audit_log
WHERE  query ILIKE '%<https://example.org/secret>%'
ORDER  BY ts DESC
LIMIT 10;
```

---

## 保留和清理

`purge_audit_log(before TIMESTAMPTZ)` 删除旧条目：

```sql
SELECT pg_ripple.purge_audit_log(before := now() - interval '90 days');
-- 返回已删除的行数。
```

对于分区的审计日志，请改为删除整个分区：

```sql
DROP TABLE _pg_ripple.audit_log_2025_q4;
```

---

## 传送到 SIEM

审计日志是一个常规的 PostgreSQL 表 — 每个适用于表的工作机制都适用于它：

- **逻辑复制** 到集中式审计仓库。
- **外部数据包装器** 将其公开在 SIEM 内部。
- **触发器 + `pg_notify`** 将条目实时推送到 SOC 管道。
- **CDC 订阅** 与 `pg_ripple_http` 的 WebSocket 暴露一起用于实时流。

---

## 性能说明

- 日志记录在 UPDATE 语句中是同步的。成本主要由插入到 `_pg_ripple.audit_log` 主导 — 大致与一次普通三元组插入相同。
- 对于具有高 UPDATE 率的 OLTP 工作负载，启用分区并添加 `BRIN(ts)` 索引。默认的 `id PRIMARY KEY` 在大多数情况下就足够了。
- 即使 SPARQL UPDATE 本身稍后回滚，也会写入日志 — 以捕获用于取证的*尝试*的更改。使用 `pg_ripple.audit_log_only_committed = on` 仅在提交时记录。

---

## 另请参阅

- [时间和出处](../features/temporal-and-provenance.md)
- [多租户图](../features/multi-tenant-graphs.md)
- [操作 → 安全性](../operations/security.md)