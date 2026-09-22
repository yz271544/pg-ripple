# SPARQL 一致性矩阵

pg_ripple 实现了广泛的 SPARQL 1.1 表面积。此页面记录已实现的功能；必需的测试证据涵盖上游标准语料库的锁定子集。

```admonish success title="必需的 SPARQL 证据"
截至 v0.136.0，pg_ripple 通过其锁定的 W3C SPARQL 1.1 套件中的所有 324 个案例。这是一个必需的、失败关闭的 CI 网关。这并不声明每个上游 W3C 案例或所宣传功能的每个组合都已通过验证。
```

---

## SPARQL 1.1 查询 — 查询形式

| 功能 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| `SELECT` | ✅ 支持 | v0.1.0 | 带表达式的完整投影 |
| `CONSTRUCT` | ✅ 支持 | v0.8.0 | 以 JSON、Turtle 或 JSON-LD 形式返回三元组 |
| `ASK` | ✅ 支持 | v0.8.0 | 返回布尔值 |
| `DESCRIBE` | ✅ 支持 | v0.8.0 | 对称简洁有界描述 |

---

## SPARQL 1.1 查询 — 代数运算

| 功能 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| 基本图模式 (BGP) | ✅ 支持 | v0.1.0 | 转换为 VP 表连接 |
| 连接（内部） | ✅ 支持 | v0.1.0 | |
| LeftJoin（`OPTIONAL`） | ✅ 支持 | v0.1.0 | 设置 SHACL `sh:minCount 1` 时降级为 INNER JOIN |
| Filter | ✅ 支持 | v0.1.0 | 所有比较、逻辑和算术运算符 |
| Union | ✅ 支持 | v0.5.0 | 生成的 SQL 中为 `UNION ALL` |
| Minus | ✅ 支持 | v0.5.0 | 生成的 SQL 中为 `EXCEPT` |
| Extend（`BIND`） | ✅ 支持 | v0.1.0 | |
| Group（`GROUP BY`） | ✅ 支持 | v0.5.0 | |
| Having | ✅ 支持 | v0.5.0 | |
| OrderBy | ✅ 支持 | v0.1.0 | |
| Project | ✅ 支持 | v0.1.0 | |
| Distinct | ✅ 支持 | v0.1.0 | 设置 SHACL `sh:maxCount 1` 时省略 |
| Reduced | ✅ 支持 | v0.5.0 | 视为提示；可能或可能不去重 |
| Slice（`LIMIT`/`OFFSET`） | ✅ 支持 | v0.1.0 | |
| Service（`SERVICE`） | ✅ 支持 | v0.16.0 | 通过 HTTP 进行联邦查询 |
| Service Silent（`SERVICE SILENT`） | ✅ 支持 | v0.16.0 | 端点失败时返回空 |
| Values（`VALUES`） | ✅ 支持 | v0.5.0 | 内联数据绑定 |
| Lateral（`LATERAL`） | ✅ 支持 | v0.22.0 | PostgreSQL `LATERAL JOIN` |
| 子查询 | ✅ 支持 | v0.5.0 | 嵌套 `SELECT` |
| 否定（`NOT EXISTS`） | ✅ 支持 | v0.5.0 | |
| 否定（`EXISTS`） | ✅ 支持 | v0.5.0 | |

---

## SPARQL 1.1 查询 — 属性路径

| 功能 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| 序列路径（`/`） | ✅ 支持 | v0.5.0 | |
| 可选路径（`\|`） | ✅ 支持 | v0.5.0 | |
| 反向路径（`^`） | ✅ 支持 | v0.5.0 | |
| 零或多个（`*`） | ✅ 支持 | v0.5.0 | `WITH RECURSIVE … CYCLE` |
| 一或多个（`+`） | ✅ 支持 | v0.5.0 | `WITH RECURSIVE … CYCLE` |
| 零或一（`?`） | ✅ 支持 | v0.5.0 | |
| 否定属性集（`!(p1\|p2)`） | ✅ 支持 | ✅ 支持 | v0.5.0 |
| 定长路径（`{n}`） | ✅ 支持 | v0.5.0 | 展开为 `n` 个连接 |
| 可变长度路径（`{n,m}`） | ✅ 支持 | v0.5.0 | 有界递归 |

```admonish note title="循环检测"
所有递归属性路径都使用 PostgreSQL 18 的原生 `CYCLE` 子句进行基于哈希的循环检测，由 `pg_ripple.max_path_depth`（默认值：10）限制。
```

