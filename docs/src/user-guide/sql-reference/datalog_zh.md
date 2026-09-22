# Datalog 推理

pg_ripple 包含一个内置的 Datalog 推理引擎（v0.10.0+），完全在 PostgreSQL 内部运行。规则从 Turtle 风格的语法解析，分层以确定求值顺序，并编译为原生 SQL — 无需外部推理器。

派生的三元组被写回 VP 存储，标记为 `source = 1`，因此显式和推断的三元组始终可以区分。

---

## 快速入门

```sql
-- 加载两个自定义规则
SELECT pg_ripple.load_rules('
  ?x <http://example.org/grandparent> ?z :-
    ?x <http://example.org/parent> ?y ,
    ?y <http://example.org/parent> ?z .
', 'family');

-- 插入一些数据
SELECT pg_ripple.load_ntriples('
<http://example.org/alice> <http://example.org/parent> <http://example.org/bob> .
<http://example.org/bob>   <http://example.org/parent> <http://example.org/carol> .
');

-- 运行推理 — 插入 alice grandparent carol
SELECT pg_ripple.infer('family');
-- 返回：1（一个派生的三元组）

-- 查询结果
SELECT * FROM pg_ripple.sparql('
  SELECT ?gp WHERE {
    <http://example.org/alice> <http://example.org/grandparent> ?gp
  }
');
```

---

## 规则语法

规则使用 Turtle 风格的 Datalog 语法：

```
head :- body .
```

- **变量**写为 `?x`、`?y` 等。
- **IRI** 使用尖括号：`<http://example.org/knows>`
- **前缀化 IRI** 使用 `prefix:local` 形式（如果已注册前缀）
- **字面量**使用带引号的字符串：`"hello"`、`"42"^^<xsd:integer>`
- **体原子**用逗号分隔
- **否定**使用 `NOT`：`NOT ?x <http://example.org/blocked> ?y`

### 示例规则

```
-- knows 的传递闭包
?x <http://example.org/knowsTransitive> ?z :-
  ?x <http://example.org/knows> ?y ,
  ?y <http://example.org/knowsTransitive> ?z .

?x <http://example.org/knowsTransitive> ?y :-
  ?x <http://example.org/knows> ?y .

-- 约束（空头）：每个 Person 必须有姓名
:- ?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/Person> ,
   NOT ?x <http://example.org/name> ?n .
```

---

## load_rules

```sql
pg_ripple.load_rules(rules TEXT, rule_set TEXT DEFAULT 'custom') → BIGINT
```

解析 Turtle 风格的 Datalog 规则，对其进行分层（检查否定循环），将每个层编译为 SQL，并将它们存储在 `_pg_ripple.rules` 中。返回加载的规则数。

```sql
SELECT pg_ripple.load_rules('
  ?x <http://example.org/sibling> ?y :-
    ?x <http://example.org/parent> ?z ,
    ?y <http://example.org/parent> ?z .
', 'family');
-- 返回：1
```

规则按**规则集名称**分组。您可以独立加载多个规则集并分别对每个规则集运行推理。

---

## load_rules_builtin

```sql
pg_ripple.load_rules_builtin(name TEXT) → BIGINT
```

按名称加载预定义规则集。返回加载的规则数。

| 名称 | 规则数 | 描述 |
|------|-------|-------------|
| `'rdfs'` | 13 | 完整 RDFS 蕴含（rdfs2–rdfs12、子类、域、范围） |
| `'owl-rl'` | ~20 | 核心 OWL RL：类层次结构、属性链、逆、对称、传递 |

```sql
-- 加载 RDFS 蕴含规则
SELECT pg_ripple.load_rules_builtin('rdfs');
-- 返回：13

-- 加载 OWL RL 规则
SELECT pg_ripple.load_rules_builtin('owl-rl');
```

### RDFS 蕴含的作用

如果您的数据包含 `rdfs:subClassOf`、`rdfs:domain`、`rdfs:range` 和类似的 RDFS 词汇表，运行 RDFS 推理会物化所有隐含的类成员资格和属性分配。如果 `ex:Student rdfs:subClassOf ex:Person` 且 `ex:alice rdf:type ex:Student`，则派生 `ex:alice rdf:type ex:Person`。

### OWL RL 推理的作用

OWL RL 处理更丰富的本体构造：`owl:inverseOf`（如果 `ex:knows` 是 `ex:knownBy` 的逆，则两个方向都被物化）、`owl:TransitiveProperty`（传递闭包）、`owl:SymmetricProperty`、`owl:propertyChainAxiom` 和类层次结构公理。

---

## infer

