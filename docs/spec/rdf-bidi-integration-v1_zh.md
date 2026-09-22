# RDF 双向集成配置文件，v1（草案）

> **状态**：草案，供应商中立。编辑者：pg_ripple 项目。参考实现：[pg_ripple](https://github.com/trickle-labs/pg-ripple) v0.77.0+。
>
> 本文档指定任何 RDF 三元组存储都可以实现的线级和行为配置文件，以与外部系统（CRM、ERP、SaaS API、其他 RDF 存储）进行**双向集成**，而无需每个中继的协调粘合代码。该配置文件以 W3C 标准为基础（RDF 1.1、RDF-star、OWL 2 RL、SHACL、SPARQL 1.1、JSON-LD 1.1、PROV-O），并且不添加任何无法从它们派生的内容；其贡献在于*组合*和*协议*。
>
> "双向"意味着同一个存储既可以是源（发出 CDC 风格的变更事件），又可以是接收器（摄取变更事件），冲突、回声、目标分配的标识符和跨源等价性由协议处理，而不是由特别的中继代码处理。

## 1. 动机

当今每个 RDF 三元组存储都公开某种形式的变更数据捕获（Jena listeners、Stardog 集群事件、Oxigraph diffs、GraphDB 通知、pg_ripple 订阅）。每个都是定制的。一旦三元组存储位于集成拓扑中，它们都无法解决出现的更棘手的问题：

1. **来源归属**：哪个外部系统贡献了哪个三元组，以及来自*该系统时钟*的什么时间戳？
2. **回声抑制**：当源自系统 A 的写入被镜像到系统 B 然后又被镜像回来时，我们如何在不丢失 B 派的合法并发编辑的情况下抑制回声？
3. **冲突解决**：当两个源对 `sh:maxCount 1` 谓词产生分歧时，谁获胜，失败者去哪里？
4. **跨源身份**：当系统 A 和系统 B 在不同的 IRI 下引用同一现实世界实体时，我们如何在没有必须追溯更新的全局 IRI 注册表的情况下正确路由写入？
5. **目标分配的标识符**：当集成目标（例如 ERP）在接受创建后分配规范 ID 时，我们如何将生成的 IRI 绑定回发起事件？
6. **乐观并发**：中继如何安全地应用事件而不覆盖它不知道的并发编辑？
7. **Schema 演化**：对投影形状的途中更改如何与排队的事件交互？
8. **运维不变性**：队列限制、死信、审计、RBAC、编辑、协调。

现有的 CDC 机制最多隐式地回答 (1)，而对 (2)–(8) 则根本没有回答。Bidi 配置文件以每个三元组存储都可以实现的方式回答所有八个问题。

## 2. 候选一致性级别

此草案勾勒出供审查的候选一致性级别。它们是**非规范性**的，直到配置文件从 pg_ripple 路线图中提取并稳定下来。

如果三元组存储实现 §3（数据模型）、§4（线格式）、§5（接收方协议）、§6（协调）以及 §11 候选一致性测试语料库，则它声明符合 **Bidi-1.0**。

如果三元组存储此外还实现 §7（Schema 演化）、§8（认证和编辑）、§9（审计）和 §10（运维界面），则它声明符合 **Bidi-1.0-Ops**。

三元组存储可以仅实现 Bidi-1.0；它在没有 Bidi-1.0 的情况下不得声明 Bidi-1.0-Ops。

## 3. 数据模型

### 3.1 源是命名图

从外部系统摄取的每个三元组必须存储在 IRI 唯一标识源的命名图中。推荐约定是 `urn:source:<system>`（例如 `urn:source:crm`、`urn:source:erp`）；实现不得采用不同的约定。

默认图（RDF 默认图；在某些实现中为"graph 0"）保留用于非源归属的三元组。符合 Bidi 配置文件的摄取不得写入默认图。

### 3.2 每源时间戳

源时间戳作为 RDF-star 注释在三元组上承载，使用谓词 `prov:generatedAtTime`（PROV-O）。对于三元组 `<<:s :p :o>>`，时间戳被断言为：

```turtle
<< :s :p :o >> prov:generatedAtTime "2026-04-22T14:00:00Z"^^xsd:dateTime .
```

没有 RDF-star 的实现可以使用具体化，但必须记录映射。

### 3.3 已解析的投影

实现必须维护一个派生投影，回答"在所有源中 `(subject, predicate)` 的当前共识值是多少？"此投影是非破坏性的：源图保留其原始三元组；只有投影反映解析。

定义了四种解析策略。实现必须支持所有四种。

| 策略 | 定义 |
|---|---|
| `source_priority [s1, s2, ...]` | 来自具有该值的最高优先级源的值获胜。较低优先级的源仅在较高优先级的源静默时才贡献。 |
| `latest_wins` | 具有最新 `prov:generatedAtTime` 的值获胜。关系由源优先级打破。 |
| `reject_on_conflict` | 如果多个源具有不同的值，则 NO 值进入投影；冲突被记录以供操作员审查。 |
| `union` | 所有值进入投影（仅对没有 `sh:maxCount 1` 的谓词有效）。 |

策略可以按谓词或全局配置。解析的单位是 `(subject, predicate)`，而不是 `(subject, predicate, object)`。

### 3.4 回声感知归一化

冲突策略可以使用 `normalize` SPARQL 表达式进行限定。当且仅当两个值的归一化形式相等时，它们被视为"出于冲突目的相同"。示例：

```
latest_wins normalize=`STRDT(STR(?v), xsd:string)`     # 忽略数据类型不匹配
latest_wins normalize=`UCASE(STR(?v))`                  # 不区分大小写
latest_wins normalize=`REPLACE(STR(?v), "[\\s-]", "")`  # 忽略空格和连字符
```

当 source-A 和 source-B 的值归一化为相同的形式时，不会引发冲突，也不会发出事件（回声抑制）。保留精度：底层三元组保留其原始词法形式；只有冲突检测比较归一化形式。

### 3.5 跨源等价

跨源身份使用 `owl:sameAs`。协议不要求完整的 OWL 2 推理；它只要求在源图的并集上 `owl:sameAs` 的对称-传递闭包。

实现必须提供等效于"给我 IRI `i` 的等价类"的查询原语，具有有界延迟。

## 4. 线格式

### 4.1 事件形状

出站事件是具有以下顶级 schema 的 JSON 文档：

```json
{
  "version":           "1.0",
  "event_id":          "<uuid>",
  "event_type":        "INSERT" | "UPDATE" | "DELETE",
  "subject":           "<canonical-iri>",
  "subject_resolved":  true | false,
  "graph":             "<source-graph-iri>",
  "timestamp":         "<RFC 3339 instant>",
  "@context":          { ... JSON-LD context ... },
  "after":             { ... JSON-LD framed object ... },
  "base":              { ... JSON-LD framed object ... }
}
```

字段语义：

- **`version`**：唯一的顶级鉴别符。接收方必须切换到此。未来版本必须保持默认的向前兼容。接收方忽略未知字段。
- **`event_id`**：全局唯一。用于可追溯性和带外回调，例如 linkback、差异报告和放弃。
- **`event_type`**：`INSERT`、`UPDATE`、`DELETE` 之一。SHACL 语义（原子删除-然后-插入）折叠为 `UPDATE`。
- **`subject`**：已更改实体的规范 hub IRI。
- **`subject_resolved`**：布尔值。`true` 表示 `subject` 已经是接收方在接收方的 `iri_match_pattern` 下的延迟绑定重写后应使用的 IRI；`false` 表示事件是未解析的 INSERT，其目标分配的 ID 必须在后续事件发出之前通过 linkback 报告。
- **`graph`**：源图 IRI。接收方将其用于回声检测（不要重新摄取 `graph` 与接收方自己的源图匹配的事件）。
- **`timestamp`**：事件发出时间，RFC 3339。
- **`@context`**：用于 `after` 和 `base` 的 JSON-LD context。应该是稳定的 URL 或小型内联 context。
- **`after`**：实体的投影后状态，作为 JSON-LD 框架对象。对于 `DELETE`，必须为 `null`。
- **`base`**（sparse-CAS）：对于 `UPDATE`，是*已更改的谓词的稀疏前状态*。对于 `INSERT`，必须为 `{}`。对于 `DELETE`，必须是删除前的完整帧。有关 CAS 语义，请参阅 §5.2。

`version` 的 schema URI 通过 HTTP `Link: rel="describedby"` 头（§4.3）公布。不得在每个事件体内重复。

### 4.2 框架对称规则

`after` 和 `base` 必须使用*相同*的 JSON-LD context 和 frame 进行框架化。具体来说：

- 对于同时存在的每个键，当底层 RDF 值相等时，值的词法表示（字符串、数字、列表顺序、数据类型标记的存在）必须逐字节相同。
- 实现不得对一侧应用有损归一化而对另一侧不应用。

这保证了接收方可以通过字节字符串相等性（`actual[k] == base[k]`）执行 CAS，而无需重新解析。

### 4.3 HTTP 传输

通过 HTTP 公开事件的实现必须：

- 在事件响应上设置 `Content-Type: application/json`。
- 设置 `Link: <https://example.org/specs/bidi/v1/event.schema.json>; rel="describedby"`（使用实现的实际 schema URL）。
- 通过 `Authorization: Bearer <token>` 使用 bearer 令牌认证。
- 支持每路由范围检查（§8.1）。

实现还可以提供非 HTTP 传输（Postgres SPI、gRPC、AMQP、Kafka）。无论传输方式如何，线格式都相同。

### 4.4 对象级分组

必须为每个 `(subject, transaction)` 对发出一个事件。涉及 N 个主体的交易发出 N 个事件；涉及一个主体的 K 个谓词的交易发出一个事件，其 K 个填充键位于 `after` 中，对于 UPDATE 位于 `base` 中。

实现可以提供三元组级分组模式作为旧版逃生口，但它必须是选择加入的。

## 5. 接收方协议

### 5.1 传递底层

接收方从实现定义的至少一次传递底层消费事件。此草案有意不规定 `pull`、`ack` 或 `nack` 函数：符合的实现可以使用事务性发件箱表、Kafka、NATS、webhook、逻辑复制或直接轮询。

底层必须提供至少一次传递、持久重试以及操作员可见的传递死信机制或等效的失败界面。实现应通过 §10 公开传递状态（深度、上次传递、上次错误、重试/DLQ 计数）。

对于 pg_ripple，此底层是 pg-trickle 的发件箱/收件箱和中继；pg_ripple 本身不公开 `next_event`、`ack_event` 或 `nack_event`。

### 5.2 Sparse-CAS 应用

对于 `UPDATE` 事件，接收方应用以下算法：

```
for each (predicate, base_value) in event.base:
    actual_value = read_actual(target_system, event.subject, predicate)
    if actual_value == base_value:
        continue  # CAS 对此谓词成立
    elif actual_value == event.after[predicate]:
        continue  # 已应用；幂等
    else:
        escalate(event, predicate, actual_value, base_value, event.after[predicate])
        return

apply_writes(target_system, event.subject, event.after)
mark_delivery_complete(event.event_id)  # 底层特定
```

`escalate` 必须调用协调工具包（§6），而不是静默地覆盖。

对于在目标系统上分配其自己主键的 INSERT 事件，请参阅 §5.3。

对于 `DELETE` 事件，接收方应在删除之前验证至少一个标识谓词的 `actual == base`；如果不匹配，则升级。

### 5.3 目标分配的标识符的 Linkback

当目标系统为新的 INSERT 分配规范 ID（例如 ERP 在接受创建后返回 `4011`）时，接收方：

1. 在目标系统中执行插入，捕获分配的 ID。
2. 在源存储上调用 `record_linkback(event_id, target_id)`。

源存储：

1. 通过接收图的 `iri_template` 扩展 `target_id` 以产生目标 IRI。
2. 原子地在原始 `subject` 和扩展的目标 IRI 之间写入 `owl:sameAs`。
3. 刷新该主体的任何订阅缓冲存储的后续事件（§5.4）。

实现还必须接受 `target_iri` 形式，用于目标系统返回规范 URL 而非裸 ID 的情况。

源存储必须使初始未解析的 INSERT 对接收方可见，以便接收方可以创建目标实体并调用 `record_linkback`。在 linkback 落地之前，同一挂起主体的后续事件由 §5.4 保留。未解析 INSERT 的传递完成是底层特定的；实现应记录如果 linkback 从未到达则重新传递的行为。

### 5.4 挂起 Linkback 期间的订阅缓冲

在 `(subject, subscription)` 的 linkback 挂起期间，同一 `(subject, subscription)` 的后续事件必须以未渲染形式持久化，并在 `record_linkback` 落地时原子刷新。实现不得为其目标 ID 尚未知的实体发出后续事件。

如果 linkback 从未落地（操作员放弃、目标系统失败），缓冲的事件必须在 `linkback_timeout`（默认 1 小时）后过期，并通知操作员。

### 5.5 延迟绑定 IRI 重写

当为主题 `s` 向 `target_graph` 为 `g` 的订阅发出事件时：

1. 计算等价类 `E = closure_owl_sameAs(s)`。
2. 如果 `E` 的任何成员匹配 `g` 的 `iri_match_pattern`，则将该成员作为 `subject` 发出，并将 `subject_resolved = true`。
3. 否则，将规范的 hub IRI 作为 subject 发出，并且仅对于需要 linkback 的未解析 INSERT，将 `subject_resolved = false`；对于其他事件类型，根据实现的缺失重写策略发出。

此重写是**延迟绑定**：它在发出时发生，而不是在写入时发生。闭包更改（新发现的 `owl:sameAs`）不得追溯地重写已排队的事件。

### 5.6 循环预防

订阅必须支持 `exclude_graphs` 参数。`graph` 字段出现在排除列表中的事件必须被过滤掉。

订阅必须支持 `propagation_depth` 参数（默认 `1`）。对图 G 中的三元组的更改会触发 G 的事件以及通过最多 `propagation_depth` `owl:sameAs` 跳数可发现的等价三元组的事件，但不会再进一步。这限制了密集链接的等价类中的扇出。

## 6. 协调

当 sparse-CAS 失败（§5.2 `escalate`）时，接收方必须将协调条目排队：

```
reconciliation_enqueue(
    event_id,
    divergence_summary = {
        predicate_iri: { actual: ..., base: ..., after: ... },
        ...
    }
)
```

实现应提供面向操作员的原语 `reconciliation_next(subscription)`，它使用 `SKIP LOCKED` 或等效的工作队列机制租用下一个未解析的条目。这是协调队列，而不是主事件传递底层。

实现必须支持四种解析操作：

| 操作 | 语义 |
|---|---|
| `accept_external` | 将外部系统的实际值摄取到相应的源图中，就好像它是通过正常摄取到达的。原始事件被确认为无操作。 |
| `force_internal` | 重新发出标记为强制覆盖的相同事件。接收方在忽略 `base` 的情况下应用它。 |
| `merge_via_owl_sameAs` | 在差异值的主体之间断言 `owl:sameAs`（在差异揭示重复项时使用）。确认原始事件。 |
| `dead_letter` | 将事件移动到死信存储，并附加差异摘要。 |

实现可以提供其他操作，但不得重新定义这四种操作。

## 7. Schema 演化

订阅会随着时间的推移而演化：frame 更改、IRI 模板/匹配模式更改、排除列表更改。协议为每个定义了明确的策略。

### 7.1 Frame 更改

当订阅的 frame 被更改时，排队的事件受 `frame_change_policy` 影响：

- `new_events_only`（推荐的默认）：已经渲染的排队事件保持不变地排出；新事件使用新 frame。
- `reframe_queued` 和 `drain_then_switch` 是高级实现定义的策略。它们要求排队的事件以未渲染形式存储，并且此草案不要求。

### 7.2 IRI 模板/匹配模式更改

当图的 `iri_template` 或 `iri_match_pattern` 被更改时，排队的事件受 `iri_change_policy` 影响：

- `new_events_only`（推荐的默认）：排队的事件保留原始渲染的 IRI；新事件使用新模板/匹配模式。
- 追溯重写是实现定义的，并要求排队的事件未渲染。

### 7.3 排除图更改

当 `exclude_graphs` 被更改时，排队的事件受 `exclude_change_policy` 影响：

- `new_events_only`（推荐的默认）：新排除列表仅适用于更改后发出的事件。排队的事件保持不变地排出。
- 排队事件的追溯过滤是实现定义的。

所有 schema 更改必须记录在审计表中，包含旧值、新值、应用的策略和受影响的事件计数。

## 8. 认证和编辑

### 8.1 每订阅范围令牌

实现必须支持限定于单个订阅的 bearer 令牌，具有以下权限的子集：

| 范围 | 允许 |
|---|---|
| `linkback` | `record_linkback` |
| `divergence` | 报告 CAS 差异 |
| `abandon` | 放弃挂起的 linkback |
| `outbox_read` | 读取订阅的发件箱表或等效传递流 |
| `dead_letter_admin` | 重新排队/丢弃语义死信行 |

令牌必须使用 SHA-256 进行静态哈希。原始令牌只能在注册时返回。

为订阅 A 注册的令牌不得允许对订阅 B 的操作，即使范围匹配。

### 8.2 Frame 级编辑

Frame 可以使用 `"@redact": true` 标记谓词：

```json
{
  "@context": { "ex": "https://example.com/ns#" },
  "@type":    "ex:Contact",
  "ex:name":  {},
  "ex:phone": { "@redact": true },
  "ex:taxId": { "@redact": true }
}
```

对于编辑后的消费者，已编辑的谓词必须在 `after` 和 `base` 中作为字面量对象 `{"@redacted": true}` 发出。仅看到已编辑传递界面的接收方无法独立验证已编辑的谓词的 CAS。需要严格接收方侧 CAS 的谓词必须通过未编辑的界面传递或从 `base` 中省略。

对于未编辑的消费者，明文值可以通过单独的订阅、发件箱、流或授权边界发出。实现应优先选择写入时编辑，并为已编辑/未编辑的传递界面提供单独的传递界面，而不是按令牌拉取时间渲染。

## 9. 审计

实现必须记录每个带外可变调用（`record_linkback`、差异报告、放弃、语义死信重新排队/丢弃、协调解决），至少包含：

| 字段 | 必需 |
|---|---|
| `event_id`（或 `reconciliation_id`） | 是 |
| `subscription_name` | 是 |
| `action` | 是 |
| `actor_token_hash` | 通过认证传输调用时是 |
| `actor_session` | 通过直接查询界面调用时是 |
| `remote_addr` | 通过 HTTP 调用时是 |
| `observed_at` | 是 |

传递底层审计（拉取/确认/否定、接收器传递、重试）是实现定义的，可以存在于 Bidi 存储之外。

实现必须提供可配置保留窗口，默认值为 90 天。

## 10. 运维界面

实现必须公开每订阅状态视图，至少包含：

- 队列深度
- 最旧事件年龄
- 死信计数
- 冲突拒绝率（滚动窗口）
- 挂起 linkback 计数和最旧挂起年龄
- 传递底层状态，例如最后传递时间戳、最后错误、重试计数和传递 DLQ 计数
- 协调打开计数

实现必须公开一个总体健康状态，值为 `healthy` / `degraded` / `paused` / `failing` 以及触发条件列表。映射规则由实现自行决定，但必须记录。

公开 HTTP 的实现必须在状态为 `failing` 时从健康端点返回 `503`。

## 11. 一致性测试语料库

参考测试语料库随本规范在 `tests/bidi-conformance/` 中提供（TBD；pg_ripple 在 `tests/fixtures/bidi/` 下提供临时版本）。该语料库由黑盒场景组成：

| 场景 | 断言 |
|---|---|
| `single_source_insert` | 在 `urn:source:s1` 下摄取一个三元组；验证它出现在已解析的投影中并发出一个 INSERT 事件。 |
| `two_source_latest_wins` | 从 `s1` 和 `s2` 摄取冲突的 `sh:maxCount 1` 值；验证最新时间戳获胜；验证一个带有稀疏 `base` 的 UPDATE 事件。 |
| `echo_suppression` | 从 `s1` 摄取，镜像到 `s2`，镜像回来；验证在 `normalize` 规则下没有第二个事件。 |
| `late_binding_rewrite` | 在断言 `owl:sameAs(s2:foo, s1:bar)` 之前摄取对 `s2:foo` 的 `s1` 引用；验证 `s1:bar` 的下一个事件携带 `subject = s2:foo` 且 `subject_resolved = true`；验证已经渲染的排队事件不会追溯重写。 |
| `linkback_round_trip` | 向具有目标分配 ID 的订阅发出 INSERT；用裸 ID 记录 linkback；验证写入 `owl:sameAs`；验证同一主体的后续 UPDATE 携带重写的 IRI。 |
| `subscription_buffer_flush` | 与 linkback_round_trip 相同，但在 linkback 落地之前发出第二个事件；验证第二个事件被缓冲并在 linkback 落地时原子刷新。 |
| `cas_divergence_escalates` | 接收方应用 UPDATE；同时目标系统独立更改了 `actual`；验证 CAS 失败并重组对排队。 |
| `four_resolutions` | 对于四种解析操作中的每一种，将协调驱动到该解析并断言结果状态。 |
| `frame_change_new_events_only` | 在途中更改 frame；验证已经渲染的排队事件在旧 frame 下排出，新发出的事件使用新 frame。 |
| `frame_change_advanced_policy` | 可选：支持排队重新框架或 drain-then-switch 的实现证明排队的事件未渲染存储并且不会竞争传递。 |
| `redacted_predicate` | 使用 `@redact` 框架化谓词；验证已编辑的传递界面发出 `{"@redacted": true}`，并且在配置时未编辑的传递界面发出明文。 |
| `scope_isolation` | 针对订阅 B 使用订阅 A 的令牌；验证拒绝。 |
| `dead_letter_overflow` | 在三种溢出策略中的每一种下将事件推到超过 `max_queue_depth`；验证预期的死信内容。 |
| `audit_completeness` | 在一轮 linkback/差异/解析/死信管理操作之后，验证每个带外可变调用恰好有一个审计记录。 |
| `convergence_random` | 从 4 个源应用 1000 个随机插入/更新/删除操作；验证在 `latest_wins` 下，已解析的投影与操作到达顺序无关。 |

候选映射：Bidi-1.0 将涵盖场景 1–8；Bidi-1.0-Ops 将另外涵盖 9–15。此草案尚未将这些关卡设为规范性。

## 12. 非规范性参考实现

[pg_ripple](https://github.com/trickle-labs/pg-ripple) v0.77.0+ 是参考实现。本规范的词汇到 pg_ripple 特定界面的映射记录在 [docs/src/operations/pg-trickle-relay.md](../src/operations/pg-trickle-relay.md)。超出本规范范围的 pg_ripple 特定问题：

- 存储布局（HTAP delta/main 分区）。
- 队列底层（使用 `SKIP LOCKED` 租约的 Postgres 表）。
- HTTP 配套服务。
- 迁移机制。
- Prometheus 指标命名。

实现此配置文件的其他三元组存储可以自由采用不同的底层，只要从线可观察到的协议与 §4–§10 匹配。

## 13. 版本控制

本规范通过事件的顶级 `version` 字段进行版本控制。不向后兼容的更改需要主要版本号提升；接收方可以拒绝未知的主要版本。向后兼容的添加需要次要版本号提升，并且必须可被不理解它们的接收方忽略。

Schema URL 遵循模式 `https://<editor>/specs/bidi/v<MAJOR>/event.schema.json`，必须通过 HTTP `Link: rel="describedby"`（§4.3）公布。

## 14. 安全注意事项

- 令牌必须静态哈希。存储原始令牌的实现是不符合规范的。
- 每订阅范围检查是必需的；允许全局"admin"令牌用于操作员端点，但不得接受用于每订阅传输调用。
- Frame 级编辑是深度防御措施，不能替代传输加密。HTTP 传输需要 HTTPS。
- 审计日志应该是仅追加或由仅追加存储支持；允许删除的实现必须记录删除。
- `force_internal` 协调操作应需要提升的授权。

## 15. 推荐限定符词汇（非规范性）

鼓励使用 RDF-star 注释（§3.2）承载每语句上下文的实现遵循此词汇。这些谓词是推荐的，不是强制的；实现可以使用替代方案，但必须记录其选择。

### 时间

| 谓词 | 用途 | 示例 |
|---|---|---|
| `prov:generatedAtTime` | 源系统的时间戳 | `<< :s :p :o >> prov:generatedAtTime "2026-04-22T14:00:00Z"^^xsd:dateTime .` |
| `dcterms:valid` | 有效性间隔（开始、结束或时间点） | `<< :s :contract :active "true"^^xsd:boolean >> dcterms:valid "[2026-01-01T00:00:00Z, 2026-12-31T23:59:59Z]"^^xsd:string .` |
| `prov:startedAtTime` | 关系/状态何时开始 | `<< :contact :employedAt :company >> prov:startedAtTime "2020-01-15T00:00:00Z"^^xsd:dateTime .` |
| `prov:endedAtTime` | 关系/状态何时结束 | `<< :contact :employedAt :company >> prov:endedAtTime "2024-03-31T00:00:00Z"^^xsd:dateTime .` |

### 源出处

| 谓词 | 用途 | 示例 |
|---|---|---|
| `prov:wasGeneratedBy` | 链接到产生此值的活动/代理 | `<< :s :p :o >> prov:wasGeneratedBy [ a prov:Activity ; prov:agent <urn:agent:salesforce-sync> ] .` |
| `dcterms:source` | 源系统的参考 URL 或标识符 | `<< :s :phone :value >> dcterms:source "https://salesforce.com/records/0011..." .` |

### 质量/置信度

| 谓词 | 用途 | 示例 |
|---|---|---|
| `prov:confidence` | 置信度分数（0–1）；对 ML 派生增强有用 | `<< :contact :likelyRole "VP Marketing" >> prov:confidence "0.87"^^xsd:decimal .` |
| `dcterms:issued` | 断言的日期（与生效日期相对） | `<< :product :costPerUnit "123.45" >> dcterms:issued "2026-04-20"^^xsd:date .` |

### 集成特定

| 谓词 | 用途 | 示例 |
|---|---|---|
| `bidi:sourceSystemId`（实现定义） | 外部系统对此断言的内部 ID；支持往返 linkback | `<< :s :p :o >> bidi:sourceSystemId "SF-0011-PHONE-2026" .` |
| `bidi:integrationTimestamp`（实现定义） | 断言被摄取到集成层的时间 | `<< :s :p :o >> bidi:integrationTimestamp "2026-04-22T14:15:33Z"^^xsd:dateTime .` |

**使用说明**：避免为每个集成商创建新谓词。在适用的情况下使用这些标准谓词；对于实现特定的元数据，采用一致的 URI 命名空间（例如 `https://yourdomain.com/integration/v1#`）并在映射定义中记录它。

## 16. 参考

- [RDF 1.1 Concepts and Abstract Syntax](https://www.w3.org/TR/rdf11-concepts/)（W3C Recommendation）
- [RDF-star and SPARQL-star](https://w3c.github.io/rdf-star/)（W3C Working Draft）
- [SPARQL 1.1 Query Language](https://www.w3.org/TR/sparql11-query/)（W3C Recommendation）
- [SHACL](https://www.w3.org/TR/shacl/)（W3C Recommendation）
- [OWL 2 RL Profile](https://www.w3.org/TR/owl2-profiles/#OWL_2_RL)（W3C Recommendation）
- [JSON-LD 1.1](https://www.w3.org/TR/json-ld11/)（W3C Recommendation）
- [JSON-LD 1.1 Framing](https://www.w3.org/TR/json-ld11-framing/)（W3C Recommendation）
- [PROV-O](https://www.w3.org/TR/prov-o/)（W3C Recommendation）
- [CloudEvents 1.0](https://github.com/cloudevents/spec)（CNCF）
- [pg_ripple v0.77.0 + v0.78.0 roadmap](https://github.com/trickle-labs/pg-ripple/blob/main/roadmap/v0.77.0-full.md)（参考实现）