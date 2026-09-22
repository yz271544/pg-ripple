# 全文搜索

pg_ripple 使用 PostgreSQL 内置的 GIN `tsvector` 索引对 RDF 字面量对象提供全文搜索。这使得可以在字符串值谓词上进行快速自由文本查询，而无需扫描整个字典。

## fts_index

```sql
pg_ripple.fts_index(predicate TEXT) RETURNS BIGINT
```

为 `_pg_ripple.dictionary` 表上的字符串字面量对象创建 GIN `tsvector` 索引。返回谓词的字典 ID。

`predicate` 参数接受原始 IRI 字符串和带尖括号的 N-Triples 形式。

```sql
-- 为 dc:description 谓词创建 FTS 索引
SELECT pg_ripple.fts_index('http://purl.org/dc/elements/1.1/description');

-- 也接受 N-Triples 形式
SELECT pg_ripple.fts_index('<http://purl.org/dc/elements/1.1/description>');
```

索引创建为 `IF NOT EXISTS`，因此对同一谓词多次调用 `fts_index` 是安全的。

> **索引范围**：GIN 索引涵盖字典中的所有纯字符串字面量（`kind = 2`）。`fts_search` 函数通过 VP 表 JOIN 限制谓词的结果。

### 示例：为所有关键谓词建立索引

```sql
-- 加载一些数据
SELECT pg_ripple.load_ntriples('
    <https://example.org/paper1> <https://schema.org/abstract> "A study of RDF triplestores" .
    <https://example.org/paper2> <https://schema.org/abstract> "PostgreSQL extensions for graph data" .
');

-- 为抽象谓词建立索引
SELECT pg_ripple.fts_index('<https://schema.org/abstract>');

-- 现在搜索
SELECT * FROM pg_ripple.fts_search('RDF | triplestore', '<https://schema.org/abstract>');
```

---

## fts_search

```sql
pg_ripple.fts_search(query TEXT, predicate TEXT)
    RETURNS TABLE(s TEXT, p TEXT, o TEXT)
```

对指定谓词的字面量对象执行全文搜索。将匹配的三元组作为 N-Triples 格式的字符串返回。

- `query` — PostgreSQL `tsquery` 表达式（见下文）
- `predicate` — 谓词 IRI，带或不带尖括号

```sql
SELECT s, o
FROM pg_ripple.fts_search('semantic & query', '<https://schema.org/abstract>');
```

### tsquery 语法

PostgreSQL `tsquery` 使用 `&`（与）、`|`（或）、`!`（非）和 `<->`（短语接近）：

| 查询 | 匹配 |
|---|---|
| `'rdf'` | 包含 "rdf" 的文档 |
| `'rdf & sparql'` | 同时包含两者的文档 |
| `'rdf | sparql'` | 包含任一个的文档 |
| `'!relational'` | 不包含 "relational" 的文档 |
| `'rdf <-> store'` | 紧邻 "store" 的 "rdf" 文档 |

所有术语都会自动词干提取（默认情况下为英语词干分析器） — 搜索 `"querying"` 也匹配 `"query"` 和 `"queries"`。

### 在没有先调用 fts_index 的情况下搜索

`fts_search` 即使没有先调用 `fts_index` 也可以工作，方法是对连接到 VP 表的字典进行顺序扫描。对于大型存储，请先为经常搜索的每个谓词调用 `fts_index`。

### 返回列

| 列 | 内容 |
|---|---|
| `s` | 以 N-Triples 形式的主语 IRI |
| `p` | 以 N-Triples 形式的谓词 IRI |
| `o` | 以 N-Triples 形式的字面量值 |

---

## 语言配置

默认文本配置是 `'english'`，它应用英语词干提取和停用词删除。如果您的数据是另一种语言，您可以手动创建索引：

```sql
-- 示例：自定义谓词上的法语语言索引
CREATE INDEX my_fr_fts ON _pg_ripple.dictionary
    USING GIN (to_tsvector('french', value))
    WHERE kind = 2;
```

内置的 `fts_index` 和 `fts_search` 始终使用 `'english'`。多语言支持计划在未来版本中提供。