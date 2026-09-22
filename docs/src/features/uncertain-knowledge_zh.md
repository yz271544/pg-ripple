# 不确定知识：概率 Datalog、模糊 SPARQL 和软 SHACL

pg_ripple v0.87.0 引入了**不确定知识引擎** — 一套用于处理不精确、概率或模糊数据的推理功能。

---

## 概述

真实世界的知识图包含各种可靠性的数据。从科学论文中提取的事实与从社交媒体抓取的事实具有不同的置信度。pg_ripple 现在允许您：

1. **使用概率权重（`@weight`）注释 Datalog 规则**，以便派生的事实携带置信度分数。
2. **使用 SPARQL 通过 `pg:confidence()` 扩展函数查询置信度**。
3. **使用 `pg:confPath()` 通过模糊相似性过滤器遍历图**。
4. **使用 `pg:fuzzy_match()` 和 `pg:token_set_ratio()` 模糊匹配字符串**。
5. **使用每形状严重性权重对 SHACL 验证结果进行评分**。
6. **导出带有 RDF\* 置信度注释的 Turtle**。

---

## 概率 Datalog（`@weight`）

向任何 Datalog 规则添加 `@weight(0.8)` 注释，以声明派生的事实的置信度上限为 0.8。引擎使用**噪声 OR 组合**传播置信度：

```
parent(X, Y) :- father(X, Y).           @weight(1.0)
parent(X, Y) :- mother(X, Y).           @weight(1.0)
ancestor(X, Z) :- parent(X, Z).         @weight(0.9)
ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z). @weight(0.85)
```

通过以下方式启用：

```sql
SET pg_ripple.probabilistic_datalog = on;
```

置信度分数存储在 `_pg_ripple.confidence` 中：

```sql
SELECT statement_id, confidence, model
FROM   _pg_ripple.confidence
LIMIT  10;
```

### 概率求值的 GUC

| GUC | 默认 | 描述 |
|-----|---------|-------------|
| `pg_ripple.probabilistic_datalog` | `off` | 启用 `@weight` 规则处理 |
| `pg_ripple.prob_datalog_cyclic` | `off` | 允许对循环规则集进行近似求值 |
| `pg_ripple.prob_datalog_max_iterations` | `100` | 循环求值的最大迭代次数 |
| `pg_ripple.prob_datalog_convergence_delta` | `0.001` | 提前退出阈值 |
| `pg_ripple.prob_datalog_cyclic_strict` | `off` | 将非收敛从 WARNING 提升为 ERROR |

---

## `pg:confidence()` — SPARQL 置信度查找

```sparql
PREFIX pg: <http://pg-ripple.org/functions/>

SELECT ?s ?p ?o ?conf WHERE {
  ?s ?p ?o .
  BIND(pg:confidence(?s, ?p, ?o) AS ?conf)
  FILTER(?conf > 0.7)
}
```

返回三元组 `(?s, ?p, ?o)` 在所有模型中的最大置信度。当不存在置信度行时返回 `1.0`。

---

## 模糊 SPARQL 函数

这些函数需要 `CREATE EXTENSION IF NOT EXISTS pg_trgm;`。

### `pg:fuzzy_match(a, b)`

返回两个字符串字面量之间的 trigram 相似度（`similarity()`）。

```sparql
PREFIX pg: <http://pg-ripple.org/functions/>

SELECT ?label WHERE {
  ?entity rdfs:label ?label .
  FILTER(pg:fuzzy_match(?label, "Alice Smith") > 0.6)
}
```

### `pg:token_set_ratio(a, b)`

返回词集相似度（`word_similarity()`） — 更好地用于子字符串匹配。

```sparql
FILTER(pg:token_set_ratio(?label, "Smith") > 0.5)
```

### `pg:confPath(predicate, threshold)` — 置信度属性路径

```sparql
SELECT ?x WHERE {
  <http://example.org/alice> <pg:conf_path/http://example.org/knows/0.8> ?x
}
```

以最小置信度阈值 0.8 遍历 `knows` 谓词。

### GUC

| GUC | 默认 | 描述 |
|-----|---------|-------------|
| `pg_ripple.default_fuzzy_threshold` | `0.7` | 未明确指定时的默认阈值 |

---

## 软 SHACL 评分

pg_ripple 可以计算图的**加权数据质量评分**，而不是通过/失败的验证：

```sql
SELECT pg_ripple.shacl_score('http://example.org/data');
-- 返回 [0.0, 1.0] 中的 float8，其中 1.0 = 完全合规
```

