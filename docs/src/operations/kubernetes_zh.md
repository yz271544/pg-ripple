# Kubernetes 部署

pg_ripple 提供一个 Helm chart（`charts/pg_ripple/`），它在任何 Kubernetes 集群上
部署一应俱全的镜像 — 预装了 pg_ripple、PostGIS 和 pgvector 的 PostgreSQL 18。

## 先决条件

- Kubernetes ≥ 1.25
- Helm ≥ 3.10
- 持久卷配置器（任何云提供商或 `local-path-provisioner`）

## 安装

### 添加 Helm 仓库

```bash
helm repo add pg-ripple https://trickle-labs.github.io/pg-ripple/charts
helm repo update
```

### 使用默认值安装

```bash
helm install pg-ripple pg-ripple/pg-ripple \
  --set postgres.password=mysecretpassword
```

### 从源代码安装

```bash
helm install pg-ripple ./charts/pg_ripple \
  --set postgres.password=mysecretpassword
```

### 验证部署

```bash
kubectl get pods -l app.kubernetes.io/name=pg-ripple
kubectl exec -it <pod-name> -- psql -U postgres \
  -c "SELECT extname, extversion FROM pg_extension WHERE extname = 'pg_ripple';"
```

## Values 参考

| Key | 默认 | 描述 |
|-----|---------|-------------|
| `replicaCount` | `1` | PostgreSQL Pods 数量 |
| `image.repository` | `ghcr.io/trickle-labs/pg-ripple` | 镜像仓库 |
| `image.tag` | `0.54.0` | 镜像标签 |
| `image.pullPolicy` | `IfNotPresent` | 镜像拉取策略 |
| `postgres.password` | `ripple` | 超级用户密码（在生产中使用 Secret） |
| `postgres.database` | `postgres` | 要创建的数据库 |
| `persistence.enabled` | `true` | 启用持久存储 |
| `persistence.size` | `10Gi` | PVC 大小 |
| `persistence.storageClass` | `""` | StorageClass（空 = 集群默认） |
| `service.type` | `ClusterIP` | PostgreSQL 服务类型 |
| `service.port` | `5432` | PostgreSQL 端口 |
| `http.enabled` | `true` | 启用 SPARQL HTTP sidecar |
| `http.service.type` | `ClusterIP` | SPARQL HTTP 服务类型 |
| `http.service.port` | `7878` | SPARQL HTTP 端口 |
| `ripple.federationEndpoints` | `[]` | SPARQL 联邦端点 |
| `ripple.shacl.shapesConfigMap` | `""` | SHACL shapes ConfigMap |
| `ripple.llm.apiKeySecret` | `""` | LLM API 密钥 Secret 名称 |

## 常见配置

### 外部公开 SPARQL HTTP（LoadBalancer）

```bash
helm upgrade pg-ripple ./charts/pg_ripple \
  --set http.service.type=LoadBalancer \
  --set http.service.port=7878
```

### 增加存储

```bash
helm upgrade pg-ripple ./charts/pg_ripple \
  --set persistence.size=100Gi \
  --set persistence.storageClass=premium-ssd
```

### 配置联邦端点

```bash
helm upgrade pg-ripple ./charts/pg_ripple \
  --set 'ripple.federationEndpoints[0].name=wikidata' \
  --set 'ripple.federationEndpoints[0].url=https://query.wikidata.org/sparql'
```

### 启用 Prometheus 监控

```bash
helm upgrade pg-ripple ./charts/pg_ripple \
  --set metrics.enabled=true \
  --set metrics.serviceMonitor.enabled=true
```

## 健康探针

chart 使用 `pg_isready` 配置存活和就绪探针：

```yaml
livenessProbe:
  exec:
    command: ["pg_isready", "-U", "postgres"]
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  exec:
    command: ["pg_isready", "-U", "postgres"]
  initialDelaySeconds: 5
  periodSeconds: 5
```

## Prometheus 集成

当 `metrics.serviceMonitor.enabled = true` 时，chart 为 Prometheus Operator 创建一个
`ServiceMonitor` 资源。pg_ripple 通过 `pg_stat_statements` 公开查询统计，
并通过 `pg_ripple.tracing_otlp_endpoint` GUC 进行 OTEL 跟踪。

配置 OTEL 端点：

```bash
kubectl exec -it <pod-name> -- psql -U postgres \
  -c "SET pg_ripple.tracing_otlp_endpoint = 'http://otel-collector:4318';"
```

## 未来：Kubernetes Operator

未来版本将提供一个使用 `controller-runtime` 构建的 Go operator，它将
pg_ripple 集群作为一流的 Kubernetes 资源进行管理 — 类似于 CloudNativePG，
但专为 RDF 工作负载生命周期（批量加载、VP 合并调度、SHACL 验证管道）量身定制。

该 operator 将提供：
- 具有声明式 schema 管理的 `RDFTripleStore` 自定义资源
- 零停机的自动滚动升级
- 内置的 Prometheus 指标 CRD
- 通过 ConfigMap 引用自动部署 SHACL shapes

## 预安装的扩展

一应俱全的镜像预装：

| 扩展 | 版本 | 激活方式 |
|-----------|---------|---------------|
| pg_ripple | 0.54.0 | `CREATE EXTENSION pg_ripple;` |
| PostGIS | 3.4.3 | `CREATE EXTENSION postgis;` |
| pgvector | 0.7.4 | `CREATE EXTENSION vector;` |
