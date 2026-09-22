# 安全升级

pg_ripple 遵循 PostgreSQL 的标准扩展升级机制。每个版本都附带一个迁移脚本，`ALTER EXTENSION pg_ripple UPDATE` 会自动执行该脚本，沿着版本链从您当前的版本走到目标版本。

---

## 扩展升级的工作原理

PostgreSQL 扩展使用一系列迁移脚本在版本之间移动。pg_ripple 为每个连续的版本对提供一个脚本：

```
pg_ripple--0.1.0--0.2.0.sql
pg_ripple--0.2.0--0.3.0.sql
pg_ripple--0.3.0--0.4.0.sql
...
pg_ripple--0.45.0--0.46.0.sql
```

当您运行 `ALTER EXTENSION pg_ripple UPDATE` 时，PostgreSQL 会找到从您当前版本到最新版本的最短路径，并按顺序执行每个脚本。

```admonish info title="迁移脚本是幂等的"
每个迁移脚本都使用 `IF NOT EXISTS`、`CREATE OR REPLACE` 和类似的保护措施。如果迁移被部分应用（例如，由于崩溃），重新运行它是安全的。
```

---

## 升级前清单

### 1. 检查您当前的版本

```sql
SELECT extversion FROM pg_extension WHERE extname = 'pg_ripple';
```

### 2. 备份数据库

```bash
pg_dump -Fc -f pre_upgrade_backup.dump mydb
```

```admonish danger title="始终在升级前备份"
虽然迁移脚本经过测试，但备份允许您在出现问题时恢复到升级前的状态。这对于添加 schema 更改的主要功能发布尤为重要。
```

### 3. 查看更新日志

阅读[发布说明](../reference/changelog.md)中您当前版本和目标版本之间的每个版本。请注意：

- **破坏性更改**：重命名的函数、更改的返回类型、删除的 GUC 参数
- **Schema 更改**：内部表上的新列、新索引
- **新依赖项**：其他 `shared_preload_libraries` 条目

### 4. 检查活动连接

```sql
SELECT count(*) FROM pg_stat_activity
WHERE datname = current_database()
  AND pid != pg_backend_pid();
```

在升级之前断开所有应用程序连接。升级会修改扩展目录条目，并且可能需要对内部表的独占锁。

### 5. 验证新包已安装

必须在运行 `ALTER EXTENSION` 之前将新的 `.so`（共享库）和 SQL 迁移文件存在于 PostgreSQL 扩展目录中：

```bash
# 检查目标版本的迁移脚本是否存在
ls $(pg_config --sharedir)/extension/pg_ripple--*

# 检查共享库是否已更新
ls -la $(pg_config --pkglibdir)/pg_ripple.so
```

---

## 执行升级

### 第 1 步：安装新包

```bash
# 从源代码
cargo pgrx install --pg-config $(pg_config) --release

# 或从预构建的包
# dpkg -i pg_ripple-0.32.0-pg18.deb
```

### 第 2 步：安排维护窗口

```admonish warning title="无零停机升级"
pg_ripple 尚不支持零停机升级。在维护窗口期间安排升级。如果您有读副本，请在升级窗口期间将读取流量路由到副本，但请注意副本在升级之前也需要安装新的共享库。
```

### 第 3 步：重新启动 PostgreSQL（如果需要）

某些版本会更新共享库或更改共享内存布局。请检查发布说明 — 如果它们提到共享内存更改或新的后台工作线程，请重新启动 PostgreSQL：

```bash
pg_ctl restart -D $PGDATA
```

### 第 4 步：运行迁移

```sql
-- 升级到最新安装的版本
ALTER EXTENSION pg_ripple UPDATE;

-- 或升级到特定版本
ALTER EXTENSION pg_ripple UPDATE TO '0.46.0';
```

PostgreSQL 将按顺序执行每个中间迁移脚本：

```
NOTICE:  updating extension "pg_ripple" from version "0.44.0" to "0.45.0"
NOTICE:  updating extension "pg_ripple" from version "0.45.0" to "0.46.0"
```

### 第 5 步：验证

```sql
-- 确认新版本
SELECT extversion FROM pg_extension WHERE extname = 'pg_ripple';

-- 运行健康检查
SELECT pg_ripple.canary();

-- 验证统计信息
SELECT pg_ripple.stats();
```

---

## 升级后验证

每次升级后运行这些检查：

```sql
-- 1. 扩展版本符合预期
SELECT extversion FROM pg_extension WHERE extname = 'pg_ripple';

-- 2. 健康检查通过
SELECT pg_ripple.canary();

-- 3. 三元组计数未更改
SELECT pg_ripple.stats();

-- 4. SPARQL 查询有效
SELECT pg_ripple.sparql($$
  SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5
$$);

-- 5. 合并工作线程正在运行（如果设置了 shared_preload_libraries）
SELECT (pg_ripple.stats()->>'merge_worker_pid')::int > 0 AS merge_worker_ok;

-- 6. 字典缓存正在运行
SELECT
  (s->>'encode_cache_hits')::bigint + (s->>'encode_cache_misses')::bigint > 0
    AS cache_active
FROM pg_ripple.stats() s;
```

