# 安全

pg_ripple 提供多层安全性：PostgreSQL 的本机身份验证和授权、命名图行级安全性（RLS）、通过字典编码防止 SQL 注入，以及 `pg_ripple_http` 配套服务的安全配置。

---

## 身份验证和授权

pg_ripple 完全依赖 PostgreSQL 的内置身份验证（`pg_hba.conf`）和基于角色的访问控制。没有单独的用户数据库。

### SPARQL 查询的最低权限

```sql
-- 创建只读角色
CREATE ROLE sparql_reader LOGIN PASSWORD 'strong_password';
GRANT USAGE ON SCHEMA pg_ripple TO sparql_reader;
GRANT USAGE ON SCHEMA _pg_ripple TO sparql_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA _pg_ripple TO sparql_reader;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA pg_ripple TO sparql_reader;
```

### 数据加载的最低权限

```sql
-- 创建写入者角色
CREATE ROLE sparql_writer LOGIN PASSWORD 'strong_password';
GRANT USAGE ON SCHEMA pg_ripple TO sparql_writer;
GRANT USAGE ON SCHEMA _pg_ripple TO sparql_writer;
GRANT SELECT, INSERT, DELETE ON ALL TABLES IN SCHEMA _pg_ripple TO sparql_writer;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA _pg_ripple TO sparql_writer;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA pg_ripple TO sparql_writer;
```

```admonish warning title="默认权限"
使用 `ALTER DEFAULT PRIVILEGES` 确保新创建的 VP 表（在遇到新谓词时创建）继承正确的授权：
```

```sql
ALTER DEFAULT PRIVILEGES IN SCHEMA _pg_ripple
  GRANT SELECT ON TABLES TO sparql_reader;
ALTER DEFAULT PRIVILEGES IN SCHEMA _pg_ripple
  GRANT SELECT, INSERT, DELETE ON TABLES TO sparql_writer;
```

---

## 命名图行级安全性

pg_ripple 使用 PostgreSQL 的行级安全性（RLS）基础结构支持命名图级别的细粒度访问控制。这允许不同的用户看到知识图的不同子集。

### 启用图 RLS

```sql
-- 在所有 VP 表上启用 RLS
SELECT pg_ripple.enable_graph_rls();
```

这将在每个 VP 表（包括 `vp_rare`）上创建根据 `g`（图）列过滤行的 RLS 策略。

### 授予图访问权限

```sql
-- 向角色授予对特定命名图的访问权限
SELECT pg_ripple.grant_graph('sparql_reader', 'http://example.org/confidential');

-- 授予对默认图的访问权限（g = 0）
SELECT pg_ripple.grant_graph('sparql_reader', '');

-- 授予对所有图的访问权限
SELECT pg_ripple.grant_graph('sparql_reader', '*');
```

### 撤销图访问权限

```sql
-- 撤销对特定图的访问权限
SELECT pg_ripple.revoke_graph('sparql_reader', 'http://example.org/confidential');
```

### 工作原理

当启用图 RLS 时：

1. 每个 VP 表都会获得一个 RLS 策略，根据用户的允许图 ID 检查 `g` 列
2. 字典将图 IRI 编码为 `i64` 标识符
3. 内部映射表（`_pg_ripple.graph_grants`）存储 `(role, graph_id)` 对
4. PostgreSQL 透明地强制执行策略 — SPARQL 查询自动过滤结果

```admonish note title="超级用户绕过"
PostgreSQL 超级用户默认会绕过 RLS。要对超级用户强制执行图安全性，只需将 `row_security = on` 显式设置即可。用户必须显式 `SET row_security = on` 并且不能是表所有者。对于生产环境，对应用程序连接使用非超级用户角色。
```

### 示例：多租户知识图

```sql
-- 创建租户角色
CREATE ROLE tenant_a LOGIN PASSWORD 'pw_a';
CREATE ROLE tenant_b LOGIN PASSWORD 'pw_b';

-- 授予基本访问权限
GRANT USAGE ON SCHEMA pg_ripple TO tenant_a, tenant_b;
GRANT USAGE ON SCHEMA _pg_ripple TO tenant_a, tenant_b;
GRANT SELECT ON ALL TABLES IN SCHEMA _pg_ripple TO tenant_a, tenant_b;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA pg_ripple TO tenant_a, tenant_b;

-- 启用图 RLS
SELECT pg_ripple.enable_graph_rls();

-- 租户 A 只能看到他们的图
SELECT pg_ripple.grant_graph('tenant_a', 'http://example.org/tenant-a');

-- 租户 B 只能看到他们的图
SELECT pg_ripple.grant_graph('tenant_b', 'http://example.org/tenant-b');

-- 两者都看到共享参考数据
SELECT pg_ripple.grant_graph('tenant_a', 'http://example.org/shared');
SELECT pg_ripple.grant_graph('tenant_b', 'http://example.org/shared');
```

