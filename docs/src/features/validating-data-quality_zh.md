# §2.4 验证数据质量

**状态**：自 v0.3.0 起可用（SHACL-01）  
**要求**：无外部依赖。  
**SQL**：`pg_ripple.load_shacl_shapes()`、`pg_ripple.validate_shapes()`、`pg_ripple.list_shacl_violations()`  

---

## 是什么和为什么

存储知识只是成功的一半 — 您还需要确保它是**正确的**。SHACL（Shapes Constraint Language）是用于声明和验证 RDF 数据上约束的 W3C 标准。它回答以下问题：

- 每篇论文是否至少有一位作者？
- 所有电子邮件地址在语法上是否有效？
- 每个人都有恰好一个姓名吗？
- 日期值格式是否正确？

pg_ripple 将 SHACL 验证直接集成到数据库引擎中。您可以按需验证、在每次插入时同步强制执行约束，或将三元组排队进行异步后台验证，违规被路由到死信队列。

```admonish note
SHACL 对于 RDF 就像 CHECK 约束和触发器对于关系数据库 — 但 SHACL 形状是声明性的、可组合的，并且跨所有 RDF 系统标准化。
```

---

## 它是如何工作的

### SHACL 模型

SHACL **形状**对一组**焦点节点**（匹配目标模式的实体）声明约束。每个形状包含一个或多个**属性形状**，它们约束特定谓词的值。

```
NodeShape（目标：bibo:AcademicArticle 的实例）
  └─ PropertyShape（路径：dct:title）
       ├─ sh:minCount 1     ← 每篇论文必须至少有一个标题
       ├─ sh:maxCount 1     ← 最多一个标题
       └─ sh:datatype xsd:string  ← 标题必须是字符串
```

### 验证模式

| 模式 | GUC 设置 | 行为 |
|---|---|---|
| **Off**（默认）| `pg_ripple.shacl_mode = 'off'` | 无自动验证 |
| **Sync** | `pg_ripple.shacl_mode = 'sync'` | 每个 `insert_triple()` 在提交前都被验证；违规会引发 ERROR |
| **Async** | `pg_ripple.shacl_mode = 'async'` | 三元组立即插入；后台工作线程验证并将违规路由到死信队列 |

### 支持的约束

| 约束 | 描述 |
|---|---|
| `sh:minCount` | 值的最小数量 |
| `sh:maxCount` | 值的最大数量 |
| `sh:datatype` | 值必须具有特定的 XSD 数据类型 |
| `sh:class` | 值必须是某个类的实例 |
| `sh:in` | 值必须来自枚举集 |
| `sh:pattern` | 值必须匹配正则表达式 |
| `sh:node` | 值必须符合另一个形状 |
| `sh:or` | 值必须至少符合多个形状之一 |
| `sh:and` | 值必须符合所有列出的形状 |
| `sh:not` | 值必须不符合某个形状 |
| `sh:qualifiedValueShape` | 限定基数约束 |
| `sh:hasValue` | 至少一个值必须等于给定项 |
| `sh:nodeKind` | 值必须是 IRI、空节点或字面量 |
| `sh:languageIn` | 语言标签必须在允许列表中 |
| `sh:uniqueLang` | 没有重复的语言标签 |
| `sh:lessThan` / `sh:greaterThan` | 属性之间的比较约束 |
| `sh:closed` | 拒绝未知谓词 |

---

## 详细的示例

### 加载简单形状

为书目数据集定义形状：

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:     <http://www.w3.org/ns/shacl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:     <https://example.org/> .
@prefix dct:    <http://purl.org/dc/terms/> .
@prefix bibo:   <http://purl.org/ontology/bibo/> .
@prefix foaf:   <http://xmlns.com/foaf/0.1/> .
@prefix schema: <https://schema.org/> .

ex:PaperShape a sh:NodeShape ;
    sh:targetClass bibo:AcademicArticle ;
    sh:property [
        sh:path dct:title ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
    ] ;
    sh:property [
        sh:path dct:creator ;
        sh:minCount 1 ;
        sh:class foaf:Person ;
    ] .

ex:PersonShape a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    sh:property [
        sh:path foaf:name ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
    ] ;
    sh:property [
        sh:path schema:affiliation ;
        sh:maxCount 1 ;
        sh:nodeKind sh:IRI ;
    ] .
');
-- 返回：2（加载的形状数）
```

### 运行验证

针对所有活动形状验证默认图：

```sql
SELECT pg_ripple.validate();
```

结果是 JSONB 验证报告：

```json
{
  "conforms": false,
  "violations": [
    {
      "focusNode": "<https://example.org/paper/99>",
      "shapeIRI": "<https://example.org/PaperShape>",
      "path": "<http://purl.org/dc/terms/creator>",
      "constraint": "sh:class",
      "message": "value <https://example.org/person/carol> is not an instance of <http://xmlns.com/foaf/0.1/Person>",
      "severity": "sh:Violation"
    }
  ]
}
```

验证特定的命名图：

```sql
SELECT pg_ripple.validate('https://example.org/graph/pubmed');
```

一次验证所有图：

```sql
SELECT pg_ripple.validate('*');
```

### 同步验证

启用同步模式，以便在插入时拒绝无效三元组：

```sql
SET pg_ripple.shacl_mode = 'sync';

