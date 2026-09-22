# AI 代理集成（LangChain、LlamaIndex 等）

pg_ripple 的 AI 功能 — 向量搜索、`rag_context()`、`sparql_from_nl()`、图扩展 — 公开为**普通 SQL 函数**。任何可以调用 PostgreSQL 的框架都可以使用它们。无需 SDK。

本页介绍如何将 pg_ripple 接入两个最常见的 Python 代理框架，以及一个与框架无关的工具调用模式。

---

## 心智模型

AI 代理循环如下所示：

```
用户问题
     │
     ▼
   LLM 决定：我是否需要查找某些内容？
     │
     ├── 是 → 调用工具（通过 SQL 的 pg_ripple 函数）→ 获取上下文
     │          └── 使用上下文循环回到 LLM
     │
     └── 否  → 生成最终答案
```

pg_ripple 是工具。LLM 通过 `rag_context()`、`sparql_from_nl()` 或自定义 SQL 包装器调用它 — 无论您的框架公开哪个。

---

## 先决条件

```sql
-- 配置嵌入端点（每个数据库一次）。
ALTER SYSTEM SET pg_ripple.embedding_api_url     = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.embedding_api_key_env = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.embedding_model       = 'text-embedding-3-small';

ALTER SYSTEM SET pg_ripple.llm_endpoint          = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.llm_api_key_env       = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.llm_model             = 'gpt-4o';

SELECT pg_reload_conf();

-- 嵌入您的知识图（一次，然后在新的三元组到达时增量地）。
SELECT pg_ripple.embed_entities();
```

---

## LangChain

最干净的集成是一个调用 `rag_context()` 的 `Tool`，以及一个调用 `sparql_from_nl()` 用于事实型答案的第二个 `Tool`。

```python
from langchain.tools import Tool
from langchain_openai import ChatOpenAI
from langchain.agents import AgentExecutor, create_openai_tools_agent
from langchain_core.prompts import ChatPromptTemplate, MessagesPlaceholder
import psycopg

DB_URL = "postgresql://..."

def graph_context(question: str) -> str:
    """从知识图中检索问题的图上下文。"""
    with psycopg.connect(DB_URL) as conn:
        cur = conn.cursor()
        cur.execute("SELECT pg_ripple.rag_context(%s, 8)", (question,))
        return cur.fetchone()[0]


def graph_query(sparql: str) -> str:
    """针对知识图执行 SPARQL 查询并以 JSON 形式返回结果。"""
    with psycopg.connect(DB_URL) as conn:
        cur = conn.cursor()
        cur.execute("SELECT jsonb_agg(row_to_json(r)) FROM pg_ripple.sparql(%s) r", (sparql,))
        result = cur.fetchone()[0]
        return str(result) if result else "无结果。"


def nl_to_sparql_and_run(question: str) -> str:
    """将自然语言问题转换为 SPARQL 并运行它。"""
    with psycopg.connect(DB_URL) as conn:
        cur = conn.cursor()
        cur.execute(
            "SELECT jsonb_agg(row_to_json(r)) FROM "
            "pg_ripple.sparql(pg_ripple.sparql_from_nl(%s)) r",
            (question,)
        )
        result = cur.fetchone()[0]
        return str(result) if result else "无结果。"


tools = [
    Tool(name="graph_context",       func=graph_context,
         description="为开放式问题检索其情境图。"),
    Tool(name="nl_to_sparql_query",  func=nl_to_sparql_and_run,
         description="使用 SPARQL 自动生成回答精确的事实性问题。"),
]

llm    = ChatOpenAI(model="gpt-4o", temperature=0)
prompt = ChatPromptTemplate.from_messages([
    ("system", "您是一个乐于助人的助手，可以访问知识图。"
               "对广泛的问题使用 'graph_context'，对特定事实使用 'nl_to_sparql_query'。"),
    ("human", "{input}"),
    MessagesPlaceholder("agent_scratchpad"),
])
agent    = create_openai_tools_agent(llm, tools, prompt)
executor = AgentExecutor(agent=agent, tools=tools, verbose=True)

result = executor.invoke({"input": "哪些药物与胰岛素相互作用，应该避免使用？"})
print(result["output"])
```

---

## LlamaIndex

LlamaIndex 的 `FunctionTool` 直接映射到 pg_ripple SQL 调用。