使用 `sh:severityWeight` 注释形状以控制它们的贡献：

```turtle
ex:MyShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:name ; sh:minCount 1 ] ;
    sh:severityWeight "2.0"^^xsd:decimal .
```

> **扩展说明 (STD-05, v0.92.0)**：`sh:severityWeight` 是 W3C SHACL Core 规范的 **pg_ripple 特定扩展**。它不是由 <https://www.w3.org/TR/shacl/> 定义的。正在考虑向 [W3C SHACL 社区组](https://www.w3.org/community/shacl/) 提交社区提案；有关进度，请参阅 [GitHub 讨论](https://github.com/trickle-labs/pg-ripple/discussions)。该注释对于 pg_ripple 1.x API 线是稳定的。

评分历史记录在 `_pg_ripple.shacl_score_log` 中：

```sql
SELECT pg_ripple.log_shacl_score('http://example.org/data');
SELECT * FROM _pg_ripple.shacl_score_log ORDER BY measured_at DESC;
```

---

## 使用置信度加载三元组

```sql
SELECT pg_ripple.load_triples_with_confidence(
    '<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .',
    confidence => 0.85,
    format => 'ntriples'
);
```

---

## 使用置信度注释导出（RDF\*）

```sql
SET pg_ripple.export_confidence = on;
SELECT pg_ripple.export_turtle_with_confidence('http://example.org/data');
```

返回带有 `<< s p o >> pg:confidence "0.85"^^xsd:float .` 注释的 Turtle。

---

## PROV-O 置信度传播

设置 `pg_ripple.prov_confidence = on` 以启用从 `pg:sourceTrust` 谓词的自动置信度传播 — 从低信任源派生的三元组继承较低的置信度。

---

## HTTP API（pg_ripple_http）

| 方法 | 路径 | 描述 |
|--------|------|-------------|
| `POST` | `/confidence/load` | 使用置信度加载三元组 |
| `GET` | `/confidence/shacl-score?graph=<IRI>` | 计算 SHACL 质量评分 |
| `GET` | `/confidence/shacl-report?graph=<IRI>` | 评分的违规报告 |
| `POST` | `/confidence/vacuum` | 清除孤立的置信度行 |

---

## 垃圾收集

孤立的置信度行（其 `statement_id` 不再存在于任何 VP 表中）会被清除：

1. **自动**在每次 HTAP 合并周期期间。
2. **按需**：`SELECT pg_ripple.vacuum_confidence();`

---

## 循环概率规则的收敛保证（v0.90.0）

当 `pg_ripple.prob_datalog_cyclic = on` 时，pg_ripple 将噪声 OR 组合迭代到不动点。

$$c_i^{(k+1)} = 1 - \prod_{j \in \text{parents}(i)} (1 - w_{ij} \cdot c_j^{(k)})$$

由于噪声 OR 在 [0, 1] 上是**单调**的（添加更多证据只能增加置信度，永远不会降低），并且序列 $\{c_i^{(k)}\}$ 是非递减的且以 1.0 为上界，根据 Knaster–Tarski 不动点定理，迭代收敛到概率传播算子的**最小不动点**。

> **扩展说明 (STD-03, v0.91.0)**：pg_ripple 的噪声 OR 置信度组合是实现概率 Datalog 语义的 pg_ripple 特定扩展。它**不**由 W3C RDF 或 SPARQL 规范定义。数学基础是：
>
> De Raedt, L., Kimmig, A., & Toivonen, H. (2007). ProbLog: A probabilistic Prolog and its application in link discovery. *Proceedings of IJCAI 2007*, pp. 2468–2473.
> <https://ijcai.org/proceedings/2007/2>

收敛速度取决于循环深度和置信度值；在循环中具有接近 1.0 置信度的程序可能收敛缓慢。`prob_datalog_max_iterations` GUC（默认 100）和 `prob_datalog_convergence_delta` GUC（默认 1e-6）控制终止。

```sql
-- 针对深度循环程序调整收敛
SET pg_ripple.prob_datalog_max_iterations = 500;
SET pg_ripple.prob_datalog_convergence_delta = 1e-8;
```

## 进一步阅读

- [博客：概率 Datalog](https://github.com/trickle-labs/pg-ripple/blob/main/blog/probabilistic-datalog.md) — pg_ripple 中的噪声 OR、置信度传播和软推理
- [博客：不确定知识](https://github.com/trickle-labs/pg-ripple/blob/main/blog/uncertain-knowledge.md) — 概率推理的实用模式