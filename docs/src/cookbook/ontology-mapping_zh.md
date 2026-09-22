# Cookbook：本体映射和对齐

**目标。** 您从外部数据集（Wikidata、schema.org、行业词汇表）导入数据，并希望它与您的内部本体对齐 —— 相同的概念，不同的 IRI。手动映射很繁琐；KGE 驱动的候选生成加上 SHACL 门加上 owl:sameAs 规范化使这项工作变得可处理。

**为什么选择 pg_ripple。** 组合与记录链接相同的构建块，但应用于*类*和*属性*而不是*个体*。

**核心结果的时间。** ~25 分钟。

---

## 挑战

您的内部本体将人员称为 `intkb:Employee`。Wikidata 将他们称为 `wd:Q5`（人类）。Schema.org 将他们称为 `schema:Person`。这些都不错误 —— 它们描述了重叠但不完全相同的概念。您希望 SPARQL 查询能够*透明地*将它们视为同一类进行检索，但您也希望*保留*特定来源的公理。

---

## 第 1 步 —— 加载所有三个词汇表

```sql
-- 内部本体。
SELECT pg_ripple.load_turtle_into_graph('https://example.org/ontologies/intkb', $TTL$
@prefix intkb: <https://intkb.example/> .
@prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .

intkb:Employee  rdfs:label "Employee" ;
                rdfs:comment "An employee of the organisation" .
intkb:Department rdfs:label "Department" .
intkb:reports   rdfs:label "reports to" .
$TTL$);

-- Schema.org 子集。
SELECT pg_ripple.load_turtle_file_into_graph(
    'https://example.org/ontologies/schemaorg', '/data/schemaorg-people.ttl');

-- Wikidata 子集。
SELECT pg_ripple.load_turtle_file_into_graph(
    'https://example.org/ontologies/wikidata',  '/data/wikidata-people.ttl');
```

## 第 2 步 —— 在所有三个上训练 KGE

```sql
SET pg_ripple.kge_enabled = on;
SELECT pg_ripple.kge_train(model := 'RotatE', dimensions := 200, epochs := 200);
```

首选 RotatE 进行本体对齐，因为它捕获 `owl:inverseOf` 公理中常见的逆和对称模式。

## 第 3 步 —— 生成候选对齐

```sql
SELECT * FROM pg_ripple.find_alignments(
    source_graph := 'https://example.org/ontologies/intkb',
    target_graph := 'https://example.org/ontologies/schemaorg',
    threshold    := 0.85
)
ORDER BY similarity DESC;

-- 对 Wikidata 也是一样。
SELECT * FROM pg_ripple.find_alignments(
    source_graph := 'https://example.org/ontologies/intkb',
    target_graph := 'https://example.org/ontologies/wikidata',
    threshold    := 0.85
)
ORDER BY similarity DESC;
```

## 第 4 步 —— 使用结构化 SHACL 作为门

您只想将*类*与*类*对齐，将*属性*与*属性*对齐。SHACL 形状阻止跨类型对齐错误：

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

# owl:equivalentClass 链接必须从 Class 指向 Class。
[] a sh:NodeShape ;
   sh:targetSubjectsOf owl:equivalentClass ;
   sh:property [ sh:path rdf:type ; sh:hasValue owl:Class ] .

# owl:equivalentProperty 链接必须从 Property 指向 Property。
[] a sh:NodeShape ;
   sh:targetSubjectsOf owl:equivalentProperty ;
   sh:property [ sh:path rdf:type ;
                 sh:in ( owl:ObjectProperty owl:DatatypeProperty owl:AnnotationProperty ) ] .
$TTL$);

ALTER SYSTEM SET pg_ripple.shacl_mode = 'sync';
SELECT pg_reload_conf();
```

## 第 5 步 —— 应用对齐

对于*精确*等价，使用 `owl:equivalentClass` / `owl:equivalentProperty`。对于*更广泛*的关系，使用 `rdfs:subClassOf` / `rdfs:subPropertyOf`。然后 OWL 2 RL 规则集将在两个方向上传播推理。

```sql
-- 高置信度的精确等价。
SELECT pg_ripple.insert_triple(
    s1, '<http://www.w3.org/2002/07/owl#equivalentClass>', s2
)
FROM pg_ripple.find_alignments(
    'https://example.org/ontologies/intkb',
    'https://example.org/ontologies/schemaorg', 0.95);

-- 中等置信度：子类而不是等价。
SELECT pg_ripple.insert_triple(
    s1, '<http://www.w3.org/2000/01/rdf-schema#subClassOf>', s2
)
FROM pg_ripple.find_alignments(
    'https://example.org/ontologies/intkb',
    'https://example.org/ontologies/schemaorg', 0.85)
WHERE similarity < 0.95;
```

## 第 6 步 —— 物化推理

```sql
SELECT pg_ripple.load_rules_builtin('owl-rl');
SELECT pg_ripple.infer('owl-rl');
```

在此之后，对 `?p a schema:Person` 的 SPARQL 查询返回每个是 `intkb:Employee` 的实体（反之亦然），无需应用程序级别的胶水代码。

---

## 选择等价还是子类

| 置信度 | 建议的公理 |
|---|---|
| ≥ 0.95 | `owl:equivalentClass` / `owl:equivalentProperty` |
| 0.85 – 0.95 | `rdfs:subClassOf`（单向泛化） |
| < 0.85 | 拒绝；提交给人类本体论者 |

双向的 `owl:equivalentClass` 是一个强有力的声明 —— 两个类具有*完全相同*的实例。如果源词汇表在边缘情况下不一致（例如 *未成年雇员* 与 *人类*），则较弱的 `rdfs:subClassOf` 更安全。

---

## 另请参阅

- [知识图谱嵌入](../features/knowledge-graph-embeddings.md)
- [OWL 2 Profile](../features/owl-profiles.md)
- [记录链接](../features/record-linkage.md) — 本体对齐的*个体*对应部分。