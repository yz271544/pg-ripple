# SHACL 验证

pg_ripple v0.7.0 添加了 **SHACL Core** — W3C 标准，用于在 RDF 图上表达数据质量规则。规则从 Turtle 加载，存储在数据库中，可以在插入时内联强制执行或按需进行验证。

---

## 快速入门

```sql
-- 1. 从 Turtle 加载 shapes
SELECT pg_ripple.load_shacl($SHACL$
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix ex:  <https://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:PersonShape
    a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [
        sh:path ex:name ;
        sh:minCount 1 ;
        sh:datatype xsd:string ;
    ] ;
    sh:property [
        sh:path ex:email ;
        sh:maxCount 1 ;
    ] .
$SHACL$);

-- 2. 验证默认图
SELECT pg_ripple.validate();

-- 3. 启用内联拒绝违规
SET pg_ripple.shacl_mode = 'sync';
```

---

## 函数

### `load_shacl(data TEXT) → INTEGER`

解析 `data`（Turtle 格式的 SHACL shapes）并将每个 shape 存储在 `_pg_ripple.shacl_shapes` 中。返回已加载的 shape 数量。在 Turtle 语法错误时引发错误，因此不会提交任何部分状态。

**支持的 shape 类型：**
- `sh:NodeShape` — 目标为类或特定节点
- `sh:PropertyShape` — 对谓词路径的约束

**支持的约束（v0.7.0 Core）：**

| 约束 | 描述 |
|---|---|
| `sh:minCount` | 每个焦点节点的最小值节点数 |
| `sh:maxCount` | 每个焦点节点的最大值节点数 |
| `sh:datatype` | 值节点的必需数据类型 IRI |
| `sh:in (...)` | 允许的值集（Turtle 列表）|
| `sh:pattern "regex"` | 对词法形式进行正则表达式匹配 |
| `sh:class` | 值节点的必需 `rdf:type` |
| `sh:node` | 嵌套 shape 引用 — 值节点必须符合引用的 shape |
| `sh:or (...)` | 值/焦点节点必须符合至少一个列出的形状（v0.8.0）|
| `sh:and (...)` | 值/焦点节点必须符合所有列出的形状（v0.8.0）|
| `sh:not` | 值/焦点节点必须**不**符合引用的形状（v0.8.0）|
| `sh:qualifiedValueShape` | 与 `sh:qualifiedMinCount`/`sh:qualifiedMaxCount` 结合使用（v0.8.0）|
| `sh:equals <path>` | 声明路径的值集必须等于 `<path>` 的值集（v0.45.0）|
| `sh:disjoint <path>` | 声明路径的值集必须与 `<path>` 的值集不相交（v0.45.0）|

**支持的目标声明：**

| 声明 | 描述 |
|---|---|
| `sh:targetClass` | 类的所有实例（`rdf:type` 成员）|
| `sh:targetNode` | 一个或多个特定节点 |
| `sh:targetSubjectsOf` | 给定谓词的所有主语 |
| `sh:targetObjectsOf` | 给定谓词的所有对象 |

```sql
-- 返回加载的 shape 数量
SELECT pg_ripple.load_shacl('
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://example.org/> .

ex:ThingShape
    a sh:NodeShape ;
    sh:targetClass ex:Thing ;
    sh:property [
        sh:path ex:name ;
        sh:minCount 1 ;
    ] .
');
```

---

### `validate(graph TEXT DEFAULT NULL) → JSONB`

对所有活动 shape 运行完整的离线 SHACL 验证报告。

| `graph` 值 | 范围 |
|---|---|
| `NULL`（默认）| 默认图（id 0）|
| `''`（空字符串）| 默认图 |
| `'*'` | 所有命名图 |
| `'<https://example.org/g1>'` | 特定命名图 |

**返回值** — 一个具有两个键的 JSONB 对象：

```json
{
  "conforms": false,
  "violations": [
    {
      "focusNode": "https://example.org/alice",
      "shapeIRI":  "https://example.org/PersonShape",
      "path":      "https://example.org/email",
      "constraint": "sh:maxCount",
      "message":   "expected at most 1 value(s) for <https://example.org/email>, found 2",
      "severity":  "Violation"
    }
  ]
}
```

