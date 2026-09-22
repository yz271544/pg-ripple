# Cookbook：跨系统去重客户记录

**目标。** 两个源系统各自保存客户列表。有些客户同时存在于两者，但存在细微差异（`Robert Smith` vs `Bob Smith`，`JaneDoe` vs `Jane Q. Doe`）。您希望获得一个统一的视图，其中每个真实世界的客户显示为一个实体，同时原始记录仍然可审计。

**为什么使用 pg_ripple。** 结合知识图嵌入（高召回率）、SHACL 硬规则（安全）和 `owl:sameAs` 规范化（查询时透明）— 记录链接管道所需的三个部分，无需外部服务。

**首次结果时间。** 约 20 分钟。

> 本食谱是[记录链接和实体解析](../features/record-linkage.md) 的实用版本。先阅读该页面了解战略背景。

---

## 第 1 步 — 将两个源加载到命名图中

命名图保留了每条记录的原始出处。

```sql
SELECT pg_ripple.load_turtle_into_graph('https://example.org/source/crm', $TTL$
@prefix ex:   <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.0.1/> .

ex:crm/c1 foaf:name "Robert Smith"  ; foaf:mbox <mailto:rob@x.com> ; ex:dob "1985-03-12"^^<http://www.w3.org/2001/XMLSchema#date> .
ex:crm/c2 foaf:name "Jane Doe"      ; foaf:mbox <mailto:jane@x.com>; ex:dob "1990-07-09"^^<http://www.w3.org/2001/XMLSchema#date> .
$TTL$);

SELECT pg_ripple.load_turtle_into_graph('https://example.org/source/erp', $TTL$
@prefix ex:   <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.0.1/> .

ex:erp/c1 foaf:name "Bob Smith"     ; foaf:mbox <mailto:rob@x.com> ; ex:dob "1985-03-12"^^<http://www.w3.org/2001/XMLSchema#date> .
ex:erp/c2 foaf:name "Jane Q. Doe"   ; foaf:mbox <mailto:j.doe@x.com>; ex:dob "1990-07-09"^^<http://www.w3.org/2001/XMLSchema#date> .
ex:erp/c3 foaf:name "Carl Larsen"   ; foaf:mbox <mailto:carl@y.com>; ex:dob "1972-11-02"^^<http://www.w3.org/2001/XMLSchema#date> .
$TTL$);
```

## 第 2 步 — 添加结构化上下文

当客户拥有*相关的*事实（购买、地址、互动历史）时，他们具有更强的匹配能力。加载您拥有的任何内容。下面的示例使用购买：

```sql
SELECT pg_ripple.load_turtle_into_graph('https://example.org/source/crm', $TTL$
@prefix ex: <https://example.org/> .
ex:crm/c1 ex:purchased ex:product/widget1, ex:product/widget2 .
ex:crm/c2 ex:purchased ex:product/widget3 .
$TTL$);

SELECT pg_ripple.load_turtle_into_graph('https://example.org/source/erp', $TTL$
@prefix ex: <https://example.org/> .
ex:erp/c1 ex:purchased ex:product/widget1, ex:product/widget4 .
ex:erp/c2 ex:purchased ex:product/widget3, ex:product/widget5 .
$TTL$);
```

## 第 3 步 — 生成候选对

运行**两个**文本和 KGE 候选生成器，然后合并结果。它们捕获不同的错误。

```sql
-- 文本嵌入候选。
SELECT pg_ripple.embed_entities();

CREATE TEMP TABLE candidates AS
SELECT s1, s2, similarity, 'text' AS source
FROM pg_ripple.suggest_sameas(threshold := 0.85);

-- KGE 候选。
SET pg_ripple.kge_enabled = on;
SELECT pg_ripple.kge_train(model := 'TransE', epochs := 100);

INSERT INTO candidates
SELECT s1, s2, similarity, 'kge'
FROM pg_ripple.find_alignments(
    source_graph := 'https://example.org/source/crm',
    target_graph := 'https://example.org/source/erp',
    threshold    := 0.85
);

SELECT * FROM candidates ORDER BY similarity DESC;
```

## 第 4 步 — 使用 SHACL 阻止不安全的合并

`dob`（出生日期）是不可变的且每个人唯一。两条具有不同 DOB 的记录不能是同一个人，无论它们的其他属性看起来多么相似。

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix ex:   <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.0.1/> .

ex:CustomerSafetyShape a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    # 如果 owl:sameAs 链接两个人，他们的 dob 必须一致。
    sh:property [ sh:path ex:dob ; sh:maxCount 1 ] .
$TTL$);

ALTER SYSTEM SET pg_ripple.shacl_mode = 'sync';
SELECT pg_reload_conf();
```

当 `apply_sameas_candidates()` 创建违规（合并的人有两个 `dob` 值）时，插入被拒绝 — 不安全的合并无法发生。

## 第 5 步 — 应用自动合并层

```sql
-- 高置信度对自动合并。
SELECT pg_ripple.apply_sameas_candidates(min_similarity := 0.95);

-- 中等置信度对进入审核表。
CREATE TABLE customer_review_queue AS
SELECT DISTINCT ON (least(s1, s2), greatest(s1, s2))
       s1, s2, max(similarity) AS similarity
FROM   candidates
WHERE  similarity BETWEEN 0.85 AND 0.95
GROUP  BY s1, s2;

SELECT * FROM customer_review_queue ORDER BY similarity DESC;
```

审核员将 `customer_review_queue` 中的对标记为 `accepted` 或 `rejected`；`accepted` 行触发：

```sql
SELECT pg_ripple.insert_triple(s1, '<http://www.w3.org/2002/07/owl#sameAs>', s2),
       pg_ripple.insert_triple(s2, '<http://www.w3.org/2002/07/owl#sameAs>', s1)
FROM   customer_review_queue WHERE status = 'accepted';
```

## 第 6 步 — 查询统一的图

`pg_ripple.sameas_reasoning = on`（默认值）意味着 SPARQL 查询将合并的客户视为一个实体：

```sql
-- Robert/Bob 的总支出 — 跨 CRM 和 ERP 合并。
SELECT * FROM pg_ripple.sparql($$
    PREFIX ex: <https://example.org/>
    SELECT (COUNT(?p) AS ?n) WHERE {
        <https://example.org/crm/c1> ex:purchased ?p .
    }
$$);
```

查询以 CRM 标识符为目标，但 `sameas_reasoning` 透明地将其重写为包括 ERP 标识符。

---

## 审计合并

每次合并操作都由[审计日志](../reference/audit-log.md)捕获。与 [point_in_time](../features/temporal-and-provenance.md) 结合，监管机构可以重放在任何过去时间戳系统的确切想法。

```sql
SELECT ts, role, query
FROM   _pg_ripple.audit_log
WHERE  query ILIKE '%owl:sameAs%'
ORDER  BY ts DESC
LIMIT 50;
```

---

## 另请参阅

- [记录链接](../features/record-linkage.md) — 战略背景。
- [知识图嵌入](../features/knowledge-graph-embeddings.md)
- [验证数据质量](../features/validating-data-quality.md)