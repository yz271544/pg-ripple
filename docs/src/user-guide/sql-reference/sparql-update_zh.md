# SPARQL 更新

pg_ripple 通过 `sparql_update()` 函数支持完整的 [SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/) 规范。所有更新操作 — `INSERT DATA`、`DELETE DATA`、`DELETE/INSERT WHERE`、`LOAD`、`CLEAR`、`DROP` 和 `CREATE` — 从 v0.12.0 开始可用。

## sparql_update

```sql
pg_ripple.sparql_update(query TEXT) RETURNS BIGINT
```

执行 SPARQL Update 语句，返回受影响的三元组数量（已插入 + 已删除）。

### INSERT DATA

将一个或多个基础三元组（无变量）添加到存储：

```sql
-- 插入单个三元组
SELECT pg_ripple.sparql_update(
    'INSERT DATA { <https://example.org/alice> <https://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> }'
);

-- 在一个语句中插入多个三元组
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> .
        <https://example.org/bob>   <https://example.org/name>  "Bob"
    }
');
```

所有主语、谓词和对象在插入前都进行字典编码。符合内联编码条件的类型化字面量（`xsd:integer`、`xsd:boolean`、`xsd:date`、`xsd:dateTime`）以位打包的 ID 形式存储，而不是字典行。

### DELETE DATA

从存储中删除精确匹配的三元组：

```sql
SELECT pg_ripple.sparql_update(
    'DELETE DATA { <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> }'
);
```

如果三元组不存在则返回 `0`（无错误）。

### DELETE/INSERT WHERE

基于模式的更新通过 WHERE 子句查找匹配的三元组，并为每个匹配删除和/或插入三元组。这是 SQL `UPDATE` 的 SPARQL 等价物。

```sql
-- 将所有"draft"状态值替换为"published"：
SELECT pg_ripple.sparql_update('
    DELETE { ?s <https://example.org/status> <https://example.org/draft> }
    INSERT { ?s <https://example.org/status> <https://example.org/published> }
    WHERE  { ?s <https://example.org/status> <https://example.org/draft> }
');

-- 为每个没有 email 的人添加占位符：
SELECT pg_ripple.sparql_update('
    INSERT { ?person <https://schema.org/email> "no-reply@example.org" }
    WHERE  {
        ?person a <https://schema.org/Person> .
        FILTER NOT EXISTS { ?person <https://schema.org/email> ?e }
    }
');
```

WHERE 子句通过现有的 SPARQL→SQL 引擎进行编译。WHERE 子句中的所有绑定变量在 DELETE 和 INSERT 模板中都可用。对于每个绑定行，DELETE 阶段在 INSERT 阶段之前运行。整个操作是事务性的。

返回值：`(已删除的三元组数) + (已插入的三元组数)`。

### 命名图

所有形式都支持命名图：

```sql
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        GRAPH <https://example.org/graph1> {
            <https://example.org/alice> <https://example.org/memberOf> <https://example.org/org1>
        }
    }
');

-- 命名图中的基于模式的更新：
SELECT pg_ripple.sparql_update('
    DELETE { GRAPH <https://example.org/graph1> { ?s ?p ?o } }
    WHERE  { GRAPH <https://example.org/graph1> { ?s ?p ?o } }
');
```

默认图的 ID 为 `0`。命名图在插入第一个三元组时隐式创建。

## 图管理

### LOAD

通过 HTTP(S) 从 URL 获取 RDF 文档并将所有三元组插入存储。

```sql
-- 加载到默认图中：
SELECT pg_ripple.sparql_update(
    'LOAD <https://www.w3.org/People/Berners-Lee/card.rdf>'
);

-- 加载到命名图中：
SELECT pg_ripple.sparql_update(
    'LOAD <https://example.org/data.ttl> INTO GRAPH <https://example.org/mygraph>'
);

-- 忽略错误（例如网络故障）：
SELECT pg_ripple.sparql_update(
    'LOAD SILENT <https://example.org/data.nt>'
);
```

格式检测：如果 `Content-Type` 包含 `turtle` 或 URL 以 `.ttl` 结尾则为 Turtle；如果 `rdf+xml` 或 `.rdf`/`.owl` 则为 RDF/XML；否则为 N-Triples。TriG 内的命名图不会被分割出来 — 目标图会覆盖它们。

### CLEAR

从目标图中删除所有三元组，但不从字典中删除图名称。

```sql
-- 清除特定命名图：
SELECT pg_ripple.sparql_update(
    'CLEAR GRAPH <https://example.org/mygraph>'
);

-- 清除默认图：
SELECT pg_ripple.sparql_update('CLEAR DEFAULT');

-- 清除所有命名图（默认图未触及）：
SELECT pg_ripple.sparql_update('CLEAR NAMED');

-- 清除所有内容：
SELECT pg_ripple.sparql_update('CLEAR ALL');

-- 如果图不存在则忽略：
SELECT pg_ripple.sparql_update(
    'CLEAR SILENT GRAPH <https://example.org/nonexistent>'
);
```

### DROP

类似于 CLEAR，但还会取消注册图。在 pg_ripple 中，图的 IRI 仍保留在字典中（取消注册除了清除三元组之外是无操作），因此 DROP 和 CLEAR 在功能上是等效的。

```sql
SELECT pg_ripple.sparql_update(
    'DROP GRAPH <https://example.org/mygraph>'
);
SELECT pg_ripple.sparql_update('DROP ALL');
SELECT pg_ripple.sparql_update(
    'DROP SILENT GRAPH <https://example.org/nonexistent>'
);
```

### CREATE

