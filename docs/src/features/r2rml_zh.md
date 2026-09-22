# R2RML — 将关系表映射到 RDF

大多数企业已经将数据存储在关系表中。手动将数据重新建模为三元组是乏味且容易出错的。**R2RML**（[W3C 建议](https://www.w3.org/TR/r2rml/)）是标准映射语言，它声明性地表示：*"这一列变成这个谓词，这一行变成这个主语 IRI"*。pg_ripple 附带一个 R2RML 执行器，它在同一个数据库内运行映射 — 不需要 ETL 管道。

> 自 v0.55.0 起可通过 `pg_ripple.r2rml_load(mapping_ttl)` 获得。

---

## 为什么使用 R2RML？

| 没有 R2RML | 使用 R2RML |
|---|---|
| 每个应用软件专门的加载脚本 | 一个映射文档，可版本控制 |
| 映射逻辑分布在应用程序代码中 | 映射逻辑*就是* schema，使用标准语法 |
| 重新运行需要完整导出 | 增量重新运行；只有更改的行产生新三元组 |
| 本体论偏差不可见 | 本体论偏差作为 SHACL 违规出现 |
| 难以推理导出的内容 | R2RML 文档*就是*合约 |

如果您已经拥有 RDF 三元组存储并正在从 PostgreSQL 迁移，R2RML 允许您避免接触源 schema。如果您正在与关系 schema 同时在 R RDFS 之上构建 RDF 图，它可以让两者通过一个声明性伪文件保持同步。

---

## 工作示例

假设您有一个小型关系 schema：

```sql
CREATE TABLE customer (
    id          SERIAL PRIMARY KEY,
    full_name   TEXT NOT NULL,
    email       TEXT,
    country     CHAR(2)
);

CREATE TABLE purchase (
    id          SERIAL PRIMARY KEY,
    customer_id INT REFERENCES customer(id),
    sku         TEXT,
    amount_cents INT
);

INSERT INTO customer (full_name, email, country) VALUES
    ('Alice Chen', 'alice@example.org', 'US'),
    ('Bob Smith',  'bob@example.org',   'GB');

INSERT INTO purchase (customer_id, sku, amount_cents) VALUES
    (1, 'WIDGET-1', 1999),
    (2, 'WIDGET-2', 2999);
```

下面的 R2RML 映射将每一行转换为一个三元组簇。

```sql
SELECT pg_ripple.r2rml_load($TTL$
@prefix rr:    <http://www.w3.org/ns/r2rml#> .
@prefix rml:   <http://semweb.mmlab.be/ns/rml#> .
@prefix ex:    <https://example.org/> .
@prefix foaf:  <http://xmlns.com/foaf/0.1/> .
@prefix schema:<https://schema.org/> .

# 将 customer 行映射到 ex:customer/{id}
<#CustomerMap>
    rr:logicalTable      [ rr:tableName "customer" ] ;
    rr:subjectMap        [ rr:template "https://example.org/customer/{id}" ;
                           rr:class    foaf:Person ] ;
    rr:predicateObjectMap [ rr:predicate foaf:name  ; rr:objectMap [ rr:column "full_name" ] ] ;
    rr:predicateObjectMap [ rr:predicate foaf:mbox  ; rr:objectMap [ rr:template "mailto:{email}" ;
                                                                     rr:termType rr:IRI ] ] ;
    rr: predicateObjectMap [ rr:predicate schema:addressCountry ; rr:objectMap [ rr:column "country" ] ] .

# 将 purchase 行映射到 ex:purchase/{id}，并使用外键引用到 customer
<#PurchaseMap>
    rr:logicalTable      [ rr:tableName "purchase" ] ;
    rr:subjectMap        [ rr:template "https://example.org/purchase/{id}" ;
                           rr:class    schema:Order ] ;
    rr:predicateObjectMap [ rr:predicate schema:customer ;
                            rr:objectMap [ rr:template "https://example.org/customer/{customer_id}" ;
                                           rr:termType rr:IRI ] ] ;
    rr:predicateObjectMap [ rr:predicate schema:sku    ; rr:objectMap [ rr:column "sku" ] ] ;
    rr:predicateObjectMap [ rr:predicate schema:price  ; rr:objectMap [ rr:column "amount_cents" ;
                                                                        rr:datatype <http://www.w3.org/2001/XMLSchema#integer> ] ] .
$TTL$);
-- 返回生成的三元组数。
```

调用之后，您的知识图包含：

```turtle
<https://example.org/customer/1>
    a foaf:Person ;
    foaf:name "Alice Chen" ;
    foaf:mbox <mailto:alice@example.org> ;
    schema:addressCountry "US" .

<https://example.org/purchase/1>
    a schema:Order ;
    schema:customer <https://example.org/customer/1> ;
    schema:sku "WIDGET-1" ;
    schema:price 1999 .
```

…SPARQL 查询立即可用：

```sql
SELECT * FROM pg_ripple.sparql($$
    PREFIX schema: <https://schema.org/>
    PREFIX foaf:   <http://xmlns.com/foaf/0.1/>
    SELECT ?name ?sku WHERE {
        ?order schema:customer ?c ; schema:sku ?sku .
        ?c     foaf:name       ?name .
    }
$$);
```

---

## pg_ripple 的 R2RML 支持什么

| R2RML 功能 | 状态 |
|---|---|
| 带 `rr:template`、`rr:column`、`rr:constant` 的主语映射 | ✅ |
| 带 `rr:column`、`rr:template`、`rr:datatype`、`rr:language` 的谓词-对象映射 | ✅ |
| 主语映射上的 `rr:class` 快捷方式 | ✅ |
| 带 `rr:tableName` 或 `rr:sqlQuery` 的 `rr:logicalTable`（R2RML 视图）| ✅ |
| 两个三元组映射之间的 `rr:joinCondition` | ✅ |
| `rr:graphMap`（将三元组分配到命名图）| ✅ |
| `rr:termType`（`rr:IRI`、`rr:Literal`、`rr:BlankNode`）| ✅ |
| 用于非 SQL 源的 RML 扩展 | ❌ — 使用单独的 ETL 步骤 |

---

## 模式和配方

### 通过图映射实现每行来源

将表中的每个三元组路由到其自己的命名图，以便下游租户隔离或审计：

```turtle
<#CustomerMap>
    rr:graphMap [ rr:template "https://example.org/source/customer-table" ] ;
    ...
```

### 软删除处理

使用 `rr:sqlQuery` 限制导出的内容：

```turtle
<#ActiveCustomersMap>
    rr:logicalTable [ rr:sqlQuery "SELECT * FROM customer WHERE deleted_at IS NULL" ] ;
    ...
```

### 增量重新运行

R2RML 是幂等的：运行两次会产生相同的三元组（字典 ID 由 XXH3-128 哈希确定，因此不会累积重复项）。将其调度为在您的关系 ETL 之后运行的 cron 作业。

### 使用 SHACL 验证结果

将每个 R2RML 映射与编码输出*预期*形状的 SHACL 形状配对。该形状捕获映射错误和源数据偏差，在单次检查中：

```sql
SELECT pg_ripple.shacl_validate();
```

### 与 FDW 结合

`rr:tableName` 接受任何表 — 包括 `postgres_fdw` 提供的外部表。这允许您将*远程*关系数据库映射到本地三元组存储，而无需复制数据。

---

## 何时**不**使用 R2RML

- 源数据已经是 RDF（改用 `load_turtle()`）。
- 映射是一次性的，您永远不会重新运行它（手工编写的 INSERT 编写起来更快）。
- 您需要双向同步（R2RML 是单向的：关系 → RDF）。

---

## 另请参阅

- [加载数据](loading-data.md) — 用于直接 RDF 格式。
- [验证数据质量](validating-data-quality.md) — 将每次 R2RML 运行与 SHACL 配对。
- [Cookbook：来自关系目录的知识图](../cookbook/relational-to-rdf.md)

## 进一步阅读

- [博客：R2RML — 关系到图](https://github.com/trickle-labs/pg-ripple/blob/main/blog/r2rml-relational-to-rdf.md) — 将现有 PostgreSQL 表格映射到 RDF