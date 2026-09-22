# pg_ripple 文档 — 改进计划

> **状态**：历史计划。已于 2026-05-22 由 [plans/documentation-2.md](../plans/documentation-2.md) 取代。
> 以下项目反映 v0.99.1 时代的审计，并出于上下文保留，
> 而非当前的文档积压。

> **日期**：2026-05-11
> **输入**：[GAP_ANALYSIS.md](GAP_ANALYSIS.md)
> **排序**：关键 → 高 → 中 → 低。在每个层级内，项目按影响和依赖关系排序。

---

## 关键优先级 —— 必须在 v1.0.0 之前修复

---

## DOC-001：修复 Hello World 有缺陷的 SPARQL 查询

**优先级**：关键
**工作量**：S（< 1 天）
**负责领域**：入门
**阻塞**：无 —— 但阻塞用户信任

### 现状

`hello-world.md` 的第 3 步将 `?movie` 同时重用为实体变量和名称绑定：
```sparql
?movie schema:name ?movie .  -- 变量冲突
```
第 3 步的说明文字也有令人困惑的变量名（`?director` 实际上是人员名称）。

### 需要更改的内容

- 将名称绑定中的 `?movie` 重命名为 `?movieName`
- 将 `?director` 重命名为 `?directorName` 以与第 4 步保持一致
- 验证演练中的所有三个查询都产生所描述的结果

### 完成标准

- [ ] 第 3 步查询使用不同的变量名（`?movieName`、`?directorName`）
- [ ] 第 3 步说明文字与查询输出匹配
- [ ] 三个查询（第 3、4、5 步）内部一致

---

## DOC-002：完成 GUC 参考 —— 完整覆盖率

**优先级**：关键
**工作量**：L（3–7 天）
**负责领域**：参考
**阻塞**：DOC-030（生产检查清单取决于 GUC 知识）

### 现状

在 197 个 GUC 参数中记录了 41 个。缺失：PageRank（28）、Datalog（33）、存储（44）、SPARQL（22）、可观测性（6）、SHACL（3+）、LLM（14+）。

### 需要更改的内容

审计 `src/gucs/` 中的每个 `GucSetting` 定义。对于每个，添加到 `guc-reference.md`：
- 参数名称（带 `pg_ripple.` 前缀）
- 类型、默认值、范围
- 描述
- 何时更改它
- 相关的交互说明

按子系统组织，章节标题清晰。

### 完成标准

- [ ] `src/gucs/*.rs` 中的每个 `pub static` GucSetting 都有相应的条目
- [ ] 每个条目都有：类型、默认值、范围、描述和"何时更改"指南
- [ ] 子系统分组与源模块结构匹配

---

## DOC-003：完成 HTTP API 参考

**优先级**：关键
**工作量**：M（1–3 天）
**负责领域**：参考
**阻塞**：DOC-041（OpenAPI 规范）

### 现状

记录了 5 个端点。源中存在 55+ 路由。

### 需要更改的内容

记录 `pg_ripple_http/src/routing/` 中的所有路由：
- Datalog API（24 路由）
- PageRank/Centrality（10 路由）
- Confidence（4 路由）
- 流式/Subscribe 路由
- 管理路由（`/openapi.yaml`、`/explorer`、`/void`、`/service`、`/health/ready`、`/metrics/extension`）
- Arrow Flight（`/flight/do_get`）

每个端点需要：方法、路径、请求/响应格式、身份验证要求、示例。

### 完成标准

- [ ] `routing/*.rs` 中的每个路由都有相应的条目
- [ ] 每个条目都有方法、路径、描述、请求/响应示例
- [ ] 每个端点都注明了身份验证要求

---

## DOC-004：审计并验证 SQL 函数参考

**优先级**：关键
**工作量**：M（1–3 天）
**负责领域**：参考
**阻塞**：无

### 现状

记录了 156 个函数。源中有 298 个 `#[pg_extern]` 定义（许多是内部的）。

### 需要更改的内容

- 交叉引用每个 `#[pg_extern]` 与已记录列表
- 识别哪些是面向公众的 SQL 函数与内部辅助函数
- 添加任何缺失的公共函数
- 验证签名与源代码匹配

### 完成标准

- [ ] 每个公共 `pg_ripple.*` SQL 函数都已记录
- [ ] 文档与源之间没有签名不匹配
- [ ] 计数标头更新以反映实际数量

---

