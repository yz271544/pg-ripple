# 记录链接和实体解析

**状态**：自 v0.67.0 起可用（ER-01）  
**要求**：[`pgvector`](https://github.com/pgvector/pgvector) 扩展用于基于嵌入的候选生成（可选 — 基于规则的链接在没有它的情况下也有效）。可选的 `pg_ripple_http` 用于 REST 访问。  
**SQL**：`pg_ripple.find_entity_candidates()`、`pg_ripple.merge_entities()`、`pg_ripple.entity_resolution_pipeline()`  
**降级**：没有 `pgvector`，向量相似度检测的候选生成不可用；SHACL 硬否决规则和 `owl:sameAs` 规范化仍然有效。  

---

> 这个问题的其他名称：*实体解析*、*去重*、*主数据管理 (MDM)*、*身份解析*、*模糊匹配*。它们都意味着：**查找引用同一现实世界事物的记录，并安全地合并它们。**

记录链接是数据管理中最具决定性和最困难的问题之一。合并两个患者数据库的医院、整合客户记录的银行、统一在线和店内资料零售商 — 所有这些都必须回答同一个问题：*这两行是否描述了同一个人？*

pg_ripple 在 PostgreSQL 内部提供了一个完整的记录链接堆栈。它结合了四种传统上分别存在于不同工具中的技术：

1. **知识图嵌入**用于快速、模糊的候选生成。
2. **跨文本嵌入的向量相似性**用于语义匹配。
3. **SHACL 硬规则**否决未例外的合并（例如，"绝不合并两个血型不同的患者"）。
4. **`owl:sameAs` 规范化**使您的其余查询看到一个实体，而不是两个。

每一步都是一个 SQL 函数调用。每个决策都是可审计的。

---

## 管道

```
   源 A 三元组           源 B 三元组
        │                          │
        └────────┬─────────────────┘
                 │
                 ▼
   ┌─────────────────────────────┐
   │ 1. 生成候选对               │
   │    suggest_sameas()         │ ← KGE 或文本嵌入 + HNSW
   │    或 find_alignments()     │
   └──────────────┬──────────────┘
                  │ （s1、s2、similarity）行
                  ▼
   ┌─────────────────────────────┐
   │ 2. 应用硬规则               │
   │    SHACL 形状阻止违反约束的 │ ← 例如血型上的 sh:disjoint
   │    对                       │
   └──────────────┬──────────────┘
                  │ 过滤后的候选行
                  ▼
   ┌─────────────────────────────┐
   │ 3. 人工审核（可选）          │
   │    显示候选                  │ ← UI / 批准队列
   │    与相似性、来源和规则跟踪  │
   └──────────────┬──────────────┘
                  │ 已接受的对
                  ▼
   ┌─────────────────────────────┐
   │ 4. 应用 owl:sameAs          │
   │    apply_sameas_candidates  │ ← 插入两个方向
   └──────────────┬──────────────┘
                  │
                  ▼
   ┌─────────────────────────────┐
   │ 5. 在读取时规范化            │
   │    sameas_reasoning = on    │ ← SPARQL 和 Datalog 看到一个实体
   │    audit_log 捕获所有        │
   │    更新                      │
   └─────────────────────────────┘
```

---

## 工作示例：跨两个系统链接客户记录

假设您经营两个零售品牌，每个都有自己的客户数据库。您希望在不丢失品牌特定历史的情况下获得统一的视图。

### 第 1 步：将两个源加载到命名图中

使用命名图保留出处 — 您始终可以分辨出哪条记录来自哪里。

```sql
-- 源 A
SELECT pg_ripple.load_turtle_into_graph(
    'https://example.org/source-a',
    $TTL$
@prefix ex:   <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:a/c1  foaf:name  "Robert Smith"  ; foaf:mbox <mailto:rob.smith@example.com> .
ex:a/c2  foaf:name  "Jane Doe"      ; foaf:mbox <mailto:jdoe@example.com> .
$TTL$);

-- 源 B
SELECT pg_ripple.load_turtle_into_graph(
    'https://example.org/source-b',
    $TTL$
@prefix ex:   <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:b/c1  foaf:name  "Bob Smith"     ; foaf:mbox <mailto:rob.smith@example.com> .
ex:b/c2  foaf:name  "Jane Q. Doe"   ; foaf:mbox <mailto:jane.doe@example.com> .
$TTL$);
```

### 第 2 步：生成候选对

您有两个候选生成选项。选择适合您数据的任何一个。

**选项 A — 文本嵌入。** 当您具有丰富的文本描述（姓名、地址、产品标题）时最佳。

```sql
-- 使用标签嵌入每个客户。
SELECT pg_ripple.embed_entities();

-- 首先在宽松阈值下建议 sameAs 对。
SELECT s1, s2, similarity
FROM pg_ripple.suggest_sameas(threshold := 0.85)
ORDER BY similarity DESC
LIMIT 50;
```

**选项 B — 知识图嵌入 (KGE)。** 当实体具有丰富的关系结构（客户的购买、地址、设备）时最佳。

```sql
SET pg_ripple.kge_enabled = on;
SELECT pg_ripple.kge_train(model := 'TransE', epochs := 100);

SELECT * FROM pg_ripple.find_alignments(
    source_graph := 'https://example.org/source-a',
    target_graph := 'https://example.org/source-b',
    threshold    := 0.85
);
```

您可以运行两者并合并结果 — 文本和 KGE 嵌入捕获不同的错误。

### 第 3 步：使用 SHACL 阻止未例外的合并

定义硬规则，指出 *"这两条记录不能是同一实体"*。经典的例子是矛盾的不可变属性 — 不同的出生日期、不同的血型、不同的国家 ID。

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix ex:   <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:CustomerSafetyShape a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    # 如果两个人共享 owl:sameAs，他们的出生日期必须一致。
    sh:property [
        sh:path ex:birthDate ;
        sh:disjoint ex:contradictoryBirthDate ;
    ] .
$TTL$);
```

当 `pg_ripple.shacl_mode = 'sync'` 时，违反此形状的 `apply_sameas_candidates()` 调用在提交之前被拒绝。

### 第 4 步：应用幸存的候选对

```sql
-- 以严格的阈值应用以进行自动合并。
SELECT pg_ripple.apply_sameas_candidates(min_similarity := 0.95);

