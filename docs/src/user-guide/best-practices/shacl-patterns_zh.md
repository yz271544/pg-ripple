# SHACL 模式

使用 pg_ripple 定义和使用 SHACL 形状的实用模式。

---

## NodeShape 与 PropertyShape

SHACL 定义了两类形状：

| 类型 | 使用场景 |
|---|---|
| **NodeShape** | 适用于一组焦点节点（由 `sh:targetClass`、`sh:targetNode` 等标识） |
| **PropertyShape** | 定义对特定谓词值的约束，嵌套在 NodeShape 内 |

在 pg_ripple 的 Turtle 解析器中，`sh:NodeShape` 带有一个或多个 `sh:property [...]` 块，每个块描述一个内联的 `PropertyShape`：

```turtle
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix ex:  <https://example.org/> .

ex:ProductShape            # ← NodeShape
    a sh:NodeShape ;
    sh:targetClass ex:Product ;
    sh:property [          # ← 内联 PropertyShape
        sh:path ex:sku ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .
```

---

## `sh:datatype` — 强制值类型

使用 `sh:datatype` 要求字面量值为特定的 XSD 数据类型：

```turtle
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix ex:  <https://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:SensorShape
    a sh:NodeShape ;
    sh:targetClass ex:Sensor ;
    sh:property [
        sh:path ex:temperature ;
        sh:datatype xsd:decimal ;
    ] ;
    sh:property [
        sh:path ex:label ;
        sh:datatype xsd:string ;
    ] .
```

插入形状，然后使用正确的数据类型后缀加载数据：

```sql
SELECT pg_ripple.load_ntriples('
<https://example.org/s1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://example.org/Sensor> .
<https://example.org/s1> <https://example.org/temperature> "23.5"^^<http://www.w3.org/2001/XMLSchema#decimal> .
');

SELECT pg_ripple.validate();
-- {"conforms": true, ...}
```

`ex:temperature` 的普通字符串（例如没有 `^^xsd:decimal` 的 `"23.5"`）将导致 `sh:datatype` 违规。

---

## `sh:minCount` 和 `sh:maxCount`

这些是最常见的 SHACL 约束，自然地映射到基数检查。

| 模式 |含义 |
|---|---|
| `sh:minCount 1` | 必填字段 — 每个焦点节点必须至少有一个值 |
| `sh:maxCount 1` | 最多一个值 — 用于函数式属性 |
| `sh:minCount 1 ; sh:maxCount 1` | 恰好一个值 |

```turtle
ex:PersonShape
    a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [
        sh:path ex:fullName ;
        sh:minCount 1 ;       -- 必填
        sh:maxCount 1 ;       -- 每人唯一
        sh:datatype xsd:string ;
    ] ;
    sh:property [
        sh:path ex:phoneNumber ;
        sh:maxCount 3 ;       -- 最多 3 个电话号码
    ] .
```

> **重要提示**：`sh:minCount` 仅由 `pg_ripple.validate()` 检查，而不会在同步模式的 `insert_triple()` 期间强制执行。这是因为缺失值无法从单次插入中检测到——它需要在事后扫描所有焦点节点。
>
> `sh:maxCount` **会** 在同步模式下检查，因为超过最大值可以在每个新值到达时被检测到。

---

## 同步模式：延迟权衡

`pg_ripple.shacl_mode = 'sync'` 在每次 `insert_triple()` 调用时运行 SHACL 验证器计划。这会增加与以下因素成比例的延迟：

1. 活动形状的数量
2. 目标类的选择性（多数情况为少的焦点节点 = 更快）
3. 为 `sh:maxCount` 计算现有值节点的数量

**推荐用于**：低吞吐量、高完整性的工作流（主数据、配置图、相关知识库）。

**不推荐用于**：批量数据摄入、高频事件流，或在插入时应进行后处理而不是拒绝的违规情况。

```sql
-- 对于批量加载：保持 shacl_mode 关闭，加载后再验证：
SET pg_ripple.shacl_mode = 'off';
SELECT pg_ripple.load_turtle($$ ... $$);
SELECT pg_ripple.validate();  -- 事后检查
```

---

## 按需调用 `validate()`

`validate()` 对每个活动形状的所有焦点节点进行全面扫描。使用场景：
- 批量加载后检测导入数据中的任何违规
- 作为计划数据质量检查的一部分
- 在发布命名图之前

```sql
-- 验证特定的命名图
SELECT pg_ripple.validate('<https://example.org/my-data>');

-- 验证所有图
SELECT pg_ripple.validate('*');

-- 仅将违规作为集合提取
SELECT v
FROM jsonb_array_elements(
    pg_ripple.validate() -> 'violations'
) AS v;
```

---

## `sh:in` — 受控词汇表

使用 `sh:in` 将属性限制为一组特定的允许值：

```turtle
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix ex:  <https://example.org/> .

ex:OrderShape
    a sh:NodeShape ;
    sh:targetClass ex:Order ;
    sh:property [
        sh:path ex:status ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:in ( ex:pending ex:confirmed ex:shipped ex:delivered ex:cancelled ) ;
    ] .
```

---

## `sh:pattern` — 正则表达式约束

使用 POSIX 正则表达式验证字符串值：

