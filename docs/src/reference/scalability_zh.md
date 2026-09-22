# 可扩展性参考

pg_ripple 通过 [Citus](https://www.citusdata.com/)（针对 PostgreSQL）具有实验性的水平分片支持。当前的 CI 在没有 Citus 的情况下演练优雅降级；它不创建多节点集群。

## Citus 集成

当 `pg_ripple.citus_sharding_enabled = on` 并且安装了 Citus 扩展时，pg_ripple 会使用主语（`s`）列作为分片键，将 VP 表分布到 Citus 工作节点。

### 功能

- **绑定主语的分片剪枝**：具有绑定主语 IRI 的 SPARQL 查询被重写以包含 `WHERE s = <encoded_id>`，以便 Citus 将查询直接路由到持有该主语的分片（v0.59.0）。
- **直接分片批量加载**：`load_*` 函数直接将三元组写入物理 Citus 分片表，绕过协调器路由步骤（v0.61.0 CITUS-21）。
- **再平衡时 BRIN 汇总**：VP 主分区是 BRIN 索引的；在分片再平衡后，对受影响分片重新运行汇总步骤（v0.63.0）。
- **HyperLogLog COUNT(DISTINCT)**：Citus HLL 扩展用于联邦查询中的近似 `COUNT(DISTINCT ?var)` 聚合（v0.63.0、v0.68.0）。
- **每个命名图的 RLS 传播**：`grant_graph_access` 通过 `run_command_on_all_nodes` 将行级安全策略传播到所有 Citus 工作节点（v0.61.0 CITUS-05）。集成测试：`tests/integration/citus_rls_propagation.sh`（计划于 v0.71.0）。

### 配置

| GUC | 默认值 | 描述 |
|-----|---------|-------------|
| `pg_ripple.citus_sharding_enabled` | `off` | 启用 VP 表的 Citus 分布 |
| `pg_ripple.citus_shard_count` | `32` | 每个 VP 表的分片数 |
| `pg_ripple.citus_hll_enabled` | `off` | 使用 HLL 扩展进行 COUNT(DISTINCT) |

### 限制

- 当前 CI 作业不安装 Citus 或运行真实的多节点集群。
- 再平衡、故障转移、跨节点写入行为和容量需要特定于工作负载的验证。
- 跨分片属性路径查询可能无法从分片剪枝中受益。

另请参阅：[性能调优](../operations/performance.md), [架构](architecture.md)。