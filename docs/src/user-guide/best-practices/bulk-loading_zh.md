# 批量加载最佳实践

## UNNEST 数组批量 INSERT 路径 (v0.113.0)

自 v0.113.0 起，`pg_ripple.bulk_load_use_copy` 默认值为 `on`。启用后，`load_ntriples()` 和 `load_turtle()` 通过 UNNEST 数组批量 INSERT 辅助函数（`copy_into_vp()`）写入字典编码的三元组，该函数将整个批次作为 `BIGINT[]` 数组发送，并通过单次参数化 SQL 调用插入。

**基准测试结果**（1000 万三元组 N-Triples 文件，PG 18，16 GB 内存）：

| 模式 | 吞吐量 | 备注 |
|---|---|---|
| 逐行 VALUES INSERT（v0.113.0 之前的默认） | ~180K triples/s | 大量小 SQL 字符串 |
| UNNEST 数组批量 INSERT（v0.113.0 默认） | ~1.1M triples/s | **~5–10× 更快** |

该路径与 R2RML 和 CDC 加载器共享，因此所有摄入路径都能受益。

若需恢复为旧的逐行 VALUES INSERT 行为以便调试：

```sql
SET pg_ripple.bulk_load_use_copy = off;
SELECT pg_ripple.load_ntriples($$ <data> $$);
SET pg_ripple.bulk_load_use_copy = on;  -- 恢复默认值
```

## 批量大小

`load_ntriples()` 和 `load_turtle()` 在单个批次中处理整个输入。对于超大的文件（数亿条三元组），请将输入拆分为每块 100–1000 万条三元组的块，并依次加载。这可以保持事务大小可控，并允许在各批次之间定期运行 `ANALYZE`。

```bash
# 将大型 NT 文件拆分为 100 万三元组的块
split -l 1000000 large.nt chunk_
for f in chunk_*; do
    psql -c "SELECT pg_ripple.load_ntriples_file('/data/$f');"
    psql -c "ANALYZE _pg_ripple.vp_rare;"
done
```

## VP 提升阈值

默认阈值为 1000 条三元组。对于由少量非常常见谓词（例如 `rdf:type`）主导的工作负载，可以考虑降低阈值以便更快触发提升：

```sql
SET pg_ripple.vp_promotion_threshold = 100;
```

提升后，专用的 VP 表会在 `(s, o)` 和 `(o, s)` 上建立 B-tree 索引，针对特定谓词的查询比共享的 `vp_rare` 表快得多。

## 大批量加载后运行 ANALYZE

PostgreSQL 查询规划器依赖表统计信息来选择连接策略。加载超过约 100K 三元组后，请运行：

```sql
-- 分析共享的 rare 表
ANALYZE _pg_ripple.vp_rare;

-- 分析任何新提升的 VP 表
-- （将 XXX 替换为 _pg_ripple.predicates 中显示的实际谓词 ID）
ANALYZE _pg_ripple.vp_XXX;
```

如果统计信息不是最新的，规划器可能会选择顺序扫描而非 VP 表上的索引扫描。

## 空白节点作用域

每次调用批量加载函数都是一个独立的空白节点作用域。如果加载两个文件，每个文件都包含 `_:b0`，它们将获得不同的字典 ID——这是 RDF 规范所要求的。

**这意味着**：如果空白节点在切分点处共享，请勿将使用空白节点的 N-Triples 文件拆分为多个 `load_ntriples_file()` 调用。要么在一次调用中加载完整文件，要么使用全局唯一的空白节点 ID（例如基于 UUID 的 `_:b_{uuid}`）。

## 超大数据集使用 COPY

对于数十亿三元组的加载，可以考虑两阶段方法：

1. 在 staging 脚本中使用 `pg_ripple.encode_term()` 将术语预编码为 `BIGINT` ID
2. 使用 PostgreSQL `COPY` 将数据直接流式写入目标 VP 表

这避免了 Rust 解析-插入路径中每行的字典查找开销。详情请参阅 `plans/implementation_plan.md` 中的批量加载实现说明。

## 批量加载期间的 HTAP delta 增长（v0.6.0）

使用 v0.6.0 的 HTAP 存储布局时，所有插入首先写入 delta 表。在大型批量加载期间，delta 表可能会变得非常大，然后合并 worker 才有机会将其压缩到主表。

**delta 失控增长的症状**：
- 对批量加载谓词的查询会扫描大型 delta 表（比预期慢）
- `SELECT pg_ripple.stats() -> 'unmerged_delta_rows'` 显示数百万行
- `pg_stat_user_tables` 显示 `*_delta` 表上的 `n_live_tup` 非常高

**应对策略**：

### 选项 A：调整合并积极度

在开始大型加载之前，降低合并阈值以使 worker 能够跟上：

```sql
ALTER SYSTEM SET pg_ripple.merge_threshold = 10000;
ALTER SYSTEM SET pg_ripple.latch_trigger_threshold = 5000;
ALTER SYSTEM SET pg_ripple.merge_interval_secs = 5;
SELECT pg_reload_conf();
```

加载完成后，恢复生产环境的值。

### 选项 B：定期手动压缩

对于离线批量加载（加载期间的查询新鲜度不重要），可以定期调用 `compact()`：

```bash
for f in chunk_*; do
    psql -c "SELECT pg_ripple.load_ntriples_file('/data/$f');"
    # 每加载 10 个块压缩一次
    if (( i % 10 == 0 )); then
        psql -c "SELECT pg_ripple.compact();"
    fi
    ((i++))
done
# 完成后最终压缩
psql -c "SELECT pg_ripple.compact();"
```

### 选项 C：为加载谓词禁用 HTAP（高级）

对于只会进行批量加载（而非流式加载）的谓词，您可以在加载后迁移回平面布局以保持其平面布局。这是一个高级用例——请联系维护者获取指导。

### 最终清理

批量加载和压缩周期之后，运行 `ANALYZE` 以更新规划器统计信息：

```sql
-- 分析 delta、main 和 vp_rare 表
ANALYZE _pg_ripple.vp_rare;

-- 分析 _pg_ripple schema 中的所有对象
DO $$
DECLARE t text;
BEGIN
  FOR t IN SELECT relname FROM pg_class c
           JOIN pg_namespace n ON n.oid = c.relnamespace
           WHERE n.nspname = '_pg_ripple'
             AND c.relkind = 'r'
  LOOP
    EXECUTE 'ANALYZE _pg_ripple.' || quote_ident(t);
  END LOOP;
END $$;
```

## 并行加载

多个并发的 `load_ntriples()` 调用是安全的——字典插入使用 `ON CONFLICT DO NOTHING … RETURNING`，这是 MVCC 安全的。但是，对 `vp_rare` 的重度并发写入可能会导致锁争用。为获得最佳吞吐量，请使用单个数据库连接进行加载。