```sql
-- 检查默认图是否符合
SELECT (pg_ripple.validate() ->> 'conforms')::boolean AS ok;

-- 计算违规数量
SELECT jsonb_array_length(pg_ripple.validate() -> 'violations') AS violation_count;

-- 验证命名图
SELECT pg_ripple.validate('<https://example.org/my-graph>');
```

---

### `list_shapes() → TABLE(shape_iri TEXT, active BOOLEAN)`

返回 shape 目录中的所有 shape。

```sql
SELECT * FROM pg_ripple.list_shapes();
```

---

### `drop_shape(shape_uri TEXT) → INTEGER`

按其 IRI 删除 shape。如果找到并删除则返回 1，如果未找到则返回 0。

```sql
SELECT pg_ripple.drop_shape('https://example.org/PersonShape');
```

---

## 验证模式（`pg_ripple.shacl_mode`）

| 模式 | 行为 |
|---|---|
| `off`（默认）| 不强制 SHACL。shape 已存储但未在插入时使用。|
| `sync` | 在 `insert_triple()` 期间内联检测违规。插入将被拒绝并显示错误消息；不写入部分数据。|
| `async` |（v0.8.0）三元组在 `_pg_ripple.validation_queue` 中排队以进行后台验证。违规被移至 `_pg_ripple.dead_letter_queue`。|

```sql
-- 启用内联强制执行
SET pg_ripple.shacl_mode = 'sync';

-- 如果超出 shape 的 sh:maxCount，这将引发错误：
SELECT pg_ripple.insert_triple(
    '<https://example.org/alice>',
    '<https://example.org/email>',
    '"alice3@example.org"'
);
-- ERROR:  SHACL violation: <https://example.org/alice> sh:maxCount 1 for
--         <https://example.org/email>: found 1 existing value(s), limit is 1

-- 恢复默认
RESET pg_ripple.shacl_mode;
```

> **延迟说明**：`sync` 模式在 `shacl_mode = 'sync'` 时为每个 `insert_triple()` 调用执行 per-shape 验证器计划。对于高吞吐量入站，使用 `off`（加载后使用 `validate()` 验证）或配置 `async` 模式（v0.8.0）。

---

## 示例：完整工作流

```sql
-- 1. 加载 shape
SELECT pg_ripple.load_shacl($SHACL$
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix ex:  <https://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:EmployeeShape
    a sh:NodeShape ;
    sh:targetClass ex:Employee ;
    sh:property [
        sh:path ex:employeeId ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:integer ;
    ] ;
    sh:property [
        sh:path ex:department ;
        sh:minCount 1 ;
        sh:in ( ex:Engineering ex:Sales ex:HR ) ;
    ] .
$SHACL$);

-- 2. 加载数据
SELECT pg_ripple.load_ntriples($NQ$
<https://example.org/emp1> <http://www.w3.org/1999/1999/02/22-rdf-syntax-ns#> <https://example.org/Employee> .
<https://example.org/emp1> <https://example.org/employeeId> "42"^^<http://www.w3.org/2001/XMLSchema#integer> .
<https://example.org/emp1> <https://example.org/department> <https://example.org/Engineering> .
$NQ$);

-- 3. 验证
SELECT pg_ripple.validate();
-- {"conforms": true, "violations": []}

-- 4. 确认加载的 shape
SELECT * FROM pg_ripple.list_shapes();
```

---

## 内部表

| 表 | 描述 |
|---|---|
| `_pg_ripple.shacl_shapes` | Shape 目录：`shape_iri`、`shape_json`（JSONB IR）、`active`、时间戳 |
| `_pg_ripple.validation_queue` | 异步验证收件箱（当 `shacl_mode = 'async'` 时填充）|
| `_pg_ripple.dead_letter_queue` | 被异步验证拒绝的三元组，包含违规报告 |

---

## 限制（v0.7.0）

- 不支持直接谓词之外的属性路径（例如 `sh:inversePath`、`sh:alternativePath`）。
- `sh:minCount` 仅由 `validate()` 检查，在 sync 模式下 `insert_triple()` 期间不检查（无法在单个插入上检测缺失）。

---

## 异步验证管道（v0.8.0）

