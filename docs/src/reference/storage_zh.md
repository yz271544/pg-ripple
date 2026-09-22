# 存储参考

此页面是 pg_ripple HTAP 存储层的参考。

## 概述

pg_ripple 使用垂直分区（VP）存储：每个唯一谓词一个表，主语和对象存储为 `BIGINT` 字典 ID。HTAP（混合事务/分析处理）设计将写入拆分到 delta 表（堆 + B 树），并从 main 表（BRIN 索引）读取。后台合并工作线程定期将 main + delta（减去墓碑）组合成一个新的 main 表。

## 状态

```sql
SELECT feature_name, status FROM pg_ripple.feature_status()
WHERE feature_name LIKE '%htap%' OR feature_name LIKE '%storage%';
```

## 表布局

每个 VP 谓词在 `_pg_ripple` 中获得三个物理表：

| 表 | 描述 |
|---|---|
| `vp_{id}_delta` | 最近的写入（堆、`(s,o)` 和 `(o,s)` 上的 B 树索引） |
| `vp_{id}_main` | 历史数据（BRIN 索引、读取优化） |
| `vp_{id}_tombstones` | 已从 `main` 删除的行（合并后清除） |

查询路径为：`(main EXCEPT tombstones) UNION ALL delta`。

罕见谓词（低于 `pg_ripple.vp_promotion_threshold`，默认值 1,000 个三元组）存储在 `_pg_ripple.vp_rare (p, s, o, g, i, source)` 中。

## 字典编码

每个 IRI、空白节点和字面量在存储之前通过 XXH3-128 哈希映射到 `BIGINT` (i64)。`_pg_ripple.dictionary` 表存储完整的往返映射。共享内存 LRU 缓存（大小由 `pg_ripple.dictionary_cache_size` 控制）避免了重复的数据库查找。

## VP 表列

| 列 | 类型 | 描述 |
|---|---|---|
| `s` | `BIGINT` | 主语字典 ID |
| `o` | `BIGINT` | 对象字典 ID |
| `g` | `BIGINT` | 命名图字典 ID（0 = 默认图） |
| `i` | `BIGINT` | 来自共享序列的语句 ID (SID) |
| `source` | `SMALLINT` | 0 = 显式，1 = 推导 |

## 合并工作线程

后台合并工作线程持续运行，并：
1. 检测 delta 超过 `pg_ripple.merge_threshold` 的 VP 表。
2. 获取谓词 ID 上的咨询锁。
3. 构建新的 `vp_{id}_main` 表（main EXCEPT tombstones UNION ALL delta）。
4. 使用 DDL 锁以原子方式重命名表（交换）。
5. 截断 delta 和 tombstones。

## SQL 函数

| 函数 | 描述 |
|---|---|
| `pg_ripple.vacuum_triples(graph_iri TEXT) → BIGINT` | 对图中的三元组去重 |
| `pg_ripple.reindex_vp(predicate_iri TEXT) → void` | 在 VP 表上重建索引 |
| `pg_ripple.force_merge(predicate_iri TEXT) → void` | 触发谓词的立即合并 |
| `pg_ripple.promote_predicate(predicate_iri TEXT) → void` | 从 vp_rare 提升到专用表 |
| `pg_ripple.recover_interrupted_promotions() → BIGINT` | 恢复卡在 'promoting' 状态的 VP 表 |

## 相关页面

- [架构内部](architecture.md)
- [可扩展性](scalability.md)
- [功能状态分类](feature-status-taxonomy.md)