```turtle
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix ex:  <https://example.org/> .

ex:ContactShape
    a sh:NodeShape ;
    sh:targetClass ex:Contact ;
    sh:property [
        sh:path ex:email ;
        sh:pattern "^[a-zA-Z0-9._%+\\-]+@[a-zA-Z0-9.\\-]+\\.[a-zA-Z]{2,}$" ;
    ] .
```

> **注意**：pg_ripple 使用 PostgreSQL 的 `~` 运算符进行正则表达式匹配，该运算符遵循 POSIX 扩展正则表达式。在 Turtle 字符串字面量中，反斜杠必须加倍。

---

## 管理多个形状

从单独的 Turtle 文档加载形状，每个文档一次调用：

```sql
-- 加载 Person 形状
SELECT pg_ripple.load_shacl(pg_read_file('/etc/shapes/person-shapes.ttl'));

-- 加载 Product 形状
SELECT pg_ripple.load_shacl(pg_read_file('/etc/shapes/product-shapes.ttl'));

-- 列出所有活动形状
SELECT shape_iri, active FROM pg_ripple.list_shapes();

-- 在不删除形状的情况下取消活动状态（手动设置 active=false 或删除它）
SELECT pg_ripple.drop_shape('https://example.org/OldPersonShape');
```

---

## 部署前清单

在生产环境中使用 SHACL 之前：

1. 在批量导入数据**之前**加载所有形状 — 这确保从一开始就捕获违规。
2. 对于现有的大型数据集，加载形状后运行 `SELECT pg_ripple.validate()` 以识别预先存在的违规。
3. 根据吞吐量要求选择 `shacl_mode`：ETL 管道使用 `off`，交互式 / 低容量插入使用 `sync`。
4. 为 `ex:targetClass` 谓词建立索引 — `sh:targetClass` 形状会完全扫描 `rdf:type` 三元组以收集焦点节点。确保 `rdf:type` 具有专用 VP 表（通常在几百个三元组后就会具有专用 VP 表）。

---

## sh:or / sh:and / sh:not 模式（v0.8.0）

### 使用 sh:or 验证多个有效类型

当焦点节点可以是多种有效类型之一时使用 `sh:or`：

```turtle
ex:PartyShape
    a sh:NodeShape ;
    sh:targetClass ex:Party ;
    sh:or (ex:PersonShape ex:OrganizationShape) .
```

焦点节点必须满足所列形状中的**至少一个**。对于联合类型很有用 — 合同方可以是个人或公司，但必须满足至少一个的必填字段。

### 使用 sh:and 要求符合多个形状

当每个焦点节点必须同时满足所有列出的形状时使用 `sh:and`：

```turtle
ex:AuditedEntityShape
    a sh:NodeShape ;
    sh:targetClass ex:AuditedEntity ;
    sh:and (ex:BaseEntityShape ex:ComplianceShape) .
```

### 使用 sh:not 排除禁止的组合

使用 `sh:not` 防止焦点节点符合特定形状：

```turtle
ex:ActiveUserShape
    a sh:NodeShape ;
    sh:targetClass ex:User ;
    sh:not ex:SuspendedUserShape .
```

如果 `ex:User` 同时也是 `ex:SuspendedUser`，`validate()` 将报告 `sh:not` 违规。

### 性能注意事项

`sh:or`、`sh:and` 和 `sh:not` 都涉及递归形状一致性检查。对于具有深度形状层次结构的大型数据集，相对于 `sync` 模式，更倾向于在非高峰时段使用 `validate()`。

---

## 高吞吐量摄入的异步模式（v0.8.0）

以高速插入数据时，使用 `async` 模式以避免在 SHACL 检查上阻塞插入：

```sql
-- 启用异步验证
SET pg_ripple.shacl_mode = 'async';

-- 批量加载（违规排队，不引发）
SELECT pg_ripple.load_ntriples(pg_read_file('/data/large-dataset.nt'));

-- 重置模式
RESET pg_ripple.shacl_mode;

-- 加载后排出队列
SELECT pg_ripple.process_validation_queue(10000);

-- 查看任何违规
SELECT pg_ripple.dead_letter_queue();

-- 修复数据后清除
SELECT pg_ripple.drain_dead_letter_queue();
```

### 读取死信队列

`dead_letter_queue()` 中的每个条目都是一个 JSON 对象。将 ID 解码为 IRI：

```sql
SELECT
    pg_ripple.decode_id((item->>'s_id')::bigint) AS subject,
    pg_ripple.decode_id((item->>'p_id')::bigint) AS predicate,
    pg_ripple.decode_id((item->>'o_id')::bigint) AS object,
    item->'violation'->>'message'                AS violation
FROM jsonb_array_elements(pg_ripple.dead_letter_queue()) AS item;
```

---

## sh:qualifiedValueShape 模式（v0.8.0）

当基数约束应仅计算符合特定形状的值节点时，使用限定值形状：

```turtle
ex:EmployerShape
    a sh:NodeShape ;
    sh:targetClass ex:Employer ;
    sh:property [
        sh:path ex:officeAddress ;
        sh:qualifiedValueShape ex:PrimaryAddressShape ;
        sh:qualifiedMinCount 1 ;
        sh:qualifiedMaxCount 1 ;
    ] .
```

这强制执行：雇主必须恰好有一个作为主要地址（如 `ex:PrimaryAddressShape` 所定义）的办公地址，同时允许沿同一路径有任何数量的其他地址类型。