---

## SPARQL 1.1 查询 — 聚合

| 功能 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| `COUNT` | ✅ 支持 | v0.5.0 | 包括 `COUNT(DISTINCT *)` |
| `SUM` | ✅ 支持 | v0.5.0 | |
| `AVG` | ✅ 支持 | v0.5.0 | |
| `MIN` | ✅ 支持 | v0.5.0 | |
| `MAX` | ✅ 支持 | v0.5.0 | |
| `GROUP_CONCAT` | ✅ 支持 | v0.5.0 | 带自定义分隔符 |
| `SAMPLE` | ✅ 支持 | v0.5.0 | |

---

## SPARQL 1.1 查询 — 内置函数

| 函数 | 状态 | 起始版本 |
|---|---|---|
| `STR()` | ✅ 支持 | v0.1.0 |
| `LANG()` | ✅ 支持 | v0.3.0 |
| `DATATYPE()` | ✅ 支持 | v0.3.0 |
| `IRI()` / `URI()` | ✅ 支持 | v0.5.0 |
| `BNODE()` | ✅ 支持 | v0.5.0 |
| `RAND()` | ✅ 支持 | v0.5.0 |
| `ABS()` | ✅ 支持 | v0.1.0 |
| `CEIL()` | ✅ 支持 | v0.1.0 |
| `FLOOR()` | ✅ 支持 | v0.1.0 |
| `ROUND()` | ✅ 支持 | v0.1.0 |
| `CONCAT()` | ✅ 支持 | v0.5.0 |
| `STRLEN()` | ✅ 支持 | v0.1.0 |
| `UCASE()` | ✅ 支持 | v0.1.0 |
| `LCASE()` | ✅ 支持 | v0.1.0 |
| `ENCODE_FOR_URI()` | ✅ 支持 | v0.5.0 |
| `CONTAINS()` | ✅ 支持 | v0.1.0 |
| `STRSTARTS()` | ✅ 支持 | v0.1.0 |
| `STRENDS()` | ✅ 支持 | v0.1.0 |
| `STRBEFORE()` | ✅ 支持 | v0.5.0 |
| `STRAFTER()` | ✅ 支持 | v0.5.0 |
| `YEAR()` | ✅ 支持 | v0.5.0 |
| `MONTH()` | ✅ 支持 | v0.5.0 |
| `DAY()` | ✅ 支持 | v0.5.0 |
| `HOURS()` | ✅ 支持 | v0.5.0 |
| `MINUTES()` | ✅ 支持 | v0.5.0 |
| `SECONDS()` | ✅ 支持 | v0.5.0 |
| `TIMEZONE()` | ✅ 支持 | v0.5.0 |
| `TZ()` | ✅ 支持 | v0.5.0 |
| `NOW()` | ✅ 支持 | v0.5.0 |
| `UUID()` | ✅ 支持 | v0.5.0 |
| `STRUUID()` | ✅ 支持 | v0.5.0 |
| `MD5()` | ✅ 支持 | v0.5.0 |
| `SHA1()` | ✅ 支持 | v0.5.0 |
| `SHA256()` | ✅ 支持 | v0.5.0 |
| `SHA384()` | ✅ 支持 | v0.5.0 |
| `SHA512()` | ✅ 支持 | v0.5.0 |
| `COALESCE()` | ✅ 支持 | v0.1.0 |
| `IF()` | ✅ 支持 | v0.1.0 |
| `STRLANG()` | ✅ 支持 | v0.5.0 |
| `STRDT()` | ✅ 支持 | v0.5.0 |
| `isIRI()` / `isURI()` | ✅ 支持 | v0.1.0 |
| `isBlank()` | ✅ 支持 | v0.1.0 |
| `isLiteral()` | ✅ 支持 | v0.1.0 |
| `isNumeric()` | ✅ 支持 | v0.5.0 |
| `REGEX()` | ✅ 支持 | v0.1.0 |
| `REPLACE()` | ✅ 支持 | v0.5.0 |
| `SUBSTR()` | ✅ 支持 | v0.5.0 |
| `BOUND()` | ✅ 支持 | v0.1.0 |
| `IN` / `NOT IN` | ✅ 支持 | v0.5.0 |
| `TRIPLE()` (RDF-star) | ✅ 支持 | v0.4.0 |
| `SUBJECT()` (RDF-star) | ✅ 支持 | v0.4.0 |
| `PREDICATE()` (RDF-star) | ✅ 支持 | v0.4.0 |
| `OBJECT()` (RDF-star) | ✅ 支持 | v0.4.0 |
| `isTRIPLE()` (RDF-star) | ✅ 支持 | v0.4.0 |

