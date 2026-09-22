# §2.5 推理和推理

**状态**：自 v0.6.0 起可用（RULE-01）  
**要求**：无外部依赖。规则集由 Datalog 引擎编译为 SQL。  
**SQL**：`pg_ripple.add_rule()`、`pg_ripple.run_inference()`、`pg_ripple.enable_builtin_rules()`  

---

## 是什么和为什么

推理允许 pg_ripple 使用逻辑规则从现有数据派生**新事实**。如果 Alice 在 MIT 工作，而 MIT 位于马萨诸塞州，那么推理可以得出 Alice 位于马萨诸塞州的结论 — 而无需任何人显式插入该三元组。

pg_ripple 提供完整的 Datalog 推理引擎，支持：

- **内置规则集**：开箱即用的 RDFS 和 OWL RL 蕴含。
- **自定义规则**：Turtle 风格的 Datalog 语法中的特定领域推理。
- **分层否定**："标记没有电子邮件地址的人。"
- **聚合**：跨分组三元组模式的 COUNT、SUM、MIN、MAX、AVG。
- **魔法集**：目标定向推理，仅物化相关事实。
- **半朴素求值**：有效的不动点迭代，跳过未更改的行。
- **基于良基语义**：处理具有循环否定的程序（v0.32.0）。

派生的三元组与显式三元组（`source = 0`）一起以 `source = 1`（推断）存储，因此您始终可以区分断言的和派生的事实。

---

## 它是如何工作的

### Datalog 管道

1. **解析** — 规则从 Turtle 风格的 Datalog 语法解析为内部规则 IR。
2. **分层** — 分析依赖图；将规则分组为层，以便否定的谓词在较低的层中完全计算。
3. **编译** — 每个层编译为 PostgreSQL SQL：非递归规则变为 `INSERT ... SELECT`，递归规则变为 `WITH RECURSIVE ... CYCLE`。
4. **执行** — 层自底向上运行；每个层的 SQL 通过 SPI 执行，将派生的三元组插入 VP delta 表中。
6. **不动点** — 递归层迭代直到没有派生新事实（半朴素求值）。

### 规则语法

规则使用类似 Prolog 的符号和 RDF 术语。来自 `register_prefix()` 的前缀注册表可用：

```
head_triple :- body_triple1 , body_triple2 .
```

变量以 `?` 开头。常量是 IRI（前缀或完整）。否定使用 `NOT`。

### 内置规则集

| 名称 | 规则 | 涵盖内容 |
|---|---|---|
| `rdfs` | ~12 条规则 | `rdfs:subClassOf` 传递性、`rdfs:subPropertyOf` 传递性、通过子类/子属性传播 `rdf:type`、`rdfs:domain`/`rdfs:range` 推理 |
| `owl-rl` | ~80 条规则 | OWL RL profile：对称/传递/逆属性、`owl:equivalentClass`、`owl:sameAs`、`owl:unionOf`、`owl:intersectionOf`、属性链等 |

---

## 详细的示例

### 加载内置 RDFS 规则

```sql
-- 加载 RDFS 蕴含规则
SELECT pg_ripple.load_rules_builtin('rdfs');
-- 返回：12（规则数）

-- 加载一些类层次结构数据
SELECT pg_ripple.load_turtle('
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix bibo: <http://purl.org/ontology/bibo/> .
@prefix ex:   <https://example.org/> .

bibo:AcademicArticle rdfs:subClassOf bibo:Article .
bibo:Article rdfs:subClassOf bibo:Document .
bibo:Document rdfs:subClassOf rdfs:Resource .

ex:paper/42 rdf:type bibo:AcademicArticle .
');

-- 运行推理
SELECT pg_ripple.infer('rdfs');

-- 现在 ex:paper/42 也是 bibo:Article、bibo:Document 和 rdfs:Resource
SELECT * FROM pg_ripple.sparql('
PREFIX rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX bibo: <http://purl.org/ontology/bibo/>

SELECT ?type
WHERE {
    <https://example.org/paper/42> rdf:type ?type .
}
');
```

### 加载 OWL RL 规则

