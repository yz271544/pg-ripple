# Cookbook：SHACL + Datalog 数据质量管道

**目标。** 根据 SHACL 规则验证书目（或任何领域）图，识别违规，使用 Datalog 推理修复或丰富数据，并在单个可重现管道中确认质量。

**为什么使用 pg_ripple。** SHACL 验证和 Datalog 推理在同一个事务中共存。修复然后重新验证的循环完全在 SQL 中运行 — 无需往返外部验证器。

**首次结果时间。** 约 10 分钟。

---

## 模式

```
   原始数据加载
        │
        ▼
   定义 SHACL 形状
        │
        ▼
   运行 shacl_validate()  → 发现违规？
        │                        │
        │         yes            ▼
        │          ──── Datalog 推理丰富缺失的三元组
        │                        │
        │                        ▼
        │               再次运行 shacl_validate()
        │                        │
        ▼         无违规           ▼
   已接受                  拒绝/上报
```

---

## 第 1 步 — 加载不完美的图

示例使用书目图，其中一些书籍缺少必需的元数据：

```sql
CREATE EXTENSION IF NOT EXISTS pg_ripple;

SELECT pg_ripple.load_turtle($TTL$
@prefix bib:    <https://example.org/bib/> .
@prefix schema: <https://schema.org/> .
@prefix dc:     <http://purl.org/dc/terms/> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .

bib:book1  a schema:Book ;
    dc:title   "Foundations of Databases" ;
    dc:creator  bib:author1 ;
    dc:date     "1995"^^xsd:gYear .

-- 故意缺失：dc:creator 和 dc:date。
bib:book2  a schema:Book ;
    dc:title   "SPARQL 1.1 Query Language" .

bib:author1  a schema:Person ;
    schema:name "Abiteboul, Hull, Vianu" .
$TTL$);
```

## 第 2 步 — 定义 SHACL 形状

一本书必须有标题、至少一个创作者和出版日期。

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:     <http://www.w3.org/ns/shacl#> .
@prefix schema: <https://schema.org/> .
@prefix dc:     <http://purl.org/dc/terms/> .

<https://shapes.example.org/BookShape>  a sh:NodeShape ;
    sh:targetClass  schema:Book ;
    sh:property [
        sh:path      dc:title ;
        sh:minCount  1 ;
        sh:datatype  <http://www.w3.org/2001/XMLSchema#string>
    ] ;
    sh:property [
        sh:path     dc:creator ;
        sh:minCount 1 ;
        sh:message  "A book must have at least one creator."
    ] ;
    sh:property [
        sh:path     dc:date ;
        sh:minCount 1 ;
        sh:message  "A book must have a publication date."
    ] .
$TTL$);
```

## 第 3 步 — 运行第一个验证通过

```sql
SELECT *
FROM   pg_ripple.shacl_validate()
WHERE  severity = 'Violation'
ORDER  BY focus_node, result_path;
```

预期输出：

```
focus_node         | result_path | message
───────────────────┼─────────────┼────────────────────────────────────
bib:book2          | dc:creator  | A book must have at least one creator.
bib:book2          | dc:date     | A book must have a publication date.
```

两个违规。在上报给人工审核员之前，尝试用推理修复它们。

## 第 4 步 — 应用 Datalog 推理

假设您有一条规则：*"如果一本书的 W3C 规范 URI 与已知的 SPARQL 规范匹配，则将工作组推导为创作者"*。这是一个特定领域的修复规则。

```sql
SELECT pg_ripple.load_rules($RULES$
# 如果一本书有已知的 W3C 规范 URI，则推导 W3C SPARQL WG 为创作者。
?book dc:creator ex:W3C_SPARQL_WG :-
    ?book a schema:Book ,
    ?book dc:title "SPARQL 1.1 Query Language" .

# 从规范出版记录推导出版年份。
?book dc:date "2013"^^xsd:gYear :-
    ?book a schema:Book ,
    ?book dc:title "SPARQL 1.1 Query Language" .
$RULES$, 'bib_repair');

SELECT pg_ripple.infer('bib_repair');
```

对于更一般的推理，如果您的词汇表使用 `owl:sameAs` 或 `rdfs:subClassOf`，则具体化 OWL RL 公理：

```sql
SELECT pg_ripple.load_rules_builtin('owl-rl');
SELECT pg_ripple.infer('owl-rl');
```

## 第 5 步 — 推理后重新验证

```sql
SELECT count(*) AS remaining_violations
FROM   pg_ripple.shacl_validate()
WHERE  severity = 'Violation';
```

如果计数降至零，则数据质量管道通过。如果违规仍然存在，则将其上报给数据管理员：

```sql
-- 将剩余违规导出为 JSON 以供工单系统使用。
SELECT jsonb_agg(to_jsonb(v))
FROM   pg_ripple.shacl_validate() v
WHERE  severity = 'Violation';
```

## 第 6 步 — 使管道幂等

将步骤 3-5 包装在一个函数中，以便在每次加载后调用：

```sql
CREATE OR REPLACE FUNCTION run_quality_gate(
    rule_set TEXT DEFAULT 'bib_repair'
) RETURNS TABLE (violations BIGINT, violations_json JSONB) AS $$
BEGIN
    -- 重新运行推理以获取上次加载中的任何新三元组。
    PERFORM pg_ripple.infer(rule_set);

    RETURN QUERY
        SELECT count(*)::BIGINT,
               jsonb_agg(to_jsonb(v))
        FROM   pg_ripple.shacl_validate() v
        WHERE  v.severity = 'Violation';
END;
$$ LANGUAGE plpgsql;

SELECT * FROM run_quality_gate();
```

---

## 生产模式

### 加载时硬失败

设置 `pg_ripple.shacl_mode = 'sync'` — 任何创建违规的 SPARQL UPDATE 立即回滚。对于无效数据绝不能进入存储的模式使用此设置。

### 异步队列进行审查

设置 `pg_ripple.shacl_mode = 'async'` — 违规被写入 `_pg_ripple.shacl_violations` 而不会阻塞写入。定期作业检查队列并将标记的三元组路由到审查工作流。

### 置信度加权违规

将 SHACL 与 Datalog lattice 置信度相结合：仅上报违规的违规三元组的置信度低于 0.7（数据可能通过自动推理到达，而不是人工输入）。

```sql
SELECT v.focus_node, v.result_message, conf.confidence
FROM   pg_ripple.shacl_validate() v
JOIN   LATERAL (
    SELECT CAST(o AS FLOAT) AS confidence
    FROM   pg_ripple.sparql(format(
        'SELECT ?conf WHERE { << <%s> %s ?o >> ex:confidence ?conf }',
        v.focus_node, v.result_path
    ))
) conf ON true
WHERE  v.severity = 'Violation'
  AND  conf.confidence < 0.7;
```

---

## 另请参阅

- [验证数据质量（SHACL）](../features/validating-data-quality.md) — 所有形状类型和模式。
- [推理和推理（Datalog）](../features/reasoning-and-inference.md)
- [Cookbook：审计跟踪](audit-trail.md) — 将 SHACL 与 PROV-O 组合以获得完整证据链。