# Cookbook：与 Wikidata 和 DBpedia 联邦

**目标。** 在单个查询中组合来自本地知识图和公共 SPARQL 端点（Wikidata、DBpedia 或任何其他）的数据。无需 ETL，无需复制数据，无需定时同步。

**为什么使用 pg_ripple。** SPARQL 1.1 `SERVICE` 联邦是内置的。基于成本的规划器将过滤器下推到远程端点，断路器处理超时，处理时间缓存避免每次查询都命中远程端点。

**首次结果时间。** 约 10 分钟。

---

## 第 1 步 — 注册远程端点

注册一次端点可让规划器分配成本估算，并让健康监控器跟踪其可用性。

```sql
SELECT pg_ripple.register_federation_endpoint(
    endpoint := 'https://dbpedia.org/sparql',
    label    := 'DBpedia'
);

SELECT pg_ripple.register_federation_endpoint(
    endpoint        := 'https://query.wikidata.org/sparql',
    label           := 'Wikidata',
    -- 可选：固定 TLS 证书以防止 MITM。
    pin_fingerprint := NULL   -- 例如 'sha256:AA:BB:CC:...'
);
```

未注册时，`SERVICE` 仍然有效，但获得通用成本估算且没有健康监控。

## 第 2 步 — 加载本地事实

```sql
SELECT pg_ripple.load_turtle($TTL$
@prefix ex:   <https://example.org/people/> .
@prefix dbr:  <http://dbpedia.org/resource/> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .

ex:alice  <http://schema.org/name>  "Alice Smith" .
ex:alice  owl:sameAs  dbr:Alice_Smith_(scientist) .

ex:bob    <http://schema.org/name>  "Bob Jones" .
ex:bob    owl:sameAs  dbr:Robert_Jones_(chemist) .
$TTL$);
```

`owl:sameAs` 链接告诉规划器您的本地实体对应于 DBpedia 资源。当 `pg_ripple.sameas_reasoning = on`（默认值）时，SERVICE 查询可以使用任一 IRI。

## 第 3 步 — 一个简单的联邦查询

从 DBpedia 检索您本地人员的出生日期：

```sql
SELECT * FROM pg_ripple.sparql($$
    PREFIX schema: <http://schema.org/>
    PREFIX dbo:    <http://dbpedia.org/ontology/>
    PREFIX owl:    <http://www.w3.org/2002/07/owl#>

    SELECT ?name ?birthDate WHERE {
        -- 本地：我们知道谁？
        ?local  schema:name    ?name ;
                owl:sameAs     ?remote .

        -- 远程：从 DBpedia 获取他们的出生日期。
        SERVICE <https://dbpedia.org/sparql> {
            ?remote  dbo:birthDate  ?birthDate .
        }
    }
$$);
```

规划器将 `?remote dbo:birthDate ?birthDate` 发送到 DBpedia，其中 `?remote` 的值从本地结果绑定 — 这是 **bound-join 联邦**，是最有效的模式。

## 第 4 步 — 多源富化

用 Wikidata（药物适应症）和本地专有存储（临床试验数据）富化医学图：

```sql
SELECT * FROM pg_ripple.sparql($$
    PREFIX ex:  <https://example.org/>
    PREFIX wd:  <http://www.wikidata.org/entity/>
    PREFIX wdt: <http://www.wikidata.org/prop/direct/>
    PREFIX owl: <http://www.w3.org/2002/07/owl#>

    SELECT ?drugName ?indication ?trialId WHERE {
        -- 本地存储：药物及其 Wikidata 交叉链接。
        ?drug  ex:name       ?drugName ;
               owl:sameAs    ?wikidataDrug .

        -- Wikidata：批准的适应症。
        SERVICE <https://query.wikidata.org/sparql> {
            ?wikidataDrug  wdt:P2175  ?indicationEntity .
            ?indicationEntity  wdt:P1813  ?indication .   -- 简称
        }

        -- 本地存储：临床试验 ID。
        OPTIONAL { ?drug  ex:clinicalTrial  ?trialId }
    }
    ORDER BY ?drugName
$$);
```

## 第 5 步 — 检查端点健康和缓存

```sql
-- 哪些端点现在可达？
SELECT endpoint, label, last_ping_ms, is_healthy
FROM pg_ripple.federation_endpoint_health()
ORDER BY last_ping_ms;

-- 缓存了多少远程结果？
SELECT * FROM pg_ripple.federation_cache_stats();

-- 清除缓存以强制新获取。
SELECT pg_ripple.reset_cache_stats();
```

## 第 6 步 — 检查联邦计划

在运行重型联邦查询之前，检查规划器打算如何执行它：

```sql
SELECT pg_ripple.explain_sparql($$
    PREFIX schema: <http://schema.org/>
    PREFIX dbo:    <http://dbpedia.org/ontology/>
    SELECT ?name ?birthDate WHERE {
        ?local schema:name ?name .
        SERVICE <https://dbpedia.org/sparql> {
            ?local dbo:birthDate ?birthDate .
        }
    }
$$, analyze := false);
```

健康的联邦计划显示 **BoundJoin** 节点而不是 **IndependentService** 节点。BoundJoin 将本地值批量放入 VALUES 子句并发送单个远程请求；IndependentService 为每行执行 SERVICE 子句 — 对于大型本地结果集要慢得多。

如果您在应该 bound-join 的查询上看到 IndependentService，请确保共享变量（示例中的 `?local`）在查询文本中由 SERVICE 子句*之前*的本地模式绑定。

---

## 断路器和超时

| GUC | 默认 | 效果 |
|---|---|---|
| `pg_ripple.federation_timeout_ms` | `5000` | 在这么多毫秒后中止远程调用 |
| `pg_ripple.federation_retry_count` | `2` | 在跳闸断路器之前重试这么多次 |
| `pg_ripple.federation_circuit_open_ms` | `60000` | 跳闸后等待这么久再重试 |

当断路器打开时，使用该 SERVICE 端点的查询返回该子模式的空绑定并继续使用本地结果 — 它们不会完全失败。`pg_ripple.federation_endpoint_health()` 视图显示哪些断路器是打开的。

---

## 安全注意事项

只注册您信任并控制的端点（或众所周知的公共端点）。联邦电路将查询文本发送到远程端点 — 可能包括来自本地存储的字面值。对于敏感数据，使用本地 SPARQL-over-HTTP 缓存（或 [pg_ripple_http](../features/apis-and-integration.md)）作为代理，而不是直接联邦到公共端点。

---

## 另请参阅

- [联邦（SPARQL SERVICE）](../user-guide/sql-reference/federation.md) — 完整 SQL 函数参考。
- [向量联邦](../user-guide/vector-federation.md) — 将向量搜索联邦到 Weaviate / Qdrant / Pinecone。
- [SPARQL 查询调试器](../user-guide/explain-sparql.md)