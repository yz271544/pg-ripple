# Playground（试用沙盒）

试用 pg_ripple 的最快方法是使用 Docker。无需安装 PostgreSQL。

## 启动沙盒

```bash
docker run --rm -p 5432:5432 \
  -e POSTGRES_PASSWORD=ripple \
  ghcr.io/trickle-labs/pg-ripple:latest
```

> **注意**：沙盒容器配置用于开发/测试，对外部 TCP 连接使用信任认证。对于生产用途，请参阅[安装](../getting-started/installation.md)和[安全](../operations/security.md)。

使用任何 PostgreSQL 客户端连接（无需密码）：

```bash
psql -h localhost -U postgres -d postgres
```

## 预加载的示例数据集

沙盒镜像包含一个小型 FOAF 风格的数据集，预加载到 `examples` 数据库中：

```bash
psql -h localhost -U postgres -d examples
```

```sql
-- Alice 认识谁？
SELECT * FROM pg_ripple.sparql('
  SELECT ?name WHERE {
    <https://example.org/alice> <https://xmlns.com/foaf/0.1/knows> ?person .
    ?person <https://xmlns.com/foaf/0.1/name> ?name
  }
');
```

```sql
-- 传递性：通过 knows+ 可从 Alice 到达的每个人
SELECT * FROM pg_ripple.sparql('
  SELECT ?target WHERE {
    <https://example.org/alice>
      <https://xmlns.com/foaf/0.1/knows>+
    ?target
  }
');
```

```sql
-- 按组织统计人数
SELECT * FROM pg_ripple.sparql('
  SELECT ?org (COUNT(?person) AS ?headcount) WHERE {
    ?person <https://xmlns.com/foaf/0.1/member> ?org
  } GROUP BY ?org ORDER BY DESC(?headcount)
');
```

## 尝试您自己的数据

```sql
-- 加载您自己的 N-Triples
SELECT pg_ripple.load_ntriples('
<https://my.example/a> <https://my.example/p> <https://my.example/b> .
<https://my.example/b> <https://my.example/p> <https://my.example/c> .
');

-- 运行路径查询
SELECT * FROM pg_ripple.sparql('
  SELECT ?target WHERE {
    <https://my.example/a> <https://my.example/p>+ ?target
  }
');
```

## 本地构建

要自行构建 Docker 镜像：

```bash
git clone https://github.com/trickle-labs/pg-ripple.git
cd pg-ripple
docker build -t pg-ripple:local .
docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=ripple pg-ripple:local
```

## 下一步

- [安装](../getting-started/installation.md) — 将 pg_ripple 安装到您自己的 PostgreSQL 实例
- [入门](../getting-started/hello-world.md) — 五分钟教程
- [SPARQL 查询](sql-reference/sparql-query.md) — 完整的 SPARQL 参考