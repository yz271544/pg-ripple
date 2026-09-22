# Cookbook：LLM 工作流 — 自然语言到知识图答案

**目标。** 构建一个完整的端到端工作流，其中用户用简单的英语提问，LLM 生成 SPARQL 查询，查询对您的知识图运行，结果作为流畅的自然语言答案返回给用户。

**为什么使用 pg_ripple。** 所有四个步骤在一个 SQL 调用链中运行 — 从 NL 问题到 SPARQL 到结果到摘要答案。无需单独的编排层。

**首次结果时间。** 约 10 分钟。

---

## 四步工作流

```
   用户问题（NL）
          │
          ▼
   sparql_from_nl()   ← LLM 从您的模式 + few-shot 示例生成 SPARQL
          │
          ▼
   sparql()           ← 针对 VP 表执行 SPARQL
          │
          ▼
   sparql_construct_turtle()  ← （可选）将上下文子图作为 Turtle 获取
          │
          ▼
   LLM 将结果汇总为流畅的答案
```

pg_ripple 原生处理步骤 1–3。步骤 4 是您的 LLM 调用。

---

## 第 1 步 — 设置 LLM 端点

```sql
ALTER SYSTEM SET pg_ripple.llm_endpoint    = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.llm_api_key_env = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.llm_model       = 'gpt-4o';

-- （可选）嵌入实体以进行向量辅助的 SPARQL 生成。
ALTER SYSTEM SET pg_ripple.embedding_api_url     = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.embedding_api_key_env = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.embedding_model       = 'text-embedding-3-small';

SELECT pg_reload_conf();
```

## 第 2 步 — 加载示例图

```sql
SELECT pg_ripple.load_turtle($TTL$
@prefix ex:     <https://example.org/> .
@prefix schema: <https://schema.org/> .

ex:Alice  a schema:Person ;
    schema:name     "Alice Smith" ;
    schema:jobTitle "Senior Engineer" ;
    schema:knows    ex:Bob .

ex:Bob  a schema:Person ;
    schema:name     "Bob Jones" ;
    schema:jobTitle "Data Scientist" ;
    schema:knows    ex:Carol .

ex:Carol  a schema:Person ;
    schema:name     "Carol Kim" ;
    schema:jobTitle "VP Engineering" .
$TTL$);
```

## 第 3 步 — 添加 few-shot 示例（推荐）

领域特定的示例显著提高 `sparql_from_nl` 的准确性：

```sql
SELECT pg_ripple.add_llm_example(
    'Who does Alice know?',
    'PREFIX ex: <https://example.org/>
     PREFIX schema: <https://schema.org/>
     SELECT ?name WHERE {
         ex:Alice schema:knows ?p .
         ?p schema:name ?name .
     }'
);

SELECT pg_ripple.add_llm_example(
    'What is Bob''s job title?',
    'PREFIX ex: <https://example.org/>
     PREFIX schema: <https://schema.org/>
     SELECT ?title WHERE {
         ex:Bob schema:jobTitle ?title .
     }'
);
```

## 第 4 步 — 生成并运行 SPARQL 查询

```sql
-- 从自然语言问题生成 SPARQL。
SELECT pg_ripple.sparql_from_nl('Who does Bob know?') AS generated_sparql;
```

返回：

```sparql
PREFIX ex:     <https://example.org/>
PREFIX schema: <https://schema.org/>
SELECT ?name WHERE {
    ex:Bob schema:knows ?person .
    ?person schema:name ?name .
}
```

执行它：

```sql
SELECT result->>'name' AS person_name
FROM pg_ripple.sparql(
    pg_ripple.sparql_from_nl('Who does Bob know?')
);
```

## 第 5 步 — 获取相关子图作为上下文

对于更丰富的答案，将整个子图传递给 LLM，而不仅仅是结果行：

```sql
SELECT pg_ripple.sparql_construct_turtle($Q$
    CONSTRUCT { ?s ?p ?o }
    WHERE {
        ?s ?p ?o .
        FILTER(?s IN (<https://example.org/Alice>, <https://example.org/Bob>))
    }
$Q$) AS context_turtle;
```

Turtle 块与 SPARQL 结果行和用户问题一起进入 LLM 提示。

## 第 6 步 — 与混合向量搜索组合

对于无法准确命名相关实体的问题（"谁在做机器学习"），将 NL→SPARQL 路径与向量相似度预过滤器组合：