---

## SPARQL 1.1 查询 — 类型化字面量

| 数据类型 | 状态 | 备注 |
|---|---|---|
| `xsd:integer` | ✅ 支持 | 映射到 PostgreSQL `BIGINT` |
| `xsd:decimal` | ✅ 支持 | 映射到 `NUMERIC` |
| `xsd:float` | ✅ 支持 | 映射到 `REAL` |
| `xsd:double` | ✅ 支持 | 映射到 `DOUBLE PRECISION` |
| `xsd:boolean` | ✅ 支持 | 映射到 `BOOLEAN` |
| `xsd:string` | ✅ 支持 | 默认字面量类型 |
| `xsd:dateTime` | ✅ 支持 | 映射到 `TIMESTAMPTZ` |
| `xsd:date` | ✅ 支持 | 映射到 `DATE` |
| `xsd:time` | ✅ 支持 | 映射到 `TIME` |
| `xsd:gYear` | ✅ 支持 | 作为字符串存储，按字典顺序比较 |
| 语言标记字符串 | ✅ 支持 | `"text"@en` 语法 |

---

## SPARQL 1.1 更新

| 操作 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| `INSERT DATA` | ✅ 支持 | v0.7.0 | |
| `DELETE DATA` | ✅ 支持 | v0.7.0 | |
| `DELETE WHERE` | ✅ 支持 | v0.7.0 | |
| `DELETE/INSERT WHERE` | ✅ 支持 | v0.7.0 | |
| `INSERT WHERE` | ✅ 支持 | v0.7.0 | |
| `LOAD` | ✅ 支持 | v0.7.0 | 通过 `pg_ripple_http` 或直接文件 |
| `CLEAR GRAPH` | ✅ 支持 | v0.7.0 | |
| `CLEAR DEFAULT` | ✅ 支持 | v0.7.0 | |
| `CLEAR NAMED` | ✅ 支持 | v0.7.0 | |
| `CLEAR ALL` | ✅ 支持 | v0.7.0 | |
| `DROP GRAPH` | ✅ 支持 | v0.7.0 | |
| `DROP DEFAULT` | ✅ 支持 | v0.7.0 | |
| `DROP NAMED` | ✅ 支持 | v0.7.0 | |
| `DROP ALL` | ✅ 支持 | v0.7.0 | |
| `CREATE GRAPH` | ✅ 支持 | v0.7.0 | |
| `CREATE SILENT GRAPH` | ✅ 支持 | v0.7.0 | |
| `COPY` | ✅ 支持 | v0.21.0 | |
| `MOVE` | ✅ 支持 | v0.21.0 | |
| `ADD` | ✅ 支持 | v0.21.0 | |
| 多语句（`;` 分隔符） | ✅ 支持 | v0.7.0 | |
| `USING` / `USING NAMED` | ✅ 支持 | v0.7.0 | 更新的数据集子句 |

---

## SPARQL 1.1 协议

| 功能 | 状态 | 备注 |
|---|---|---|
| 通过 HTTP GET 查询 | ✅ 支持 | 通过 `pg_ripple_http` |
| 通过 HTTP POST 查询（表单编码） | ✅ 支持 | 通过 `pg_ripple_http` |
| 通过 HTTP POST 查询（直接主体） | ✅ 支持 | 通过 `pg_ripple_http` |
| 通过 HTTP POST 更新 | ✅ 支持 | 通过 `pg_ripple_http` |
| 内容协商（`Accept` 标头） | ✅ 支持 | JSON、Turtle、N-Triples、XML |
| `default-graph-uri` 参数 | ✅ 支持 | |
| `named-graph-uri` 参数 | ✅ 支持 | |
| 多个 `default-graph-uri` | ✅ 支持 | |
| 多个 `named-graph-uri` | ✅ 支持 | |

```admonish note title="协议端点"
SPARQL 协议支持需要 `pg_ripple_http` 配套服务。请参阅 [API 和集成](../features/apis-and-integration.md) 以获取设置说明。
```

---

## SPARQL 1.1 服务描述

