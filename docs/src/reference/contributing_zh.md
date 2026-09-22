# 贡献

感谢您有兴趣为 pg_ripple 做出贡献。本指南涵盖环境设置、测试、代码约定和拉取请求工作流。

```admonish note title="贡献"
pg_ripple 是开源的，欢迎各种形式的贡献 — 错误报告、文档修复、测试用例和功能实现。如果您不确定某个想法是否合适，请打开 GitHub 问题进行讨论，然后再编写代码。
```

---

## 开发环境

### 先决条件

| 工具 | 版本 | 目的 |
|---|---|---|
| Rust | Edition 2024，stable 工具链 | 语言 |
| PostgreSQL | 18.x | 目标数据库 |
| pgrx | 0.18 | PostgreSQL 扩展框架 |
| cargo-pgrx | 0.18 | 构建和测试工具 |
| git | 2.x+ | 版本控制 |

### 设置

```bash
# 1. 克隆存储库
git clone https://github.com/your-org/pg_ripple.git
cd pg_ripple

# 2. 如果尚未安装 cargo-pgrx，请安装
cargo install cargo-pgrx --version 0.18 --locked

# 3. 使用 PostgreSQL 18 初始化 pgrx
cargo pgrx init --pg18 $(which pg_config)

# 4. 验证构建
cargo build
```

```admonish tip title="macOS"
在 macOS 上，通过 Homebrew 安装 PostgreSQL 18：`brew install postgresql@18`。确保 `pg_config` 在您的 `PATH` 上。
```

---

## 运行测试

pg_ripple 使用三个级别的测试：

### 单元和集成测试（pgrx）

在临时 PostgreSQL 实例中运行 Rust 测试：

```bash
cargo pgrx test pg18
```

这会启动一个临时 PG18 集群，安装扩展，运行所有 `#[pg_test]` 函数，并拆除集群。

### 回归测试（pg_regress）

运行基于 SQL 的回归测试，比较预期输出：

```bash
cargo pgrx regress pg18
```

测试 SQL 文件位于 `sql/` 中，预期输出位于 `expected/` 中。如果您添加新的 SQL 函数，请为其添加回归测试。

### 迁移链测试

验证所有迁移脚本（`sql/pg_ripple--X.Y.Z--X.Y.Z+1.sql`）可以按顺序应用：

```bash
# 需要 pgrx PG18 运行
cargo pgrx start pg18
bash tests/test_migration_chain.sh
```

### 运行测试子集

```bash
# 按名称运行单个测试
cargo pgrx test pg18 -- test_name_pattern

# 运行测试时显示输出
cargo pgrx test pg18 -- --nocapture
```

---

## 代码约定

这些 conventions 由 CI 和代码审查强制执行。

### 仅安全 Rust

所有代码必须是安全 Rust。`unsafe` 仅在必需的 FFI 边界（pgrx 宏、共享内存访问）允许，并且必须包括解释其正确性的 `// SAFETY:` 注释。

### SQL 函数暴露

通过 `#[pg_extern]` 属性公开 SQL 函数。永远不要编写原始的 `PG_FUNCTION_INFO_V1` C 宏。

```rust
#[pg_extern]
fn my_function(input: &str) -> String {
    // 实现
}
```

### 内部 SQL 的 SPI

对扩展代码内执行的所有 SQL 使用 `pgrx::SpiClient`。永远不要使用原始 libpq 或基于字符串的查询执行。

```rust
Spi::connect(|client| {
    client.select("SELECT count(*) FROM _pg_ripple.dictionary", None, None)?;
    Ok(())
})?;
```

### 处处整数连接

SPARQL 到 SQL 的翻译必须在生成 SQL **之前**将所有绑定术语编码为 `i64`。VP 表查询绝不能包含字符串比较 — 这是一个错误。

### 表名不使用动态 SQL 字符串连接

始终在 `_pg_ripple.predicates` 中查找 VP 表 OID，并使用带有适当转义的 `format!` 风格引号。永远不要将用户输入插入到表名中。

### 错误消息

遵循 PostgreSQL 风格：第一个单词小写，没有尾随句点。

```rust
// 好
return Err(pg_ripple_error!("dictionary encode failed: hash collision detected"));

// 坏
return Err(pg_ripple_error!("Dictionary encode failed: hash collision detected."));
```

### 批量字典操作

对字典的所有批量插入使用 `ON CONFLICT DO NOTHING … RETURNING`。永远不要使用 SELECT 然后 INSERT 模式。

---

## 项目结构

```
src/
├── lib.rs              # 入口点，_PG_init，GUC 参数
├── dictionary/         # IRI/blank-node/literal → i64 编码器
├── storage/            # VP 表，HTAP delta/main，合并工作线程
├── sparql/             # SPARQL → 代数 → SQL → SPI
├── datalog/            # Datalog 解析器、分层器、SQL 编译器
├── shacl/              # SHACL 形状 → DDL 约束 + 验证
├── export/             # Turtle / N-Triples / JSON-LD 序列化
├── stats/              # 监控，pg_stat_statements 集成
└── admin/              # 真空，重新索引，前缀注册表

sql/                    # 迁移脚本和回归测试 SQL
tests/                  # 基于 shell 的集成测试
docs/                   # mdBook 文档站点
```

---

## 拉取请求工作流

### 分支策略

