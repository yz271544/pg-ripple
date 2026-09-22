# 知识图谱嵌入（KGE）

**知识图谱嵌入**是从图的**结构**（实体的关系）而不是从描述实体的任何文本中学习的实体向量表示。KGE 支持文本嵌入做不好的三个工作：

1. **跨图实体对齐** — 基于共享邻居（而不是字符串）将图 A 中的 `Apple` 匹配到图 B 中的 `Apple Inc.`。
2. **链接预测** — 对图中今天*不*存在的合理边进行评分（推荐、冷启动、模式补全）。
3. **集群发现** — 即使它们没有共同标签，也要找到结构相似的实体组。

pg_ripple 配备两个著名的 KGE 模型 **TransE** 和 **RotatE**，具有统一的 SQL 接口和用于快速最近邻搜索的 HNSW 索引。

> 自 v0.55.0 起可用（`pg_ripple.kge_enabled` GUC）。需要 `pgvector`。

---

## TransE 和 RotatE 的区别

| 模型 | 几何思想 | 最适合 | 成本 |
|---|---|---|---|
| **TransE** | 关系是向量空间中的*平移*：`head + relation ≈ tail` | 层次结构、简单关系模式 | 便宜；在数百万三元组上几分钟内训练 |
| **RotatE** | 关系是复向量空间中的*旋转* | 对称、反对称、逆和组合模式 | 约为 TransE 成本的 2 倍；在密集图上质量更好 |

如有疑问，请从 TransE 开始。如果您的图具有大量逆或对称关系（`spouse`、`siblingOf`、`coAuthor`），请切换到 RotatE。

---

## 快速入门

```sql
-- 1. 启用功能。
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

-- 4. 使用向量进行实体对齐。
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
| `dimensions` | `128` | 小图（< 100 K 实体）为 50，超大或密集图为 200–400 |
| `epochs` | `100` | 直到验证损失稳定；每 25 个 epoch 检查一次 |
| `learning_rate` | `0.01` | 如果损失振荡则减半；如果损失下降太慢则加倍 |
| `margin` | `1.0` | 仅限 TransE；正负三元组分数之间的边界 |
| `batch_size` | `1024` | 较大的批次提供更平滑的梯度但使用更多内存 |

训练将其损失曲线写入 PostgreSQL 日志，以便您可以实时监视收敛情况。

---

## 人们常犯的三个错误

1. **过早嵌入。** KGE 需要连接的图。如果在加载 `owl:sameAs` 和逆属性之前训练，模型会学到孤立的孤岛。始终在训练*之前*物化内置 RDFS / OWL 推理（`pg_ripple.infer('rdfs')`、`pg_ripple.infer('owl-rl')`）。
2. **跨模型比较。** 使用 TransE 128 维训练的向量对使用 RotatE 256 维训练的模型毫无意义。`_pg_ripple.kge_embeddings` 表按行跟踪 `(model, dimensions)`；查询自动限定于一个模型。
3. **忘记重新训练。** 随着图的增长，KGE 质量会下降。每当实体数量增长约 25% 时安排重新训练，或对于高速摄取则每周一次。

---

## 用例：链接预测

```sql
-- 对候选三元组的合理性进行评分。
SELECT pg_ripple.kge_score(
    head     := '<https://example.org/Alice>',
    relation := '<https://example.org/worksAt>',
    tail     := '<https://example.org/MIT>'
);
-- 返回一个实值分数；越高 = 越合理。

-- 查找 Alice 的 10 个最合理的雇主。
SELECT tail, score
FROM pg_ripple.kge_predict_tails(
    head     := '<https://example.org/Alice>',
    relation := '<https://example.org/worksAt>',
    k        := 10
);
```

这是冷启动推荐和模式补全工作流的基础。

---

## 用例：跨图对齐

`find_alignments()` 是一个轻量包装器，对 `source_graph` 中的每个实体与 `target_graph` 中的每个实体执行 HNSW 余弦搜索，返回高于阈值的对。输出的形状与 `suggest_sameas()` 完全相同，因此无需修改即可插入[记录链接](record-linkage.md)管道。

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
| `_pg_ripple.kge_embeddings(entity_id, model, dimensions, vector)` | 每个（实体，模型）一行。向量类型是 pgvector。 |
| 在 `(model, vector vector_cosine_ops)` 上的 HNSW 索引 | 亚毫秒级 top-k 余弦查询 |
| `_pg_ripple.kge_models(name, dimensions, trained_at, loss)` | 每次训练运行一行，用于监视 |

具有 128 维 TransE 嵌入的 1 M 实体图在 `_pg_ripple.kge_embeddings` 中占用约 512 MB。请相应地规划磁盘。

---

## 何时**不**使用 KGE

- 您的图较小（< 10 K 实体）。TransE 会过拟合；文本嵌入更简单。
- 您的实体没有信息性关系。KGE 没有可学习的内容。
- 您需要可解释的分数。KGE 是一个黑盒；SHACL 约束和 `owl:sameAs` 是面向监管者的决策的正确答案。

---

## 另请参阅

- [记录链接](record-linkage.md) — 使用 `find_alignments()` 进行跨图实体解析。
- [向量与混合搜索](vector-and-hybrid-search.md) — 文本嵌入的表亲 KGE。
- [推理与演绎](reasoning-and-inference.md) — 在训练*之前*物化推理以使图密集化。