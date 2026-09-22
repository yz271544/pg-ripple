# 部署模型

pg_ripple 作为 PostgreSQL 18 扩展运行。它可以部署在任何支持 PostgreSQL 18 并支持扩展加载的环境中。本页涵盖三种主要部署模型，并提供评估和有界试点的配置示例。

---

## 部署选项一览

| 模型 | 最适合 | 复杂性 | SPARQL 协议 |
|---|---|---|---|
| 独立 PostgreSQL | 生产、现有的 PG 基础设施 | 低 | 通过 `pg_ripple_http` sidecar |
| Docker / Compose | 评估、CI/CD、小型部署 | 低 | 内置 |
| 托管 PostgreSQL | 云原生、最少运维 | 中 | 通过 `pg_ripple_http` sidecar |

```admonish tip title="推荐"
评估和开发使用 **Docker Compose**。生产工作负载使用**专用 PostgreSQL 18 实例** — 这可以完全控制共享内存、后台工作线程和存储配置。
```

---

## 模型 1：独立 PostgreSQL

将 pg_ripple 安装到标准 PostgreSQL 18 实例中。这是推荐的生产部署。

### 先决条件

- 从包或源代码安装的 PostgreSQL 18.x
- Rust 工具链（用于从源代码构建）或预构建的 `.so`/`.dylib`
- `pgrx` 0.18（如果从源代码构建）

### 安装

```bash
# 从源代码构建并安装
cargo pgrx install --pg-config $(which pg_config) --release

# 或者使用特定的 PG18 二进制文件
cargo pgrx install --pg-config /usr/lib/postgresql/18/bin/pg_config --release
```

### PostgreSQL 配置

添加到 `postgresql.conf`：

```ini
# 必需：在服务器启动时加载 pg_ripple 以支持后台工作线程和共享内存
shared_preload_libraries = 'pg_ripple'

# 字典缓存的共享内存（根据数据集调整）
pg_ripple.dictionary_cache_size = 65536   # 64K 条目（默认）
pg_ripple.cache_budget = 64               # MB（默认）

# HTAP 合并工作线程
pg_ripple.merge_threshold = 10000
pg_ripple.merge_interval_secs = 60
pg_ripple.worker_database = 'mydb'        # 合并工作线程连接的数据库
```

### 启用扩展

```sql
CREATE EXTENSION pg_ripple;

-- 验证安装
SELECT pg_ripple.stats();
```

### 添加 SPARQL 协议端点

SPARQL 协议 HTTP 端点由 `pg_ripple_http` 提供，这是一个独立的伴随服务：

```bash
# 构建 HTTP 服务
cd pg_ripple_http
cargo build --release

-- 运行它
PG_RIPPLE_HTTP_PG_URL="postgresql://user:pass@localhost/mydb" \
PG_RIPPLE_HTTP_PORT=7878 \
./target/release/pg_ripple_http
```

```admonish info title="pg_ripple_http 是可选的"
您可以通过 SQL 完全使用 pg_ripple — `pg_ripple.sparql()`、`pg_ripple.insert_triple()` 等。HTTP 服务为 Yasgui、RDF4J 等工具或其他端点的联邦查询添加 W3C SPARQL 协议兼容性。
```

---

## 模型 2：Docker / Docker Compose

Docker 部署将 PostgreSQL 18、pg_ripple 和 pg_ripple_http 打包到由 Docker Compose 管理的容器中。这是入门最快的方法。

### docker-compose.yml

```yaml
# pg_ripple 与 SPARQL 协议 HTTP 端点的 Docker Compose。
#
# 用法：
#   docker compose up -d
#   curl http://localhost:7878/health
#   curl -G http://localhost:7878/sparql \
#     --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 10"

services:
  postgres:
    build: .
    ports:
      - "5432:5432"
    environment:
      POSTGRES_PASSWORD: ripple
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  sparql:
    build: .
    entrypoint: ["/usr/local/bin/pg_ripple_http"]
    ports:
      - "7878:7878"
    environment:
      PG_RIPPLE_HTTP_PG_URL: "postgresql://postgres:ripple@postgres/postgres"
      PG_RIPPLE_HTTP_PORT: "7878"
      PG_RIPPLE_HTTP_POOL_SIZE: "8"
      PG_RIPPLE_HTTP_CORS_ORIGINS: "*"
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  pgdata:
```

