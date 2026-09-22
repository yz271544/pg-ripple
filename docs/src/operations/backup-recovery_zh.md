# 备份和灾难恢复

pg_ripple 将所有数据存储在 `_pg_ripple` schema 内的标准 PostgreSQL 表中。
这意味着**每个 PostgreSQL 备份工具都可以开箱即用** — VP 表、字典、谓词目录、SHACL 约束、Datalog 规则和推断的三元组都被 `pg_dump`、WAL 归档和流复制捕获。

```admonish info title="不需要特殊导出"
与需要单独的 RDF 转储/重新加载周期的三元组存储不同，pg_ripple 数据就是 PostgreSQL 数据。您现有的备份基础设施已经涵盖了它。
```

---

## 备份的内容

| 对象 | Schema | 由 pg_dump 捕获？ | 注释 |
|---|---|---|---|
| 字典表 | `_pg_ripple.dictionary` | 是 | 所有 IRI、空节点和字面量映射 |
| 谓词目录 | `_pg_ripple.predicates` | 是 | 谓词 → VP 表 OID 映射 |
| VP 表（main + delta + tombstones）| `_pg_ripple.vp_{id}_*` | 是 | 每个谓词一个表集 |
| 稀有谓词表 | `_pg_ripple.vp_rare` | 是 | 合并的低基数谓词 |
| SHACL 约束 | `_pg_ripple.shacl_*` | 是 | Shape 定义和验证状态 |
| Datalog 规则 | `_pg_ripple.rules` | 是 | 规则文本和已编译的计划 |
| 推断的三元组 | VP 表，`source = 1` | 是 | 物化的推理结果 |
| 扩展元数据 | `pg_catalog` | 是 | 扩展版本和控制文件 |
| 共享内存状态 | 仅内存 | **否** | 字典 LRU 缓存、合并工作线程计数器 |

```admonish note title="共享内存状态"
字典 LRU 缓存和合并工作线程计数器位于共享内存中，**不**持久化到磁盘。它们在 PostgreSQL 重新启动时自动重建。这是有意设计的 — 缓存从正常查询流量中快速预热。
```

---

## 使用 pg_dump 进行逻辑备份

### 全数据库转储

```bash
# 自定义格式（推荐 — 压缩、可并行恢复）
pg_dump -Fc -f pg_ripple_backup.dump mydb

# 纯 SQL（人类可读，用于审计）
pg_dump -Fp -f pg_ripple_backup.sql mydb
```

### 仅扩展转储

仅备份 pg_ripple 数据而不备份数据库的其余部分：

```bash
pg_dump -Fc \
  --schema=_pg_ripple \
  --schema=pg_ripple \
  -f pg_ripple_only.dump mydb
```

```admonish warning title="包括两个 schema"
始终包括**两个** `_pg_ripple`（内部存储）和 `pg_ripple`（公共 API 函数）。仅恢复其中一个会使扩展处于不一致状态。
```

### 大型数据集的并行转储

对于具有数百万三元组的数据库，使用并行工作线程：

```bash
# 并行转储需要目录格式
pg_dump -Fd -j 4 -f pg_ripple_backup_dir/ mydb
```

字典表和大型 VP 表将并行转储，显著减少备份时间。

---

## 从备份恢复

### 全恢复到新数据库

```bash
# 创建目标数据库
createdb mydb_restored

# 恢复（自定义格式）
pg_restore -d mydb_restored -Fc pg_ripple_backup.dump

# 恢复（目录格式，并行）
pg_restore -d mydb_restored -Fd -j 4 pg_ripple_backup_dir/
```

### 从纯 SQL 恢复

```bash
psql -d mydb_restored -f pg_ripple_backup.sql
```

### 恢复后验证

恢复后，验证扩展是否完好：

```sql
-- 检查扩展版本
SELECT extversion FROM pg_extension WHERE extname = 'pg_ripple';

-- 验证三元组计数
SELECT pg_ripple.stats();

-- 运行健康检查
SELECT pg_ripple.canary();

-- 抽查 SPARQL 查询
SELECT pg_ripple.sparql($$
  SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }
$$);
```

```admonish tip title="VP 表在转储/恢复后是否幸存？"
**是的。** VP 表是带有 B-tree 或 BRIN 索引的标准 PostgreSQL 堆表。`pg_dump` 像捕获任何其他表一样捕获它们。HTAP delta/main/tombstone 分割、索引和合并工作线程视图定义都被保留。恢复后，一旦 `shared_preload_libraries` 包含 `pg_ripple`，合并工作线程就会恢复正常操作。
```

