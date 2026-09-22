# 规则库

> **版本**：v0.104.0
> **基础**：[plans/expert-system.md §5](https://github.com/trickle-labs/pg-ripple/blob/main/plans/expert-system.md)

**规则库**是 Datalog 推理规则和 SHACL 验证形状的版本化、可分发包。规则库允许团队共享领域特定的推理逻辑 —— 患者匹配规则、反洗钱模式、供应链约束 —— 而无需每个团队从头开始重新实现相同的逻辑。

规则库类似于 npm 包或 pip 包，但是是用于知识图谱推理的。

---

## 库格式

规则库是具有以下结构的单个 Turtle（`.ttl`）文件：

```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix pg: <http://pg-ripple.org/lib/> .
@prefix sh: <http://www.w3.org/ns/shacl#> .

# ── 必需：库声明 ────────────────────────────────────────────

<urn:my-org:my-library> a pg:RuleLibrary ;
    dcterms:title       "my-library" ;
    dcterms:description "Human-readable description of what the library does." ;
    dcterms:license     <https://spdx.org/licenses/MIT.html> ;
    owl:versionInfo     "1.2.0" ;

    # ── 可选：依赖项 ───────────────────────────────────────────────

    # 将另一个库声明为依赖项（其 URL 或本地路径）。
    pg:dependsOn <https://example.org/shared-library.ttl> ;

    # ── 可选：Datalog 规则 ──────────────────────────────────────────────

    pg:rules """
        ?x <https://schema.org/ancestor> ?z :-
            ?x <https://schema.org/parent> ?z .

        ?x <https://schema.org/ancestor> ?z :-
            ?x <https://schema.org/parent> ?y ,
            ?y <https://schema.org/ancestor> ?z .
    """ .

# ── 可选：SHACL 形状 ───────────────────────────────────────────

<https://my-org.example/shapes/PersonShape>
    a sh:NodeShape ;
    sh:targetClass <https://schema.org/Person> ;
    sh:property [
        sh:path <https://schema.org/name> ;
        sh:minCount 1 ;
        sh:datatype <http://www.w3.org/2001/XMLSchema#string> ;
    ] .
```

### 必需的元数据三元组

这四个三元组都**必须**出现在 `pg:RuleLibrary` 主语上：

| 属性 | 值 |
|---|---|
| `dcterms:title` | 用作目录中库名称的短标识符（纯字符串） |
| `dcterms:description` | 关于库做什么的可读性解释 |
| `dcterms:license` | SPDX 许可证 IRI（请参阅[许可证要求](#许可证要求)） |
| `owl:versionInfo` | 版本字符串（例如 `"1.2.0"`） |

### 可选三元组

| 属性 | 值 |
|---|---|
| `pg:rules` | 包含一个或多个 Datalog 规则体的 `xsd:string` 字面量 |
| `pg:dependsOn` | 依赖库的 IRI（URL 或本地路径）；可以出现多次 |
| SHACL 形状 | 同一文件中的任何 `sh:NodeShape` / `sh:PropertyShape` 资源 |

---

## 安装库

```sql
-- 从本地文件安装（绝对路径）：
SELECT pg_ripple.install_rule_library('/opt/pg_ripple/libraries/my-library.ttl');

-- 从 HTTPS URL 安装：
SELECT pg_ripple.install_rule_library(
    'https://libraries.example.org/my-library-1.2.0.ttl'
);
```

`install_rule_library` 在成功时返回库名称。

**幂等性**：重新安装相同的名称 + 版本是无操作的。函数返回名称，没有错误或重复。

### 接受非宽松许可证

如果库携带非宽松许可证（MIT、Apache-2.0 或 PostgreSQL License 以外的任何内容），您必须明确确认这一点：

```sql
SELECT pg_ripple.install_rule_library(
    '/opt/libraries/gpl-library.ttl',
    accept_license => true
);
```

没有 `accept_license => true`，`install_rule_library` 引发 **PT0455** 并在不加载库的情况下退出。

---

## 列出已安装的库

```sql
SELECT * FROM pg_ripple.list_rule_libraries();
```

| 列 | 类型 | 说明 |
|---|---|---|
| `name` | `TEXT` | 库标识符（来自 `dcterms:title`） |
| `version` | `TEXT` | 来自 `owl:versionInfo` 的版本 |
| `installed_at` | `TEXT` | 安装时间戳 |
| `description` | `TEXT` | 库描述 |
| `license_iri` | `TEXT` | SPDX 许可证 IRI |

---

## 升级库

```sql
SELECT pg_ripple.upgrade_rule_library('my-library');
```

`upgrade_rule_library` 从其原始 `source_url` 重新获取库，替换所有已安装的规则和形状，并更新目录中的版本。

**PT0456**：如果另一个已安装的库依赖于 `my-library`，则升级被拒绝 —— 首先卸载依赖库。

---

## 卸载库

```sql
SELECT pg_ripple.uninstall_rule_library('my-library');
```

这将删除：
- 库规则集名称的所有 Datalog 规则。
- 从库的 Turtle 文件加载的所有 SHACL 形状。
- `_pg_ripple.rule_libraries` 中的目录行。

**PT0456**：当另一个已安装的库将 `my-library` 声明为依赖项时被拒绝。首先卸载依赖库。

---

## REST API

`pg_ripple_http` 配套服务公开了一个只读端点：

```http
GET /rule-libraries
Authorization: Bearer <token>
```

返回已安装库对象的 JSON 数组：

```json
[
  {
    "name": "my-library",
    "version": "1.2.0",
    "installed_at": "2026-05-10 12:00:00",
    "description": "Human-readable description.",
    "license_iri": "https://spdx.org/licenses/MIT.html"
  }
]
```

---

## 依赖项解析

使用 `pg:dependsOn` 声明的依赖项按**拓扑顺序递归解析**：依赖库在需要它们的库之前安装。

**约束**：
- 依赖图必须**无环**。循环引发 **PT0453**。
- 仅支持**单一版本**依赖项 —— 不支持 semver 范围。
- 如果无法获取声明的依赖项 URL，则引发 **PT0454**。

---

## 许可证要求

pg_ripple 区分**宽松**和**非宽松**许可证：

| 许可证 | SPDX IRI | 自动接受 |
|---|---|---|
| MIT | `https://spdx.org/licenses/MIT.html` | ✅ 是 |
| Apache-2.0 | `https://spdx.org/licenses/Apache-2.0.html` | ✅ 是 |
| PostgreSQL License | `https://spdx.org/licenses/PostgreSQL.html` | ✅ 是 |
| 任何其他 | — | ❌ 需要 `accept_license => true` |

### 操作员责任

在使用 `accept_license => true` 安装库之前：

1. **阅读 SPDX IRI 处的许可证文本**。
2. **咨询您的法律团队**，如果该库将在商业产品中使用或作为服务的一部分分发。
3. **记录**变更管理日志中已接受的许可证决策。
4. **审查库内容** —— pg_ripple 不会审计库规则的正确性、安全性或法规合规性。

pg_ripple 不提供法律建议。`accept_license` 标志是一种技术确认机制，而非法律保证。

---

## SSRF 保护

基于 URL 的库源在发出任何网络请求之前会根据联邦 SSRF 允许列表进行验证。该检查遵循与 SPARQL 查询中的 `SERVICE` 端点相同的策略：

- **`default-deny`**（默认）：阻止 RFC-1918、loopback、link-local 地址。
- **`allowlist`**：仅允许 `pg_ripple.federation_allowed_endpoints` 中列出的 URL。
- **`open`**：无限制（仅用于开发/测试）。

被阻止的 URL 引发 **PT0452**。

---

## 编写库

### 1. 编写 Turtle 文件

遵循上面的[库格式](#库格式)。规则必须是有效的 pg_ripple Datalog 语法 —— 首先使用 `pg_ripple.load_rules()` 测试它们。

### 2. 本地验证

```bash
# 测试规则语法：
psql -c "SELECT pg_ripple.load_rules(\$\$<rules here>\$\$, 'test')"

# 从本地路径测试安装：
psql -c "SELECT pg_ripple.install_rule_library('/path/to/my-library.ttl')"

# 验证它出现在目录中：
psql -c "SELECT * FROM pg_ripple.list_rule_libraries()"

# 测试后卸载：
psql -c "SELECT pg_ripple.uninstall_rule_library('my-library')"
```

### 3. 发布

规则库可以通过以下方式发布：
- HTTPS 服务器上的文件。
- 带有原始 URL 的 GitHub releases。
- OCI 工件或 Helm config map（用于 Kubernetes 部署）。

没有中央注册表 —— 操作员从他们信任的任何 URL 或本地路径安装。

### 4. 版本控制

在 `owl:versionInfo` 中使用语义版本控制（`MAJOR.MINOR.PATCH`）。当您发布新版本时，更新 Turtle 文件并重新发布。用户使用 `upgrade_rule_library()` 进行升级。

---

## 错误参考

| 代码 | 何时 | 消息 |
|---|---|---|
| PT0452 | URL 被 SSRF 允许列表阻止 | `install_rule_library: URL '...' is blocked by the SSRF allowlist` |
| PT0453 | 检测到循环依赖 | `install_rule_library: dependency cycle detected involving library '...'` |
| PT0454 | 依赖项 URL 不可访问 | `install_rule_library: dependency '...' could not be fetched` |
| PT0455 | 非宽松许可证，未接受 | `install_rule_library: library '...' uses license '...' — set accept_license => TRUE` |
| PT0456 | 依赖库阻止操作 | `upgrade/uninstall_rule_library: library '...' is required by '...' — uninstall the dependent first` |
| PT0459 | 名称与内置 bundle 冲突 | `install_rule_library: name '...' conflicts with a built-in bundle` |

---

## 与内置 Bundle 的关系

pg_ripple 附带几个**内置 bundle**（SKOS、OWL RL、RDFS、DCTERMS、Schema.org、FOAF），可通过 `load_datalog_bundle()` 和 `load_shape_bundle()` 访问。这些被编译到扩展二进制中。

外部**规则库**（此功能）是用户可扩展的对应物：它们是在运行时加载的 Turtle 文件，并存储在 `_pg_ripple.rule_libraries` 目录中。

两个系统共存。规则库可以使用 `pg:dependsOn "skos"` 将内置 bundle 声明为依赖项 —— `install_rule_library` 将在加载库自身规则之前通过 `load_datalog_bundle()` 激活该 bundle。

与内置 bundle 名称冲突的库名称将被 PT0459 拒绝。