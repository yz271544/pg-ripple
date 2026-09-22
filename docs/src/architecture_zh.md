# 架构：子系统依赖图

本文档描述了 pg_ripple 的主要子系统及其依赖关系。
它作为模块分解工作（A16）的一部分在 v0.114.0 中引入。

## 子系统图

```
┌─────────────────────────────────────────────────────────────────────┐
│                         查询层                                       │
│  src/sparql/                    src/sparql/wcoj/                     │
│  ├─ parse.rs                    ├─ mod.rs（协调器）                  │
│  ├─ plan.rs                     ├─ executor.rs                       │
│  ├─ execute.rs                  ├─ trie.rs                           │
│  └─ decode.rs                   └─ leapfrog.rs                       │
│                                                                      │
│  src/sparql/embedding/                                               │
│  ├─ mod.rs                                                           │
│  ├─ index.rs（API 客户端，pgvector）                                 │
│  ├─ hybrid.rs（混合 SPARQL+向量搜索）                                │
│  └─ rag.rs（RAG 检索）                                                │
└──────────────────────────────────┬──────────────────────────────────┘
                                   │ uses
┌──────────────────────────────────▼──────────────────────────────────┐
│                         推理层                                       │
│                                                                      │
│  src/datalog/        src/datalog_api/     src/skos/                  │
│  （规则引擎，        ├─ parse.rs          ├─ mod.rs                  │
│   分层器，           ├─ validate.rs       ├─ bundle.rs               │
│   SQL 编译器，        ├─ explain.rs        ├─ inference.rs            │
│   RDFS/OWL RL）       └─ conflict.rs       ├─ broader_narrower.rs     │
│                                           └─ export.rs               │
│                                                                      │
│  src/shacl/          src/shacl/validator/                            │
│  (shapes→DDL，        ├─ mod.rs                                       │
│   异步管道）          ├─ node.rs                                      │
│                      ├─ property.rs                                  │
│                      ├─ sparql.rs                                    │
│                      └─ severity.rs                                  │
└──────────────────────────────────┬──────────────────────────────────┘
                                   │ uses
┌──────────────────────────────────▼──────────────────────────────────┐
│                         存储层                                       │
│                                                                      │
│  src/storage/        src/dictionary/      src/citus/                 │
│  （VP 表，            （IRI/BNode/Lit →    ├─ mod.rs（检测）           │
│   HTAP delta/main，   i64 via XXH3-128）    ├─ shard_pruning.rs        │
│   合并工作线程）                           ├─ ddl_hooks.rs            │
│                                           ├─ query_rewriting.rs      │
│                                           └─ rebalance.rs            │
└─────────────────────────────────────────────────────────────────────┘
```

## 关键依赖关系

| 子系统 | 依赖于 | 注释 |
|-----------|-----------|-------|
| SKOS（`src/skos/`） | Datalog（`src/datalog/`） | Bundle 加载为 SKOS 闭包注入 Datalog 规则 |
| OWL RL | Datalog（`src/datalog/`） | OWL 2 RL 蕴含规则编译为 Datalog 分层 |
| NS-RL（神经符号） | `sparql/embedding/` + Datalog + `datalog_api/conflict` | 将向量相似性与基于规则的推理相结合 |
| 冲突检测 | SHACL（`src/shacl/`）+ Datalog | 基于格的冲突检查使用 shape 验证和 Datalog 约束 |
| 假设推理 | 存储（`src/storage/`） | 为 what-if 查询创建临时 VP 表快照 |
| 视图（`src/views/`） | SPARQL + Datalog + 存储 | CONSTRUCT/DESCRIBE/ASK 视图将 SPARQL 查询包装为 SQL 视图 |
| Citus（`src/citus/`） | 存储 | 分片修剪读取 VP 谓词目录和 VP 表 OID |
| PageRank（`src/pagerank/`） | Datalog + 存储 | 具有 IVM 的 Datalog 原生 PageRank |

## 模块大小策略

每个 `.rs` 文件报告为 **800 LOC**，限制为 **1,200 LOC**（CI 硬性失败）。
该检查通过每个 PR 上的 `scripts/check_module_size.sh` 运行。超过 800 行的文件将被报告以供审查；超过 1,200 行的文件将使 CI 失败。

当模块超出限制时，将其分解为 `src/<module>/` 目录：

```
src/mymodule.rs                     # 之前（>1500 LOC）
  ↓
src/mymodule/mod.rs                 # 协调器 + 公共重新导出（< 400 LOC）
src/mymodule/submodule_a.rs         # 重点子模块（< 400 LOC）
src/mymodule/submodule_b.rs         # 重点子模块（< 400 LOC）
```

参考实现：`src/datalog/`、`src/sparql/`、`src/views/`、`src/skos/`。