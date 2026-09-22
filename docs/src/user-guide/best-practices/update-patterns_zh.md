# 更新模式

本页面涵盖了将数据写入 pg_ripple 的最佳实践 — 何时使用 `INSERT DATA` / `DELETE DATA` 与 `DELETE/INSERT WHERE`，如何管理命名图，以及如何编写幂等更新脚本。

## 选择正确的写入 API

| 场景 | 推荐 API |
|---|---|
| 加载大型文件（> ~1,000 个三元组） | `load_ntriples()` / `load_turtle()` |
| 从 SQL 插入单个已知三元组 | `insert_triple()` |
| 从支持 SPARQL 的客户端插入三元组 | `sparql_update()` 与 INSERT DATA |
| 删除精确的三元组 | `delete_triple()` 或 DELETE DATA |
| 基于模式的更新（查找并替换） | `DELETE/INSERT WHERE` |
| 清除命名图 | `CLEAR GRAPH <g>` |
| 加载远程 RDF 数据 | `LOAD <url>` |

---

## INSERT DATA 与批量加载

`INSERT DATA` 和批量加载（`load_ntriples`）都会产生相同的磁盘存储，但它们的性能特征不同：

| | `sparql_update`（INSERT DATA） | `load_ntriples` |
|---|---|---|
| 每个三元组的开销 | 中（每个术语的 SPL + 字典查找） | 低（批量字典操作） |
| 事务边界 | 每个调用一个 PG 事务 | 每个调用一个 PG 事务 |
| 典型吞吐量 | ~1,000–5,000 triples/sec | ~50,000–200,000 triples/sec |
| 使用场景 | 小型、针对性的写入 | 批量摄入 |

对于初始数据加载，始终使用 `load_ntriples` 或 `load_turtle`。将 `sparql_update` / `INSERT DATA` 保留用于增量更新。

---

## 基于模式的更新（DELETE/INSERT WHERE）

`DELETE/INSERT WHERE` 是 SPARQL 等价于 SQL `UPDATE` 的功能。它使用 WHERE 子句匹配三元组，然后为每个匹配项删除和/或插入三元组。WHERE 子句通过与 SELECT 查询相同的 SPARQL→SQL 引擎进行编译。

### 重命名属性值

```sql
-- 将所有 "draft" 状态值更改为 "published"：
SELECT pg_ripple.sparql_update('
    DELETE { ?s <https://example.org/status> <https://example.org/draft> }
    INSERT { ?s <https://example.org/status> <https://example.org/published> }
    WHERE  { ?s <https://example.org/status> <https://example.org/draft> }
');
```

返回值：`(# deleted) + (# inserted)`。对于 `N` 个匹配的主语，返回 `2N`。

### 有条件添加属性

```sql
-- 对于每个缺少电子邮件的人员，插入一个占位符：
SELECT pg_ripple.sparql_update('
    INSERT { ?person <https://schema.org/email> "no-reply@example.org" }
    WHERE  {
        ?person a <https://schema.org/Person> .
        FILTER NOT EXISTS { ?person <https://schema.org/email> ?e }
    }
');
```

### 仅按模式删除

可以省略 `INSERT` 以仅删除：

```sql
SELECT pg_ripple.sparql_update('
    DELETE { ?s <https://example.org/temp> ?o }
    WHERE  { ?s <https://example.org/temp> ?o }
');
```

或省略 `DELETE` 以仅插入：

```sql
SELECT pg_ripple.sparql_update('
    INSERT { ?s <https://example.org/indexed> "true"^^<http://www.w3.org/2001/XMLSchema#boolean> }
    WHERE  { ?s <https://example.org/name> ?name }
');
```

### 性能说明

对于每个 WHERE 绑定，DELETE 阶段和 INSERT 阶段单独运行。对于大型结果集（数千个绑定），考虑通过 `load_*` API 进行批处理，或使用预计算数据的单个 `INSERT DATA`。

---

## 图生命周期管理

### 创建和填充命名图

命名图在插入第一个三元组时隐式创建。`CREATE GRAPH` 对 SPARQL 合规性很有用，或在任何三元组到达之前预先注册字典中的图 IRI。

```sql
-- 显式创建（可选）：
SELECT pg_ripple.sparql_update(
    'CREATE GRAPH <https://example.org/mygraph>'
);

-- 通过 INSERT 隐式创建：
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        GRAPH <https://example.org/mygraph> {
            <https://example.org/a> <https://example.org/b> <https://example.org/c>
        }
    }
');
```

### CLEAR 与 DROP

两种操作都会删除图中的所有三元组。区别在于概念上 — DROP "删除"图，而 CLEAR 保留为空容器。在 pg_ripple 中，两者在存储上的行为相同（无论如何图 IRI 都保留在字典中）。

```sql
-- 删除所有三元组，保留图：
SELECT pg_ripple.sparql_update(
    'CLEAR GRAPH <https://example.org/mygraph>'
);

-- 删除所有三元组和图：
SELECT pg_ripple.sparql_update(
    'DROP GRAPH <https://example.org/mygraph>'
);
```

### 一次清除多个图

