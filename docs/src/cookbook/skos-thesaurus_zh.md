# PostgreSQL 中的 SKOS 词库管理

本 cookbook 章节展示如何使用 v0.98.0 中添加的 W3C 兼容的 SKOS 推理规则在 pg_ripple 中加载、查询和验证 SKOS 分类法。

## 什么是 SKOS？

SKOS（Simple Knowledge Organization System）是 W3C 标准词汇表，用于词库、分类法、分类方案和主题标题系统。SKOS 分类法由 `skos:Concept` 节点组成，这些节点通过 `skos:broader`/`skos:narrower` 层次结构、`skos:related` 关联链接链接，并带有 `skos:prefLabel` 标签。

## 加载 SKOS 规则集

```sql
-- 加载所有 28 个 W3C SKOS 推理规则（S7–S45）。
SELECT pg_ripple.load_builtin_rules('skos');

-- 或使用命名包 API 进行版本化激活：
SELECT pg_ripple.load_datalog_bundle('skos');
```

加载后，pg_ripple 自动推断：

- `skos:broaderTransitive` 和 `skos:narrowerTransitive` 传递闭包
- 逆 `skos:narrower`/`skos:broader` 对
- `skos:related` 对称性
- 映射谓词的子属性层次结构（broadMatch、exactMatch 等）
- 标签子属性（`prefLabel` → `rdfs:label`）

## 加载词库

```sql
-- 加载 Turtle 编码的词库数据。
SELECT pg_ripple.load_turtle($$
@prefix skos: <http://www.w3.org/2004/02/skos/core#> .
@prefix ex:   <http://example.org/thesaurus/> .

ex:Animals a skos:ConceptScheme ;
    skos:hasTopConcept ex:Mammals .

ex:Mammals a skos:Concept ;
    skos:prefLabel "Mammals"@en ;
    skos:inScheme ex:Animals .

ex:Cats a skos:Concept ;
    skos:prefLabel "Cats"@en ;
    skos:broader ex:Mammals .

ex:Dogs a skos:Concept ;
    skos:prefLabel "Dogs"@en ;
    skos:broader ex:Mammals .

ex:Cats skos:related ex:Dogs .
$$);

-- 运行推理以具体化传递闭包。
SELECT pg_ripple.infer('skos');
```

## 使用 SPARQL 查询层次结构

```sql
-- 查找 Cats 的所有更广泛的传递祖先。
SELECT * FROM pg_ripple.sparql($$
    SELECT ?ancestor WHERE {
        <http://example.org/thesaurus/Cats>
            <http://www.w3.org/2004/02/skos/core#broaderTransitive> ?ancestor .
    }
$$);

-- 查找 Mammals 子树中的所有概念（narrowerTransitive 闭包）。
SELECT * FROM pg_ripple.sparql($$
    SELECT ?child WHERE {
        <http://example.org/thesaurus/Mammals>
            <http://www.w3.org/2004/02/skos/core#narrowerTransitive> ?child .
    }
$$);

-- 跨方案映射：查找 exactMatch 对应项。
SELECT * FROM pg_ripple.sparql($$
    SELECT ?concept ?match WHERE {
        ?concept <http://www.w3.org/2004/02/skos/core#exactMatch> ?match .
    }
$$);

## SQL 帮助器函数

五个 SQL 帮助器函数提供对常见词库操作的便捷访问，无需编写 SPARQL：

```sql
-- 遍历 broaderTransitive 闭包。
SELECT * FROM pg_ripple.skos_ancestors(
    'http://example.org/thesaurus/Cats'
);
-- 返回：(ancestor_iri, depth)

-- 遍历 narrowerTransitive 闭包。
SELECT * FROM pg_ripple.skos_descendants(
    'http://example.org/thesaurus/Mammals'
);
-- 返回：(descendant_iri, depth)

-- 查找英文首选标签。
SELECT pg_ripple.skos_label(
    'http://example.org/thesaurus/Cats',
    'en'
);
-- 返回："Cats"

