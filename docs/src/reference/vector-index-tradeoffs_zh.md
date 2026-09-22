# 向量索引权衡

pg_ripple 通过 `pg_ripple.hybrid_search()` 函数支持混合 SPARQL + 语义搜索，该函数使用 pgvector 的 ANN（近似最近邻）索引。两种索引类型可用 — **HNSW** 和 **IVFFlat** — 在三种精度级别：**single**（32 位浮点）、**half**（16 位浮点）和 **binary**（1 位）。

作负载选择正确的组合。

## 基准测试设置

| 参数 | 值 |
|-----------|-------|
| 数据库大小 | 100,000 个嵌入 |
| 维度 | 128（text-embedding-3-small 使用 1536） |
| 查询 | 1,000 个随机查询向量 |
| k（邻居） | 10 |
| PostgreSQL | 18 |
| pgvector | 0.7.4 |
| 硬件 | 8 核 CPU，32 GB RAM |

自行运行基准测试：

```bash
psql -U postgres -f benchmarks/vector_index_compare.sql
```

## 参考结果

### HNSW（m=16，ef_construction=64）

| 精度 | 构建时间 | 召回率 p50 | 召回率 p95 | 延迟 p50 | 延迟 p95 |
|-----------|-----------|-----------|-----------|------------|------------|
| single (32-bit) | ~45 s | 99.2% | 98.1% | 0.4 ms | 0.8 ms |
| half (16-bit) | ~30 s | 98.7% | 97.5% | 0.3 ms | 0.6 ms |
| binary (1-bit) | ~8 s | 91.3% | 88.0% | 0.08 ms | 0.15 ms |

### IVFFlat（lists=100）

| 精度 | 构建时间 | 召回率 p50 | 召回率 p95 | 延迟 p50 | 延迟 p95 |
|-----------|-----------|-----------|-----------|------------|------------|
| single (32-bit) | ~5 s | 96.4% | 94.2% | 0.9 ms | 2.1 ms |
| half (16-bit) | ~4 s | 95.8% | 93.7% | 0.7 ms | 1.8 ms |
| binary (1-bit) | ~1.5 s | 85.2% | 81.0% | 0.2 ms | 0.5 ms |

## 建议

| 场景 | 推荐 |
|----------|-------------|
| 高精度语义搜索 (RAG) | HNSW，single 精度 |
| 延迟敏感的实时搜索 | HNSW，half 精度 |
| 非常大的数据集（> 10 M 嵌入），内存受限 | IVFFlat，half 精度 |
| 精确重排序前的粗略预过滤 | HNSW 或 IVFFlat，binary |
| 快速原型/开发 | IVFFlat，single 精度（构建快速） |

## 配置索引类型

通过 GUC 控制索引类型和精度：

```sql
SET pg_ripple.embedding_index_type = 'hnsw';   -- 或 'ivfflat'
SET pg_ripple.embedding_precision = 'single';  -- 或 'half' 或 'binary'
```

更改后重建索引：

```sql
SELECT pg_ripple.rebuild_embedding_index();
```

## 内存占用

| 精度 | 每 1 M 1536 维向量的内存 |
|-----------|--------------------------------|
| single (32-bit) | ~6 GB |
| half (16-bit) | ~3 GB |
| binary (1-bit) | ~190 MB |

对于具有数百万嵌入的生产部署，**half 精度**提供了最佳的召回率与内存权衡。

## 相关

- [嵌入函数](../reference/embedding-functions.md) — 完整 API 参考
- [GUC 参考](../reference/guc-reference.md) — `embedding_index_type`、`embedding_precision`
- [混合向量搜索示例](https://github.com/trickle-labs/pg-ripple/blob/main/examples/hybrid_vector_search.sql)