```sql
-- 加载 OWL RL 蕴含规则
SELECT pg_ripple.load_rules_builtin('owl-rl');

-- 加载具有 OWL 构造的本体的主体
SELECT pg_ripple.load_turtle('
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix ex:   <https://example.org/> .

ex:cites owl:inverseOf ex:citedBy .
ex:collaboratesWith a owl:SymmetricProperty .
ex:influencedBy a owl:TransitiveProperty .

ex:paper/42 ex:cites ex:paper/99 .
ex:person/alice ex:collaboratesWith ex:person/bob .
ex:person/carol ex:influencedBy ex:person/alice .
ex:person/alice ex:influencedBy ex:person/dave .
');

-- 运行 OWL RL 推理
SELECT pg_ripple.infer('owl-rl');

-- 派生的：ex:paper/99 ex:citedBy ex:paper/42  （逆）
-- 派生的：ex:person/bob ex:collaboratesWith ex:person/alice  （对称）
-- 派生的：ex:person/carol ex:influencedBy ex:person/dave  （传递）
```

### 编写自定义规则

为书目数据集定义特定领域的规则：

```sql
SELECT pg_ripple.load_rules('
# 派生共同作者关系：共同创作同一论文的两位
2 ?a ex:coAuthor ?b :- ?paper dct:creator ?a , ?paper dct:creator ?b .

# 派生机构合作
2 ?inst1 ex:collaboratesWith ?inst2 :-
    ?paper dct:creator ?a ,
    ?paper dct:creator ?b ,
    ?a schema:affiliation ?inst1 ,
    ?b schema:affiliation ?inst2 .

# 派生高产作者（创作了 5+ 篇论文）
# 使用算术保护：至少 5 篇论文
2 ?author ex:isProlific "true"^^xsd:boolean :-
    ?paper1 dct:creator ?author ,
    ?paper2 dct:creator ?author ,
    ?paper3 dct:creator ?author ,
    ?paper4 dct:creator ?author ,
    ?paper5 dct:creator ?author .
', 'biblio');

-- 运行自定义规则集
SELECT pg_ripple.infer('biblio');
```

### 否定失败

标记缺少预期属性的实体：

```sql
SELECT pg_ripple.load_rules('
# 标记没有日期的论文
2 ?paper ex:missingDate "true"^^xsd:boolean :-
    ?paper rdf:type bibo:AcademicArticle ,
    NOT ?paper dct:date ?_ .

# 标记没有附属机构的人
2 ?person ex:missingAffiliation "true"^^xsd:boolean :-
    ?person rdf:type foaf:Person ,
    NOT ?person schema:affiliation ?_ .
', 'quality');

SELECT pg_ripple.infer('quality');

-- 查询派生的质量标记
SELECT * FROM pg_ripple.sparql('
PREFIX ex: <https://example.org/>
SELECT ?paper WHERE { ?paper ex:missingDate "true"^^<http://www.w3.org/2001/XMLSchema#boolean> }
');
```

### 命名图作用域

将派生的三元组写入单独的图：

```sql
SELECT pg_ripple.load_rules('
# 所有 RDFS 推理进入 "inferred" 图
GRAPH ex:graph/inferred { ?x rdf:type ?c } :-
    ?x rdf:type ?b , ?b rdfs:subClassOf ?c .
', 'scoped-rdfs');

SELECT pg_ripple.infer('scoped-rdfs');

-- 仅查询派生的类型
SELECT * FROM pg_ripple.sparql('
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
SELECT ?x ?type WHERE {
    GRAPH <https://example.org/graph/inferred> {
        ?x rdf:type ?type .
    }
}
');
```

### 使用统计信息的半朴素求值

获取详细的推理统计信息：

```sql
SELECT pg_ripple.infer_with_stats('rdfs');
```

返回 JSONB：

```json
{
  "derived": 156,
  "iterations": 4,
  "eliminated_rules": [
    "?x rdf:type rdfs:Resource :- ?x ?p ?o ."
  ]
}
```

`eliminated_rules` 字段显示通过包含检查消除的规则 — 主体是另一个规则主体的超集的规则。

### 使用魔法集进行目标定向推理

当您只需要派生的子集时，魔法集可以避免物化所有内容：

```sql
-- 仅派生与 "paper/42 具有哪些类型？" 相关的事实
SELECT pg_ripple.infer_goal('rdfs', '?x rdf:type <http://xmlns.com/foaf/0.1/Person>');
```

返回 JSONB：

```json
{
  "derived": 12,
  "iterations": 3,
  "matching": 5
}
```

与完整推理进行比较：

```sql
-- 完整物化：派生所有事实
SELECT pg_ripple.infer('rdfs');
-- derived: 156

-- 目标定向：仅派生目标所需的内容
SELECT pg_ripple.infer_goal('rdfs', '?x rdf:type foaf:Person');
-- derived: 12（更少）
```

