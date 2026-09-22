# SHACL 参考

此页面是 pg_ripple 的 SHACL（Shapes Constraint Language，形状约束语言）引擎的参考。

## 概述

pg_ripple 实现了 SHACL Core 和 SHACL-SPARQL 约束集。使用 `pg_ripple.load_shacl()` 从三元组存储加载形状定义。约束被编译为 DDL CHECK 约束（用于同步验证）和由后台工作线程支持的异步验证管道（用于复杂形状层次结构）。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE 'shacl%';
```

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.load_shacl(graph_iri TEXT) → void` | 从命名图加载 SHACL 形状 |
| `pg_ripple.validate_shacl(graph_iri TEXT) → SETOF record` | 根据已加载形状验证命名图 |
| `pg_ripple.list_shapes() → SETOF record` | 列出所有已加载的 SHACL 形状 |
| `pg_ripple.drop_shape(shape_iri TEXT) → void` | 删除形状及其约束 |

## SHACL Core 约束覆盖范围

实现了所有 35 个 SHACL Core 约束组件：

- **属性约束**：`sh:minCount`、`sh:maxCount`、`sh:minLength`、`sh:maxLength`、`sh:pattern`、`sh:languageIn`、`sh:uniqueLang`、`sh:equals`、`sh:disjoint`、`sh:lessThan`、`sh:lessThanOrEquals`
- **值约束**：`sh:in`、`sh:hasValue`、`sh:class`、`sh:datatype`、`sh:nodeKind`
- **形状约束**：`sh:node`、`sh:property`、`sh:qualifiedValueShape`、`sh:qualifiedMinCount`、`sh:qualifiedMaxCount`
- **逻辑约束**：`sh:and`、`sh:or`、`sh:not`、`sh:xone`
- **封闭形状**：`sh:closed`、`sh:ignoredProperties`

## SHACL-SPARQL 约束

通过形状节点上的 `sh:sparql` 支持使用 SPARQL SELECT 和 ASK 的自定义约束组件。约束查询被编译为 pg_trickle 流表，用于连续验证。

## SHACL-AF 规则执行

支持 `sh:rule` 三元组规则（SHACL Advanced Features），并编译为 CONSTRUCT 回写规则，允许形状从现有数据派生新三元组。

## 性能说明

- `sh:maxCount 1` 提示会传播到 SPARQL 计划器：省略 DISTINCT。
- `sh:minCount 1` 提示允许将 LEFT JOIN 重写为 INNER JOIN。
- 形状提示缓存在 `_pg_ripple.shape_hints` 中，并在计划时查询。

## 相关页面

- [SHACL SQL 参考](../user-guide/sql-reference/shacl.md)
- [验证数据质量](../features/validating-data-quality.md)
- [功能状态分类](feature-status-taxonomy.md)