| 功能 | 状态 | 备注 |
|---|---|---|
| 端点根目录的服务描述 | ✅ 支持 | 通过 `pg_ripple_http` |
| `sd:supportedLanguage` | ✅ 支持 | 报告 SPARQL 1.1 查询和更新 |
| `sd:resultFormat` | ✅ 支持 | JSON、XML、CSV、TSV |
| `sd:defaultDataset` | ✅ 支持 | |
| `sd:feature` | ✅ 支持 | 报告 `sd:UnionDefaultGraph`、`sd:RequiresDataset` |

---

## SPARQL 1.1 图存储 HTTP 协议

| 操作 | 状态 | 备注 |
|---|---|---|
| `GET`（检索图） | ✅ 支持 | 通过 `pg_ripple_http` |
| `PUT`（替换图） | ✅ 支持 | 通过 `pg_ripple_http` |
| `POST`（合并到图） | ✅ 支持 | 通过 `pg_ripple_http` |
| `DELETE`（删除图） | ✅ 支持 | 通过 `pg_ripple_http` |
| `?default` 参数 | ✅ 支持 | |
| `?graph=<uri>` 参数 | ✅ 支持 | |

---

## RDF-star / SPARQL-star

| 功能 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| 引号三元组存储 | ✅ 支持 | v0.4.0 | `qt_s`、`qt_p`、`qt_o` 字典列 |
| BGP 中的引号三元组 | ✅ 支持 | v0.4.0 | 仅基础模式 |
| `TRIPLE()` 构造器 | ✅ 支持 | v0.4.0 | |
| `SUBJECT()`、`PREDICATE()`、`OBJECT()` | ✅ 支持 | v0.4.0 | |
| `isTRIPLE()` | ✅ 支持 | v0.4.0 | |
| 注释语法（`{| \|}`） | ✅ 支持 | v0.4.0 | Turtle-star 和 SPARQL-star |

---

## 超出 W3C 的扩展

pg_ripple 通过附加功能扩展 SPARQL 标准：

| 功能 | 备注 |
|---|---|
| `pg:similar()` 自定义函数 | SPARQL FILTER 中的向量相似度 |
| `pg:fts()` 自定义函数 | SPARQL FILTER 中的全文搜索 |
| `pg:embed()` 自定义函数 | 内联嵌入生成 |
| Datalog 物化谓词 | 通过标准 SPARQL 查询已推导的三元组 |
| SHACL 优化的查询计划 | 来自 SHACL 形状的基数提示 |
| 计划缓存 | 跨查询缓存已编译的 SQL 计划 |

---

## 已知限制

| 功能 | 状态 | 备注 |
|---|---|---|
| `langMatches()` | ⚠️ 部分 | 返回 0 行；完整的 BCP 47 匹配已规划 |
| 自定义聚合操作符扩展 | ❌ 不支持 | 标准聚合完全支持 |
| 引号三元组中的变量 `<< ?s ?p ?o >>` | ⚠️ 部分 | 返回 0 行并发出 WARNING；基础模式有效 |
| 从任意 HTTP 加载 `LOAD <url>` | ⚠️ 取决于 | 需要 `pg_ripple_http` 或服务器端文件 |
| `DESCRIBE` 策略自定义 | ✅ 支持 | 通过 GUC 的四种策略（v0.55.0） |
| `SELECT` 的多种结果格式 | ⚠️ 部分 | JSON 是主要的；XML/CSV/TSV 仅通过 `pg_ripple_http` |

---

## DESCRIBE 策略参考（SC13-04，v0.86.0 — 取代 v0.55.0）

pg_ripple 通过在 v0.86.0 中引入的 **`pg_ripple.describe_form`** GUC（默认值：`cbd`）支持三种可选择的 DESCRIBE 算法。较旧的 `pg_ripple.describe_strategy` GUC 已弃用（请参阅 [deprecated-gucs.md](deprecated-gucs.md)）并将在 v1.0.0 中移除。

### `cbd` — 简洁有界描述（默认）

返回所有在描述资源作为主语时出现的三元组，以及通过递归跟踪空白节点对象可达的所有三元组。这是 W3C 定义的最小 DESCRIBE 语义。

```sql
SET pg_ripple.describe_form = 'cbd';
SELECT * FROM pg_ripple.sparql('DESCRIBE <https://example.org/Alice>');
```

### `scbd` — 对称简洁有界描述

