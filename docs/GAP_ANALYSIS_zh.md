# pg_ripple 文档 — 差距分析报告

> **状态**：历史审计。已于 2026-05-22 由 [plans/documentation-2.md](../plans/documentation-2.md) 取代。
> 以下计数、版本引用和关键问题反映 v0.99.1 时代的文档状态，不应视为当前的真实来源。

> **日期**：2026-05-11
> **范围**：对 `docs/src/`、`blog/` 的全面审计，与 `src/`、`CHANGELOG.md` 和 `AGENTS.md` 进行交叉引用
> **当前版本**：v0.99.1（即将发布 v1.0.0）

---

## 执行摘要

pg_ripple 文档**基本完整且达到生产质量**。全部 112+ 文档页面均非占位符，代码示例丰富，覆盖了完整的功能集。最薄弱的领域是**参考完整性**（GUC 和 HTTP API 覆盖率），以及**最重要页面中的一个错误的示例**（Hello World）。结构、语气和深度已经达到高标准。

### 数据概览

| 指标 | 数值 | 评估 |
|--------|-------|------------|
| 文档页面总数 | 112+ | 覆盖范围极广 |
| 占位符页面（< 10 行） | 0 | 无缺口 |
| 功能页面完整 | 29/29 (100%) | 优秀 |
| Cookbook 配方 | 14/14 (100%) | 优秀 |
| 运维页面 | 25/25 (100%) | 优秀 |
| 博客文章 | 39 | 强 —— 对所有主要功能覆盖良好 |
| 已记录的 SQL 函数 | 156 个，共约 298 个 `#[pg_extern]`（52%） | **缺口** —— 许多为内部函数但需要审计 |
| 已记录的 GUC 参数 | 197 个已定义中记录 41 个（21%） | **重大缺口** |
| 已记录的 HTTP 端点 | 55+ 路由中记录 5 个（9%） | **重大缺口** |
| 已记录的错误代码 | PT001–PT799（全面） | 优秀 |

---

## 关键问题（阻塞 v1.0.0）

### CRIT-01：Hello World 第 3 步 —— 有缺陷的 SPARQL 查询

**文件**：`docs/src/getting-started/hello-world.md`，第 3 步

查询将 `?movie` 同时重用为电影实体和名称绑定：

```sparql
SELECT ?movie ?director WHERE {
  ?movie schema:director ?person .
  ?movie schema:name ?movie .        -- 错误：?movie 已经绑定为实体
  ?person foaf:name ?director .
}
```

这将产生**不正确的结果**或零行。变量 `?movie` 不能同时是 IRI（实体）和字面量（名称）。应为 `?movieName` 或类似名称。

**影响**：这是文档中阅读量最大的一页。破碎的第一个示例在前 90 秒内就摧毁了信任。

### CRIT-02：Hello World 第 3 步 —— 变量命名不一致

查询返回 `?movie` 和 `?director`，但正文说"你应该看到由'Alice'导演的'The Graph'"。`?director` 变量绑定到*人员的名称*（通过 `foaf:name`），而不是导演实体 —— 命名混乱，会误导读者。

### CRIT-03：GUC 参考严重不完整

**文件**：`docs/src/reference/guc-reference.md`

仅记录了 197 个 GUC 参数中的 **41** 个（21%）。缺失的类别包括：

- **PageRank**（28 个 GUC：`pagerank_damping`、`pagerank_incremental`、`pagerank_partition` 等）
- **Datalog**（约 33 个 GUC：`magic_sets`、`dred_enabled`、`tabling`、`probabilistic_datalog`、`owl_profile` 等）
- **存储**（约 44 个 GUC：`merge_threshold`、`citus_sharding_enabled`、`arrow_flight_secret` 等）
- **SPARQL**（约 22 个 GUC：`wcoj_enabled`、`star_join_collapse`、`sparql_max_rows`、`sparql_max_algebra_depth` 等）
- **可观测性**（6 个 GUC：`tracing_enabled`、`audit_log_enabled` 等）

任何 DBA 调整生产部署都会遇到未记录的参数。

### CRIT-04：HTTP API 参考 —— 不完整

**文件**：`docs/src/reference/http-api.md`

仅记录 5 个端点。实际的 `pg_ripple_http` 服务公开了 **55+ 路由**，包括：

- `/datalog/rules/*`（24 路由）—— 规则管理、推理、视图、约束
- `/pagerank/*`、`/centrality/*`（10 路由）
- `/confidence/*`（4 路由）
- `/subscribe/*`（SSE/WebSocket）
- `/openapi.yaml`
- `/explorer`
- `/flight/do_get`（Arrow Flight）
- `/health/ready`、`/metrics/extension`、`/void`、`/service`

