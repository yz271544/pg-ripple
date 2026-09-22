# RAG 管道 — `rag_context()`

pg_ripple v0.50.0 引入了 `pg_ripple.rag_context()` — 一个 SQL 函数，可从您的知识图谱组装检索增强生成（RAG）上下文字符串，可用作 LLM 系统提示或用户消息。

---

## 函数签名

```sql
pg_ripple.rag_context(
    question    TEXT,
    k           INT DEFAULT 10
) RETURNS TEXT
```

### 参数

| 参数 | 默认值 | 描述 |
|-----------|---------|-------------|
| `question` | （必填） | 要检索上下文的自然语言问题 |
| k | `10` | 上下文中要包含的最大实体数 |

---

## 工作原理

该函数完全在 PostgreSQL 内部执行一个五步管道：

```
question TEXT
    │
    ▼  第 1 步：嵌入问题
   _pg_ripple.embeddings 上的 HNSW 余弦搜索
    │
    ▼  第 2 步：向量召回
   Top-k 最相似的实体
    │
    ▼  第 3 步：SPARQL 图扩展
   每个实体的 1 跳邻域（标签、类型、属性、邻居）
    │
    ▼  第 4 步：组装上下文
   JSON-LD 片段连接为纯文本上下文字符串
    │
    ▼  第 5 步（可选）：NL→SPARQL 执行
   如果设置了 pg_ripple.llm_endpoint，则执行 sparql_from_nl(question)
   并附加 SPARQL 结果集
    │
    ▼
context TEXT
```

---

## 先决条件

`rag_context()` 需要：

1. **安装 pgvector 扩展**（`CREATE EXTENSION vector`）
2. **`pg_ripple.pgvector_enabled = on`**（默认：`on`）
3. 通过 `pg_ripple.embed_entities()` 或手动加载到 `_pg_ripple.embeddings` 中的实体加载了嵌入

当 pgvector 缺失或嵌入表为空时，函数会优雅降级，并返回空字符串和 WARNING 而非引发 ERROR。

---

## 示例

### 基本上下文检索

```sql
-- 检索问题的上下文（返回纯文本）
SELECT pg_ripple.rag_context(
    'What drugs are used to treat headaches?',
    k := 5
);
```

### 将上下文用作 LLM 系统提示

```sql
-- 组装上下文并传递给 sparql_from_nl
SELECT pg_ripple.sparql_from_nl(
    'What drugs treat headaches? Use the context: ' ||
    pg_ripple.rag_context('What treats headaches?', k := 5)
);
```

### 端到端 RAG 与自动 SPARQL 执行

当配置了 `pg_ripple.llm_endpoint` 时，`rag_context()` 自动调用 `sparql_from_nl()` 并附加 SPARQL 查询结果：

```sql
-- 设置 LLM 端点（每个会话一次或在 postgresql.conf 中设置）
SET pg_ripple.llm_endpoint = 'https://api.openai.com/v1';
SET pg_ripple.llm_api_key_env = 'OPENAI_API_KEY';

-- rag_context 现在包括向量上下文 + SPARQL 结果
SELECT pg_ripple.rag_context('Who are the key authors in the knowledge graph?', k := 10);
```

---

## 调优

### 调整 `k`

更大的 `k` 返回更多上下文但增加 token 使用。大多数用例从 `k = 5`–`10` 开始。

```sql
-- 窄上下文：k=3
SELECT pg_ripple.rag_context('What is aspirin?', k := 3);

-- 宽上下文：k=20
SELECT pg_ripple.rag_context('Give me a broad overview of drug interactions', k := 20);
```

### 嵌入新鲜度

上下文质量取决于嵌入是否是最新的。定期或在批量加载后运行 `embed_entities()`：

```sql
-- 在默认图中重新嵌入所有实体
SELECT pg_ripple.embed_entities(graph_iri := NULL, model := NULL, batch_size := 100);
```

### GUC 设置

| GUC | 默认值 | 效果 |
|-----|---------|--------|
| `pg_ripple.pgvector_enabled` | `on` | 设置为 `off` 禁用 pgvector（返回空上下文） |
| `pg_ripple.llm_endpoint` | `''` | 设置后，启用第 5 步（NL→SPARQL） |
| `pg_ripple.llm_model` | `'gpt-4o'` | 第 5 步的 LLM 模型名称 |

---

## 输出格式

每个实体的上下文字符串具有以下结构：

```
Entity: https://example.org/aspirin
Label: aspirin
Context:
{
  "label": "aspirin",
  "types": ["https://pharma.example/Drug"],
  "properties": [
    {"predicate": "...", "object": "..."}
  ],
  "neighbors": ["https://pharma.example/Ibuprofen"]
}

---

Entity: https://example.org/ibuprofen
...
```

当第 5 步执行 SPARQL 查询时，结果会附加：

```
---

SPARQL Result for: What treats headaches?
[{"?drug": "<https://pharma.example/aspirin>"}]
```

---

## 优雅降级

| 条件 | 行为 |
|-----------|-----------|
| pgvector 未安装 | WARNING + 空字符串 |
| `pgvector_enabled = off` | WARNING + 空字符串 |
| 嵌入表为空 | 空字符串（无 WARNING） |
| `llm_endpoint` 未设置 | 仅步骤 1–4；无 SPARQL 执行 |