```sql
pg_ripple.infer(rule_set TEXT DEFAULT 'custom') → BIGINT
```

运行命名规则集中的所有层，并以 `source = 1` 插入派生三元组。返回插入的新三元组数。可以安全地重复调用 — 重复的三元组将被忽略。

非递归层使用 `INSERT … SELECT … ON CONFLICT DO NOTHING`。递归层使用 `WITH RECURSIVE … CYCLE`（PostgreSQL 18 原生循环检测）。

---

## infer_with_stats

```sql
pg_ripple.infer_with_stats(rule_set TEXT) → JSONB
```

对命名规则集运行半朴素不动点求值，并返回带有派生三元组数和不动点迭代次数的 JSONB 对象（v0.24.0+）。

```sql
SELECT pg_ripple.infer_with_stats('rdfs');
-- 返回：{"derived": 42, "iterations": 3, "eliminated_rules": []}
```

**为什么使用此函数而非 `infer()`？** 对于大型本体，半朴素求值显著更快，因为每个不动点迭代仅根据前一次迭代中派生的*新*三元组（ΔR 增量）重新评估规则，而不是重新扫描整个派生关系。`iterations` 计数器告诉您引擎达到不动点所需的迭代次数 — 由最长派生链限制，而不是数据集大小。

`eliminated_rules` 数组（v0.29.0+）列出了在求值前通过**子句包含检查**删除的任何规则：当 R1 的体是 R2 的体的多重子集时（R2 仅派生 R1 派生的子集），R2 被 R1 包含。删除被包含的规则会减少每次不动点迭代执行的 SQL 语句数。

### 半朴素求值机制

引擎为每个派生关系 `R` 维护一个 *delta 表* `ΔR`，其中仅包含最近一次迭代中派生的行。每次迭代：

1. 对于当前层中的每个规则，根据`body`（其输入关系的增量）重新评估规则体。
2. 使用 `ON CONFLICT DO NOTHING` 将任何新行插入到 `ΔR_new` 中。
3. 处理完所有规则后：`ΔR ← ΔR_new`，然后如果 `ΔR` 非空则继续。

这意味着迭代成本与新衍生的*新前沿*成比例，而不是与关系的总大小成比例。在最长 subClassOf 链深度为 5 的数据集上的 RDFS 闭包上，无论有多少三元组，引擎都会在 5 次迭代内收敛。

保留分层求值顺序：每个层在下一个层开始之前完全收敛。半朴素在*每个层内*应用。

### OWL RL 覆盖范围

内置 `owl-rl` 规则集实现以下 OWL 2 RL 公理：

| OWL RL 规则 | 公理 | 状态 |
|---|---|---|
| rdfs2 | 域推理 | ✅ |
| rdfs3 | 范围推理 | ✅ |
| rdfs4a/4b | 资源成员资格 | ✅ |
| rdfs5 | subPropertyOf 传递性 | ✅ |
| rdfs7 / prp-spo1 | subPropertyOf 传播 | ✅ |
| rdfs9 / cax-sco | subClassOf 类型传播 | ✅ |
| rdfs11 | subClassOf 传递性 | ✅ |
| cls-avf | allValuesFrom 链接 | ✅ |
| prp-ifp | InverseFunctionalProperty | ✅ |
| prp-sym | SymmetricProperty | ✅ |
| prp-trp | TransitiveProperty | ✅ |
| prp-inv1/2 | inverseOf | ✅ |
| prp-fp | FunctionalProperty | ✅ |
| cax-eqc1 | equivalentClass | ✅ |
| prp-eqp1 | equivalentProperty | ✅ |
| prp-chm | propertyChainAxiom（2-link） | ✅ |
| cls-hv1 | hasValue 限制 | ✅ |
| cls-int1 | intersectionOf 成员资格 | ✅ |
| eq-sym | sameAs 对称性 | ✅ |
| eq-trans | sameAs 传递性 | ✅ |
| eq-rep-c | sameAs 类传播 | ✅ |
| owl:onProperty + allValuesFrom | cls-avf 完整形式 | ✅ |

需要可判定枚举（例如 `owl:oneOf`、`cls-oo`）或二阶模式的规则超出了 OWL RL profile，未实现。

---

## infer_goal

```sql
pg_ripple.infer_goal(rule_set TEXT, goal TEXT) → JSONB
```

使用简化的魔集变换运行**目标制导推理**（v0.29.0+）。不是派生每个可能的事实，而是仅物化与指定目标三元组模式相关的事实。返回具有三个字段的 JSONB 对象：

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `derived` | bigint | 推理插入的三元组总数 |
| `iterations` | integer | 不动点迭代次数 |
| `matching` | bigint | 推理后匹配目标模式的三元组 |

