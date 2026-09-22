# Docker 部署

## 一应俱全的镜像

`ghcr.io/trickle-labs/pg-ripple:<version>` 镜像在单个容器中捆绑了六个扩展：

| 扩展 | 用途 |
|-----------|---------|
| `pg_ripple` | 具有原生 SPARQL 的 RDF 三元组存储 |
| `PostGIS` | 通过 GeoSPARQL 进行地理空间查询 |
| `pgvector` | 用于混合 SPARQL + 语义的向量相似性搜索 |
| `pg_trickle` | 增量物化的 SPARQL 视图 |
| `pg_tide` | 用于变更数据捕获的中继、outbox 和 inbox 子系统 |

无需额外设置 — 只需启动容器并执行 `CREATE EXTENSION`。

## 快速开始

```bash
docker run --rm -p 5432:5432 \
  -e POSTGRES_PASSWORD=ripple \
  ghcr.io/trickle-labs/pg-ripple:0.127.0
```

```bash
psql -h localhost -U postgres -c "CREATE EXTENSION pg_ripple CASCADE;"
psql -h localhost -U postgres \
  -c "SELECT pg_ripple.load_ntriples('<https://example.org/s> <https://example.org/p> <https://example.org/o> .');"
```

启用可选扩展：

```sql
CREATE EXTENSION postgis;   -- GeoSPARQL 函数
CREATE EXTENSION vector;    -- 混合向量 + SPARQL 搜索
```

## Docker Compose

仓库附带一个 `docker-compose.yml`，它一起启动 pg_ripple 和 SPARQL HTTP 服务：

```bash
docker compose up -d
curl http://localhost:7878/health
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 10"
```

## 预安装的扩展版本

| 扩展 | 版本 | 注释 |
|-----------|---------|-------|
| pg_ripple | 0.127.0 | 具有原生 SPARQL 的 RDF 三元组存储 |
| PostGIS | 3.5.6 | 通过 GeoSPARQL 进行地理空间查询 |
| pgvector | 0.8.2 | 用于混合 SPARQL + 语义的向量相似性搜索 |
| pg_trickle | 0.57.0 | 增量物化的 SPARQL 视图 |
| pg_tide | 0.33.0 | 用于 CDC 的中继、outbox 和 inbox |

## 环境变量

| 变量 | 默认 | 描述 |
|----------|---------|-------------|
| `POSTGRES_PASSWORD` | *（必需）* | 超级用户密码 |
| `POSTGRES_DB` | `postgres` | 要创建的数据库 |
| `POSTGRES_USER` | `postgres` | 超级用户名 |

SPARQL HTTP 服务（`pg_ripple_http`）也接受：

| 变量 | 默认 | 描述 |
|----------|---------|-------------|
| `PG_RIPPLE_HTTP_PG_URL` | `postgresql://postgres:…@localhost/postgres` | 连接字符串 |
| `PG_RIPPLE_HTTP_PORT` | `7878` | 监听端口 |
| `PG_RIPPLE_HTTP_POOL_SIZE` | `8` | 连接池大小 |
| `PG_RIPPLE_HTTP_CORS_ORIGINS` | `*` | 允许的 CORS 来源 |

## 示例：GeoSPARQL 查询

```sql
CREATE EXTENSION postgis;

-- 加载地理三元组
SELECT pg_ripple.load_ntriples(
  '<https://example.org/Berlin> <http://www.opengis.net/ont/geosparql#asWKT> "POINT(13.405 52.52)"^^<http://www.opengis.net/ont/geosparql#wktLiteral> .'
);

-- 通过 SPARQL 查询
SELECT * FROM pg_ripple.sparql(
  'SELECT ?city ?wkt WHERE { ?city <http://www.opengis.net/ont/geosparql#asWKT> ?wkt }'
);
```

## 示例：混合向量 + SPARQL 搜索

```sql
CREATE EXTENSION vector;

-- 为资源创建嵌入
INSERT INTO _pg_ripple.embeddings (subject_id, embedding)
SELECT id, '[0.1, 0.2, ...]'::vector
FROM _pg_ripple.dictionary
WHERE value = 'https://example.org/Berlin';

-- 混合搜索：语义相似性 + SPARQL 过滤
SELECT * FROM pg_ripple.hybrid_search(
  query_embedding := '[0.1, 0.2, ...]'::vector,
  sparql_filter   := '?s <http://schema.org/type> <http://schema.org/City>',
  k               := 10
);
```

## 发布到 GHCR

每个版本都通过发布 GitHub Actions 工作流自动发布到 GitHub Container Registry。
您也可以在本地构建一应俱全的镜像：

```bash
docker build --tag my-pg-ripple:local .
```
