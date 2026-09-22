# PPRL：跨组织实体解析而不共享原始 PII

> **您将学到：** 如何使用 `bloom_encode()` 和 `pg:dice_similarity` 查找两个组织之间引用同一现实世界人物的记录 — 而任何一方都不会看到对方的原始个人数据。

## 为什么这很重要

医院、银行和政府机构经常需要知道他们的记录是否重叠 — 两个系统中出现相同的患者、账户持有人或公民。法规（GDPR、HIPAA、CCPA）通常禁止共享原始标识符，如全名、出生日期或社会安全号码。

**隐私保护记录链接（PPRL）** 使用 Bloom 过滤编码解决这个问题：敏感标识符被转换为支持相似度比较的定长位向量，而不会泄露底层值。

> **参考：** Schnell, Bachteler & Reiher (2009) — "Privacy-preserving record linkage using Bloom filters." *BMC Medical Informatics and Decision Making* 9:41.

## 架构

```
组织 A                               组织 B
──────────────                       ──────────────
原始 PII → bloom_encode() → bloomA    原始 PII → bloom_encode() → bloomB
          |                                    |
          └──── RDF 三元组存储 ─────────────────┘
                     |
              SPARQL SERVICE 联邦
                     |
              FILTER(pg:dice_similarity(?bloomA, ?bloomB) > 0.85)
                     |
              候选匹配（未交换原始 PII）
```

两个组织使用**相同的共享密钥**（通过带外方式建立安全通道 — 例如 TLS 保护的密钥协议），以便两侧相同值的 Bloom 编码相同。

## 第 1 步：编码患者标识符

每个组织在自己的数据库上运行此操作。共享密钥必须在组织之间完全匹配。

```sql
-- 启用扩展
CREATE EXTENSION IF NOT EXISTS pg_ripple;

-- 编码患者记录。'shared_secret_key' 必须在两个组织之间通过带外方式协商。
INSERT INTO patient_triples (subject, predicate, object)
VALUES (
    '<http://hospitalA.example.org/patient/1001>',
    '<http://pprl.example.org/nameBloom>',
    pg_ripple.bloom_encode(
        'Alice M Smith',          -- 串联相关字段
        'shared_secret_key',      -- 必须与组织 B 的密钥匹配
        30,                       -- hash_count（最小推荐：30）
        1024                      -- 位长度（最小推荐：1024）
    )
);

-- 将完整的患者记录存储为 RDF 三元组
SELECT pg_ripple.insert_triple(
    '<http://hospitalA.example.org/patient/1001>',
    '<http://pprl.example.org/nameBloom>',
    pg_ripple.bloom_encode('Alice M Smith 1985-04-12', 'shared_secret_key')
);
```

## 第 2 步：运行跨组织匹配

使用 SPARQL `SERVICE` 联邦同时查询两个组织。SPARQL `pg:dice_similarity` FILTER 比较 Bloom 过滤器而不检索底层 PII。

```sparql
PREFIX pprl: <http://pprl.example.org/>
PREFIX pg:   <http://pg-ripple.org/functions/>

SELECT ?patientA ?patientB ?sim WHERE {
  -- 医院 A 的 Bloom 编码标识符（本地图）
  ?patientA pprl:nameBloom ?bloomA .

  -- 医院 B 的 Bloom 编码标识符（远程 SPARQL 端点）
  SERVICE <http://hospitalB.example.org/sparql> {
    ?patientB pprl:nameBloom ?bloomB .
  }

  -- 查找 Bloom 过滤器高度相似的候选
  FILTER(pg:dice_similarity(?bloomA, ?bloomB) > 0.85)

  -- 计算精确相似度以进行排序
  BIND(pg:dice_similarity(?bloomA, ?bloomB) AS ?sim)
}
ORDER BY DESC(?sim)
```

从 SQL：

```sql
SELECT * FROM pg_ripple.sparql($$
    PREFIX pprl: <http://pprl.example.org/>
    PREFIX pg:   <http://pg-ripple.org/functions/>

    SELECT ?patientA ?patientB ?sim WHERE {
      ?patientA pprl:nameBloom ?bloomA .
      SERVICE <http://hospitalB.example.org/sparql> {
        ?patientB pprl:nameBloom ?bloomB .
      }
      FILTER(pg:dice_similarity(?bloomA, ?bloomB) > 0.85)
      BIND(pg:dice_similarity(?bloomA, ?bloomB) AS ?sim)
    }
    ORDER BY DESC(?sim)
$$);
```

