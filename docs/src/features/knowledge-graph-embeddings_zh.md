# 知识图嵌入（KGE）

**知识图嵌入**是从图的**结构** — 其关系 — 学习到的实体的向量表示，而不是从描述实体的任何文本中学习。KGE 支持文本向量做得不好的三个任务：

1. **跨图实体对齐** — 基于共享邻居将图 A 中的 `Apple` 匹配到图 B 中的 `Apple Inc.`，而不是基于字符串。
2. **链接预测** — 评分*当前不在图中*的合理边（推荐、冷启动、schema 完成）。
3. **集群发现** — 找到结构相似实体的组，即使它们没有共同的标签。

pg_ripple 提供两个众所周知的 KGE 模型，**TransE** 和 **RotatE**，具有统一的 SQL 接口和用于快速最近邻搜索的 HNSW 索引。

> 自 v0.55.0 起可用（`pg_ripple.kge_enabled` GUC）。需要 `pgvector`。

---

## TransE 和 RotatE 的区别

| 模型 | 几何思想 | 适用于 | 成本 |
|---|---|---|---|
| **TransE** | 关系是向量空间中的 *平移*：`head + relation ≈ tail` | 层次结构、简单关系模式 | 便宜；在数百万个三元组上几分钟内训练 |
| **RotatE** | 关系是复杂向量空间中的 *旋转* | 对称、反对称、逆和组合模式 | 约 TransE 成本的 2 倍；在密集图上质量更好 |

如有疑问，从 TransE 开始。如果您的图有许多逆或对称关系（`spouse`、`siblingOf`、`coAuthor`），请切换到 RotatE。

---

## 快速入门

```sql
-- 1. 启用该功能。
SET pg_ripple.kge_enabled = on;

-- 2. 在整个存储上训练模型。
SELECT pg_ripple.kge_train(
    model        := 'TransE',
    dimensions   := 128,
    epochs       := 100,
    learning_rate:= 0.01,
    margin       := 1.0
);

-- 3. 检查训练好的向量。
SELECT entity_iri, vector
FROM _pg_ripple.kge_embeddings
LIMIT 5;

-- 4. 将向量用于实体对齐。
SELECT * FROM pg_ripple.find_alignments(
    source_graph := 'https://example.org/g1',
    target_graph := 'https://example.org/g2',
    threshold    := 0.9
);
```

---

## 选择超参数

| 参数 | 默认值 | 调优建议 |
|---|---|---|
| `dimensions` | `128` | 小图（< 100K 实体）使用 50，超大或密集图使用 200–400 |
| `epochs` | `100` | 直到验证损失达到平稳；每 25 个 epoch 检查一次 |
| `learning_rate` | `0.01` | 如果损失振荡则减半；如果损失减少太慢则加倍 |
| `margin` | `1.0` | 仅 TransE；正负三元组分数之间的边距 |
| `batch_size` | `1024` | 更大的批次给出更平滑的梯度，但使用更多内存 |

训练将其损失曲线写入 PostgreSQL 日志，以便您可以实时监控收敛。

---

## 人们常犯的三个错误

1. **过早嵌入。** KGE 需要一个连接的图。如果您在加载 `owl:sameAs` 和逆属性之前训练，模型会学习孤立的岛屿。**在**训练之前始终实现内置的 RDFS / OWL 推理（`pg_ripple.infer('rdfs')`、`pg_ripple.infer('owl-rl')`）。
2. **跨模型比较。** 使用 TransE 128-dim 训练的向量对使用 RotatE 256-dim 训练的模型毫无意义。`_pg_ripple.kge_embeddings` 表按行跟踪 `(model, dimensions)`；查询自动限定于一个模型。
3. **忘记重新训练。** 随着图的增长，KGE 质量会下降。每当实体计数增长约 25 % 时安排重新训练，或对于高速摄取每周进行一次。

---

## 类别：链接预测

```sql
-- 评估候选三元组的合理性。
SELECT pg_ripple.kge_score(
    head     := '<https://example.org/Alice>',
    relation := '<https://example.org/worksAt>',
    tail     := '<https://example.org/MIT>'
);
-- 返回一个实值分数；越高 = 越合理。

-- 查找 Alice 的前 10 个最合理的雇主。
SELECT tail, score
FROM pg_ripple.kge_predict_tails(
    head     := '<https://example.org/Alice>',
    relation := '<https://example.org/worksAt>',
    k        := 10
);
```

这是冷启动推荐和 schema 完成工作流的元条件。

---

## 类别：跨图对齐

`find_alignments()` 是一个薄的包装器，对 `source_graph` 中的每个实体执行 HNSW 余弦搜索以匹配 `target_graph` 中的每个实体，返回高于阈值的对。输出的格式与 `suggest_sameas()` 完全相同，因此它可以插入[记录链接](record-linkage.md)管道，无需修改。

```sql
SELECT s1, s2, similarity
FROM pg_ripple.find_alignments(
    source_graph := 'https://wikidata.example/',
    target_graph := 'https://internal-kb.example/',
    threshold    := 0.92
)
ORDER BY similarity DESC;
```

---

## 存储和索引

| 对象 | 目的 |
|---|---|
| `_pg_ripple.kge_embeddings(entity_id, model, dimensions, vector)` | 每个 (entity, model) 一行。向量类型是 pgvector。 |
| 在 `(model, vector vector_cosine_ops)` 上的 HNSW 索引 | 亚毫秒级 top-k 余弦查询 |
| `_pg_ripple.kge_models(name, dimensions, trained_at, loss)` | 每个训练运行一行，用于监控 |

具有 128-dim TransE 嵌入的 1M 实体图在 `_pg_ripple.kge_embeddings` 中占用约 512 MB。请相应地规划磁盘。

---

## 何时**不**使用 KGE

- 您的图很小（< 10K 实体）。TransE 会过拟合；文本向量更简单。
- 您的实体没有信息丰富的关系。KGE 没有东西可学。
- 您需要可解释的分数。KGE 是一个黑匣子；SHACL 约束和 `owl:sameAs` 是面向监管者的决策的正确答案。

---

## 另请参阅

- [记录链接](record-linkage.md) — 使用 `find_alignments()` 进行跨图实体解析。
- [向量和混合搜索](vector-and-hybrid-search.md) — KGE 的文本嵌入表亲。
- [推理和推理](reasoning-and-inference.md) — 在训练之前实现推理以密集化图腾。