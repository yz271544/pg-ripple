# 高可用性决策树

本页帮助您为 pg_ripple 部署选择正确的 HA 拓扑。

## 决策树

```
您需要跨多个节点的亚秒级读可扩展性吗？
├─ 是 → 使用流复制（主 + 读副本）
│        + pg_ripple 逻辑复制用于 RDF 特定的应用
└─ 否 → 单节点配合良好硬件可能就足够了

您在 Kubernetes 上运行吗？
├─ 是 → 使用 CloudNativePG operator（参见 cloudnativepg.md）
│        或 pg_ripple Helm chart（参见 kubernetes.md）
└─ 否 → 自管理 PostgreSQL 与流复制

您需要副本对 RDF 数据运行 SPARQL 查询吗？
├─ 是 → 在副本上启用 pg_ripple.replication_enabled = on
│        这样逻辑应用工作线程保持字典 + VP 表同步
└─ 否 → 标准 PostgreSQL 流复制就足够了
         （副本仍然可以通过 PG 的内置机制提供 SELECT 查询）
```

## 支持的拓扑

### 1. 单节点

适用于最多约 5000 万三元组和中等写入率的工作负载。无 HA — 使用
pg_ripple 内置的 WAL + 定期备份来实现持久性。

```
  [Client] → [pg_ripple primary]
```

### 2. 流复制 + RDF 逻辑应用

推荐用于生产 HA 的拓扑。PostgreSQL 流复制保持副本逐字节相同。
pg_ripple 的逻辑应用工作线程额外地将 VP 表的变更解码为 N-Triples
并重新应用它们，使副本的字典和 VP 表可以独立查询。

```
  [Writes] → [pg_ripple primary] ──streaming──→ [pg_ripple replica]
                                  ──logical──→  [logical_apply_worker]
```

**要求：**
- 主服务器上 `wal_level = logical`
- 副本上 `pg_ripple.replication_enabled = on`
- 每个副本一个复制槽

**延迟目标：** 在 10 k-triple/s 插入速率下 < 1 秒。

### 3. CloudNativePG

Kubernetes 环境的推荐拓扑。CNP 自动管理主服务器选举、故障转移和滚动升级。
使用扩展镜像卷以避免维护自定义 PostgreSQL 容器。

```
  [Writes] → [CNP primary Pod] ── CNP streaming ──→ [CNP standby Pods ×2]
```

**要求：** CloudNativePG operator ≥ 1.24。
有关设置，请参阅 [cloudnativepg.md](cloudnativepg.md)。

### 4. 多区域联邦查询

对于全球分布的数据，每个区域保留单独的 pg_ripple 实例，并使用 SPARQL `SERVICE`
联邦进行跨区域查询：

```sparql
SELECT ?s ?p ?o WHERE {
  SERVICE <https://us.example.com/sparql> { ?s ?p ?o }
  UNION
  SERVICE <https://eu.example.com/sparql> { ?s ?p ?o }
}
```

每个区域实例通过拓扑 2 或 3 独立实现 HA。

## 权衡

| 拓扑 | 设置复杂度 | 故障转移 RTO | 写入横向扩展 | 副本上的 SPARQL |
|----------|-----------------|-------------|-----------------|-------------------|
| 单节点 | 低 | N/A（手动恢复）| 否 | N/A |
| 流复制 + 逻辑应用 | 中 | 约 30 秒（手动故障转移）| 否 | 是 |
| CloudNativePG | 中 | 约 10 秒（自动） | 否 | 是 |
| 多区域联邦 | 高 | 每个区域 | 是（写入本地）| 是 |

## pg_ripple 与标准 PG 流复制

标准 PostgreSQL 流复制复制原始 WAL 字节 — 包括内部存储格式细节。
这对于读副本来说已经足够了，但如果没有逻辑应用工作线程
（它从解码的 N-Triples 变更重建字典和 VP 表），
副本的 pg_ripple 状态可能无法独立通过 SPARQL 查询。

在副本上启用 `pg_ripple.replication_enabled = on` 以激活逻辑应用工作线程
并确保完整的 SPARQL 查询能力。

## 监控复制延迟

```sql
-- 在副本上
SELECT * FROM pg_ripple.replication_stats();

-- 在主服务器上（标准 PG 视图）
SELECT slot_name, active, lag
FROM pg_replication_slots;
```

当 `lag_bytes > 10 MB`（通常写入率下约 1–2 秒）时设置告警。
