# 时间查询和出处

任何数据系统中最常被问到的两个合规问题是：

1. *"上周二 03:14 时图是什么样子？"* — 一个**时间**问题。
2. *"这个事实来自哪里，是谁引入的？"* — 一个**出处**问题。

pg_ripple 通过一流的功能回答这两个问题，无需 schema 更改和热路径上的额外存储。

| 能力 | 函数 / GUC | 页面部分 |
|---|---|---|
| 在过去的时间戳重放图 | `pg_ripple.point_in_time(ts)` | [时间查询](#时间查询) |
| 为每次批量加载记录 `prov:Activity` | `pg_ripple.prov_enabled = on` | [PROV-O 出处](#prov-o-出处) |
| 捕获每个 SPARQL UPDATE | `pg_ripple.audit_log_enabled = on` | [审计日志](#审计日志) |
| 每个事实的置信度和来源 | RDF-star 引用三元组 | [存储知识 → RDF-star](storing-knowledge.md) |

这四个功能被设计为可组合的。监管机构可辩护的审计日志通常使用全部四个。

---

## 时间查询

pg_ripple 中的每个三元组都带有一个全局唯一的语句 ID（`i BIGINT`），它来自共享序列。序列值随插入时间单调增加，因此它充当逻辑时间戳。`_pg_ripple.statement_id_timeline` 表将挂钟时间戳映射到 SID 范围。

`point_in_time(ts)` 设置一个会话 GUC，将每个后续的 SPARQL 和 Datalog 查询限制为在 `ts` 时存在的 SID 范围。

```sql
-- 查看上周二 03:14 时图的状态。
SELECT pg_ripple.point_in_time('2026-04-21 03:14:00+00'::timestamptz);

-- 此会话中的所有查询现在都限制在该时间点。
SELECT * FROM pg_ripple.sparql('
    SELECT ?paper ?title WHERE {
        ?paper <http://purl.org/dc/elements/1.1/title> ?title
    }
');

-- 重置为 "now"。
SELECT pg_ripple.point_in_time(NULL);
```

### 这适用于什么

- **审计**：证明您的应用程序*本应*在事件发生时向用户显示的内容。
- **可重现的分析**：针对原始报告所看到的准确数据重新运行报告。
- **调试**：通过反向遍历时间对质量回归进行二分查找。
- **历史变更检测**：与 [CDC 订阅](cdc-subscriptions.md) 结合以查看*两个时间点之间发生了什么变化*。

### 这**不**适用于什么

- 它**不是**完整的时间二段存储。您无法查询"在三月被*相信*但在*一*月有效的事实 X 的版本"。为此，您需要 RDF-star 以及您自己的有效时间谓词。
- 它在 `_pg_ripple.statement_id_timeline` 的 `VACUUM FULL` 期间**不**安全。常规的 `VACUUM` 是可以的。

---

## PROV-O 出处

[PROV-O](https://www.w3.org/TR/prov-o/) 是用于描述数据来源的 W3C 标准。设置一个 GUC，每次批量加载操作都会自动使用 `prov:Activity` 和 `prov:Entity` 三元组进行注释。

```sql
SET pg_ripple.prov_enabled = on;

-- 每次加载都被记录。
SELECT pg_ripple.load_turtle_file('/data/products.ttl');
SELECT pg_ripple.load_nquads_file('/data/customers.nq');

-- 检查记录的活动。
SELECT * FROM pg_ripple.prov_stats();
```

典型的记录活动如下所示：

```turtle
_:act_42 a prov:Activity ;
    prov:startedAtTime "2026-04-27T10:00:00Z"^^xsd:dateTime ;
    prov:endedAtTime   "2026-04-27T10:00:14Z"^^xsd:dateTime ;
    prov:wasAssociatedWith <urn:postgres-role:loader_service> ;
    pg:loadFunction       "load_turtle_file" ;
    pg:sourceFile         "/data/products.ttl" ;
    pg:tripleCount        128432 .
```

您可以使用 SPARQL 像任何其他数据一样查询出处。PROV-O 与 `point_in_time()` 干净地集成 — *"截至昨天收盘，哪个 loader 接触了这个图？"* 是一个查询。

---

## 审计日志

虽然 PROV-O 捕获批量加载，但**审计日志**捕获每个 SPARQL UPDATE — `INSERT DATA`、`DELETE DATA`、`INSERT { … } WHERE`、`MOVE`、`COPY`、`LOAD` 等。设置 GUC 后，条目将出现在 `_pg_ripple.audit_log` 中：

```sql
SET pg_ripple.audit_log_enabled = on;

-- 每个 UPDATE 都被捕获。
SELECT pg_ripple.sparql_update('
    INSERT DATA { <https://example.org/x> <https://example.org/y> "z" }
');

-- 检查。
SELECT ts, role, txid, operation, query
FROM _pg_ripple.audit_log
ORDER BY ts DESC
LIMIT 20;

-- 清理（例如，每晚）。
SELECT pg_ripple.purge_audit_log(before := now() - interval '90 days');
```

审计日志是一个 PostgreSQL 表 — 您可以像任何其他表一样将其传输到 SIEM、分区或复制它。

### 使用 RDF-star 进行每事实出处

对于细粒度的、*每个三元组*的出处（而不是每次加载或每次更新），请使用 RDF-star 引用三元组。有关语法，请参阅 [Storing Knowledge → RDF-star](storing-knowledge.md)。最常见的模式：

```turtle
<< <:alice> <:knows> <:bob> >>
    :assertedBy <:dataset/foaf2024> ;
    :confidence "0.95"^^xsd:decimal ;
    :timestamp  "2026-04-27"^^xsd:date .
```

这些引用三元组可以使用与普通三元组相同的 SPARQL 模式进行查询。

---

## 将它们组合在一起 — 工作示例

监管机构问：*"3 月 21 日，您的系统是否告诉用户药物 A 与药物 B 相互作用？如果是，依据什么证据？"*

```sql
-- 1. 重放图的状态。
SELECT pg_ripple.point_in_time('2026-03-21 12:00:00+00');

-- 2. 重新提出问题。
SELECT * FROM pg_ripple.sparql('
    ASK { <https://example.org/drugA> <https://example.org/interactsWith> <https://example.org/drugB> }
');
-- → true

-- 3. 查找证据（RDF-star + PROV-O）。
SELECT * FROM pg_ripple.sparql('
    SELECT ?source ?confidence ?activity WHERE {
        << <https://example.org/drugA> <https://example.org/interactsWith> <https://example.org/drugB> >>
            <https://example.org/source>     ?source ;
            <https://example.org/confidence> ?confidence .
        ?activity <http://www.w3.org/ns/prov#generated> ?source .
    }
');

-- 4. 查找加载它的操作员。
SELECT * FROM pg_ripple.sparql('
    SELECT ?role WHERE {
        ?activity <http://www.w3.org/ns/prov#wasAssociatedWith> ?role
    }
');
```

这四行链是受监管行业所需的证据类型，黑盒系统无法产生这种证据。

---

## 性能和存储说明

- `point_in_time()` 是**只读**的，在写入路径上零成本。
- `_pg_ripple.statement_id_timeline` 表是一个小型仅追加日志：每个时间戳边界约 24 字节。24/7 存储每天累积几 KB。
- `prov_enabled` 每次批量加载添加约 5–10 个三元组。对于任何非平凡的加载可以忽略不计。
- `audit_log_enabled` 每个 UPDATE 语句写入一行。对于 OLTP 重负载工作负载，请考虑按月对该表进行分区。
- 所有这三个功能默认关闭。根据您的合规姿势按数据库启用它们。

---

## 另请参阅

- [CDC 订阅](cdc-subscriptions.md) — 实时推送*更改*，补充时间点*重放*。
- [多租户图](multi-tenant-graphs.md) — 将审计日志与 RLS 配对，将每次更改归属于谁，绑定到租户角色。
- [Cookbook：使用 PROV-O 和时间查询的审计跟踪](../cookbook/audit-trail.md)

## 进一步阅读

- [博客：时间时间旅行查询](https://github.com/trickle-labs/pg-ripple/blob/main/blog/temporal-time-travel-queries.md) — 知识图的时间点重放
- [博客：使用 PROV-O 的出处跟踪](https://github.com/trickle-labs/pg-ripple/blob/main/blog/provenance-tracking-prov-o.md) — 跟踪每个事实来自哪里