## 第 3 步：使用差分隐私进行聚合报告

```sql
-- 使用 ε=0.1 差分隐私计数共享患者
SELECT pg_ripple.dp_noisy_count(
    'SELECT COUNT(DISTINCT subject) FROM _pg_ripple.vp_rare
     WHERE predicate = pg_ripple.encode_iri(''http://pprl.example.org/nameBloom'')',
    0.1  -- epsilon：越小 = 越隐私，噪声越大
) AS approx_shared_patients;
```

对于匹配置信度桶的直方图：

```sql
CREATE TEMP TABLE match_buckets (bucket TEXT, n BIGINT);
INSERT INTO match_buckets VALUES
    ('high (>0.9)',    42),
    ('medium (0.7-0.9)', 17),
    ('low (0.5-0.7)',  8);

SELECT * FROM pg_ripple.dp_noisy_histogram(
    'SELECT bucket, n FROM match_buckets',
    'bucket',
    'n',
    0.5  -- epsilon
);
```

## 第 4 步：用于实体解析的 Datalog 规则

您还可以将匹配表达为 Datalog 规则并使用增量具体化：

```sql
SELECT pg_ripple.load_rules($$
    -- 生成 Bloom 过滤器相似的候选对
    pprl:candidate(?x, ?y) :-
        ?x <http://pprl.example.org/nameBloom> ?bx .
        ?y <http://pprl.example.org/nameBloom> ?by .
        ?x != ?y .
        pg:dice_similarity(?bx, ?by) > 0.85 .
$$);

-- 运行推理以具体化所有候选对
SELECT pg_ripple.infer();
```

## 安全注意事项

### 推荐参数

| 参数 | 最小推荐 | 默认 |
|-----------|--------------------:|--------:|
| `hash_count` | 30 | 30 |
| `length` | 1024 位 | 1024 |

使用少于 30 个哈希函数或少于 1024 位会显著增加通过基于图的攻击重新识别的风险（Schnell et al. 2009, §4）。当参数低于推荐最小值时，pg_ripple 记录 WARNING。

### 密钥管理

共享密钥（`key` 参数）必须：

1. **带外协商** — 永远不与 Bloom 编码数据一起传输
2. **保密** — 受损密钥允许反转编码数据
3. **特定于组织对** — 为每对组织使用不同的密钥以限制爆炸半径

### Bloom 过滤器保护什么

- **诚实但好奇的对手**：仅从位向量无法推断原始值（没有密钥）
- **窃听者**：仅看到位向量；没有密钥，无法恢复值

### 已知限制

- 具有低 `hash_count` 或小 `length` 的 Bloom 过滤器即使没有密钥也容易受到基于图的重构攻击
- 高频值（例如非常常见的名称）更容易受到推理攻击
- CLK 编码不是量子抗性的

### 专利状态

Schnell et al. (2009) 中描述的 CLK（加密长期密钥）构造是已发表的学术工作，没有已知的专利纠缠。

## 验证您的编码

```sql
-- 自测试：编码相同值两次应给出 Dice = 1.0
SELECT pg_ripple.dice_similarity(
    pg_ripple.bloom_encode('Alice M Smith', 'testkey', 30, 1024),
    pg_ripple.bloom_encode('Alice M Smith', 'testkey', 30, 1024)
) AS should_be_one;

-- 编码不同值应给出 Dice < 1.0
SELECT pg_ripple.dice_similarity(
    pg_ripple.bloom_encode('Alice M Smith', 'testkey', 30, 1024),
    pg_ripple.bloom_encode('Bob J Jones',   'testkey', 30, 1024)
) AS should_be_less_than_half;
```

## 进一步阅读

- Schnell, R., Bachteler, T., & Reiher, J. (2009). Privacy-preserving record linkage using Bloom filters. *BMC Medical Informatics and Decision Making*, 9, 41.
  <https://doi.org/10.1186/1472-6947-9-41>
- Christen, P., Ranbaduge, T., & Schnell, R. (2020). *Linking Sensitive Data: Methods and Techniques for Practical Privacy-Preserving Information Sharing*. Springer.
- pg_ripple SPARQL 联邦指南: [federation-wikidata.md](federation-wikidata.md)
- pg_ripple 差分隐私函数: `pg_ripple.dp_noisy_count()`, `pg_ripple.dp_noisy_histogram()`