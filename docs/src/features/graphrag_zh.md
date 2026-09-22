# GraphRAG 端到端

[Microsoft GraphRAG](https://github.com/microsoft/graphrag) 是一个流行的开源管道，可将非结构化文档转换为实体、关系、文本单元和社区摘要的结构化知识图谱。pg_ripple 是 GraphRAG 的**一流存储和查询后端**：您可以直接导入 GraphRAG 输出，使用 Datalog 规则丰富它，使用 SHACL 验证它，并将其作为 Parquet 导出回下游工具 —— 所有这些都无需离开 PostgreSQL。

此页面是规范的端到端指南。它假定您已有可用的 GraphRAG 输出（标准的 `entities.parquet`、`relationships.parquet`、`text_units.parquet` 文件）。

---

## pg_ripple 为 GraphRPG 添加了什么

| 库存 GraphRAG | 使用 pg_ripple |
|---|---|
| 输出以 Parquet 形式存储在磁盘上 | 输出以可查询 RDF 形式存储在 PostgreSQL 中 |
| 通过自定义 Python 管道进行搜索 | 通过 SPARQL、SHACL、向量 + RAG 进行搜索 |
| 无增量更新 —— 重建整个图 | 增量：将新文档加载到命名图中并重新推理 |
| 质量问题仅在查询时出现 | SHACL 在插入时捕获缺失标签、悬空关系等 |
| 一个进程拥有数据 | 多个读取器、事务性写入器、完整 ACID |

---

## 数据模型

GraphRAG 映射到四个 RDF 类：

| 类 | 它表示的内容 |
|---|---|
| `gr:Entity` | 命名实体（人员、组织、地点、事件……） |
| `gr:Relationship` | 两个实体之间的有向加权边 |
| `gr:TextUnit` | 提到实体的源文本块 |
| `gr:Community` | 相关实体的检测到集群 |
| `gr:CommunityReport` | 社区的 LLM 生成摘要 |

关键属性：

| 属性 | 域 | 范围 |
|---|---|---|
| `gr:title` | `gr:Entity` | `xsd:string` |
| `gr:type` | `gr:Entity` | `xsd:string`（PERSON、ORG……） |
| `gr:description` | 任何 | `xsd:string` |
| `gr:source` | `gr:Relationship` | `gr:Entity` |
| `gr:target` | `gr:Relationship` | `gr:Entity` |
| `gr:weight` | `gr:Relationship` | `xsd:float` |
| `gr:text` | `gr:TextUnit` | `xsd:string` |
| `gr:tokenCount` | `gr:TextUnit` | `xsd:integer` |

包括所有 SHACL 形状在内的完整本体位于 `examples/graphrag_byog.sql`。

---

## 端到端管道

```
   GraphRAG Python 管道
            │
            ▼
   entities.parquet                    │
   relationships.parquet               │  步骤 1：导入
   text_units.parquet                  │
            │                          │
            ▼                          │
   pg_ripple.import_graphrag_parquet() ▼
   ──────────────────────────────────────────
            │
            ▼
   步骤 2：使用 Datalog 丰富
   pg_ripple.load_rules_builtin('graphrag-enrichment')
   pg_ripple.infer('graphrag-enrichment')
   → 派生 gr:coworker、gr:collaborates、
     gr:indirectReport、gr:relatedOrg
            │
            ▼
   步骤 3：使用 SHACL 验证
   pg_ripple.load_shacl(...)
   pg_ripple.shacl_validate()
   → 捕获缺失标签、悬空引用
            │
            ▼
   步骤 4：查询
   - SPARQL 用于关系遍历
   - rag_context() 用于基础 LLM 提示
   - hybrid_search() 用于相似性 + 结构
            │
            ▼
   步骤 5（可选）：导出回 Parquet
   pg_ripple.export_graphrag_entities()
   pg_ripple.export_graphrag_relationships()
   → 用于下游 Microsoft GraphRAG 工具
```

---

## 步骤 1 —— 导入 GraphRAG 输出

```sql
-- 注册 gr: 前缀一次。
SELECT pg_ripple.register_prefix('gr', 'https://graphrag.org/ns/');

-- 导入三个核心文件。每个文件成为命名图中的三元组。
SELECT pg_ripple.import_graphrag_parquet(
    entities_path     := '/data/graphrag/entities.parquet',
    relationships_path:= '/data/graphrag/relationships.parquet',
    text_units_path   := '/data/graphrag/text_units.parquet',
    target_graph      := 'https://example.org/kb-2026-04'
);
-- 返回插入的三元组数。
```

对于临时加载，直接使用 `load_turtle()`：

```sql
SELECT pg_ripple.load_turtle($TTL$
@prefix gr:  <https://graphrag.org/ns/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<https://example.org/entity/alice>
    rdf:type  gr:Entity ;
    gr:title  "Alice" ;
    gr:type   "PERSON" .
$TTL$);
```

---

## 步骤 2 —— 使用 Datalog 丰富

捆绑的 `graphrag-enrichment` 规则集从原始 GraphRAG 输出派生四个有用的关系：

| 派生属性 | 含义 |
|---|---|
| `gr:coworker` | 两个实体都有针对同一组织的关系 |
| `gr:collaborates` | 两个实体在同一文本单元中共现 |
| `gr:indirectReport` | `gr:manages` 的传递闭包 |
| `gr:relatedOrg` | 两个组织共享一个实体桥梁 |

```sql
SELECT pg_ripple.load_rules_builtin('graphrag-enrichment');
SELECT pg_ripple.infer('graphrag-enrichment');
-- 返回派生三元组的计数（source = 1）。
```

您可以编写自己的规则 —— 请参阅[推理与演绎](reasoning-and-inference.md) —— 但捆绑集是一个很好的起点。

---

## 步骤 3 —— 使用 SHACL 验证

GraphRAG 输出是由 LLM 生成的，可能存在质量问题：没有标题的实体、指向已删除实体的关系、[0, 1] 之外的权重。使用 SHACL 捕获它们：

```sql
SELECT pg_ripple.load_shacl($TTL$
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix gr:   <https://graphrag.org/ns/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

gr:EntityShape a sh:NodeShape ;
    sh:targetClass gr:Entity ;
    sh:property [ sh:path gr:title ; sh:minCount 1 ; sh:datatype xsd:string ] ;
    sh:property [ sh:path gr:type  ; sh:in ( "PERSON" "ORG" "GEO" "EVENT" ) ] .

gr:RelationshipShape a sh:NodeShape ;
    sh:targetClass gr:Relationship ;
    sh:property [ sh:path gr:source ; sh:minCount 1 ; sh:class gr:Entity ] ;
    sh:property [ sh:path gr:target ; sh:minCount 1 ; sh:class gr:Entity ] ;
    sh:property [ sh:path gr:weight ; sh:datatype xsd:float ] .
$TTL$);

-- 验证存储中当前的所有内容。
SELECT focus_node, message FROM pg_ripple.shacl_validate();
```

设置 `pg_ripple.shacl_mode = 'sync'` 以在写入时拒绝违规插入，或设置 `'async'` 以将它们路由到死信队列。请参阅[验证数据质量](validating-data-quality.md)。

---

## 步骤 4 —— 查询丰富的图

GraphRAG 特定查询与 `rag_context()` 自然搭配：

```sql
-- 查找问题的 LLM 就绪上下文。
SELECT pg_ripple.rag_context(
    'Who collaborates with Alice on machine learning?',
    k := 10
);
```

对于纯粹的结构查询，直接使用 SPARQL：

```sql
-- Alice 同事的所有同事（2 跳搜索）。
SELECT * FROM pg_ripple.sparql($$
    PREFIX gr: <https://graphrag.org/ns/>
    SELECT DISTINCT ?friend WHERE {
        <https://example.org/entity/alice> gr:coworker/gr:coworker ?friend .
        FILTER(?friend != <https://example.org/entity/alice>)
    }
$$);
```

---

## 步骤 5 —— 往返导出

如果您需要将富化的图反馈给 Microsoft GraphRAG 的 Python 工具：

```sql
SELECT pg_ripple.export_graphrag_entities('', '/tmp/entities.parquet');
SELECT pg_ripple.export_graphrag_relationships('', '/tmp/relationships.parquet');
```

为默认图传递 `''`，或为命名图 IRI 传递以确定导出范围。

---

## 操作提示

- **每个摄取运行使用一个命名图**（`https://example.org/kb-2026-04`）。当重新摄取运行时，`clear_graph()` 以原子方式删除旧版本。
- **在加载之后运行推理**，而不是交错进行。推理对批处理友好且可并行化；按行推理则不然。
- **在丰富之后物化嵌入。** Datalog 派生属性改善 KGE 质量和 `rag_context()` 召回率。

---

## 另请参阅

- [AI 概述](ai-overview.md)
- [推理与演绎](reasoning-and-inference.md)
- [验证数据质量](validating-data-quality.md)
- [Cookbook：基于知识图谱的聊天机器人](../cookbook/grounded-chatbot.md)
- [GraphRAG 函数参考](../reference/graphrag-functions.md)
- [GraphRAG 本体参考](../reference/graphrag-ontology.md)

## 进一步阅读

- [博客：GraphRAG 知识导出](https://github.com/trickle-labs/pg-ripple/blob/main/blog/graphrag-knowledge-export.md) — 使用 pg_ripple 构建 Microsoft GraphRAG 管道