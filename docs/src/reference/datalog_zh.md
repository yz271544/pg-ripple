# Datalog 参考

此页面是 pg_ripple 的 Datalog 推理引擎的参考。

## 概述

pg_ripple 包含一个完整的 Datalog 引擎，它将 Datalog 规则编译为递归 SQL（`WITH RECURSIVE`），在 PostgreSQL 中执行推理，并将派生的三元组具体化回三元组存储。该引擎支持分层否定、半朴素求值、聚合（`Datalog^agg`）、用于目标导向推理的 magic 集、`owl:sameAs` 实体规范化、循环本体的良好基础语义、列表化、Delete-Rederive（DRed）用于撤回以及并行分层求值。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE 'datalog%';
```

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.load_rules(rules TEXT, rule_set TEXT DEFAULT 'custom') → BIGINT` | 解析和存储 Datalog 规则集；返回规则计数 |
| `pg_ripple.load_rules_builtin(name TEXT) → BIGINT` | 加载内置规则集（`'rdfs'` 或 `'owl-rl'`） |
| `pg_ripple.add_rule(rule_set TEXT, rule_text TEXT) → BIGINT` | 将单个规则添加到现有规则集；返回新规则 ID |
| `pg_ripple.remove_rule(rule_id BIGINT) → BIGINT` | 按目录 ID 删除规则；返回撤回的三元组 |
| `pg_ripple.drop_rules(rule_set TEXT) → BIGINT` | 删除命名规则集中的所有规则；返回规则计数 |
| `pg_ripple.enable_rule_set(name TEXT) → void` | 启用规则集而不重新加载 |
| `pg_ripple.disable_rule_set(name TEXT) → void` | 禁用规则集而不删除它 |
| `pg_ripple.list_rules() → JSONB` | 列出所有存储的规则及其 id、rule_set、rule_text、stratum、active |
| `pg_ripple.list_rule_sets() → TABLE(rule_set, active, rule_count, created_at)` | 列出所有命名规则集 |
| `pg_ripple.infer(rule_set TEXT DEFAULT 'custom') → BIGINT` | 运行正向链接推理；返回三元组计数 |
| `pg_ripple.infer_with_stats(rule_set TEXT DEFAULT 'custom') → JSONB` | 运行半朴素推理并提供详细统计信息 |
| `pg_ripple.infer_goal(rule_set TEXT, goal TEXT) → JSONB` | 使用 magic 集进行目标导向推理 |
| `pg_ripple.infer_agg(rule_set TEXT DEFAULT 'custom') → JSONB` | 为聚合规则运行 Datalog^agg 推理 |
| `pg_ripple.infer_wfs(rule_set TEXT DEFAULT 'custom') → JSONB` | 循环程序的良好基础语义推理 |
| `pg_ripple.infer_lattice(rule_set TEXT, lattice_name TEXT) → JSONB` | 基于 lattice 的单调不动点推理 |
| `pg_ripple.retract_inferred(rule_set TEXT) → BIGINT` | 删除规则集的所有具体化三元组；返回计数 |
| `pg_ripple.check_constraints(rule_set TEXT DEFAULT NULL) → JSONB` | 运行完整性约束规则；返回违规 |
| `pg_ripple.explain_inference(s TEXT, p TEXT, o TEXT, g TEXT DEFAULT NULL) → TABLE` | 返回推导三元组的推导树 |
| `pg_ripple.explain_datalog(rule_set_name TEXT) → JSONB` | 完整的解释文档：strata、规则、SQL、上次运行统计信息 |
| `pg_ripple.dred_on_delete(pred_id BIGINT, s BIGINT, o BIGINT, g BIGINT) → BIGINT` | 已删除基础三元组的手动 DRed 撤回 |

## 规则语法

规则使用 Turtle 风格的 IRI 表示法或前缀限定的名称：

```
:ancestor(?x, ?z) :- :parent(?x, ?y), :ancestor(?y, ?z).
:ancestor(?x, ?y) :- :parent(?x, ?y).
```

内置 RDFS/OWL RL 规则包括在内，并在 `pg_ripple.enable_owl_rl = true`（默认值：false）时自动激活。

## 推理架构

1. 规则被解析和分层（通过 WFS 进行循环规则的失败即否定）。
2. 每个层被编译为 `WITH RECURSIVE` SQL 查询。
3. 半朴素求值跟踪迭代之间的增量。
4. Magic 集转换规则以进行需求驱动（目标导向）求值。
5. 派生的三元组被插入到三元组存储中，`source = 1`。

## OWL RL 支持

内置的 OWL 2 RL 规则集涵盖了完整的约 100 个 OWL RL 规则，包括：

- 类和属性层次结构（`rdfs:subClassOf`、`rdfs:subPropertyOf`）
- 逆、对称、传递和函数属性
- `owl:allValuesFrom`、`owl:someValuesFrom`、`owl:hasValue`
- `owl:sameAs` 规范化

## 相关页面

- [Datalog SQL 参考](../user-guide/sql-reference/datalog.md)
- [基于 Lattice 的 Datalog](lattice-datalog.md)
- [LUBM OWL RL 结果](lubm-results.md)
- [OWL 2 RL 结果](owl2rl-results.md)
- [功能状态分类](feature-status-taxonomy.md)