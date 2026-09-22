# 开发参考

此页面为从事 pg_ripple 工作的贡献者和扩展开发人员提供参考。

## 概述

pg_ripple 使用 `pgrx` 0.18 框架为 PostgreSQL 18 用 Rust 编写。该扩展独家使用安全 Rust（在必需的 FFI 边界除外），通过 `#[pg_extern]` 公开 SQL 函数，并对扩展代码内执行的所有 SQL 使用 `pgrx::SpiClient`。

## 构建和测试

```bash
# 针对 PG18 安装 pgrx
cargo pgrx init --pg18 $(which pg18)

# 运行所有单元和集成测试
cargo pgrx test pg18

# 运行 pg_regress 套件
cargo pgrx regress pg18 --postgresql-conf "allow_system_table_mods=on"

# 运行迁移链测试
bash tests/test_migration_chain.sh

# 格式化代码
cargo fmt --all

# Lint（要求零警告）
cargo clippy --features pg18 -- -D warnings

# 本地安装
cargo pgrx install --pg-config $(which pg_config)
```

## 项目布局

| 路径 | 描述 |
|---|---|
| `src/lib.rs` | 扩展入口点，`_PG_init`，GUC 参数 |
| `src/dictionary/` | IRI/blank-node/literal → i64 编码器（XXH3-128 + LRU） |
| `src/storage/` | VP 表，HTAP delta/main 分区，合并后台工作线程 |
| `src/sparql/` | SPARQL 文本 → spargebra 代数 → SQL → SPI 执行 → 解码 |
| `src/construct_rules/` | CONSTRUCT 回写规则 |
| `src/datalog/` | Datalog 规则解析器、分层器、SQL 编译器 |
| `src/shacl/` | SHACL 形状 → DDL 约束 + 异步验证 |
| `src/export/` | Turtle / N-Triples / JSON-LD 序列化 |
| `src/stats/` | 监控，pg_stat_statements 集成 |
| `src/admin/` | vacuum、reindex、prefix 注册表 |
| `pg_ripple_http/` | Axum HTTP 配套服务 |
| `sql/` | 扩展 SQL 和迁移脚本 |
| `tests/pg_regress/` | pg_regress 测试套件 |

## 代码约定

- **仅安全 Rust**：`unsafe` 仅在必需的 FFI 边界允许，并带有 `// SAFETY:` 注释。
- **SQL 函数**：通过 `#[pg_extern]` 公开，绝不使用原始 `PG_FUNCTION_INFO_V1`。
- **数据库访问**：对扩展代码内的所有 SQL 使用 `pgrx::SpiClient`。
- **错误消息**：遵循 PostgreSQL 风格（第一个单词小写，无尾随句点）。
- **整数连接**：所有 SPARQL 到 SQL 的翻译必须在 SQL 生成之前将绑定术语编码为 `i64`；VP 查询中的字符串比较是一个错误。

## 迁移脚本

每个版本都需要一个 `sql/pg_ripple--X.Y.Z--X.Y.(Z+1).sql` 迁移脚本。如果没有架构更改，请添加注释头说明提供了哪些新函数/GUC。

```bash
# 示例：为 v0.74.0 创建迁移脚本
cat > sql/pg_ripple--0.73.0--0.74.0.sql << 'EOF'
-- 迁移 0.73.0 → 0.74.0
-- 架构更改：<在此描述>
EOF
```

有关完整发布清单，请参阅[发布过程](release-process.md)。

## 测试约定

- 单元测试：实现文件中的 `#[pg_test]`
- 集成测试：`tests/pg_regress/sql/*.sql` 文件
- 每个版本里程碑都应该有一个 `v0XX_features.sql` 回归文件
- 预期输出：`tests/pg_regress/expected/*.out`

## 相关页面

- [CONTRIBUTING.md](../../../CONTRIBUTING.md)
- [发布过程](release-process.md)
- [架构内部](architecture.md)
- [API 稳定性保证](api-stability.md)