当 `pg_ripple.shacl_mode = 'async'` 时，违规**不会**被内联引发。每个通过 `insert_triple()` 插入的三元组都记录在 `_pg_ripple.validation_queue` 中。后台工作线程（合并工作进程）批量排空队列，对每个三元组运行完整的 SHACL 验证。违反任何活动 shape 的三元组将写入 `_pg_ripple.dead_letter_queue`，其中包含结构化的违规报告。

### 管理函数

| 函数 | 返回值 | 描述 |
|---|---|---|
| `process_validation_queue(batch_size BIGINT DEFAULT 1000)` | `BIGINT` | 处理最多 `batch_size` 项；返回已处理的数量 |
| `validation_queue_length()` | `BIGINT` | 队列中待处理项的数量 |
| `dead_letter_count()` | `BIGINT` | 记录的违规数量 |
| `dead_letter_queue()` | `JSONB` | 所有死信条目作为 JSON 数组 |
| `drain_dead_letter_queue()` | `BIGINT` | 删除所有死信条目；返回已删除的数量 |

```sql
-- 启用异步模式
SET pg_ripple.shacl_mode = 'async';

-- 插入永远不会引发错误 — 违规被排队
SELECT pg_ripple.insert_triple(
    '<https://example.org/thing>',
    '<https://example.org/value>',
    '"wrong-type"'
);

-- 检查队列深度
SELECT pg_ripple.validation_queue_length();

-- 手动排空队列（后台工作线程也会执行此操作）
SELECT pg_ripple.process_validation_queue();

-- 检查违规
SELECT pg_ripple.dead_letter_queue();

-- 审阅后清除
SELECT pg_ripple.drain_dead_letter_queue();
```

### 何时使用异步模式

- **高吞吐量入站**，其中内联 SHACL 开销不可接受
- **批量导入** — 加载后通过调用一次 `process_validation_queue()` 进行验证
- **非阻塞管道** — 违规被异步标记，而不中断写入者

> **注意**：后台工作线程自动处理队列；`process_validation_queue()` 是用于测试和一次性管道的手动覆盖。

---

## 复杂 Shape 约束（v0.8.0）

### `sh:or` — 至少一个 shape 必须匹配

```turtle
ex:AgentShape
    a sh:NodeShape ;
    sh:targetClass ex:Agent ;
    sh:or (ex:PersonShape ex:OrganizationShape) .
```

焦点节点必须符合列出的 shape 中的**至少一个**。在 `validate()` 和 `sync` 模式下都会进行评估。

### `sh:and` — 所有 shape 必须匹配

```turtle
ex:VerifiedPersonShape
    a sh:NodeShape ;
    sh:targetClass ex:VerifiedPerson ;
    sh:and (ex:PersonShape ex:VerifiedShape) .
```

焦点节点必须符合**每个**列出的 shape。

### `sh:not` — shape 必须不匹配

```turtle
ex:ActiveEntityShape
    a sh:NodeShape ;
    sh:targetClass ex:ActiveEntity ;
    sh:not ex:BannedEntityShape .
```

焦点节点**不**得符合引用的 shape。

### `sh:node` — 值节点的嵌套 shape

```turtle
ex:CompanyShape
    a sh:NodeShape ;
    sh:targetClass ex:Company ;
    sh:property [
        sh:path ex:headquarterAddress ;
        sh:node ex:AddressShape ;
    ] .
```

沿路径的每个值节点必须符合 `ex:AddressShape`。递归深度限制为 32 层。

### `sh:qualifiedValueShape` — 限定基数

```turtle
ex:EmployerShape
    a sh:NodeShape ;
    sh:targetClass ex:Employer ;
    sh:property [
        sh:path ex:officeAddress ;
        sh:qualifiedValueShape ex:USAddressShape ;
        sh:qualifiedMinCount 1 ;
        sh:qualifiedMaxCount 3 ;
    ] .
```

沿路径的至少 `sh:qualifiedMinCount`（且最多 `sh:qualifiedMaxCount`）个值节点必须符合限定的 shape。

---

## 限制（v0.8.0）

- 不支持直接谓词之外的属性路径（例如 `sh:inversePath`、`sh:alternativePath`）。
- `sh:minCount` 仅由 `validate()` 检查，在 sync 模式下 `insert_triple()` 期间不检查。
- `sh:qualifiedMinCount` 仅由 `validate()` 在 sync 模式下检查（无法在单个插入上检测缺失）。