现在 `tenant_a` 运行的 SPARQL 查询将仅看到 `tenant-a` 和 `shared` 图中的三元组，无需应用程序级过滤。

---

## SQL 注入防御

pg_ripple 的架构从设计上提供了对 SQL 注入的强大防御。

### 字典编码作为安全层

所有 SPARQL 查询都经过多步骤转换管道：

1. **解析**：SPARQL 文本由 `spargebra` 解析为抽象代数树
2. **编码**：所有绑定常量（IRI、字面量）在 SQL 生成**之前**被字典编码为 `i64` 整数
3. **生成**：SQL 使用带整数占位符的参数化查询构造
4. **执行**：SQL 通过 `pgrx::SpiClient` 使用绑定参数运行

```admonish success title="VP 查询中没有原始字符串"
因为 VP 表仅存储 `BIGINT` 列（`s`、`o`、`g`、`i`、`source`），所以没有基于字符串的 SQL 注入攻击面。即使恶意 IRI 在 SPARQL 查询中传递，它在生成任何 SQL 之前也会被散列为整数。
```

### 表名安全

VP 表引用使用来自 `_pg_ripple.predicates` 的 OID 查找，而不是字符串连接：

```rust
// 内部：表名从不从用户输入插入
let table_oid = predicates::get_table_oid(predicate_id)?;
// SQL 直接使用 OID：FROM pg_class WHERE oid = $1
```

### 用户面向的函数安全

接受文本输入的函数（如 `pg_ripple.sparql()`）通过 `spargebra` 解析 SPARQL 文本，它会拒绝任何不是有效 SPARQL 的内容。不会传递原始 SQL。

---

## 文件路径加载器和超级用户要求

从服务器文件系统读取的函数需要超级用户权限：

| 函数 | 需要超级用户 | 原因 |
|---|---|---|
| `pg_ripple.load_turtle_file(path)` | **是** | 读取任意文件系统路径 |
| `pg_ripple.load_ntriples_file(path)` | **是** | 读取任意文件系统路径 |
| `pg_ripple.load_rdfxml_file(path)` | **是** | 读取任意文件系统路径 |
| `pg_ripple.load_turtle(text)` | 否 | 仅解析内存中文本 |
| `pg_ripple.load_ntriples(text)` | 否 | 仅解析内存中文本 |

```admonish danger title="文件系统访问"
文件路径加载器可以读取 PostgreSQL 进程有权访问的任何文件。永远不要将超级用户授予应用程序角色。而是，以超级用户身份加载数据，并通过 schema 权限向应用程序角色授予读取访问权限。
```

### 安全的批量加载模式

```sql
-- 以超级用户身份：加载数据
SELECT pg_ripple.load_turtle_file('/data/import/dataset.ttl');

-- 以超级用户身份：向应用程序角色授予访问权限
GRANT SELECT ON ALL TABLES IN SCHEMA _pg_ripple TO app_role;
```

---

## pg_ripple_http 安全性

`pg_ripple_http` 配套服务通过 HTTP 公开 SPARQL 协议端点。适当地保护它。

### TLS 配置

始终在生产中的 TLS 后运行 `pg_ripple_http`：

```toml
# pg_ripple_http.toml
[server]
bind = "0.0.0.0:8443"
tls_cert = "/etc/ssl/certs/pg_ripple_http.crt"
tls_key = "/etc/ssl/private/pg_ripple_http.key"
```

```admonish danger title="永远不要在没有 TLS 的情况下公开 HTTP"
SPARQL 查询可能包含敏感数据模式。没有 TLS，查询和结果将以明文形式传输。始终在服务或反向代理层终止 TLS。
```

### 身份验证

配置 `pg_ripple_http` 以对传入请求进行身份验证：

```toml
[auth]
# 由 PostgreSQL 角色支持的 HTTP Basic 身份验证
method = "pg_role"
# 或使用静态 API 密钥
# method = "api_key"
# api_key = "your-secret-key-here"
```

使用 `pg_role` 身份验证，HTTP Basic 凭据将转发到 PostgreSQL。图 RLS 策略应用于经过身份验证的角色。

### 反向代理设置

对于生产环境，将 `pg_ripple_http` 放在反向代理后面：

```nginx
# nginx 配置
server {
    listen 443 ssl;
    server_name sparql.example.org;

    ssl_certificate /etc/ssl/certs/sparql.crt;
    ssl_certificate_key /etc/ssl/private/sparql.key;

    location /sparql {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;

        # 速率限制
        limit_req zone=sparql burst=20 nodelay;
    }
}
```