通过还包括描述资源作为**对象**时出现的所有三元组来扩展 CBD。这会捕获传出和传入的边。当您需要资源的完整邻域时适用。

```sql
SET pg_ripple.describe_form = 'scbd';
SELECT * FROM pg_ripple.sparql('DESCRIBE <https://example.org/Alice>');
```

### `symmetric` — `scbd` 的永久别名（CB-10，v0.92.0）

`symmetric` 是 `scbd` 的**永久别名**，以提高可读性：

```sql
SET pg_ripple.describe_form = 'symmetric';
SELECT * FROM pg_ripple.sparql('DESCRIBE <https://example.org/Alice>');
```

> **别名契约**：别名 `symmetric` = `scbd` 保证在 pg_ripple 1.x API 线上保持稳定。除非经过弃用周期，否则不会将其移除，并且仅当 W3C SCBD 和"对称扩展"定义在未来 RDF 标准中分离时，才会与 `scbd` 语义不同。此保证已得到正式记录。

### 选择形式

| 形式 | 传出边 | 传入边 | 空白节点闭包 | 速度 |
|---|---|---|---|---|
| `cbd` | ✅ | ❌ | ✅ | 中等 |
| `scbd` / `symmetric` | ✅ | ✅ | ✅ | 较慢 |

GUC 可以在会话或事务级别设置：

```sql
-- 会话级别
SET pg_ripple.describe_form = 'scbd';

-- 事务级别
BEGIN;
SET LOCAL pg_ripple.describe_form = 'cbd';
SELECT * FROM pg_ripple.sparql('DESCRIBE <https://example.org/Bob>');
COMMIT;
```

> **迁移说明**：将 `SET pg_ripple.describe_strategy = 'simple'` 替换为 `SET pg_ripple.describe_form = 'cbd'`（`simple` 策略现在是 CBD 默认值）。策略 `scbd` 直接映射到 `describe_form = 'scbd'`。

---

## RDF-star 引号三元组中的空白节点限制（C13-03，v0.85.0）

pg_ripple 中 RDF-star 引号三元组内的空白节点（例如 `<< _:b1 :p :o >>）不具有规范的往返形式。当匿名空白节点作为引号三元组的主语或对象出现时，对同一三元组进行编码然后解码可能会产生不同的空白节点标签。

**解决方法：** 使用命名 IRI 或众所周知的空白节点标识符（例如具有稳定标签的 `_:b1`）作为引号三元组的主语/对象。或者，完全避免在 `<< >>` 模式的主语/对象位置中使用空白节点。

**影响：** 此限制仅影响引号三元组中包含空白节点的模式。非引号三元组中的常规空白节点可正确往返。

---

## `GRAPH ?g` 默认图排除（C13-06，v0.85.0）

根据 SPARQL 1.1 规范 §8.3，当使用 `GRAPH ?g { ... }` 时，变量 `?g` 仅绑定到**命名图**三元组。要专门查询默认图，请使用 `WHERE { ... }` 而不包装在 `GRAPH ?g` 子句中。

**示例：**
```sparql
# ❌ 仅匹配命名图（默认图被排除）
SELECT ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } }

