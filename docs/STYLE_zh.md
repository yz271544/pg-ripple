# pg_ripple 文档样式指南

> 本指南适用于 `docs/src/` 下的所有页面。生成的参考页面（GUC 表、SQL 函数列表、HTTP 端点清单）免于散文样式规则，但必须遵循标题和代码栅栏约定。

## 1. 指导原则

- **当前行为优先。** 描述现在真实的情况。将版本历史放在注释或该部分底部的"Since"行中。
- **运维框架。** 为设置或调试部署的从业者编写，而不是为阅读设计文档的研究人员编写。
- **不要虚假精确。** 除非链接了基准测试文件或 CI 结果，否则不要声明确切的吞吐量数字。
- **简短是好的。** 一个必须滚动浏览 200 行背景才能到达第一个 SQL 示例的读者将会放弃。

---

## 2. 页面类型和职责

| 类型 | 目的 | 所在位置 |
|---|---|---|
| 功能页面 | 解释概念及其主要工作流 | `docs/src/features/` |
| 参考页面 | 列举确切的 API、签名、GUC、限制、错误代码 | `docs/src/reference/` |
| 指南/演练 | 具有先决条件和决策点的端到端任务 | `docs/src/guides/` |
| Cookbook 配方 | 正好解决一个具体问题，最少的迂回 | `docs/src/cookbook/` |
| 运维页面 | 部署、失败模式和运维调优 | `docs/src/operations/` |
| 入门页面 | 为首次读者提供的短成功路径 | `docs/src/getting-started/` |

**不要跨页面类型复制内容。** 而是互相链接。
Cookbook 配方可以引用功能页面作为背景，但不得冗长地重新解释概念。

---

## 3. 扩展和配套服务名称

| 名称 | 它指的是什么 | 不用于 |
|---|---|---|
| `pg_ripple` | PostgreSQL 扩展本身 | HTTP 配套服务 |
| `pg_ripple_http` | 独立 HTTP/Flight 配套服务 | PG 扩展 |
| `pg_trickle` | IVM 支持的视图、ExtVP、SHACL DAG 监视器、实时统计 | 中继或 CDC 传递 |
| `pg_tide` | 中继/outbox/inbox 传输、CDC 传递桥 | IVM 视图 |

**仅在代码路径明确要求 `pg_ripple.trickle_integration = on` 或调用 pg_trickle 扩展时使用 `pg_trickle`**。

**仅在代码路径涉及 outbox/inbox 中继协议或 `pg_tide_available()` 函数保护时使用 `pg_tide`**。

---

## 4. SQL 格式

- 对 SQL 示例使用 `` sql `` 代码栅栏。
- 仅对 PostgreSQL 方言特定内容（例如 `EXPLAIN` 输出、`pg_dump` 调用）使用 `postgresql` 栅栏。
- 首次提及时使用 schema 限定函数名：`pg_ripple.sparql_select(...)`，而不是 `sparql_select(...)`。
- 在示例中使用小写 SQL 关键字（`select`、`insert`、`where`、`from`），仅在引用 PostgreSQL 文档或解释解析规则时使用大写。
- 标记无法在外部服务下运行的示例：

  ```sql
  -- requires: pg_tide (relay), pg_trickle (IVM), pgvector, PostGIS
  ```

---

## 5. 代码块标签

每个代码块应为以下之一：

| 标签 | 含义 | 如何标记 |
|---|---|---|
| 可执行 | 在新的 pg_ripple 数据库上运行 | 不需要特殊注释 —— 除非另有标记，否则假定 |
| 说明性 | 概念性的；可能使用省略号或占位符值 | 将 `-- illustrative` 注释为第一行 |
| 伪输出 | 显示预期的终端或查询输出 | 使用纯文本或 `text` 栅栏 |

---

## 6. 提示框

对提示框使用 mdbook-admonish 语法。优先使用以下四个：

```
> **Note**: short informational aside.
```

```admonish note
Short informational aside.
```

```admonish warning
Something the reader might do wrong.
```

```admonish tip
Shortcut or best practice.
```

```admonish danger
Data-loss or security risk.
```

当 `mdbook-admonish` 可用时，不要使用 `> **Warning:**` 或 `> **Note:**` 原始引用块（CI 使用它；本地构建优雅降级）。

---

## 7. 标题和结构

- `#` 保留给页面标题。
- 对主要部分使用 `##`，对子部分使用 `###`。
- 不要跳过标题级别。
- 保持标题文本为句子大小写："SPARQL query execution" 而不是 "SPARQL Query Execution"。

---

## 8. 链接

- 在 `docs/src/` 中的页面之间使用相对链接。
- 不要使用从 `docs/src/` 到顶级 `blog/` 或 `plans/` 目录的相对路径 —— 而是使用绝对 GitHub URL：
  `https://github.com/trickle-labs/pg-ripple/blob/main/blog/pagerank.md`
- 外部链接仅在计划的 CI 中检查，而不是在每个 PR 上。

---

## 9. 版本引用

- 对于功能可用性，使用简短的 "Since v0.X.Y" 注释，放在相关部分的末尾或前面表格中。
- 不要在运行散文里写 "as of v0.X.Y" —— 它很快就会过时，并且在更新版本发布后会令人困惑。
- 当某个功能被移除或替换时，说明是什么取代了它。

---

## 10. 可选依赖

每个描述需要可选扩展或服务的功能的页面都应在顶部包含一个前面表格形式的依赖表：

```
| Dependency | Required? | Notes |
|---|---|---|
| `pg_trickle` | Optional | IVM live-view updates only |
| `pg_tide` | Optional | Relay/outbox delivery only |
| `pgvector` | Optional | Vector hybrid search |
| `PostGIS` | Optional | Geospatial SPARQL functions |
| `pg_ripple_http` | Optional | REST/Arrow Flight access |
| Citus | Optional | Horizontal sharding |
```

---

## 11. PR 检查清单

打开文档 PR 时，验证：

- [ ] `python3 scripts/check_docs_links.py` 退出码为 0。
- [ ] `python3 scripts/check_docs_summary.py` 退出码为 0。
- [ ] `mdbook build docs` 成功。
- [ ] 新路由在 `docs/src/reference/http-api.md` 中。
- [ ] 新 GUC 在 `docs/src/reference/guc-reference.md` 和 `docs/gucs.md` 中。
- [ ] 新 SQL 函数在 SQL 参考中或明确标记为内部函数。
- [ ] 可选依赖表存在于需要它们的功能页面上。
- [ ] 代码示例遵循第 5 节标记约定。