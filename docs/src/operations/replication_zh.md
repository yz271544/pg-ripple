# 逻辑复制

pg_ripple v0.54.0 增加了 RDF 逻辑复制：主数据库使用 PostgreSQL 的内置逻辑解码基础设施
将 RDF 三元组变更近实时流式传输到一个或多个副本数据库。

## 架构

```
┌──────────────────────────────────────────────────────────────────┐
│  PRIMARY                                                          │
│  ┌──────────────┐   INSERT/DELETE   ┌────────────────────────┐   │
│  │  VP delta    │ ──────────────→  │  WAL (logical decoding)│   │
│  │  tables      │                   │  slot: pg_ripple_sub   │   │
│  └──────────────┘                   └───────────┬────────────┘   │
└─────────────────────────────────────────────────│────────────────┘
                                                  │ streaming replication
┌─────────────────────────────────────────────────│────────────────┐
│  REPLICA                                         ↓               │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  _pg_ripple.replication_status (pending batches)         │    │
│  └─────────────────────────┬────────────────────────────────┘    │
│                             ↓ logical_apply_worker               │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  pg_ripple.load_ntriples() — applies triples in order    │    │
│  └──────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

逻辑解码槽（`pg_ripple_sub`）捕获对 `_pg_ripple` schema 的 VP delta 表的每个 `INSERT` 和 `DELETE`。
变更被解码为 N-Triples 格式并写入副本上的 `_pg_ripple.replication_status`。
`logical_apply_worker` 后台进程（当 `pg_ripple.replication_enabled = on` 时启用）
读取这些待处理的批次并通过 `load_ntriples()` 应用它们。

## 设置演练

### 1. 主服务器配置

在主服务器的 `postgresql.conf` 中：

```conf
wal_level = logical
max_replication_slots = 4
max_wal_senders = 4
```

在 `CREATE EXTENSION pg_ripple` 之后创建发布：

```sql
CREATE PUBLICATION pg_ripple_pub
  FOR ALL TABLES IN SCHEMA _pg_ripple;
```

### 2. 副本配置

在副本的 `postgresql.conf` 中添加：

```conf
pg_ripple.replication_enabled = on
pg_ripple.replication_conflict_strategy = 'last_writer_wins'
```

在副本上创建扩展和订阅：

```sql
CREATE EXTENSION pg_ripple;

CREATE SUBSCRIPTION pg_ripple_sub
  CONNECTION 'host=primary-host port=5432 dbname=mydb user=replication password=secret'
  PUBLICATION pg_ripple_pub;
```

### 3. 验证复制是否正在运行

```sql
SELECT * FROM pg_ripple.replication_stats();
```

```
   slot_name    | lag_bytes | last_applied_lsn | last_applied_at
----------------+-----------+------------------+---------------------
 pg_ripple_sub  |         0 | 0/15000A8        | 2026-04-24 12:00:01
```

`lag_bytes` 为 0 意味着副本已完全赶上。

## 延迟监控

随时查询以字节为单位的复制延迟：

```sql
SELECT slot_name, lag_bytes
FROM pg_ripple.replication_stats()
WHERE lag_bytes > 1000000;  -- 落后超过 1 MB 时告警
```

通过 `pg_stat_statements` 或内置 OTEL 跟踪端点
（`pg_ripple.tracing_otlp_endpoint`）与 Prometheus 集成。

## 冲突解决

`last_writer_wins` 策略（v0.54.0 中唯一的策略）当两个副本收到相同的 `(s, p, g)`
三元组但对象不同时，保留具有最高语句 ID（SID）的三元组。
这与典型 RDF 工作负载的最终一致性语义相匹配。

通过以下方式设置策略：

```sql
SET pg_ripple.replication_conflict_strategy = 'last_writer_wins';
```

## 故障转移过程

1. 停止对主服务器的写入（或等待副本延迟达到 0）。
2. 升级副本：`pg_ctl promote -D /var/lib/postgresql/18/main`
3. 更新应用程序连接字符串以指向新的主服务器。
4. 在任何新副本上重新创建复制订阅。

## GUC 参考

| GUC | 默认 | 描述 |
|-----|---------|-------------|
| `pg_ripple.replication_enabled` | `off` | 启用逻辑应用工作线程 |
| `pg_ripple.replication_conflict_strategy` | `last_writer_wins` | 冲突解决策略 |

另请参阅完整的 [GUC 参考](../reference/guc-reference.md)。