### CORS 配置

如果从浏览器应用程序访问 SPARQL 端点：

```toml
[cors]
allowed_origins = ["https://app.example.org"]
allowed_methods = ["GET", "POST"]
allowed_headers = ["Content-Type", "Authorization"]
max_age = 3600
```

```admonish warning title="避免通配符来源"
不要在生产中设置 `allowed_origins = ["*"]`。这允许任何网站使用访问者的凭据向您的端点发送 SPARQL 查询。
```

---

## 网络隔离

### 生产拓扑

```
┌─────────────┐     TLS      ┌──────────────────┐     Unix socket     ┌─────────────┐
│  Clients    │ ────────────→ │  pg_ripple_http   │ ──────────────────→ │ PostgreSQL  │
│             │               │  (reverse proxy)  │                     │ (pg_ripple) │
└─────────────┘               └──────────────────┘                     └─────────────┘
```

### 建议

1. **PostgreSQL**：仅绑定到 `localhost` 或私有网络接口。永远不要将端口 5432 公开到公共互联网。

```ini
# postgresql.conf
listen_addresses = '127.0.0.1'
```

2. **pg_ripple_http**：通过 Unix 套接字连接到 PostgreSQL，以获得最低延迟且无网络暴露。

3. **防火墙规则**：仅允许来自预期客户端网络的 HTTPS 端口（443）上的流量。

```bash
# iptables 示例
iptables -A INPUT -p tcp --dport 443 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j DROP
iptables -A INPUT -p tcp --dport 5432 -j DROP
```

4. **pg_hba.conf**：按源 IP 和身份验证方法限制连接：

```
# TYPE  DATABASE  USER              ADDRESS        METHOD
local   all       postgres                         peer
host    mydb      pg_ripple_http    127.0.0.1/32   scram-sha-256
host    mydb      sparql_reader     10.0.0.0/8     scram-sha-256
host    all       all               0.0.0.0/0      reject
```

```admonish tip title="使用 scram-sha-256"
始终使用 `scram-sha-256` 身份验证（PostgreSQL 18 中的默认设置）。避免使用 `md5`，永远不要使用 `trust`。
```

---

## 安全清单

| 项目 | 状态 |
|---|---|
| `shared_preload_libraries` 仅包含受信任的扩展 | ☐ |
| 所有应用程序连接都使用非超级用户角色 | ☐ |
| 为多租户部署启用图 RLS | ☐ |
| `pg_hba.conf` 将连接限制为已知网络 | ☐ |
| 在 `pg_ripple_http` 或反向代理上启用 TLS | ☐ |
| 文件路径加载器仅限制为超级用户（默认） | ☐ |
| 为生产启用了 `synchronous_commit`（不是 `off`） | ☐ |
| 连接池使用 `scram-sha-256` | ☐ |
| CORS 来源未通配符 | ☐ |
| 启用 PostgreSQL 日志以进行审计跟踪 | ☐ |
| PostgreSQL 和 pg_ripple 的定期安全更新 | ☐ |

---

## 计划的 v1.0.0 外部审计

在 v0.132.0 之后，项目计划进行独立的安全审计，涵盖 Rust 扩展、HTTP 配套服务、SQL 迁移、Docker 镜像和 Helm chart。请求的审查范围包括 OWASP ASVS 2 级、PostgreSQL 扩展攻击面、SSRF、注入、身份验证绕过和权限提升。参与简报和交付物清单在 [`audit/engagement-brief.md`](../../../audit/engagement-brief.md) 中跟踪。

审计计划在 v0.132.0 标签后四周内开始。其报告和补救确认是 v1.0.0 的发布证据；本节不是声明审计已完成。

## 审计日志记录

启用 PostgreSQL 的日志记录以维护审计跟踪：

```ini
# postgresql.conf
log_statement = 'all'          # 或仅针对 schema 更改的 'ddl'
log_connections = on
log_disconnections = on
log_line_prefix = '%t [%p] %u@%d '
```

对于细粒度的审计日志记录，请考虑将 `pgaudit` 扩展与 pg_ripple 一起使用。

```admonish note title="SPARQL 查询日志记录"
pg_ripple 通过 PostgreSQL 的标准语句日志记录来记录生成的 SQL。要查看原始 SPARQL 文本，请启用 `log_statement = 'all'` — SPARQL 文本将显示为 `pg_ripple.sparql()` 的参数。
```

---

## 日志钩子审计和密钥抑制（LOG-HOOK-01）

> **v0.76.0 审计发现**：未安装 `RegisterEmitLogHook` 以从 PostgreSQL 错误消息中抑制密钥。执行了深度审计以验证没有错误或警告调用站点泄露原始密钥。

### 审计结果

