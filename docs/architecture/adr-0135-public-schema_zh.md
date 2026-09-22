# ADR：在 v0.135.0 中保留 `pg_ripple` 公共 schema

状态：已接受

v0.135.0 稳定 API 保留现有的 `pg_ripple` schema。将其重命名为 `ripple` 将一次性破坏所有限定函数调用、生成的扩展对象、HTTP 查询以及升级路径。因此扩展保留 schema 名称，并为需要它的安装记录所需的 PostgreSQL `allow_system_table_mods` 设置。

内部 `_pg_ripple` schema 保持私有。新的稳定函数和目录遵循现有的拆分。客户端可以采用 v1 manifest 而无需重写命名空间，未来的主要版本可以重新审视公共名称，并提供明确的迁移和兼容性时间窗口。