-- 这会成功（论文有标题）
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/700>',
    '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>',
    '<http://purl.org/ontology/bibo/AcademicArticle>'
);

-- 如果形状要求 dct:title 但论文在相同事务中还没有它
-- 这将失败（同步验证检查每个三元组，而不是事务性的）
```

```admonish warning
同步验证为每个 `insert_triple()` 调用增加开销。将其用于低量、高完整性的场景。对于批量加载，应使用 `'off'` 模式并在加载后验证。
```

### 带有死信队列的异步验证

为高吞吐量管道启用异步模式：

```sql
SET pg_ripple.shacl_mode = 'async';

-- 三元组立即插入；验证在后台进行
SELECT pg_ripple.insert_triple(
    '<https://example.org/paper/800>',
    '<http://purl.org/dc/terms/title>',
    '"A New Paper"'
);

-- 检查验证队列长度
SELECT pg_ripple.validation_queue_length();

-- 手动处理队列（通常由后台工作线程处理）
SELECT pg_ripple.process_validation_queue(1000);

-- 检查违规
SELECT pg_ripple.dead_letter_count();

-- 查看完整的死信队列
SELECT pg_ripple.dead_letter_queue();
```

### 复杂形状

**析取约束（sh:or）**：

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:   <https://example.org/> .
@prefix dct:  <http://purl.org/dc/terms/> .

ex:DateShape a sh:NodeShape ;
    sh:targetSubjectsOf dct:date ;
    sh:property [
        sh:path dct:date ;
        sh:or (
            [ sh:datatype xsd:date ]
            [ sh:datatype xsd:gYear ]
            [ sh:datatype xsd:dateTime ]
        ) ;
    ] .
');
```

**封闭形状（拒绝未知谓词）**：

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:     <http://www.w3.org/ns/shacl#> .
@prefix ex:     <https://example.org/> .
@prefix dct:    <http://purl.org/dc/terms/> .
@prefix bibo:   <http://purl.org/ontology/bibo/> .
@prefix schema: <https://schema.org/> .

ex:StrictPaperShape a sh:NodeShape ;
    sh:targetClass bibo:AcademicArticle ;
    sh:closed true ;
    sh:ignoredProperties (
        <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
    ) ;
    sh:property [
        sh:path dct:title ;
        sh:minCount 1 ;
    ] ;
    sh:property [
        sh:path dct:creator ;
        sh:minCount 1 ;
    ] ;
    sh:property [
        sh:path dct:date ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path schema:keywords ;
    ] ;
    sh:property [
        sh:path bibo:cites ;
    ] ;
    sh:property [
        sh:path bibo:citedBy ;
    ] .
');
```

**限定基数**：

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:     <http://www.w3.org/ns/shacl#> .
@prefix ex:     <https://example.org/> .
@prefix dct:    <http://purl.org/dc/terms/> .
@prefix bibo:   <http://purl.org/ontology/bibo/> .
@prefix foaf:   <http://xmlns.com/foaf/0.1/> .

ex:CollabPaperShape a sh:NodeShape ;
    sh:targetClass bibo:AcademicArticle ;
    sh:property [
        sh:path dct:creator ;
        sh:qualifiedValueShape [
            sh:class foaf:Person ;
        ] ;
        sh:qualifiedMinCount 2 ;
    ] .
');
```

### 管理形状

```sql
-- 列出所有加载的形状
SELECT * FROM pg_ripple.list_shapes();

-- 停用形状而不删除它
SELECT pg_ripple.disable_rule_set('custom');

-- 完全删除形状
SELECT pg_ripple.drop_shape('https://example.org/StrictPaperShape');
```

### SHACL DAG 监视器

对于实时违规检测，请启用 DAG 监视器（需要 pg_trickle）：

```sql
-- 首先加载形状
SELECT pg_ripple.load_shacl('...');

-- 启用每形状违规流表
SELECT pg_ripple.enable_shacl_dag_monitors();

-- 查看实时违规摘要
SELECT * FROM _pg_ripple.violation_summary_dag;

-- 列出活动的监视器
SELECT * FROM pg_ripple.list_shacl_dag_monitors();

-- 不再需要时禁用
SELECT pg_ripple.disable_shacl_dag_monitors();
```

---

## 常见模式

### 模式：批量加载后验证

最常见的工作流 — 先加载，后验证：

```sql
-- 在加载期间关闭验证
SET pg_ripple.shacl_mode = 'off';

-- 加载数据
SELECT pg_ripple.load_turtle_file('/data/papers.ttl');

-- 加载形状
SELECT pg_ripple.load_shacl('...');

-- 验证
SELECT pg_ripple.validate();
```

### 模式：数据质量仪表板

将死信队列用作数据质量监视器：