-- 剩余的 0.85–0.95 区间进入审核队列。
CREATE TABLE review_queue AS
SELECT s1, s2, similarity
FROM pg_ripple.suggest_sameas(0.85)
WHERE similarity < 0.95;
```

然后人工审核员可以批准或拒绝 `review_queue` 中的对；已批准的对通过 `pg_ripple.insert_triple(s1, '<http://www.w3.org/2002/07/owl#sameAs>', s2)` 及其逆应用。

### 第 5 步：查询统一的图

使用 `pg_ripple.sameas_reasoning = on`（默认），每个 SPARQL 和 Datalog 查询都会透明地看到合并的实体。没有单独的"黄金记录"表需要维护。

```sql
-- 返回 Robert/Bob Smith 从两个源数据库的购买作为一个客户。
SELECT * FROM pg_ripple.sparql($$
    SELECT ?purchase WHERE {
        <https://example.org/a/c1>
            <https://example.org/purchased> ?purchase .
    }
$$);
```

---

## 调整阈值：精确率与召回率

| 阈值带 | 精确率 | 召回率 | 推荐操作 |
|---|---|---|---|
| ≥ 0.98 | 非常高 | 低 | 自动应用，无需审核 |
| 0.95 – 0.98 | 高 | 中 | 自动应用，每周抽样审核 |
| 0.90 – 0.95 | 中 | 高 | 审核队列以供人工批准 |
| 0.85 – 0.90 | 低 | 非常高 | 仅显示用于探索性分析 |
| < 0.85 | 非常低 | 接近完整 | 避免 — 太嘈杂 |

正确的区间取决于每种错误类型的成本。在医疗保健中，错误的合并可能危及患者 — 将自动合并阈值推至 0.99 并将其余部分路由给临床医生。在广告技术中，错过的合并意味着不太个性化的广告 — 0.92 的自动合并是可以接受的。

---

## 可审计性

每个记录链接操作都会留下跟踪。三个 GUC 启用审计链：

| GUC | 捕获的内容 |
|---|---|
| `pg_ripple.audit_log_enabled = on` | 所有 SPARQL UPDATE（角色、txid、查询文本）— 请参阅 [Audit Log](../reference/audit-log.md) |
| `pg_ripple.prov_enabled = on` | 每个批量加载的 `prov:Activity` 三元组 — 请参阅 [Temporal & Provenance](temporal-and-provenance.md) |
| (RDF-star 引用三元组) | 每个事实的置信度、来源、时间戳 — 请参阅 [Storing Knowledge](storing-knowledge.md) |

这三个共同提供了纯 ML 管道无法提供的、监管机构可辩护的跟踪。

---

## 为什么这在其他地方很难做好

大多数记录链接系统强制在*神经*（高召回率，不透明）和*符号*（可审计，低召回率）之间进行选择。pg_ripple 允许您组合两者。扩展的基本原理在 `plans/neuro-symbolic-record-linkage.md` 中 — 该文档是本页所有内容的战略背景。

---

## 函数参考

| 函数 | 目的 | 在以下位置记录 |
|---|---|---|
| `embed_entities()` | 计算所有标记实体的文本嵌入 | [向量和混合搜索](vector-and-hybrid-search.md) |
| `kge_train()` | 训练 TransE/RotatE 实体嵌入 | [知识图嵌入](knowledge-graph-embeddings.md) |
| `suggest_sameas(threshold)` | 从文本嵌入返回候选对 | 本页 |
| `find_alignments(src, tgt, threshold)` | 从 KGE 返回候选对 | [知识图嵌入](knowledge-graph-embeddings.md) |
| `apply_sameas_candidates(min_similarity)` | 为已接受的对插入 `owl:sameAs` | 本页 |
| `load_shacl()` | 加载否决未例外合并的硬规则形状 | [验证数据质量](validating-data-quality.md) |
| `point_in_time(ts)` | 重放过去某个时间戳的记录链接决策 | [时间和出处](temporal-and-provenance.md) |

## 进一步阅读

- [博客：神经符号实体解析](https://github.com/trickle-labs/pg-ripple/blob/main/blog/neuro-symbolic-entity-resolution.md) — 将嵌入与符号规则相结合以进行去重
- [博客：owl:sameAs 实体解析](https://github.com/trickle-labs/pg-ripple/blob/main/blog/owl-sameas-entity-resolution.md) — 等价规范化如何工作