# Cookbook：以知识图为依据的聊天机器人

**目标。** 构建一个 LLM 驱动的问答助手，使用**您的知识图**作为真实来源，而不是 LLM 的训练数据。幻觉急剧下降；每个答案都可以追溯到真实的三元组。

**为什么使用 pg_ripple。** 一次 SQL 函数调用（`rag_context()`）返回一个 LLM 就绪的提示，融合向量召回、图扩展和（可选的）已执行 SPARQL。无需同步的向量存储，无需部署编排框架。

**首次结果时间。** 约 10 分钟。

---

## 管道

```
   用户问题                                   LLM 响应
        │                                                ▲
        ▼                                                │
  ┌─────────────────────────────────────────────────────┴───────┐
  │  应用程序代码（Python / TS / Go）                           │
  │      ┌─────────────────────────────────────────────────┐   │
  │      │  context = pg_ripple.rag_context(question, k=8) │   │
  │      │  prompt  = SYSTEM_PROMPT + context + question   │   │
  │      │  answer  = openai.chat(prompt)                  │   │
  │      └─────────────────────────────────────────────────┘   │
  └─────────────────────────────────────────────────────────────┘
                                                            ▲
                                                            │
                                                       LLM API
```

`rag_context()` 在单个 PostgreSQL 事务中执行每个 RAG 管道都必须做的四件事：

1. 嵌入用户问题（对您的嵌入端点的 HTTP 调用）。
2. HNSW 余弦搜索以检索前 *k* 个匹配实体。
3. SPARQL 图扩展以收集每个实体的 1 跳邻域。
4. 将上下文组装为 JSON-LD 或纯文本。

---

## 第 1 步 — 配置 LLM 和嵌入端点

```sql
ALTER SYSTEM SET pg_ripple.embedding_api_url     = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.embedding_api_key_env = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.embedding_model       = 'text-embedding-3-small';

ALTER SYSTEM SET pg_ripple.llm_endpoint          = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.llm_api_key_env       = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.llm_model             = 'gpt-4o';

SELECT pg_reload_conf();
```

API 密钥在调用时从环境变量中读取，永远不会存储在数据库中。

## 第 2 步 — 加载并嵌入您的知识图

```sql
-- 加载您拥有的任何内容。可以是 Turtle 文件、R2RML 映射、GraphRAG 输出…
SELECT pg_ripple.load_turtle_file('/data/medical_kb.ttl');

-- （推荐）具体化 OWL RL 推理，以便 'subClassOf' 链可见。
SELECT pg_ripple.load_rules_builtin('owl-rl');
SELECT pg_ripple.infer('owl-rl');

-- 嵌入每个有标签的实体。每次大加载后运行一次。
SELECT pg_ripple.embed_entities();
```

对于非常大的图，设置 `pg_ripple.use_graph_context = on`，使每个实体的嵌入输入包括其 1 跳邻居 — 对于标签本身不明确的实体的召回率显著提高。

## 第 3 步 — 为问题检索上下文

```sql
SELECT pg_ripple.rag_context(
    question := 'What drugs treat moderate hypertension?',
    k        := 8
);
```

返回一个 TEXT 块，准备放入提示中：

```
You are answering using the following knowledge graph context:

ENTITY: <https://example.org/lisinopril>
  type: Drug, ACEInhibitor
  rdfs:label: "Lisinopril"
  ex:treats: Hypertension, HeartFailure
  ex:contraindication: Pregnancy

ENTITY: <https://example.org/amlodipine>
  type: Drug, CalciumChannelBlocker
  ...
```

## 第 4 步 — 与 NL→SPARQL 组合以获得事实式答案

对于答案是小的、明确定义的集合（"how many"、"list the"、"who is"）的问题，让 LLM 还生成一个 SPARQL 查询来提取精确的答案：

```sql
SELECT pg_ripple.sparql_from_nl(
    'How many drugs in the knowledge graph treat hypertension?'
);
-- 返回一个已解析、已验证的 SPARQL 字符串。

-- 然后执行它。
SELECT * FROM pg_ripple.sparql(
    pg_ripple.sparql_from_nl(
        'How many drugs in the knowledge graph treat hypertension?'
    )
);
```

当设置 `pg_ripple.llm_endpoint` 时，`rag_context()` 自动执行此操作：组装的上下文包括向量检索的邻域**和**自动生成的 SPARQL 查询的结果行。

## 第 5 步 — 接入到您的应用程序

```python
import psycopg
import openai

SYSTEM_PROMPT = """
You are a medical information assistant. Answer ONLY using the
knowledge graph context provided. If the answer is not in the
context, say "I do not have that information." Cite IRIs of
entities you reference.
""".strip()

def answer(question: str) -> str:
    with psycopg.connect("...") as conn:
        cur = conn.cursor()
        cur.execute("SELECT pg_ripple.rag_context(%s, 8)", (question,))
        context = cur.fetchone()[0]
    prompt = f"{SYSTEM_PROMPT}\n\n=== CONTEXT ===\n{context}\n\n=== QUESTION ===\n{question}"
    resp = openai.chat.completions.create(
        model="gpt-4o",
        messages=[{"role": "user", "content": prompt}],
        temperature=0.0,
    )
    return resp.choices[0].message.content
```

---

## 调整

| 杠杆 | 作用 | 何时更改 |
|---|---|---|
| `k`（rag_context）| 包括的实体数 | 实际更多实体 = 更丰富的上下文，更多 token |
| `pg_ripple.use_graph_context` | 使用邻域嵌入实体 | 改善模糊标签的召回率 |
| `pg_ripple.llm_include_shapes` | 在 NL→SPARQL 提示中包括 SHACL 形状 | 在具有许多谓词的谓词上提高查询准确性 |
| Few-shot 示例 | 通过 `pg_ripple.add_llm_example()` 添加 | 领域特定词汇表需要 5–10 个示例 |

---

## 为什么这优于单独的向量数据库

- **一个事务。** 加载新三元组和更新嵌入是原子发生的。崩溃后没有半更新的向量存储。
- **无漂移。** 单独的向量数据库有自己的模式；几个月后，两者发生分歧。这里有一个模式，一个真实来源。
- **默认多租户。** 应用 [graph RLS](../features/multi-tenant-graphs.md)，相同的 `rag_context()` 调用返回租户范围的上下文，无需应用级过滤。
- **审计就绪。** 启用 [`audit_log_enabled`](../reference/audit-log.md)，每个 RAG 时间 UPDATE 都被捕获。单独的向量数据库无法做到这一点。

---

## 另请参阅

- [AI 概述](../features/ai-overview.md)
- [RAG 管道参考](../user-guide/rag-pipeline.md)
- [NL → SPARQL](../features/nl-to-sparql.md)