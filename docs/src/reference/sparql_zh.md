# SPARQL 参考

此页面是 pg_ripple 的 SPARQL 1.1 查询和更新引擎的参考。

## 概述

pg_ripple 将 SPARQL 1.1 查询语言和 SPARQL 1.1 更新实现为原生 PostgreSQL SQL 函数。所有 SPARQL 执行都在扩展内通过 `spargebra` 解析器、代数优化器（`sparopt`）和将 SPARQL 代数转换为通过 SPI 执行的 PostgreSQL SQL 的转换层来执行。结果通过字典解码以将 RDF 词项作为文本返回。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE 'sparql%';
```

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.sparql(query TEXT) → SETOF record` | 执行 SPARQL SELECT 查询 |
| `pg_ripple.sparql_update(update TEXT) → void` | 执行 SPARQL 1.1 更新（INSERT DATA、DELETE DATA、DELETE/INSERT WHERE、CLEAR、DROP、COPY、MOVE、ADD） |
| `pg_ripple.sparql_construct(query TEXT) → TEXT` | 执行 SPARQL CONSTRUCT，返回 Turtle |
| `pg_ripple.sparql_describe(iri TEXT) → TEXT` | 执行 SPARQL DESCRIBE，返回 Turtle |
| `pg_ripple.sparql_ask(query TEXT) → BOOLEAN` | 执行 SPARQL ASK 查询 |
| `pg_ripple.explain_sparql(query TEXT, analyze BOOLEAN) → TEXT` | 返回 SPARQL 查询的 JSON 解释计划 |
| `pg_ripple.sparql_cursor(query TEXT, page_size INT) → TEXT` | 为大型结果集打开服务器端游标 |
| `pg_ripple.sparql_cursor_next(cursor_id TEXT, page_size INT) → SETOF record` | 从游标获取下一页 |
| `pg_ripple.sparql_cursor_close(cursor_id TEXT) → void` | 关闭游标并释放资源 |
| `pg_ripple.sparql_cursor_turtle(query TEXT, page_size INT) → TEXT` | 打开返回 Turtle 页面的 CONSTRUCT 游标 |
| `pg_ripple.sparql_cursor_jsonld(query TEXT, page_size INT) → TEXT` | 打开返回 JSON-LD 页面的 CONSTRUCT 游标 |
| `pg_ripple.subscribe_sparql(id TEXT, query TEXT, graph_iri TEXT) → void` | 注册实时订阅 |
| `pg_ripple.unsubscribe_sparql(id TEXT) → void` | 删除实时订阅 |
| `pg_ripple.list_sparql_subscriptions() → SETOF record` | 列出活动的订阅 |

## SPARQL 1.1 功能覆盖范围

pg_ripple 支持完整的 SPARQL 1.1 规范：

- **SELECT**，带投影、DISTINCT、REDUCED、LIMIT、OFFSET、ORDER BY
- **CONSTRUCT**，带图模式和模板三元组
- **DESCRIBE**，返回 CBD（简洁有界描述）
- **ASK**，返回布尔值
- **图模式**：BGP、OPTIONAL、UNION、MINUS、GRAPH、SERVICE、FILTER、BIND、VALUES
- **属性路径**：`|`、`/`、`^`、`?`、`*`、`+`、`!`、`{n}`、`{n,}`、`{n,m}`
- **聚合函数**：COUNT、SUM、MIN、MAX、AVG、GROUP_CONCAT、SAMPLE
- **内置函数**：所有 ~50+ SPARQL 1.1 标量函数
- **子查询**：嵌套 SELECT 模式
- **SPARQL 更新**：所有 10 种更新形式

## RDF-star 支持

支持主语和对象位置的三元组引号模式 `<<s p o>>`。字典将 RDF-star 词项存储为已编码的三元组（已编码的引号三元组的主语、谓词和对象的哈希）。

## 性能说明

- 整数连接：所有 SPARQL 到 SQL 的转换在生成 SQL 之前将绑定词项编码为 `BIGINT`；在 VP 表查询内不会发生字符串比较。
- 过滤器下推：FILTER 常量在转换时编码。
- 自连接消除：同一主语上的星形模式被折叠到单次扫描计划中。
- 计划缓存（`_pg_ripple.plan_cache`）存储已编译的 SQL，以在重复查询中重用。

## 相关页面

- [SPARQL 查询 SQL 参考](../user-guide/sql-reference/sparql-query.md)
- [SPARQL 更新 SQL 参考](../user-guide/sql-reference/sparql-update.md)
- [SPARQL 一致性矩阵](sparql-compliance.md)
- [计划缓存](plan-cache.md)
- [查询优化](query-optimization.md)
- [功能状态分类](feature-status-taxonomy.md)

---

## SPARQL 扩展函数 IRI 命名空间（API-04，v0.91.0）

所有 pg_ripple SPARQL 扩展函数都在规范命名空间下定义：

```
http://pg-ripple.org/fn/
```

简写 `pg:` 前缀映射到所有通过 pg_ripple 执行的 SPARQL 查询中的此命名空间。该前缀是**自动声明**的 — 查询不需要显式声明 `PREFIX pg: <http://pg-ripple.org/fn/>`，尽管这样做是无害的，并且是针对多个 SPARQL 端点运行的查询的推荐样式。

### 可用扩展函数

| 短形式 | 完整 IRI | 起始版本 | 描述 |
|---|---|---|---|
| `pg:confidence(?s, ?p, ?o)` | `http://pg-ripple.org/fn/confidence` | v0.87.0 | 三元组跨模型的最高置信度分数 |
| `pg:pagerank(?node)` | `http://pg-ripple.org/fn/pagerank` | v0.88.0 | 节点的 PageRank 分数（默认主题） |
| `pg:pagerank(?node, ?topic)` | `http://pg-ripple.org/fn/pagerank` | v0.88.0 | 节点在命名主题中的 PageRank 分数 |
| `pg:similar(?a, ?b)` | `http://pg-ripple.org/fn/similar` | v0.27.0 | 嵌入向量之间的余弦相似度 |
| `pg:fuzzy_match(?a, ?b)` | `http://pg-ripple.org/fn/fuzzy_match` | v0.87.0 | 三元组相似度（需要 pg_trgm） |
| `pg:confPath(?pred, ?minConf)` | `http://pg-ripple.org/fn/confPath` | v0.87.0 | 带置信度阈值过滤的属性路径 |

### 联邦说明

希望远程调用 pg_ripple 扩展函数的联邦合作伙伴必须使用**完整 IRI 形式**，因为远程端点不会自动声明 `pg:` 前缀：

```sparql
FILTER(<http://pg-ripple.org/fn/confidence>(?s, ?p, ?o) > 0.8)
```