```sql
-- 首先嵌入所有实体（每个加载周期一次）。
SELECT pg_ripple.embed_entities();

-- 混合：查找相似实体，然后通过 SPARQL 扩展。
SELECT * FROM pg_ripple.sparql($$
    PREFIX schema: <https://schema.org/>
    SELECT ?name ?title WHERE {
        ?p schema:name ?name ;
           schema:jobTitle ?title .
        FILTER(?p IN (
            SELECT s FROM pg_ripple.similar_entities('machine learning engineer', k := 5)
        ))
    }
$$);
```

## 第 7 步 — 完整的 Python 应用程序

```python
import psycopg, openai

DB_URL = "postgresql://..."
client = openai.OpenAI()

SYSTEM_PROMPT = """
You are a helpful assistant. You have been given the result of a SPARQL query against
a knowledge graph, as well as the subgraph context as Turtle. Answer the user's
question concisely using only the provided data. If the data does not contain the
answer, say so.
""".strip()

def answer_question(question: str) -> str:
    with psycopg.connect(DB_URL) as conn:
        cur = conn.cursor()

        # 步骤 A：生成 SPARQL 并运行它。
        cur.execute(
            "SELECT jsonb_agg(row_to_json(r)) FROM pg_ripple.sparql(pg_ripple.sparql_from_nl(%s)) r",
            (question,)
        )
        sparql_rows = cur.fetchone()[0] or []

        # 步骤 B：获取上下文子图（可选但提高答案质量）。
        # 为简单起见，我们提取结果中每个 ?s 的 1 跳邻域。
        turtle_context = ""
        if sparql_rows:
            subjects = [row.get("s") or row.get("p") or "" for row in sparql_rows if row]
            if subjects:
                subj_filter = ", ".join(f"<{s}>" for s in subjects if s.startswith("http"))
                if subj_filter:
                    cur.execute(
                        "SELECT pg_ripple.sparql_construct_turtle("
                        "  'CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o . FILTER(?s IN (" + subj_filter + ")) }'"
                        ")"
                    )
                    turtle_context = cur.fetchone()[0] or ""

    prompt = (
        f"{SYSTEM_PROMPT}\n\n"
        f"=== SPARQL RESULT ===\n{sparql_rows}\n\n"
        f"=== GRAPH CONTEXT (Turtle) ===\n{turtle_context}\n\n"
        f"=== QUESTION ===\n{question}"
    )
    resp = client.chat.completions.create(
        model="gpt-4o",
        messages=[{"role": "user", "content": prompt}],
        temperature=0.0,
    )
    return resp.choices[0].message.content

if __name__ == "__main__":
    print(answer_question("Who does Bob know and what are their job titles?"))
```

---

## 何时使用每种方法

| 方法 | 最适合 |
|---|---|
| 仅 `sparql_from_nl()` | 精确的事实问题（"how many"、"list all"） |
| 仅 `rag_context()` | 开放、多跳的问题（"如何解释 X"） |
| 两者组合（此食谱）| 需要精确度和广度的生产代理 |
| `sparql_from_nl()` + Turtle 上下文 | 答案需要丰富叙述解释的场景 |

---

## 另请参阅

- [NL → SPARQL](../features/nl-to-sparql.md) — `sparql_from_nl` 深度剖析和调整指南。
- [AI 概述](../features/ai-overview.md) — 所有 AI 功能的决策树。
- [Cookbook：基于知识图的聊天机器人](grounded-chatbot.md) — 更简单的单遍变体。
- [Cookbook：SPARQL 修复工作流](sparql-repair.md) — 当 NL→SPARQL 失败时该怎么办。
- [AI 代理集成](../features/ai-agent-integration.md) — LangChain / LlamaIndex 接线。

---

## 词汇表感知的 NL→SPARQL（v0.119.0）

当您的数据集使用自定义本体或领域特定的词汇表时，`sparql_from_nl()` 可以自动将词汇表元数据包括在 LLM 系统提示中，从而提高本体丰富的知识图的翻译准确性。

### 启用包注入

```sql
-- 在 NL→SPARQL 提示中启用词汇表包元数据（默认：on）
SET pg_ripple.nl_sparql_include_bundles = on;
```

启用后，`sparql_from_nl()` 查询三元组存储的：

- `skos:prefLabel` — 词汇表术语的首选标签
- `dcterms:title` — 数据集和资源标题
- `schema:name` — Schema.org 类型名称
- `foaf:name` — FOAF 人/组织名称

此元数据作为基础上下文添加到 LLM 提示中，帮助模型生成与您的实际数据匹配的属性 URI。

### 为性能禁用

对于不使用这些词汇表谓词的数据集，禁用以跳过额外的 SPI 查询：

```sql
SET pg_ripple.nl_sparql_include_bundles = off;
```