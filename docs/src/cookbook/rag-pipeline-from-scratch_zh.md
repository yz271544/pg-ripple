# 从头开始构建 RAG 管道

本配方引导您使用 pg_ripple 的混合 SPARQL + 向量搜索构建完整的检索增强生成（RAG）管道。最后，您将拥有一个使用您的知识图谱作为上下文来源来回答自然语言问题的系统。

## 架构

```mermaid
flowchart LR
    Q[用户问题] --> E[嵌入问题\npg_ripple.embed_text]
    E --> V[向量相似性搜索\n_pg_ripple.embeddings]
    V --> F[SPARQL 过滤器\nopt. 约束]
    F --> R[检索到的实体\n+ 图上下文]
    R --> P[构建 LLM 提示]
    P --> L[LLM API 调用\ngpt-4o / llama3]
    L --> A[答案 + 引用]
```

## 先决条件

- pg_ripple v0.49.0 或更高版本（NL→SPARQL），v0.28.0+（向量嵌入）
- 已安装 pgvector 扩展
- OpenAI API 密钥（或任何 OpenAI 兼容端点）
- Python 3.11+ 及 `psycopg`、`openai`、`requests`

```bash
pip install psycopg openai requests
```

---

## 第 1 步：启用嵌入

```sql
-- 设置嵌入端点（OpenAI 兼容）
ALTER SYSTEM SET pg_ripple.embedding_api_url = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.embedding_api_key = 'sk-...';
ALTER SYSTEM SET pg_ripple.embedding_model = 'text-embedding-3-small';
ALTER SYSTEM SET pg_ripple.embedding_dimensions = 1536;
ALTER SYSTEM SET pg_ripple.auto_embed = on;
SELECT pg_reload_conf();
```

对于本地 Ollama 嵌入：

```sql
ALTER SYSTEM SET pg_ripple.embedding_api_url = 'http://localhost:11434/v1';
ALTER SYSTEM SET pg_ripple.embedding_model = 'nomic-embed-text';
ALTER SYSTEM SET pg_ripple.embedding_dimensions = 768;
ALTER SYSTEM SET pg_reload_conf();
```

---

## 第 2 步：加载您的知识图谱

```sql
-- 加载领域数据
SELECT pg_ripple.load_turtle($$
  @prefix ex: <https://pharma.example/> .
  @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
  @prefix schema: <https://schema.org/> .

  ex:aspirin a ex:Drug ;
    rdfs:label "Aspirin" ;
    ex:treats ex:headache, ex:fever, ex:inflammation ;
    ex:class ex:NSAID ;
    ex:approvedBy ex:FDA .

  ex:ibuprofen a ex:Drug ;
    rdfs:label "Ibuprofen" ;
    ex:treats ex:pain, ex:inflammation ;
    ex:class ex:NSAID .

  ex:headache rdfs:label "headache" .
  ex:fever rdfs:label "fever" .
  ex:pain rdfs:label "pain" .
$$, 'https://pharma.example/graph');
```

---

## 第 3 步：嵌入知识图谱

为所有加载的实体触发嵌入生成：

```sql
-- 手动嵌入所有实体（如果加载期间 auto_embed 关闭）
SELECT pg_ripple.embed_all_entities();

-- 检查嵌入覆盖率
SELECT COUNT(*) FROM _pg_ripple.embeddings;
```

或者使用 `auto_embed = on`，后端工作进程会在加载三元组时自动生成嵌入。

---

## 第 4 步：通过 HTTP API 查询

`/rag` 端点处理完整管道：

```bash
# 简单问题
curl -X POST http://localhost:7878/rag \
  -H "Content-Type: application/json" \
  -d '{
    "question": "what drugs treat headaches?",
    "k": 5
  }'
```

响应：

```json
{
  "results": [
    {
      "entity_iri": "https://pharma.example/aspirin",
      "label": "Aspirin",
      "context_json": {
        "label": "Aspirin",
        "types": ["Drug"],
        "properties": [
          {"predicate": "treats", "object": "headache"},
          {"predicate": "class", "object": "NSAID"}
        ],
        "contextText": "Aspirin. Type: Drug, NSAID. Treats: headache, fever, inflammation."
      },
      "distance": 0.08
    }
  ],
  "context": "Aspirin. Type: Drug, NSAID. Treats: headache, fever, inflammation.\n\nIbuprofen. Type: Drug. Treats: pain, inflammation."
}
```

---

## 第 5 步：构建 Python RAG 客户端

