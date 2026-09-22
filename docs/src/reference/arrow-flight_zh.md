# Arrow Flight 参考

pg_ripple 通过 `pg_ripple_http` 配套服务公开一个 [Apache Arrow Flight](https://arrow.apache.org/docs/format/Flight.html) 批量导出端点。

## 端点

```
GET /flight/do_get
```

使用 Apache Arrow Flight 协议直接从 VP 表（或 SPARQL SELECT 查询结果）向客户端流式传输 Arrow IPC 记录批次。

## 身份验证

票证使用 HMAC-SHA256 签名，具有到期时间戳和随机 nonce 以防止重放攻击。密钥通过 `pg_ripple.arrow_flight_secret` 配置。除非 `pg_ripple.arrow_unsigned_tickets_allowed = on`（默认禁用），否则未签名的票证将被拒绝。

## 票证格式

有效的 Arrow Flight 票证是 JSON 对象：

```json
{
  "query": "SELECT * FROM pg_ripple.sparql_select($1)",
  "exp": 1735689600,
  "nonce": "a1b2c3d4e5f6",
  "sig": "HMAC-SHA256 十六进制签名"
}
```

`sig` 字段是使用配置的密钥对 `query + exp + nonce` 计算的。

## 流式传输行为

端点使用 `Transfer-Encoding: chunked` HTTP 流（通过 `axum::body::Body::from_stream`），以便客户端可以在完整导出完成之前开始解码 Arrow IPC 记录批次。响应字节以 64 KiB 块发送，因为 IPC 缓冲区已生成。

**内存限制**：整个导出的 Arrow IPC 缓冲区在流式传输开始之前在内存中构建。对于非常大的结果集，`pg_ripple_http` 的 RSS 与结果集大小成比例增长（IPC 缓冲区中每行约 32 字节，PostgreSQL 客户端内存中每行约 200 字节）。单个导出调用的推荐上限是**1000 万行**（在 HTTP 配套服务有 1 GB 可用的主机上，RSS ≲ 512 MB）。对于更大的导出，按命名图或谓词进行分区，并分批调用端点。

客户端应使用流式读取（例如分块 IPC 阅读器），而不是缓冲完整的响应主体。

## 配置

| 参数 | 默认 | 描述 |
|-----------|---------|-------------|
| `pg_ripple.arrow_flight_secret` | — | 票证签名的 HMAC 密钥（必需） |
| `pg_ripple.arrow_unsigned_tickets_allowed` | `off` | 允许未签名的票证（仅限开发） |
| `ARROW_BATCH_SIZE` 环境变量 | `1000` | 每个 Arrow IPC 记录批次的行数 |

## 响应头

| 头 | 描述 |
|--------|-------------|
| `Content-Type` | `application/vnd.apache.arrow.stream` |
| `X-Arrow-Rows` | 导出的三元组总数 |
| `X-Arrow-Batches` | 发送的 Arrow IPC 记录批次数 |
| `Transfer-Encoding` | `chunked` — 响应被流式传输，而不是缓冲 |

## 状态

Arrow Flight 批量导出在 v0.71.0 中是**实验性的**。HMAC-SHA256 签名、到期和 nonce 检查已完全实现（v0.67.0 FLIGHT-SEC-02）。通过 `Body::from_stream` 的分块 HTTP 流已确认并验证（v0.71.0 FLIGHT-STREAM-01）。

---

## v1 → v2 HMAC 票证迁移（L16-05）

pg_ripple v0.72.0 引入了**v2 票证**，具有基于 nonce 的重放保护缓存（`FLIGHT-NONCE-01`）。较旧的 v1 票证（pre-v0.72.0）缺少 `nonce` 字段，因此仅在 `arrow_unsigned_tickets_allowed = on` 时被接受。

### 什么构成有效的 v1 票证

**v1 票证**是任何满足以下条件的票证 JSON：
- 包含 `query` 和 `exp` 字段。
- **不**包含 `nonce` 字段。
- 具有对 `query + exp`（签名有效负载中没有 nonce）计算的有效 HMAC-SHA256 `sig`。

v1 票证不受重放保护，因为 nonce 缓存不适用于它们。因此它们在到期前**固有地可重放**。

### 迁移路径

1. **生成新的 v2 票证** — 在每个新票证中包括一个随机 `nonce`（十六进制字符串，≥ 12 字节的熵），并对 `query + exp + nonce` 进行签名。
2. **过渡窗口** — v2 票证立即被接受。当服务器运行 `arrow_unsigned_tickets_allowed = on` 时，v1 票证在其 `exp` 窗口的剩余时间内继续被接受。一旦所有活动客户端都迁移到 v2 票证，请将其设置为 `off`。
3. **识别过期的 v1 票证** — 当 `exp < now()`（Unix 时间戳）时，v1 票证已过期。使用以下方法检查原始票证 JSON：
   ```bash
   echo '<ticket_json>' | python3 -c "
   import json, sys, time
   t = json.load(sys.stdin)
   exp = t.get('exp', 0)
   print('expired' if exp < time.time() else f'valid until {time.ctime(exp)}')
   print('v1 ticket (no nonce)' if 'nonce' not in t else 'v2 ticket (has nonce)')
   "
   ```
4. **禁用 v1 接受** — 一旦所有正在进行的票证都迁移了，请在 `pg_ripple_http` 环境中设置 `ARROW_UNSIGNED_TICKETS_ALLOWED=false` 并重新启动。

### 摘要

| 特性 | v1 票证 | v2 票证 |
|---------|-----------|-----------|
| 重放保护 | 无（到期前可重放） | Nonce 缓存（一次性使用） |
| HMAC 输入 | `query + exp` | `query + exp + nonce` |
| `nonce` 字段 | 不存在 | 存在（≥ 12 字节十六进制） |
| 服务器要求 | `arrow_unsigned_tickets_allowed = on` | 始终接受（默认） |

另请参阅：[HTTP API](http-api.md)、[架构](architecture.md)、[兼容性矩阵](../operations/compatibility.md)。