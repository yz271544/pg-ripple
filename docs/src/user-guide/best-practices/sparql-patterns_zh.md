# SPARQL 模式

针对 pg_ripple 编写高效 SPARQL 查询的最佳实践。

## 星形模式 — 让规划器折叠连接

当单个主语有多个谓词时，将它们表达为同一 WHERE 子句中的单独三元组模式。SQL 生成器将它们折叠为具有多个表连接的单次扫描：

```sparql
-- 推荐：星形模式 — 所有谓词共享同一主语变量
SELECT ?person ?name ?age WHERE {
  ?person <ex:worksAt> <ex:acme> .
  ?person <ex:name>    ?name .
  OPTIONAL { ?person <ex:age> ?age }
}

-- 避免：每个谓词单独的子查询（强制额外连接）
SELECT ?person ?name ?age WHERE {
  { SELECT ?person WHERE { ?person <ex:worksAt> <ex:acme> } }
  { SELECT ?person ?name WHERE { ?person <ex:name> ?name } }
}
```

## 过滤下推

针对字典编码常量的 SPARQL FILTER 表达式在编译时在 SQL WHERE 子句中求值。常量在发出 SQL 之前被编码为 `BIGINT` — 查询时不会发生字典查找。

为了获得最佳性能，请尽早（在三元组模式中靠近绑定变量的位置）应用过滤器，而不是在包装子查询的晚期应用。

## OPTIONAL 与 INNER JOIN

当结果行即使未匹配可选模式也应出现时，使用 `OPTIONAL`。这会编译为 `LEFT JOIN`。当所有结果都必须绑定变量时，使用普通三元组模式（内连接行为）。

```sparql
-- 所有具有 worksAt 的人员，加上可用的姓名（LEFT JOIN）
SELECT ?person ?name WHERE {
  ?person <ex:worksAt> ?company .
  OPTIONAL { ?person <ex:name> ?name }
}

-- 仅同时具有 worksAt 和 name 的人员（INNER JOIN）
SELECT ?person ?name WHERE {
  ?person <ex:worksAt> ?company .
  ?person <ex:name>    ?name .
}
```

## 计划缓存命中率

已编译的 SPARQL→SQL 计划在后端的 LRU 缓存中按后端缓存。相同的查询字符串（包括空白）将命中缓存。为了最大化缓存命中率：
- 通过将常量绑定到 `VALUES` 内联数据来参数化查询，而不是将它们作为字面量字符串嵌入查询文本中
- 通过 `pg_ripple.plan_cache_size` 为查询工作负载保持足够大的缓存

使用 `sparql_explain()` 检查缓存效率——一旦计划被缓存，重复调用相同查询会立即返回。

## 属性路径配方

### 传递闭包（跟踪所有跳跃）

```sparql
SELECT ?target WHERE {
  <ex:alice> <ex:knows>+ ?target
}
```

编译为 `WITH RECURSIVE` CTE。指定深度限制以避免在密集图上失控的查询：

```sql
SET pg_ripple.max_path_depth = 10;
```

### 包括起始节点（零或多次跳跃）

```sparql
SELECT ?target WHERE {
  <ex:alice> <ex:follows>* ?target
}
-- 返回 alice 自身以及所有可达节点
```

### 多谓词路径（备选）

```sparql
SELECT ?contact WHERE {
  <ex:alice> (<ex:knows>|<ex:follows>) ?contact
}
```

### 序列（双跳连接）——

```sparql
SELECT ?friend_of_friend WHERE {
  <ex:alice> <ex:knows>/<ex:knows> ?friend_of_friend
}
```

`/` 运算符编译为链式连接 — spargean 将中间表示符表示为匿名空白节点，pg_ripple 通过应用 equi-join 约束来处理。

### 逆向路径（查找指向节点的对象）

```sparql
SELECT ?who WHERE {
  ?who ^<ex:knows> <ex:bob>
}
-- 等价于：<ex:bob> 是对象，?who 是主语
```

## 资源耗尽保护

对于输入查询无法完全信任的面向用户的应用程序：

```sql
-- 设置每个会话的深度上限
SET pg_ripple.max_path_depth = 15;

-- 设置每个查询的时间限制
SET statement_timeout = '5s';
```

两个设置都可以在连接池 `after_connect` 钩子或行级安全策略中应用。深度上限包含在计划缓存键中，不会导致跨会话污染。

## 用于多值查找的 VALUES

`VALUES` 编译为 SQL 内联数据，对于查找已知资源列表非常有效：

```sparql
SELECT ?person ?name WHERE {
  VALUES ?person { <ex:alice> <ex:bob> <ex:carol> }
  ?person <ex:name> ?name .
}
```

