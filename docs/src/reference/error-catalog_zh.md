# 错误消息目录

pg_ripple 使用 **PT001–PT799** 范围内的结构化错误代码，按子系统组织。错误消息遵循 PostgreSQL 约定：首单词小写，无尾随句点。

```admonish tip title="查找错误代码"
错误代码出现在 PostgreSQL 错误消息的 `DETAIL` 字段中。在 psql 中使用 `\errverbose` 查看包括代码在内的完整错误上下文。
```

---

## PT001–PT099：字典

来自 IRI/字面值/空白节点 → 整数编码子系统的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT001 | dictionary encode failed: hash collision detected | 两个不同的术语产生了相同的 XXH3-128 哈希（极罕见） | 将两个冲突的术语报告给维护者 |
| PT002 | dictionary decode failed: id not found | 整数 ID 不存在于 `_pg_ripple.dictionary` 中 | 数据可能已损坏；运行 `pg_ripple.vacuum_dictionary()` 并检查 VP 表 |
| PT003 | invalid term kind: expected 0 (IRI), 1 (literal), 2 (blank node) | 传递给 `encode_term()` 的 `kind` 整数错误 | IRI 用 0，字面值用 1，空白节点用 2 |
| PT004 | quoted triple components not found | 引号三元组 ID 引用了字典中缺失的 `qt_s`/`qt_p`/`qt_o` 值 | 重新加载 RDF-star 数据；可能表示部分加载失败 |
| PT005 | inline-encoded literal decode failed | 小型内联编码字面值的内部解码错误 | 将字面值报告给维护者 |
| PT006 | dictionary batch insert failed | `ON CONFLICT DO NOTHING … RETURNING` 批量插入遇到意外错误 | 检查 PostgreSQL 日志中的磁盘空间或权限问题 |
| PT007 | dictionary lookup: NULL term | 在期望 IRI、字面值或空白节点的位置传递了 NULL 值 | 确保所有参数都是非 NULL |
| PT008 | malformed IRI: `<detail>` | IRI 字符串不符合 RFC 3987 | 修复 IRI 语法；IRI 必须用尖括号 `<…>` 包裹 |
| PT009 | malformed literal: `<detail>` | 字面值字符串无法解析 | 使用 N-Triples 语法：`"value"`、`"value"@lang` 或 `"value"^^<datatype>` |
| PT010 | malformed blank node: `<detail>` | 空白节点标签无效 | 空白节点必须以 `_:` 开头，后跟有效标签 |
| PT011 | dictionary cache full, eviction failed | LRU 缓存无法驱逐条目 | 增加 `pg_ripple.dictionary_cache_size` |
| PT012 | prewarm_dictionary_hot: table not found | 字典表不存在 | 首先运行 `CREATE EXTENSION pg_ripple` |

---

## PT100–PT199：存储

来自 VP 表存储层、HTAP 分区和稀有谓词管理的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT100 | insert_triple: predicate IRI required | 谓词参数为 NULL 或空 | 提供有效的谓词 IRI |
| PT101 | VP table creation failed | 创建新 VP 表时出现 DDL 错误 | 在 `pg_log` 中检查底层 PostgreSQL 错误 |
| PT102 | htap_migrate_predicate: predicate not found | 谓词 ID 不存在于 `_pg_ripple.predicates` 中 | 验证谓词 IRI 并确认存在其三元组 |
| PT103 | merge: lock_timeout exceeded during main table swap | 另一个事务持有 VP 表上的锁时间过长 | 重试；考虑为维护窗口增加 `lock_timeout` |
| PT104 | rare-predicate promotion failed | 将谓词从 `vp_rare` 提升到专用 VP 表时出错 | 检查磁盘空间和用户权限 |
| PT105 | delete_triple: predicate not found in catalog | 三元组的谓词没有 VP 表 | 谓词可能从未被使用过，或已被压缩 |
| PT106 | VP table not found: `<table_name>` | `_pg_ripple.predicates` 中引用的 VP 表在磁盘上不存在 | 运行 `pg_ripple.compact()` 协调目录 |
| PT107 | delta table insert failed | 写入 HTAP delta 分区时出错 | 检查 PostgreSQL 日志中的表空间或权限问题 |
| PT108 | tombstone insert failed | 在墓碑表中记录删除时出错 | 检查 PostgreSQL 日志 |
| PT109 | merge worker: unexpected state | 后台合并工作线程遇到不一致状态 | 重启 PostgreSQL；在 `pg_log` 中检查崩溃详情 |
| PT110 | statement_id_seq: sequence exhausted | 全局语句 ID 序列已达到最大值 | 这在 `BIGINT` 下不太可能；请联系维护者 |
| PT111 | vp_rare: row limit exceeded | 稀有谓词表中的单个谓词行数过多 | 使用 `promote_rare_predicates()` 手动提升或降低 `pg_ripple.vp_promotion_threshold` |
| PT112 | deduplicate: advisory lock not acquired | 另一个去重操作已在运行 | 等待并重试 |
| PT113 | create_graph: invalid graph IRI | 图 IRI 格式错误 | 图 IRI 必须是尖括号中的有效绝对 IRI |
| PT114 | drop_graph: graph not found | 命名图不存在 | 使用 `list_graphs()` 检查可用的图 |