## 高优先级 —— 应在 v1.0.0 之前修复

---

## DOC-010：添加生产就绪检查清单

**优先级**：高
**工作量**：M（1–3 天）
**负责领域**：运维
**阻塞**：DOC-002（需要完整的 GUC 知识）

### 现状

各个运维页面涵盖安全、HA、监控等。没有合并的检查清单。

### 需要更改的内容

创建 `docs/src/operations/production-checklist.md`，包含以下部分：
- 部署前（PostgreSQL 配置、shared_preload_libraries、GUC 调优）
- 安全（RLS、身份验证令牌、TLS）
- 监控（Prometheus 告警、关键指标、仪表板示例）
- 备份和恢复（pg_dump、WAL 归档、PITR）
- 性能（合并工作线程、缓存大小、vacuum 调优）
- 升级路径（迁移脚本、兼容性矩阵）

添加到 `SUMMARY.md`。

### 完成标准

- [ ] 检查清单页面存在，包含可操作的项目
- [ ] 每个项目链接到相关的详细页面
- [ ] 添加到 SUMMARY.md 的运维部分

---

## DOC-011：着陆页人物角色决策树

**优先级**：高
**工作量**：S（< 1 天）
**负责领域**：入门
**阻塞**：无

### 现状

着陆页有"Next steps"，带有人物角色定向链接。没有可视化的决策树。

### 需要更改的内容

添加清晰的"从这里开始"部分，包含三条路径：
1. **PostgreSQL DBA** → 安装 → Hello World → 运维
2. **数据/AI 工程师** → AI 概述 → RAG 管道 → 基于知识的聊天机器人
3. **语义 Web 工程师** → 关键概念 → SPARQL → Datalog → SHACL

使用可立即扫描的表格或结构化布局。

### 完成标准

- [ ] 三个人物角色决策矩阵在第一屏内容中可见
- [ ] 每条路径有 3-4 个链接步骤
- [ ] 保留现有的"Next steps"部分

---

## DOC-012：将博客交叉链接添加到功能页面

**优先级**：高
**工作量**：M（1–3 天）
**负责领域**：功能 / 博客
**阻塞**：无

### 现状

大多数功能都存在博客文章，但未从功能页面链接。

### 需要更改的内容

对于每个功能页面，添加"延伸阅读"部分链接到相关的博客文章：

| 功能页面 | 博客文章 |
|---|---|
| `key-concepts.md` | `dictionary-encoding-integer-joins.md`、`vertical-partitioning-explained.md` |
| `querying-with-sparql.md` | `sparql-to-sql-translation.md`、`property-paths-recursive-ctes.md` |
| `reasoning-and-inference.md` | `datalog-inside-postgresql.md`、`builtin-reasoning-rules-explained.md` |
| `advanced-inference.md` | `leapfrog-triejoin.md`、`well-founded-semantics.md`、`magic-sets-goal-directed.md` |
| `validating-data-quality.md` | `shacl-data-quality.md` |
| `live-views-and-subscriptions.md` | `construct-views-live-transformations.md`、`ivm-pg-trickle-integration.md` |
| `vector-and-hybrid-search.md` | `vector-sparql-hybrid-search.md` |
| `pagerank.md` | `pagerank.md`（博客） |
| `uncertain-knowledge.md` | `probabilistic-datalog.md` |
| `cdc-subscriptions.md` | `cdc-knowledge-graphs.md` |
| `geospatial.md` | `geosparql-postgis-spatial.md` |
| `graphrag.md` | `graphrag-knowledge-export.md` |
| `multi-tenant-graphs.md` | `multi-tenant-knowledge-graphs.md` |
| `nl-to-sparql.md` | `natural-language-to-sparql.md` |
| `record-linkage.md` | `neuro-symbolic-entity-resolution.md`、`owl-sameas-entity-resolution.md` |
| `r2rml.md` | `r2rml-relational-to-graph.md` |
| `temporal-and-provenance.md` | `temporal-time-travel-queries.md`、`provenance-tracking-prov-o.md` |
| `exporting-and-sharing.md` | `json-ld-framing-nested-json.md` |
| `storing-knowledge.md` | `rdf-star-statements-about-statements.md` |

### 完成标准

- [ ] 每个具有匹配博客文章的功能页面都有"延伸阅读"或"深入了解"部分
- [ ] 链接使用相对路径

---

## DOC-013：将"你将学到什么"添加到教程段

