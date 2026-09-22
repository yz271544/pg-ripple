# Jena 一致性测试套件结果

Apache Jena SPARQL 1.1 一致性测试套件（约 1,000 个测试）涵盖了完整的 SPARQL 1.1 查询和更新规范。pg_ripple 目标是在 v0.55.0 时在此套件上达到 ≥95% 的通过率。

## Jena 套件测试内容

Jena 一致性测试源自 W3C SPARQL 1.1 测试套件，并由 Apache Jena 自己的测试用例补充。它们涵盖：

| 类别 | 测试（约） | 测试内容 |
|---|---|---|
| 基本图模式 | ~150 | BGP 匹配、三元组模式、空白节点 |
| SPARQL 代数 | ~200 | UNION、OPTIONAL、MINUS、JOIN |
| 聚合 | ~80 | COUNT、SUM、AVG、GROUP BY、HAVING |
| 属性路径 | ~100 | `*`、`+`、`?`、`|`、`^`、`/` 运算符 |
| SPARQL 更新 | ~150 | INSERT DATA、DELETE DATA、MODIFY、COPY、MOVE |
| SPARQL 协议 | ~50 | 内容协商、结果格式 |
| 数字/字符串函数 | ~120 | 所有 SPARQL 内置函数 |
| RDF-star | ~50 | 引号三元组、TRIPLE()、isTriple() |
| 子查询 / VALUES | ~60 | 相关子 SELECT、VALUES 行 |
| 其他 | ~40 | 其他边缘情况 |

## 通过率

通过 CI 在每次推送到 `main` 时使用 `cargo test --test jena_suite` 测量通过率。当前通过率为 ≥95%。

徽章状态由 CI `jena-suite` 作业自动更新。

## 已知排除项

Jena 套件中的一些测试会测试非阻塞功能，直到在 ≥95% 通过率时强制执行完整套件要求：

- 需要通过 SPARQL UPDATE 进行命名图写操作的测试与 PostgreSQL 事务语义冲突（特定于平台）
- 针对 `spargebra` SPARQL 1.2 语法支持的功能门控测试

## 在本地运行

```bash
# 获取 Jena 测试数据（需要网络访问）
bash scripts/fetch_conformance_tests.sh --jena

# 运行套件
cargo test --test jena_suite -- --nocapture
```

该套件在 `tests/conformance/report.json` 处生成报告。