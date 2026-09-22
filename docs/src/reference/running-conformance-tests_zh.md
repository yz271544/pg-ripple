# 运行一致性测试

pg_ripple 拥有必需的执行门禁、信息性的执行套件和仅源的检查。针对 PostgreSQL 18 运行每个套件，并安装当前的扩展。

## 测试用例采集

采集脚本固定并验证外部数据源：

```sh
bash scripts/fetch_conformance_tests.sh --w3c
bash scripts/fetch_conformance_tests.sh --jena
bash scripts/fetch_conformance_tests.sh --watdiv
bash scripts/fetch_conformance_tests.sh --owl2rl
```

OWL 命令下载并校验和验证 W3C `all.rdf` 语料库。当前的 OWL 测试装置不执行该单体语料库，因此此命令是源检查，而非 OWL 一致性运行。

## W3C SPARQL 1.1

必需套件包含 324 个锁定案例，并串行运行，因为这些案例共享一个数据库。

```sh
cargo pgrx start pg18
cargo test --test w3c_smoke -- --nocapture
cargo test --test w3c_suite -- --nocapture
```

必需 CI 设置 `REQUIRE_CONFORMANCE=1`。然后，缺失的测试用例、空运行、跳过、预期失败或意外失败会使作业失败。

## Apache Jena

Jena 套件是信息性的。它涵盖了其他 SPARQL 查询、更新、语法和代数案例。

```sh
bash scripts/fetch_conformance_tests.sh --jena
cargo test --test jena_suite -- --nocapture
```

## WatDiv 查询执行

当前的 CI 作业对空数据库执行 32 个已检入的 WatDiv 模板。查询错误会使作业失败。它不加载 1000 万个三元组的数据集、不与已填充基线比较结果计数，也不强制延迟阈值。

```sh
cargo test --test watdiv_suite -- --nocapture
```

仅将生成的 WatDiv 数据集用于单独的、明确实记录的本地验证。

## LUBM

必需的 LUBM 套件是自包含的。它针对捆绑的测试用例运行 14 个规范查询，并检查 OWL RL 推导行为。

```sh
cargo test --test lubm_suite -- --nocapture
```

## 已知失败和报告

预期失败放在 `tests/conformance/known_failures.txt` 中。意外通过应删除相应的条目。

执行套件在支持时写入 JSON 报告。CI 工件是特定提交的权威结果；`tests/conformance/history.csv` 是历史数据，不被当前 CI 附加。

## 另请参阅

- [W3C 一致性](w3c-conformance.md)
- [LUBM 结果](lubm-results.md)
- [WatDiv 结果](watdiv-results.md)
- [OWL 2 RL 源状态](owl2rl-results.md)