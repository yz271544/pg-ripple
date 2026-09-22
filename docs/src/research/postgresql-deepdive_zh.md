# PostgreSQL 作为高性能三元组存储

> *此页面镜像 [`plans/postgresql-triplestore-deep-dive.md`](https://github.com/trickle-labs/pg-ripple/blob/main/plans/postgresql-triplestore-deep-dive.md) —— pg_ripple 设计阶段编写的架构蓝图。它解释了*为什么*做出每个主要的设计决策。*

---

## 引言

在关系数据库之上构建世界级的三元组存储需要弥合一个根本性的差距：图模式（节点和有向边）与关系表（行和列）。核心挑战是弥合这种*结构阻抗失配*，而不会引入不可接受的查询延迟。

从历史上看，RDF 三元组存储和关系数据库占据了相互竞争的生态系统。本地三元组存储在任意图遍历和 schema 灵活性方面表现出色；关系数据库在 OLTP、严格并发控制、ACID 保证和大规模并行分析聚合方面占主导地位。PostgreSQL 18 提供了正确的基础，因为它提供了*两者*：成熟的基于成本的优化器、并行查询执行、强大的 MVCC，以及一个功能强大的扩展 API 来添加原生 SPARQL 支持。

此处记录的设计决策是评估几种替代方法并选择具有最佳查询性能、写入吞吐量和运维简单性组合的结果。

---

## RDF 数据的关系存储布局

已经在关系数据库中存储三元组的三种主要布局已被研究：

### 三元组表

最简单的方法：一张表具有三列 `(subject, predicate, object)`。每个涉及 N 个三元组模式的 SPARQL 查询都需要在此单个表上进行 N 路自连接。在企业规模（数十亿三元组）下，即使跨所有列排列（SPO、POS、OSP、PSO）使用全面的复合 B 树索引，复杂查询也会耗尽 `work_mem` 并溢出到磁盘 —— 响应时间呈指数级下降。

**结论**：不适合高性能工作负载。

### 属性表

按本体类型将主语分组为宽的、每个属性一列的表。减少了统一实体类型的自连接，但 RDF 数据本质上是稀疏且异构的。宽表饱和于 NULL 值，浪费磁盘空间，并降低顺序扫描性能。Schema 演化需要阻塞 `ALTER TABLE` 操作，从而抵消了 RDF 的 schema 灵活性优势。

**结论**：对于开放世界、schema 灵活的图谱不可行。

### 垂直分区 (VP) —— pg_ripple 的选择

每个唯一谓词一个单独的两列表 `(subject, object)`。这是 Abadi 等人（VLDB 2007）开创的方法，pg_ripple 从第一天起就使用了它。

**为什么 VP 获胜：**

| 属性 | 三元组表 | 属性表 | **VP** |
|---|---|---|---|
| 自连接 | N 路自连接 | 低 | **每谓词，有针对性** |
| 稀疏性 | 无 | 大量 NULL | **无** |
| Schema 更改 | 无 | 阻塞 DDL | **每谓词动态** |
| 绑定谓词的 I/O | 全扫描 | 整行 | **单个谓词表** |
| 存储开销 | 谓词列 33% | 宽行 | **密集，无 NULL** |

谓词列在表名中*是隐式的* —— 在任何压缩之前消除了每个三元组 33% 的存储。历史基准（Barton 库目录数据集）显示查询解析时间比三元组表提高了数量级，从几分钟下降到几秒。

每个 VP 表在 `(s, o)` 和 `(o, s)` 上都有双重 B 树索引，可以实现高效的合并连接，无论遍历方向如何。

### 扩展 VP (ExtVP) —— 未来方向

ExtVP 通过预先计算频繁连接的谓词之间的半连接来构建在标准 VP 之上。当查询分析显示谓词 A 在主语列上经常与谓词 B 连接时，实体化视图保存 B 中也存在主语的 A 的子集。SPARQL→SQL 转换器重写查询以针对这些实体化视图，绕过数十亿个 CPU 周期。

这在 pg_ripple 中作为 1.0 之后的工作负载驱动优化实现（请参阅 v0.11.0）。

---

## 字典编码

以 50–200 字节的原始 IRI 字符串本地存储在 VP 表中将使整数连接不可行。pg_ripple 在写入任何 VP 表之前，将每个 IRI、空白节点和字面量字典编码为 `BIGINT`。

### 设计

- **哈希函数**：XXH3-128 作用于 `kind_le_bytes ‖ term_utf8` —— 种类字节（IRI/blank-node/literal）被混合到哈希中，因此具有不同词项类型的相同字符串获得不同的 ID。不截断为 64 位：完整的 16 字节哈希存储为 `BYTEA` 冲突检测键；PostgreSQL `GENERATED ALWAYS AS IDENTITY` 序列生成密集的、顺序的 `i64` 连接键。
- **为什么不截断为 64 位？** 生日问题：在 64 位空间中，碰撞预期在 ~40 亿个词项处。使用单独存储的完整 128 位哈希，冲突检测是表查找，而不是哈希比较。
- **内联编码**（v0.5.1+）：常见类型化字面量（`xsd:integer`、`xsd:boolean`、`xsd:dateTime`、`xsd:date`）使用位 63 设置为类型标记进行内联编码。对这些类型的 FILTER 比较需要零字典往返。

### 缓存（v0.1.0–v0.5.1 与 v0.6.0+）

- **v0.1.0–v0.5.1**：后端本地的 `lru::LruCache<u128, i64>` —— 简单，无 `shared_preload_libraries` 依赖。
- **v0.6.0+**：通过 pgrx `PgSharedMem` 在共享内存中的分片 `HashMap<u128, i64>`，分为 64 个分片，每个分片具有每分片轻量级锁。消除并发编码队列中的全局锁争用。由 `pg_ripple.dictionary_cache_size` GUC 控制大小。

### 查询解码

所有输出 ID 都从结果集中收集，并在单个 `WHERE id = ANY(...)` 查询中解码 —— 从不按行解码。这种"批量解码"模式对于大型结果集至关重要。

---

## HTAP 双分区架构（v0.6.0）

现代生产系统需要同时进行大量读取和写入。如果不特别小心，写入会阻塞读取，反之亦然。

### Delta / Main 拆分

所有 INSERT 和 DELETE 都针对 `_delta` 分区（标准堆 + B 树）。`_main` 分区是只读的、BRIN 索引的、按主语物理排序的。两者之间是用于跨分区删除的墓碑表。

**查询路径**：`(main EXCEPT tombstones) UNION ALL delta`

当墓碑表为空时（插入密集型工作负载的合并之间很常见），这简化为 main 和 delta 的 `UNION ALL`。

### 后台合并工作线程

pgrx `BackgroundWorker` 在 delta 超过 `pg_ripple.merge_threshold` 行时将 delta 合并到 main：

1. 创建 `vp_{id}_main_new`（空堆）
2. `INSERT … SELECT … ORDER BY s FROM (main − tombstones) UNION ALL delta` —— 物理排序，使 BRIN 最有效
3. `ALTER TABLE … RENAME` 原子地替换旧 main
4. `TRUNCATE vp_{id}_delta, vp_{id}_tombstones`
5. 对合并表进行 `ANALYZE`

在表创建时的 `ORDER BY s` 至关重要：BRIN 需要物理排序的数据才能使摘要范围准确。以随机顺序插入到现有 main 表会使 BRIN 几乎无用。

### Bloom 过滤器

每个 VP 表在共享内存中都有一个每分片 bloom 过滤器。针对已知仅在 main 中的数据的查询可以完全跳过 delta 扫描。

### `vp_rare` 豁免

三元组数少于 `pg_ripple.vp_promotion_threshold` 的谓词存储在单个扁平 `vp_rare` 表中，而不是专用的 VP 表 + delta/main 拆分。罕见谓词按定义写入很少，delt/main 开销将超过收益。标准 PostgreSQL 行级锁定安全地处理并发访问。

### 为什么不在写入时分区？

PostgreSQL 的声明性表分区是为稳定的分区键（日期、范围）设计的。RDF 写入在谓词空间上实际上是随机的 —— 在此处使用声明性分区将需要自定义路由逻辑，而不会比显式 delta/main 设计带来任何好处。

---

## SPARQL 到 SQL 的转换

从 SPARQL 代数到 SQL 的天真转换会产生冗长的嵌套子查询，使 PostgreSQL 计划器感到困惑。pg_ripple 实现了几个结构重写：

### 自连接消除

星形模式（同一主语、N 个谓词）折叠为跨 N 个 VP 表的单个主语扫描，通过主语 ID 相等连接。消除通过 SPI 的冗余往返。

### Optional 到内部降级

OPTIONAL → SQL 中的 LEFT JOIN。当优化器可以证明所需属性始终存在时（例如，通过具有 `sh:minCount 1` 的已加载 SHACL 形状），它降级为 INNER JOIN。仅在语义保留查询域时保守地应用。

### 过滤器下推

绑定 IRI 上的 SPARQL FILTER 子句在生成 SQL *之前*解析为整数 ID。这确保了 B 树索引使用。类型化的数字/日期字面量使用内联编码的 i64 范围（v0.5.1+）以发出 `BETWEEN $lo AND $hi` 范围扫描，无需解码步骤。

### 属性路径编译

SPARQL `+`、`*`、`?` 路径编译为 `WITH RECURSIVE` CTE，使用 PG18 的 `CYCLE` 子句进行基于哈希的循环检测：

```sql
WITH RECURSIVE path(s, o, depth) AS (
    SELECT s, o, 1 FROM _pg_ripple.vp_{id} WHERE s = $1
  UNION ALL
    SELECT p.s, vp.o, p.depth + 1
    FROM path p
    JOIN _pg_ripple.vp_{id} vp ON p.o = vp.s
    WHERE p.depth < pg_ripple.max_path_depth
)
CYCLE o SET is_cycle USING cycle_path
SELECT DISTINCT s, o FROM path WHERE NOT is_cycle;
```

PG18 的 `CYCLE` 子句使用基于哈希的循环检测（$O(1)$ 成员检查 vs. 旧基于数组的方法的 $O(n)$ 数组扫描）。

---

## SHACL 验证

SHACL 是用于在 RDF 图上定义数据质量规则的 W3C 标准。pg_ripple 以*规范优先*语义实现 SHACL 验证：加载的形状被解析为保留 W3C SHACL 语义的形状 IR。当 PostgreSQL 约束、触发器或流表对于特定形状模式被证明语义保留时，它们可以用作*内部加速器* —— 它们永远不是约束行为的规范性定义。

### 同步与异步验证

- **`pg_ripple.shacl_mode = 'sync'`**（v0.7.0）：验证在 INSERT 上内联运行；坏三元组立即被拒绝。适用于低容量事务写入。
- **`pg_ripple.shacl_mode = 'async'`**：轻量级触发器将插入的三元组 ID 排队到 `_pg_ripple.validation_queue`；后台工作线程根据加载的形状进行验证。无效三元组随 JSONB 违规报告一起移动到 `_pg_ripple.dead_letter_queue`。

### 通过 SHACL 进行查询优化

SPARQL→SQL 转换器在计划时读取加载的 SHACL 形状：
- 当查询限制在与已验证形状相同的焦点节点总体时，`sh:maxCount 1` 可以启用基数敏感优化。
- 当语义安全时，`sh:minCount 1` 可以将 OPTIONAL → INNER JOIN 降级。

---

## 性能目标

| 指标 | 目标 | 策略 |
|---|---|---|
| 批量插入 | >100K 三元组/秒 | 批量 COPY、延迟索引、HTAP delta |
| 事务插入 | >10K 三元组/秒 | Delta 分区、异步 SHACL |
| 简单 BGP 查询 | <5 ms（10M 三元组） | 整数连接、VP 表上的 B 树 |
| 星形查询（5 个模式） | <20 ms（10M 三元组） | 自连接消除、PG 并行哈希连接 |
| 属性路径（深度 10） | <100 ms（10M 三元组） | 递归 CTE + `CYCLE` 子句 |
| 字典编码（缓存命中） | <1 μs | 共享内存中的分片 LRU |
| 字典编码（未命中） | <50 μs | 哈希上的 B 树索引 |
| 批量解码（1,000 个 ID） | <1 ms | 单个 `WHERE id = ANY(...)` 查询 |

校准参考：QLever（C++、Apache-2.0）在 DBLP（3.9 亿三元组）上以 1.7M 三元组/秒的速度加载，并在 0.7 秒内回答平均基准查询。QLever 的扁平预排序置换文件使每个 SPARQL 连接都是零随机 I/O 的合并连接。pg_ripple 的 B 树/堆设计在批量顺序扫描上支付约 5 倍的开销，以换取事务并发写入、MVCC 和完整的 PostgreSQL 生态系统。

---

## 为什么是 PostgreSQL 18？

PostgreSQL 18 带来了几项显著提高 pg_ripple 性能的功能：

- **异步 I/O**（Linux 上 `io_method = io_uring`）：降低 BRIN 索引的主分区的顺序扫描延迟
- **`WITH RECURSIVE` 中的 `CYCLE` 子句**：属性路径查询中的基于哈希的循环检测
- **改进的并行查询**：星形模式 BGP 的更好的并行哈希连接计划
- **B 树上的 Skip scan**：在仅绑定 `o` 时（未绑定主语查询），在复合 `(s, o)` 索引上启用高效查找

---

## 进一步阅读

- [plans/implementation_plan.md](https://github.com/trickle-labs/pg-ripple/blob/main/plans/implementation_plan.md) —— 详细的 API 签名、算法和 crate 选择
- [plans/postgresql-triplestore-deep-dive.md](https://github.com/trickle-labs/pg-ripple/blob/main/plans/postgresql-triplestore-deep-dive.md) —— 扩展的研究调研与学术参考
- [Abadi 等人 2007](https://dsf.berkeley.edu/cs286/papers/rdf-vldb2007.pdf) —— 原始 VP 论文
- [qEndpoint](https://www.semantic-web-journal.net/content/qendpoint-novel-triple-store-architecture-large-rdf-graphs) —— delta/main 拆分的 HTAP 灵感
- [Berlin SPARQL Benchmark (BSBM)](http://wifo5-03.informatik.uni-mannheim.de/bizer/berlinsparqlbenchmark/) —— pg_ripple 基准测试中使用的查询组合