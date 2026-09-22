# RDF-star

RDF-star (RDF\*) 通过允许将三元组用作其他三元组的主语或对象来扩展 RDF。这允许关于语句的语句*关于*语句 — 对出处、时间注释和 LPG 风格的边属性很有用。

## 概述

在 RDF-star 中，**引用三元组** `<< s p o >>` 可以出现在主语或对象位置：

```
<< <ex:alice> <ex:knows> <ex:bob> >> <ex:assertedBy> <ex:carol> .
```

pg_ripple 在字典中以 `kind = 5`（`KIND_QUOTED_TRIPLE`）存储引用三元组，并使用它们的组件字典 ID 对它们进行编码。

## encode_triple

```sql
pg_ripple.encode_triple(
    subject   TEXT,
    predicate TEXT,
    object    TEXT
) RETURNS BIGINT
```

将三元组编码为字典条目并返回其 `BIGINT` ID。幂等 — 使用相同三元组的重复调用返回相同的 ID。

```sql
SELECT pg_ripple.encode_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
);
```

## decode_triple

```sql
pg_ripple.decode_triple(id BIGINT) RETURNS JSONB
```

将给定字典 ID 处编码的三元组作为 JSONB 对象 `{"s":…,"p":…,"o":…}` 返回。

```sql
SELECT pg_ripple.decode_triple(42);
-- 返回：{"s":"<https://example.org/alice>","p":"<https://example.org/knows>","o":"<https://example.org/bob>"}
```

## insert_triple 和 SID

`insert_triple()` 返回**语句标识符**（SID）— 插入三元组的全局唯一 `BIGINT`。SID 可用作后续三元组中的主语或对象，以注释该语句。

```sql
DECLARE sid BIGINT;
SELECT pg_ripple.insert_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
) INTO sid;

-- 使用出处注释该语句
SELECT pg_ripple.insert_triple(
    '<https://example.org/sid-' || sid || '>',
    '<https://example.org/assertedBy>',
    '<https://example.org/carol>'
);
```

## get_statement

```sql
pg_ripple.get_statement(i BIGINT) RETURNS JSONB
```

按其 SID 查找三元组并将其作为 `{"s":…,"p":…,"o":…,"g":…}` 返回。

```sql
SELECT pg_ripple.get_statement(1);
-- 返回：{"s":"<ex:alice>","p":"<ex:knows>","o":"<ex:bob>","g":"0"}
```

## 加载 RDF-star 数据

`load_ntriples()` 接受在主语位置和对象位置带引用三元组的 N-Triples-star 输入：

```sql
SELECT pg_ripple.load_ntriples('
<< <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> >>
    <https://example.org/assertedBy>
    <https://example.org/carol> .
');
```

## SPARQL-star 模式

在 SPARQL WHERE 子句中支持基础（全部为常量）引用三元组模式：

```sql
SELECT * FROM pg_ripple.sparql('
  SELECT ?who WHERE {
    << <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> >>
        <https://example.org/assertedBy> ?who
  }
');
```

## LPG 边属性映射

RDF-star 自然适合编码 LPG 边属性：引用三元组表示边，关于引用三元组的后续三元组对属性进行编码。

```
<< <ex:alice> <ex:knows> <ex:bob> >> <ex:since>   "2023-01-01"^^xsd:date .
<< <ex:alice> <ex:knows> <ex:bob> >> <ex:strength> "strong" .
```

## 引用三元组内的变量模式（v0.48.0）

从 v0.48.0 开始，支持引用三元组模式内的变量。这允许将变量绑定到作为另一个三元组的主语或对象出现的引用三元组的组件：

```sql
-- 将 ?v 绑定到匹配引用三元组的对象组件
SELECT * FROM pg_ripple.sparql('
  PREFIX ex: <http://example.org/>
  SELECT ?v ?who WHERE {
    << ex:alice ex:age ?v >> ex:assertedBy ?who .
  }
');

-- 绑定所有三个组件
SELECT * FROM pg_ripple.sparql('
  PREFIX ex: <http://example.org/>
  SELECT ?s ?p ?o ?who WHERE {
    << ?s ?p ?o >> ex:assertedBy ?who .
  }
');
```

这是通过将 `_pg_ripple.dictionary` 表连接在 `qt_s`、`qt_p` 和 `qt_o` 列上（可用于 `kind = 5` 的条目）来实现的。