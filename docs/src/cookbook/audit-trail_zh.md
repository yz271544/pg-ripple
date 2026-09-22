# Cookbook：使用 PROV-O 和时间查询的审计跟踪

**目标。** 建立证据链，使您能够回答监管问题，例如*"在 3 月 21 日，您的系统是否告诉用户 X 事实 Y 为真？基于什么证据？"*。

**为什么选择 pg_ripple。** 三个可组合的功能 —— `point_in_time`、`prov_enabled`、`audit_log_enabled` —— 结合成纯 ML 管道无法产生的审计跟踪类型。此外 RDF-star 提供每个事实的出处。

**核心结果的时间。** ~10 分钟。

---

## 第 1 步 —— 开启每一层

```sql
ALTER SYSTEM SET pg_ripple.prov_enabled       = on;   -- 每次加载的 PROV-O
ALTER SYSTEM SET pg_ripple.audit_log_enabled  = on;   -- 每次 UPDATE 的日志
SELECT pg_reload_conf();
```

第三层 —— 用于每个事实的置信度/来源的 RDF-star 引用三元组 —— 与数据本身一起加载。

## 第 2 步 —— 加载具有每个事实出处的数据

```sql
SELECT pg_ripple.load_turtle($TTL$
@prefix ex:   <https://example.org/> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

ex:drugA ex:interactsWith ex:drugB .

# 注释事实本身。
<< ex:drugA ex:interactsWith ex:drugB >>
    ex:source     <https://pubmed.example/article/12345> ;
    ex:confidence "0.92"^^xsd:decimal ;
    ex:assertedAt "2026-03-15T09:00:00Z"^^xsd:dateTime ;
    prov:wasAttributedTo ex:loader/medkb-v3 .
$TTL$);
```

## 第 3 步 —— 时间流逝，数据变化

```sql
-- 在下个月发生几次更新。
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        <https://example.org/drugA> <https://example.org/manufacturer>
            <https://example.org/acme>
    }
');

-- 不同的角色进行更正。
SET ROLE editor_alice;
SELECT pg_ripple.sparql_update('
    DELETE DATA { <https://example.org/drugA> <https://example.org/manufacturer>
                  <https://example.org/acme> }
');
SELECT pg_ripple.sparql_update('
    INSERT DATA { <https://example.org/drugA> <https://example.org/manufacturer>
                  <https://example.org/acmecorp> }
');
RESET ROLE;
```

## 第 4 步 —— 回答监管者

问题是：*"在 3 月 21 日，您的系统是否告诉用户药物 A 与药物 B 相互作用？基于什么证据？"*

```sql
-- 1. 重放截至 3 月 21 日 12:00 的图。
SELECT pg_ripple.point_in_time('2026-03-21 12:00:00+00');

-- 2. 重新询问问题。
SELECT * FROM pg_ripple.sparql($$
    ASK { <https://example.org/drugA>
          <https://example.org/interactsWith>
          <https://example.org/drugB> }
$$);
-- → true

-- 3. 拉取每个事实的证据（RDF-star）。
SELECT * FROM pg_ripple.sparql($$
    SELECT ?source ?confidence ?assertedAt WHERE {
        << <https://example.org/drugA>
           <https://example.org/interactsWith>
           <https://example.org/drugB> >>
            <https://example.org/source>     ?source ;
            <https://example.org/confidence> ?confidence ;
            <https://example.org/assertedAt> ?assertedAt .
    }
$$);

-- 4. 拉取加载程序活动（PROV-O）。
SELECT * FROM pg_ripple.prov_stats()
WHERE  source_file LIKE '%medkb%';

-- 5. 拉取自加载以来的每个 UPDATE（审计日志）。
SELECT ts, role, operation, query
FROM   _pg_ripple.audit_log
WHERE  ts >= '2026-03-15'
   AND query ILIKE '%drugA%'
ORDER  BY ts;

-- 6. 将时间点重置为"现在"。
SELECT pg_ripple.point_in_time(NULL);
```

该序列是监管机构寻找的证据链：*某个时间点的真值*、*每个事实的证据*、*每个 UPDATE 的归属*。

---

## 构建一次性审计报告

将上述查询包装在 SQL 函数中，以便合规团队可以运行单个命令：

```sql
CREATE FUNCTION audit_report(
    fact_subject  TEXT,
    fact_predicate TEXT,
    fact_object   TEXT,
    as_of         TIMESTAMPTZ
)
RETURNS TABLE (kind TEXT, payload JSONB) AS $$
BEGIN
    PERFORM pg_ripple.point_in_time(as_of);

    RETURN QUERY
        SELECT 'truth_value'::TEXT,
               jsonb_build_object('asof', as_of, 'value', (
                   SELECT bindings FROM pg_ripple.sparql(
                       format('ASK { %s %s %s }', fact_subject, fact_predicate, fact_object)
                   )
               ));

    RETURN QUERY
        SELECT 'rdf_star_evidence'::TEXT, to_jsonb(s)
        FROM   pg_ripple.sparql(format($q$
            SELECT ?p ?o WHERE {
                << %s %s %s >> ?p ?o
            }
        $q$, fact_subject, fact_predicate, fact_object)) s;

    RETURN QUERY
        SELECT 'audit_log'::TEXT, to_jsonb(a)
        FROM   _pg_ripple.audit_log a
        WHERE  ts <= as_of AND query ILIKE '%' || fact_subject || '%'
        ORDER  BY ts;

    PERFORM pg_ripple.point_in_time(NULL);
END;
$$ LANGUAGE plpgsql;
```

```sql
SELECT * FROM audit_report(
    '<https://example.org/drugA>',
    '<https://example.org/interactsWith>',
    '<https://example.org/drugB>',
    '2026-03-21 12:00:00+00'
);
```

---

## 另请参阅

- [时间与出处](../features/temporal-and-provenance.md)
- [审计日志](../reference/audit-log.md)
- [存储知识 —— RDF-star](../features/storing-knowledge.md)