---

## PT200–PT299：SPARQL

来自 SPARQL 解析器、代数优化器和 SQL 代码生成器的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT200 | SPARQL parse error: `<detail>` | SPARQL 查询有语法错误 | 修复语法；使用 `sparql_explain()` 在不执行的情况下验证 |
| PT201 | unsupported SPARQL algebra node: `<type>` | 查询使用了尚未实现的功能 | 检查[合规性矩阵](sparql-compliance.md)了解支持的功能 |
| PT202 | SPARQL SELECT: no projected variables | SELECT 子句没有变量 | 至少向 SELECT 子句添加一个 `?variable` |
| PT203 | property path depth exceeded `max_path_depth` | 递归属性路径超过了配置的深度限制 | 增加 `pg_ripple.max_path_depth` 或简化路径表达式 |
| PT204 | SPARQL federated SERVICE: endpoint not reachable | 远程 SPARQL 端点没有响应 | 检查端点 URL 和网络连接 |
| PT205 | SPARQL VALUES clause: column count mismatch | VALUES 行中的值数与变量列表不匹配 | 确保每个 VALUES 行的列数与变量相同 |
| PT206 | SPARQL type error: `<detail>` | FILTER 表达式中的类型不匹配 | 检查操作数类型；例如，将字符串与整数比较 |
| PT207 | SPARQL CONSTRUCT: template variable not in WHERE | CONSTRUCT 模板中的变量未在 WHERE 子句中绑定 | 在 WHERE 子句中绑定所有模板变量 |
| PT208 | SPARQL DESCRIBE: no resource specified | DESCRIBE 至少需要一个资源或变量 | 向 DESCRIBE 子句添加资源 IRI 或变量 |
| PT209 | SPARQL aggregate: variable not grouped | 非聚合变量在 GROUP BY 之外使用 | 将变量添加到 GROUP BY 或用聚合函数包装它 |
| PT210 | SPARQL HAVING: refers to non-aggregate | HAVING 子句引用了不是聚合结果的变量 | 在 HAVING 中使用聚合函数 |
| PT211 | generated SQL execution failed: `<detail>` | 从 SPARQL 生成的 SQL 执行失败 | 在 `pg_log` 中检查底层错误；如果可重现则报告 |
| PT212 | plan cache: entry evicted during execution | 缓存的计划在查询仍在运行时被驱逐 | 增加 `pg_ripple.plan_cache_size` |
| PT213 | SPARQL SERVICE: response parse error | 联邦端点返回了格式错误的结果 | 检查远程端点的响应格式 |
| PT214 | SPARQL SERVICE: timeout after `<N>` ms | 联邦请求超过了 `pg_ripple.federation_timeout` | 增加超时或简化 SERVICE 查询 |
| PT215 | SPARQL UPDATE parse error: `<detail>` | SPARQL Update 语句有语法错误 | 修复语法 |
| PT216 | SPARQL UPDATE: LOAD failed for `<url>` | LOAD 操作无法检索远程资源 | 检查 URL、网络和 `pg_ripple.federation_timeout` |
| PT217 | SPARQL UPDATE: unsupported content type `<type>` | LOAD 目标提供无法识别的 RDF 格式 | URL 必须提供 Turtle、N-Triples、N-Quads、TriG 或 RDF/XML |
| PT218 | SPARQL UPDATE: CREATE GRAPH already exists | 图已存在 | 使用 `CREATE SILENT GRAPH` 抑制此错误 |
| PT219 | SPARQL UPDATE: DROP GRAPH not found | 图不存在 | 使用 `DROP SILENT GRAPH` 抑制此错误 |