这比将值嵌入为各个 FILTER 子句更友好于缓存，因为查询结构保持不变，只有 VALUES 行发生变化。

## 使用 sparql_explain 调试

在归咎于 pg_ripple 之前，请始终检查生成的 SQL：

```sql
SELECT pg_ripple.sparql_explain(
    'SELECT ?name WHERE { ?p <ex:name> ?name . FILTER(?name = "Alice") }',
    false
);
```

查找：
- 期望了专用 VP 表却出现 `vp_rare` 表扫描 — 谓词可能尚未被提升
- 缺少 `WHERE` 子句条件 — FILTER 可能未能编码其常量
- 属性路径中出现额外的 `UNION ALL` 分支 — 对于 `*` 和 `?` 运算符是预期的

---

## 使用 HTTP 端点

`pg_ripple_http` 伴生服务公开了一个标准 SPARQL 端点，可与任何兼容 SPARQL 的工具或库一起使用。

### Python（SPARQLWrapper）

```python
from SPARQLWrapper import SPARQLWrapper, JSON

sparql = SPARQLWrapper("http://localhost:7878/sparql")
sparql.setQuery("""
    SELECT ?name WHERE {
        ?person <https://example.org/name> ?name
    } LIMIT 10
""")
sparql.setReturnFormat(JSON)
results = sparql.query().convert()

for result in results["results"]["bindings"]:
    print(result["name"]["value"])
```

### Java（Apache Jena）

```java
import org.apache.jena.query.*;

String endpoint = "http://localhost:7878/sparql";
String queryStr = "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10";
Query query = QueryFactory.create(queryStr);

try (QueryExecution qexec = QueryExecutionFactory.sparqlService(endpoint, query)) {
    ResultSet results = qexec.execSelect();
    ResultSetFormatter.out(System.out, results, query);
}
```

### curl

```bash
# URL 编码的 GET
curl -G http://localhost:7878/sparql \
  --data-urlencode "query=SELECT ?s WHERE { ?s ?p ?o } LIMIT 5"

# 使用 SPARQL body 的 POST
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-query" \
  -d "SELECT ?s WHERE { ?s ?p ?o } LIMIT 5"

# SPARQL Update
curl -X POST http://localhost:7878/sparql \
  -H "Content-Type: application/sparql-update" \
  -d "INSERT DATA { <ex:s> <ex:p> \"hello\" }"
```

---

## CONSTRUCT 视图与 SELECT 视图（v0.18.0）

CONSTRUCT 视图和 SELECT 视图都是 pg_trickle 流表，随着三元组的更改而保持最新。根据下游消费者的需要进行选择。

| 考虑事项 | SELECT 视图 | CONSTRUCT 视图 |
|---------------|-------------|----------------|
| 输出形状 | 表格（列 = SPARQL 变量） | 三元组（s, p, o, g BIGINT） |
| 最适合 | 仪表板、API、SQL 连接 | 推理、反规范化、RDF 导出 |
| 模板计数 | 每个解决方案 1 行 | 每个解决方案 N 行（N = 模板大小） |
| `decode = true` | 解码每个变量列 | 将 s, p, o 解码为 TEXT |

### 物化推理结果

使用 CONSTRUCT 视图物化 RDFS/OWL 蕴含，而无需运行 Datalog。对于简单的单跳模式，这会更快：

```sql
-- 物化 rdfs:subClassOf 继承单跳
SELECT pg_ripple.create_construct_view(
    'subclass_instances',
    'CONSTRUCT { ?i <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?super }
     WHERE {
       ?i   <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>     ?sub .
       ?sub <http://www.w3.org/2000/01/rdf-schema#subClassOf>      ?super
     }',
    '5s'
);
```

对于多跳推理（传递闭包），请改用具有 `rdfs` 内置规则集的 Datalog 视图。

### 将 ASK 视图用作实时约束监控

ASK 视图维护一个布尔结果，随着数据变化而翻转。适用于：

- 作为触发器运行过于昂贵的 SHACL 样式基数检查
- 仪表板"健康指标"灯
- 应用程序侧事件触发器（查询流表）

```sql
-- 当任何订单超过 24 小时未发货时发出警报
SELECT pg_ripple.create_ask_view(
    'stale_orders',
    'ASK { ?order <https://schema.org/orderStatus>
                  <https://schema.org/OrderProcessing> .
           FILTER NOT EXISTS { ?order <https://schema.org/estimatedDelivery> ?d } }',
    '30s'
);

-- 应用程序查询以下内容：
SELECT result FROM pg_ripple.ask_view_stale_orders;
```

当 `result` 从 `false` 翻转为 `true` 时，表示违反约束。使用 PostgreSQL NOTIFY/LISTEN 或 pg_logical 复制槽将更改推送给应用程序订阅者。