# 关键概念 — PostgreSQL 用户的 RDF

如果您了解 PostgreSQL，您已经了解使用 pg_ripple 所需的大部分内容。本页将 RDF 概念映射到它们的 PostgreSQL 等价物。

## 三元组

**三元组**是 RDF 中的原子数据单元。它有三个部分：

| 部分 | 是什么 | PostgreSQL 类比 |
|---|---|---|
| **主语** | 被描述的实体 | 行的主键 |
| **谓词** | 关系或属性 | 列名 |
| **对象** | 值或相关实体 | 单元格值或外键 |

例如，"Alice 认识 Bob"的事实是三元组：

```
<http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> <http://example.org/bob> .
```

在 pg_ripple 中，此三元组存储在以谓词命名的 VP 表中（`foaf:knows`），主语和对象列使用整数编码。

## IRI

**IRI**（国际化资源标识符）是实体或关系的全局唯一标识符。可以将其视为保证在世界上所有数据集中都唯一的命名空间主键。

```
http://example.org/alice          -- 一个实体
http://xmlns.com/foaf/0.1/knows   -- 一个关系
```

**前缀**是快捷方式，可避免反复编写完整的 IRI：

```sql
SELECT pg_ripple.register_prefix('ex', 'http://example.org/');
-- 现在 ex:alice 意味着 http://example.org/alice
```

## 空节点

**空节点**是匿名实体 — 类似于没有主键的行。它仅存在于创建它的文档中。

```turtle
ex:alice foaf:address [ foaf:city "Boston" ; foaf:country "US" ] .
```

该地址没有 IRI。它是一个空节点，由系统生成的标签在内部标识。来自不同 `load_turtle()` 调用的空节点始终是不同的实体，即使它们共享相同的标签。

```admonish warning
空节点无法从其原始加载调用之外引用。如果您需要从多个位置引用一个实体，请给它一个 IRI。
```

## 字面量

**字面量**是数据值 — 字符串、数字、日期或布尔值。字面量可以具有数据类型或语言标签。

| 字面量 | 类型 | PostgreSQL 等价物 |
|---|---|---|
| `"Alice"` | 纯字符串 | `TEXT` |
| `"42"^^xsd:integer` | 类型化整数 | `INTEGER` |
| `"2024-01-15"^^xsd:date` | 类型化日期 | `DATE` |
| `"Bonjour"@fr` | 带语言标签的字符串 | 无直接等价物 |

在 pg_ripple 中，所有字面量都进行字典编码为紧凑的整数 ID 以供存储。原始的字符串表示形式被保留并在查询输出时解码。

## 谓词和 VP 表

在关系数据库中，表对单个实体类型的所有属性进行分组。在 pg_ripple 中，数据按**谓词**组织 — 每个唯一谓词都获得自己的表（垂直分区或 VP 表）。

```
关系型：  persons(id, name, email, knows_id)
pg_ripple：   vp_foaf_name(s, o)      -- 主语 → 名称
             vp_foaf_knows(s, o)     -- 主语 → 对象
             vp_schema_email(s, o)   -- 主语 → 电子邮件
```

这种结构使 JOIN 繁重的 SPARQL 查询变得快速，因为每个谓词的数据都是并置的并已建立索引。

## 命名图

**命名图**是带标签的三元组集合 — 类似于对相关表进行分组的 PostgreSQL schema。

```sql
-- 创建命名图
SELECT pg_ripple.create_graph('http://example.org/publications');

-- 将数据加载到其中
SELECT pg_ripple.load_turtle_into_graph(
  '<http://example.org/paper1> <http://purl.org/dc/elements/1.1/title> "My Paper" .',
  'http://example.org/publications'
);
```

命名图可用于：

- **多源数据**：将来自不同源的数据分开
- **访问控制**：按角色授予对特定图的读取权限
- **版本控制**：将新数据加载到新图中，验证，然后交换

所有没有显式图的三元组都属于**默认图**（图 ID = 0）。

## RDF-star

标准 RDF 说"Alice 认识 Bob"。但是如果您想说 Alice 是*何时*遇到 Bob 的，或者是*谁*记录了该事实呢？**RDF-star** 允许您对语句进行陈述：

```
<< ex:alice foaf:knows ex:bob >> ex:since "2020"^^xsd:gYear .
```

这表示："Alice 认识 Bob 的事实从 2020 年以来就是真的。" 在 pg_ripple 中，每个三元组都有一个语句标识符（SID），可以用作其他三元组的主语或对象，从而启用类似于标记属性图的边属性。

## SPARQL

**SPARQL** 是 RDF 数据的标准查询语言 — 关系数据库的 SQL 等价物。SQL 查询表，而 SPARQL 查询图模式。

| SQL | SPARQL |
|---|---|
| `SELECT name FROM persons WHERE id = 1` | `SELECT ?name WHERE { ex:person1 foaf:name ?name }` |
| `JOIN` | 图模式匹配（隐式）|
| `LEFT JOIN` | `OPTIONAL { }` |
| `WHERE x IN (...)` | `VALUES (?x) { ... }` |
| `GROUP BY ... HAVING` | `GROUP BY ... HAVING` |
| `WITH RECURSIVE` | 属性路径（`foaf:knows+`）|

在 pg_ripple 中，SPARQL 查询被编译为 SQL 并通过 PostgreSQL 的查询引擎执行。您通过 `pg_ripple.sparql()` 调用它们：

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name WHERE { ?person foaf:name ?name }
');
```

## 字典编码

pg_ripple 不在其数据表中存储原始字符串。每个 IRI、空节点和字面量都由字典编码器映射为紧凑的 `BIGINT`（i64）。VP 表仅包含整数列，从而使 JOIN 和比较快速。

您永远不需要直接与字典 ID 交互 — `sparql()` 和 `find_triples()` 自动处理编码和解码。对于高级用例，`encode_term()` 和 `decode_id()` 可用。

## 类比总结

| RDF 概念 | PostgreSQL 类比 |
|---|---|
| 三元组 | 表中的一行 |
| 主语 | 主键值 |
| 谓词 | 列名 / 表名（VP）|
| 对象 | 单元格值或外键 |
| IRI | 全局唯一标识符 |
| 空节点 | 带系统生成 ID 的行 |
| 字面量 | 类型化列值 |
| 命名图 | Schema |
| SPARQL | SQL |
| SHACL shape | CHECK 约束 / 触发器 |
| Datalog 规则 | 物化视图定义 |

## 延伸阅读

- [博客：为什么在 PostgreSQL 中使用 RDF？](https://github.com/trickle-labs/pg-ripple/blob/main/blog/why-rdf-in-postgresql.md) — pg_ripple 背后的设计哲学
- [博客：字典编码和整数连接](https://github.com/trickle-labs/pg-ripple/blob/main/blog/dictionary-encoding-integer-joins.md) — pg_ripple 如何实现快速查询性能
- [博客：垂直分区详解](https://github.com/trickle-labs/pg-ripple/blob/main/blog/vertical-partitioning-explained.md) — 为什么每个谓词一个表是可行的

## 下一步

- [存储知识](stores-knowledge.md) — 使用三元组进行数据建模
- [加载数据](loading-data.md) — 所有导入格式和方法
- [使用 SPARQL 查询](querying-with-sparql.md) — 完整的查询语言