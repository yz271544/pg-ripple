# 功能状态分类

`pg_ripple.feature_status()` 为每个主要功能返回一行。每行包括从七个值的固定词汇表中提取的 `status` 列。本页记录每个状态的含义、其升级条件以及代码库中的一个具体示例。

---

## 状态值

| 状态 | 含义 | 升级条件 | 示例 |
|---|---|---|---|
| `planned` | 路线图项存在；没有面向用户的实现。 | 第一个代码落地；至少一个快乐路径测试通过；添加 CHANGELOG 条目。 → `experimental` | `sparql_12`（在 `plans/sparql12_tracking.md` 中跟踪） |
| `experimental` | 实现存在；文档化限制；可能会更改。 | 添加完整的行为测试矩阵（正面 + 负面 + 错误情况）；更新文档；添加 CI 门控。 → `implemented` | `arrow_flight` — 真正的 IPC 流式传输工作；添加了 nonce 重放保护；票据需要 HMAC 签名 |
| `stub` | API 存在（注册了 SQL 函数）但未实现生产行为。 | 真正的实现取代了 stub；现有调用者继续工作；回归测试涵盖新行为。 → `experimental` | 任何需要尚未连接的外部基础设施的未来函数 |
| `implemented` | 完整执行路径已连接、在 CI 中测试并记录。 | 这是稳定状态。如果删除了必需的依赖项，可能会退化为 `degraded`。 | `sparql_select`、`datalog_inference`、`construct_writeback` |
| `planner_hint` | 发出优化提示或指导，但没有自定义执行器处理该路径。 | 实现完整的自定义执行器；根据参考结果验证。 → `implemented` | `wcoj` — 循环 BGP 连接重排序；未实现真正的 Leapfrog Triejoin 执行器 |
| `manual_refresh` | 功能仅在执行手动操作时才产生正确的输出（例如，`SELECT pg_ripple.refresh_view(...)`）。 | 连接自动刷新或变更检测；不需要手动步骤。 → `implemented` | 添加基于 `cron` 的刷新之前的物化 SPARQL 视图 |
| `degraded` | 缺少必需的依赖项或配置；正在使用回退或无操作。 | 依赖项已安装，功能自动回退到 `implemented`。 | 未安装 `pgvector` 时的 `vector_hybrid_search` |

---

## 升级过程

1. **`planned` → `experimental`**：打开一个 PR，添加第一个实现和至少一个快乐路径回归测试。在 `[Unreleased]` 下更新 CHANGELOG。更新 `src/feature_status.rs` 中的 `feature_status()` 行。

2. **`experimental` → `implemented`**：添加完整的行为测试矩阵（请参阅功能的 pg_regress 测试），更新 `docs/src/` 下的文档页面，并在 `feature_status()` 行的 `ci_gate` 字段中添加 CI 门控条目。

3. **`stub` → `experimental`**：确保现有调用者仍然通过（无破坏性 API 更改）；添加一个练习真实路径的回归测试。

---

## 向 `feature_status()` 添加新功能

1. 在 `src/feature_status.rs` 中的 `vec!` 中添加一个条目。
2. 从上表中选择正确的初始 `status`。
3. 在 `ci_gate` 中填写验证功能的测试（例如，`"ci/regress: my_feature.sql"`）。
4. 在 `docs_path` 中填写主要文档页面。
5. 如果存在证据文件（例如，测试输出或基准基线），请在 `evidence_path` 中填写。
6. 更新 CHANGELOG。

---

## 另请参阅

- `pg_ripple.feature_status()` SQL 函数 — `docs/src/reference/sql-functions.md`
- 存储库中的 `src/feature_status.rs`