# 全文搜索

有时对知识图的正确查询不是*查找具有此确切标签的实体*，而是*查找其标签、描述或注释提及这些词的任何实体*。pg_ripple 使用 PostgreSQL 的内置全文搜索机制 — `tsvector`、`tsquery`、GIN 索引 — 用于此目的，作为 SQL 函数和 SPARQL 过滤器公开。

---

## 设置

全文索引是按谓词选择加入的。告诉 pg_ripple 哪些字面量值谓词可搜索：

```sql
SELECT pg_ripple.create_fts_index(
    predicate := '<http://www.w3.org/2000/01/rdf-schema#label>',
    config    := 'english'
);

SELECT pg_ripple.create_fts_index('<http://purl.org/dc/elements/1.1/title>',  'english');
SELECT pg_ripple.create_fts_index('<https://schema.org/description>',          'english');
```

在后台，pg_ripple 在相关的 VP 表上维护一个生成的 `tsvector` 列和一个 GIN 索引。插入和更新自动流过。

`config` 参数是任何 PostgreSQL 文本搜索配置名称（`english`、`simple`、`spanish` 等）。对于没有词干分析器的语言或专有名词繁重的数据，请使用 `simple`。

---

## 从 SQL 搜索

```sql
-- 标签与查询匹配的所有主语。
SELECT * FROM pg_ripple.fts_search(
    predicate := '<http://www.w3.org/2000/01/rdf-schema#label>',
    query     := 'machine & learning'
);
```

`query` 参数遵循 PostgreSQL `tsquery` 语法：`&` 表示 AND，`|` 表示 OR，`!` 表示 NOT，`<->` 表示相邻性。请参阅 [PostgreSQL FTS 文档](https://www.postgresql.org/docs/current/textsearch-controls.html)。

---

## 从 SPARQL 搜索

`pg:fts()` SPARQL 过滤器函数在实体的字面量值与 tsquery 匹配时返回 true。它与其他图模式自然组合：

```sparql
PREFIX pg:    <http://pg-ripple.io/fn/>
PREFIX rdfs:  <http://www.w3.org/2000/01/rdf-schema#>
PREFIX schema:<https://schema.org/>

SELECT ?paper ?title WHERE {
    ?paper a            <https://example.org/ScholarlyArticle> ;
           schema:author <https://example.org/alice> ;
           rdfs:label    ?title .
    FILTER(pg:fts(?title, "graph & neural & networks"))
}
```

仅当匹配的字面量位于已建立索引的谓词上时，`pg:fts()` 才会触发。否则它返回 `false`（并发出调试级别的日志条目）。

---

## 排名

对于排名结果，`pg_ripple.fts_search_ranked()` 返回每行的 FTS 排名分数：

```sql
SELECT subject, rank
FROM pg_ripple.fts_search_ranked(
    predicate := '<https://schema.org/description>',
    query     := 'sustainable & supply & chain'
)
ORDER BY rank DESC
LIMIT 25;
```

排名是 `ts_rank_cd` 使用默认规范化。要覆盖规范化，请使用 `fts_search_ranked(predicate, query, normalisation := 32)`。

---

## 组合 FTS 和向量搜索

FTS 捕获精确的词法匹配；向量搜索捕获释义。组合它们可以提高召回率：

```sql
SELECT entity_iri, fused_score
FROM pg_ripple.hybrid_search(
    sparql := 'SELECT ?p WHERE { ?p a <https://example.org/Paper> .
                                 FILTER(pg:fts(?p, "neural & networks")) }',
    text   := 'deep learning architectures',
    k      := 25,
    alpha  := 0.4
);
```

这种模式 — *FTS 用于必须包含的关键字，向量用于语义扩展* — 是 RAG 管道中最有用的技巧之一。

---

## 何时**不**使用 FTS

- 数据使用受控词汇表（`skos:Concept` 分类、代码列表）进行结构化 — 请改用[精确 SPARQL 模式](querying-with-sparql.md)。
- 您的数据集很小（< 10K 标签）。简单的 `LIKE '%keyword%'` 就可以了。
- 您只查询单一语言。对于这种情况，PostgreSQL 的 `simple` 配置比语言感知配置更快。

---

## 另请参阅

- [向量和混合搜索](vector-and-hybrid-search.md) — FTS 的语义对应物。
- [PostgreSQL 全文搜索文档](https://www.postgresql.org/docs/current/textsearch.html)