```sql
-- 我们能派生多少 rdfs:type triples 到类型 foaf:Person？
SELECT pg_ripple.infer_goal('rdfs', '?x <http://xmlns.com/foaf/0.1/type> <http://xmlns.com/foaf/0.1/Person>');
-- 返回：{"derived": 14, "iterations": 2, "matching": 5}

-- 完全开放的目标（等效于 infer_with_stats，但目标制导机制仍在内部修剪）
SELECT pg_ripple.infer_goal('rdfs', '?x ?p ?y');
```

### 目标语法

目标是三元组模式字符串。变量写为 `?name`。IRI 使用尖括号引号：

- `?x rdf:type ex:Person` — 查找所有人员（与 `x` 前缀形式 — 使用注册前缀映射）
- `?x <http://example.org/knows> ?y` — 所有 knows 三元组
- `<http://example.org/alice> ?p ?o` — 关于 Alice 的所有三元组
- `?x ?p ?y` — 完全开放（所有三元组）

### 魔集策略

对于目标中的每个绑定项，引擎确定哪些规则可以推导与该模式匹配的三元组，并将不动点求值限制为这些规则。魔集临时表（`_magic_{rule_set}_{pred}`）保存请求的绑定集，并在推理结束时自动删除。

将 `pg_ripple.magic_sets = false` 以禁用变换并回退到完全自底向上求值（用于调试）。

---

## check_constraints

```sql
pg_ripple.check_constraints(rule_set TEXT DEFAULT NULL) → JSONB
```

评估所有完整性约束（具有空头的规则）并返回违规作为 JSONB 数组。传递 `NULL` 以检查所有规则集，或传递特定名称以检查一个。

```sql
SELECT pg_ripple.check_constraints();
-- [
--   {"rule_set": "family", "rule_index": 3, "bindings": {"x": "<http://example.org/alice>"}},
--   ...
-- ]
```

空数组表示没有违规。

---

## list_rules

```sql
pg_ripple.list_rules() → JSONB
```

返回所有活动规则作为 JSONB 数组。每个元素包括规则集名称、层、头、体和已编译的 SQL。

```sql
SELECT pg_ripple.list_rules();
```

---

## drop_rules

```sql
pg_ripple.drop_rules(rule_set TEXT) → BIGINT
```

删除命名规则集中的所有规则。返回已删除的规则数。

```sql
SELECT pg_ripple.drop_rules('family');
```

> **注意**：这不会删除已由这些规则派生的三元组。要删除已派生的三元组，请从相关 VP 表中删除 `source = 1` 的行，或在清除后使用 `vacuum_dictionary()`。

---

## enable_rule_set / disable_rule_set

```sql
pg_ripple.enable_rule_set(name TEXT) → VOID
pg_ripple.disable_rule_set(name TEXT) → VOID
```

在活动和非活动之间切换规则集。禁用的规则集被 `infer()` 和 `check_constraints()` 跳过，但保留在目录中。

```sql
-- 临时禁用 OWL RL 推理
SELECT pg_ripple.disable_rule_set('owl-rl');

-- 稍后重新启用
SELECT pg_ripple.enable_rule_set('owl-rl');
```

---

## prewarm_dictionary_hot

```sql
pg_ripple.prewarm_dictionary_hot() → BIGINT
```

将频繁使用的 IRI（≤ 512 字节）加载到 UNLOGGED 热表（`_pg_ripple.dictionary_hot`）中，以便在推理期间进行亚微秒查找。返回加载的行数。

热表在连接池后仍然存在，但不能在数据库重启后存在。当 `pg_ripple.inference_mode != 'off'` 时，它在 `_PG_init` 时自动填充。

```sql
SELECT pg_ripple.prewarm_dictionary_hot();
-- 返回：1024
```

---

## SHACL-AF 桥接

当通过 `load_shacl()` 加载的形状包含 `sh:rule` 属性时，pg_ripple 会检测它们并在 Datalog 规则目录中注册占位符条目。这将 SHACL 高级功能（SHACL-AF）规则定义与 Datalog 引擎桥接起来。

---

## 聚合规则（Datalog^agg、v0.30.0）

pg_ripple v0.30.0 向 Datalog 引擎添加了**聚合字面量**，允许规则派生依赖于三元组存储上计算的聚合（COUNT、SUM、MIN、MAX、AVG）的事实。这直接从推理规则解锁图分析和指标 — 例如，"计算每个人有多少朋友"或"查找每个部门中的最高工资"。

