# HTTP 配套服务

`pg_ripple_http` 为已安装 `pg_ripple` 扩展的 PostgreSQL 实例公开 SPARQL HTTP 端点。

## 生产配置

该服务默认为生产模式，绑定到 loopback，并需要读取令牌。尽可能从挂载的密钥设置 `PG_RIPPLE_HTTP_AUTH_TOKEN_FILE`。不要将令牌放在 Compose 文件、Dockerfile、Helm values 文件或 Kubernetes 清单中。

```bash
export AUTH_TOKEN="$(openssl rand -base64 32)"
docker run --rm -p 7878:7878 \
  -e PG_RIPPLE_HTTP_BIND="0.0.0.0:7878" \
  -e PG_RIPPLE_HTTP_PG_URL="postgresql://user:password@postgres:5432/mydb" \
  -e PG_RIPPLE_HTTP_AUTH_TOKEN="${AUTH_TOKEN}" \
  -e PG_RIPPLE_HTTP_PG_SSLMODE="verify-full" \
  -e PG_RIPPLE_HTTP_PG_CA_FILE="/run/secrets/pg-ca.pem" \
  -e PG_RIPPLE_HTTP_RATE_LIMIT="100" \
  -e PG_RIPPLE_HTTP_CORS_ORIGINS="https://app.example.com" \
  ghcr.io/trickle-labs/pg-ripple-http:0.134.0
```

在 `Authorization` 标头中发送令牌：

```bash
curl -H "Authorization: Bearer ${AUTH_TOKEN}" \
  http://localhost:7878/health
```

默认速率限制为每个客户端 IP 每秒 100 个请求。默认 CORS 允许列表为空，因此该服务不允许跨源请求。

密钥值可以使用读取、写入、管理、指标和 PostgreSQL 密码设置的 `_FILE` 变体提供。值在启动时加载一次，并且永远不包含在诊断响应中。`PG_RIPPLE_HTTP_BIND` 默认为 `127.0.0.1:7878`；为公共侦听器显式设置它。

## 本地开发

要在没有令牌的情况下运行，请显式设置 `PG_RIPPLE_HTTP_MODE=development` 和 `PG_RIPPLE_HTTP_ALLOW_UNAUTHENTICATED=1`。生产模式拒绝该覆盖，即使它是从容器环境继承的。

```bash
docker run --rm -p 7878:7878 \
  -e PG_RIPPLE_HTTP_BIND="0.0.0.0:7878" \
  -e PG_RIPPLE_HTTP_PG_URL="postgresql://user:password@postgres:5432/mydb" \
  -e PG_RIPPLE_HTTP_MODE=development \
  -e PG_RIPPLE_HTTP_ALLOW_UNAUTHENTICATED=1 \
  ghcr.io/trickle-labs/pg-ripple-http:0.134.0
```

## Helm

在 chart 外部创建令牌并引用现有的 Secret：

```bash
kubectl create secret generic my-ripple-http \
  --from-literal=auth-token="$AUTH_TOKEN"
helm install my-ripple ./charts/pg_ripple \
  --set http.authTokenSecret.name=my-ripple-http
```

可选的写入、管理和指标凭据可以使用 `http.writeTokenSecret`、`http.adminTokenSecret` 和 `http.metricsTokenSecret` 提供。

对于本地开发，设置 `http.mode=development` 和 `http.allowUnauthenticated=true`。chart 拒绝未经身份验证的生产配置。设置 `networkPolicy.enabled=true` 并为联邦或 LLM 目标提供显式的 `networkPolicy.egress` 条目。

对于 PostgreSQL TLS，将 `PG_RIPPLE_HTTP_PG_SSLMODE` 设置为 `require`、`verify-ca` 或 `verify-full`，并通过 pod 挂载 CA 文件。使用 `sql/roles/pg_ripple_http.sql` 中的专用 `pg_ripple_http` 角色；在使用它之前设置其密码或配置证书身份验证。它在 `_pg_ripple` 上没有直接权限。