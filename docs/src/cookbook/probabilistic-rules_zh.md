# Cookbook：用于软约束的概率规则

**目标。** 您希望规则*传播置信度*，而不仅仅是事实。*"如果 A 与 B 相似于置信度 0.9，而 B 与 C 相似于置信度 0.85，那么 A 与 C 相似于 ≥ 0.85 × 0.9 的置信度。"* 经典 Datalog 无法做到这一点；它处理真/假。**Lattice Datalog** 可以。

**为什么选择 pg_ripple。** 内置 lattice（`min`、`max`、`set`、`interval`），并允许您注册自定义的 lattice。推理的不动点位于 lattice 上而不是布尔值上。

**第一次结果的时间。** ~10 分钟。

---

## 直觉

标准 Datalog：事实*在关系中*或*不在*。有一个真值：派生的。

Lattice Datalog：事实具有*关联的值* —— 置信度、成本、概率、时间间隔。lattice 告诉引擎如何*组合*同一事实的多个派生。对于置信度，我们使用 `min`：派生链仅与其最弱链接一样自信。

---

## 第 1 步 —— 选择 lattice

对于置信度传播，内置的 `min` lattice 正是我们想要的。（lattice 顶部 = 1.0 = 确定的。底部 = 0.0 = 未知。同一事实的多个派生采用*最强* —— 即弱链接置信度的*最小值*。）

如果您更喜欢 max-of-min 语义（最强单链条获胜），在 `min` 链上构建 `max` lattice。捆绑的 `min` lattice 是最常见的起点。

## 第 2 步 —— 使用 RDF-star 编码置信度

置信度是*三元组的属性*，因此 RDF-star 是自然的编码：

```sql
SELECT pg_ripple.load_turtle($TTL$
@prefix ex:  <https://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:alice ex:similarTo ex:bob   .
ex:bob   ex:similarTo ex:carol .
ex:carol ex:similarTo ex:dan   .

<< ex:alice ex:similarTo ex:bob   >> ex:confidence "0.90"^^xsd:decimal .
<< ex:bob   ex:similarTo ex:carol >> ex:confidence "0.85"^^xsd:decimal .
<< ex:carol ex:similarTo ex:dan   >> ex:confidence "0.95"^^xsd:decimal .
$TTL$);
```

## 第 3 步 —— 编写 lattice 规则

```sql
SELECT pg_ripple.load_rules($RULES$
# 传递相似性：置信度是链的最小值。
?x ex:transSimilarTo ?y :- ?x ex:similarTo ?y .

?x ex:transSimilarTo ?z :-
    ?x ex:similarTo      ?y ,
    ?y ex:transSimilarTo ?z .

# Lattice 类型绑定：每个派生的 ex:transSimilarTo 携带一个置信度。
@lattice ex:transSimilarTo confidence min .
$RULES$, 'similarity');

SELECT pg_ripple.infer_lattice('similarity', 'min');
```

`@lattice` 指令告诉引擎：*每当派生 `ex:transSimilarTo` 三元组时，其置信度是主体原子置信度的 `min`*。然后引擎将 lattice 作为连接运算符迭代到不动点。

## 第 4 步 —— 查询

```sql
SELECT * FROM pg_ripple.sparql($$
    PREFIX ex: <https://example.org/>
    SELECT ?z ?conf WHERE {
        <https://example.org/alice> ex:transSimilarTo ?z .
        << <https://example.org/alice> ex:transSimilarTo ?z >> ex:confidence ?conf .
    }
    ORDER BY DESC(?conf)
$$);
```

```
?z              ?conf
ex:bob          0.90
ex:carol        0.85   （0.90、0.85 的最小值）
ex:dan          0.85   （0.90、0.85、0.95 的最小值）
```

请注意，`ex:dan` 保持 `0.85` 的瓶颈，而不是 `0.95 × 0.85 × 0.90`。这正是 `min` 语义给您的 —— *最弱链接*。如果您想要*乘法*传播，请注册自定义 lattice（下一节）。

## 第 5 步 —— 用于乘法置信度的自定义 lattice

```sql
-- 乘法组合两个置信度的 PostgreSQL 聚合。
CREATE OR REPLACE FUNCTION conf_mul(state DOUBLE PRECISION, val DOUBLE PRECISION)
RETURNS DOUBLE PRECISION
LANGUAGE plpgsql IMMUTABLE AS $$ BEGIN RETURN COALESCE(state, 1.0) * val; END; $$;

CREATE AGGREGATE prob_join(DOUBLE PRECISION) (
    SFUNC = conf_mul, STYPE = DOUBLE PRECISION, INITCOND = '1.0'
);

SELECT pg_ripple.create_lattice(
    name    := 'probability',
    join_fn := 'prob_join',
    bottom  := '0.0'
);

SELECT pg_ripple.infer_lattice('similarity', 'probability');
```

现在 `?dan` 的置信度是 `0.90 × 0.85 × 0.95 = 0.726` —— 链衰减，而不是最弱链接。

---

## Lattice Datalog 是正确工具的情况

- **置信度传播**（此配方）。
- **最短路径**（边权重上的 `min` lattice）。
- **最大带宽**（边容量上的 `max` lattice，然后沿链 `min`）。
- **时间区间推理**（`interval` lattice —— *"所有这些都为真的时间段"*）。
- **出处半环**（见证集上的自定义 lattice）。

当规则的主体仅具有布尔合取而头部仅需要*真/假*时，经典 Datalog 更简单。Lattice Datalog 仅在与事实关联的值重要时才具有其复杂性。

---

## 另请参阅

- [基于 Lattice 的 Datalog 参考](../reference/lattice-datalog.md)
- [推理与演绎](../features/reasoning-and-inference.md)