```python
from llama_index.core.tools import FunctionTool
from llama_index.core.agent import ReActAgent
from llama_index.llms.openai import OpenAI
import psycopg

DB_URL = "postgresql://..."


def retrieve_graph_context(question: str) -> str:
    """
    在知识图中搜索与问题相关的实体和关系。
    返回适合 LLM 提示的结构化文本块。
    """
    with psycopg.connect(DB_URL) as conn:
        cur = conn.cursor()
        cur.execute("SELECT pg_ripple.rag_context(%s, 8)", (question,))
        return cur.fetchone()[0]


def sparql_fact_lookup(question: str) -> str:
    """
    通过自动生成和执行 SPARQL 查询来回答精确的事实性问题。
    用于诸如'有多少'、'列出全部'、'是谁'、'是什么'之类的问题。
    """
    with psycopg.connect(DB_URL) as conn:
        cur = conn.cursor()
        cur.execute(
            "SELECT jsonb_agg(row_to_json(r)) FROM "
            "pg_ripple.sparql(pg_ripple.sparql_from_nl(%s)) r",
            (question,)
        )
        result = cur.fetchone()[0]
        return str(result) if result else "查询未返回结果。"


tools = [
    FunctionTool.from_defaults(fn=retrieve_graph_context),
    FunctionTool.from_defaults(fn=sparql_fact_lookup),
]

agent = ReActAgent.from_tools(tools, llm=OpenAI(model="gpt-4o"), verbose=True)
response = agent.chat("将二甲双胍与胰岛素联合使用有哪些已知副作用？")
print(response)
```

---

## 与框架无关：OpenAI 工具调用

如果您不使用 LangChain 或 LlamaIndex，请直接使用 OpenAI 的工具调用 API。这适用于任何框架或纯 Python。

```python
import json, psycopg, openai

DB_URL = "postgresql://..."
client = openai.OpenAI()

TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "rag_context",
            "description": "从知识图中检索与开放式问题相关的上下文。",
            "parameters": {
                "type": "object",
                "properties": {
                    "question": {"type": "string"},
                    "k":        {"type": "integer", "default": 8,
                                 "description": "要检索的实体数量。"}
                },
                "required": ["question"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "sparql_fact_lookup",
            "description": "使用自动生成的 SPARQL 回答精确的事实性问题。",
            "parameters": {
                "type": "object",
                "properties": {"question": {"type": "string"}},
                "required": ["question"],
            },
        },
    },
]


def dispatch(tool_name: str, args: dict) -> str:
    with psycopg.connect(DB_URL) as conn:
        cur = conn.cursor()
        if tool_name == "rag_context":
            cur.execute("SELECT pg_ripple.rag_context(%s, %s)", (args["question"], args.get("k", 8)))
            return cur.fetchone()[0]
        elif tool_name == "sparql_fact_lookup":
            cur.execute(
                "SELECT jsonb_agg(row_to_json(r)) FROM "
                "pg_ripple.sparql(pg_ripple.sparql_from_nl(%s)) r",
                (args["question"],)
            )
            result = cur.fetchone()[0]
            return str(result) if result else "无结果。"
    return "未知工具。"


def agent_loop(question: str, max_turns: int = 5) -> str:
    messages = [{"role": "user", "content": question}]
    for _ in range(max_turns):
        response = client.chat.completions.create(
            model="gpt-4o", messages=messages, tools=TOOLS, tool_choice="auto"
        )
        msg = response.choices[0].message
        messages.append(msg)
        if msg.tool_calls:
            for call in msg.tool_calls:
                result = dispatch(call.function.name, json.loads(call.function.arguments))
                messages.append({
                    "role": "tool", "tool_call_id": call.id, "content": result
                })
        else:
            return msg.content
    return "达到最大轮数。"


print(agent_loop("哪些药物与胰岛素相互作用，应该避免使用？"))
```

---

## 生产代理的提示

- **缓存 `rag_context()` 结果**每个问题哈希（大多数问题会重复）。Redis 或 PostgreSQL 缓存在工具调用之前可以显著降低 LLM 成本和延迟。
- **设置 `pg_ripple.sparql_query_timeout`** 以限制失控的自动生成 SPARQL 查询。
- **使用 few-shot 示例**（`pg_ripple.add_llm_example()`）用于特定领域词汇 — 大幅降低 NL→SPARQL 错误率。
- **将工具调用结果记录到** `_pg_ripple.audit_log`（当 `audit_log_enabled = on` 时默认启用）— 然后每个 RAG 检索都是可审计的，这对受监管的行业很重要。
- **多租户**：在每个租户的代理会话使用的 PostgreSQL 连接上应用 [图 RLS](multi-tenant-graphs.md) — `rag_context()` 自动遵守 RLS。

---

## v1.1.0 中即将推出的内容

- 原生 LangChain `BaseTool` 和 LlamaIndex `QueryEngineTool` 包装器发布到 PyPI 作为 `pg-ripple-langchain` 和 `pg-ripple-llamaindex`。
- 通过 pg_ripple 游标 API 流式传输工具结果（用于大型图上下文块）。
- 图感知对话记忆：将对话历史存储为 RDF 三元组，以便代理可以推理过去的交互。

---

## 另请参阅

- [AI 概述](ai-overview.md) — 何时使用哪个 AI 功能。
- [RAG 管道](../user-guide/rag-pipeline.md) — `rag_context()` 深入探讨。
- [NL → SPARQL](nl-to-sparql.md) — `sparql_from_nl()` 和 few-shot 调整。
- [Cookbook：基于地面的聊天机器人](../cookbook/grounded-chatbot.md) — 简单的单代理配方。