```admonish tip title="自动执行升级后检查"
将这些验证查询添加到在 `ALTER EXTENSION` 之后立即运行的脚本中。如果任何检查失败，该脚本可以在流量路由回升级后的实例之前提醒操作员。
```

---

## 多版本跳跃升级

PostgreSQL 自动遍历整个迁移链。从 v0.5.0 直接升级到 v0.46.0 会执行所有中间脚本：

```sql
-- 这是可行的 — PG 找到路径 0.5.0 → 0.5.1 → 0.6.0 → ... → 0.46.0
ALTER EXTENSION pg_ripple UPDATE TO '0.46.0';
```

```admonish note title="长升级链"
每个迁移脚本通常很快（毫秒到秒）。但是，在大型表上添加列或创建索引的脚本可能需要更长时间。对于非常长的跳跃（10+ 版本），预计在大型数据集上需要几分钟。在升级期间监控 `pg_stat_activity` 的锁等待。
```

---

## 回滚策略

没有内置的降级路径。如果升级导致问题：

### 选项 A：从备份恢复

```bash
# 删除升级后的数据库
dropdb mydb

# 恢复升级前的备份
createdb mydb
pg_restore -d mydb pre_upgrade_backup.dump
```

这是最安全的回滚方法 — 它将所有内容返回到确切的升级前状态。

### 选项 B：重新安装旧版本

```bash
# 安装旧的共享库
cargo pgrx install --pg-config $(pg_config) --release  # （来自旧源签出）

# 重新启动 PostgreSQL
pg_ctl restart -D $PGDATA
```

```admonish danger title="降级限制"
仅当迁移脚本未进行不可逆的 schema 更改（例如，删除列）时，重新安装旧的 `.so` 文件才有效。在依赖此方法之前，请始终检查迁移脚本内容。
```

---

## 升级 PostgreSQL 本身

升级 PostgreSQL 主要版本时（例如，17 → 18）：

1. **pg_ripple 需要 PostgreSQL 18**。不支持早期版本。
2. 照常使用 `pg_upgrade` — pg_ripple 的表和扩展元数据正确传输。
3. `pg_upgrade` 之后，验证扩展：

```bash
psql -d mydb -c "SELECT extversion FROM pg_extension WHERE extname = 'pg_ripple';"
psql -d mydb -c "SELECT pg_ripple.canary();"
```

```admonish warning title="重新编译共享库"
在 PostgreSQL 主要版本升级之后，必须根据新的 PostgreSQL 头文件重新编译 `pg_ripple.so` 共享库。旧的二进制文件将无法加载。
```

---

## 版本兼容性矩阵

| pg_ripple 版本 | PostgreSQL 版本 | 注释 |
|---|---|---|
| 0.1.0 – 0.46.0 | 18.x | 唯一受支持的版本 |
| 任何 | < 18 | 不支持 |
| 任何 | 19+ | 尚未测试 |

---

## 升级故障排除

### "no update path from version X to version Y"

扩展目录中缺少中间迁移脚本。重新安装 pg_ripple 以确保所有 SQL 文件都存在：

```bash
ls $(pg_config --sharedir)/extension/pg_ripple--*.sql | wc -l
```

### "could not open extension control file"

缺少 `pg_ripple.control` 文件。重新安装扩展。

### 迁移脚本因锁超时而失败

另一个会话在内部表上持有锁。确保在升级之前关闭所有连接，或增加 `lock_timeout`：

```sql
SET lock_timeout = '60s';
ALTER EXTENSION pg_ripple UPDATE;
```

### 共享库版本不匹配

`.so` 文件版本与 SQL 迁移目标不匹配。确保在运行 `ALTER EXTENSION` 之前安装了匹配的二进制文件：

```bash
cargo pgrx install --pg-config $(pg_config) --release
pg_ctl restart -D $PGDATA
ALTER EXTENSION pg_ripple UPDATE;
```

---

## Schema 版本标记（v0.37.0+）

从 v0.37.0 开始，每次 `ALTER EXTENSION pg_ripple UPDATE` 都会在 `_pg_ripple.schema_version` 中标记一行。您可以验证升级完整性：

```sql
SELECT version, installed_at, upgraded_from
FROM _pg_ripple.schema_version
ORDER BY installed_at DESC;
```

从 0.36.0 升级到 0.37.0 后的示例输出：

```
  version  |          installed_at          | upgraded_from 
-----------+--------------------------------+---------------
 0.37.0    | 2026-04-19 10:00:00+00         | 0.36.0
 0.36.0    | 2026-02-01 08:00:00+00         | 0.35.0
```

`diagnostic_report()` 函数还会报告当前架构版本：

```sql
SELECT value FROM pg_ripple.diagnostic_report() WHERE key = 'schema_version';
```

这在监控脚本中很有用，可以确认滚动升级已在所有副本上完成。
