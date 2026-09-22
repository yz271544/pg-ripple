# 架构一览

两分钟的 pg_ripple 构建概述。完整的架构参考位于 [参考 → 架构](../reference/architecture.md)；此页面是您可以在评估期间交给架构师的摘要。

---

## pg_ripple 的位置

```
┌────────────────────────────────────────────────────────────────────┐
│                          您的应用程序                                │
│                                                                     │
│   psql / asyncpg / JDBC          HTTP / SPARQL 协议                  │
└──────────────┬───────────────────────────┬─────────────────────────┘
               │ SQL                       │ HTTP
               ▼                           ▼
┌────────────────────────┐    ┌────────────────────────┐
│   PostgreSQL 18        │    │  pg_ripple_http        │
│                        │◄───┤  （配套服务）            │
│  ┌──────────────────┐ │    │  Rust + Axum + deadpool│
│  │   pg_ripple       │ │    └────────┬───────────────┘
│  │   扩展            │ │             │
│  │  (Rust + pgrx)    │ │             │ pg_ripple.sparql(...)
│  └──────────────────┘ │             │
└────────────────────────┘             │
            │                          │
            ▼                          │
┌────────────────────────────────────────┘
│  存储层（PostgreSQL 表，全部整数编码）
│
│   _pg_ripple.dictionary        — IRI / blank / literal → BIGINT
│   _pg_ripple.vp_<predicate_id> — 每个谓词一个 VP 表
│   _pg_ripple.vp_rare           — 长尾谓词
│   _pg_ripple.embeddings        — pgvector 向量，HNSW 索引
│   _pg_ripple.kge_embeddings    — 图结构嵌入
│   _pg_ripple.audit_log         — 每个 SPARQL UPDATE
│   …
```

一切都是普通的 PostgreSQL。没有单独的图存储、没有单独的向量存储、没有单独的缓存。单个 `pg_dump` 就可以捕获所有内容。

---

## 五个子系统

| 子系统 | 拥有的内容 | 实现于 |
|---|---|---|
| **字典** | 将每个 IRI / 空白节点 / 字面量编码为稳定的 `BIGINT` | `src/dictionary/` |
| **存储** | VP 表、HTAP delta + main 拆分、合并后台工作线程 | `src/storage/` |
| **SPARQL 引擎** | SPARQL 文本 → 代数 → SQL → SPI 执行 → 解码 | `src/sparql/` |
| **Datalog 引擎** | 规则解析器、分层器、SQL 编译器、OWL RL/EL/QL 配置文件 | `src/datalog/` |
| **SHACL 验证器** | 形状 → DDL 约束 + 异步验证管道 | `src/shacl/` |

所有五个子系统位于同一 PostgreSQL 进程和同一 SQL 事务中。这是使混合检索、原子记录链接和审计级出处成为可能的属性。

---

## 塑造一切的三个架构选择

### 1. **垂直分区 (VP) —— 每个谓词一个表**

每个唯一谓词获得自己的表。`vp_<id>(s, o, g, i, source)` —— 全部为整数。星形模式（同一主语、多个谓词）折叠到同一主语值上的一次自连接。没有 `triples(s, p, o, g)` 巨型表。

结果：SPARQL 星形模式查询与针对类似形状的关系 schema 手工编写的 SQL 查询的速度相匹配。通常更快，因为每个连接都是整数相等。

### 2. **字典编码 —— 到处都是 BIGINT**

每个三元组，甚至在解析器边界处，在触及存储之前都被转换为 `BIGINT` ID 元组。连接是整数相等。过滤器是整数比较。仅在查询输出时解码为文本。字典使用 XXH3-128 进行无冲突哈希，并使用 LRU 共享内存缓存处理热词项。

### 3. **HTAP 存储 —— delta + main + 合并工作线程**

大量摄取进入每个谓词的 **delta** 表（常规 B 树堆）。读取查询看到 `(main EXCEPT tombstones) UNION ALL delta`。后台工作线程异步将 delta 合并到 BRIN 索引的 main 中。写入者从不阻塞读取者；读取者永远不会看到部分加载。

---

## 您在图中*看不到*的内容

这些对用户来说是故意不可见的，但具有巨大的影响：

- **语句 ID 时间线** —— 每个三元组携带来自共享序列的全局唯一 SID。驱动 `point_in_time()` 查询。
- **谓词目录** —— `_pg_ripple.predicates(id, table_oid, triple_count)`。缓存在共享内存中；在扩展重新加载后保留。
- **计划缓存** —— 翻译后的 SPARQL → SQL 计划按查询文本哈希缓存，在 schema 更改时失效。
- **后台工作线程** —— 合并工作线程、嵌入工作线程（当 `auto_embed = on` 时）、CDC 发布者。

---

## 它如何扩展

| 轴 | 机制 |
|---|---|
| **一台机器上更多 CPU** | 并行合并工作线程、并行 Datalog 层评估、VP 表上的 PostgreSQL 并行扫描 |
| **一台机器上更多磁盘** | `vp_<id>_main` 上的 BRIN 索引、由 GUC 控制大小的字典缓存 |
| **许多小型并发查询** | PostgreSQL 连接池 + `pg_ripple_http` deadpool 池 |
| **读取副本** | `pg_ripple.read_replica_dsn` 自动路由读取查询 |
| **跨多台机器** | Citus 集成：VP 表成为分布式表；绑定主语的 SPARQL 模式被剪枝到一个分片 |

有关每个轴的深入覆盖，请参阅 [操作 → 扩展](../operations/scaling.md) 和 [操作 → Citus 集成](../operations/citus-integration.md)。

---

## 在哪里阅读更多

- [参考 → 架构](../reference/architecture.md) — 完整的、与代码链接的架构描述。
- [操作 → 架构概述](../operations/architecture.md) — 运营团队对同一架构的看法。
- [计划 → 实施计划](https://github.com/trickle-labs/pg-ripple/blob/main/plans/implementation_plan.md) — *最终目标架构*的权威描述。