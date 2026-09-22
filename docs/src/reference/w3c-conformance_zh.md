# W3C 一致性

此页面总结了 pg_ripple 当前的标准证据。套件名称并不意味着涵盖了每个上游测试或部署模式。

截至 v0.136.0，锁定的 324 案例 W3C SPARQL 套件和 14 查询 LUBM 套件是必需的 CI 网关。Jena 和 WatDiv 是额外的信号。OWL 2 RL CI 当前仅验证可以下载并校验和验证固定的上游语料库。

## 测试套件

pg_ripple 运行这些互补的验证作业：

| 套件 | 测试 | 验证内容 |
|---|---|---|
| W3C SPARQL 1.1 | 324 个锁定案例 | 必需的查询和更新一致性网关 |
| Apache Jena | ~1,000 | 实现边缘情况（类型强制转换、日期时间、空白节点范围） |
| WatDiv | 32 个已检入模板 | 对空数据库的查询执行；当前没有规模或延迟声明 |
| LUBM | 14 个查询 | 本体推理下的 OWL RL 推理正确性（v0.44.0+） |
| OWL 2 RL | 489 个源案例 | 必需的语料库采集和校验和检查；当前测试装置不执行案例 |

执行套件在支持时写入报告。OWL 源作业上传已验证的 `all.rdf` 语料库，而不是一致性报告。

请参阅 [运行一致性测试](running-conformance-tests.md) 以获取本地设置说明，[WatDiv 结果](watdiv-results.md) 页面以获取性能指标，[LUBM 结果](lubm-results.md) 页面以获取 OWL RL 一致性详细信息。

---

## W3C SPARQL 1.1 测试装置（v0.41.0+）

