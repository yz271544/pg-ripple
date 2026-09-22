# CONSTRUCT 回写规则参考

此页面是 pg_ripple 的 CONSTRUCT 回写规则（CWB）的参考。

## 概述

CONSTRUCT 回写规则允许 SPARQL CONSTRUCT 查询充当转换管道：每当三元组被写入源图时，该规则重新运行 CONSTRUCT 查询并将派生的三元组写入目标图。这可以在 PostgreSQL 中完全实现实时、增量更新的规范视图、原始到干净的 ETL 管道和模式映射层。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE 'construct%';
```

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.register_construct_rule(name TEXT, construct_query TEXT, source_graphs TEXT[], target_graph TEXT) → void` | 注册 CONSTRUCT 回写规则 |
| `pg_ripple.drop_construct_rule(name TEXT) → void` | 删除规则及其派生的三元组 |
| `pg_ripple.list_construct_rules() → SETOF record` | 列出所有已注册的规则 |
| `pg_ripple.recompute_construct_rule(name TEXT) → BIGINT` | 触发规则的完整重新计算 |

## 工作原理

1. **注册**：`register_construct_rule()` 将规则存储在 `_pg_ripple.construct_rules` 中。CONSTRUCT 查询被解析和验证。源图和目标图被编码为字典 ID。

2. **触发器**：在对源图的任何写入之后（通过 `mutation_journal::flush()`），为每个受影响的图调用 `construct_rules::on_graph_write()`。

3. **增量维护**：引擎在受影响的源图上重新执行 CONSTRUCT 查询，并使用 Delete-Rederive（DRed）计算差异（减去现有三元组的新三元组、要撤回的已删除三元组）。

4. **回写**：新的三元组被批量插入到目标图中。撤回的三元组被删除。

## 出处

每个派生的三元组在 `_pg_ripple.construct_rule_triples` 中被跟踪，并带有对生成它的规则的引用。当规则被更新或删除时，这可以实现精确的撤回。

## 管道分层

规则按其源/目标图依赖关系进行拓扑排序。循环在注册时被检测和拒绝。规则在每次写入后按依赖顺序触发。

## 性能说明

- 每语句延迟：变更日志在语句期间累积写入并在语句结束时刷新一次，因此单个语句中的 N 个插入只触发一次 CWB 规则，而不是 N 次。
- CONSTRUCT 查询被编译为 SQL 并缓存在计划缓存中。

## 相关页面

- [实时视图和订阅](../features/live-views-and-subscriptions.md)
- [架构内部](architecture.md)
- [功能状态分类](feature-status-taxonomy.md)