# WatDiv 查询执行状态

[WatDiv](https://dsg.uwaterloo.ca/watdiv/) 定义了对星形、链形、雪花形和复杂图模式施加压力的查询模板。

## 当前 v0.136.0 CI 覆盖

当前作业执行 32 个已检入的模板：

| 类 | 模板 |
|---|---:|
| 星形 | 7 |
| 链形 | 3 |
| 雪花形 | 5 |
| 复杂 | 17 |

CI 在空数据库中创建一个干净的扩展，并在查询执行错误时失败。它不加载 WatDiv 数据集。已检入的延迟基线为空，因此该作业不能在 1000 万个三元组的规模上建立正确性，也不强制执行性能阈值。

```sh
cargo test --test watdiv_suite -- --nocapture
```

## 规模验证运行必须保留的内容

可重现的 WatDiv 结果必须标识生成器修订、数据集大小和校验和、具体替换、PostgreSQL 和 pg_ripple 版本、硬件、原始结果计数和原始计时。当前版本中尚未检入此类结果。

## 另请参阅

- [运行一致性测试](running-conformance-tests.md)
- [性能结果](../evaluate/performance-results.md)