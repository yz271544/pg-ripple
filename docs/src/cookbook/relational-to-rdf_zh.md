# Cookbook：从关系目录构建知识图谱

**目标。** 您有一个用于产品目录的工作 PostgreSQL schema。您希望获得一个可查询的 RDF 图，它与该 schema **保持同步**，验证输出，并允许分析师在 SPARQL 中回答关系问题。

**为什么选择 pg_ripple。** 两个 schema 位于同一数据库中。没有 ETL 作业，没有最终一致性，一个 SHACL 契约保护输出。

**第一次结果的时间。** ~15 分钟。

---

## 您构建的内容

```
   关系表                 RDF 三元组                SPARQL 客户端
   ──────────                 ───────────                ──────────────
   product (id, name, …)   ──┐      <…/product/42> a Product
   category (id, parent_id) ──┼──►  <…/product/42> in <…/category/7>      ──►  pg_ripple.sparql(...)
   inventory (sku, qty)    ──┘      <…/product/42> sku "WIDGET-1"              "玩具层次结构中的所有
                                    <…/category/7> parent <…/category/3>        缺货产品"
                                    …
                                              ▲
                                              │
                                       SHACL 验证
                                       在每次加载时触发
```

---

## 第 1 步 —— 关系源

```sql
CREATE TABLE category (
    id        SERIAL PRIMARY KEY,
    name      TEXT NOT NULL,
    parent_id INT REFERENCES category(id)
);

CREATE TABLE product (
    id          SERIAL PRIMARY KEY,
    sku         TEXT UNIQUE NOT NULL,
    name        TEXT NOT NULL,
    category_id INT REFERENCES category(id),
    price_cents INT
);

CREATE TABLE inventory (
    product_id INT REFERENCES product(id),
    qty        INT NOT NULL DEFAULT 0
);
```

用几行填充它；此配方不关心数据。

## 第 2 步 —— 定义 R2RML 映射

```sql
SELECT pg_ripple.r2rml_load($TTL$
@prefix rr:    <http://www.w3.org/ns/r2rml#> .
@prefix ex:    <https://example.org/cat/> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix schema:<https://schema.org/> .
@prefix skos:  <http://www.w3.org/2004/02/skos/core#> .

<#CategoryMap>
    rr:logicalTable [ rr:tableName "category" ] ;
    rr:subjectMap   [ rr:template "https://example.org/cat/{id}" ;
                      rr:class    skos:Concept ] ;
    rr:predicateObjectMap [ rr:predicate skos:prefLabel ; rr:objectMap [ rr:column "name" ] ] ;
    rr:predicateObjectMap [ rr:predicate skos:broader   ;
                            rr:objectMap [ rr:template "https://example.org/cat/{parent_id}" ;
                                           rr:termType rr:IRI ] ] .

<#ProductMap>
    rr:logicalTable [ rr:tableName "product" ] ;
    rr:subjectMap   [ rr:template "https://example.org/product/{id}" ;
                      rr:class    schema:Product ] ;
    rr:predicateObjectMap [ rr:predicate schema:sku   ; rr:objectMap [ rr:column "sku" ] ] ;
    rr:predicateObjectMap [ rr:predicate schema:name  ; rr:objectMap [ rr:column "name" ] ] ;
    rr:predicateObjectMap [ rr:predicate schema:category ;
                            rr:objectMap [ rr:template "https://example.org/cat/{category_id}" ;
                                           rr:termType rr:IRI ] ] ;
    rr:predicateObjectMap [ rr:predicate schema:price ;
                            rr:objectMap [ rr:column "price_cents" ;
                                           rr:datatype xsd:integer ] ] .

<#InventoryMap>
    rr:logicalTable [ rr:sqlQuery "SELECT product_id, qty FROM inventory" ] ;
    rr:subjectMap   [ rr:template "https://example.org/product/{product_id}" ] ;
    rr:predicateObjectMap [ rr:predicate <https://example.org/onHand> ;
                            rr:objectMap [ rr:column "qty" ; rr:datatype xsd:integer ] ] .
$TTL$);
```

## 第 3 步 —— 定义 SHACL 契约

这是此配方中最被低估的技巧。形状编码输出的*预期形状*。如果源 schema 发生偏移（列被重命名、外键被删除），SHACL 运行会立即标记它。

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:    <http://www.w3.org/ns/shacl#> .
@prefix schema:<https://schema.org/> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:    <https://example.org/> .

<https://shapes.example.org/ProductShape> a sh:NodeShape ;
    sh:targetClass schema:Product ;
    sh:property [ sh:path schema:sku   ; sh:minCount 1 ; sh:maxCount 1 ] ;
    sh:property [ sh:path schema:name  ; sh:minCount 1 ; sh:datatype xsd:string ] ;
    sh:property [ sh:path schema:category ; sh:minCount 1 ; sh:nodeKind sh:IRI ] ;
    sh:property [ sh:path schema:price ; sh:datatype xsd:integer ; sh:minInclusive 0 ] ;
    sh:property [ sh:path ex:onHand    ; sh:maxCount 1 ; sh:datatype xsd:integer ] .
$TTL$);

ALTER SYSTEM SET pg_ripple.shacl_mode = 'sync';
SELECT pg_reload_conf();
```

## 第 4 步 —— 增量重新运行

安排 R2RML 加载在每次关系 ETL 传递后运行。因为字典 ID 是确定性的，重复加载是幂等的 —— 只有更改的行才会生成新的三元组。

```sql
-- Cron 作业，每分钟一次：
SELECT pg_ripple.r2rml_reload();   -- 简写：重新运行最近的 r2rml_load()
SELECT * FROM pg_ripple.shacl_validate() LIMIT 10;
```

如果 `shacl_validate()` 返回行，则关系源已发生偏移，并触发警报。

## 第 5 步 —— 在 SPARQL 中查询

```sql
SELECT * FROM pg_ripple.sparql($$
    PREFIX schema: <https://schema.org/>
    PREFIX skos:   <http://www.w3.org/2004/02/skos/core#>
    PREFIX ex:     <https://example.org/>

    # "Toys"层次结构中的缺货产品。
    SELECT ?sku ?name WHERE {
        ?p a schema:Product ;
           schema:sku ?sku ;
           schema:name ?name ;
           schema:category/skos:broader* <https://example.org/cat/toys> ;
           ex:onHand 0 .
    }
$$);
```

`skos:broader*` 属性路径在类别层次上展开，没有递归深度上限。尝试用纯 SQL 表达它，您将重新发现为什么存在图查询。

---

## 变体

- **多源。** 向每个映射添加 `rr:graphMap`，使三元组落入每个源的命名图（`https://example.org/source/postgres-prod`）。然后[多租户图](../features/multi-tenant-graphs.md)为您提供每个源的 RLS。
- **软删除。** 将 `rr:tableName` 替换为过滤掉 `deleted_at IS NOT NULL` 的 `rr:sqlQuery`。重新运行将*删除*相应的三元组。
- **外部数据包装器。** `rr:tableName` 接受外部表，因此您可以摄取远程 PostgreSQL 或 MySQL schema 而无需复制数据。

---

## 另请参阅

- [R2RML](../features/r2rml.md)
- [验证数据质量](../features/validating-data-quality.md)