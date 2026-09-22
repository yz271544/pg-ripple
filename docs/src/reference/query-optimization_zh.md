# 查询优化参考

此页面是 pg_ripple 的 SPARQL 查询优化器的参考。

## 概述

pg_ripple 在从 SPARQL 代数生成 SQL 之前应用多个优化过程：

1. **sparopt 代数优化器**：第一遍代数重写（变量替换、过滤器下推、带有 SHACL 提示的 DISTINCT 消除）。
2. **自连接消除**：同一主语上的星形模式被折叠到带有多个连接的单次扫描计划中。
3. **过滤器下推**：FILTER 常量在转换时编码为 `BIGINT`，以便在整数值（而非字符串）上进行比较。
4. **SHACL 提示**：`sh:maxCount 1` 谓词省略 DISTINCT；`sh:minCount 1` 谓词使用 INNER JOIN 而不是 LEFT JOIN。
5. **计划缓存**：已编译的 SQL 存储在 `_pg_ripple.plan_cache` 中，并在相同查询中重复使用（按查询文本 + 当前角色 + 相关 GUC 进行键控）。
6. **TopN 下推**：`ORDER BY ... LIMIT N` 模式被推入子查询以避免对完整结果集进行排序。
7. **Leapfrog TrieJoin (WCOJ)**：针对循环 SPARQL 图模式的次最优连接规划。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE '%plan%' OR feature_name LIKE '%cache%' OR feature_name LIKE '%wcoj%';
```

## 计划缓存

计划缓存避免为重复查询重新编译 SPARQL→SQL。关键细节：

- 缓存键：（查询文本、current_role、GUC 快照）的 SHA-256
- 在以下情况时使缓存失效：VP 提升（schema 更改）、扩展升级、`plan_cache_reset()`
- 最大条目数：`pg_ripple.plan_cache_size`（默认值：512）
- 驱逐策略：LRU

```sql
-- 检查缓存
SELECT * FROM _pg_ripple.plan_cache;

-- 手动失效
SELECT pg_ripple.plan_cache_reset();
```

## 属性路径优化

属性路径使用 PostgreSQL 18 的原生 `CYCLE` 子句编译为 `WITH RECURSIVE … CYCLE` 查询，以进行基于哈希的循环检测。有界深度路径（`{n,m}`）使用迭代 CTE 限制为 `pg_ripple.max_property_path_depth` 跳数（默认值：20）。提前不动点终止避免了针对有界层次结构进行超出收敛的迭代。

## Magic Sets（目标导向推导）

当使用 `pg_ripple.query_goal()` 进行 Datalog 查询时，magic sets 转换会重写规则以将推导集中在回答目标所需的绑定上，避免完整的前向链接物化。

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.plan_cache_reset() → void` | 使所有缓存的查询计划失效 |
| `pg_ripple.explain_sparql(query TEXT, analyze BOOLEAN) → TEXT` | 检查查询计划，可选择实时统计 |

## 相关页面

- [计划缓存](plan-cache.md)
- [SPARQL 参考](sparql.md)
- [架构内部](architecture.md)
- [功能状态分类](feature-status-taxonomy.md)