### 聚合规则语法

聚合字面量出现在规则体中，并使用以下形式：

```
FUNC(?aggVar WHERE subject pred object) = ?resultVar
```

其中：
- `FUNC` 是 `COUNT`、`SUM`、`MIN`、`MAX`、`AVG` 之一
- `?aggVar` 是要聚合的变量（必须出现在原子的主语或宾语位置）
- `subject pred object` 是原子模式（每个可以是变量或 IRI 常量）
- `?resultVar` 必须出现在规则头中

**示例 — 计算朋友数：**

```sql
SELECT pg_ripple.load_rules(
  '?x <https://example.org/friendCount> ?n :-
     COUNT(?y WHERE ?x <https://xmlns.com/foaf/0.1/knows> ?y) = ?n .',
  'social'
);

-- 插入数据
SELECT pg_ripple.insert_triple(
  '<https://example.org/Alice>',
  '<https://xmlns.com/foaf/0.1/knows>',
  '<https://example.org/Bob>'
);
SELECT pg_ripple.insert_triple(
  '<https://example.org/Alice>',
  '<https://xmlns.com/foaf/0.1/knows>',
  '<https://example.org/Carol>'
);

-- 运行聚合推理
SELECT pg_ripple.infer_agg('social');
-- 返回：{"derived": 0, "aggregate_derived": 1, "iterations": 0}

-- 查询结果：Alice 有 2 个朋友
SELECT * FROM pg_ripple.find_triples(
  '<https://example.org/Alice>',
  '<https://example.org/friendCount>',
  NULL
);
```

### `pg_ripple.infer_agg(rule_set TEXT DEFAULT 'custom') RETURNS JSONB`

为规则集运行 Datalog^agg 推理。非聚合规则首先通过半朴素不动点求值；聚合规则之后在单个 GROUP BY 过程中求值。

| JSON 键 | 类型 | 描述 |
|----------|------|-------------|
| `derived` | bigint | 派生的三元组总数（非聚合 + 聚合） |
| `aggregate_derived` | bigint | 从聚合规则派生的三元组 |
| `iterations` | int | 非聚合规则的半朴素不动点迭代次数 |

```sql
SELECT pg_ripple.infer_agg('social');
-- {"derived": 1, "aggregate_derived": 1, "iterations": 0}
```

### 聚合分层（PT510）

聚合规则必须**分层**：任何派生的谓词不能出现在规则的体中，该规则聚合的谓词也依赖于该派生的谓词。如果检测到分层违规，pg_ripple 会发出 `WARNING PT510` 并跳过聚合规则（回退到仅安全运行非聚合规则）。

```sql
-- 此规则对通过聚合创建循环 — 将发出 PT510：
SELECT pg_ripple.load_rules(
  '?x <ex:a> ?y :- ?x <ex:b> ?y .', 'bad');
SELECT pg_ripple.load_rules(
  '?x <ex:b> ?n :- COUNT(?y WHERE ?x <ex:a> ?y) = ?n .', 'bad');

SELECT pg_ripple.infer_agg('bad');
-- WARNING:  infer_agg: aggregation stratification violation (PT510): …
```

---

## 规则计划缓存（v0.30.0）

每个规则集的已编译 SQL 缓存在进程本地的 LRU 中，以便对同一规则集的重复 `infer()` / `infer_agg()` 调用跳过解析 + 编译步骤。

当为该规则集调用 `load_rules()` 或 `drop_rules()` 时，缓存会自动失效。

### `pg_ripple.rule_plan_cache_stats() RETURNS TABLE(...)`

返回计划缓存的统计信息。

| 列 | 类型 | 描述 |
|--------|------|-------------|
| `rule_set` | text | 规则集的名称 |
| `hits` | bigint | 使用缓存 SQL 的次数 |
| `misses` | bigint | 缓存未命中数（SQL 是从头编译的） |
| `entries` | int | 缓存中当前的规则集总数 |

---

## 实体解析与需求变换（v0.31.0）

### `owl:sameAs` 实体规范化

当 `pg_ripple.sameas_reasoning = on`（默认）时，推理引擎自动处理 `owl:sameAs` 三元组。在每个不动点迭代之前，它计算所有 `owl:sameAs` 三元组的等价类，然后将规则体常量重写为它们的规范（最低字典 ID）代表。

这意味着如果您的知识图谱包含：
```sparql
ex:Alice owl:sameAs ex:A.Smith .
ex:Alice ex:name "Alice" .
```
那么将为规范 `ex:Alice` 实体产生原本为 `ex:A.Smith` 派生新事实的规则（例如，从匹配 `ex:name` 的模式）。引用 `ex:A.Smith` 的 SPARQL 查询会被透明地重定向到 `ex:Alice`。