### 启动堆栈

```bash
docker compose up -d

# 等待健康检查
docker compose ps

# 测试 SPARQL 端点
curl http://localhost:7878/health

# 运行查询
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 5"
```

### 通过 Docker 加载数据

```bash
# 将 Turtle 文件复制到容器中并加载
docker compose cp data.ttl postgres:/tmp/data.ttl
docker compose exec postgres psql -U postgres -c \
  "SELECT pg_ripple.load_turtle_file('/tmp/data.ttl');"

# 或内联加载
docker compose exec postgres psql -U postgres -c \
  "SELECT pg_ripple.load_turtle('@prefix ex: <http://example.org/> .
    ex:Alice ex:knows ex:Bob .
    ex:Bob ex:age \"30\"^^<http://www.w3.org/2001/XMLSchema#integer> .');"
```

### Docker 的生产强化

对于生产 Docker 部署，添加资源限制和持久配置：

```yaml
services:
  postgres:
    build: .
    ports:
      - "5432:5432"
    environment:
      POSTGRES_PASSWORD: ${PG_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./postgresql.conf:/etc/postgresql/postgresql.conf
    command: postgres -c config_file=/etc/postgresql/postgresql.conf
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: "2.0"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  sparql:
    build: .
    entrypoint: ["/usr/local/bin/pg_ripple_http"]
    ports:
      - "7878:7878"
    environment:
      PG_RIPPLE_HTTP_PG_URL: "postgresql://postgres:${PG_PASSWORD}@postgres/postgres"
      PG_RIPPLE_HTTP_PORT: "7878"
      PG_RIPPLE_HTTP_POOL_SIZE: "16"
      PG_RIPPLE_HTTP_CORS_ORIGINS: "https://yourdomain.com"
      PG_RIPPLE_HTTP_AUTH_TOKEN: ${SPARQL_AUTH_TOKEN}
    depends_on:
      postgres:
        condition: service_healthy
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: "1.0"
```

```admonish warning title="安全"
切勿在生产中使用默认密码。通过环境变量或 Docker 密钥设置 `POSTGRES_PASSWORD` 和 `PG_RIPPLE_HTTP_AUTH_TOKEN`。将 `PG_RIPPLE_HTTP_CORS_ORIGINS` 限制为您的实际域。
```

---

## 模型 3：托管 PostgreSQL 服务

pg_ripple 可以在支持自定义扩展和 PostgreSQL 18 的托管 PostgreSQL 服务上运行。关键要求是：

1. **PostgreSQL 18** — pg_ripple 使用 PG18 特定功能（例如，`WITH RECURSIVE ... CYCLE`）。
2. **自定义扩展加载** — 服务必须允许安装 `.so` 扩展并添加到 `shared_preload_libraries`。
3. **共享内存访问** — 字典缓存和合并工作线程所必需的。

### 支持的托管服务

| 服务 | 自定义扩展 | shared_preload_libraries | 状态 |
|---|---|---|---|
| AWS RDS for PostgreSQL | 是（通过自定义构建） | 是 | 支持自定义 AMI |
| Azure Database for PostgreSQL Flexible Server | 是 | 是 | 支持 |
| Google Cloud SQL | 有限 | 有限 | 部分支持 |
| 在 EC2/GCE/Azure VM 上自管理 | 完全控制 | 完全控制 | 完全支持 |

```admonish note title="云 VM 推荐"
对于托管云部署，在云 VM（EC2、GCE、Azure VM）上运行 PostgreSQL 18 并安装扩展可以完全控制并避免托管服务的限制。使用托管服务的块存储来实现持久性，使用快照进行备份。
```