**优先级**：高
**工作量**：S（< 1 天）
**负责领域**：入门
**阻塞**：无

### 现状

教程段有时间估计，但没有"你将学到什么"前言。

### 需要更改的内容

在每个段顶部添加一个简短（2-3 个要点）的"你将学到什么"框。

### 完成标准

- [ ] 4 个教程段中每个都有学习目标摘要
- [ ] 使用一致的格式（admonish 或引用块）

---

## DOC-014：标准化提示框系统

**优先级**：高
**工作量**：S（< 1 天）
**负责领域**：所有
**阻塞**：无

### 现状

混合了 `````admonish note````` 插件语法和 `> **Note**：` markdown 引用块。

### 需要更改的内容

在所有地方标准化使用 mdBook admonish 插件（`admonish note`、`admonish warning`、`admonish tip`、`admonish important`）。这提供一致的样式。

### 完成标准

- [ ] 没有剩余的 `> **Note**：` / `> **Warning**：` 原始引用块（替换为 admonish）
- [ ] 抽查 10+ 页面以确保一致性

---

## 中优先级 —— v1.0.0 可有可无

---

## DOC-020：创建"从 Neo4j 迁移"配方

**优先级**：中
**工作量**：M（1–3 天）
**负责领域**：Cookbook
**阻塞**：无

### 现状

`features/lpg-mapping.md` 记录了 Cypher → RDF 翻译模式。

### 需要更改的内容

创建 `cookbook/migrate-from-neo4j.md` —— 一个涵盖以下内容的分步配方：
- 从 Neo4j 导出（APOC、CSV、JSON）
- 属性图 → RDF 映射模式
- pg_ripple 导入
- 查询翻译（Cypher → SPARQL 备忘单）
- 验证

### 完成标准

- [ ] 配方页面存在，具有端到端的工作示例
- [ ] 添加到 SUMMARY.md 和 cookbook 索引
- [ ] 从 `features/lpg-mapping.md` 交叉链接

---

## DOC-021：创建"从头开始构建 RAG 管道"教程

**优先级**：中
**工作量**：M（1–3 天）
**负责领域**：Cookbook / AI
**阻塞**：无

### 现状

`user-guide/rag-pipeline.md` 记录了 API。`cookbook/grounded-chatbot.md` 显示了一个配方。两者都不是从零开始。

### 需要更改的内容

创建一个统一的端到端教程：Docker 设置 → 加载域数据 → 配置嵌入 → 运行 `rag_context()` → 调用 LLM → 评估答案质量。包括 LangChain/Python 示例。

### 完成标准

- [ ] 教程页面存在，包含从 Docker 到工作的聊天机器人的完整代码
- [ ] 包括 LangChain Python 集成
- [ ] 添加到 SUMMARY.md

---

## DOC-022：添加架构图（Mermaid）

**优先级**：中
**工作量**：M（1–3 天）
**负责领域**：功能
**阻塞**：无

### 现状

着陆页有 ASCII 图。功能页面中没有 Mermaid 图。

### 需要更改的内容

向以下内容添加 Mermaid 图：
- HTAP 存储架构（delta/main/合并流程）
- CDC 管道（写入 → delta → NOTIFY → 消费者）
- 联邦查询执行（本地 + 远程 SERVICE 规划）

### 完成标准

- [ ] 在功能深入页面中添加 3+ 个 Mermaid 图
- [ ] 图在 mdBook 中正确渲染

---

## DOC-023：添加"限制和配额"参考页面

**优先级**：中
**工作量**：S（< 1 天）
**负责领域**：参考
**阻塞**：无

### 现状

没有专门的页面。限制分散在 GUC 文档和运维页面中。

### 需要更改的内容

创建 `docs/src/reference/limits.md`：
- 经过测试的三元组数（100M+）
- 最大谓词数
- SPARQL 查询复杂度限制（代数深度、三元组模式）
- 联邦限制
- 语句 ID 序列容量
- 字典大小

### 完成标准

- [ ] 页面存在，包含具体的数字和引用
- [ ] 添加到 SUMMARY.md

---

## DOC-024：GUC 调优配置文件

**优先级**：中
**工作量**：S（< 1 天）
**负责领域**：运维
**阻塞**：DOC-002

### 现状

`operations/tuning.md` 和 `operations/performance.md` 存在。

### 需要更改的内容