**GUC：** `pg_ripple.sameas_reasoning`（bool，默认 `true`）— 设置为 `false` 以禁用预处理。

### `pg_ripple.infer_demand(rule_set TEXT DEFAULT 'custom', demands JSONB) RETURNS JSONB`

受限于需要派生指定需求的规则子集的目标制导推理。这是 `infer_goal()`（单目标、单谓词）的推广，支持同时多个目标模式。

`demands` 是目标模式的 JSONB 数组。每个元素都是一个带有可选 `"s"`、`"p"`、`"o"` 键的对象：

```sql
SELECT pg_ripple.infer_demand('rdfs', '[{"p": "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"}]');
-- 仅派生回答 rdf:type 查询所需的三元组。
```

当 `demands` 是空数组（`'[]'`）时，运行与 `infer()` 完全相同的完全推理。

**返回值**（JSONB）：

| 键 | 类型 | 描述 |
|-----|------|-------------|
| `derived` | number | 派生的三元组总数 |
| `iterations` | number | 不动点迭代计数 |
| `demand_predicates` | array | 用作需求种子的谓词引用（已解码） |

```sql
-- 仅派生 "descendantOf" 及其依赖项，忽略不相关的规则。
SELECT pg_ripple.infer_demand('hierarchy', '[{"p": "<https://ex.org/descendantOf>"}]');
-- {"derived": 5, "iterations": 2, "demand_predicates": ["https://ex.org/descendantOf"]}
```

**GUC：** `pg_ripple.demand_transform`（bool，默认 `true`）— 当为 `true` 时，`create_datalog_view()` 在指定多个目标时自动应用需求变换。

---

## 配置

| GUC | 默认值 | 描述 |
|-----|---------|-------------|
| `pg_ripple.inference_mode` | `'on_demand'` | `'off'` 完全禁用引擎；`'on_demand'` 在调用 `infer()` 时通过 CTE 求值；`'materialized'` 使用 pg_trickle 流表进行自动刷新 |
| `pg_ripple.enforce_constraints` | `'warn'` | `'off'` 静默约束违规；`'warn'` 记录它们；`'error'` 引发异常 |
| `pg_ripple.rule_graph_scope` | `'default'` | `'default'` 仅将规则应用于默认图；`'all'` 跨所有命名图应用规则 |
| `pg_ripple.magic_sets` | `true` | 目标制导魔集推理的核心开关（v0.29.0+） |
| `pg_ripple.datalog_cost_reorder` | `true` | 在编译时按 VP 表基数对 Datalog 体原子进行排序（v0.29.0+） |
| `pg_ripple.datalog_antijoin_threshold` | `1000` | 用于否定的 `LEFT JOIN … IS NULL` 反连接形式的最小 VP 表行数（v0.29.0+） |
| `pg_ripple.delta_index_threshold` | `500` | 在 `(s, o)` 上创建 B-tree 索引之前的最小 delta 表行数（v0.29.0+） |
| `pg_ripple.rule_plan_cache` | `true` | Datalog 规则计划缓存的核心开关（v0.30.0+） |
| `pg_ripple.rule_plan_cache_size` | `64` | 保存在计划缓存中的最大规则集数；溢出时最旧的条目被淘汰（v0.30.0+） |
| `pg_ripple.sameas_reasoning` | `true` | 在每次推理运行之前启用 `owl:sameAs` 规范化预处理（v0.31.0+） |
| `pg_ripple.demand_transform` | `true` | 在具有多个目标的 `create_datalog_view()` 中自动应用需求变换（v0.31.0+） |

```sql
-- 启用严格约束验证
SET pg_ripple.enforce_constraints = 'error';

-- 跨所有图应用规则
SET pg_ripple.rule_graph_scope = 'all';

-- 禁用魔集以调试目标制导推理
SET pg_ripple.magic_sets = false;

-- 为所有否定原子强制反连接形式（即使是小表）
SET pg_ripple.datalog_antijoin_threshold = 1;

-- 禁用规则计划缓存（用于测试）
SET pg_ripple.rule_plan_cache = false;

-- 设置较小的计划缓存（在具有许多规则集的服务器上节省内存）
SET pg_ripple.rule_plan_cache_size = 16;
```

---

## 内部表

