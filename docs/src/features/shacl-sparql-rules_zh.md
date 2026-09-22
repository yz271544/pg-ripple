# SHACL-SPARQL 规则和约束

标准的 SHACL Core 涵盖了实践中约 95% 的约束 — 基数、数据类型、值范围、属性路径。剩下 5% 是 SHACL Core 表达力不足的地方：跨形状条件、复杂的逻辑组合、"此属性必须等于那些属性的总和"，等等。**SHACL-SPARQL**（[Advanced Features](https://www.w3.org/TR/shacl-af/)）通过允许您在形状内嵌入 SPARQL 查询来弥补这一差距。

pg_ripple 支持 `sh:SPARQLConstraint`（验证）、`sh:TripleRule`（推理）和 `sh:SPARQLRule`（基于 SPARQL CONSTRUCT 的推理，v0.79.0 中添加）。

---

## `sh:SPARQLConstraint` — 自定义验证

`sh:SPARQLConstraint` 为每个焦点节点运行一个 `ASK` 或 `SELECT` 查询。如果查询返回 true（对于 `ASK`）或任何行（对于 `SELECT`），pg_ripple 会记录违规。

经典示例是 *"人的出生日期必须早于其死亡日期"* —— 不能用纯 SHACL Core 表示，因为它需要比较同一节点的两个属性：

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix ex:   <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:LifeSpanShape a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    sh:sparql [
        sh:message  "出生日期必须早于死亡日期" ;
        sh:select   """
            SELECT $this WHERE {
                $this <https://example.org/birthDate> ?b ;
                      <https://example.org/deathDate> ?d .
                FILTER(?d <= ?b)
            }
        """ ;
    ] .
$TTL$);

-- 验证整个存储。
SELECT focus_node, message FROM pg_ripple.shacl_validate();
```

在查询内部，特殊变量 `$this` 绑定到焦点节点。查询由您用于任何其他查询的相同 SPARQL 引擎求值，因此您可以在 SPARQL 中编写的任何内容 — 属性路径、FILTER、BIND、子 SELECT — 在约束内都可以使用。

---

## `sh:TripleRule` — 从形状推理

`sh:TripleRule` 为每个匹配形状的焦点节点向存储添加三元组。它是 pg_ripple 中推荐的 SHACL-AF 推理原语，因为它直接编译为 Datalog 规则。

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix ex:   <https://example.org/> .

ex:AdultRule a sh:NodeShape ;
    sh:targetClass <https://schema.org/Person> ;
    sh:rule [
        a sh:TripleRule ;
        sh:subject   sh:this ;
        sh:predicate ex:isAdult ;
        sh:object    "true"^^<http://www.w3.org/2001/XMLSchema#boolean> ;
    ] .
$TTL$);

-- 应用规则。
SELECT pg_ripple.shacl_apply_rules();
```

由 `sh:TripleRule` 生成的三元组以 `source = 1`（推断）写入 — 它们与显式三元组共存并保持可区分性。

---

## `sh:SPARQLRule` — 从验证形状推理

`sh:SPARQLRule` 运行一个 `CONSTRUCT` 查询，其图模式针对每个焦点节点进行求值，构造的三元组将添加到存储中。这本质上是"用 SPARQL 拼写的 Datalog" — 当您的验证已经存在于 SHACL 中，并且您希望从同一数据派生新事实而无需编写单独的 Datalog 规则集时很有用。

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix ex:   <https://example.org/> .

ex:AdultRule a sh:NodeShape ;
    sh:targetClass <https://schema.org/Person> ;
    sh:rule [
        a sh:SPARQLRule ;
        sh:construct """
            CONSTRUCT { $this a ex:Adult }
            WHERE     { $this <https://example.org/age> ?age .
                        FILTER(?age >= 18) }
        """ ;
    ] .
$TTL$);

-- 应用规则。
SELECT pg_ripple.shacl_apply_rules();
```

由 `sh:SPARQLRule` 构造的三元组以 `source = 1`（推断）写入 — 它们与显式三元组共存并保持可区分性。

---

## 何时使用 SHACL-SPARQL 与 Datalog

两者都可以表达自定义验证和推理。按受众选择：

| 关注点 | SHACL-SPARQL | Datalog |
|---|---|---|
| **受众** | 已经编写 SHACL 的数据架构师 | 熟悉逻辑编程的工程师 |
| **工具** | 标准 SHACL 编辑器和验证器 | pg_ripple 特定的 `.pl` 风格文件 |
| **表达力** | 形状内完整的 SPARQL | 递归、魔法集、lattice、良基语义 |
| **性能** | 每个焦点节点运行一次查询 | 每个层编译为单个 SQL `INSERT … SELECT` |
| **递归** | 受限 — 您可以使用属性路径手动递归 | 一类 — 单元求值、不动点 |
| **否定** | SPARQL `FILTER NOT EXISTS` | 分层否定；良基语义 |

经验法则：如果规则适合单个 SHACL 形状，请在 SHACL-SPARQL 中编写。如果它自然递归或需要否定，请使用 Datalog。

---

## 性能说明

- `sh:SPARQLConstraint` 按焦点节点求值。对于目标匹配数百万节点的形状，使用更严格的 `sh:targetClass` 或 `sh:targetSubjectsOf` 预过滤目标。
- `sh:SPARQLRule` 在每次 `shacl_apply_rules()` 调用时重新应用。它*不是*增量的。对于增量推理，请使用 Datalog 引擎。
- 两者都在单个事务中运行；要么全部成功，要么全部不更改。

---

## 另请参阅

- [验证数据质量](validating-data-quality.md) — SHACL Core 约束。
- [推理和推理](reasoning-and-inference.md) — Datalog 替代方案。
- [SHACL Advanced Features (W3C)](https://www.w3.org/TR/shacl-af/)