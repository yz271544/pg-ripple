# 生产就绪清单

在将 pg_ripple 部署到生产环境之前，请使用此清单。每个项目链接到相关文档以获取详细信息。

---

## PostgreSQL 配置

- [ ] **PostgreSQL 18** 已安装 — pg_ripple 需要 PostgreSQL 18.x
- [ ] **`shared_preload_libraries`** 包含 `'pg_ripple'` — 后台合并工作线程和共享内存字典缓存所需（[配置](configuration.md)）
- [ ] **`pg_ripple.worker_database`** 设置为目标数据库 — 合并工作线程连接到此数据库（[合并工作线程](merge-workers.md)）
- [ ] **共享内存** 大小正确 — `pg_ripple.dictionary_cache_size` 决定共享内存使用量；检查操作系统限制（[故障排除 §6](troubleshooting.md)）
- [ ] **PostgreSQL 已重启** 在更改 `shared_preload_libraries` 之后

## 安全

- [ ] **行级安全 (RLS)** 在多租户命名图上启用 — `pg_ripple.enable_graph_rls()` + 角色授权（[安全](security.md)、[多租户图](../features/multi-tenant-graphs.md)）
- [ ] **联邦 SSRF 保护** 已配置 — `pg_ripple.federation_allow_private = off`（默认）防止对私有 IP 的 SERVICE 查询（[GUC 参考](../reference/guc-reference.md)）
- [ ] **`pg_ripple_http` 认证令牌** 已设置 — `PG_RIPPLE_HTTP_AUTH_TOKEN` 环境变量用于 Bearer 令牌认证（[HTTP API 参考](../reference/http-api.md)）
- [ ] **TLS 终止** 已配置 — 使用反向代理（nginx、Caddy）进行 HTTPS；pg_ripple_http 不直接处理 TLS
- [ ] **`pg_hba.conf`** 限制到 pg_ripple_http 服务账户的连接
- [ ] **嵌入 API 密钥** 未存储在 `postgresql.conf` 中 — 使用 `ALTER SYSTEM` 或通过会话 `SET` 注入（[GUC 参考](../reference/guc-reference.md)）

## 性能

- [ ] **合并工作线程** 已调优 — `pg_ripple.merge_workers` = 2–4 用于具有许多谓词的工作负载（[合并工作线程](merge-workers.md)）
- [ ] **字典缓存** 大小适应工作集 — 通过 `pg_ripple.stats()` 监控 `encode_cache_evictions`，目标命中率 > 90%（[故障排除 §7](troubleshooting.md)）
- [ ] **Autovacuum** 已为 VP 表调优 — 在高 churn delta 表上考虑 `autovacuum_vacuum_scale_factor = 0.01`（[性能](performance.md)）
- [ ] **`work_mem`** 足以支持 SPARQL 生成的 SQL — 大型查询 64–256 MB
- [ ] **属性路径深度** 有界 — `pg_ripple.max_path_depth` 防止失控的递归（默认：10）（[故障排除 §3](troubleshooting.md)）

## 监控

- [ ] **Prometheus 指标** 已配置 — `pg_ripple_http` 公开 `/metrics` 端点（[监控](monitoring.md)）
- [ ] **关键指标已监控**：
  - `pg_ripple_triple_count` — 存储的三元组总数
  - `pg_ripple_merge_worker_lag` — 合并积压
  - `pg_ripple_dictionary_cache_hit_rate` — 编码效率
  - `pg_ripple_sparql_query_duration_seconds` — 查询延迟
- [ ] **健康检查** 已配置 — `GET /health` 和 `GET /health/ready` 用于负载均衡器探针
- [ ] **基于日志的告警** PT 系列错误代码 — 请参阅[错误目录](../reference/error-catalog.md)

## 备份和恢复

- [ ] **`pg_dump`** 已测试 — pg_ripple 将所有数据存储在标准 PostgreSQL 表中；`pg_dump`/`pg_restore` 无需特殊步骤即可工作（[备份](backup-recovery.md)）
- [ ] **WAL 归档** 已启用以进行时间点恢复
- [ ] **备份计划** 已记录并经过恢复测试