| 表 | 描述 |
|-------|-------------|
| `_pg_ripple.rules` | 存储每个已解析的规则及其集合名称、层、头、体和已编译的 SQL |
| `_pg_ripple.rule_sets` | 跟踪命名规则集及其活动/非活动标志 |
| `_pg_ripple.dictionary_hot` | 用于频繁使用的 IRI 的 UNLOGGED 热缓存 |

派生的三元组存储在与显式三元组相同的 VP 表中，通过 `source` 列区分：`0` = 显式，`1` = 派生。

---

## 良基语义（v0.32.0）

对于具有**循环否定**（通过 `NOT` 相互递归的规则）的程序，分层失败且无法使用 `infer()`。`infer_wfs()` 使用交替不动点算法（Van Gelder 等人，1991）处理这些程序，该算法为既不能证明为真也不能证明为假的事实分配第三个真值 — *未知*。

```sql
-- 加载不可分层的规则集（相互否定）
SELECT pg_ripple.load_rules('
  ?x <https://ex.org/trusted>   <https://ex.org/yes> :-
    ?x <https://ex.org/person>    <https://ex.org/yes> ,
    NOT ?x <https://ex.org/untrusted> <https://ex.org/yes> .
  ?x <https://ex.org/untrusted> <https://ex.org/yes> :-
    ?x <https://ex.org/person>    <https://ex.org/yes> ,
    NOT ?x <https://ex.org/trusted>   <https://ex.org/yes> .
', 'trust');

SELECT pg_ripple.infer_wfs('trust');
-- 返回：{"certain": 0, "unknown": 2, "derived": 2, "iterations": 2, "stratifiable": false}
```

对于**可分层**的程序，`infer_wfs()` 回退到 `run_inference_seminaive`，并返回 `"stratifiable": true` 和 `"unknown": 0`。

### `pg_ripple.infer_wfs(rule_set TEXT DEFAULT 'custom') RETURNS JSONB`

| 键 | 类型 | 描述 |
|-----|------|-------------|
| `certain` | integer | 确定地确定的事实（真） |
| `unknown` | integer | 既不真也不假的事实 |
| `derived` | integer | 派生事实总数（`certain + unknown`） |
| `iterations` | integer | 交替不动点轮数 |
| `stratifiable` | boolean | 程序是否可分层（快速路径） |

### 表缓存集成

`infer_wfs()` 结果在 `_pg_ripple.tabling_cache` 中自动缓存（当 `pg_ripple.tabling = on` 时）。对同一规则集的重复调用返回缓存结果，无需重新运行不动点。缓存在 `insert_triple()`、`delete_triple()`、`drop_rules()` 和 `load_rules()` 时失效。

---

## 表操作 / 记忆化（v0.32.0）

表操作缓存存储 `infer_wfs()` 调用的结果，键是目标字符串的 XXH3-64 哈希，因此重复的相同调用在 TTL 内支付零计算成本。

### `pg_ripple.tabling_stats() RETURNS TABLE(...)`

```sql
SELECT * FROM pg_ripple.tabling_stats();
-- goal_hash | hits | computed_ms | cached_at
```

| 列 | 类型 | 描述 |
|--------|------|-------------|
| `goal_hash` | `BIGINT` | 缓存目标键的 XXH3-64 哈希 |
| `hits` | `BIGINT` | 此条目的命中计数 |
| `computed_ms` | `FLOAT8` | 原始计算的挂钟时间（毫秒） |
| `cached_at` | `TEXT` | 条目写入的 ISO-8601 时间戳 |

### 表操作 GUC

| GUC | 类型 | 默认值 | 描述 |
|-----|------|---------|-------------|
| `pg_ripple.tabling` | bool | `true` | 启用/禁用表缓存 |
| `pg_ripple.tabling_ttl` | integer | `300` | 缓存 TTL（秒）；`0` = 永不过期 |
| `pg_ripple.wfs_max_iterations` | integer | `100` | WFS 不动点轮次的安全上限；如果超过则发出 WARNING PT520 |

### 良基语义限制（v0.45.0）

当程序包含阻止交替不动点收敛的计数否定时，`infer_wfs()` 在 `pg_ripple.wfs_max_iterations` 轮后停止，并返回带有 `"stratifiable": false` 的**部分结果**。带有代码 **PT520** 的 PostgreSQL WARNING 被发送到客户端。

```sql
SET pg_ripple.wfs_max_iterations = 10;
SELECT pg_ripple.infer_wfs('cyclic_rules');
-- 返回：{"certain": 3, "unknown": 6, "derived": 9, "iterations": 10, "stratifiable": false}
-- WARNING:  PT520: WFS fixpoint cap reached (10 iterations)
```

**检测上限**：检查结果中的 `"stratifiable": false` 和 `"iterations" = pg_ripple.wfs_max_iterations`。

