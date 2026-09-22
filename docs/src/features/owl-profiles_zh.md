# OWL 2 Profiles — RL、EL、QL

OWL 2 在数据库内部进行评估时过于富有表现力 — 完整的 OWL 对某些查询是不可判定的。因此 W3C 标准化了三个**OWL 2 profiles**，它们在表达能力和可处理性之间进行权衡。pg_ripple 为所有三个都内置了规则集和查询重写器。

| Profile | 最适合 | 可处理性 | 使用以下方式加载 |
|---|---|---|---|
| **OWL 2 RL** | 通用企业推理，RDFS 加强版 | 多项式时间物化 | `load_rules_builtin('owl-rl')` |
| **OWL 2 EL** | 大型术语层次结构（医学本体、SNOMED）| 多项式时间分类 | `load_rules_builtin('owl-el')` |
| **OWL 2 QL** | 在本体上的读取密集型访问，查询重写 | 次多项式查询回答 | `load_rules_builtin('owl-ql')` |

单个 GUC 为新连接选择活动的 profile：

```sql
ALTER SYSTEM SET pg_ripple.owl_profile = 'rl';   -- 'rl' | 'el' | 'ql' | 'off'
SELECT pg_reload_conf();
```

设置 `owl_profile = 'off'` 会禁用本体重写 — 当您想在没有推理平滑的情况下检查原始三元组时很有用。

---

## OWL 2 RL — 工作马

OWL 2 RL 是大多数用户想要的 profile。它涵盖：

- 类层次结构：`rdfs:subClassOf`、`owl:equivalentClass`、`owl:disjointWith`
- 属性层次结构：`rdfs:subPropertyOf`、`owl:equivalentProperty`、`owl:propertyChainAxiom`
- 属性特征：`owl:TransitiveProperty`、`owl:SymmetricProperty`、`owl:InverseOf`、`owl:FunctionalProperty`、`owl:InverseFunctionalProperty`
- 类构造函数：`owl:unionOf`、`owl:intersectionOf`、`owl:hasValue`、`owl:someValuesFrom`（在受限位置）
- `owl:sameAs` 和 `owl:differentFrom`

使用以下方式运行：

```sql
SELECT pg_ripple.load_rules_builtin('owl-rl');
SELECT pg_ripple.infer('owl-rl');
```

OWL 2 RL 规则集有大约 80 条规则。在您的本体和数据上测量物化时间；仓库不包含当前的大规模结果。

当前 CI 验证对固定 W3C OWL 语料库的获取，但不执行它。请参阅[OWL 2 RL 源状态](../reference/owl2rl-results.md)。

---

## OWL 2 EL — 用于大型术语

EL 是为具有非常大型类层次结构和少量个体的本体设计的 — 医学术语（SNOMED、NCIt）、基因本体、产品分类法。它支持：

- 带有存在限制的类包含：`owl:someValuesFrom`
- 属性链：例如 `partOf ∘ partOf ⊑ partOf`
- 嵌套的 `owl:hasSelf`
- 自反属性

EL 分类（计算完整的子类层次结构）在本体大小上是多项式的。pg_ripple 的 EL 实现使用基于饱和的算法，并将闭包存储在 `_pg_ripple.el_classified` 中。

```sql
SELECT pg_ripple.load_rules_builtin('owl-el');
SELECT pg_ripple.infer('owl-el');

-- 所有包含 :BacterialPneumonia 的类。
SELECT * FROM pg_ripple.sparql('
    SELECT ?super WHERE { <https://example.org/BacterialPneumonia>
                          <http://www.w3.org/2000/01/rdf-schema#subClassOf>+ ?super }
');
```

---

## OWL 2 QL — 用于查询重写

QL 是在不需要完整表征化推理的情况下在本体上回答 SPARQL 查询的首选 profile。pg_ripple **在翻译时使用本体公理重写查询**，而不是展开数据。这保持了数据存储的小巧，并允许您在不重新物化的情况下更改本体。

QL 支持一组刻意小的构造 — `rdfs:subClassOf`、`rdfs:subPropertyOf`、`owl:inverseOf`、`owl:someValuesFrom`（仅在对象位置）、`owl:disjointWith`。权衡是所有内容都是快速的。

```sql
SELECT pg_ripple.load_rules_builtin('owl-ql');
SET pg_ripple.owl_profile = 'ql';

-- 这个 SELECT 在执行之前使用 subClassOf 公理进行重写。
SELECT * FROM pg_ripple.sparql('
    SELECT ?animal WHERE { ?animal a <https://example.org/Mammal> }
');
```

如果 `Dog rdfs:subClassOf Mammal`，重写器会自动将 `Mammal` 扩展为 `(Mammal | Dog | Cat | …)` — 而无需插入任何新三元组。

---

## SPARQL-DL — 直接 OWL 公理查询

当您想查询*T-Box*本身（本体，而不是数据）时，pg_ripple 公开两个 SPARQL-DL 帮助器：

```sql
-- :Mammal 的直接子类。
SELECT * FROM pg_ripple.sparql_dl_subclasses('<https://example.org/Mammal>');

-- :Dog 的所有超类。
SELECT * FROM pg_ripple.sparql_dl_superclasses('<https://example.org/Dog>');
```

这些将 OWL 词汇表 BGP（`owl:subClassOf`、`owl:equivalentClass`、`owl:disjointWith`）直接路由到 T-Box VP 表 — 无需单独的索引。

---

## 选择 profile

| 问题 | Profile |
|---|---|
| 我只想让 SPARQL 查询"看到" RDFS+ 推理 | **RL**（默认）|
| 我有一百万类的分类法，需要快速分类 | **EL** |
| 我有一个本体和一个小型数据存储，并希望跳过物化 | **QL** |
| 我不确定 | **RL** |

您还可以一次加载多个 profile 的规则集，并在不同的规则集名称下运行它们 — 它们是独立的。

---

## 另请参阅

- [推理和推理](reasoning-and-inference.md) — 为所有三个 profile 提供支持的 Datalog 引擎。
- [OWL 2 RL 一致性结果](../reference/owl2rl-results.md)
- [SPARQL 合规性矩阵](../reference/sparql-compliance.md)