-- 查找所有相关概念。
SELECT * FROM pg_ripple.skos_related(
    'http://example.org/thesaurus/Cats'
);
-- 返回：(related_iri, relation) — 例如 Dogs, skos:related

-- 查找兄弟概念（共享公共 skos:broader 父级）。
SELECT * FROM pg_ripple.skos_siblings(
    'http://example.org/thesaurus/Cats'
);
-- 返回：(sibling_iri, shared_broader_iri) — 例如 Dogs, Mammals
```

## 启用 SKOS-XL 扩展标签

SKOS-XL 允许将结构化元数据附加到标签。pg_ripple 的 SKOS-XL 规则集自动将 `skosxl:Label` 实例投影到纯 `skos:prefLabel` 三元组：

```sql
SELECT pg_ripple.load_builtin_rules('skosxl');
SELECT pg_ripple.infer('skosxl');
```

加载后，具有 `<label> skosxl:literalForm "text"@en` 的三元组 `<concept> skosxl:prefLabel <label>` 将自动产生 `<concept> skos:prefLabel "text"@en`。

## 运行完整性检查

`"skos-integrity"` 形状包从 W3C SKOS 规范验证 10 个结构性完整性条件：

```sql
-- 激活完整性形状包（自动加载 skos-transitive）。
SELECT pg_ripple.load_shape_bundle('skos-integrity');

-- 运行验证。
SELECT * FROM pg_ripple.validate_skos();
-- 返回：(violation_id, subject, message)
-- 示例违规：SKOS-IC-05：具有相同语言标签的多个 prefLabel。
```

10 个验证器涵盖：

| ID | W3C 规则 | 约束 |
|---|---|---|
| SKOS-IC-01 | S9 | ConceptScheme 和 Concept 是不相交的 |
| SKOS-IC-02 | S13 | prefLabel 和 altLabel 不得共享字面值+语言 |
| SKOS-IC-03 | S13 | prefLabel 和 hiddenLabel 不得共享字面值+语言 |
| SKOS-IC-04 | S13 | altLabel 和 hiddenLabel 不得共享字面值+语言 |
| SKOS-IC-05 | S14 | 每个概念的每种语言最多一个 prefLabel |
| SKOS-IC-06 | S27 | 关联和传递广义是不相交的 |
| SKOS-IC-07 | S37 | 集合和概念是不相交的 |
| SKOS-IC-08 | S37 | 集合和概念方案是不相交的 |
| SKOS-IC-09 | S46 | exactMatch 和 broadMatch 是不相交的 |
| SKOS-IC-10 | S46 | exactMatch 和 relatedMatch 是不相交的 |

## 命名包 API

当您需要版本化的、机器可检查的包跟踪时，请使用 `load_datalog_bundle` 而不是 `load_builtin_rules`：

```sql
SELECT pg_ripple.load_datalog_bundle('skos');
SELECT pg_ripple.load_datalog_bundle('skos-transitive');

-- 检查什么处于活动状态。
SELECT * FROM pg_ripple.active_datalog_bundles;
-- 返回：(bundle_name, bundle_version, loaded_at, named_graph)
```

## 性能说明

- 通过 `WITH RECURSIVE … CYCLE` 进行 `skos:broaderTransitive` 闭包具体化在首次具体化时，对于 1,000 个概念约需 200 ms，对于 100,000 个概念约需 8 s。
- 后续 IVM 更新（启用时）每个新的 `skos:broader` 三元组约需 10 ms。
- `skos_ancestors()` 对于深度 ≤ 10 运行实时 `WITH RECURSIVE` 查询约需 5 ms，使用具体化的 `broaderTransitive` 三元组作为基础。

## 另请参阅

- [SKOS 参考](https://www.w3.org/TR/skos-reference/) — W3C 推荐标准
- [Datalog 参考](../reference/datalog.md) — 内置 RDFS 和 OWL RL 规则支持
- [SQL 函数参考](../reference/sql-functions.md) — 所有 pg_ripple SQL 函数