- **除非当前分支是 `main`，否则不要从 `main` 创建新分支**。
- 使用描述性分支名称：`feat/sparql-lateral`、`fix/dictionary-collision`、`docs/glossary`。

### 打开 PR 之前

1. **运行所有测试** 并确保它们通过：

```bash
cargo pgrx test pg18
cargo pgrx regress pg18
```

2. **运行 clippy** 无警告：

```bash
cargo clippy --all-targets -- -D warnings
```

3. **格式化代码**：

```bash
cargo fmt --check
```

4. **更新文档** 如果您更改了任何 SQL 函数签名或添加了新函数。

5. **创建或更新迁移脚本** 如果发布版本已更改（见下文）。

### 提交消息

- 使用现在时："add lateral join support" 而不是 "added lateral join support"
- 将离散更改分组到单独的提交中
- 引用问题编号（如果适用）："fix dictionary collision (#42)"

### 迁移脚本

每个版本都需要一个迁移脚本（`sql/pg_ripple--X.Y.Z--X.Y.Z+1.sql`），即使它只包含注释。有关完整清单，请参阅[发布过程](release-process.md)。

---

## 文档贡献

文档站点使用 [mdBook](https://rust-lang.github.io/mdBook/) 和 [mdbook-admonish](https://github.com/tommilligan/mdbook-admonish) 插件作为标注框。

### 在本地构建文档

```bash
# 安装 mdbook 和插件
cargo install mdbook mdbook-admonish

# 构建和服务
cd docs
mdbook serve --open
```

### 标注语法

对标注框使用带 `admonish` 的围栏代码块：

````markdown
```admonish tip title="性能"
对于大型数据集使用 `load_ntriples_file()` — 比字符串加载快 10 倍。
```

```admonish warning
此操作无法撤消。
```

```admonish note
自 v0.16.0 起可用。
```
````

### 添加新页面

1. 在适当的 `docs/src/` 子目录中创建 Markdown 文件。
2. 将页面添加到 `docs/src/SUMMARY.md`。
3. 运行 `mdbook build` 验证它可以编译。

---

## 基于属性的测试（v0.46.0）

pg_ripple 使用 `proptest` 进行随机化的基于属性的测试，这些测试断言代数不变量。这些测试完全在纯 Rust 中运行 — 不需要数据库连接。

### 运行 proptest 套件

```bash
# 运行所有基于属性的测试
cargo test --test proptest_suite

# 运行更多案例（默认：256）
PROPTEST_CASES=10000 cargo test --test proptest_suite

# 运行特定套件
cargo test --test proptest_suite sparql_roundtrip
cargo test --test proptest_suite dictionary
cargo test --test proptest_suite jsonld_framing
```

### 添加新的属性测试

1. 将您的测试添加到 `tests/proptest/` 中的适当文件：
   - SPARQL 翻译器不变量 → `sparql_roundtrip.rs`
   - 字典编码器不变量 → `dictionary.rs`
   - JSON-LD 框架不变量 → `jsonld_framing.rs`
   - 新域 → 创建 `tests/proptest/<domain>.rs` 并将 `mod <domain>;` 添加到 `tests/proptest_suite.rs`

2. 对属性测试使用 `proptest!` 宏；对确定性夹具使用常规 `#[test]`。

3. 使用 `PROPTEST_CASES=10000` 运行套件以验证 10,000 个用例通过。

### 调试 proptest 失败

当测试失败时，proptest 会打印最小的失败输入。重现它：

```rust
// 添加到失败的测试以修复种子：
ProptestConfig::with_cases(1).with_proptest_rng(seed)
```

---

## 模糊测试（v0.46.0）

pg_ripple 使用 `cargo-fuzz` 针对任意字节序列测试联邦结果解码器。

### 运行模糊目标

```bash
# 安装 cargo-fuzz
cargo install cargo-fuzz

# 运行 10 分钟
cargo fuzz run federation_result -- -max_total_time=600

# 无限期运行
cargo fuzz run federation_result

# 最小化崩溃的语料库条目
cargo fuzz tmin federation_result artifacts/federation_result/crash-<hash>
```

### 添加新的模糊目标

1. 在 `fuzz/fuzz_targets/<target_name>.rs` 中使用模糊目标函数创建模糊目标。
2. 将 `[[bin]]` 条目添加到 `fuzz/Cargo.toml`。
3. 将目标添加到 `.github/workflows/ci.yml` 中的 `fuzz-<target_name>` CI 作业。

### 模糊目标合约

每个模糊目标必须：
- 使用 `#![no_main]` 和 `libfuzzer_sys::fuzz_target!`
- **永远不要 panic** 无论输入如何（panics 被视为模糊失败）
- 对无效输入返回 `Err(...)`，永远不要崩溃

---

## 报告问题

在提交错误报告时，请包括：

- **pg_ripple 版本**：`SELECT pg_ripple.canary();` 和 `\dx pg_ripple` 的输出
- **PostgreSQL 版本**：`SELECT version();`
- **最小重现器**：触发该问题的最小 SQL 脚本
- **完整错误输出**：在 psql 中使用 `\errverbose` 获取详细的错误上下文
- **平台**：OS 和架构

```admonish warning title="安全问题"
如果您发现安全漏洞，请通过 GitHub Security Advisories 私下报告，而不是打开公共问题。
```