# ✅ 也匹配默认图三元组
SELECT ?s ?p ?o WHERE { ?s ?p ?o }
```

---

## `xsd:dateTime` 精度舍入（C13-09，v0.85.0）

当将 `xsd:dateTime` 字面量序列化回 N-Triples 格式时，PostgreSQL 会将它们截断为微秒精度（3 个小数位）。

**示例：**
```
# 输入:  "2024-01-01T12:00:00.123456789Z"^^xsd:dateTime
# 存储:  2024-01-01 12:00:00.123457+00  (由 PG 舍入为微秒)
# 输出:  "2024-01-01T12:00:00.123"^^xsd:dateTime  (截断为 3 个小数位)
```

亚毫秒精度在输出中被静默删除。如果您需要亚毫秒精度，请将该值存储为纯字符串字面量，并手动执行比较。

---

## RDF 1.2 / SPARQL-star 一致性矩阵（STD-02，v0.91.0）

pg_ripple 通过 `oxrdf` 0.3 数据模型和 v0.4.0 中引入的 `qt_s`/`qt_p`/`qt_o` 字典列支持 RDF-star。下表将每个 RDF 1.2 / SPARQL-star 功能映射到其实现状态。

| 功能 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| BGP 中主语位置的 `<< s p o >>` | ✅ 已实现 | v0.4.0 | 通过 `qt_s/qt_p/qt_o` 字典列存储 |
| `BIND` 中的 `<< s p o >>` | ✅ 已实现 | v0.16.0 | 完整表达式支持 |
| `FILTER` 中的 `<< s p o >>` | ✅ 已实现 | v0.16.0 | 比较和 isTriple() |
| `CONSTRUCT` 中的 `<< s p o >>` | ✅ 已实现 | v0.16.0 | 作为 Turtle-star 发出 |
| `SELECT` 投影中的 `<< s p o >>` | ✅ 已实现 | v0.16.0 | |
| 注释语法 `{| p o \|}` | ⚠ 部分 | v0.16.0 | 仅解析；尚不支持通过 `INSERT DATA` 的 SPARQL 写入 |
| `TRIPLE(s, p, o)` 构造器函数 | ❌ 未实现 | — | 取决于 spargebra SPARQL 1.2 语法更新 |
| `SUBJECT()` / `PREDICATE()` / `OBJECT()` 解构 | ❌ 未实现 | — | 需要字典连接；计划在 spargebra-1.2 之后 |
| `REIF` 关键字（RDF 1.2 具体化语法） | ❌ 未开始 | — | 需要 spargebra 语法更新 |
| `isTriple()` 函数 | ✅ 已实现 | v0.16.0 | 对于引号三元组主语返回 true |

**总体而言**：pg_ripple 的 RDF-star 基础（v0.4.0）涵盖了最广泛使用的 SPARQL-star 模式。剩余的差距取决于 `spargebra` 0.x 采用 SPARQL 1.2 语法，并被跟踪为 v1.0.0 之后的工作（请参阅 [SPARQL 1.2 tracking](../../../plans/sparql12_tracking.md)）。

---

## `<<` `>>` 位置支持矩阵（L15-08，v0.97.0）

此表总结了引号三元组模式 `<< s p o >>` 可以在所有 SPARQL 1.1 查询和更新形式中出现的哪些位置。在编写将 RDF-star 与其他功能结合的查询时，请使用此矩阵。

| `<< s p o >>` 位置 | 状态 | 起始版本 | 备注 |
|---|---|---|---|
| BGP 中的主语位置 | ✅ 支持 | v0.4.0 | 通过 `qt_s/qt_p/qt_o` 字典列存储 |
| BGP 中的对象位置 | ✅ 支持 | v0.4.0 | |
| `BIND` 中引号三元组的主语 | ✅ 支持 | v0.16.0 | 完整表达式支持 |
| 嵌套引号三元组 `<< << s p o >> p2 o2 >>` | ✅ 支持 | v0.4.0 | 字典递归 |
| `FILTER` 表达式中 | ✅ 支持 | v0.16.0 | `isTriple()`、比较运算符 |
| `CONSTRUCT` 头部中 | ✅ 支持 | v0.16.0 | 作为 Turtle-star 发出 |
| `SELECT` 投影中 | ✅ 支持 | v0.16.0 | |
| `INSERT DATA` 中 | ✅ 支持 | v0.16.0 | 写入 `qt_s/qt_p/qt_o` 列 |
| `DELETE DATA` 中 | ✅ 支持 | v0.16.0 | 通过字典查找匹配并删除 |
| 引号三元组中的变量 `<< ?s ?p ?o >>` | ⚠️ 部分 | v0.16.0 | 返回 0 行并发出 WARNING；基础模式有效 |
| `<< >>` 中的空白节点 | ⚠️ 部分 | v0.4.0 | 没有文档范围隔离；在多加载方案中避免 |
| `VALUES` 子句中 | ✅ 支持 | v0.25.0 | 仅基础引号三元组 |
| `GRAPH` 子句中 | ✅ 支持 | v0.4.0 | 命名图 + 引号三元组组合 |
| `UNION` 分支中 | ✅ 支持 | v0.16.0 | |
| `OPTIONAL` 中 | ✅ 支持 | v0.16.0 | |
| 属性路径中 | ❌ 不支持 | — | `<< >>` 中的路径表达式尚未处理 |
| `SERVICE`（联邦）中 | ✅ 支持 | v0.4.0 | 按原样转发到远程端点 |
| `GROUP BY` / `HAVING` 中 | ✅ 支持 | v0.16.0 | |
| `ORDER BY` 中 | ✅ 支持 | v0.16.0 | 序列化形式的字典顺序 |