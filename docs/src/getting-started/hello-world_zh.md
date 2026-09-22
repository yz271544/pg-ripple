# Hello World — 五分钟演练

本演练将带您从空数据库到五分钟内可用的 SPARQL 查询。您将加载有关人和电影的十个三元组，然后运行三个复杂度递增的查询。

## 先决条件

pg_ripple 已安装，您已连接到已创建扩展的 PostgreSQL 数据库。如果您尚未执行此操作，请参阅[安装](installation.md)。

## 第 1 步：注册前缀

前缀是长 IRI 的快捷方式。注册一些常用的：

```sql
SELECT pg_ripple.register_prefix('ex', 'http://example.org/');
SELECT pg_ripple.register_prefix('foaf', 'http://xmlns.com/foaf/0.1/');
SELECT pg_ripple.register_prefix('schema', 'http://schema.org/');
```

## 第 2 步：加载数据

加载有关人以及他们导演或出演的电影的十个三元组：

```sql
SELECT pg_ripple.load_turtle('
  @prefix ex:     <http://example.org/> .
  @prefix foaf:   <http://xmlns.com/foaf/0.1/> .
  @prefix schema: <http://schema.org/> .

  ex:alice   foaf:name     "Alice" .
  ex:alice   schema:knows  ex:bob .
  ex:bob     foaf:name     "Bob" .
  ex:bob     schema:knows  ex:carol .
  ex:carol   foaf:name     "Carol" .
  ex:movie1  schema:name   "The Graph" .
  ex:movie1  schema:director ex:alice .
  ex:movie1  schema:actor    ex:bob .
  ex:movie2  schema:name   "Linked Data" .
  ex:movie2  schema:director ex:bob .
');
```

该函数返回已加载的三元组数（10）。

## 第 3 步：查询 — 基本模式

查找所有电影及其导演：

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX schema: <http://schema.org/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?movieName ?directorName WHERE {
    ?movie schema:director ?person .
    ?movie schema:name ?movieName .
    ?person foaf:name ?directorName .
  }
');
```

结果中的每一行都是一个带有变量绑定的 JSONB 对象。您应该看到"由 Alice 导演的 The Graph"和"由 Bob 导演的 Linked Data"。

## 第 4 步：查询 — OPTIONAL

查找所有电影及其导演，以及演员（如果有）：

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX schema: <http://schema.org/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?movieName ?directorName ?actorName WHERE {
    ?movie schema:name ?movieName .
    ?movie schema:director ?director .
    ?director foaf:name ?directorName .
    OPTIONAL {
      ?movie schema:actor ?actor .
      ?actor foaf:name ?actorName .
    }
  }
');
```

"The Graph"有一位演员（Bob），而"Linked Data"没有 — 该行的 `actorName` 列为空。`OPTIONAL` 关键字的工作方式类似于 SQL `LEFT JOIN`。

## 第 5 步：查询 — 属性路径

查找通过 `schema:knows` 链接直接或间接连接到 Alice 的所有人：

```sql
SELECT * FROM pg_ripple.sparql('
  PREFIX ex: <http://example.org/>
  PREFIX schema: <http://schema.org/>
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name WHERE {
    ex:alice schema:knows+ ?person .
    ?person foaf:name ?name .
  }
');
```

`+` 运算符沿 `schema:knows` 关系跟踪一次或多次。Alice 直接认识 Bob，Bob 又认识 Carol，因此查询返回"Bob"和"Carol"。

## 您刚刚学到的内容

- **三元组**是具有三个部分的事实：主语、谓词、对象
- **前缀**是长 IRI 的快捷方式
- **`load_turtle()`** 以 Turtle 格式加载数据
- **`sparql()`** 运行 SPARQL 查询并将结果作为 JSONB 返回
- **OPTIONAL** 类似于 SQL `LEFT JOIN`
- **属性路径**（`+`、`*`）沿关系链进行跟踪

## 下一步

- [引导式教程](tutorial.md) — 构建具有验证和推理的完整知识图谱
- [关键概念](key-concepts.md) — 使用 PostgreSQL 类比理解 RDF 概念
- [使用 SPARQL 查询](../features/querying-with-sparql.md) — 完整的 SPARQL 功能集