### 类型化绑定和前缀管理（v0.135.0）

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT0570 | bindings must be a JSON object | 初始绑定负载不是对象 | 发送一个以变量名作为键的 JSON 对象 |
| PT0571 | invalid, duplicate, or excessive binding variable | 变量名无效、规范化后重复或超过配置的限制 | 使用有效且唯一的变量，并根据需要调整 `pg_ripple.sparql_max_initial_bindings` |
| PT0572 | malformed binding term | 术语描述符为 null、缺少字段或具有未知字段 | 使用 W3C `type` 和 `value` 字段 |
| PT0573 | invalid IRI, datatype, or language tag | IRI、数据类型或语言标记无效 | 提供绝对 IRI 和有效的语言标记 |
| PT0574 | binding value exceeds the configured byte limit | URI 或字面值过大 | 减少值或调整 `pg_ripple.sparql_max_binding_value_bytes` |
| PT0575 | blank-node and RDF-star bindings are unsupported | 绑定使用不支持的 RDF 术语类型 | 使用 URI 或字面值 |
| PT0576 | binding variable is outside query scope | 查询未公开请求的变量 | 在查询模式或投影中绑定变量 |
| PT0577 | invalid registered prefix expansion | 前缀标签或扩展格式错误 | 注册有效的前缀和绝对 IRI |
| PT0578 | prefix mutation is not authorized | 调用方不是允许的注册表管理员 | 使用数据库所有者、超级用户或 `pg_ripple_admin` |
| PT0579 | bindings are not supported for this query form | 请求是 Update 或不支持的查询形式 | 使用 SELECT、ASK、CONSTRUCT 或 DESCRIBE 且不带 Update 绑定 |

---

## PT300–PT399：SHACL

来自 SHACL 形状加载器、验证器和异步监控管道的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT300 | SHACL parse error: `<detail>` | Turtle 编码的 SHACL 形状有语法错误 | 修复形状定义中的 Turtle 语法 |
| PT301 | SHACL sync validation failed: `<shape>` — `<message>` | 三元组在同步验证期间违反了 SHACL 约束 | 修复数据以符合形状，或修改形状 |
| PT302 | SHACL shape not found: `<iri>` | 引用的形状尚未加载 | 首先使用 `load_shacl()` 加载形状 |
| PT303 | SHACL DAG monitor: pg_trickle not installed | DAG 感知监控器需要 pg_trickle 扩展 | 安装 pg_trickle 或使用 `enable_shacl_monitors()` 进行基于触发器的验证 |
| PT304 | SHACL: unsupported constraint component `<type>` | 形状使用 SHACL-AF 或 SHACL-JS 约束 | 仅支持 SHACL Core 约束 |
| PT305 | SHACL: sh:path too complex | 形状中的属性路径超过了支持的复杂性 | 简化 `sh:path` 表达式 |
| PT306 | SHACL: validation queue overflow | 异步验证队列已超过其容量 | 使用 `process_validation_queue()` 处理队列或增加队列大小 |
| PT307 | SHACL: dead letter queue threshold reached | 累积了太多验证失败 | 使用 `dead_letter_queue()` 检查并处理失败 |
| PT308 | SHACL: sh:targetClass not found | 数据中不存在目标类 IRI | 加载具有目标类的数据，或修复类 IRI |
| PT309 | SHACL: circular shape reference | 形状通过 `sh:node` 或 `sh:qualifiedValueShape` 引用自身 | 打破循环引用 |
| PT310 | SHACL: drop_shape: shape has active monitors | 无法删除具有活动监控器的形状 | 首先使用 `disable_shacl_dag_monitors()` 禁用监控器 |

---

## PT400–PT499：Datalog — 规则

