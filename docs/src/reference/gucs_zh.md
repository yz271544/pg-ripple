# GUC

请参阅[完整 GUC 参考](guc-reference.md)。

`pg_ripple.llm_api_key_env` 仅接受匹配 `^[A-Z_][A-Z0-9_]*$` 的名称。原始值被拒绝。从 0.131 之前迁移时，超级用户可以临时设置 `pg_ripple.llm_api_key_env_allow_raw = on`。