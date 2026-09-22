# Cookbook：SPARQL 修复工作流

**目标。** 构建一个迭代循环，其中 LLM 提出 SPARQL 查询，pg_ripple 告诉它出了什么问题（解析错误、缺失前缀、无行、模式不匹配），然后 LLM 再试一次。大多数"无结果"案例在一两次迭代中会自行修复。

**为什么使用 pg_ripple。** `sparql_from_nl()` 和 `explain_sparql()` 被设计为可组合的。将它们与[错误目录](../reference/error-catalog.md)结合使用，您将获得一个自动化的 SPARQL 配对程序员。

**首次结果时间。** 约 10 分钟。

---

## 循环

```
   用户问题（NL）
        │
        ▼
   sparql_from_nl(question)
        │
        ├── 解析失败（PT702）──► 描述错误 → LLM 重试
        │
        ▼
   sparql(query)
        │
        ├── 异常（PT…）       ──► 描述错误 → LLM 重试
        │
        ▼
   行数 = 0 ？
        │
        ├── 是 ──► explain_sparql(query, analyze:=true) → LLM 重试
        │
        ▼
   答案
```

最大迭代次数：3。比这更糟糕的通常是一个数据质量问题，应该上报给人工。

---

## 第 1 步 — 配置 NL → SPARQL

```sql
ALTER SYSTEM SET pg_ripple.llm_endpoint    = 'https://api.openai.com/v1';
ALTER SYSTEM SET pg_ripple.llm_api_key_env = 'OPENAI_API_KEY';
ALTER SYSTEM SET pg_ripple.llm_model       = 'gpt-4o';
ALTER SYSTEM SET pg_ripple.llm_include_shapes = on;
SELECT pg_reload_conf();
```

`llm_include_shapes = on` 将您的活动 SHACL 形状随每个提示一起发送 — LLM 看到模式并生成尊重它的查询。

## 第 2 步 — 为您的词汇表添加 few-shot 示例

```sql
SELECT pg_ripple.add_llm_example(
    'List all proteins that interact with insulin',
    'PREFIX bio: <https://bio.example/>
     SELECT ?p WHERE {
       ?p a bio:Protein ; bio:interactsWith bio:Insulin .
     }'
);
```

每个领域通常 5 到 10 个示例就足够了。

## 第 3 步 — 修复函数

```sql
CREATE OR REPLACE FUNCTION sparql_repair(question TEXT, max_attempts INT DEFAULT 3)
RETURNS TABLE (attempt INT, query TEXT, status TEXT, rows JSONB) AS $$
DECLARE
    q TEXT;
    feedback TEXT := '';
    n INT := 0;
    err TEXT;
    row_count INT;
    out_rows JSONB;
BEGIN
    LOOP
        n := n + 1;
        EXIT WHEN n > max_attempts;

        -- 1. 生成查询。
        BEGIN
            q := pg_ripple.sparql_from_nl(question || E'\n' || feedback);
        EXCEPTION WHEN OTHERS THEN
            feedback := format('Previous attempt failed to generate SPARQL: %s. Try again.', SQLERRM);
            CONTINUE;
        END;

        -- 2. 尝试执行它。
        BEGIN
            SELECT count(*), jsonb_agg(s) INTO row_count, out_rows
            FROM   pg_ripple.sparql(q) s;
        EXCEPTION WHEN OTHERS THEN
            err := SQLERRM;
            attempt := n; query := q; status := 'parse_or_runtime_error: ' || err;
            rows := NULL;
            RETURN NEXT;
            feedback := format('Previous SPARQL caused error: %s. The query was: %s', err, q);
            CONTINUE;
        END;

        -- 3. 如果为空，要求 LLM 放宽。
        IF row_count = 0 THEN
            attempt := n; query := q; status := 'empty_result_set'; rows := NULL;
            RETURN NEXT;
            feedback := format(
                'Previous query returned zero rows: %s. Loosen FILTERs or remove restrictive prefixes.',
                q
            );
            CONTINUE;
        END IF;

        -- 4. 
        attempt := n; query := q; status := 'ok'; rows := out_rows;
        RETURN NEXT;
        RETURN;
    END LOOP;
END;
$$ LANGUAGE plpgsql;
```

## 第 4 步 — 使用它

```sql
SELECT * FROM sparql_repair('Which proteins interact with the gene encoding insulin?');
```

输出：

```
attempt | query                                           | status                 | rows
--------+-------------------------------------------------+------------------------+-----
1       | SELECT ?p WHERE { ?p bio:interactsWith ...      | parse_or_runtime_error | NULL
2       | PREFIX bio: <https://bio.example/> SELECT ...   | empty_result_set       | NULL
3       | PREFIX bio: <https://bio.example/> SELECT ...   | ok                     | [...]
```

三次尝试，自动化。用户获得正确答案；审计日志捕获每个尝试的查询以供稍后分析。

---

## 遥测

将 `sparql_repair()` 接入到您的应用程序遥测中：

- **每个问题的平均尝试次数** — > 2 意味着您需要更多 few-shot 示例。
- **最常见的错误代码** — 可能是词汇表差距。
- **重试深度** — > 3 意味着 LLM 在原地打转；上报给人工。

---

## 强化

- **设置 `pg_ripple.sparql_max_algebra_depth`** 在执行之前拒绝病理生成的查询。
- **设置 `pg_ripple.sparql_query_timeout`** 这样错误的生成无法卡住数据库。
- **缓存成功的（question → query）对** — LLM 不需要为重复调用。

---

## 另请参阅

- [NL → SPARQL](../features/nl-to-sparql.md)
- [SPARQL 查询调试器 — explain_sparql](../user-guide/explain-sparql.md)
- [错误目录](../reference/error-catalog.md)