来自 Datalog 规则解析器、分层器和规则管理的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT400 | rule parse error: `<detail>` | Datalog 规则有语法错误 | 修复规则语法；请参阅[推理和推导](../features/reasoning-and-inference.md)了解语法参考 |
| PT401 | rule stratification failed: unstratifiable program | 规则集包含通过否定的循环，无法分层 | 重写规则以打破否定循环，或使用 `infer_wfs()` 处理良好基础语义 |
| PT402 | rule set not found: `<name>` | 引用的规则集尚未加载 | 使用 `load_rules()` 或 `load_rules_builtin()` 加载它 |
| PT403 | inference: maximum iteration depth exceeded | 半朴素求值在迭代限制内未收敛 | 简化规则集或增加 `statement_timeout` |
| PT404 | constraint violation detected: `<rule>` | 约束规则（`:- body.`）触发 | 根据约束主体检查数据 |
| PT405 | rule set already exists: `<name>` | 同名规则集已加载 | 首先使用 `drop_rules()` 删除它，或选择不同的名称 |
| PT406 | rule: unsafe variable `<var>` | 头部中出现的变量不在正文字面量中 | 确保每个头部变量也出现在正文字面量中 |
| PT407 | rule: built-in predicate not recognized: `<name>` | 使用了未知的内建谓词 | 检查可用的内建谓词：`=`、`!=`、`<`、`>`、`<=`、`>=`、`+`、`-`、`*`、`/` |
| PT408 | rule: aggregation variable not in group-by | 聚合变量在分组上下文之外使用 | 将变量添加到 group-by 列表 |
| PT440 | SPARQL query exceeds algebra depth or pattern limit | 查询过于复杂：其代数树深度超过 `pg_ripple.sparql_max_algebra_depth`，或包含的三元组模式多于 `pg_ripple.sparql_max_triple_patterns` | 简化查询，将其拆分为较小的查询，或增加限制 |
| PT480 | SHACL-AF sh:rule not compiled | 形状中找到了 `sh:rule` 三元组，但 Datalog 推导被禁用或桥接失败 | 使用 `pg_ripple.datalog_inference = 'on'` 启用推导，或删除 `sh:rule` 三元组 |
| PT481 | SHACL-SPARQL constraint query failed | `sh:sparql` 约束的嵌入式 SPARQL 查询无法执行 | 检查嵌入式 SPARQL 语法，并确保所有前缀在形状文档中都已声明 |
| PT482 | SHACL-AF sh:rule compilation failed | `sh:rule` 主体无法编译为 Datalog 规则，已被跳过 | 检查规则主体是否有不支持的构造；该规则未注册 |

---

## PT500–PT599：Datalog — 推导引擎