---

## 高优先级改进

### HIGH-01：着陆页 —— 缺少人物角色决策树

**文件**：`docs/src/landing.md`

"Next steps" 部分有针对人物角色的链接，这很好。但是，没有可视化的决策树（流程图或矩阵）来立即引导新访问者。与 Prisma 文档着陆页（一眼就能看到三条清晰的路径）相比。

### HIGH-02：教程 —— 缺少"你将学到什么"前言

**文件**：`docs/src/getting-started/tutorial.md`

教程具有良好的结构（四段、时间估计），但顶部没有"你将学到什么"摘要。每个段应以一句话的学习目标开头。

### HIGH-03：Docker 快速入门 —— 不够突出

**文件**：`docs/src/getting-started/installation.md`

Docker 列在第一位（很好），但该部分可以更突出。像 `> **最快路径**：docker compose up -d` 这样的提示框会有所帮助。

### HIGH-04：SQL 函数参考 —— `#[pg_extern]` 覆盖率 52%

**文件**：`docs/src/reference/sql-functions.md`

在约 298 个 `#[pg_extern]` 定义中记录了 156 个函数。许多未记录的可能是内部的，但需要审计以确定哪些面向公众的函数是缺失的。`user-guide/sql-reference/` 部分添加了详细信息，但可能未涵盖所有内容。

### HIGH-05：博客文章 —— 功能页面无交叉链接

博客文章是独立的，不从相关功能深入页面链接。例如：
- `blog/dictionary-encoding-integer-joins.md` 应从关键概念页面链接
- `blog/htap-reads-and-writes.md` 应从 HTAP 功能页面链接
- `blog/leapfrog-triejoin.md` 应从高级推理链接

### HIGH-06：版本引用

`pg_ripple.control` 显示 v0.99.1，但各种文档可能仍引用旧版本。需要进行扫描以确保版本一致性。

### HIGH-07：缺少生产就绪检查清单

没有单一的"生产就绪检查清单"页面。运维页面涵盖各个主题（安全、HA、监控等），但没有 DevOps 工程师在上线前可以走查的合并检查清单。`docs/src/operations/bidi-production-checklist.md` 专门针对双向中继存在，但不针对整个 pg_ripple。

---

## 中优先级改进

### MED-01：术语不一致

- "triple" 与 "statement" —— 在一些地方可互换使用
- "graph" 与 "named graph" —— 有时模棱两可
- "rule" 与 "constraint" —— Datalog 约束与 SHACL 约束需要更清晰的区别

### MED-02：某些功能页面缺少交叉链接

功能页面通常有"Next steps"，但并非所有页面都交叉引用相关的 cookbook 配方或博客文章。

### MED-03：缺少教程 —— 从 Neo4j 迁移

存在 LPG → RDF 映射功能页面（`features/lpg-mapping.md`），但没有端到端的迁移配方。

### MED-04：缺少教程 —— 从头开始构建 RAG 管道

`user-guide/rag-pipeline.md` 和 `cookbook/grounded-chatbot.md` 涵盖了 RAG，但没有从"空数据库"到"工作的 LangChain + pg_ripple 聊天机器人"的单一端到端教程。

### MED-05：缺少教程 —— Kubernetes 生产设置

`operations/kubernetes.md` 存在，但从 Helm 安装到运行查询的逐步教程会有所帮助。

### MED-06：缺少"限制和配额"页面

没有页面记录经过测试的限制：最大三元组数、最大谓词数、最大 SPARQL 查询复杂度等。

### MED-07：提示框系统不一致

某些页面使用 `````admonish note`````（mdBook admonish 插件），其他页面使用 `> **Note**：` markdown 引用块。应进行标准化。

### MED-08：缺少 Mermaid 架构图

着陆页有 ASCII 架构图。HTAP、CDC 和联邦的功能深入页将受益于使用正确的 Mermaid 图。

---

## 低优先级 / 可有可无

### LOW-01：博客文章新鲜度审计

存在 39 篇博客文章。有些可能引用了较旧的 API 模式。扫描以更新或使用版本注释进行标注将是很有价值的。

### LOW-02：词汇表可以更全面

`reference/glossary.md` 存在，但可以使用更多 pg_ripple 特定术语进行扩展。

### LOW-03：FAQ 可以扩展

`reference/faq.md` 存在，但可以根据支持模式处理更多常见问题。

### LOW-04：搜索优化

mdBook 搜索有效，但页面标题可以更利于搜索（例如，"SPARQL 查询参考"而非仅仅是"SPARQL 参考"）。

### LOW-05：代码示例语言标签

一些代码块可能缺少语法高亮的语言标签。扫描将捕获这些。

---

