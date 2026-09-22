# IVM（增量视图维护）— 架构和边界

> **版本**：v0.91.0（IVM-01）

pg_ripple 实现两种**独立**的增量视图维护（IVM）机制：**CWB-IVM**（CONSTRUCT 回写 IVM）和 **PageRank-IVM**（K 跳脏边传播）。此页面解释了每种机制及其操作边界。

---

## IVM 边界：CWB 与 PageRank

这两种机制在不同的表上运行并由不同的事件触发。它们**不交互**：CWB 重新计算不会自动触发 PageRank 重新计算，PageRank IVM 更新也不会将派生的三元组写回 VP 表。

| 属性 | CWB-IVM | PageRank-IVM |
|---|---|---|
| **维护内容** | 从 SPARQL CONSTRUCT 规则派生的三元组 | 近似 PageRank 分数 |
| **算法** | Delete-Rederive（Z-set 增量） | 从脏边的 K 跳局部推送 |
| **源模块** | `src/construct_rules/delta.rs` | `src/pagerank/ivm.rs` |
| **队列表** | `_pg_ripple.cwb_queue` | `_pg_ripple.pagerank_dirty_edges` |
| **触发器** | 通过 `cwb_queue` 的 VP 表 delta INSERT/DELETE | 通过 `pagerank_dirty_edges` 的 VP 表 delta INSERT/DELETE |
| **输出** | 写入 VP 表的新派生三元组 | 更新 `_pg_ripple.pagerank_scores` 中的分数 |
| **完全重新计算函数** | `pg_ripple.run_full_recompute(rule_name)` | `pg_ripple.pagerank_run(...)` |

### CWB-IVM（CONSTRUCT 回写）

CWB-IVM 维护从 SPARQL CONSTRUCT 规则派生的派生三元组。当 VP 表 delta 被修改（插入或删除新三元组）时，使用 Delete-Rederive 算法重新评估受影响的规则：

1. 计算哪些以前派生的三元组不再可派生（Z-set 负数）。
2. 计算现在可派生哪些新三元组（Z-set 正数）。
3. 应用增量：撤回负数，断言正数。

`_pg_ripple.cwb_queue` 表保存挂起的增量事件。该队列由 `run_full_recompute()` 或增量维护后台工作线程排空。

**源**：`src/construct_rules/delta.rs`

### PageRank-IVM（K 跳脏边队列）

PageRank-IVM 使用从最近更改的边的有界 K 跳传播来维护近似 PageRank 分数。当 VP 表 delta 被修改时，受影响的边被添加到 `_pg_ripple.pagerank_dirty_edges`。IVM 工作线程通过以下方式处理队列：

1. 识别脏节点（入边已更改的节点）。
2. 重新计算这些节点及其 K 跳邻域的分数。
3. 更新受影响节点的 `_pg_ripple.pagerank_scores`。

如果边谓词受到影响，则在大规模 CWB 重新计算后需要完全 `pagerank_run()` — K 跳传播无法有效捕获大规模图重组。

**源**：`src/pagerank/ivm.rs`

---

## 监控

### CWB 队列深度

```sql
SELECT COUNT(*) FROM _pg_ripple.cwb_queue;
```

### PageRank 队列深度

```sql
SELECT * FROM pg_ripple.pagerank_queue_stats();
-- 返回：queued_edges、max_delta、oldest_enqueue、estimated_drain_seconds
```

这两个指标都通过 Prometheus 公开（请参阅[可观测性参考](observability.md)）：

- `pg_ripple_pagerank_queue_depth{topic=""}` — 等待 PageRank 刷新的脏边
- `pg_ripple_pagerank_queue_max_delta{topic=""}` — 最大挂起分数 delta
- `pg_ripple_pagerank_queue_oldest_enqueue_seconds{topic=""}` — 最旧脏条目的年龄

---

## GUC 参数

| GUC | 默认值 | 描述 |
|---|---|---|
| `pg_ripple.pagerank_incremental` | `off` | 启用 K 跳 IVM（脏边队列） |
| `pg_ripple.pagerank_queue_warn_threshold` | 10000 | 当队列深度超过此值时记录 WARNING |
| `pg_ripple.pagerank_ivm_k_hop` | 3 | 从脏边传播的跳数 |

---

## 跨模块依赖调度（IVM-03，v0.92.0）

写入 `_pg_ripple.confidence`（概率 Datalog 引擎）的规则**不包括**在 CWB（CONSTRUCT 回写）拓扑排序中。CWB 调度器仅处理 SPARQL CONSTRUCT 规则；Datalog 推导（包括置信度传播）通过 `pg_ripple.infer()` 或后台推导工作线程触发。

跨模块依赖边 — 例如，从 `_pg_ripple.confidence` 读取的 CONSTRUCT 规则或依赖于 CWB 维护谓词的 Datalog 规则 — 将需要显式注册调用。此 API 保留供将来使用：

```sql
-- 未来 API（尚未公开）：注册跨模块依赖
-- SELECT pg_ripple.register_ivm_dependency(
--     source_module => 'construct_rules',
--     source_rule   => 'my_pipeline',
--     target_module => 'datalog',
--     target_rule   => 'confidence_propagation'
-- );
```

在此 API 可用之前，需要 CONSTRUCT → 置信度或 Datalog → CONSTRUCT 依赖关系的应用程序必须手动排序调用：

```sql
-- 手动排序：首先运行 CONSTRUCT 管道，然后运行置信度推导
SELECT pg_ripple.run_construct_pipeline('my_pipeline');
SELECT pg_ripple.infer('confidence');
```

---

## 相关页面

- [PageRank](../features/pagerank.md)
- [CONSTRUCT 回写规则](construct-rules.md)
- [可观测性参考](observability.md)