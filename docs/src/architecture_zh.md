# 架构：子系统依赖图

本文档描述了 pg_ripple 的主要子系统及其依赖关系。
它作为模块分解工作（A16）的一部分在 v0.114.0 中引入。

## 子系统映射

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Query Layer                                  │
│  src/sparql/                    src/sparql/wcoj/                     │
│  ├─ parse.rs                    ├─ mod.rs (coordinator)              │
│  ├─ plan.rs                     ├─ executor.rs                       │
│  ├─ execute.rs                  ├─ trie.rs                           │
│  └─ decode.rs                   └─ leapfrog.rs                       │
│                                                                      │
│  src/sparql/embedding/                                               │
│  ├─ mod.rs                                                           │
│  ├─ index.rs  (API client, pgvector)                                 │
│  ├─ hybrid.rs (hybrid SPARQL+vector search)                          │
│  └─ rag.rs    (RAG retrieval)                                        │
└──────────────────────────────────┬──────────────────────────────────┘
                                   │ 使用
┌──────────────────────────────────▼──────────────────────────────────┐
│                         Inference Layer                                │
│                                                                      │
│  src/datalog/        src/datalog_api/     src/skos/                  │
│  (rule engine,       ├─ parse.rs          ├─ mod.rs                  │
│   stratifier,        ├─ validate.rs       ├─ bundle.rs               │
│   SQL compiler,      ├─ explain.rs        ├─ inference.rs            │
│   RDFS/OWL RL)       └─ conflict.rs       ├─ broader_narrower.rs     │
│                                           └─ export.rs               │
│                                                                      │
│  src/shacl/          src/shacl/validator/                            │
│  (shapes→DDL,        ├─ mod.rs                                       │
│   async pipeline)    ├─ node.rs                                      │
│                      ├─ property.rs                                  │
│                      ├─ sparql.rs                                    │
│                      └─ severity.rs                                  │
└──────────────────────────────────┬──────────────────────────────────┘
                                   │ 使用
┌──────────────────────────────────▼──────────────────────────────────┐
│                         Storage Layer                                │
│                                                                      │
│  src/storage/        src/dictionary/      src/citus/                 │
│  (VP tables,         (IRI/BNode/Lit →     ├─ mod.rs (detection)      │
│   HTAP delta/main,   i64 via XXH3-128)    ├─ shard_pruning.rs        │
│   merge worker)                           ├─ ddl_hooks.rs            │
│                                           ├─ query_rewriting.rs      │
│                                           └─ rebalance.rs            │
└─────────────────────────────────────────────────────────────────────┘
```

## 关键依赖关系

| 子系统 | 依赖于 | 注释 |
|-----------|-----------|-------|
| SKOS (`src/skos/`) | Datalog (`src/datalog/`) | Bundle 加载为 SKOS 闭包注入 Datalog 规则 |
| OWL RL | Datalog (`src/datalog/`) | OWL 2 RL 推理规则编译为 Datalog 层 |
| NS-RL (神经-符号) | `sparql/embedding/` + Datalog + `datalog_api/conflict` | 将向量相似度与基于规则的推理相结合 |
| 冲突检测 | SHACL (`src/shacl/`) + Datalog | 基于网格的冲突检查同时使用形状验证和 Datalog 约束 |
| 假设推理 | 存储 (`src/storage/`) | 为假设查询创建临时 VP 表快照 |
| 视图 (`src/views/`) | SPARQL + Datalog + 存储 | CONSTRUCT/DESCRIBE/ASK 视图将 SPARQL 查询包装为 SQL 视图 |
| Citus (`src/citus/`) | 存储 | 分片剪枝读取 VP 谓词目录和 VP 表 OID |
| PageRank (`src/pagerank/`) | Datalog + 存储 | 具有 IVM 的 Datalog 原生 PageRank |

## 模块大小策略

每个 `.rs` 文件报告为 **800 LOC**，限制为 **1,200 LOC**（CI 硬失败）。
该门控通过每个 PR 上的 `scripts/check_module_size.sh` 运行。超过 800
行的文件报告以供审查；超过 1,200 行的文件 CI 失败。

当模块增长超出限制时，将其分解为 `src/<module>/` 目录：

```
src/mymodule.rs                     # 之前 (>1500 LOC)
  ↓
src/mymodule/mod.rs                 # 协调器和公共重新导出 (< 400 LOC)
src/mymodule/submodule_a.rs         # 专注的子模块 (< 400 LOC)
src/mymodule/submodule_b.rs         # 专注的子模块 (< 400 LOC)
```

参考实现：`src/datalog/`、`src/sparql/`、`src/views/`、`src/skos/`。