来自具体化引擎、magic sets 优化器、WFS 求值器和列表化的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT500 | infer: no enabled rule sets | 调用 `infer()` 时未启用任何规则集 | 使用 `enable_rule_set()` 至少启用一个规则集 |
| PT501 | infer: SPI execution failed during iteration `<N>` | 为规则主体生成的 SQL 失败 | 在 `pg_log` 中检查底层错误 |
| PT502 | infer_demand: magic set rewriting failed | 无法应用需求转换 | 简化目标模式或规则集 |
| PT503 | infer_demand: goal pattern too broad | 目标没有绑定参数，违背了需求驱动求值的目的 | 在目标中绑定至少一个参数 |
| PT504 | infer_wfs: unfounded set computation exceeded limit | 良好基础语义不动点未收敛 | 简化规则集或检查异常的否定模式 |
| PT505 | infer_wfs: three-valued model contains undefined atoms | 某些原子无法分类为真或假 | 这在 WFS 中是预期的；查询 `undefined` 结果集以查看哪些原子 |
| PT506 | tabling: memo store overflow | 列表化备忘录存储超过其大小限制 | 增加 `pg_ripple.tabling_memo_size` |
| PT507 | infer_agg: aggregation cycle detected | 聚合规则依赖于自身的聚合结果 | 重写以打破循环 |
| PT508 | infer_goal: predicate not in any rule set | 目标谓词未由任何已加载的规则定义 | 加载定义该谓词的规则集 |
| PT509 | owl:sameAs canonicalization: cycle limit exceeded | `owl:sameAs` 等价类合并超过了迭代限制 | 检查非常大的 `owl:sameAs` 集群 |
| PT510 | infer_agg: aggregation-stratification violation | 规则的聚合直接或通过递归依赖于自身的派生结果 | 重写规则，使聚合谓词不在派生它们的规则集中重复使用 |
| PT511 | infer_agg: unsupported aggregate function | 规则使用了 SQL 编译器不支持的聚合 | 使用 `COUNT`、`SUM`、`MIN`、`MAX`、`AVG` 之一 |
| PT520 | infer_wfs: iteration cap reached (`<N>` iterations) | WFS 交替不动点在 `pg_ripple.wfs_max_iterations` 内未收敛 | 作为 WARNING 发出；返回带有 `"stratifiable": false` 的部分结果；增加上限或简化规则集 |
| PT530 | DRed cycle detected: `<rule_set>` | Delete-Rederive 检测到规则推导图中无法安全解析的循环；系统回退到完全重新计算 | 这是 WARNING，不是错误；操作仍然成功。减少规则集中的循环依赖，或设置 `pg_ripple.dred_enabled = off` 以始终使用完全重新计算 |
| PT540 | lattice: fixpoint did not converge after `<N>` iterations | lattice 不动点在 `pg_ripple.lattice_max_iterations` 内未稳定 | 增加 `pg_ripple.lattice_max_iterations` 或验证 join 函数是否单调 |
| PT541 | lattice: join_fn `<name>` could not be resolved | 用户提供的 join 函数名无法通过 `regprocedure` 解析 | 检查函数名、模式和参数类型；使用完全限定名 |
| PT542 | federation: result decoder received unparseable XML/JSON | 来自远程 SERVICE 端点的 SPARQL 结果响应无法解析 | 检查端点的响应格式；确保它返回 `application/sparql-results+xml` 或 `+json` |
| PT543 | federation response body exceeds `federation_max_response_bytes` | 来自远程 SERVICE 端点的 HTTP 响应主体大于配置的限制 | 增加 `pg_ripple.federation_max_response_bytes`，或将其设置为 `-1` 以禁用；考虑改为在远程端点进行过滤 |
| PT550 | owl:sameAs cluster too large: `<size>` members | `owl:sameAs` 等价类超过 `pg_ripple.sameas_max_cluster_size`；此集群跳过规范化 | 检查数据中是否有虚假的 `owl:sameAs` 三元组；如果集群是合法的，则增加限制 |

---

## PT600–PT699：导出 / HTTP

来自导出序列化器、GraphRAG 导出和 HTTP 配套服务的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT600 | export: serialization failed for triple `<sid>` | 三元组无法序列化为目标格式 | 检查三元组的字典条目是否完整 |
| PT601 | export: unsupported format `<format>` | 请求了无法识别的导出格式 | 使用 `ntriples`、`nquads`、`turtle` 或 `jsonld` |
| PT602 | export_turtle_stream: batch_size must be > 0 | 无效的批处理大小 | 使用正整数 |
| PT603 | export_jsonld: framing failed | JSON-LD 框架算法遇到错误 | 检查框架结构；请参阅[JSON-LD 框架](../features/exporting-and-sharing.md) |
| PT604 | export_graphrag_entities: no entities found | 没有实体符合 GraphRAG 导出条件 | 加载数据或调整 GraphRAG 本体 |
| PT605 | jsonld_frame_to_sparql: invalid frame | JSON-LD 框架无法转换为 SPARQL | 检查框架 JSON 结构 |
| PT606 | SERVICE endpoint blocked by federation_endpoint_policy | 联邦端点 URL 被 `pg_ripple.federation_endpoint_policy` 阻止（v0.55.0）。被阻止的目标包括 `default-deny` 模式下的 RFC-1918 地址、loopback、link-local 和 `file://` URL，或 `allowlist` 模式下不在 `federation_allowed_endpoints` 中的 URL | 设置 `federation_endpoint_policy = 'open'`（仅开发），将 URL 添加到 `federation_allowed_endpoints`，或使用公共端点。也用作流式中断代码（v0.48.0 含义：流式导出已取消） |
| PT607 | vector service endpoint not registered | 向量端点 URL 未在 `_pg_ripple.federation_endpoints` 中注册 | 使用 `pg_ripple.register_endpoint()` 注册端点 |
| PT620 | SERVICE result set exceeds inline limit | 远程 SERVICE 返回的行数超过 `pg_ripple.federation_inline_max_rows`；结果通过临时表具体化 | 增加 `federation_inline_max_rows` 或在远程端点进行过滤 |
| PT621 | register_endpoint: private IP rejected | 端点 URL 解析为私有/loopback IP 且 `pg_ripple.federation_allow_private = off` | 设置 `federation_allow_private = on`（仅开发）或使用公共端点 |
| PT640 | SPARQL result set exceeded sparql_max_rows | 结果集过大；截断为 `pg_ripple.sparql_max_rows` | 增加 `pg_ripple.sparql_max_rows` 或向查询添加 LIMIT/OFFSET |
| PT642 | export truncated to `<N>` rows | 流式导出达到 `pg_ripple.export_max_rows` | 增加 `pg_ripple.export_max_rows` 或使用 OFFSET 分页 |