---

## 基于 WAL 的连续归档

对于时间点恢复（PITR），配置 WAL 归档：

### 启用 WAL 归档

在 `postgresql.conf` 中：

```ini
wal_level = replica
archive_mode = on
archive_command = 'cp %p /backup/wal_archive/%f'
max_wal_senders = 3
```

### 进行基础备份

```bash
pg_basebackup -D /backup/base -Ft -z -P
```

### 时间点恢复

创建一个 `recovery.signal` 文件并配置恢复目标：

```ini
# postgresql.conf（或 postgresql.auto.conf）
restore_command = 'cp /backup/wal_archive/%f %p'
recovery_target_time = '2026-04-19 14:30:00'
```

启动 PostgreSQL — 它将重放 WAL 到指定的时间。

```admonish warning title="HTAP 合并和 PITR"
如果您恢复到合并中间的点，合并工作线程将在启动时检测到不完整的状态并重新运行合并。无需手动干预，但恢复后的第一个合并循环可能比平时花费更长的时间。
```

---

## 流复制

pg_ripple 与 PostgreSQL 流复制透明地配合使用：

```bash
# 在副本上
pg_basebackup -h primary-host -D /var/lib/postgresql/18/main -R -P
```

`-R` 标志写入 `standby.signal` 和连接参数。所有 VP 表、字典数据和 HTAP 状态通过 WAL 复制。

```admonish note title="副本上的合并工作线程"
后台合并工作线程**不会**在只读副本上运行。副本通过来自主服务器的 WAL 重放接收合并状态。这是正确的行为 — 副本永远不应写入。
```

---

## 备份策略建议

### 小型数据集（< 1M 三元组）

| 组件 | 建议 |
|---|---|
| 方法 | 每晚 `pg_dump -Fc` |
| 保留 | 7 个每日 + 4 个每周 |
| RPO | 24 小时 |
| RTO | 分钟 |

### 中型数据集（1M – 100M 三元组）

| 组件 | 建议 |
|---|---|
| 方法 | WAL 归档 + 每日基础备份 |
| 保留 | 7 个每日基础 + 连续 WAL |
| RPO | 秒（WAL）|
| RTO | 分钟到小时 |

### 大型数据集（> 100M 三元组）

| 组件 | 建议 |
|---|---|
| 方法 | WAL 归档 + pgBackRest 或 Barman |
| 保留 | 增量基础 + 连续 WAL |
| RPO | 秒（WAL）|
| RTO | 与数据集大小成正比 |

```admonish tip title="测试您的恢复"
安排每月恢复演练。从未测试过的备份不是备份。将上面显示的验证查询自动化，作为演练的一部分。
```

---

## 灾难恢复清单

1. **灾难发生之前**：WAL 归档已启用，基础备份按计划进行，复制滞后已监控
2. **事件期间**：确定故障范围（单个表、完整数据库或主机丢失）
3. **恢复步骤**：
   - 主机丢失 → 升级副本或从基础备份 + WAL 恢复
   - 损坏 → PITR 到最后一个已知良好时间
   - 意外删除 → PITR 到 DROP/DELETE 之前
4. **恢复后**：
   - 运行 `SELECT pg_ripple.canary()` 验证健康状况
   - 检查 `pg_ripple.stats()` 以获取预期的三元组计数
   - 验证合并工作线程是否正在运行（`merge_worker_pid > 0`）
   - 运行代表性的 SPARQL 查询以确认数据完整性
   - 恢复 WAL 归档和复制

---

## 常见陷阱

```admonish danger title="不要忘记 shared_preload_libraries"
恢复到新的 PostgreSQL 实例后，确保在**启动**服务器之前在 `postgresql.conf` 中设置 `shared_preload_libraries = 'pg_ripple'`。没有它，合并工作线程将不会启动，字典缓存将不可用，并且查询将回退到未缓存的字典查找。
```

- **Schema 所有权**：恢复用户必须是超级用户或拥有 `_pg_ripple` 和 `pg_ripple` schema
- **序列值**：`pg_dump` 捕获序列状态 — 恢复后语句 ID（`i` 列）将从正确的值继续
- **表空间放置**：如果您为 VP 表使用了自定义表空间，请确保它们在恢复之前已存在于目标服务器上