在字典中注册命名图。由于 pg_ripple 在首次插入时隐式创建图，因此 `CREATE GRAPH` 很少需要，但出于 SPARQL 兼容性而受支持。

```sql
SELECT pg_ripple.sparql_update(
    'CREATE GRAPH <https://example.org/newgraph>'
);
-- 如果图已存在则无操作：
SELECT pg_ripple.sparql_update(
    'CREATE SILENT GRAPH <https://example.org/newgraph>'
);
```

## 返回值

`sparql_update()` 返回受影响的三元组的总数：

| 语句 | 返回值 |
|---|---|
| `INSERT DATA { t1 . t2 . t3 }` | 如果全部都是新的则为 3 |
| `DELETE DATA { t1 }` | 如果找到则为 1，如果未找到则为 0 |
| `DELETE { … } INSERT { … } WHERE { … }` | 删除 + 插入 |
| `CLEAR GRAPH <g>` | 已删除的三元组 |
| `DROP GRAPH <g>` | 已删除的三元组 |
| `CREATE GRAPH <g>` | 0（未触及三元组）|
| `LOAD <url>` | 已插入的三元组 |

## 与 insert_triple / delete_triple 的比较

| | `insert_triple` | `sparql_update` |
|---|---|---|
| 输入格式 | N-Triples 字符串（主语、谓词、对象）| SPARQL 文本 |
| 多个三元组 | 每个三元组一个调用 | 每个语句多个三元组 |
| 基于模式 | 否 | 是（DELETE/INSERT WHERE）|
| 图管理 | 否 | 是（CLEAR/DROP/CREATE/LOAD）|
| 用例 | 从 SQL 进行编程式插入 | 基于 SPARQL 的工具和符合标准的客户端 |

## sparql_update

```sql
pg_ripple.sparql_update(query TEXT) RETURNS BIGINT
```

执行 SPARQL Update 语句，返回受影响的三元组数量。

### INSERT DATA

将一个或多个基础三元组（无变量）添加到存储：

```sql
-- 插入单个三元组
SELECT pg_ripple.sparql_update(
    'INSERT DATA { <https://example.org/alice> <https://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> }'
);

-- 在一个语句中插入多个三元组
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> .
        <https://example.org/bob>   <https://example.org/name>  "Bob"
    }
');
```

所有主语、谓词和对象在插入前都进行字典编码。符合内联编码条件的类型化字面量（`xsd:integer`、`xsd:boolean`、`xsd:date`、`xsd:dateTime`）以位打包的 ID 形式存储，而不是字典行。

### DELETE DATA

从存储中删除精确匹配的三元组：

```sql
SELECT pg_ripple.sparql_update(
    'DELETE DATA { <https://example.org/alice> <https://example.org/knows> <https://example.org/bob> }'
);
```

如果三元组不存在则返回 `0`（无错误）。DELETE DATA 是一个原子集操作 — 语句中的所有三元组都被删除，或都不被删除（如果任何缺失，计数仅反映实际删除的三元组）。

### 命名图

两种形式都支持命名图：

```sql
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        GRAPH <https://example.org/graph1> {
            <https://example.org/alice> <https://example.org/memberOf> <https://example.org/org1>
        }
    }
');
```

默认图的 ID 为 `0`。命名图在插入第一个三元组时隐式创建。

## 返回值

`sparql_update()` 返回已插入或已删除的三元组数：

| 语句 | 返回值 |
|---|---|
| `INSERT DATA { t1 . t2 . t3 }` | 如果全部都是新的则为 3；如果某些已存在则更少 |
| `DELETE DATA { t1 }` | 如果找到则为 1，如果未找到则为 0 |

## 与 insert_triple / delete_triple 的比较

| | `insert_triple` | `sparql_update` |
|---|---|---|
| 输入格式 | N-Triples 字符串（主语、谓词、对象）| SPARQL 文本 |
| 缺少参数 | 不适用 — INSERT/DELETE DATA 的所有部分都是必需的 | 不适用 |
| 多个三元组 | 每个三元组一个调用 | 每个语句多个三元组 |
| 用例 | 从 SQL 或应用程序代码进行编程式插入 | 基于 SPARQL 的工具和符合标准的客户端 |

## 不支持的形式（计划在以后的版本中提供）

v0.5.1 中**尚未支持**以下 SPARQL Update 形式：

- `DELETE/INSERT WHERE { … }` — 基于模式的更新（v0.12.0）
- `LOAD <url>`（v0.12.0）
- `CLEAR GRAPH <g>`（v0.12.0）
- `DROP GRAPH <g>`（v0.12.0）
- `CREATE GRAPH <g>`（v0.12.0）
- 更新序列（`; UPDATE1 ; UPDATE2`）（v0.12.0）

## 图管理操作（v0.38.0）

从 v0.38.0 开始支持 `ADD`、`COPY` 和 `MOVE`。这些操作在执行前由 spargebra 解析器去糖化为等效的 `INSERT DATA` / `DELETE DATA` 序列。

```sql
-- ADD：将所有三元组从 SOURCE 复制到 DEST（保留 SOURCE）
SELECT sparql_update('ADD GRAPH <http://example.org/source> TO GRAPH <http://example.org/dest>');

-- COPY：用 SOURCE 的内容替换 DEST（保留 SOURCE）
SELECT sparql_update('COPY GRAPH <http://example.org/source> TO GRAPH <http://example.org/dest>');

-- MOVE：将 SOURCE 重命名为 DEST（清空 SOURCE）
SELECT sparql_update('MOVE GRAPH <http://example.org/source> TO GRAPH <http://example.org/dest>');
```

所有三个操作都使用 `DEFAULT` 代替命名图 IRI 来引用默认图。