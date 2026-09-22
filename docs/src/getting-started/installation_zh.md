# 安装

pg_ripple 是一个用 Rust 编写的 PostgreSQL 18 扩展。选择适合您环境的安装方法。

## Docker（推荐）

```admonish tip title="最快路径 — 无需构建工具"
`docker compose up -d` 可在不到一分钟内启动一个可用的 pg_ripple 实例。不需要 Rust 编译器，也不需要 PostgreSQL 开发头文件 —— 只需 Docker。
```

启动可用的 pg_ripple 实例的最快路径。无需构建工具。

```bash
# 使用 Docker Compose 启动 pg_ripple
docker compose up -d

# 连接
psql -h localhost -p 5432 -U postgres -d pg_ripple
```

仓库根目录中的 `docker-compose.yml` 启动预装了 pg_ripple 的 PostgreSQL 18，并在默认数据库中创建该扩展。

### 验证安装

```sql
SELECT pg_ripple.triple_count();
```

结果应为 `0` —— 扩展已安装并就绪。

## 从源代码（cargo pgrx）

直接构建并安装到本地 PostgreSQL 18 实例。

### 先决条件

- Rust（stable，edition 2024）
- PostgreSQL 18 开发头文件
- `cargo-pgrx` 0.18

```bash
# 安装 cargo-pgrx
cargo install cargo-pgrx --version 0.18 --locked

# 使用 PostgreSQL 18 初始化 pgrx
cargo pgrx init --pg18 $(which pg_config)

# 构建并安装
cargo pgrx install --release --pg-config $(which pg_config)
```

### 创建扩展

连接到您的数据库并运行：

```sql
CREATE EXTENSION pg_ripple;
```

### 验证

```sql
SELECT pg_ripple.triple_count();
```

## 配置

pg_ripple 使用默认设置开箱即用。对于生产部署，您可能希望调整 GUC 参数 — 请参阅[配置和调优](../operations/configuration.md)。

对于 HTAP 存储（后台合并工作线程）和共享内存字典缓存，请将 pg_ripple 添加到 `postgresql.conf` 中的 `shared_preload_libraries`：

```
shared_preload_libraries = 'pg_ripple'
```

在此更改后重新启动 PostgreSQL。

## 故障排除

### PostgreSQL 版本错误

pg_ripple 需要 PostgreSQL 18。检查您的版本：

```bash
pg_config --version
```

### 缺少 shared_preload_libraries

如果您看到关于共享内存或合并工作线程未启动的错误，请确保 `pg_ripple` 在 `shared_preload_libraries` 中，并且 PostgreSQL 已重新启动。

### pgrx 版本不匹配

pg_ripple 需要 `cargo-pgrx` 0.18。如果您有较旧的版本：

```bash
cargo install cargo-pgrx --version 0.18 --locked --force
```

### 安装后找不到扩展

如果 `CREATE EXTENSION pg_ripple` 失败并显示"extension not found"，请验证扩展文件是否已安装到正确的 PostgreSQL 目录：

```bash
pg_config --sharedir
ls $(pg_config --sharedir)/extension/pg_ripple*
```

### Docker 容器启动失败

检查日志：

```bash
docker compose logs pg_ripple
```

常见原因：端口 5432 已被使用（更改端口映射）、内存不足（pg_ripple 建议至少 512MB）。

## 下一步

- [Hello World — 五分钟演练](hello-world.md) — 加载和查询您的第一批三元组
- [引导式教程](tutorial.md) — 在 30 分钟内构建知识图谱