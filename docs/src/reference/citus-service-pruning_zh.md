# Citus SERVICE 分片修剪

当 pg_ripple 部署在 [Citus](https://www.citusdata.com/) 分布式 PostgreSQL 集群上时，针对 Citus 工作器分片的 SPARQL `SERVICE` 查询可以从分片注解修剪中受益。当可以使用主语 IRI 来标识目标分片时，这会减少查询的分片数。

## 什么是 SERVICE 分片注解？

在 SPARQL SERVICE 查询中，例如：

```sparql
SELECT ?name WHERE {
    SERVICE <https://worker-node.example/sparql> {
        <https://example.org/Alice> schema:name ?name
    }
}
```

当主语 IRI（`<https://example.org/Alice>`）被绑定时，pg_ripple 可以使用 Citus 分片键来计算哪些分片包含该主语的三元组。启用修剪后，仅查询相关的分片，绕过其他分片。

## 分片注解如何工作

1. 在查询规划时，pg_ripple 为每个 `SERVICE` 子句调用 `citus_service_shard_annotation(endpoint_url)`。
2. 如果端点被标识为 Citus 工作器（`is_citus_worker_endpoint()` 返回 true），并且存在绑定的主语 IRI，则计算分片路由键。
3. 生成的 SQL 附加 `WHERE dist_key = $shard_key` 以修剪分片扇出。

## 配置

| GUC | 默认 | 描述 |
|-----|---------|-------------|
| `pg_ripple.citus_service_pruning` | `off` | 为 Citus 工作器启用 SERVICE 分片注解 |

```sql
-- 为会话启用
SET pg_ripple.citus_service_pruning = on;

-- 或全局设置（Citus 部署推荐）
ALTER SYSTEM SET pg_ripple.citus_service_pruning = on;
SELECT pg_reload_conf();
```

## 验证

存储库不保留当前多节点延迟比较的原始证据。在目标集群上使用 `EXPLAIN` 验证路由决策，然后保留计划、数据集身份、拓扑和计时。

使用 `EXPLAIN` 重现：

```sql
-- 无修剪
SET pg_ripple.citus_service_pruning = off;
SELECT pg_ripple.explain_sparql(
    'SELECT ?name WHERE {
        SERVICE <http://citus-worker-1:5432/sparql> {
            <https://example.org/Alice> <https://schema.org/name> ?name
        }
    }',
    false
);

-- 有修剪
SET pg_ripple.citus_service_pruning = on;
SELECT pg_ripple.explain_sparql(
    'SELECT ?name WHERE {
        SERVICE <http://citus-worker-1:5432/sparql> {
            <https://example.org/Alice> <https://schema.org/name> ?name
        }
    }',
    false
);
```

在正确配置的集群上，比较两个计划中的分片数。在没有测量目标拓扑的情况下，不要假设特定的延迟改进。

## 状态

`citus_service_pruning` 是实验性的。单节点 CI 测试在没有 Citus 的情况下验证 GUC 和解释管道。它们不能保证多节点路由、正确性或性能。

另请参阅：[近似聚合（HLL）](approximate-aggregates.md)、[Citus 集成](../operations/citus-integration.md)、[兼容性矩阵](../operations/compatibility.md)。