```sql
-- 仅清除默认图：
SELECT pg_ripple.sparql_update('CLEAR DEFAULT');

-- 清除所有命名图（默认图不触及）：
SELECT pg_ripple.sparql_update('CLEAR NAMED');

-- 清除所有内容（默认 + 所有命名）：
SELECT pg_ripple.sparql_update('CLEAR ALL');
```

### SILENT 修饰符

添加 `SILENT` 可以抑制错误（例如，如果图不存在）：

```sql
SELECT pg_ripple.sparql_update(
    'DROP SILENT GRAPH <https://example.org/nonexistent>'
);
SELECT pg_ripple.sparql_update(
    'CLEAR SILENT GRAPH <https://example.org/nonexistent>'
);
```

---

## 加载远程 RDF 数据（LOAD）

`LOAD <url>` 通过 HTTP(S) 获取远程 RDF 文档并插入所有三元组。

```sql
-- 加载到默认图：
SELECT pg_ripple.sparql_update(
    'LOAD <https://www.w3.org/People/Berners-Lee/card.rdf>'
);

-- 加载到命名图：
SELECT pg_ripple.sparql_update(
    'LOAD <https://example.org/data.ttl> INTO GRAPH <https://example.org/remote>'
);

-- 忽略 HTTP 错误：
SELECT pg_ripple.sparql_update(
    'LOAD SILENT <https://example.org/maybe-missing.nt>'
);
```

格式从 `Content-Type` 或 URL 扩展名检测：
- `text/turtle` / `.ttl` → Turtle
- `application/rdf+xml` / `.rdf` / `.owl` → RDF/XML
- 其他所有内容 → N-Triples

对于大型远程文件，在单独获取文件后优先使用文件加载 API（`load_ntriples`、`load_turtle`）— `LOAD` 在解析前将整个响应缓冲在内存中。

---

## 幂等插入模式

由于 VP 表使用使用 `ON CONFLICT DO NOTHING`，插入已存在的数据是安全的 — SID 将返回插入并将其计为 1 个受影响的三元组，`sparql_update()` 也将其计为 1 个受影响的三元组。

要编写幂等 SQL 迁移脚本：

```sql
-- 可以安全运行多次
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        <https://example.org/config> <https://example.org/version>
            "2"^^<http://www.w3.org/2001/XMLSchema#integer>
    }
');
```

要实现"如果不存在则设置"模式（仅在主语还没有谓词时插入）：

```sql
-- 仅当 alice 还没有电子邮件时插入。
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_ripple.find_triples(
            '<https://example.org/alice>',
            '<https://schema.org/email>',
            NULL
        )
    ) THEN
        PERFORM pg_ripple.insert_triple(
            '<https://example.org/alice>',
            '<https://schema.org/email>',
            '"alice@example.org"'
        );
    END IF;
END $$;
```

---

## 原子替换（删除 + 插入）

要原子地替换属性的值：

```sql
BEGIN;

-- 删除旧值
SELECT pg_ripple.sparql_update(
    'DELETE DATA { <https://example.org/alice> <https://schema.org/name> "Alice Smith" }'
);

-- 插入新值
SELECT pg_ripple.sparql_update(
    'INSERT DATA { <https://example.org/alice> <https://schema.org/name> "Alice Jones" }'
);

COMMIT;
```

> **提示**：替换值时，将删除和插入包装在单个 `BEGIN / COMMIT` 块中，以便读取者永远看不到属性缺失的中间状态。

---

## 使用内联编码类型进行高效范围查询

对于将在 SPARQL FILTER 中进行比较的数字或日期谓词，使用具有内联兼容类型的类型化字面量：

| 使用此类型 | 替代 |
|---|---|
| `"42"^^xsd:integer` | `"42"`（纯字符串） |
| `"2024-01-01"^^xsd:date` | `"2024-01-01"`（纯字符串） |
| `"true"^^xsd:boolean` | `"true"`（纯字符串） |

使用内联编码类型，像 `FILTER(?age > 30)` 这样的 FILTER 比较会编译为 `WHERE o > <inline_id>` — 在 VP 表列上进行简单整数比较，无需字典连接。

```sql
-- 推荐：使用 age 的内联编码
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        <https://example.org/alice> <https://example.org/age>
            "30"^^<http://www.w3.org/2001/XMLSchema#integer>
    }
');

-- 效率较低：存储为纯字符串；FILTER 比较需要字典连接
SELECT pg_ripple.sparql_update('
    INSERT DATA {
        <https://example.org/alice> <https://example.org/age> "30"
    }
');
```

---

## 批量删除

要在单个 SQL 调用中删除主语的所有三元组（比每个三元组的 DELETE DATA 更快）：

```sql
-- 删除 alice 作为主语的所有三元组
SELECT pg_ripple.delete_triple(s, p, o)
FROM pg_ripple.find_triples('<https://example.org/alice>', NULL, NULL)
AS t(s TEXT, p TEXT, o TEXT, g TEXT, i BIGINT);
```

对于命名图隔离，按图 ID 过滤：

```sql
SELECT pg_ripple.delete_triple(s, p, o)
FROM pg_ripple.find_triples(NULL, NULL, NULL)
WHERE g = pg_ripple.graph_id('<https://example.org/draft-graph>');
```