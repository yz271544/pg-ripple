# LUBM 一致性结果

此页面总结了 pg_ripple 在 Lehigh University Benchmark (LUBM) 上的结果，这是 OWL RL 知识库系统的规范基准。

## 概述

LUBM (Guo et al., 2005) 在合成大学域本体（`univ-bench.owl`）上定义了 14 个规范的 SPARQL 查询。这些查询演练：

- 带有 OWL RL 子类/子属性蕴含的 `rdf:type` 查找
- 多跳属性链（`memberOf` + `subOrganizationOf` + `undergraduateDegreeFrom`）
- 域和范围推理
- 已断言和已推断三元组上的合取模式

截至 v0.44.0，所有 14 个 LUBM 查询都通过了捆绑的 `univ1` 合成测试用例，**0 个已知失败**。

## 测试用例

pg_ripple 使用自包含的合成测试用例（`tests/lubm/fixtures/univ1.ttl`），而不是原始的 Java UBA 生成器。该测试用例建模 1 所大学、1 个系、1 个研究组、4 名教职员工、7 名研究生、5 名本科生、6 门研究生课程、1 门本科生课程和 4 篇出版物 —— 全部带有显式的父类型断言，因此无需 OWL RL 推导过程即可匹配参考计数。

互补的 **Datalog 验证子套件**（`tests/lubm/datalog/`）单独验证在相同数据的隐式类型版本上运行 `pg_ripple.load_rules_builtin('owl-rl')` 和 `pg_ripple.infer('owl-rl')` 是否产生相同的查询结果。

## 查询结果（univ1 测试用例）

| 查询 | 描述 | 演练的推导规则 | 预期 | 结果 | 状态 |
|---|---|---|---|---|---|
| Q1  | 选修 GraduateCourse0 的研究生 | `rdf:type` + 子类蕴含 | 3 | 3 | ✅ 通过 |
| Q2  | 其部门是其本科大学一部分的研究生 | 多跳连接 | 2 | 2 | ✅ 通过 |
| Q3  | AssistantProfessor0 的出版物 | 直接查找 | 2 | 2 | ✅ 通过 |
| Q4  | Dept0 中具有姓名/电子邮件/电话的教授 | 属性星形模式 | 4 | 4 | ✅ 通过 |
| Q5  | 选修 GraduateCourse0 的 Dept0 中的人员 | `ub:Person` 父类 | 3 | 3 | ✅ 通过 |
| Q6  | 所有学生 | `ub:Student` 父类 | 12 | 12 | ✅ 通过 |
| Q7  | 由 FullProfessor 指导的选修 GC0 的学生 | 顾问 + 课程合取 | 3 | 3 | ✅ 通过 |
| Q8  | Dept0/University0 中带有电子邮件的学生 | 完全连接模式 | 12 | 12 | ✅ 通过 |
| Q9  | 选修 AssistantProfessor 课程的学生 | 三向连接 | 7 | 7 | ✅ 通过 |
| Q10 | ResearchGroup0 中的研究生 | `ub:memberOf` 查找 | 3 | 3 | ✅ 通过 |
| Q11 | Department0 的子组织 | `ub:subOrganizationOf` | 1 | 1 | ✅ 通过 |
| Q12 | 负责 Department0 的教授 | `ub:headOf` | 1 | 1 | ✅ 通过 |
| Q13 | 担任助教角色的教授 | `ub:teachingAssistantOf` | 1 | 1 | ✅ 通过 |
| Q14 | 所有本科生 | `ub:UndergraduateStudent` 子类 | 5 | 5 | ✅ 通过 |

## Datalog 验证子套件

Datalog 子套件独立于 SPARQL 转换器验证 OWL RL 推导引擎。

| 测试 | 验证内容 | 状态 |
|---|---|---|
| 规则编译 | `load_rules_builtin('owl-rl')` 编译 ≥ 20 条规则 | ✅ 通过 |
| 推导迭代 | `infer_with_stats()` 在 1–10 次迭代内达到不动点 | ✅ 通过 |
| 推断三元组计数 | 关键父类型蕴含产生正确的行计数 | ✅ 通过 |
| 目标查询 | `infer_goal()` 和 SPARQL 计数对 Q1/Q6/Q14 一致 | ✅ 通过 |
| 物化性能 | `infer('owl-rl')` 在 univ1 上 < 5 秒内完成 | ✅ 通过 |
| 自定义规则 | 用户定义的传递闭包规则正确工作 | ✅ 通过 |

## 在本地运行 LUBM

```sh
# 启动 pg_ripple（使用 pgrx 默认端口 28818）
cargo pgrx start pg18

# 运行 LUBM 套件（自包含 —— 无需下载数据）
cargo test --test lubm_suite -- --nocapture
```

要手动运行 Datalog 子套件 SQL 文件：

```sh
# 假设 pg_ripple 已安装并正在运行
psql -c "SELECT pg_ripple.load_turtle(pg_read_file('tests/lubm/fixtures/univ1.ttl'), false)"
psql -f tests/lubm/datalog/rule_compilation.sql
psql -f tests/lubm/datalog/inference_iterations.sql
psql -f tests/lubm/datalog/inferred_triples.sql
```

## 添加已知失败

如果 LUBM 查询失败，请将 `lubm:Q{N}` 条目添加到 `tests/conformance/known_failures.txt`：

```
# 示例 — Q2 由于多跳连接 bug 失败，正在修复
lubm:Q2  multi-hop memberOf/subOrganizationOf join returns wrong count
```

修复底层 bug 后删除该条目。

## 另请参阅

- [运行一致性测试](running-conformance-tests.md) — 本地设置和所有套件
- [W3C 一致性](w3c-conformance.md) — W3C SPARQL 1.1、Jena、WatDiv 结果
- [WatDiv 结果](watdiv-results.md) — 性能基准详细信息