### 托管服务配置

在托管服务上运行时：

```ini
# 添加到 PostgreSQL 参数组/配置
shared_preload_libraries = 'pg_ripple'

# 共享内存 — 托管服务通常会限制此值；从保守值开始
pg_ripple.dictionary_cache_size = 32768
pg_ripple.cache_budget = 32

# 合并工作线程以主数据库为目标
pg_ripple.worker_database = 'mydb'
```

### pg_ripple_http 作为 Sidecar

在托管服务上，将 `pg_ripple_http` 作为 sidecar 容器或 systemd 服务运行：

```bash
# Kubernetes sidecar 示例
PG_RIPPLE_HTTP_PG_URL="postgresql://user:pass@pg-host:5432/mydb" \
PG_RIPPLE_HTTP_PORT=7878 \
PG_RIPPLE_HTTP_POOL_SIZE=16 \
pg_ripple_http
```

---

## pg_ripple_http 配置参考

HTTP 伴随服务完全通过环境变量配置：

| 变量 | 默认 | 描述 |
|---|---|---|
| `PG_RIPPLE_HTTP_PG_URL` | *（必需）* | PostgreSQL 连接字符串 |
| `PG_RIPPLE_HTTP_PORT` | `7878` | HTTP 监听端口 |
| `PG_RIPPLE_HTTP_POOL_SIZE` | `8` | 连接池大小 |
| `PG_RIPPLE_HTTP_CORS_ORIGINS` | `*` | 允许的 CORS 来源 |
| `PG_RIPPLE_HTTP_AUTH_TOKEN` | *（无）* | 用于认证的 Bearer 令牌 |
| `PG_RIPPLE_HTTP_RATE_LIMIT` | `0` | 每秒请求数（0 = 无限制） |

### 端点

| 路径 | 方法 | 描述 |
|---|---|---|
| `/sparql` | GET, POST | SPARQL 协议查询/更新端点 |
| `/health` | GET | 健康检查（如果 PG 连接处于活动状态则返回 200） |
| `/metrics` | GET | 兼容 Prometheus 的指标 |

---

## 网络架构

```
                    ┌─────────────┐
                    │   客户端     │
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
     ┌─────────────────┐      ┌──────────────────┐
     │  pg_ripple_http  │      │  psql / JDBC /   │
     │  :7878           │      │  application     │
     │  (SPARQL 协议)   │      │  (:5432)         │
     └────────┬─────────┘      └────────┬─────────┘
              │                         │
              └────────────┬────────────┘
                           │
                           ▼
              ┌─────────────────────────┐
              │  PostgreSQL 18          │
              │  + pg_ripple 扩展        │
              │  + 合并工作线程 (BGW)     │
              └─────────────────────────┘
```

```admonish tip title="读副本"
对于读密集型工作负载，PostgreSQL 流复制开箱即用。读副本通过 WAL 接收所有 VP 表更改。通过连接到副本的单独 `pg_ripple_http` 实例将只读 SPARQL 查询指向副本。
```

---

## 部署后验证

部署 pg_ripple 后，验证安装：

```sql
-- 检查扩展版本
SELECT extversion FROM pg_extension WHERE extname = 'pg_ripple';

-- 验证统计（确认共享内存和合并工作线程）
SELECT pg_ripple.stats();

-- 运行健康检查
SELECT pg_ripple.canary();

-- 插入并查询测试三元组
SELECT pg_ripple.insert_triple(
    '<http://example.org/test>',
    '<http://example.org/status>',
    '"deployed"'
);

SELECT * FROM pg_ripple.sparql('
    SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 1
');
```

```admonish success title="健康部署清单"
- `stats()` 返回 `merge_worker_pid > 0`
- `canary()` 显示 `merge_worker: "ok"` 和 `catalog_consistent: true`
- 初始数据加载后 `encode_cache_hits / (hits + misses) > 0.90`
- SPARQL 查询返回结果
```
