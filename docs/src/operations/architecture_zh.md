# 架构概述

pg_ripple 是一个 PostgreSQL 18 扩展，实现了具有原生 SPARQL 查询执行功能的高性能 RDF 三元组存储。本页描述内部架构：数据的存储方式、查询的执行方式以及子系统之间的交互。

---

## 系统架构图

```
┌──────────────────────────────────────────────────────────────────────┐
│                        客户端应用程序                                  │
│   psql / JDBC / SPARQL 协议（pg_ripple_http）/ REST / ODBC           │
└────────────────────────────┬─────────────────────────────────────────┘
                             │
                             ▼
┌──────────────────────────────────────────────────────────────────────┐
│                     PostgreSQL 18 后端                                │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                    pg_ripple 扩展                              │  │
│  │                                                                │  │
│  │  ┌──────────────┐  ┌───────────────┐  ┌────────────────────┐  │  │
│  │  │  SPARQL       │  │  Datalog       │  │  SHACL             │  │  │
│  │  │  引擎         │  │  推理器         │  │  验证器             │  │  │
│  │  │              │  │               │  │                    │  │  │
│  │  │  解析 →     │  │  分层 →       │  │  shapes → DDL       │  │  │
│  │  │  优化 →    │  │  编译 →      │  │  约束 +           │  │  │
│  │  │  SQL 生成 → │  │  半朴素        │  │  异步管道          │  │  │
│  │  │  SPI 执行 → │  │  不动点        │  │                    │  │  │
│  │  │  解码       │  │               │  │                    │  │  │
│  │  └──────┬───────┘  └───────┬───────┘  └────────┬───────────┘  │  │
│  │         │                  │                    │              │  │
│  │         ▼                  ▼                    ▼              │  │
│  │  ┌─────────────────────────────────────────────────────────┐  │  │
│  │  │              字典编码器（XXH3-128）                       │  │  │
│  │  │    IRI / 空节点 / 字面量  ──→  i64 标识符                  │  │  │
│  │  │         共享内存 LRU 缓存（64 分片）                       │  │  │
│  │  └────────────────────────┬────────────────────────────────┘  │  │
│  │                           │                                    │  │
│  │                           ▼                                    │  │
│  │  ┌─────────────────────────────────────────────────────────┐  │  │
│  │  │                VP 存储引擎（HTAP）                        │  │  │
│  │  │                                                         │  │  │
│  │  │   vp_{id}_delta  ──┐                                    │  │  │
│  │  │   （写入收件箱）     │                                    │  │  │
│  │  │                    ├──→  vp_{id}（读取视图）              │  │  │
│  │  │   vp_{id}_main   ──┤    = (main − 墓碑)                  │  │  │
│  │  │   （BRIN 存档）     │      UNION ALL delta                │  │  │
│  │  │                    │                                    │  │  │
│  │  │   vp_{id}_tombstones                                    │  │  │
│  │  │   （挂起的删除）     │                                    │  │  │
│  │  │                    │                                    │  │  │
│  │  │   vp_rare ─────────┘  （合并的稀有谓词）                   │  │  │
│  │  └─────────────────────────────────────────────────────────┘  │  │
│  │                           │                                    │  │
│  │  ┌────────────────────────┴────────────────────────────────┐  │  │
│  │  │           后台合并工作线程（BGW）                         │  │  │
│  │  │   delta + main − 墓碑 ──→ 新 main（BRIN）               │  │  │
│  │  │   轮询间隔：merge_interval_secs（默认 60s）             │  │  │
│  │  │   阈值：merge_threshold（默认 10,000 行）               │  │  │
│  │  └─────────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  _pg_ripple schema：dictionary、predicates、vp_*、statements  │
│  │  pg_ripple schema：公共 SQL 函数（sparql、insert 等）         │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 字典编码器

字典编码器是 pg_ripple 存储模型的基础。每个 RDF 项 — IRI、空节点、纯字面量、类型化字面量或带语言标签的字面量 — 在存储之前都被映射为紧凑的 `i64` 标识符。

### 编码工作原理

1. 输入项按类型分类：IRI（0）、空节点（1）、字面量（2）、类型化字面量（3）或带语言标签的字面量（4）。
2. 类型判别符作为两个小端字节混合到哈希输入中，因此相同的字符串编码为 IRI 和编码为空节点时总是产生不同的字典行。
3. 在 `(kind_le_bytes || term_utf8)` 上计算 XXH3-128 哈希。
4. 完整的 16 字节哈希存储在 `_pg_ripple.dictionary` 表中，并使用 `ON CONFLICT (hash) DO NOTHING` 上插。密排的 `i64` 连接键是由 `IDENTITY` 生成的列 — 是顺序的，独立于哈希。
5. 结果 `i64` 用于所有 VP 表列中。

```admonish info title="为什么使用整数编码？"
VP 表从不包含原始字符串。所有 JOIN、比较和索引查找都在 `i64` 值上运行。这消除了一些复杂性开销，将存储减少了 5–20 倍，并使 B-tree 索引扫描无论 IRI 长度如何都保持一致快速。
```

### 共享内存缓存

字典缓存位于 PostgreSQL 共享内存中（在 postmaster 启动时分配），并组织为 64 分片集关联结构。每个后端通过原子操作读取和写入共享缓存 — 无任何后台复制。

关键参数：
- **`pg_ripple.dictionary_cache_size`** — 缓存条目数（默认：65,536）。需要重新启动。
- **`pg_ripple.cache_budget`** — 内存预算上限（MB）（默认：64）。批量加载在 90% 利用率时进行限流。

缓存命中率由 `pg_ripple.stats()` 报告，在生产环境中应保持在 95% 以上。

---

## VP（垂直分区）表

pg_ripple 使用垂直分区：每个唯一谓词一个物理表。这是 SW-Store 和面向列的三元组存储等研究系统使用的存储模型。

### 表布局

每个至少 `p` 个三元组（默认：1,000）的谓词都会获得一个专用的 VP 表：

```sql
-- 每个 VP 表中的列
s      BIGINT  NOT NULL   -- 主语字典 ID
o      BIGINT  NOT NULL   -- 对象字典 ID
g      BIGINT  NOT NULL DEFAULT 0  -- 图 ID（0 = 默认图）
i      BIGINT  NOT NULL DEFAULT nextval('statement_id_seq')  -- 唯一 SID
source SMALLINT NOT NULL DEFAULT 0  -- 0 = 显式，1 = 推断
```

`(s, o)` 和 `(o, s)` 上的双 B-tree 索引支持主语到对象和对象到主语的访问模式。

### 稀有谓词合并

三元组少于提升阈值的谓词存储在共享的 `_pg_ripple.vp_rare` 表中，其中包含额外的 `p BIGINT` 列。这避免了不常用谓词的 schema 膨胀。当稀有谓词的计数超过阈值时，它会自动提升到专用的 VP 表。

### 谓词目录

`_pg_ripple.predicates` 表将每个谓词 ID 映射到其 VP 表 OID 和当前三元组计数：

```sql
SELECT id, table_oid, triple_count
FROM _pg_ripple.predicates
ORDER BY triple_count DESC;
```

```admonish warning title="无动态 SQL 字符串拼接"
SPARQL 到 SQL 转换器从不将表名拼接为 SQL 字符串。它在 `_pg_ripple.predicates` 中查找 OID，并使用带有安全格式引号的参数化查询。这从设计上防止了 SQL 注入。
```

---

## HTAP 存储架构

从 v0.6.0 开始，pg_ripple 使用 HTAP（混合事务/分析处理）存储架构，为每个 VP 表分离读写路径。

### 三表分割

对于每个谓词，存储层维护：

| 表 | 用途 | 索引类型 |
|---|---|---|
| `vp_{id}_delta` | 写入收件箱 — 所有 INSERT 都在这里落地 | B-tree on `(s, o)` |
| `vp_{id}_main` | 读优化存档 | BRIN（块范围）|
| `vp_{id}_tombstones` | 主表中的挂起删除 | B-tree on `(s, o, g)` |

读取视图 `vp_{id}` 组合它们：

```sql
(main EXCEPT tombstones) UNION ALL delta
```

### 后台合并工作线程

合并工作线程是一个在轮询循环中运行的 PostgreSQL 后台工作线程（`BGWorker`）：

1. **轮询** — 每隔 `merge_interval_secs`（默认：60）唤醒一次，或被写入路径锁存器触发时唤醒。
2. **扫描** — 检查每个 HTAP 谓词的 delta 行数与 `merge_threshold`（默认：10,000）的对比。
3. **合并** — 对于符合条件的谓词：从 `(old_main − tombstones) UNION ALL delta` 创建新的主表，通过 `ALTER TABLE ... RENAME` 原子地交换，在 `merge_retention_seconds` 后删除旧的主表。
4. **维护** — 重建主语/对象模式表，提升越过阈值的稀有谓词，在新的主表上运行 `ANALYZE`（当 `auto_analyze` 开启时），清除过期的联邦缓存条目。

```admonish tip title="写入路径"
写入者永远不会阻塞合并。所有 INSERT 直接进入 delta 表（堆 + B-tree）。合并工作线程异步运行，并使用 PostgreSQL 的 MVCC 进行隔离。
```

---

## SPARQL 查询执行流程

当客户端调用 `pg_ripple.sparql('SELECT ...')` 时，查询经历五个阶段：

### 1. 解析

SPARQL 文本由 `spargebra` crate 解析为代数表示形式。这处理完整的 SPARQL 1.1 语法：SELECT、CONSTRUCT、DESCRIBE、ASK、属性路径、子查询、聚合、联邦（SERVICE）和 SPARQL-star。

### 2. 优化

`sparopt` 优化器重写代数树：
- **BGP 重排序** — 当 `bgp_reorder` 开启时，三元组模式按估计的选择性排序（最小的 VP 表优先）。
- **过滤器下推** — FILTER 常量在翻译时编码为 `i64`，并下推到生成 SQL 的 WHERE 子句中。
- **自连接消除** — 星形模式（同一主语，多个谓词）被折叠为多路连接，而不是冗余的子查询。
- **SHACL 提示** — 如果声明了 `sh:maxCount 1`，则省略 `DISTINCT`；如果声明了 `sh:minCount 1`，则将 `LEFT JOIN` 升级为 `INNER JOIN`。

### 3. 生成 SQL

优化的代数被编译为 PostgreSQL SQL：
- 每个三元组模式成为相应 VP 表（或带有谓词过滤器的 `vp_rare`）的扫描。
- 模式之间的 JOIN 成为带有 `i64` 等值谓词的 SQL `JOIN` 子句。
- 属性路径使用 PostgreSQL 18 的基于哈希的循环检测编译为 `WITH RECURSIVE ... CYCLE`。
- `SERVICE` 子句被编译为对远程 SPARQL 端点的 HTTP 调用。
- 聚合、`ORDER BY`、`LIMIT` 和 `OFFSET` 直接转换为它们的 SQL 等价物。

### 4. SPI 执行

生成的 SQL 通过 PostgreSQL 的服务器编程接口（SPI）执行。结果是 `i64` 字典 ID 的数组。

计划缓存（`plan_cache_size`，默认：256）存储最近使用的 SPARQL 查询的已编译 SQL，以避免重复的解析/优化/生成周期。

### 5. 解码

`i64` 结果列使用字典解码回人类可读的 RDF 项（IRI、字面量、空节点）。共享内存缓存加速此步骤 — 缓存命中避免每个值的字典表查找。

```admonish note title="到处都是整数连接"
SPARQL 引擎在生成 SQL *之前*将所有绑定常量编码为 i64，并在执行*之后*解码结果。VP 表查询永远不包含字符串比较 — 这是一个硬性架构不变性。
```

---

## Schema 组织

pg_ripple 使用两个 PostgreSQL schema：

| Schema | 内容 | 可见性 |
|---|---|---|
| `pg_ripple` | 公共 SQL 函数（`sparql()`、`insert_triple()`、`stats()` 等）| 面向用户 |
| `_pg_ripple` | 字典表、谓词目录、VP 表、语句映射、内部状态 | 内部 |

```admonish warning title="不要直接修改 _pg_ripple"
内部 schema 由扩展管理。直接修改 `_pg_ripple` 表可能会破坏字典或违反 VP 表不变性。
```

---

## 子系统摘要

| 子系统 | 源目录 | 用途 |
|---|---|---|
| 字典 | `src/dictionary/` | 使用 XXH3-128 进行项 ↔ i64 编码 |
| 存储 | `src/storage/` | VP 表、HTAP 分区、稀有谓词合并 |
| SPARQL | `src/sparql/` | 解析 → 优化 → SQL 生成 → SPI → 解码 |
| Datalog | `src/datalog/` | 规则解析、分层、半朴素不动点、magic sets |
| SHACL | `src/shacl/` | Shape 验证、并行约束、异步管道 |
| 导出 | `src/export/` | Turtle、N-Triples、JSON-LD 序列化 |
| 工作线程 | `src/worker.rs` | 后台合并工作线程、嵌入队列、SHACL 异步 |
| 统计 | `src/stats/` | 监控、缓存指标、健康检查 |
| 联邦 | `src/sparql/federation` | 远程 SERVICE 调用执行、连接池、缓存 |
| HTTP | `pg_ripple_http/` | SPARQL 协议端点（独立伴随服务）|