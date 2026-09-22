# 向量联邦

pg_ripple v0.28.0 引入了**向量联邦**：已注册的外部向量服务（Weaviate、Qdrant、Pinecone 或其他 pgvector 实例）可与本地三元组存储一起被查询。

---

## 为什么要向量联邦？

组织通常有多个向量数据库：保存产品嵌入的 Qdrant 集群、用于客户数据的 Weaviate 实例以及用于知识图谱的 pg_ripple。向量联邦让您能够在单个 `pg_ripple.hybrid_search()` 调用中混合所有这三个来源，无需移动数据。

---

## 支持的服务类型

| `api_type` | 备注 |
|---|---|
| `pgvector` | 远程 PostgreSQL + pgvector 实例 |
| `weaviate` | Weaviate v1 GraphQL 端点（`/v1/graphql`） |
| `qdrant` | Qdrant v1 REST API（`/collections/{name}/points/search`） |
| `pinecone` | Pinecone v1 REST API（`/query`） |

---

## 注册端点

```sql
-- 注册 Qdrant 实例。
SELECT pg_ripple.register_vector_endpoint(
    'https://qdrant.internal:6333',
    'qdrant'
);

-- 注册 Weaviate 实例。
SELECT pg_ripple.register_vector_endpoint(
    'https://weaviate.internal:8080',
    'weaviate'
);

-- 注册 Pinecone 索引。
SELECT pg_ripple.register_vector_endpoint(
    'https://my-index-abc123.pinecone.io',
    'pinecone'
);
```

注册是幂等的——使用相同 URL 两次调用 `register_vector_endpoint()` 是安全的。

端点存储在 `_pg_ripple.vector_endpoints` 中：

```sql
SELECT url, api_type, enabled, registered_at
FROM _pg_ripple.vector_endpoints;
```

---

## 超时配置

联邦超时适用于对外部向量服务的所有出站 HTTP 调用：

```sql
-- 设置 10 秒超时。
SET pg_ripple.vector_federation_timeout_ms = 10000;
```

---

## 安全注意事项

- 端点存储在内部表中，只有 `pg_ripple` schema 用户可访问。
- 推荐对所有外部端点使用 HTTPS。
- 外部服务的 API 密钥不由 pg_ripple 存储；通过环境变量或应用程序层中的密钥管理进行配置。
- `pg_ripple.vector_federation_timeout_ms` GUC 可防止失控的联邦查询阻塞数据库。

---

## 目录表参考

### `_pg_ripple.vector_endpoints`

| 列 | 类型 | 描述 |
|---|---|---|
| `url` | `TEXT` | 端点基本 URL（主键） |
| `api_type` | `TEXT` | 之一：`pgvector`、`weaviate`、`qdrant`、`pinecone` |
| `enabled` | `BOOLEAN` | 此端点是否处于活动状态 |
| `registered_at` | `TIMESTAMPTZ` | 端点注册的时间 |

---

## GUC 参考

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.vector_federation_timeout_ms` | `5000` | 联邦向量端点查询的 HTTP 超时（毫秒） |