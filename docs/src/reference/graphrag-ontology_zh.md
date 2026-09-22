# GraphRAG 本体参考

GraphRAG 命名空间 `https://graphrag.org/ns/`（前缀 `gr:`）定义了用于在 pg_ripple 中表示 GraphRAG 知识图谱的词汇表。

## 类

### `gr:Entity`
从源文本中提取的命名实体。

**必需属性：** `gr:title`
**可选属性：** `gr:type`、`gr:description`、`gr:frequency`、`gr:degree`

### `gr:Relationship`
两个实体之间的有向关系。

**必需属性：** `gr:source`、`gr:target`
**可选属性：** `gr:description`、`gr:weight`

### `gr:TextUnit`
从中提取实体的源文本块。

**必需属性：** `gr:text`
**可选属性：** `gr:tokenCount`、`gr:documentId`、`gr:mentionsEntity`

### `gr:Community`
相关实体的社区（来自社区检测）。

**可选属性：** `gr:title`、`gr:description`、`gr:rank`、`gr:level`

### `gr:CommunityReport`
社区的自然语言摘要报告。

**可选属性：** `gr:summary`、`gr:findings`、`gr:rating`

## 属性

### 数据属性

| 属性 | 定义域 | 值域 | 注释 |
|---|---|---|---|
| `gr:title` | `gr:Entity`、`gr:Community` | `xsd:string` | 显示标签 |
| `gr:type` | `gr:Entity` | `xsd:string` | 例如 `"PERSON"`、`"ORGANIZATION"` |
| `gr:description` | 任何 | `xsd:string` | 自由文本描述 |
| `gr:text` | `gr:TextUnit` | `xsd:string` | 原始源文本 |
| `gr:tokenCount` | `gr:TextUnit` | `xsd:integer` | 标记计数 |
| `gr:frequency` | `gr:Entity` | `xsd:integer` | 提及频率 |
| `gr:degree` | `gr:Entity` | `xsd:integer` | 图度数（# 关系） |
| `gr:weight` | `gr:Relationship` | `xsd:float` | 关系强度 0–1 |
| `gr:rank` | `gr:Community` | `xsd:integer` | 社区排名 |
| `gr:level` | `gr:Community` | `xsd:integer` | 层次级别 |
| `gr:summary` | `gr:CommunityReport` | `xsd:string` | 摘要文本 |
| `gr:rating` | `gr:CommunityReport` | `xsd:float` | 影响评级 |

### 对象属性

| 属性 | 定义域 | 值域 | 注释 |
|---|---|---|---|
| `gr:source` | `gr:Relationship` | `gr:Entity` | 关系源实体 |
| `gr:target` | `gr:Relationship` | `gr:Entity` | 关系目标实体 |
| `gr:mentionsEntity` | `gr:TextUnit` | `gr:Entity` | 文本单元中提及的实体 |
| `gr:documentId` | `gr:TextUnit` | `gr:Document` | 源文档 |
| `gr:hasReport` | `gr:Community` | `gr:CommunityReport` | 社区报告 |

### 派生属性（Datalog）

这些属性不是直接加载的，而是由富化规则集派生的：

| 属性 | 语义 |
|---|---|
| `gr:coworker` | 对称 — 共享共同的关系目标 |
| `gr:collaborates` | 对称 — 都在同一文本单元中提及 |
| `gr:indirectReport` | `gr:manages` 的传递闭包 |
| `gr:relatedOrg` | 由共享实体桥接的组织 |

## 本体文件

完整的 OWL 本体作为 `sql/graphrag_ontology.ttl` 提供。使用以下命令加载：

```sql
SELECT pg_ripple.load_turtle(pg_read_file('/path/to/graphrag_ontology.ttl'));
```

## SHACL 形状

验证形状在 `sql/graphrag_shapes.ttl` 中。使用以下命令加载：

```sql
SELECT pg_ripple.load_shacl(pg_read_file('/path/to/graphrag_shapes.ttl'));
```