测试装置（`tests/w3c/`）针对实时 pg_ripple 安装运行从官方 [W3C SPARQL 1.1 测试套件](https://www.w3.org/2009/sparql/docs/tests/) 派生的 324 个锁定案例集。它串行运行，因为这些案例共享一个 PostgreSQL 数据库。已检查的 v0.136.0 报告有 324 个通过，没有失败、跳过或预期失败。

### 按类别覆盖

| 子套件 | 测试 | CI 状态 |
|---|---|---|
| 聚合 | 47 | 必需 |
| 基本更新 | 13 | 必需 |
| bind | 10 | 必需 |
| exists | 6 | 必需 |
| 函数 | 75 | 必需 |
| 分组 | 6 | 必需 |
| 否定 | 12 | 必需 |
| project-expression | 7 | 必需 |
| 属性路径 | 33 | 必需 |
| service | 7 | 必需 |
| 子查询 | 14 | 必需 |
| syntax-query | 94 | 必需 |

### 在本地运行

```sh
# 首先下载测试数据（一次性设置）：
bash scripts/fetch_conformance_tests.sh --w3c

# 运行单独的精选 smoke 子集：
cargo test --test w3c_smoke

# 串行运行锁定的 324 案例套件：
cargo test --test w3c_suite -- --nocapture
```

---

## Apache Jena 测试套件（v0.43.0+）

Jena 适配器（`tests/jena/`）运行来自 Apache Jena 的 `sparql-query`、`sparql-update`、`sparql-syntax` 和 `algebra` 子套件的约 1,000 个测试。Jena 测试涵盖了 W3C 套件未充分指定的实现边缘情况。

### Jena 特定的覆盖区域

| 区域 | 测试 |
|---|---|
| XSD 数字提升（`xsd:integer` → `xsd:decimal` → `xsd:double`） | sparql-query |
| 混合类型算术和比较 | sparql-query |
| 时区感知的 `xsd:dateTime` 比较 | sparql-query |
| 日期/时间内置函数：`NOW()`、`YEAR()`、`MONTH()`、`DAY()`、`HOURS()`、`MINUTES()`、`SECONDS()`、`TZ()` | sparql-query |
| `xsd:decimal` 算术：`ROUND()`、`CEIL()`、`FLOOR()`、`ABS()` | sparql-query |
| CONSTRUCT 模板中的空白节点 | sparql-query |
| OPTIONAL 和 GRAPH 边界上的空白节点身份 | sparql-query |
| 字符串函数：`STRLEN()`、`SUBSTR()`、`UCASE()`、`LCASE()`、`STRSTARTS()`、`STRENDS()`、`CONTAINS()`、`ENCODE_FOR_URI()`、`CONCAT()` | sparql-query |
| SPARQL 更新边缘情况 | sparql-update |
| 语法接受/拒绝（正面/负面语法测试） | sparql-syntax |
| 代数规范化等价 | algebra |

### CI status

`jena-suite` CI 作业在通过率达到 ≥ 95% 之前是**非阻塞**的，然后提升为必需。类型强制转换和日期时间边缘情况的已知失败在 `tests/conformance/known_failures.txt` 中以 `jena:` 前缀进行跟踪。

### 在本地运行

```sh
# 下载 Jena 测试数据：
bash scripts/fetch_conformance_tests.sh --jena

# 运行完整的 Jena 套件：
cargo test --test jena_suite
```

---

## SPARQL 1.1 查询

**测试套件**：[W3C SPARQL 1.1 查询测试套件（2013-03-27）](https://www.w3.org/2009/sparql/test-suite-20130327/)

**网关**：所有 324 个锁定案例必须通过。

### 支持的功能

| 功能 | 状态 |
|---|---|
| 基本图模式 (BGP) | 支持 |
| 带所有比较和逻辑运算符的 FILTER | 支持 |
| OPTIONAL | 支持 |
| UNION | 支持 |
| 子查询（`SELECT … { SELECT … }`） | 支持 |
| BIND | 支持 |
| VALUES | 支持 |
| 属性路径（`/`、`|`、`*`、`+`、`?`、`^`） | 支持 |
| 否定属性集（`!(p1|p2)`） | 支持 |
| 聚合：COUNT、SUM、AVG、MIN、MAX | 支持 |
| GROUP BY、HAVING | 支持 |
| ORDER BY、LIMIT、OFFSET | 支持 |
| DISTINCT | 支持 |
| ASK | 支持 |
| CONSTRUCT | 支持 |
| DESCRIBE | 支持 |
| 命名图（`GRAPH ?g { … }`） | 支持 |
| 联邦查询（`SERVICE`） | 支持（v0.16.0） |
| 所有 XPath/SPARQL 内置函数（STR、STRLEN、UCASE、LCASE、STRSTARTS、STRENDS、CONTAINS、REGEX、ABS、CEIL、FLOOR、ROUND、IF、COALESCE、isIRI、isLiteral、isBlank、DATATYPE、LANG、BIND） | 支持 |
| 语言标记字符串（存储和 LANG() 函数） | 支持 |
| 带 xsd:integer、xsd:decimal、xsd:double、xsd:dateTime、xsd:boolean 的类型化字面量 | 支持 |
| NOT EXISTS | 支持 |
| MINUS | 支持 |
| RDF-star（引号三元组、SPARQL-star BGP） | 支持（v0.4.0） |

### 已知限制

| 功能 | 状态 |
|---|---|
| `langMatches()` 函数 | 不支持。不出错返回 0 行。完整的 BCP 47 语言标签匹配计划在未来版本中。 |
| 自定义聚合操作符扩展（属性函数） | 不支持。标准聚合（COUNT、SUM、AVG、MIN、MAX）完全支持。 |
| 引号三元组内的变量模式（`<< ?s ?p ?o >>`） | 返回 0 行并发出 WARNING。基础引号三元组模式有效。 |
| 从任意 HTTP URI 进行 `LOAD <url>` | 依赖于网络；通过 `pg_ripple_http` 配套服务支持。 |

---

## SPARQL 1.1 更新

**测试套件**：[W3C SPARQL 1.1 更新测试套件（2013）](https://www.w3.org/2013/sparql-update-tests/)

**目标**：≥ 95% 的适用测试通过。

### 支持的功能

| 功能 | 状态 |
|---|---|
| INSERT DATA | 支持 |
| DELETE DATA | 支持 |
| INSERT WHERE | 支持 |
| DELETE WHERE | 支持 |
| DELETE/INSERT WHERE | 支持 |
| CLEAR GRAPH | 支持 |
| CREATE GRAPH / DROP GRAPH | 支持 |
| 多语句更新（`;` 分隔符） | 支持 |
| 命名图更新操作 | 支持 |
| 幂等重新插入（ON CONFLICT DO NOTHING） | 支持 |

### 已知限制

| 功能 | 状态 |
|---|---|
| `COPY`、`MOVE`、`ADD` 图操作 | 支持。 |
| `LOAD <url>` | 与上述查询相同。 |

---

## SHACL Core

**测试套件**：[W3C SHACL Core 测试套件](https://w3c.github.io/shacl/tests/)

**目标**：≥ 95% 的 SHACL Core 测试通过。

### 支持的约束

| 约束 | 状态 |
|---|---|
| `sh:targetClass` | 支持 |
| `sh:targetNode` | 支持 |
| `sh:targetSubjectsOf` | 支持 |
| `sh:targetObjectsOf` | 支持 |
| `sh:property` 带 `sh:path` | 支持 |
| `sh:minCount` / `sh:maxCount` | 支持 |
| `sh:datatype` | 支持 |
| `sh:pattern`（正则表达式） | 支持 |
| `sh:minLength` / `sh:maxLength` | 支持 |
| `sh:minInclusive` / `sh:maxInclusive` | 支持 |
| `sh:minExclusive` / `sh:maxExclusive` | 支持 |
| `sh:in`（枚举） | 支持 |
| `sh:hasValue` | 支持 |
| `sh:class` | 支持 |
| `sh:nodeKind`（IRI、BlankNode、Literal） | 支持 |
| `sh:or` | 支持 |
| `sh:and` | 支持 |
| `sh:not` | 支持 |
| `sh:node`（嵌套形状引用） | 支持 |
| `sh:qualifiedValueShape` + `sh:qualifiedMinCount` / `sh:qualifiedMaxCount` | 支持 |
| 异步验证管道（`process_validation_queue`） | 支持 |
| 同步模式（插入拒绝） | 支持 |

### 已知限制

| 功能 | 状态 |
|---|---|
| SHACL Advanced Features（基于 SPARQL 的约束，`sh:SPARQLConstraint`） | 推迟到 v0.21.0。 |
| SHACL-AF（规则，`sh:TripleRule`） | 通过 Datalog 进行部分实现；完整的 SHACL-AF 集成已规划。 |

---

## 运行一致性网关

一致性测试作为标准 pg_regress 套件的一部分运行：

```bash
cargo pgrx regress pg18 --postgresql-conf "allow_system_table_mods=on"
```

相关的测试文件为：

- `tests/pg_regress/sql/w3c_sparql_query_conformance.sql`
- `tests/pg_regress/sql/w3c_sparql_update_conformance.sql`
- `tests/pg_regress/sql/w3c_shacl_conformance.sql`
- `tests/pg_regress/sql/crash_recovery_merge.sql`