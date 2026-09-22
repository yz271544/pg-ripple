# 近似聚合（HLL COUNT DISTINCT）

pg_ripple 在可用时使用 PostgreSQL [`hll` 扩展](https://github.com/citusdata/postgresql-hll) 在 SPARQL 查询中支持近似 `COUNT(DISTINCT ...)`。

## 何时使用 HLL

当满足**以下两个**条件时，HLL 用于 `COUNT(DISTINCT ...)`：

1. GUC `pg_ripple.approx_distinct` 设置为 `on`（默认值：`off`）。
2. `hll` PostgreSQL 扩展已安装在当前数据库中。

当任一条件不满足时，pg_ripple 回退到精确的 `COUNT(DISTINCT ...)` SQL。

**示例：**

```sql
-- 启用近似 COUNT(DISTINCT)
SET pg_ripple.approx_distinct = on;

-- 当 hll 扩展可用时，此 SPARQL 查询使用 HLL
SELECT *
FROM pg_ripple.sparql($$
    SELECT (COUNT(DISTINCT ?s) AS ?n) WHERE { ?s ?p ?o }
$$);
```

## 错误范围

`hll` 扩展使用默认精度 `log2m = 14` 的 HyperLogLog 草图（约 16,384 个寄存器桶）。结果错误特征是：

| 基数范围 | 标准错误 | 典型相对错误 |
|-------------------|----------------|------------------------|
| < 1,000 | ~3.5% | 高达 ±5% |
| 1,000 – 10,000 | ~1.5% | 通常 ±2% |
| ≥ 10,000 | ~0.81% | 通常 ±1% |
| ≥ 100,000 | ~0.025% | 通常 ±0.1% |

`log2m = 14` 的标准错误在渐近区（≥ 10,000 个不同值）中约为 `1.04 / sqrt(2^14) ≈ 0.0081`（0.81%）。

## 回退行为

当 `pg_ripple.approx_distinct = off` 或未安装 `hll` 扩展时：

- `COUNT(DISTINCT ...)` 使用精确的 SQL `COUNT(DISTINCT ...)` 语义。
- 不应用错误范围；结果是精确的。
- 这是默认行为。

## 启用 HLL

```sql
-- 安装 hll 扩展（需要超级用户或 pg_extension_owner）
CREATE EXTENSION IF NOT EXISTS hll;

-- 启用本会话的近似聚合
SET pg_ripple.approx_distinct = on;

-- 或全局设置
ALTER SYSTEM SET pg_ripple.approx_distinct = on;
SELECT pg_reload_conf();
```

## Citus / 分布式 COUNT DISTINCT

在 Citus 集群上，`COUNT(DISTINCT ...)` 不能原生下推到分片（因为分片可能包含重叠的值集）。当 `approx_distinct = on` 且存在 `hll` 扩展时，pg_ripple 将 `COUNT(DISTINCT ?x)` 重写为：

```sql
hll_cardinality(hll_add_agg(hll_hash_bigint(x)))::bigint
```

这允许 Citus 使用 `hll_union_agg` 在分片之间聚合 HLL 草图，提供可扩展的分布式近似计数，具有与上述相同的错误范围。

## 检查 HLL 是否处于活动状态

```sql
-- 检查当前设置
SHOW pg_ripple.approx_distinct;

-- 检查 hll 扩展是否已安装
SELECT count(*) > 0 AS hll_available
FROM pg_available_extensions
WHERE name = 'hll' AND installed_version IS NOT NULL;
```

## 配置参考

| GUC | 默认 | 描述 |
|---|---|---|
| `pg_ripple.approx_distinct` | `off` | 当 `hll` 可用时，对 `COUNT(DISTINCT ...)` 使用 HLL |

## 测试准确性

`pg_regress` 测试 `hll_accuracy.sql` 验证近似 `COUNT(DISTINCT ...)` 在 200 个不同主题的数据集上返回的结果在精确计数的 2% 范围内。在基数为 100,000 处的完整精度基准测试可以使用 `tests/http_integration/` 测试套件运行。

另请参阅：[GUC 参考](guc-reference.md)，[Citus 集成](../operations/citus-integration.md)。