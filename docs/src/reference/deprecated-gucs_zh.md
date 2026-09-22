# 已弃用的 GUC

> **A13-04 (v0.86.0)**：此页面列出 pg_ripple 中所有已弃用的 GUC 参数、它们的替换名称以及计划的删除版本。

设置已弃用的 GUC 会引发 `PT501`，并带有指向替换项的描述性消息。

---

## 当前已弃用

| 已弃用的 GUC | 引入版本 | 替换 | 删除 |
|---|---|---|---|
| `pg_ripple.property_path_max_depth` | v0.24.0 | `pg_ripple.max_path_depth` | v0.56.0（已删除） |

---

## 已删除的 GUC

这些 GUC 已被删除。尝试设置它们会引发 `PT501` 错误。

| 已删除的 GUC | 删除于 | 替换 |
|---|---|---|
| `pg_ripple.property_path_max_depth` | v0.56.0 | `pg_ripple.max_path_depth` |

---

## 计划的弃用

| GUC | 计划弃用 | 原因 | 替换 |
|---|---|---|---|
| `pg_ripple.describe_strategy` | v0.86.0（软） | 被具有 W3C 对齐值的 `describe_form` 取代 | `pg_ripple.describe_form` |

---

## 迁移说明

### `pg_ripple.property_path_max_depth` → `pg_ripple.max_path_depth`

在 v0.56.0 中重命名。更新 `postgresql.conf` 和 `ALTER SYSTEM SET` 调用：

```sql
-- 之前（引发 PT501）：
SET pg_ripple.property_path_max_depth = 200;

-- 之后：
SET pg_ripple.max_path_depth = 200;
```

### `pg_ripple.describe_strategy` → `pg_ripple.describe_form`

`describe_strategy` GUC 使用值 `cbd`、`scbd`、`simple`。新的 `describe_form` GUC 使用 W3C 对齐的名称 `cbd`、`scbd`、`symmetric`（`symmetric` 是 `scbd` 的别名）。当设置 `describe_form` 时，它优先于 `describe_strategy`。

```sql
-- 之前：
SET pg_ripple.describe_strategy = 'scbd';

-- 之后（首选）：
SET pg_ripple.describe_form = 'symmetric';  -- 与 'scbd' 相同
```

两个 GUC 在 v0.86.0 中仍然受支持。`describe_strategy` 可能会在 v0.87.0 中正式弃用。