## v0.133.0 弹性资格运行手册

从仓库根目录运行资格命令。所需的矩阵使用一次性 PostgreSQL 18
实例并重用已建立的崩溃恢复测试。它是确定性的、有界的，
并在首次失败时停止。

指定的弹性作业仅运行下面的静态 `--validate` 检查。
迁移资格作业还执行实时中断升级回滚、postmaster `SIGKILL` 和重启、
逻辑恢复、物理基础备份恢复和有界资源压力运行。
它上传原始日志和环境详细信息保留 90 天。
PITR、HA 升级和其余崩溃场景仍是操作员运行的程序，
除非保留它们的工件，否则不作为发布证据。

```bash
# CI 安全：不需要 PostgreSQL 连接、集群或外部变更。
bash tests/resilience/fault_matrix.sh --validate
find tests -type f -name '*.sh' -print0 | xargs -0 -n1 bash -n
python3 scripts/check_docs_links.py
python3 scripts/check_docs_summary.py
```

```bash
# 必需的崩溃、重启和逻辑恢复矩阵。仅使用一次性集群，
# 因为旧的崩溃测试会故意删除其测试数据库。
export PGDATA=/path/to/disposable/pgdata
export PGHOST=/path/to/disposable/socket
export PGPORT=28818
export PGUSER=postgres
export PGDATABASE=postgres
export PG_RIPPLE_RESILIENCE_ALLOW_DESTRUCTIVE=1
export PG_RIPPLE_RESILIENCE_RUN_ID=v0133
bash tests/resilience/fault_matrix.sh --all
```

驱动程序按以下固定顺序运行这些案例：

| 场景 | 命令 | 恢复断言 |
|---|---|---|
| `load-sigkill-restart` | `bash tests/resilience/fault_matrix.sh --scenario load-sigkill-restart` | 字典和崩溃后加载保持可用 |
| `merge-sigkill-restart` | `bash tests/resilience/fault_matrix.sh --scenario merge-sigkill-restart` | 重启后 main/delta/tombstone 状态可查询 |
| `promotion-sigkill-restart` | `bash tests/resilience/fault_matrix.sh --scenario promotion-sigkill-restart` | 升级恢复不会留下卡住的目录状态 |
| `writeback-sigkill-restart` | `bash tests/resilience/fault_matrix.sh --scenario writeback-sigkill-restart` | 可以再次实例化 writeback 视图 |
| `inference-sigkill-restart` | `bash tests/resilience/fault_matrix.sh --scenario inference-sigkill-restart` | 可以在没有损坏的派生状态下重新运行推理 |
| `upgrade-sigkill-restart` | `bash tests/resilience/fault_matrix.sh --scenario upgrade-sigkill-restart` | 失败的升级回滚；SIGKILL/重启保留数据 |
| `logical-dump-restore` | `bash tests/resilience/fault_matrix.sh --scenario logical-dump-restore` | `pg_dump`/`pg_restore` 保留三元组和健康状况 |

如果安装的候选版本是使用仅测试的故障注入功能构建的，
则在启动 PostgreSQL 之前设置 `PG_RIPPLE_FAULT_INJECTION` 以选择
确定性钩子，例如 `merge_phase_start=terminate`；
终止钩子只能在一次性实例上运行。

### 物理备份和 PITR

物理脚本是选择加入的，因为它启动恢复的 PostgreSQL 集群
并使用配置的主数据库作为备份源。它在备份、恢复、归档或健康错误时失败；
仅当明确未启用时才跳过。

```bash
export PG_RIPPLE_RUN_PHYSICAL=1
bash tests/resilience/fault_matrix.sh --scenario physical-backup-restore-pitr

# 仅当主服务器具有 archive_mode=on 且 archive_command
# 写入提供的目录时，添加这些。
export PG_RIPPLE_RUN_PITR=1
export PG_RIPPLE_WAL_ARCHIVE_DIR=/path/to/wal-archive
bash tests/resilience/fault_matrix.sh --scenario physical-backup-restore-pitr
```

