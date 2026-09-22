# 性能和一致性结果

此页面将当前的失败关闭证据与历史或信息性检查区分开来。

## 当前一致性证据

| 套件 | 当前结果 | CI 角色 |
|---|---|---|
| W3C SPARQL 1.1 | 324 / 324 锁定案例通过 | 必需 |
| LUBM | 14 / 14 查询通过 | 必需 |
| Apache Jena | 由其套件报告 | 信息性 |
| WatDiv | 32 个模板在空数据库上执行 | 执行检查；无规模或延迟声明 |
| OWL 2 RL | 固定的 489 案例语料库下载并验证 | 仅源检查；无当前一致性结果 |

功能矩阵描述已实现的行为。它们不扩展这些测试结果的范围。

## 当前性能证据

手动 `Benchmark` 工作流程运行有界 `synthetic-100k-v1` 工作负载。它保留：

- 确切的 pg_ripple、PostgreSQL 和 Git 版本；
- 运行器和操作系统详细信息；
- 原始命令输出；
- 包含插入吞吐量和平均点查询和 SPARQL BGP 延迟的 JSON。

所需的 BSBM 作业单独从存储库的适配测试用例加载约一百万个三元组，并在加载、规模或查询错误时失败。它记录经过时间，但没有延迟回归阈值。

检入的 v0.136.0 运行（在提交 `993af4fef88e` 处）在 Apple M1 Pro 上测量了 5,666 插入三元组/秒、0.819 毫秒平均点查询延迟和 0.277 毫秒平均单模式 SPARQL 延迟。[JSON、原始输出和消毒环境](https://github.com/trickle-labs/pg-ripple/tree/main/benchmarks/results) 被一起保留。这是一次有界的运行，不是容量或跨系统的比较。

历史合并和 PageRank CSV 文件具有不兼容的架构，并且没有为当前版本保留的原始证据。它们不是当前基线。

## 尚未通过当前证据验证的内容

- 生产规模或多节点 Citus 性能；
- 1000 万或 1 亿三元组的 WatDiv；
- 持续的 HTTP、队列或混合读/写负载；
- 当前合并工作线程、PageRank、向量或混合搜索比较；
- 长时间浸泡、故障转移、恢复或 PITR 性能。

不要使用存储库数据对这些路径进行容量声明。使用目标数据集、硬件和工作负载测量固定版本。

## 运行有界基准测试

使用新的空数据库。当三元组已存在时，脚本拒绝运行。

```bash
createdb pg_ripple_benchmark
psql -d pg_ripple_benchmark -c 'CREATE EXTENSION pg_ripple CASCADE'
PGDATABASE=pg_ripple_benchmark \
  BENCH_RUNNER=local \
  RESULT_FILE=benchmark_results.json \
  bash benchmarks/ci_benchmark.sh
```

请参阅 [`benchmarks/`](https://github.com/trickle-labs/pg-ripple/tree/main/benchmarks) 了解测试装置和证据要求。

## 另请参阅

- [SPARQL 一致性矩阵](../reference/sparql-compliance.md)
- [WatDiv 状态](../reference/watdiv-results.md)
- [LUBM 结果](../reference/lubm-results.md)
- [OWL 2 RL 源状态](../reference/owl2rl-results.md)