```sql
-- 启用异步验证
SET pg_ripple.shacl_mode = 'async';

-- 定期检查违规计数
SELECT pg_ripple.dead_letter_count();

-- 获取违规详情
SELECT pg_ripple.dead_letter_queue();

-- 使用 pg_trickle：启用违规摘要流表
SELECT pg_ripple.enable_shacl_monitors();
SELECT * FROM _pg_ripple.violation_summary;
```

### 模式：嵌入完整性检查

确保所有实体都具有向量嵌入（请参阅 [§2.7](../features/graphrag.md)）：

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:   <https://example.org/> .
@prefix pg:   <urn:pg_ripple:> .
@prefix bibo: <http://purl.org/ontology/bibo/> .

ex:EmbeddingCompletenessShape a sh:NodeShape ;
    sh:targetClass bibo:AcademicArticle ;
    sh:property [
        sh:path pg:hasEmbedding ;
        sh:minCount 1 ;
        sh:hasValue "true"^^xsd:boolean ;
    ] .
');

-- 为已嵌入的实体添加嵌入三元组
SELECT pg_ripple.add_embedding_triples();

-- 检查完整性
SELECT pg_ripple.validate();
```

### 模式：多语言支持

确保标签以必需的语言存在：

```sql
SELECT pg_ripple.load_shacl('
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix ex:   <https://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

ex:LabelShape a sh:NodeShape ;
    sh:targetSubjectsOf rdfs:label ;
    sh:property [
        sh:path rdfs:label ;
        sh:languageIn ( "en" "de" "fr" ) ;
        sh:uniqueLang true ;
    ] .
');
```

---

## 性能和权衡

| 验证模式 | 开销 | 数据完整性 | 用例 |
|---|---|---|---|
| `off` | 无 | 使用 `validate()` 手动检查 | 批量加载、开发 |
| `sync` | 高（每个三元组检查）| 立即拒绝 | 低量关键数据 |
| `async` | 低（后台工作线程）| 最终（违规在 DLQ 中）| 高吞吐量管道 |

- **形状数量**：验证时间随活动形状和焦点节点的数量线性扩展。停用您不需要的形状。
- **DAG 监视器**：每形状流表是 `IMMEDIATE` 模式 — 在同一事务中检测违规。但必须安装 pg_trickle。
- **死信队列**：无限制地增长。定期审查和清理：
  ```sql
  -- 删除超过 30 天的违规
  DELETE FROM _pg_ripple.dead_letter_queue
  WHERE detected_at < NOW() - INTERVAL '30 days';
  ```

```admonish tip
具有 `sh:maxCount 1` 的形状允许 SPARQL 查询引擎在该谓词的连接上省略 `DISTINCT`。具有 `sh:minCount 1` 的形状允许将 `LEFT JOIN` 降级为 `INNER JOIN`。声明准确的形状可以同时改善数据质量和查询性能。
```

---

## 陷阱和调试

### 形状加载错误

如果 `load_shacl()` 返回 0，则 Turtle 可能有语法错误。检查：

- 缺少 `@prefix` 声明
- 空节点属性列表中未关闭的括号
- 属性形状之间缺少分号

### 同步模式和事务边界

同步验证检查单个三元组，而不是整个事务。一篇论文可能通过 `dct:creator` 检查（因为正在插入的三元组是作者链接），但由于同一事务中尚未插入标题而未能通过 `dct:title` 检查。

解决方案：插入实体的所有三元组，然后显式验证：

```sql
SET pg_ripple.shacl_mode = 'off';

-- 插入实体的所有三元组
SELECT pg_ripple.insert_triple('<https://example.org/paper/900>', '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>', '<http://purl.org/ontology/bibo/AcademicArticle>');
SELECT pg_ripple.insert_triple('<https://example.org/paper/900>', '<http://purl.org/dc/terms/title>', '"My Paper"');
SELECT pg_ripple.insert_triple('<https://example.org/paper/900>', '<http://purl.org/dc/terms/creator>', '<https://example.org/person/alice>');

-- 然后验证
SELECT pg_ripple.validate();
```

### 查看形状定义

```sql
-- 列出所有形状及其活动状态
SELECT * FROM pg_ripple.list_shapes();
```

### 验证报告解释

验证报告 JSONB 具有两个顶级键：

- `conforms`：如果未找到违规则为 `true`
- `violations`：违规对象数组，每个都有 `focusNode`、`shapeIRI`、`path`、`constraint`、`message` 和 `severity`

```sql
-- 仅提取违规消息
SELECT v->>'message'
FROM jsonb_array_elements(
    (SELECT pg_ripple.validate()::jsonb -> 'violations')
) AS v;
```

---

## 后续步骤

- **[§2.5 推理和推理](../features/reasoning-and-inference.md)** — 从规则中派生新事实；SHACL 形状与推理交互。
- **[§2.6 导出和共享](../features/exporting-and-sharing.md)** — GraphRAG 导出的 SHACL 质量强制执行。
- **[§2.7 AI 检索和 GraphRAG](../features/graphrag.md)** — 嵌入完整性形状。

## 进一步阅读

- [博客：SHACL 数据质量](https://github.com/trickle-labs/pg-ripple/blob/main/blog/shacl-data-quality.md) — 深入探讨 SHACL 形状如何保护您的数据