### 主/备升级

升级是拓扑变更，需要先隔离旧主服务器。
harness 需要本地数据目录路径以便执行该隔离；
升级后它不会尝试重新加入或覆盖任一集群。

```bash
export PG_RIPPLE_RUN_HA=1
export PG_RIPPLE_RESILIENCE_ALLOW_TOPOLOGY_CHANGE=1
export PG_RIPPLE_PRIMARY_URL='postgresql://postgres@primary/postgres'
export PG_RIPPLE_STANDBY_URL='postgresql://postgres@standby/postgres'
export PG_RIPPLE_PRIMARY_DATA=/path/to/primary/pgdata
export PG_RIPPLE_STANDBY_DATA=/path/to/standby/pgdata
bash tests/resilience/fault_matrix.sh --scenario primary-standby-promotion
```

### 读副本写入安全性

此检查对副本是只读的，除了故意拒绝的 `insert_triple` 调用。
连接失败是测试失败，而不是只读通过。

```bash
export PG_RIPPLE_RUN_HA=1
export PG_RIPPLE_READ_REPLICA_URL='postgresql://postgres@replica/postgres'
bash tests/resilience/fault_matrix.sh --scenario read-replica-write-safety
```

### 资源压力

压力检查更改会话本地的 `statement_timeout`、`work_mem` 和
`temp_file_limit`，然后验证有界属性路径和取消诊断。
行数和超时是可配置的，但由脚本限制。

```bash
export PG_RIPPLE_RUN_RESOURCE_PRESSURE=1
export PG_RIPPLE_RESOURCE_ROWS=2000
export PG_RIPPLE_RESOURCE_TIMEOUT_MS=1500
export PG_RIPPLE_RESOURCE_WORK_MEM=1MB
bash tests/resilience/fault_matrix.sh --scenario resource-pressure
```

必需的场景必须以零退出。物理/PITR 和 HA 命令是
环境门控的：`SKIP` 仅在其选择加入变量不存在时才有效；
一旦启用，缺少先决条件或失败的断言都是失败。

## 升级路径

- [ ] **迁移脚本** 已验证 — `ALTER EXTENSION pg_ripple UPDATE` 应用顺序迁移脚本（[升级](upgrading.md)）
- [ ] **兼容性矩阵** 已检查 — 如果使用 `pg_ripple_http`，请验证版本兼容性（[兼容性](compatibility.md)）
- [ ] **在生产升级之前在暂存中测试** — 针对新版本运行 `cargo pgrx regress` 或 pg_regress 套件

## 可选组件

- [ ] **pgvector** 已安装（如果使用向量/混合搜索）— `CREATE EXTENSION vector`（[向量搜索](../features/vector-and-hybrid-search.md)）
- [ ] **pg_trickle** 已安装（如果使用实时视图）；**pg_tide** 已安装（如果使用 CDC 桥接或中继发件箱）—（[CDC 运维](cdc.md)）
- [ ] **PostGIS** 已安装（如果使用 GeoSPARQL）—（[GeoSPARQL](../features/geospatial.md)）

## 冒烟测试

部署后，验证扩展是否正常工作：

```sql
-- 检查扩展版本
SELECT pg_ripple.build_info();

-- 验证合并工作线程正在运行
SELECT (pg_ripple.stats()->>'merge_worker_pid')::int > 0 AS merge_worker_alive;

-- 验证字典缓存处于活动状态
SELECT pg_ripple.stats()->>'encode_cache_capacity' AS cache_capacity;

-- 加载测试三元组并查询
SELECT pg_ripple.insert_triple(
    'http://example.org/test',
    'http://example.org/status',
    '"production-ready"'
);
SELECT * FROM pg_ripple.sparql($$
    SELECT ?status WHERE {
        <http://example.org/test> <http://example.org/status> ?status
    }
$$);
```
