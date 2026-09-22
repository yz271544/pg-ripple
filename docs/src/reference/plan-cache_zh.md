# 计划缓存

pg_ripple 维护一个后端本地的 SPARQL 计划缓存，以避免为重复查询执行冗余的 SPARQL → SQL 转换。本文档描述缓存契约、键构造和隔离属性。

## 缓存键构造

缓存键仅从 SPARQL 查询文本派生：

1. 查询文本由 `spargebra` 解析为规范代数树。
2. 代数树使用 `spargebra` 的 `Display` 实现进行打印以生成规范字符串。
3. 附加两个 GUC 值：`pg_ripple.max_path_depth` 和 `pg_ripple.bgp_reorder`。

生成的规范字符串用作哈希映射键。

### 缓存键中**不包含**的内容

| 省略项 | 原理 |
|---|---|
| 当前角色 / 会话用户 | PostgreSQL 在执行器级别（而非计划级别）强制执行行级安全性。缓存的 SQL 字符串在任何角色下执行都是安全的；RLS 策略在计划运行后过滤行。 |
| 命名图绑定（`GRAPH <iri>`） | 命名图在转换时编码为整数 ID。图 IRI 嵌入在 SQL `WHERE g = <id>` 子句中，因此查询文本已包含图标识。 |
| RLS 相关的会话变量 | 影响 RLS 策略的会话级 GUC（例如 `app.current_tenant`）由 PostgreSQL 执行器评估，而非由 pg_ripple 的查询计划器评估。 |

### 为何这是安全的

PostgreSQL 的 RLS 机制在执行时（解析查询计划之后）应用行过滤器。在任何角色下执行时，省略角色作为其键一部分的计划会产生正确（RLS-filtered）的结果。该计划对于角色特定的工作负载可能不是*最佳*的（角色专用计划可以使用更紧密的索引扫描），但它不能返回调用者不应看到的行。

## SERVICE 子句排除

包含 `SERVICE` 子句（联邦 SPARQL）的查询**永远不会被缓存**。远程端点的可用性和结果形状在调用之间可能会有所不同；缓存此类计划会产生过时或不正确的结果。

## 缓存容量和驱逐

计划缓存是一个 `LruCache`，其固定容量由 `pg_ripple.plan_cache_capacity` 设置（默认值：256 个计划）。当缓存已满时，最近最少使用的计划将被驱逐。

## 手动失效

```sql
-- 驱逐所有缓存的计划并重置命中/未命中计数器。
SELECT pg_ripple.plan_cache_reset();

-- 检查缓存统计信息。
SELECT pg_ripple.plan_cache_stats();
```

`plan_cache_stats()` 返回一个带有键的 JSONB 对象：
- `hits` — 自上次重置以来的缓存命中数。
- `misses` — 自上次重置以来的缓存未命中（执行转换）数。
- `size` — 当前缓存的计划数。
- `capacity` — 缓存可容纳的最大计划数。

## 角色隔离回归测试

pg_regress 文件 `tests/pg_regress/sql/plan_cache_rls.sql` 验证具有不同 RLS 授权的两个角色在运行相同的 SPARQL 查询文本时仅看到各自的数据：

- 角色 `ripple_alice` 可访问 `<https://example.org/graph_a>`。
- 角色 `ripple_bob` 仅可访问 `<https://example.org/graph_b>`。
- 相同的 SPARQL 查询以每个角色身份执行。
- `ripple_alice` 接收来自图 A 的三元组；`ripple_bob` 接收来自图 B 的三元组。

此测试确认角色之间的计划缓存共享不会绕过 PostgreSQL 的 RLS 强制执行。