---

## PT700–PT799：配置 / 启动

来自扩展初始化、GUC 验证和后台工作线程的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT700 | LLM endpoint unreachable or returned HTTP error: `<detail>` | `pg_ripple.llm_endpoint` 为空，或对 LLM 的 HTTP 调用失败 | 设置有效的端点 URL；检查网络访问和 API 密钥 |
| PT701 | LLM response did not contain a valid SPARQL query | LLM 返回了不是 SPARQL 查询的文本 | 通过 `add_llm_example()` 添加 few-shot 示例；切换到更强大的模型 |
| PT702 | LLM-generated SPARQL query failed to parse: `<parse_error>` | 生成的 SPARQL 字符串无法被 spargebra 解析 | 为此问题模式添加 few-shot 示例；使用针对 SPARQL 微调的模型 |
| PT703 | merge worker watchdog: worker has been silent for `<N>` seconds | 后台合并工作线程可能已崩溃 | 在 `pg_log` 中检查崩溃详情；重启 PostgreSQL |
| PT704 | extension version mismatch: binary `<v1>`, control `<v2>` | 编译的扩展版本与 `pg_ripple.control` 不匹配 | 重建并重新安装扩展 |
| PT705 | GUC validation: `<param>` out of range | GUC 参数被设置为无效值 | 检查[GUC 参考](guc-reference.md)了解有效范围 |
| PT706 | shared_preload_libraries: pg_ripple not loaded | pg_ripple 不在 `shared_preload_libraries` 中 | 将 `pg_ripple` 添加到 `postgresql.conf` 中的 `shared_preload_libraries` 并重启 |
| PT707 | pg_trickle not installed | 调用了需要 pg_trickle 的功能 | 安装 pg_trickle 或使用非 trickle 替代方案 |
| PT708 | pgvector not installed | 在没有 pgvector 的情况下调用了向量/嵌入函数 | 安装 pgvector 或使用 `pg_ripple.pgvector_enabled = off` 禁用 |
| PT709 | enable_graph_rls: RLS policy creation failed | 无法创建行级安全策略 | 检查超级用户权限 |
| PT710 | grant_graph: invalid permission | 权限必须是 `'read'`、`'write'` 或 `'admin'` | 使用三个有效的权限字符串之一 |
| PT711 | JSON-LD: unrecognised @embed value | JSON-LD 框架包含无法识别的 `@embed` 关键字值（有效值为 `@once`、`@always`、`@never`） | 修复框架的 `@embed` 值 |
| PT712 | JSON-LD: frame nesting depth exceeded | JSON-LD 框架的嵌套深度超过 `pg_ripple.max_path_depth` | 增加 `pg_ripple.max_path_depth` 或简化框架 |

---

## PT800–PT899：pg_tide CDC Bridge

来自 pg_tide CDC 桥接集成的错误。

| 代码 | 消息 | 原因 | 修复 |
|---|---|---|---|
| PT800 | pg_tide not installed or relay integration disabled | 调用了 CDC 桥接函数，但未安装 pg_tide 扩展或 `pg_ripple.trickle_integration = off` | 安装 pg_tide，运行 `CREATE EXTENSION pg_tide`，使用 `tide.outbox_create(...)` 创建目标 outbox，并设置 `pg_ripple.trickle_integration = on`；或使用 `pg_ripple.load_ntriples()` 进行直接加载 |

```admonish warning title="报告错误"
如果您遇到此处未列出的错误代码，或显示"联系维护者"的消息，请在 GitHub 上提出 issue，并提供完整的错误输出、您的 pg_ripple 版本（`SELECT pg_ripple.canary()`）以及最小重现示例。
```