对 pg_ripple 扩展和 `pg_ripple_http` 配套服务中的所有 `pgrx::error!()`、`pgrx::warning!()`、`tracing::error!()` 和 `tracing::warn!()` 调用站点执行了以下审计：

| 组件 | 处理的密钥 | 在错误中记录？ | 注释 |
|---|---|---|---|
| `src/security_api.rs` | 无（策略名称、图 IRI） | 否 | 仅结构化元数据 |
| `src/llm/mod.rs` | LLM API 密钥（环境变量） | 否 | 密钥在调用时从环境读取，未在错误路径中引用 |
| `pg_ripple_http` 身份验证检查 | Bearer 令牌（HMAC 比较） | 否 | `check_token` 使用 `constant_time_eq`；返回 401 而无令牌值 |
| `pg_ripple_http` Arrow Flight | HMAC 签名密钥 | 否 | 错误返回通用的 `"HMAC key error: {e}"` — `e` 只是密钥长度错误 |
| `pg_ripple_http/src/common.rs` | `auth_token`、`datalog_write_token` | 否 | 令牌存储在 `AppState` 中，但从不插入到错误消息中 |
| `pg_ripple_http/src/main.rs` | `ARROW_FLIGHT_SECRET`（环境变量） | 否 | 环境变量在启动时读取；未在跟踪调用中引用 |

### 不需要 `RegisterEmitLogHook`

根据审计，当前代码库中没有错误路径记录原始 HMAC 密钥、连接字符串、持有者令牌或其他凭据。因此目前不需要 `RegisterEmitLogHook`。

**如果您添加接触密钥的新错误路径**，请遵循以下准则：
- 不要将密钥值（`token`、`key`、`password`）插入 `pgrx::error!()` 或 `pgrx::warning!()` 消息。
- 使用没有值的描述性错误类型（例如，`"HMAC key error: invalid key length"` 而不是 `"bad key: {key_value}"`）。
- 如果未来版本在热路径中处理用户提供的凭据，请考虑安装 `RegisterEmitLogHook` 以从日志行中清除敏感模式。

### 连接字符串

包含密码的 PostgreSQL 连接字符串（DSN）可能通过 `log_connections = on` 出现在 `pg_log` 中。使用 `pg_hba.conf` 和 `scram-sha-256` 以及单独进行身份验证的连接池（例如 PgBouncer），以便应用程序 DSN 从不包含内联密码。

---

## 支持的身份验证方案（S13-10，v0.86.0）

`pg_ripple_http` 仅接受 **Bearer 令牌**身份验证。

| 方案 | 支持 | 注释 |
|---|---|---|
| Bearer（RFC 6750） | ✅ 是 | 设置 `PG_RIPPLE_HTTP_AUTH_TOKEN` 环境变量；传递 `Authorization: Bearer <token>` |
| Basic（RFC 7617） | ❌ 否 | 不接受；Basic 以明文形式发送凭据（即使通过 TLS，base64 也是可轻易反转的） |
| API 密钥（标头） | ❌ 否 | 不是标准的 HTTP 身份验证方案；请改用 Bearer |
| mTLS | ⏳ 未来 | 计划用于 v1.x |

**建议**：定期轮换 Bearer 令牌。使用 32 字节（256 位）或更大的随机密钥：

```bash
openssl rand -base64 32
```

将令牌设置为环境变量或通过 Docker secrets — 永远不要将其硬编码到 `docker-compose.yml` 或配置文件中。

---

## 指标端口隔离（S13-09，v0.86.0）

```admonish warning title="/metrics 需要网络隔离"
`GET /metrics` 端点公开 Prometheus 格式的操作数据，包括查询计数、错误率、缓存统计信息和联邦端点信息。**此端点不需要身份验证** — 它依赖于网络级隔离。

**不要将指标端口（默认值：与 API 端口 7878 相同）暴露给不受信任的网络。**

在生产部署中：
- 在专用网络（例如 Kubernetes pod 网络、Docker 内部网络）中运行 `pg_ripple_http`。
- 使用反向代理（nginx、Envoy、Traefik）仅外部公开 `/sparql`、`/datalog` 和 `/health`。
- 通过反向代理 ACL 将 `/metrics` 限制为 Prometheus 抓取器 IP。
- 考虑使用单独的指标端口绑定部署 `pg_ripple_http`（未来功能）。

**CORS 宽容模式警告**：当设置 `PG_RIPPLE_HTTP_CORS_ORIGINS=*` 时，每个跨源请求都会递增 `pg_ripple_http_cors_permissive_requests_total` Prometheus 计数器。监视此计数器以检测意外的跨源流量。在生产中使用特定的来源白名单（例如 `PG_RIPPLE_HTTP_CORS_ORIGINS=https://app.example.com`）。
```