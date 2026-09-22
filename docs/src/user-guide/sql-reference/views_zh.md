# 物化视图

pg_ripple v0.11.0 与 [pg_trickle](https://github.com/trickle-labs/pg-trickle) 集成，为 SPARQL 查询、Datalog 目标和谓词半连接提供始终保持最新、增量维护的流表。所有三个功能都是软门控的 — pg_ripple 在没有 pg_trickle 的情况下可以正常加载和运行；新函数在调用时检测其缺失并返回带有安装提示的明确错误。

---

## 检查 pg_trickle 可用性

```sql
SELECT pg_ripple.pg_trickle_available();
-- true  (已安装 pg_trickle)
-- false (未安装 pg_trickle；视图函数将出错)
```

---

## SPARQL 视图

SPARQL 视图将 SPARQL SELECT 查询编译为 pg_trickle 流表，随着三元组更改自动保持最新。

### create_sparql_view

```sql
pg_ripple.create_sparql_view(
    name     TEXT,
    sparql   TEXT,
    schedule TEXT    DEFAULT '1s',
    decode   BOOLEAN DEFAULT false
) → BIGINT
```

将 SPARQL SELECT 编译为 SQL 并注册 pg_trickle 流表。返回投影列的数量。

- **name** — 视图的唯一标识符（成为 `pg_ripple` schema 中的流表名）
- **sparql** — 有效的 SPARQL SELECT 查询
- **schedule** — pg_trickle 刷新间隔（例如 `'1s'`、`'10s'`、`'1m'`）
- **decode** — 当为 `true` 时，字典 ID 在流表中解码为人类可读的字符串；当为 `false`（默认）时，列包含原始 `BIGINT` ID 以获得最大性能

```sql
-- 创建所有人和他们姓名的视图
SELECT pg_ripple.create_sparql_view(
    'people_names',
    'SELECT ?person ?name WHERE {
       ?person <http://xmlns.com/foaf/0.1/name> ?name
     }',
    '5s',
    true
);

-- 像常规查询表一样查询物化视图
SELECT * FROM pg_ripple.people_names;
```

示例输出（`decode = true`）：

```
          person           |        name
-------------------------+--------------------
 http://example.org/alice | Alice Smith
 http://example.org/bob   | Bob Johnson
 http://example.org/carol | Carol Williams
(3 rows)
```

使用 `decode = false`，列包含原始字典 ID（BIGINT）：

```
      person      |        name
-----------------+--------------------
 4728391847263   | 4728391847264
 4728391847265   | 4728391847266
 4728391847267   | 4728391847268
(3 rows)
```

### drop_sparql_view

```sql
pg_ripple.drop_sparql_view(name TEXT) → BOOLEAN
```

删除流表并删除目录条目。

### list_sparql_views

```sql
pg_ripple.list_sparql_views() → JSONB
```

返回所有已注册 SPARQL 视图的 JSONB 数组，包括名称、原始查询、计划和解码模式。

```sql
SELECT pg_ripple.list_sparql_views();
```

示例输出：

```json
[
  {
    "name": "people_names",
    "sparql": "SELECT ?person ?name WHERE { ?person <http://xmlns.com/foaf/0.1/name> ?name }",
    "schedule": "5s",
    "decode": true,
    "stream_table_name": "pg_ripple.people_names",
    "variables": ["person", "name"]
  },
  {
    "name": "all_students",
    "sparql": "SELECT ?s WHERE { ?s <http://xmlns.com/foaf/0.1/isPrimaryTopicOf> ?doc }",
    "schedule": "10s",
    "decode": false,
    "stream_table_name": "pg_ripple.all_students",
    "variables": ["s"]
  }
]
```

---

## Datalog 视图

Datalog 视图将规则集与目标模式捆绑到一个自刷新流表中。

### create_datalog_view

```sql
pg_ripple.create_datalog_view(
    name          TEXT,
    rules         TEXT,
    goal          TEXT,
    rule_set_name TEXT    DEFAULT 'custom',
    schedule      TEXT    DEFAULT '10s',
    decode        BOOLEAN DEFAULT false
) → BIGINT
```

解析内联 Datalog 规则，将目标查询编译为 SQL，并注册 pg_trickle 流表。返回投影列的数量。

```sql
-- 查看所有推断的祖父母关系，每 10 秒刷新一次
SELECT pg_ripple.create_datalog_view(
    'grandparents',
    '?x <http://example.org/grandparent> ?z :-
       ?x <http://example.org/parent> ?y ,
       ?y <http://example.org/parent> ?z .',
    '?x <http://example.org/grandparent> ?z',
    'family',
    '10s',
    true
);

SELECT * FROM pg_ripple.grandparents;
```

示例输出（从显式父三元组推断）：

```
      ?x       |      ?z
--------------+--------------
 john         | grandpa
 jane         | grandpa
 bob          | grandma
(3 rows)
```

### create_datalog_view_from_rule_set

```sql
pg_ripple.create_datalog_view_from_rule_set(
    name      TEXT,
    rule_set  TEXT,
    goal      TEXT,
    schedule  TEXT    DEFAULT '10s',
    decode    BOOLEAN DEFAULT false
) → BIGINT
```

引用现有命名规则集（之前通过 `load_rules()` 或 `load_rules_builtin()` 加载）而不是提供内联规则。

```sql
-- 加载规则一次
SELECT pg_ripple.load_rules_builtin('rdfs');

-- 使用那些规则创建视图
SELECT pg_ripple.create_datalog_view_from_rule_set(
    'all_types',
    'rdfs',
    '?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?t',
    '30s',
    true
);

SELECT * FROM pg_ripple.all_types;
```

示例输出（包括显式类型 + 推断的 RDFS 子类/域类型）：

```
       ?x        |               ?t
---------------+-------------------------------
 alice         | http://example.org/Person
 bob           | http://example.org/Person
 alice         | http://xmlns.com/foaf/0.1/Agent
 bob           | http://xmlns.com/foaf/0.1/Agent
(4 rows)
```

### drop_datalog_view / list_datalog_views

```sql
pg_ripple.drop_datalog_view(name TEXT) → BOOLEAN
pg_ripple.list_datalog_views() → JSONB
```

与 SPARQL 视图相同的生命周期管理。

```sql
SELECT pg_ripple.list_datalog_views();
```

示例输出：

```json
[
  {
    "name": "grandparents",
    "rule_set": "family",
    "goal": "?x <http://example.org/grandparent> ?z",
    "schedule": "10s",
    "decode": true,
    "stream_table_name": "pg_ripple.grandparents",
    "variables": ["?x", "?z"]
  },
  {
    "name": "all_types",
    "rule_set": "rdfs",
    "goal": "?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?t",
    "schedule": "30s",
    "decode": true,
    "stream_table_name": "pg_ripple.all_types",
    "variables": ["?x", "?t"]
  }
]
```

---

## 扩展垂直分区（ExtVP）

ExtVP 预先计算两个经常共连接的谓词对之间的半连接。当存在时，SPARQL 查询引擎自动检测并使用 ExtVP 表，为星形模式提供 2-10 倍的加速。

### create_extvp

```sql
pg_ripple.create_extvp(
    name      TEXT,
    pred1_iri TEXT,
    pred2_iri TEXT,
    schedule  TEXT DEFAULT '10s'
) → BIGINT
```

创建一个 pg_trickle 流表，其中包含两个谓词 VP 表之间的预计算半连接。返回列数。

```sql
-- 预先计算 foaf:name 和 foaf:knows 之间的连接
SELECT pg_ripple.create_extvp(
    'name_knows',
    '<http://xmlns.com/foaf/0.1/name>',
    '<http://xmlns.com/foaf/0.1/knows>',
    '10s'
);

SELECT * FROM pg_ripple.name_knows;
```

示例输出（两个谓词的主语半连接）：

```
         s
------------------
 alice
 bob
 carol
(3 rows)
```

当 SPARQL 引擎遇到连接这两个谓词的星形模式时，它将使用 ExtVP 表，而不是在查询时连接两个 VP 表。

### drop_extvp / list_extvp

```sql
pg_ripple.drop_extvp(name TEXT) → BOOLEAN
pg_ripple.list_extvp() → JSONB
```

```sql
SELECT pg_ripple.list_extvp();
```

示例输出：

```json
[
  {
    "name": "name_knows",
    "pred1_iri": "<http://xmlns.com/foaf/0.1/name>",
    "pred2_iri": "<http://xmlns.com/foaf/0.1/knows>",
    "pred1_id": 5632187461234,
    "pred2_id": 5632187461245,
    "schedule": "10s",
    "stream_table_name": "pg_ripple.name_knows"
  }
]
```

---

## 目录表

| 表 | 描述 |
|-------|-------------|
| `_pg_ripple.sparql_views` | 名称、原始 SPARQL、生成的 SQL、计划、解码模式、流表名、变量 |
| `_pg_ripple.datalog_views` | 名称、规则、规则集、目标、生成的 SQL、计划、解码模式、流表名、变量 |
| `_pg_ripple.extvp_tables` | 名称、谓词 IRI、谓词 ID、生成的 SQL、计划、流表名 |
| `_pg_ripple.construct_views` | 名称、SPARQL、生成的 SQL、计划、解码模式、模板计数、流表名 |
| `_pg_ripple.describe_views` | 名称、SPARQL、生成的 SQL、计划、解码模式、CBD 策略、流表名 |
| `_pg_ripple.ask_views` | 名称、SPARQL、生成的 SQL、计划、流表名 |

---

## CONSTRUCT 视图（v0.18.0）

CONSTRUCT 视图将 SPARQL CONSTRUCT 查询编译为具有 schema `(s BIGINT, p BIGINT, o BIGINT, g BIGINT)` 的 pg_trickle 流表。行始终反映 CONSTRUCT 输出 — 插入或删除影响 WHERE 模式的三元组会导致流表自动更新。

### create_construct_view

```sql
pg_ripple.create_construct_view(
    name     TEXT,
    sparql   TEXT,
    schedule TEXT    DEFAULT '1s',
    decode   BOOLEAN DEFAULT false
) → BIGINT
```

返回已注册的模板三元组数。流表 `pg_ripple.construct_view_{name}` 是自动创建的。当 `decode = true` 时，还会创建一个伴随视图 `pg_ripple.construct_view_{name}_decoded(s TEXT, p TEXT, o TEXT, g BIGINT)`。

**错误条件：**
- `sparql` 不是 CONSTRUCT 查询 → `"sparql must be a CONSTRUCT query"`
- 模板包含未绑定的变量 → 列出未绑定的变量
- 模板包含空节点 → 建议替换为 IRI 或斯柯伦化

```sql
-- 物化推断的类型三元组：所有 foaf:Person 也是 foaf:Agent
SELECT pg_ripple.create_construct_view(
    'inferred_agents',
    'CONSTRUCT { ?person <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
                          <http://xmlns.com/foaf/0.1/Agent> }
     WHERE { ?person <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
                     <http://xmlns.com/foaf/0.1/Person> }',
    '5s'
);

-- 物化的三元组被存储为 BIGINT ID。
SELECT * FROM pg_ripple.construct_view_inferred_agents LIMIT 5;
```

### drop_construct_view

```sql
pg_ripple.drop_construct_view(name TEXT) → void
```

删除流表并删除目录条目。如果存在，也会删除 `_decoded` 视图。

### list_construct_views

```sql
pg_ripple.list_construct_views() → JSONB
```

返回所有已注册 CONSTRUCT 视图的 JSONB 数组。

```json
[
  {
    "name": "inferred_agents",
    "sparql": "CONSTRUCT { ... } WHERE { ... }",
    "schedule": "5s",
    "decode": false,
    "template_count": 1,
    "stream_table": "pg_ripple.construct_view_inferred_agents",
    "created_at": "2026-04-16T10:00:00Z"
  }
]
```

---

## DESCRIBE 视图（v0.18.0）

DESCRIBE 视图将 SPARQL DESCRIBE 查询编译为具有 schema `(s BIGINT, p BIGINT, o BIGINT, g BIGINT)` 的 pg_trickle 流表，物化描述资源的简洁有界描述（CBD）。

`pg_ripple.describe_strategy` GUC 受到尊重：`cbd`（仅出向弧，默认）或 `scbd`（对称 — 出向 + 入向弧）。

### create_describe_view

```sql
pg_ripple.create_describe_view(
    name     TEXT,
    sparql   TEXT,
    schedule TEXT    DEFAULT '1s',
    decode   BOOLEAN DEFAULT false
) → void
```

流表 `pg_ripple.describe_view_{name}` 是自动创建的。当 `decode = true` 时，还会创建一个伴随 `_decoded` 视图。

```sql
-- 物化有关具有给定名称的人的所有三元组
SELECT pg_ripple.create_describe_view(
    'named_people',
    'DESCRIBE ?person WHERE {
       ?person <http://xmlns.com/foaf/0.1/name> "Alice"
     }',
    '10s'
);

SELECT * FROM pg_ripple.describe_view_named_people;
```

### drop_describe_view

```sql
pg_ripple.drop_describe_view(name TEXT) → void
```

### list_describe_views

```sql
pg_ripple.list_describe_views() → JSONB
```

---

## ASK 视图（v0.18.0）

ASK 视图将 SPARQL ASK 查询编译为单行流表，schema 为 `(result BOOLEAN, evaluated_at TIMESTAMPTZ)`。当基础模式的可满足性发生变化时，`result` 列会翻转 — 用于实时约束监视器和仪表板指示器。

### create_ask_view

```sql
pg_ripple.create_ask_view(
    name     TEXT,
    sparql   TEXT,
    schedule TEXT DEFAULT '1s'
) → void
```

流表 `pg_ripple.ask_view_{name}` 是自动创建的。

```sql
-- 监视是否有任何人缺少名称（约束违规指示器）
SELECT pg_ripple.create_ask_view(
    'person_missing_name',
    'ASK { ?person <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
                   <http://xmlns.com/foaf/0.1/Person> .
           FILTER NOT EXISTS { ?person <http://xmlns.com/foaf/0.1/name> ?name } }',
    '5s'
);

-- 检查实时结果。
SELECT result, evaluated_at FROM pg_ripple.ask_view_person_missing_name;
--  result | evaluated_at
-- --------+---------------------------
--  f      | 2026-04-16 10:05:00+00
```

### drop_ask_view

```sql
pg_ripple.drop_ask_view(name TEXT) → void
```

### list_ask_views

```sql
pg_ripple.list_ask_views() → JSONB
```

---

## 何时使用视图

| 用例 | 推荐 |
|----------|----------------|
| 带有几个关键指标的仪表板 | SPARQL 视图，`decode = true`，计划 `'5s'` |
| 增量 RDFS/OWL 物化 | 从内置规则集创建 Datalog 视图 |
| 星形模式繁重的工作负载 | 对前 5-10 个谓词对使用 ExtVP |
| 即席探索 | 直接使用 `sparql()` — 不需要视图 |
| 写多读少 | 避免使用视图（刷新成本超过读取节省）|