## 覆盖矩阵

| 功能 | 功能页面 | 教程 | Cookbook | 参考 | 博客文章 |
|---------|:-----------:|:--------:|:-------:|:---------:|:---------:|
| SPARQL 1.1 查询 | ✅ | ✅ | ✅ | ✅ | ✅ |
| SPARQL 1.1 更新 | ✅ | — | — | — | — |
| SPARQL 联邦 | ✅ | — | ✅ | ✅ | ✅ |
| SHACL 核心 | ✅ | ✅ | ✅ | ✅ | ✅ |
| SHACL-SPARQL 规则 | ✅ | — | ✅ | ✅ | — |
| Datalog（基础） | ✅ | ✅ | ✅ | ✅ | ✅ |
| Datalog（magic sets） | ✅ | — | — | ✅ | ✅ |
| Datalog（WFS/tabling） | ✅ | — | — | ✅ | ✅ |
| Datalog（聚合） | ✅ | — | — | ✅ | — |
| HTAP 存储 | ✅ | — | — | ✅ | ✅ |
| RDF-star | ✅ | — | — | ✅ | ✅ |
| JSON-LD 框架 | ✅ | ✅ | — | ✅ | ✅ |
| CONSTRUCT/DESCRIBE/ASK 视图 | ✅ | — | — | ✅ | ✅ |
| 向量 + 混合搜索 | ✅ | — | — | ✅ | ✅ |
| GraphRAG | ✅ | — | — | ✅ | ✅ |
| PageRank | ✅ | — | — | ✅ | ✅ |
| 概率推理 | ✅ | — | ✅ | ✅ | ✅ |
| 双向中继（CDC） | ✅ | — | ✅ | ✅ | ✅ |
| CDC 订阅 | ✅ | — | — | ✅ | ✅ |
| EXPLAIN SPARQL | ✅ | — | — | ✅ | ✅ |
| 基于成本的联邦 | ✅ | — | ✅ | ✅ | ✅ |
| Citus 集成 | ✅ | — | — | ✅ | ✅ |
| 多租户图 | ✅ | — | — | ✅ | ✅ |
| GeoSPARQL | ✅ | — | — | ✅ | ✅ |
| 全文搜索 | ✅ | — | — | ✅ | — |
| 时序/出处 | ✅ | — | ✅ | ✅ | ✅ |
| R2RML | ✅ | — | ✅ | ✅ | ✅ |
| Cypher/LPG → RDF | ✅ | — | — | — | — |
| KG 嵌入 | ✅ | — | ✅ | ✅ | ✅ |
| NL-to-SPARQL | ✅ | — | ✅ | ✅ | ✅ |
| AI 代理集成 | ✅ | — | — | — | — |
| 记录链接 | ✅ | — | ✅ | ✅ | ✅ |
| SKOS | — | — | ✅ | — | ✅ |
| DCTERMS/Schema.org/FOAF | — | — | ✅ | — | ✅ |
| OWL 2 RL/EL/QL | ✅ | — | — | ✅ | — |
| Arrow Flight | — | — | — | ✅ | — |
| 网格 Datalog | ✅ | — | — | ✅ | — |
| WCOJ | ✅ | — | — | — | ✅ |

---

## 缺少的教程（按优先级排序）

1. **从 Neo4j 迁移到 pg_ripple** —— 来自目标受众的最高需求
2. **从头开始构建 RAG 管道**（使用 LangChain 的端到端）—— AI 人物角色关键路径
3. **Kubernetes 生产部署** —— ops 人物角色关键路径
4. **GDPR 合规性和被遗忘权** —— 企业需求
5. **多租户 SaaS 知识图谱** —— 生产模式
6. **使用 CDC 的实时仪表板** —— 流式用例

---

## 结构评估

### 优点

- **导航结构**：清晰的分类法（评估 → 入门 → 功能 → Cookbook → 运维 → 参考）
- **SUMMARY.md**：全面、组织良好，无孤立页面
- **语气**：温暖、直接、同行级别 —— 避免学术术语
- **代码示例**：94%+ 的页面包含正确的语言标签
- **错误目录**：全面的 PT001–PT799，包含每个代码的原因和解决方法
- **故障排除**：真实的失败场景及诊断步骤
- **博客**：39 篇涵盖所有主要功能
- **一致性页面**：W3C、Jena、WatDiv、LUBM、OWL 2 RL 全部记录

### 需要改进的地方

- **参考完整性**：GUC 和 HTTP API 覆盖率是最大的缺口
- **交叉链接**：博客 → 功能页面，反之亦然
- **提示框一致性**：admonish 插件和 markdown 引用块的混合
- **Hello World 正确性**：最关键的页面有错误的示例