**恢复**：增加上限，或重新构建程序以打破否定循环。对于某些原子真正不确定的程序，部分结果是有效的 — `"unknown"` 桶中的原子是三值 `unknown`。

---

## 有界深度终止与增量撤销（v0.34.0）

### `pg_ripple.datalog_max_depth` GUC

| GUC | 类型 | 默认值 | 描述 |
|-----|------|---------|-------------|
| `pg_ripple.datalog_max_depth` | integer | `0` | 递归规则的最大派生深度；`0` = 无限制 |

当设置为正整数 `d` 时，由 `load_rules()` 编译的任何递归 Datalog 规则都会发出带有 `(s, o, g, depth)` 的 `WITH RECURSIVE` CTE，添加深度计数器列。基本情况从 `depth = 0` 开始；每个递归步骤递增 `depth`，当 `depth >= d` 时递归终止。非递归规则不受影响。

```sql
-- 将推理限制为 3 跳
SET pg_ripple.datalog_max_depth = 3;
SELECT pg_ripple.load_rules('...', 'my_rules');
SELECT pg_ripple.infer('my_rules');
-- 重置为无限制
SET pg_ripple.datalog_max_depth = 0;
```

### `pg_ripple.add_rule(rule_set TEXT, rule_text TEXT) RETURNS BIGINT`

将单个规则添加到现有规则集，而不丢弃先前的物化。返回新规则的 ID。

```sql
SELECT pg_ripple.add_rule(
  'my_rules',
  '?x <https://ex.org/knows2> ?z :- ?x <https://ex.org/knows> ?y , ?y <https://ex.org/knows> ?z .'
);
```

插入后，仅为受影响的层触发一个额外的半朴素遍历。所有先前派生的事实都将保留。

### `pg_ripple.remove_rule(rule_id BIGINT) RETURNS BIGINT`

仅通过其 ID 删除规则（由 `list_rules()` 或 `add_rule()` 返回）。返回已删除规则的 ID，如果没有匹配的规则，返回 `0`。

```sql
SELECT pg_ripple.remove_rule(42);
```

仅由已删除规则支持的事实通过 DRed 算法撤销（见下文），前提是 `pg_ripple.dred_enabled = true`。由其他派生路径支持的事实将被保留。

### `pg_ripple.dred_on_delete(pred_id BIGINT, s BIGINT, o BIGINT, g BIGINT) RETURNS BIGINT`

- 撤销的派生三元组数

| 参数 | 描述 |
|-----------|-------------|
| `pred_id` | 已编码的谓词 ID（来自字典） |
| `s` | 已编码的主语 ID |
| `o` | 已编码的宾语 ID |
| `g` | 已编码的图 ID |

### DRed GUC

| GUC | 类型 | 默认值 | 描述 |
|-----|------|---------|-------------|
| `pg_ripple.dred_enabled` | bool | `true` | 启用 DRed 增量撤销；`false` 回退到完全重算 |
| `pg_ripple.dred_batch_size` | integer | `1000` | 在单个 DRed 事务中处理的最大基础三元组数 |

---

## 并行层求值（v0.35.0）

### 并行求值 GUC

在单个层内，派生不同谓词且没有共享体依赖的规则可以独立执行。pg_ripple 分析此依赖图并将规则划分为组。`infer_with_stats()` 函数报告检测到的组数。

| GUC | 类型 | 默认值 | 描述 |
|-----|------|---------|-------------|
| `pg_ripple.datalog_parallel_workers` | integer | `4` | 层求值的最大并行 worker 数；`1` = 串行 |
| `pg_ripple.datalog_parallel_threshold` | integer | `10000` | 在应用并行分析之前的最小估计总行数 |

```sql
-- 为大型 OWL RL 闭包启用最大并行度。
SET pg_ripple.datalog_parallel_workers = 8;
SET pg_ripple.datalog_parallel_threshold = 0;
SELECT pg_ripple.infer_with_stats('owl-rl');
```

### `infer_with_stats()` 并行字段

`infer_with_stats()` 函数现在在其 JSONB 输出中包含两个附加字段（v0.35.0）：

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `parallel_groups` | integer | 规则集中检测到的独立规则组数 |
| `max_concurrent` | integer | 有效 worker 数：`min(parallel_groups, datalog_parallel_workers)` |

```sql
SELECT pg_ripple.infer_with_stats('owl-rl');
-- {
--   "derived": 1240,
--   "iterations": 4,
--   "eliminated_rules": [],
--   "parallel_groups": 3,
--   "max_concurrent": 3
-- }
```

