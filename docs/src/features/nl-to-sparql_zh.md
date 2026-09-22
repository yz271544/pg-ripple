# 自然语言到 SPARQL

pg_ripple v0.49.0 添加了 `pg_ripple.sparql_from_nl()`，一个 SQL 函数，使用任何配置的 OpenAI 兼容 LLM 端点将简单的英语问题转换为 SPARQL SELECT 查询。

## 快速开始

```sql
-- 配置 OpenAI 兼容端点
SET pg_ripple.llm_endpoint = 'https://api.openai.com/v1';
SET pg_ripple.llm_model    = 'gpt-4o';
-- API 密钥从指定的环境变量读取，而不是内联存储：
SET pg_ripple.llm_api_key_env = 'OPENAI_API_KEY';  -- 默认

-- 从简单的英语生成 SPARQL 查询
SELECT pg_ripple.sparql_from_nl('列出所有人和他们的电子邮件地址');
```

```
SELECT ?person ?email WHERE {
  ?person a <http://xmlns.com/foaf/0.1/Person> .
  ?person <http://xmlns.com/foaf/0.1/mbox> ?email .
}
```

返回的字符串是有效的、可解析的 SPARQL 1.1 查询，您可以直接传递给 `pg_ripple.sparql()`。

## 配置 LLM 端点

pg_ripple 支持任何 OpenAI 兼容的 `/v1/chat/completions` API，包括：

| 提供商 | 示例 `llm_endpoint` |
|----------|------------------------|
| OpenAI | `https://api.openai.com/v1` |
| Azure OpenAI | `https://<resource>.openai.azure.com/openai/deployments/<deployment>` |
| Ollama（本地）| `http://localhost:11434/v1` |
| vLLM | `http://localhost:8000/v1` |
| Together AI | `https://api.together.xyz/v1` |

### GUC 参数

| GUC | 类型 | 默认 | 说明 |
|-----|------|---------|-------------|
| `pg_ripple.llm_endpoint` | string | `''`（禁用）| OpenAI 兼容 API 的基本 URL。设置为 `'mock'` 以在没有真实 LLM 的情况下进行测试。 |
| `pg_ripple.llm_model` | string | `gpt-4o` | 在请求体中传递的模型标识符。 |
| `pg_ripple.llm_api_key_env` | string | `PG_RIPPLE_LLM_API_KEY` | 保存 API 密钥的环境变量的名称。密钥永远不存储在数据库中。 |
| `pg_ripple.llm_include_shapes` | bool | `on` | 当 `on` 时，活动的 SHACL 形状作为 schema 上下文包含在 LLM 提示中。 |

### 安全地设置 API 密钥

pg_ripple 永远不在数据库中存储 API 密钥。相反，它在调用时从指定的环境变量读取值：

```bash
# 在您的 shell 或服务环境中：
export PG_RIPPLE_LLM_API_KEY="sk-..."
```

```sql
-- 告诉 pg_ripple 读取哪个环境变量（默认为 PG_RIPPLE_LLM_API_KEY）：
ALTER SYSTEM SET pg_ripple.llm_api_key_env = 'PG_RIPPLE_LLM_API_KEY';
SELECT pg_reload_conf();
```

## 工作原理

对于每次调用 `sparql_from_nl(question)`：

1. **VoID 上下文**：pg_ripple 构建图的紧凑描述 — 谓词计数和最频繁的谓词 — 作为 LLM 的上下文。
2. **SHACL 上下文**（当 `llm_include_shapes = on` 时）：活动的 SHACL 形状附加到提示。
3. **少样本示例**：`_pg_ripple.llm_examples` 中的任何行都作为问题/SPARQL 对包含。
4. **LLM 调用**：提示以 `temperature = 0.0` 发送到 `/v1/chat/completions`。
5. **提取**：SPARQL 字符串从响应中提取，并去除任何 markdown 围栏。
6. **验证**：`spargebra` 解析查询。如果解析失败，则引发 PT702，以便调用者可以处理错误。

## 添加少样本示例

少样本示例显著提高特定领域词汇表的准确性：

```sql
SELECT pg_ripple.add_llm_example(
    '查找所有与胰岛素相互作用的蛋白质',
    'SELECT ?protein WHERE {
       ?protein <https://bio.ontology.org/interactsWith>
                <https://bio.ontology.org/Insulin> .
     }'
);

SELECT pg_ripple.add_llm_example(
    '哪些药物靶向 EGFR？',
    'SELECT ?drug WHERE {
       ?drug <https://bio.ontology.org/targets>
             <https://bio.ontology.org/EGFR> .
     }'
);
```

示例存储在 `_pg_ripple.llm_examples` 中，并自动包含在每个后续的 `sparql_from_nl()` 调用中。使用相同问题重新调用 `add_llm_example()` 会更新存储的示例（upsert 行为）。

## 在没有真实 LLM 的情况下进行测试

设置 `pg_ripple.llm_endpoint = 'mock'` 以使用内置的测试模拟。该模拟绕过 HTTP 调用并返回一个简单的 `SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10` 查询，允许您在没有外部 LLM 依赖的情况下测试下游代码（结果处理、SPARQL 执行）。

```sql
SET pg_ripple.llm_endpoint = 'mock';
SELECT pg_ripple.sparql_from_nl('anything') LIKE 'SELECT%';  -- t
```

## 错误处理

| 代码 | 条件 | 补救方法 |
|------|-----------|--------|
| PT700 | `llm_endpoint` 为空或 HTTP 调用失败 | 设置有效的端点 URL；检查网络访问和 API 密钥 |
| PT701 | LLM 响应不包含 SPARQL 查询 | 使用少样本示例改进提示；切换到更有能力的模型 |
| PT702 | 生成的 SPARQL 无法解析 | 为此问题模式添加少样本示例；或使用针对 SPARQL 微调的模型 |

## 管道模式

一种常见模式是生成查询、记录它，然后一步执行它：

```sql
DO $$
DECLARE
    sparql_q TEXT;
    result   TEXT;
BEGIN
    sparql_q := pg_ripple.sparql_from_nl(
        '查找所有在 2010 年后成立且员工超过 500 人的公司'
    );
    RAISE NOTICE '生成的 SPARQL: %', sparql_q;
    -- 执行生成的查询
    SELECT json_agg(row_to_json(t))::text
    INTO result
    FROM pg_ripple.sparql(sparql_q) t;
    RAISE NOTICE '结果: %', result;
END;
$$;
```

## 进一步阅读

- [博客：自然语言到 SPARQL](https://github.com/trickle-labs/pg-ripple/blob/main/blog/natural-language-to-sparql.md) — pg_ripple 如何将简单的英语问题翻译成 SPARQL 查询