```admonish tip
魔法集由 GUC `pg_ripple.magic_sets` 控制。当设置为 `false` 时，`infer_goal()` 回退到完整物化并在之后过滤。
```

### 按需过滤推理

对于一次多个目标，使用按需过滤推理：

```sql
SELECT pg_ripple.infer_demand('rdfs', '[
    {"p": "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"},
    {"s": "<https://example.org/paper/42>"}
]'::jsonb);
```

返回：

```json
{
  "derived": 45,
  "iterations": 3,
  "demand_predicates": [
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
  ]
}
```

### 聚合规则（Datalog^agg）

使用聚合函数派生事实：

```sql
SELECT pg_ripple.load_rules('
# 按作者计算论文数
2 ?author ex:paperCount ?count :-
    COUNT(?paper WHERE ?paper dct:creator ?author) = ?count .

# 按论文汇总引用计数
2 ?paper ex:totalCitations ?total :-
    COUNT(?citing WHERE ?citing bibo:cites ?paper) = ?total .
', 'metrics');

-- 使用感知聚合的推理函数
SELECT pg_ripple.infer_agg('metrics');
```

返回：

```json
{
  "derived": 25,
  "aggregate_derived": 25,
  "iterations": 1
}
```

### 基于良基语义（v0.32.0）

对于具有循环否定的程序（标准分层失败的情况）：

```sql
SELECT pg_ripple.load_rules('
# 循环否定：如果节点不是 "out"，则它是 "in"
2 ?x ex:in "true"^^xsd:boolean :- ?x rdf:type ex:Node , NOT ?x ex:out "true"^^xsd:boolean .
2 ?x ex:out "true"^^xsd:boolean :- ?x rdf:type ex:Node , NOT ?x ex:in "true"^^xsd:boolean .
', 'wfs-demo');

-- 标准的 infer() 将失败并显示 "unstratifiable" 错误
-- WFS 优雅地处理它
SELECT pg_ripple.infer_wfs('wfs-demo');
```

返回：

```json
{
  "derived": 6,
  "certain": 0,
  "unknown": 6,
  "iterations": 3,
  "stratifiable": false
}
```

`certainty = 'unknown'` 的事实被报告但**不**物化到 VP 表中。

---

## 常见模式

### 模式：分层推理

按顺序运行规则集 — 首先是基础蕴含，然后是领域规则：

```sql
-- 第 1 层：RDFS 蕴含
SELECT pg_ripple.load_rules_builtin('rdfs');
SELECT pg_ripple.infer('rdfs');

-- 第 2 层：OWL RL（建立在 RDFS 派生的事实之上）
SELECT pg_ripple.load_rules_builtin('owl-rl');
SELECT pg_ripple.infer('owl-rl');

-- 第 3 层：自定义领域规则
SELECT pg_ripple.load_rules('...', 'domain');
SELECT pg_ripple.infer('domain');
```

### 模式：增量重新推理

添加新数据后，重新运行推理。半朴素求值仅派生新事实：

```sql
-- 加载新数据
SELECT pg_ripple.load_turtle('
@prefix ex:  <https://example.org/> .
@prefix bibo: <http://purl.org/ontology/bibo/> .
ex:paper/newOne a bibo:AcademicArticle .
');

-- 重新运行推理 — 仅计算新的派生
SELECT pg_ripple.infer_with_stats('rdfs');
```

### 模式：显式与推断的三元组

所有 VP 表都有一个 `source` 列：`0` = 显式，`1` = 推断。您可以通过 SPARQL 查询此区别或检查完整的三元组存储：

```sql
-- 查找所有推断的类型断言
SELECT * FROM pg_ripple.find_triples(
    NULL,
    '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>',
    NULL
);
```

### 模式：owl:sameAs 规范化

当 `pg_ripple.sameas_reasoning = 'on'`（默认）时，`owl:sameAs` 链接在推理之前被规范化。等效实体的所有提及都折叠为单个规范 ID，减少冗余派生。

```sql
-- 两个 IRI 引用同一实体
SELECT pg_ripple.insert_triple(
    '<https://example.org/person/alice>',
    '<http://www.w3.org/2002/07/owl#sameAs>',
    '<https://other.org/people/a-johnson>'
);

-- 推理后，两个 IRI 被视为相同
SELECT pg_ripple.infer('owl-rl');
```

---

## 性能和权衡

### 完整物化与目标定向

| 策略 | 优点 | 缺点 |
|---|---|---|
| 完整（`infer()`）| 完整；所有派生事实可用 | 可能派生数百万个不需要的事实 |
| 目标定向（`infer_goal()`）| 仅派生相关事实 | 必须指定目标模式 |
| 按需过滤（`infer_demand()`）| 多个目标；部分物化 | 设置稍微复杂 |
| 按需（查询时）| 零物化成本 | 查询较慢 |

