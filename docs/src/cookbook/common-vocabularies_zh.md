# 常见词汇表 Bundle：DCTERMS、Schema.org 和 FOAF

本章展示如何在 pg_ripple 中加载和使用三个最广泛部署的链接数据词汇表 —— Dublin Core Terms（DCTERMS）、Schema.org 和 FOAF —— 使用 v0.98.0 中引入的命名 bundle API。

与 SKOS（v0.98.0）一起，这些 bundle 完成了"Big 5"词汇表，覆盖了绝大多数现实世界知识图谱互操作性的需求。

## 为什么选择这些词汇表？

| 词汇表 | 命名空间前缀 | 典型用途 |
|---|---|---|
| Dublin Core Terms | `dcterms:` | 元数据 —— 创建者、日期、主题、权利、格式 |
| Schema.org | `schema:` | 结构化数据 —— 人员、组织、产品、地点、事件 |
| FOAF | `foaf:` | 社交图谱数据 —— 人员、账户、社交连接 |

所有这三个已经在数十亿个公共 RDF 文档、Wikidata 导出和企业知识图谱中使用。加载它们的规则集允许 pg_ripple 对您已有的数据执行属性层次结构推理、类型子集推理和完整性验证 —— 无需任何 schema 更改。

## 加载 Bundle

```sql
-- DCTERMS：11 条 Datalog 规则 + 8 个完整性验证器
SELECT pg_ripple.load_datalog_bundle('dcterms');
SELECT pg_ripple.load_shape_bundle('dcterms-integrity');

-- Schema.org：15 条 Datalog 规则 + 6 个完整性验证器
SELECT pg_ripple.load_datalog_bundle('schema');
SELECT pg_ripple.load_shape_bundle('schema-integrity');

-- FOAF：8 条 Datalog 规则 + 5 个完整性验证器
SELECT pg_ripple.load_datalog_bundle('foaf');
SELECT pg_ripple.load_shape_bundle('foaf-integrity');
```

加载后，确认激活：

```sql
SELECT bundle_name, bundle_version, activated_at
FROM pg_ripple.active_datalog_bundles
ORDER BY activated_at;
```

## DCTERMS —— Dublin Core Terms

### 推断属性

`dcterms` bundle 添加了 11 条规则，包括：

- **DC11 兼容性别名** — `dc11:title` 被视为等同于 `dcterms:title`，类似的还有 `creator`、`subject`、`description`、`publisher`、`date`、`type`、`format`、`identifier`、`source`、`language`。
- **结构逆** — `dcterms:hasPart` ↔ `dcterms:isPartOf`，`dcterms:hasVersion` ↔ `dcterms:isVersionOf`，`dcterms:replaces` ↔ `dcterms:isReplacedBy`。
- **DC-SKOS 桥接**（DC-SKOS-01）— `dcterms:subject` 是 SKOS `ConceptScheme` 成员的资源自动接收 `skos:Concept` 类型断言。

### 示例

```sql
-- 使用旧的 dc11: 命名空间加载一些元数据三元组。
SELECT pg_ripple.load_turtle($$
@prefix dc11: <http://purl.org/dc/elements/1.1/> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix ex: <https://example.org/> .

ex:Article1 dc11:creator "Alice" ;
            dcterms:hasPart ex:Section1 .
$$);

-- 加载 dcterms 规则集。
SELECT pg_ripple.load_datalog_bundle('dcterms');

-- dc11:creator 现在映射到 dcterms:creator。
SELECT pg_ripple.sparql_select($$
  SELECT ?article ?creator
  WHERE { ?article <http://purl.org/dc/terms/creator> ?creator }
$$);

-- 从 dcterms:hasPart 推断 dcterms:isPartOf。
SELECT pg_ripple.sparql_select($$
  SELECT ?part ?whole
  WHERE { ?part <http://purl.org/dc/terms/isPartOf> ?whole }
$$);
```

## Schema.org —— 结构化数据词汇表

### 推断属性

`schema` bundle 添加了 15 条规则：

- **逆对** — `schema:subjectOf` ↔ `schema:about`，`schema:hasPart` ↔ `schema:isPartOf`，`schema:workExample` ↔ `schema:exampleOfWork`，`schema:member` ↔ `schema:memberOf`。
- **类型层次结构** — `schema:LocalBusiness` ⊆ `schema:Organization` ⊆ `schema:Thing`，`schema:Person` ⊆ `schema:Thing`，`schema:Product` ⊆ `schema:Thing` 等。
- **跨词汇表桥接** — `schema:author` → `foaf:maker`（SCHEMA-FOAF-01），`schema:name` → `dcterms:title`（SCHEMA-DC-01），`schema:dataset` → `dcat:Dataset`（SCHEMA-DCAT-01）。

### SQL 辅助函数：`schema_type_ancestors`

```sql
-- 返回给定 IRI 的所有 Schema.org 类型祖先。
SELECT ancestor_type
FROM pg_ripple.schema_type_ancestors('https://schema.org/LocalBusiness');
-- 返回：schema:LocalBusiness, schema:Organization, schema:Thing
```