添加"调优配置文件"部分，其中包含以下内容的推荐 GUC 设置：
- **OLTP**（许多小查询、低延迟）
- **OLAP**（少量大查询、批量推理）
- **小型实例**（< 1M 三元组，512MB RAM）
- **大型实例**（> 50M 三元组，16GB+ RAM）

### 完成标准

- [ ] 4 个命名的配置文件，包含 GUC 设置
- [ ] 每个配置文件都解释了权衡

---

## DOC-025：SHACL 速查卡

**优先级**：中
**工作量**：S（< 1 天）
**负责领域**：参考
**阻塞**：无

### 现状

`reference/shacl.md` 是全面的。没有速查卡。

### 需要更改的内容

添加一页"SHACL 备忘单"，其中包含所有约束组件、它们的语法和一行示例。

### 完成标准

- [ ] 列出所有受支持的 SHACL 约束及其语法的表
- [ ] 从功能页面和参考页面链接

---

## DOC-030：v1.0.0 公告博客文章

**优先级**：中
**工作量**：M（1–3 天）
**负责领域**：博客
**阻塞**：v1.0.0 发布

### 现状

没有公告文章。

### 需要更改的内容

编写 `blog/v1-what-is-new.md`：发布的内容、目标受众、关键数字、迁移指南、v1.0 之后的路线图。

### 完成标准

- [ ] 博客文章涵盖所有主要的 v1.0 功能
- [ ] 链接到相关文档页面
- [ ] 为 v0.99.x 上的用户提供迁移指南

---

## 低优先级 —— v1.0.0 之后

---

## DOC-040：博客新鲜度审计

**优先级**：低
**工作量**：M（1–3 天）
**负责领域**：博客
**阻塞**：无

### 现状

39 篇博客文章。有些可能引用了较旧的 API 模式。

### 需要更改的内容

审查每篇文章的 API 准确性。在模式已更改的位置添加版本徽章或更新说明。

### 完成标准

- [ ] 审查每篇博客文章
- [ ] 过时的示例使用版本说明进行注释

---

## DOC-041：OpenAPI 规范生成

**优先级**：低
**工作量**：M（1–3 天）
**负责领域**：参考
**阻塞**：DOC-003

### 现状

`/openapi.yaml` 端点存在于 HTTP 服务中。

### 需要更改的内容

验证生成的规范是否完整，或从路由代码生成一个综合规范。将其托管在文档站点上。

### 完成标准

- [ ] OpenAPI 规范涵盖所有 HTTP 端点
- [ ] 规范可从文档中访问

---

## DOC-042：扩展词汇表

**优先级**：低
**工作量**：S（< 1 天）
**负责领域**：参考
**阻塞**：无

### 现状

词汇表存在核心术语。

### 需要更改的内容

添加 pg_ripple 特定术语：VP 表、HTAP 拆分、合并工作线程、语句 ID、字典编码、罕见谓词合并。

### 完成标准

- [ ] 添加 10+ 个 pg_ripple 特定术语
- [ ] 每个都有 1-2 句定义

---

## DOC-043：扩展 FAQ

**优先级**：低
**工作量**：S（< 1 天）
**负责领域**：参考
**阻塞**：无

### 现状

FAQ 存在常见问题。

### 需要更改的内容

根据常见模式添加问题：
- "我可以将 pg_ripple 与 PostGIS 一起使用吗？"
- "如何备份和恢复？"
- "当 VP 表变得非常大时会发生什么？"
- "我可以在读取副本上运行 pg_ripple 吗？"

### 完成标准

- [ ] 5+ 个新的 FAQ 条目
- [ ] 每个都有直接的、可操作的答案

---

## 交付顺序

### 立即（本次会话）

1. **DOC-001**：修复 Hello World 有缺陷的查询
2. **DOC-011**：着陆页人物角色决策树
3. **DOC-013**：教程"你将学到什么"部分
4. **DOC-012**：关键功能页面上的博客交叉链接

### 下一轮迭代

5. **DOC-002**：完成 GUC 参考
6. **DOC-003**：完成 HTTP API 参考
7. **DOC-004**：审计 SQL 函数参考
8. **DOC-010**：生产就绪检查清单
9. **DOC-014**：标准化提示框系统

### 后续迭代

10. **DOC-020–025**：中优先级内容添加
11. **DOC-030**：v1.0.0 公告文章

### 持续进行

12. **DOC-040–043**：低优先级打磨