### 半朴素求值

半朴素求值跟踪每次迭代中的新事实，并仅将新事实与现有事实连接。这将每次迭代的工作量从 O(n^2) 减少到 O(n * delta)，其中 delta 是每轮迭代的新发现事实数。

### 包含检查

当两个规则具有相同的头部，并且一个规则的主体是另一个规则主体的子集时，包含的规则将被消除。这减少了每次迭代的 SQL 语句数。

### 制表 / 记忆化（v0.32.0）

目标定向推理结果和良基结果缓存在 `_pg_ripple.tabling_cache` 中。缓存条目会在数据更改（插入或删除）时自动失效。

```sql
-- 检查制表缓存统计信息
SELECT * FROM pg_ripple.tabling_stats();
```

### 规则集管理

```sql
-- 列出所有规则及其元数据
SELECT pg_ripple.list_rules();

-- 启用/禁用规则集而不删除它
SELECT pg_ripple.enable_rule_set('rdfs');
SELECT pg_ripple.disable_rule_set('quality');

-- 删除集合中的所有规则
SELECT pg_ripple.drop_rules('quality');
```

---

## 陷阱和调试

### 不可分层的程序

如果您的规则包含循环否定，标准的 `infer()` 将失败：

```
ERROR: unstratifiable rule set — negation cycle detected
DETAIL: ex:in negates ex:out, which depends on ex:in
HINT: remove the negation cycle or use infer_wfs() for well-founded semantics
```

修复：重构规则以消除循环，或使用 `infer_wfs()`。

### 前缀注册

规则使用来自 `register_prefix()` 的前缀注册表。如果未注册前缀，解析器会将其视为解析错误：

```sql
-- 在加载规则之前注册必需的前缀
SELECT pg_ripple.register_prefix('ex', 'https://example.org/');
SELECT pg_ripple.register_prefix('dct', 'http://purl.org/dc/terms/');

-- 现在加载使用这些前缀的规则
SELECT pg_ripple.load_rules('?x ex:rel ?y :- ?x dct:creator ?y .', 'test');
```

```admonish note
内置规则集（`rdfs`、`owl-rl`）会自动注册标准 RDF/RDFS/OWL 前缀。
```

### 检查派生内容

推理后，检查统计信息：

```sql
-- 总共多少三元组？
SELECT pg_ripple.triple_count();

-- 包括推断计数的详细统计信息
SELECT pg_ripple.stats();

-- 检查约束规则的违规
SELECT pg_ripple.check_constraints();
```

### 性能诊断

如果推理很慢：

1. 使用 `infer_with_stats()` 检查迭代次数 — 许多迭代表明深度递归链。
2. 如果只需要子集，请使用目标定向推理（`infer_goal()`）。
3. 检查用于包含的冗余规则（统计输出中的 `eliminated_rules`）。
4. 在推理后运行 `pg_ripple.vacuum()` 以更新规划器统计信息。

```sql
-- 获取推理诊断
SELECT pg_ripple.infer_with_stats('rdfs');

-- 检查规则计划缓存
SELECT * FROM pg_ripple.rule_plan_cache_stats();
```

---

## 后续步骤

- **[§2.4 验证数据质量](../features/validating-data-quality.md)** — SHACL 形状与推理交互；验证派生的事实。
- **[§2.6 导出和共享](../features/exporting-and-sharing.md)** — 导出派生的事实；用于 GraphRAG 的 Datalog 丰富。
- **[§2.3 使用 SPARQL 查询](../features/querying-with-sparql.md)** — 使用 SPARQL 查询显式和派生的事实。

## 进一步阅读

- [博客：PostgreSQL 中的 Datalog](https://github.com/trickle-labs/pg-ripple/blob/main/blog/datalog-inside-postgresql.md) — Datalog 引擎如何工作
- [博客：内置推理规则解释](https://github.com/trickle-labs/pg-ripple/blob/main/blog/builtin-reasoning-rules-explained.md) — RDFS 和 OWL RL 规则集
- [博客：目标定向推理的魔法集](https://github.com/trickle-labs/pg-ripple/blob/main/blog/magic-sets-goal-directed.md) — 按需求值如何加速查询
- [博客：良基语义](https://github.com/trickle-labs/pg-ripple/blob/main/blog/well-founded-semantics.md) — 处理循环规则中的否定