```python
import os
import requests
from openai import OpenAI

RAG_ENDPOINT = "http://localhost:7878/rag"
AUTH_TOKEN   = os.environ["PG_RIPPLE_HTTP_AUTH_TOKEN"]
OPENAI_KEY   = os.environ["OPENAI_API_KEY"]

openai = OpenAI(api_key=OPENAI_KEY)

def retrieve(question: str, sparql_filter: str | None = None, k: int = 5) -> dict:
    payload = {"question": question, "k": k}
    if sparql_filter:
        payload["sparql_filter"] = sparql_filter
    r = requests.post(
        RAG_ENDPOINT,
        json=payload,
        headers={"Authorization": f"Bearer {AUTH_TOKEN}"},
        timeout=30,
    )
    r.raise_for_status()
    return r.json()

def answer(question: str, sparql_filter: str | None = None) -> str:
    retrieval = retrieve(question, sparql_filter)
    context = retrieval["context"]

    response = openai.chat.completions.create(
        model="gpt-4o",
        messages=[
            {
                "role": "system",
                "content": (
                    "You are a helpful assistant. Answer the user's question using "
                    "only the following knowledge graph context. Cite entity IRIs "
                    "when making claims.\n\n"
                    f"Context:\n{context}"
                ),
            },
            {"role": "user", "content": question},
        ],
        temperature=0.2,
    )
    return response.choices[0].message.content


# 用法示例
print(answer("What NSAIDs are approved by the FDA?"))
print(answer(
    "Which drugs treat fever?",
    sparql_filter="?entity a <https://pharma.example/Drug>"
))
```

---

## 第 6 步：使用 SPARQL 约束进行过滤

使用 `sparql_filter` 将检索限制到类或子图：

```python
# 仅检索 FDA 批准的药物
answer(
    "What are the safest pain relievers?",
    sparql_filter=(
        "?entity a <https://pharma.example/Drug> . "
        "?entity <https://pharma.example/approvedBy> <https://pharma.example/FDA>"
    )
)
```

---

## 第 7 步：使用 SHACL 进行数据质量

添加 SHACL 约束以确保您的知识图谱包含 RAG 所需的完整信息：

```sql
SELECT pg_ripple.load_shacl($$
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix ex: <https://pharma.example/> .
  @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

  ex:DrugShape a sh:NodeShape ;
    sh:targetClass ex:Drug ;
    sh:property [
      sh:path rdfs:label ;
      sh:minCount 1 ;
      sh:message "Every drug must have a label for RAG context generation"
    ] ;
    sh:property [
      sh:path ex:treats ;
      sh:minCount 1 ;
      sh:message "Every drug must have at least one indication"
    ] .
$$);

-- 验证图
SELECT pg_ripple.validate();
```

---

## 第 8 步：使用图上下文获得更丰富的嵌入

对于语义更丰富的嵌入，请启用图上下文模式：

```sql
ALTER SYSTEM SET pg_ripple.use_graph_context = on;
SELECT pg_reload_conf();

-- 使用图上下文重新嵌入所有实体
SELECT pg_ripple.embed_all_entities(force_recompute := true);
```

使用 `use_graph_context = on`，每个实体都嵌入其完整的 RDF 邻域（序列化为文本），生成的嵌入捕获图结构 —— 而不仅仅是实体标签。

---

## 第 9 步：NL → SPARQL 生成（高级）

对于复杂查询，请使用 NL → SPARQL 端点直接生成并执行 SPARQL：

```sql
-- 配置 LLM
ALTER SYSTEM SET pg_ripple.llm_endpoint = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.llm_model    = 'gpt-4o';
ALTER SYSTEM SET pg_ripple.llm_api_key_env = 'OPENAI_API_KEY';
SELECT pg_reload_conf();

-- 从自然语言生成并运行 SPARQL
SELECT pg_ripple.sparql_from_nl(
  'Find all NSAIDs approved by the FDA that treat headaches'
);
```

---

## 生产检查清单

- [ ] 从密钥管理器设置 `pg_ripple.embedding_api_key`（而不是 `ALTER SYSTEM`）
- [ ] 在所有 HTTP 端点上启用 `PG_RIPPLE_HTTP_AUTH_TOKEN`
- [ ] 配置 `pg_ripple.sparql_max_rows` 以限制失控查询
- [ ] 监视 `pg_ripple_embedding_queue_depth` Prometheus gauge —— 如果 > 10,000 则增加工作线程
- [ ] 使用 `pg_ripple.export_max_rows` 防止大型 RAG 上下文溢出
- [ ] 添加 SHACL 形状以确保所有实体都有标签（RAG 上下文所需）

## 进一步阅读

- [向量和混合搜索](../features/vector-and-hybrid-search.md)
- [NL 到 SPARQL](../features/nl-to-sparql.md)
- [GraphRAG](../features/graphrag.md)
- [SHACL 验证](../features/validating-data-quality.md)
- [HTTP API 参考](../reference/http-api.md)