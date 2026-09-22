# CloudNativePG 部署

[CloudNativePG (CNP)](https://cloudnative-pg.io) 是一个用于管理 PostgreSQL 集群的 Kubernetes operator。
pg_ripple v0.98.0 为 CNP ≥ 1.24 提供预构建的扩展镜像，
允许运维人员在**没有自定义 PostgreSQL 容器镜像**和没有自定义构建步骤的情况下
将 pg_ripple 安装到托管集群中。

## 先决条件

- CloudNativePG operator ≥ 1.24
- Kubernetes ≥ 1.25
- CNP 中启用 `Image Volume` 特性门（CNP ≥ 1.24 默认启用）

## 工作原理

CloudNativePG ≥ 1.24 支持[扩展镜像](https://cloudnative-pg.io/documentation/current/extension_volumes/)：
一个最小化的 OCI 镜像，其唯一目的是提供预编译的 `.so` 和 SQL 文件。
CNP 在启动时将镜像挂载为 init-container 卷并将文件复制到 PostgreSQL 容器中。

pg_ripple 每个版本发布两个镜像：

| 镜像 | 内容 |
|-------|----------|
| `ghcr.io/trickle-labs/pg-ripple:<version>` | 完整的一应俱全的镜像 |
| `ghcr.io/trickle-labs/pg-ripple:<version>-cnpg` | CloudNativePG 的扩展卷 |

`-cnpg` 镜像包含为 PostgreSQL 18 编译的 pg_ripple 和 pgvector，位于
CNP 期望的路径：

```
/var/lib/postgresql/extension-files/lib/pg_ripple.so
/var/lib/postgresql/extension-files/ext/pg_ripple.control
/var/lib/postgresql/extension-files/ext/pg_ripple--*.sql
/var/lib/postgresql/extension-files/lib/vector.so
/var/lib/postgresql/extension-files/ext/vector.control
```

## 集群清单演练

示例清单位于 `examples/cloudnativepg_cluster.yaml`：

```yaml
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: pg-ripple-cluster
spec:
  imageName: ghcr.io/cloudnative-pg/postgresql:18
  instances: 3

  postgresql:
    extensionImages:
      - name: pg-ripple-ext
        image: ghcr.io/trickle-labs/pg-ripple:0.98.0-cnpg   # ← 扩展卷
    parameters:
      allow_system_table_mods: "on"
      shared_preload_libraries: "pg_ripple"

  storage:
    size: 20Gi

  superuserSecret:
    name: pg-ripple-superuser

  bootstrap:
    initdb:
      database: postgres
      postInitSQL:
        - "CREATE EXTENSION IF NOT EXISTS pg_ripple;"
        - "CREATE EXTENSION IF NOT EXISTS vector;"
```

要点：
- `imageName` 是标准 CNP 基础镜像 — **不是**自定义构建。
- `extensionImages` 列出 pg_ripple 扩展卷。CNP 在 PostgreSQL 启动之前挂载它并复制文件。
- `postInitSQL` 在首次集群启动时运行 `CREATE EXTENSION`。

## 部署

```bash
# 首先创建超级用户密钥
kubectl create secret generic pg-ripple-superuser \
  --from-literal=username=postgres \
  --from-literal=password=your-secure-password

# 应用集群清单
kubectl apply -f examples/cloudnativepg_cluster.yaml

# 等待所有实例就绪
kubectl wait --for=condition=Ready cluster/pg-ripple-cluster --timeout=120s
```

## 部署后验证

```bash
kubectl exec -it pg-ripple-cluster-1 -- psql -U postgres -c \
  "SELECT extname, extversion FROM pg_extension WHERE extname IN ('pg_ripple', 'vector');"
```

预期输出：

```
  extname  | extversion
-----------+------------
 pg_ripple | 0.98.0
 vector    | 0.8.2
```

加载测试三元组并运行 SPARQL 查询：

```bash
kubectl exec -it pg-ripple-cluster-1 -- psql -U postgres -c \
  "SELECT pg_ripple.load_ntriples('<https://example.org/s> <https://example.org/p> <https://example.org/o> .');"

kubectl exec -it pg-ripple-cluster-1 -- psql -U postgres -c \
  "SELECT * FROM pg_ripple.sparql('SELECT ?s ?p ?o WHERE { ?s ?p ?o }');"
```

## 升级过程

升级 pg_ripple 是集群清单中的一行更改 — 增加扩展镜像标签并应用：

```bash
# 编辑清单以更改镜像标签
sed -i 's/pg_ripple:0.97.0-cnpg/pg_ripple:0.98.0-cnpg/' \
  examples/cloudnativepg_cluster.yaml

kubectl apply -f examples/cloudnativepg_cluster.yaml

# 滚动重启完成后，运行迁移
kubectl exec -it pg-ripple-cluster-1 -- psql -U postgres \
  -c "ALTER EXTENSION pg_ripple UPDATE TO '0.98.0';"
```

CNP 自动处理滚动重启，确保零停机。

## 高可用性

CloudNativePG 提供内置 HA：如果当前主服务器发生故障，主服务器将自动从备用实例中选出。
pg_ripple 的共享内存和后台工作线程（合并工作线程、应用工作线程）由 PostgreSQL 在新主服务器上自动重启。

有关跨 CNP 集群的 RDF 逻辑复制，请参阅
[逻辑复制](replication.md)。
