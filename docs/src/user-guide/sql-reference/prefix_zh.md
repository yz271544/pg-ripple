# 前缀注册表

前缀注册表将短前缀（例如 `ex`）映射到 IRI 扩展名（例如 `https://example.org/`）。注册的前缀在导出函数将三元组序列化为 Turtle 或 N-Triples 时使用，以及当 `pg_ripple.sparql_prefix_mode` 设置为 `registered` 时由 SPARQL 使用。

> **严格模式是默认的。** 在严格模式下，SPARQL 查询必须使用尖括号中的完整 IRI（`<https://…>`）或使用 `PREFIX ex: <https://…>` 内联声明前缀。已注册模式仅添加缺失的声明；本地声明始终优先。

## register_prefix

```sql
pg_ripple.register_prefix(prefix TEXT, expansion TEXT) RETURNS VOID
```

注册或替换前缀-扩展名映射。

```sql
SELECT pg_ripple.register_prefix('ex', 'https://example.org/');
SELECT pg_ripple.register_prefix('rdf', 'http://www.w3.org/1999/02/22-rdf-syntax-ns#');
SELECT pg_ripple.register_prefix('xsd', 'http://www.w3.org/2001/XMLSchema#');
```

## prefixes

```sql
pg_ripple.prefixes() RETURNS TABLE(prefix TEXT, expansion TEXT)
```

返回所有注册的前缀-扩展名映射。

```sql
SELECT * FROM pg_ripple.prefixes();
```

## SPARQL 前缀模式

```sql
SET pg_ripple.sparql_prefix_mode = 'registered';
SELECT * FROM pg_ripple.sparql(
    'SELECT ?s WHERE { ?s ex:name ?name }'
);
```

已注册的声明经过验证并按确定性顺序注入。提交后，已注册模式的计划会因更改而失效；回滚的更改不会更改注册表生成版本。

## 示例：导出中的前缀

注册 `ex: <https://example.org/>` 后，导出函数将使用短形式：

```sql
SELECT pg_ripple.register_prefix('ex', 'https://example.org/');
SELECT pg_ripple.export_ntriples();
-- 输出使用完整限定形式 <https://example.org/…>
-- Turtle 输出在可用时使用 ex:… 短形式
```