`parallel_groups` 值为 1 表示所有规则形成单个依赖链，不可能并行。值 > 1 表示存在独立组。

---

## 基于格的 Datalog — Datalog^L（v0.36.0）

标准 Datalog^agg 要求聚合分层 — 聚合只能出现在严格高于其输入的层中。**基于格的 Datalog** 通过仅要求聚合相对于用户指定的偏序（一个格）是*单调*的来放宽此要求。

### 什么是格？

*格* 是代数结构 (L, ⊔)，其中：
- L 是值域（数字、集合、区间等）
- ⊔ 是*连接*运算 — 传递、结合、幂等
- 存在*底*元素 ⊥ 使得 ⊥ ⊔ x = x 对所有 x 成立

当*上升链条件*成立时（没有无限严格递增的链），格上的不动点计算保证终止。

### 内置格

| 名称 | 连接 | 基础 | 典型用途 |
|------|------|--------|-------------|
| `min` | MIN | +∞（i64::MAX） | 信任传播，最短路径 |
| `max` | MAX | −∞（i64::MIN） | 最长路径，可达性 |
| `set` | UNION | {}（空集） | 集合值注释 |
| `interval` | 区间凸包 | 空区间 | 时间/数值范围 |

### 新 GUC 参数（v0.36.0）

| GUC | 默认值 | 描述 |
|-----|---------|-------------|
| `pg_ripple.lattice_max_iterations` | `1000` | PT540 警告前的最大不动点迭代次数 |

### 函数

#### `pg_ripple.create_lattice(name, join_fn, bottom)` → `boolean`

注册用户定义的格。

- `name` — 唯一的格标识符
- `join_fn` — PostgreSQL 聚合函数（必须传递且结合）
- `bottom` — 基础元素（文本字符串）

返回 `true` 如果是新注册的，`false` 如果已存在。

```sql
-- 信任分数在 [0, 100] 范围内 — min-lattice（更低更受信任）。
SELECT pg_ripple.create_lattice('trust', 'min', '100');
```

#### `pg_ripple.list_lattices()` → `jsonb`

列出所有已注册的格类型。

```sql
SELECT jsonb_pretty(pg_ripple.list_lattices());
```

示例输出：

```json
[
  {"name": "min",   "join_fn": "min",       "bottom": "9223372036854775807", "builtin": true},
  {"name": "max",   "join_fn": "max",       "bottom": "-9223372036854775808", "builtin": true},
  {"name": "set",   "join_fn": "array_agg", "bottom": "{}", "builtin": true},
  {"name": "trust", "join_fn": "min",       "bottom": "100", "builtin": false}
]
```

#### `pg_ripple.infer_lattice(rule_set, lattice_name)` → `jsonb`

使用指定的格对所有规则运行基于格的 Datalog 不动点。

```sql
-- 使用 MinLattice 运行信任传播。
SELECT pg_ripple.infer_lattice('trust_rules', 'min');
-- {
--   "derived": 42,
--   "iterations": 5,
--   "lattice": "min",
--   "rule_set": "trust_rules"
-- }
```

返回带有字段的 JSONB：
- `derived` — 写入的新格值数
- `iterations` — 执行的不动点迭代次数
- `lattice` — 使用的格类型
- `rule_set` — 求值的规则集

### 信任传播示例

```sql
-- 1. 为信任分数注册 MinLattice。
SELECT pg_ripple.create_lattice('trust', 'min', '100');

-- 2. 插入直接信任边（分数 0–100，更低 = 更受信任）。
SELECT pg_ripple.load_ntriples('
  <https://example.org/alice> <https://example.org/directTrust> "80"^^<xsd:integer> .
  <https://example.org/bob>   <https://example.org/directTrust> "70"^^<xsd:integer> .
  <https://example.org/alice> <https://example.org/knows>       <https://example.org/bob> .
');

-- 3. 运行格不动点以传播传递信任。
SELECT pg_ripple.infer_lattice('trust_rules', 'trust');

-- 4. 查询传播的信任值。
SELECT * FROM pg_ripple.sparql('
  SELECT ?x ?t WHERE { ?x <https://example.org/directTrust> ?t }
');
```

### 错误代码 PT540

当超过 `pg_ripple.lattice_max_iterations` 时，引擎会发出 WARNING：

```
WARNING:  PT540: lattice fixpoint did not converge after 1000 iterations; returning partial results.
```

如果看到此警告，请执行以下操作之一：
- 增加 `pg_ripple.lattice_max_iterations`，或
- 验证您的格连接函数确实是单调的，且值域是有限的。