### 示例

```sql
-- 加载本地商家。
SELECT pg_ripple.load_turtle($$
@prefix schema: <https://schema.org/> .
@prefix ex: <https://example.org/> .

ex:AcmeCafe a schema:CafeOrCoffeeShop ;
    schema:name "ACME Café" ;
    schema:address ex:CafeAddress .
$$);

SELECT pg_ripple.load_datalog_bundle('schema');

-- CafeOrCoffeeShop 是 FoodEstablishment、LocalBusiness、Organization、Thing。
SELECT pg_ripple.sparql_select($$
  SELECT ?type
  WHERE { <https://example.org/AcmeCafe> a ?type }
  ORDER BY ?type
$$);
```

## FOAF —— Friend-of-a-Friend

### 推断属性

`foaf` bundle 添加了 8 条规则：

- **`foaf:knows` 对称性** — 如果 Alice 认识 Bob，则 Bob 认识 Alice。
- **类型子集** — `foaf:Person` ⊆ `foaf:Agent`，`foaf:Organization` ⊆ `foaf:Agent`，`foaf:Group` ⊆ `foaf:Agent`。
- **逆属性** — `foaf:account` ↔ `foaf:accountFor`，`foaf:made` ↔ `foaf:maker`。
- **DC-FOAF 桥接**（DC-FOAF-01）— `dcterms:creator` 三元组在引用的资源上生成相应的 `foaf:maker` 断言。

### SQL 辅助函数：`foaf_persons`

```sql
-- 返回所有 foaf:Person IRI 及其 foaf:name 标签。
SELECT person_iri, name_label
FROM pg_ripple.foaf_persons();
```

### 示例

```sql
-- 加载 FOAF 数据。
SELECT pg_ripple.load_turtle($$
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix ex: <https://example.org/> .

ex:Alice a foaf:Person ;
    foaf:name "Alice" ;
    foaf:knows ex:Bob .
ex:Bob a foaf:Person ;
    foaf:name "Bob" .
$$);

SELECT pg_ripple.load_datalog_bundle('foaf');

-- foaf:knows 是对称的：Bob 也认识 Alice。
SELECT pg_ripple.sparql_select($$
  SELECT ?who ?knows
  WHERE { ?who <http://xmlns.com/foaf/0.1/knows> ?knows }
$$);

-- foaf:Person ⊆ foaf:Agent：Alice 也是 foaf:Agent。
SELECT pg_ripple.sparql_select($$
  SELECT ?agent
  WHERE { ?agent a <http://xmlns.com/foaf/0.1/Agent> }
$$);
```

## 跨词汇表推理

当所有三个 bundle（和 SKOS）一起加载时，跨词汇表桥接被激活：

| 规则 ID | 效果 |
|---|---|
| DC-FOAF-01 | `dcterms:creator ?x` → `foaf:maker ?x` |
| SCHEMA-FOAF-01 | `schema:author ?x` → `foaf:maker ?x` |
| SCHEMA-DC-01 | `schema:name ?x` → `dcterms:title ?x` |
| DC-SKOS-01 | `dcterms:subject ?x` 其中 `?x` 在 ConceptScheme 中 → `?x a skos:Concept` |

这意味着使用 `schema:author` 描述的资源可以通过 `foaf:maker` 查询找到，具有 `schema:name` 的资源可以通过 `dcterms:title` 过滤器找到 —— 无需任何数据转换。

## 完整性验证

每个词汇表都附带一个相应的完整性 bundle：

```sql
-- 验证 DCTERMS 约束（标题基数、日期格式等）
SELECT pg_ripple.load_shape_bundle('dcterms-integrity');

-- 验证 Schema.org 约束（必需名称、URL 格式、价格范围等）
SELECT pg_ripple.load_shape_bundle('schema-integrity');

-- 验证 FOAF 约束（主页 IRI、mbox mailto:、名称基数）
SELECT pg_ripple.load_shape_bundle('foaf-integrity');
```

完整性违规通过标准 SHACL 验证报告呈现。

## 检查活动 Bundle

```sql
SELECT bundle_name, bundle_version, activated_at
FROM pg_ripple.active_datalog_bundles
ORDER BY bundle_name;

-- bundle_name         | bundle_version | activated_at
-- --------------------+----------------+---------------------------
-- dcterms             | 0.99.0         | 2026-05-07 10:00:00+00
-- foaf                | 0.99.0         | 2026-05-07 10:00:01+00
-- schema              | 0.99.0         | 2026-05-07 10:00:02+00
-- skos                | 0.98.0         | 2026-05-07 09:59:59+00
```

## 另请参阅

- [SKOS 词库管理](skos-thesaurus.md) — "Big 5"词汇表 bundle 套件的第四个成员
- [SHACL + Datalog 数据质量管道](shacl-datalog-quality.md) — 如何将规则和形状组合成完整的数据质量管道