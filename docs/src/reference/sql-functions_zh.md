# SQL 函数参考

pg_ripple 公开的所有 157 个 SQL 函数，按用例分组。每个函数都位于 `pg_ripple` schema 中。

```admonish tip title="Schema 限定"
所有示例都假设 `SET search_path TO pg_ripple, public;`。如果您更喜欢显式限定，请为每个调用加上 `pg_ripple.` 前缀。
```

---

## 加载

用于插入和批量加载 RDF 数据的函数。

---

### `insert_triple`

将单个三元组插入默认图。

```sql
pg_ripple.insert_triple(
    subject   TEXT,
    predicate TEXT,
    object    TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.insert_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
);
```

---

### `load_turtle`

解析 Turtle 字符串并将所有三元组加载到默认图中。

```sql
pg_ripple.load_turtle(
    data   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_turtle('
@prefix ex: <https://example.org/> .
ex:alice ex:name "Alice" ;
         ex:knows ex:bob .
');
```

---

### `load_turtle_file`

从服务器端文件路径加载 Turtle。

```sql
pg_ripple.load_turtle_file(
    path   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_turtle_file('/data/ontology.ttl');
```

---

### `load_ntriples`

解析 N-Triples 字符串并将所有三元组加载到默认图中。

```sql
pg_ripple.load_ntriples(
    data   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_ntriples('
<https://example.org/alice> <https://example.org/name> "Alice" .
<https://example.org/alice> <https://example.org/knows> <https://example.org/bob> .
');
```

---

### `load_ntriples_file`

从服务器端文件路径加载 N-Triples。

```sql
pg_ripple.load_ntriples_file(
    path   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_ntriples_file('/data/dump.nt');
```

---

### `load_nquads`

解析 N-Quads 字符串并将三元组加载到各自的命名图中。

```sql
pg_ripple.load_nquads(
    data   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_nquads('
<https://example.org/alice> <https://example.org/name> "Alice" <https://example.org/g1> .
');
```

---

### `load_nquads_file`

从服务器端文件路径加载 N-Quads。

```sql
pg_ripple.load_nquads_file(
    path   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_nquads_file('/data/dump.nq');
```

---

### `load_trig`

解析 TriG 字符串并将三元组加载到其命名图中。

```sql
pg_ripple.load_trig(
    data   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_trig('
@prefix ex: <https://example.org/> .
ex:g1 { ex:alice ex:name "Alice" . }
');
```

---

### `load_trig_file`

从服务器端文件路径加载 TriG。

```sql
pg_ripple.load_trig_file(
    path   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_trig_file('/data/dataset.trig');
```

---

### `load_rdfxml`

解析 RDF/XML 字符串并将所有三元组加载到默认图中。

```sql
pg_ripple.load_rdfxml(
    data   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_rdfxml('
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="https://example.org/">
  <rdf:Description rdf:about="https://example.org/alice">
    <ex:name>Alice</ex:name>
  </rdf:Description>
</rdf:RDF>
');
```

---

### `load_rdfxml_file`

从服务器端文件路径加载 RDF/XML。

```sql
pg_ripple.load_rdfxml_file(
    path   TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_rdfxml_file('/data/ontology.rdf');
```

---

### `load_ntriples_into_graph`

解析 N-Triples 并加载到特定命名图中。

```sql
pg_ripple.load_ntriples_into_graph(
    data  TEXT,
    graph TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_ntriples_into_graph(
    '<https://example.org/alice> <https://example.org/name> "Alice" .',
    '<https://example.org/people>'
);
```

---

### `load_turtle_into_graph`

解析 Turtle 并加载到特定命名图中。

```sql
pg_ripple.load_turtle_into_graph(
    data  TEXT,
    graph TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_turtle_into_graph(
    '@prefix ex: <https://example.org/> . ex:alice ex:name "Alice" .',
    '<https://example.org/people>'
);
```

---

### `load_rdfxml_into_graph`

解析 RDF/XML 并加载到特定命名图中。

```sql
pg_ripple.load_rdfxml_into_graph(
    data  TEXT,
    graph TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_rdfxml_into_graph(
    '<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
              xmlns:ex="https://example.org/">
       <rdf:Description rdf:about="https://example.org/alice">
         <ex:name>Alice</ex:name>
       </rdf:Description>
     </rdf:RDF>',
    '<https://example.org/people>'
);
```

---

### `load_ntriples_file_into_graph`

将 N-Triples 从服务器端文件加载到命名图中。

```sql
pg_ripple.load_ntriples_file_into_graph(
    path  TEXT,
    graph TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_ntriples_file_into_graph(
    '/data/people.nt',
    '<https://example.org/people>'
);
```

---

### `load_turtle_file_into_graph`

将 Turtle 从服务器端文件加载到命名图中。

```sql
pg_ripple.load_turtle_file_into_graph(
    path  TEXT,
    graph TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_turtle_file_into_graph(
    '/data/people.ttl',
    '<https://example.org/people>'
);
```

---

### `load_rdfxml_file_into_graph`

将 RDF/XML 从服务器端文件加载到命名图中。

```sql
pg_ripple.load_rdfxml_file_into_graph(
    path  TEXT,
    graph TEXT,
    strict BOOLEAN DEFAULT false
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_rdfxml_file_into_graph(
    '/data/people.rdf',
    '<https://example.org/people>'
);
```

---

### `load_owl_ontology`

从 Turtle 加载 OWL 本体，提取供 Datalog 推理器使用的类和方法声明。

```sql
pg_ripple.load_owl_ontology(
    data TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.load_owl_ontology('
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <https://example.org/> .
ex:Person a owl:Class .
ex:knows a owl:ObjectProperty ;
    owl:related ex:knownBy .
');
```

---

### `apply_patch`

以原子方式应用 RDF 补丁（添加和删除）。

```sql
pg_ripple.apply_patch(
    additions TEXT,
    deletions TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.apply_patch(
    '<https://example.org/alice> <https://example.org/age> "31"^^<http://www.w3.org/2001/XMLSchema#integer> .',
    '<https://example.org/alice> <https://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .'
);
```

---

## 查询

用于使用 SPARQL 和文本搜索查询三元组的函数。

---

### `sparql`

执行 SPARQL SELECT 查询并以 JSON 对象集合的形式返回结果。

```sql
pg_ripple.sparql(
    query TEXT
) RETURNS SETOF JSON
```

```sql
SELECT * FROM pg_ripple.sparql('
    PREFIX ex: <https://example.org/>
    SELECT ?name WHERE { ex:alice ex:name ?name }
');
```

---

### `sparql_ask`

执行 SPARQL ASK 查询并返回布尔结果。

```sql
pg_ripple.sparql_ask(
    query TEXT
) RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.sparql_ask('
    PREFIX ex: <https://example.org/>
    ASK { ex:alice ex:knows ex:bob }
');
```

---

### `sparql_explain`

在不执行 SPARQL 查询的情况下返回其 SQL 执行计划。

```sql
pg_ripple.sparql_explain(
    query TEXT
) RETURNS TEXT
```

```sql
SELECT pg_ripple.sparql_explain('
    PREFIX ex: <https://example.org/>
    SELECT ?x WHERE { ?x ex:knows ex:bob }
');
```

---

### `explain_sparql`

返回详细的查询计划，显示 SPARQL 代数和生成的 SQL。

```sql
pg_ripple.explain_sparql(
    query TEXT
) RETURNS TEXT
```

```sql
SELECT pg_ripple.explain_sparql('
    PREFIX ex: <https://example.org/>
    SELECT ?x ?y WHERE { ?x ex:knows ?y }
');
```

---

### `sparql_construct`

执行 SPARQL CONSTRUCT 查询并以 JSON 形式返回三元组。

```sql
pg_ripple.sparql_construct(
    query TEXT
) RETURNS SETOF JSON
```

```sql
SELECT * FROM pg_ripple.sparql_construct('
    PREFIX ex: <https://example.org/>
    CONSTRUCT { ?x ex:friendOf ?y }
    WHERE { ?x ex:knows ?y }
');
```

---

### `sparql_describe`

执行 SPARQL DESCRIBE 查询并以 JSON 形式返回有关资源的所有三元组。

```sql
pg_ripple.sparql_describe(
    query TEXT
) RETURNS SETOF JSON
```

```sql
SELECT * FROM pg_ripple.sparql_describe('
    PREFIX ex: <https://example.org/>
    DESCRIBE ex:alice
');
```

---

### `sparql_construct_turtle`

执行 SPARQL CONSTRUCT 查询并以 Turtle 字符串的形式返回结果。

```sql
pg_ripple.sparql_construct_turtle(
    query TEXT
) RETURNS TEXT
```

```sql
SELECT pg_ripple.sparql_construct_turtle('
    PREFIX ex: <https://example.org/>
    CONSTRUCT { ?x ex:friendOf ?y }
    WHERE { ?x ex:knows ?y }
');
```

---

### `sparql_construct_jsonld`

执行 SPARQL CONSTRUCT 查询并以 JSON-LD 字符串的形式返回结果。

```sql
pg_ripple.sparql_construct_jsonld(
    query TEXT
) RETURNS TEXT
```

```sql
SELECT pg_ripple.sparql_construct_jsonld('
    PREFIX ex: <https://example.org/>
    CONSTRUCT { ?x ex:friendOf ?y }
    WHERE { ?x ex:knows ?y }
');
```

---

### `sparql_describe_turtle`

执行 SPARQL DESCRIBE 查询并以 Turtle 形式返回结果。

```sql
pg_ripple.sparql_describe_turtle(
    query TEXT
) RETURNS TEXT
```

```sql
SELECT pg_ripple.sparql_describe_turtle('
    PREFIX ex: <https://example.org/>
    DESCRIBE ex:alice
');
```

---

### `sparql_describe_jsonld`

执行 SPARQL DESCRIBE 查询并以 JSON-LD 形式返回结果。

```sql
pg_ripple.sparql_describe_jsonld(
    query TEXT
) RETURNS TEXT
```

```sql
SELECT pg_ripple.sparql_describe_jsonld('
    PREFIX ex: <https://example.org/>
    DESCRIBE ex:alice
');
```

---

### `sparql_update`

执行 SPARQL 更新操作（INSERT DATA、DELETE DATA 等）。

```sql
pg_ripple.sparql_update(
    query TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.sparql_update('
    PREFIX ex: <https://example.org/>
    INSERT DATA { ex:alice ex:age 30 }
');
```

---

### `find_triples`

在默认图中查找与模式匹配的三元组。传递 `NULL` 表示通配符。

```sql
pg_ripple.find_triples(
    subject   TEXT DEFAULT NULL,
    predicate TEXT DEFAULT NULL,
    object    TEXT DEFAULT NULL
) RETURNS TABLE(subject TEXT, predicate TEXT, object TEXT)
```

```sql
SELECT * FROM pg_ripple.find_triples(
    '<https://example.org/alice>', NULL, NULL
);
```

---

### `find_triples_in_graph`

在特定命名图中查找与模式匹配的三元组。

```sql
pg_ripple.find_triples_in_graph(
    subject   TEXT DEFAULT NULL,
    predicate TEXT DEFAULT NULL,
    object    TEXT DEFAULT NULL,
    graph     TEXT DEFAULT NULL
) RETURNS TABLE(subject TEXT, predicate TEXT, object TEXT, graph TEXT)
```

```sql
SELECT * FROM pg_ripple.find_triples_in_graph(
    NULL, NULL, NULL, '<https://example.org/people>'
);
```

---

### `triple_count`

返回默认图中的三元组总数。

```sql
pg_ripple.triple_count() RETURNS BIGINT
```

```sql
SELECT pg_ripple.triple_count();
```

---

### `triple_count_in_graph`

返回特定命名图中的三元组数。

```sql
pg_ripple.triple_count_in_graph(
    graph TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.triple_count_in_graph('<https://example.org/people>');
```

---

### `fts_index`

在字面值上构建或重建全文搜索索引。

```sql
pg_ripple.fts_index() RETURNS VOID
```

```sql
SELECT pg_ripple.fts_index();
```

---

### `fts_search`

通过全文搜索在字面值中搜索包含特定术语的三元组。

```sql
pg_ripple.fts_search(
    query TEXT,
    limit_rows INTEGER DEFAULT 100
) RETURNS TABLE(subject TEXT, predicate TEXT, object TEXT, rank REAL)
```

```sql
SELECT * FROM pg_ripple.fts_search('knowledge graph', 10);
```

---

## 图

用于管理命名图的函数。

---

### `create_graph`

创建命名图。

```sql
pg_ripple.create_graph(
    graph TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_graph('<https://example.org/people>');
```

---

### `drop_graph`

删除命名图及其所有三元组。

```sql
pg_ripple.drop_graph(
    graph TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_graph('<https://example.org/people>');
```

---

### `list_graphs`

列出所有命名图。

```sql
pg_ripple.list_graphs() RETURNS TABLE(graph TEXT, triple_count BIGINT)
```

```sql
SELECT * FROM pg_ripple.list_graphs();
```

---

### `clear_graph`

从图中删除所有三元组而不删除该图。

```sql
pg_ripple.clear_graph(
    graph TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.clear_graph('<https://example.org/people>');
```

---

## 字典

用于与将 IRI、空白节点和字面量映射到整数 ID 的字典编码器交互的函数。

```admonish note title="内部使用"
大多数用户从不需要直接调用字典函数。它们对于调试、性能调优和理解存储内部很有用。
```

---

### `encode_term`

将 IRI、字面量或空白节点编码为其整数 ID。

```sql
pg_ripple.encode_term(
    term TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.encode_term('<https://example.org/alice>');
```

---

### `decode_id`

将整数 ID 解码回其字符串表示形式。

```sql
pg_ripple.decode_id(
    id BIGINT
) RETURNS TEXT
```

```sql
SELECT pg_ripple.decode_id(42);
```

---

### `encode_triple`

将完整三元组（主语、谓词、对象）编码为整数 ID。

```sql
pg_ripple.encode_triple(
    subject   TEXT,
    predicate TEXT,
    object    TEXT
) RETURNS TABLE(s BIGINT, p BIGINT, o BIGINT)
```

```sql
SELECT * FROM pg_ripple.encode_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
);
```

---

### `decode_triple`

将三元组从整数 ID 解码回字符串形式。

```sql
pg_ripple.decode_triple(
    s BIGINT,
    p BIGINT,
    o BIGINT
) RETURNS TABLE(subject TEXT, predicate TEXT, object TEXT)
```

```sql
SELECT * FROM pg_ripple.decode_triple(1, 2, 3);
```

---

### `decode_id_full`

解码整数 ID，返回带有类型信息的完整词项。

```sql
pg_ripple.decode_id_full(
    id BIGINT
) RETURNS JSON
```

```sql
SELECT pg_ripple.decode_id_full(42);
```

---

### `lookup_iri`

在不插入的情况下查找特定 IRI 的整数 ID。

```sql
pg_ripple.lookup_iri(
    iri TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.lookup_iri('<https://example.org/alice>');
```

---

### `dictionary_stats`

返回有关字典表的统计信息。

```sql
pg_ripple.dictionary_stats() RETURNS JSON
```

```sql
SELECT pg_ripple.dictionary_stats();
```

---

### `prewarm_dictionary_hot`

将最常访问的字典条目加载到共享缓存中。

```sql
pg_ripple.prewarm_dictionary_hot(
    limit_rows INTEGER DEFAULT 10000
) RETURNS INTEGER
```

```sql
SELECT pg_ripple.prewarm_dictionary_hot(50000);
```

---

### `cache_stats`

返回字典 LRU 缓存的命中/未命中统计信息。

```sql
pg_ripple.cache_stats() RETURNS JSON
```

```sql
SELECT pg_ripple.cache_stats();
```

---

## 前缀

用于管理命名空间前缀缩写的函数。

---

### `register_prefix`

注册命名空间前缀以在 SPARQL 查询和输出中使用。

```sql
pg_ripple.register_prefix(
    prefix TEXT,
    iri    TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.register_prefix('ex', 'https://example.org/');
```

---

### `prefixes`

列出所有已注册的前缀。

```sql
pg_ripple.prefixes() RETURNS TABLE(prefix TEXT, iri TEXT)
```

```sql
SELECT * FROM pg_ripple.prefixes();
```

### `drop_prefix`

删除已注册的前缀。数据库所有者、超级用户或 `pg_ripple_admin` 角色可以修改注册表。

```sql
pg_ripple.drop_prefix(prefix TEXT) RETURNS BOOLEAN
```

### `prefix_registry_generation`

返回已注册前缀注册表的提交生成。

```sql
pg_ripple.prefix_registry_generation() RETURNS BIGINT
```

### `supported_surface`

返回所请求 API 版本的稳定功能配置文件。

```sql
pg_ripple.supported_surface(profile TEXT)
RETURNS TABLE(feature_name TEXT, optional_dependency TEXT,
              unsupported_combination TEXT, evidence_artifact TEXT)
```

当前的稳定配置文件是 `v1`。

---

## 验证

用于加载 SHACL 形状、验证数据和管理异步验证的函数。

---

### `load_shacl`

从 Turtle 字符串加载 SHACL 形状。

```sql
pg_ripple.load_shacl(
    shapes TEXT
) RETURNS INTEGER
```

```sql
SELECT pg_ripple.load_shacl('
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://example.org/> .
ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:name ; sh:minCount 1 ; sh:datatype xsd:string ] .
');
```

---

### `validate`

运行 SHACL 验证并返回验证报告。

```sql
pg_ripple.validate() RETURNS TABLE(
    focus_node TEXT,
    shape      TEXT,
    path       TEXT,
    severity   TEXT,
    message    TEXT
)
```

```sql
SELECT * FROM pg_ripple.validate();
```

---

### `list_shapes`

列出所有已加载的 SHACL 形状。

```sql
pg_ripple.list_shapes() RETURNS TABLE(shape TEXT, target TEXT, property_count INTEGER)
```

```sql
SELECT * FROM pg_ripple.list_shapes();
```

---

### `drop_shape`

按 IRI 删除 SHACL 形状。

```sql
pg_ripple.drop_shape(
    shape TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_shape('<https://example.org/PersonShape>');
```

---

### `enable_shacl_monitors`

在所有 VP 表上启用基于触发器的 SHACL 验证。

```sql
pg_ripple.enable_shacl_monitors() RETURNS VOID
```

```sql
SELECT pg_ripple.enable_shacl_monitors();
```

---

### `enable_shacl_dag_monitors`

使用 pg_trickle 启用 DAG 感知的 SHACL 监视器以进行异步验证。

```sql
pg_ripple.enable_shacl_dag_monitors() RETURNS VOID
```

```sql
SELECT pg_ripple.enable_shacl_dag_monitors();
```

---

### `disable_shacl_dag_monitors`

禁用 DAG 感知的 SHACL 监视器。

```sql
pg_ripple.disable_shacl_dag_monitors() RETURNS VOID
```

```sql
SELECT pg_ripple.disable_shacl_dag_monitors();
```

---

### `list_shacl_dag_monitors`

列出所有活动的 DAG SHACL 监视器。

```sql
pg_ripple.list_shacl_dag_monitors() RETURNS TABLE(shape TEXT, predicate TEXT, enabled BOOLEAN)
```

```sql
SELECT * FROM pg_ripple.list_shacl_dag_monitors();
```

---

### `process_validation_queue`

处理异步 SHACL 验证队列中的挂起项。

```sql
pg_ripple.process_validation_queue(
    batch_size INTEGER DEFAULT 100
) RETURNS INTEGER
```

```sql
SELECT pg_ripple.process_validation_queue(500);
```

---

### `validation_queue_length`

返回验证队列中挂起项的数量。

```sql
pg_ripple.validation_queue_length() RETURNS BIGINT
```

```sql
SELECT pg_ripple.validation_queue_length();
```

---

### `dead_letter_count`

返回验证死信队列中的项目数。

```sql
pg_ripple.dead_letter_count() RETURNS BIGINT
```

```sql
SELECT pg_ripple.dead_letter_count();
```

---

### `dead_letter_queue`

返回验证死信队列的内容。

```sql
pg_ripple.dead_letter_queue() RETURNS TABLE(
    id         BIGINT,
    triple_id  BIGINT,
    shape      TEXT,
    error      TEXT,
    created_at TIMESTAMPTZ
)
```

```sql
SELECT * FROM pg_ripple.dead_letter_queue();
```

---

### `drain_dead_letter_queue`

从死信队列中删除并返回所有项。

```sql
pg_ripple.drain_dead_letter_queue() RETURNS INTEGER
```

```sql
SELECT pg_ripple.drain_dead_letter_queue();
```

---

## 推理

用于 Datalog 规则管理和推理的函数。

```admonish tip title="内置规则集"
pg_ripple 附带 RDFS 和 OWL RL 规则集。使用 `load_rules_builtin('rdfs')` 或 `load_rules_builtin('owl-rl')` 加载它们。
```

---

### `load_rules`

从程序字符串加载命名 Datalog 规则集。

```sql
pg_ripple.load_rules(
    name    TEXT,
    program TEXT
) RETURNS INTEGER
```

```sql
SELECT pg_ripple.load_rules('transitive-knows', '
    knows(X, Z) :- knows(X, Y), knows(Y, Z).
');
```

---

### `load_rules_builtin`

加载内置规则集（rdfs、owl-rl）。

```sql
pg_ripple.load_rules_builtin(
    name TEXT
) RETURNS INTEGER
```

```sql
SELECT pg_ripple.load_rules_builtin('owl-rl');
```

---

### `list_rules`

列出所有已加载的规则集。

```sql
pg_ripple.list_rules() RETURNS TABLE(name TEXT, rule_count INTEGER, enabled BOOLEAN)
```

```sql
SELECT * FROM pg_ripple.list_rules();
```

---

### `drop_rules`

按名称删除规则集。

```sql
pg_ripple.drop_rules(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_rules('transitive-knows');
```

---

### `enable_rule_set`

为推理启用规则集。

```sql
pg_ripple.enable_rule_set(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.enable_rule_set('owl-rl');
```

---

### `disable_rule_set`

禁用规则集（已推导的三元组不会被删除）。

```sql
pg_ripple.disable_rule_set(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.disable_rule_set('owl-rl');
```

---

### `infer`

使用所有已启用的规则集运行物化（半朴素求值）。

```sql
pg_ripple.infer() RETURNS BIGINT
```

```sql
SELECT pg_ripple.infer();
```

---

### `infer_with_stats`

运行物化并返回迭代统计信息。

```sql
pg_ripple.infer_with_stats() RETURNS JSON
```

```sql
SELECT pg_ripple.infer_with_stats();
```

---

### `infer_goal`

为特定查询模式运行目标导向的推理。

```sql
pg_ripple.infer_goal(
    subject   TEXT DEFAULT NULL,
    predicate TEXT DEFAULT NULL,
    object    TEXT DEFAULT NULL
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.infer_goal(
    '<https://example.org/alice>',
    '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>',
    NULL
);
```

---

### `infer_agg`

运行 Datalog 聚合规则（min、max、sum、count）。

```sql
pg_ripple.infer_agg() RETURNS BIGINT
```

```sql
SELECT pg_ripple.infer_agg();
```

---

### `infer_demand`

使用 magic sets 优化运行需求驱动的推理。

```sql
pg_ripple.infer_demand(
    subject   TEXT DEFAULT NULL,
    predicate TEXT DEFAULT NULL,
    object    TEXT DEFAULT NULL
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.infer_demand(
    '<https://example.org/alice>',
    '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>',
    NULL
);
```

---

### `infer_wfs`

为带否定的程序运行良好基础语义求值。

```sql
pg_ripple.infer_wfs() RETURNS BIGINT
```

```sql
SELECT pg_ripple.infer_wfs();
```

---

### `tabling_stats`

返回有关制表备忘存储的统计信息。

```sql
pg_ripple.tabling_stats() RETURNS JSON
```

```sql
SELECT pg_ripple.tabling_stats();
```

---

### `rule_plan_cache_stats`

返回有关 Datalog 规则计划缓存的统计信息。

```sql
pg_ripple.rule_plan_cache_stats() RETURNS JSON
```

```sql
SELECT pg_ripple.rule_plan_cache_stats();
```

---

### `check_constraints`

运行 Datalog 约束规则并报告违规。

```sql
pg_ripple.check_constraints() RETURNS TABLE(rule TEXT, subject TEXT, message TEXT)
```

```sql
SELECT * FROM pg_ripple.check_constraints();
```

---

## 导出

用于将三元组序列化为各种格式的函数。

---

### `export_ntriples`

将所有三元组导出为 N-Triples 字符串。

```sql
pg_ripple.export_ntriples() RETURNS TEXT
```

```sql
SELECT pg_ripple.export_ntriples();
```

---

### `export_nquads`

将所有三元组（带命名图）导出为 N-Quads 字符串。

```sql
pg_ripple.export_nquads() RETURNS TEXT
```

```sql
SELECT pg_ripple.export_nquads();
```

---

### `export_turtle`

将所有三元组导出为 Turtle 字符串。

```sql
pg_ripple.export_turtle() RETURNS TEXT
```

```sql
SELECT pg_ripple.export_turtle();
```

---

### `export_jsonld`

将所有三元组导出为 JSON-LD 字符串。

```sql
pg_ripple.export_jsonld() RETURNS TEXT
```

```sql
SELECT pg_ripple.export_jsonld();
```

---

### `export_turtle_stream`

将三元组作为 Turtle 块的流集合导出，适用于大型数据集。

```sql
pg_ripple.export_turtle_stream(
    batch_size INTEGER DEFAULT 1000
) RETURNS SETOF TEXT
```

```sql
SELECT * FROM pg_ripple.export_turtle_stream(5000);
```

---

### `export_jsonld_stream`

将三元组作为 JSON-LD 块的流集合导出，适用于大型数据集。

```sql
pg_ripple.export_jsonld_stream(
    batch_size INTEGER DEFAULT 1000
) RETURNS SETOF TEXT
```

```sql
SELECT * FROM pg_ripple.export_jsonld_stream(5000);
```

---

### `export_graphrag_entities`

以 Microsoft GraphRAG 或兼容工具的 GraphRAG 实体格式导出实体。

```sql
pg_ripple.export_graphrag_entities() RETURNS SETOF JSON
```

```sql
SELECT * FROM pg_ripple.export_graphrag_entities();
```

---

### `export_graphrag_relationships`

以 GraphRAG 关系格式导出关系。

```sql
pg_ripple.export_graphrag_relationships() RETURNS SETOF JSON
```

```sql
SELECT * FROM pg_ripple.export_graphrag_relationships();
```

---

### `export_graphrag_text_units`

以 GraphRAG 文本单元格式导出文本单元。

```sql
pg_ripple.export_graphrag_text_units() RETURNS SETOF JSON
```

```sql
SELECT * FROM pg_ripple.export_graphrag_text_units();
```

---

## JSON-LD 框架

用于 JSON-LD 框架和树形输出的函数。

---

### `jsonld_frame_to_sparql`

将 JSON-LD 框架转换为 SPARQL CONSTRUCT 查询。

```sql
pg_ripple.jsonld_frame_to_sparql(
    frame JSON
) RETURNS TEXT
```

```sql
SELECT pg_ripple.jsonld_frame_to_sparql('{
    "@type": "https://example.org/Person",
    "https://example.org/name": {}
}'::json);
```

---

### `export_jsonld_framed`

将由 JSON-LD 框架塑造的三元组导出为 JSON-LD 字符串。

```sql
pg_ripple.export_jsonld_framed(
    frame JSON
) RETURNS TEXT
```

```sql
SELECT pg_ripple.export_jsonld_framed('{
    "@type": "https://example.org/Person",
    "https://example.org/name": {},
    "https://example.org/knows": { "@type": "https://example.org/Person" }
}'::json);
```

---

### `export_jsonld_framed_stream`

将框架化的 JSON-LD 作为块的流集合导出。

```sql
pg_ripple.export_jsonld_framed_stream(
    frame      JSON,
    batch_size INTEGER DEFAULT 100
) RETURNS SETOF TEXT
```

```sql
SELECT * FROM pg_ripple.export_jsonld_framed_stream('{
    "@type": "https://example.org/Person"
}'::json, 50);
```

---

### `jsonld_frame`

将 JSON-LD 框架应用于现有 JSON-LD 文档。

```sql
pg_ripple.jsonld_frame(
    document JSON,
    frame    JSON
) RETURNS JSON
```

```sql
SELECT pg_ripple.jsonld_frame(
    pg_ripple.export_jsonld()::json,
    '{"@type": "https://example.org/Person"}'::json
);
```

---

## 视图

用于创建和管理物化的 SPARQL、Datalog、CONSTRUCT、DESCRIBE、ASK 和框架视图的函数。

```admonish note title="视图生命周期"
视图由 PostgreSQL 表或视图支持。使用相应的 `drop_*_view` 函数将其删除。删除扩展也会删除所有视图。
```

---

### `create_sparql_view`

创建由 SPARQL SELECT 查询支持的 PostgreSQL 视图。

```sql
pg_ripple.create_sparql_view(
    name  TEXT,
    query TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_sparql_view('people', '
    PREFIX ex: <https://example.org/>
    SELECT ?name WHERE { ?person a ex:Person ; ex:name ?name }
');
```

---

### `drop_sparql_view`

删除 SPARQL 视图。

```sql
pg_ripple.drop_sparql_view(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_sparql_view('people');
```

---

### `list_sparql_views`

列出所有 SPARQL 视图。

```sql
pg_ripple.list_sparql_views() RETURNS TABLE(name TEXT, query TEXT)
```

```sql
SELECT * FROM pg_ripple.list_sparql_views();
```

---

### `create_datalog_view`

创建由 Datalog 规则支持的 PostgreSQL 视图。

```sql
pg_ripple.create_datalog_view(
    name  TEXT,
    rule  TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_datalog_view('ancestor',
    'ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).'
);
```

---

### `create_datalog_view_from_rule_set`

从命名规则集的头谓词创建视图。

```sql
pg_ripple.create_datalog_view_from_rule_set(
    view_name     TEXT,
    rule_set_name TEXT,
    head_predicate TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_datalog_view_from_rule_set(
    'inferred_types', 'owl-rl', 'rdf:type'
);
```

---

### `drop_datalog_view`

删除 Datalog 视图。

```sql
pg_ripple.drop_datalog_view(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_datalog_view('ancestor');
```

---

### `list_datalog_views`

列出所有 Datalog 视图。

```sql
pg_ripple.list_datalog_views() RETURNS TABLE(name TEXT, rule_set TEXT, head TEXT)
```

```sql
SELECT * FROM pg_ripple.list_datalog_views();
```

---

### `create_framing_view`

创建由 JSON-LD 框架支持的 PostgreSQL 视图。

```sql
pg_ripple.create_framing_view(
    name  TEXT,
    frame JSON
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_framing_view('person_frame', '{
    "@type": "https://example.org/Person",
    "https://example.org/name": {}
}'::json);
```

---

### `drop_framing_view`

删除框架视图。

```sql
pg_ripple.drop_framing_view(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_framing_view('person_frame');
```

---

### `list_framing_views`

列出所有框架视图。

```sql
pg_ripple.list_framing_views() RETURNS TABLE(name TEXT, frame JSON)
```

```sql
SELECT * FROM pg_ripple.list_framing_views();
```

---

### `create_construct_view`

创建由 SPARQL CONSTRUCT 查询支持的视图。

```sql
pg_ripple.create_construct_view(
    name  TEXT,
    query TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_construct_view('friends', '
    PREFIX ex: <https://example.org/>
    CONSTRUCT { ?a ex:friendOf ?b }
    WHERE { ?a ex:knows ?b }
');
```

---

### `drop_construct_view`

删除 CONSTRUCT 视图。

```sql
pg_ripple.drop_construct_view(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_construct_view('friends');
```

---

### `list_construct_views`

列出所有 CONSTRUCT 视图。

```sql
pg_ripple.list_construct_views() RETURNS TABLE(name TEXT, query TEXT)
```

```sql
SELECT * FROM pg_ripple.list_construct_views();
```

---

### `create_describe_view`

创建由 SPARQL DESCRIBE 查询支持的视图。

```sql
pg_ripple.create_describe_view(
    name  TEXT,
    query TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_describe_view('alice_detail', '
    PREFIX ex: <https://example.org/>
    DESCRIBE ex:alice
');
```

---

### `drop_describe_view`

删除 DESCRIBE 视图。

```sql
pg_ripple.drop_describe_view(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_describe_view('alice_detail');
```

---

### `list_describe_views`

列出所有 DESCRIBE 视图。

```sql
pg_ripple.list_describe_views() RETURNS TABLE(name TEXT, query TEXT)
```

```sql
SELECT * FROM pg_ripple.list_describe_views();
```

---

### `create_ask_view`

创建由 SPARQL ASK 查询支持的视图。

```sql
pg_ripple.create_ask_view(
    name  TEXT,
    query TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_ask_view('has_alice', '
    PREFIX ex: <https://example.org/>
    ASK { ex:alice ex:name ?n }
');
```

---

### `drop_ask_view`

删除 ASK 视图。

```sql
pg_ripple.drop_ask_view(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_ask_view('has_alice');
```

---

### `list_ask_views`

列出所有 ASK 视图。

```sql
pg_ripple.list_ask_views() RETURNS TABLE(name TEXT, query TEXT)
```

```sql
SELECT * FROM pg_ripple.list_ask_views();
```

---

### `create_extvp`

为谓词对创建扩展 VP (ExtVP) 索引以加速星形模式连接。

```sql
pg_ripple.create_extvp(
    predicate1 TEXT,
    predicate2 TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.create_extvp(
    '<https://example.org/name>',
    '<https://example.org/age>'
);
```

---

### `drop_extvp`

删除 ExtVP 索引。

```sql
pg_ripple.drop_extvp(
    predicate1 TEXT,
    predicate2 TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.drop_extvp(
    '<https://example.org/name>',
    '<https://example.org/age>'
);
```

---

### `list_extvp`

列出所有 ExtVP 索引。

```sql
pg_ripple.list_extvp() RETURNS TABLE(predicate1 TEXT, predicate2 TEXT, row_count BIGINT)
```

```sql
SELECT * FROM pg_ripple.list_extvp();
```

---

## 联邦

用于管理 SPARQL 联邦端点的函数。

---

### `register_endpoint`

为联邦查询注册远程 SPARQL 端点。

```sql
pg_ripple.register_endpoint(
    name TEXT,
    url  TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.register_endpoint('wikidata', 'https://query.wikidata.org/sparql');
```

---

### `set_endpoint_complexity`

为联邦端点设置复杂度权重（由查询计划器使用）。

```sql
pg_ripple.set_endpoint_complexity(
    name       TEXT,
    complexity REAL
) RETURNS VOID
```

```sql
SELECT pg_ripple.set_endpoint_complexity('wikidata', 2.5);
```

---

### `remove_endpoint`

删除已注册的端点。

```sql
pg_ripple.remove_endpoint(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.remove_endpoint('wikidata');
```

---

### `disable_endpoint`

在不删除的情况下临时禁用端点。

```sql
pg_ripple.disable_endpoint(
    name TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.disable_endpoint('wikidata');
```

---

### `list_endpoints`

列出所有已注册的联邦端点。

```sql
pg_ripple.list_endpoints() RETURNS TABLE(name TEXT, url TEXT, enabled BOOLEAN, complexity REAL)
```

```sql
SELECT * FROM pg_ripple.list_endpoints();
```

---

### `register_vector_endpoint`

为混合 SPARQL+向量查询注册向量相似度搜索端点。

```sql
pg_ripple.register_vector_endpoint(
    name  TEXT,
    url   TEXT,
    model TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.register_vector_endpoint(
    'openai', 'https://api.openai.com/v1/embeddings', 'text-embedding-3-small'
);
```

---

## 向量/混合搜索

用于向量嵌入、相似度搜索和 RAG 检索的函数。

```admonish warning title="需要 pgvector"
所有向量函数都需要安装 pgvector。设置 `pg_ripple.pgvector_enabled = off` 以禁用而不卸载。
```

---

### `store_embedding`

为实体存储预计算的嵌入向量。

```sql
pg_ripple.store_embedding(
    entity TEXT,
    model  TEXT,
    vector VECTOR
) RETURNS VOID
```

```sql
SELECT pg_ripple.store_embedding(
    '<https://example.org/alice>',
    'text-embedding-3-small',
    '[0.1, 0.2, 0.3]'::vector
);
```

---

### `similar_entities`

通过向量距离查找与给定实体相似的实体。

```sql
pg_ripple.similar_entities(
    entity TEXT,
    model  TEXT DEFAULT 'text-embedding-3-small',
    k      INTEGER DEFAULT 10
) RETURNS TABLE(entity TEXT, distance REAL)
```

```sql
SELECT * FROM pg_ripple.similar_entities('<https://example.org/alice>');
```

---

### `embed_entities`

为与 SPARQL 模式匹配的实体生成并存储嵌入。

```sql
pg_ripple.embed_entities(
    query TEXT,
    model TEXT DEFAULT 'text-embedding-3-small'
) RETURNS INTEGER
```

```sql
SELECT pg_ripple.embed_entities('
    PREFIX ex: <https://example.org/>
    SELECT ?entity WHERE { ?entity a ex:Person }
');
```

---

### `refresh_embeddings`

为底层数据已更改的实体重新计算嵌入。

```sql
pg_ripple.refresh_embeddings(
    model TEXT DEFAULT 'text-embedding-3-small'
) RETURNS INTEGER
```

```sql
SELECT pg_ripple.refresh_embeddings();
```

---

### `list_embedding_models`

列出所有具有存储向量的嵌入模型。

```sql
pg_ripple.list_embedding_models() RETURNS TABLE(model TEXT, entity_count BIGINT, dimensions INTEGER)
```

```sql
SELECT * FROM pg_ripple.list_embedding_models();
```

---

### `add_embedding_triples`

将相似度关系物化为 RDF 三元组。

```sql
pg_ripple.add_embedding_triples(
    model     TEXT DEFAULT 'text-embedding-3-small',
    threshold REAL DEFAULT 0.8,
    predicate TEXT DEFAULT '<https://example.org/similarTo>'
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.add_embedding_triples('text-embedding-3-small', 0.9);
```

---

### `contextualize_entity`

返回实体邻域的文本摘要，用作 LLM 上下文。

```sql
pg_ripple.contextualize_entity(
    entity TEXT,
    hops   INTEGER DEFAULT 2
) RETURNS TEXT
```

```sql
SELECT pg_ripple.contextualize_entity('<https://example.org/alice>', 3);
```

---

### `hybrid_search`

结合 SPARQL 图模式匹配和向量相似度（倒数排名融合）。

```sql
pg_ripple.hybrid_search(
    sparql_query TEXT,
    vector_query TEXT,
    k            INTEGER DEFAULT 10,
    alpha        REAL DEFAULT 0.5
) RETURNS TABLE(entity TEXT, score REAL, sparql_rank INTEGER, vector_rank INTEGER)
```

```sql
SELECT * FROM pg_ripple.hybrid_search(
    'PREFIX ex: <https://example.org/>
     SELECT ?person WHERE { ?person a ex:Person ; ex:knows ex:bob }',
    'researchers in knowledge graphs',
    10,
    0.7
);
```

---

### `rag_retrieve`

使用图+向量搜索为 RAG（检索增强生成）检索上下文。

```sql
pg_ripple.rag_retrieve(
    query TEXT,
    k     INTEGER DEFAULT 5,
    hops  INTEGER DEFAULT 2
) RETURNS TABLE(entity TEXT, context TEXT, score REAL)
```

```sql
SELECT * FROM pg_ripple.rag_retrieve('Who knows about knowledge graphs?', 5, 2);
```

---

## 管理

用于维护、统计和管理操作的函数。

---

### `compact`

通过删除未引用的 VP 表和字典条目来压缩三元组存储。

```sql
pg_ripple.compact() RETURNS JSON
```

```sql
SELECT pg_ripple.compact();
```

---

### `vacuum`

清理所有 VP 表以回收空间并更新统计信息。

```sql
pg_ripple.vacuum() RETURNS VOID
```

```sql
SELECT pg_ripple.vacuum();
```

---

### `reindex`

在 VP 表上重建所有 B 树和 BRIN 索引。

```sql
pg_ripple.reindex() RETURNS VOID
```

```sql
SELECT pg_ripple.reindex();
```

---

### `vacuum_dictionary`

清理字典表，删除任何 VP 表未引用的条目。

```sql
pg_ripple.vacuum_dictionary() RETURNS BIGINT
```

```sql
SELECT pg_ripple.vacuum_dictionary();
```

---

### `htap_migrate_predicate`

将谓词从平坦 VP 布局迁移到 HTAP delta/main 布局。

```sql
pg_ripple.htap_migrate_predicate(
    predicate TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.htap_migrate_predicate('<https://example.org/knows>');
```

---

### `stats`

返回整体三元组存储统计信息。

```sql
pg_ripple.stats() RETURNS JSON
```

```sql
SELECT pg_ripple.stats();
```

---

### `canary`

健康检查函数，如果扩展已加载且功能正常则返回 true。

```sql
pg_ripple.canary() RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.canary();
```

---

### `enable_live_statistics`

为 VP 表启用实时统计信息收集。

```sql
pg_ripple.enable_live_statistics() RETURNS VOID
```

```sql
SELECT pg_ripple.enable_live_statistics();
```

---

### `promote_rare_predicates`

如果谓词超过阈值，则将谓词从 `vp_rare` 提升到专用 VP 表。

```sql
pg_ripple.promote_rare_predicates() RETURNS INTEGER
```

```sql
SELECT pg_ripple.promote_rare_predicates();
```

---

### `deduplicate_predicate`

从特定谓词的 VP 表中删除重复的三元组。

```sql
pg_ripple.deduplicate_predicate(
    predicate TEXT
) RETURNS BIGINT
```

```sql
SELECT pg_ripple.deduplicate_predicate('<https://example.org/knows>');
```

---

### `deduplicate_all`

从所有 VP 表中删除重复的三元组。

```sql
pg_ripple.deduplicate_all() RETURNS BIGINT
```

```sql
SELECT pg_ripple.deduplicate_all();
```

---

### `delete_triple`

从默认图中删除特定的三元组。

```sql
pg_ripple.delete_triple(
    subject   TEXT,
    predicate TEXT,
    object    TEXT
) RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.delete_triple(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>'
);
```

---

### `delete_triple_from_graph`

从命名图中删除特定的三元组。

```sql
pg_ripple.delete_triple_from_graph(
    subject   TEXT,
    predicate TEXT,
    object    TEXT,
    graph     TEXT
) RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.delete_triple_from_graph(
    '<https://example.org/alice>',
    '<https://example.org/knows>',
    '<https://example.org/bob>',
    '<https://example.org/people>'
);
```

---

### `get_statement`

通过全局唯一的语句 ID (SID) 检索语句。

```sql
pg_ripple.get_statement(
    sid BIGINT
) RETURNS TABLE(subject TEXT, predicate TEXT, object TEXT, graph TEXT)
```

```sql
SELECT * FROM pg_ripple.get_statement(42);
```

---

## 安全性

用于行级安全性、访问控制和 schema 检查的函数。

---

### `enable_graph_rls`

在 VP 表上启用行级安全性，按命名图限制访问。

```sql
pg_ripple.enable_graph_rls() RETURNS VOID
```

```sql
SELECT pg_ripple.enable_graph_rls();
```

---

### `grant_graph`

授予用户对命名图的访问权限。

```sql
pg_ripple.grant_graph(
    username   TEXT,
    graph      TEXT,
    permission TEXT DEFAULT 'read'
) RETURNS VOID
```

```sql
SELECT pg_ripple.grant_graph('analyst', '<https://example.org/public>', 'read');
```

---

### `revoke_graph`

撤销用户对命名图的访问权限。

```sql
pg_ripple.revoke_graph(
    username TEXT,
    graph    TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.revoke_graph('analyst', '<https://example.org/public>');
```

---

### `list_graph_access`

列出所有图访问授权。

```sql
pg_ripple.list_graph_access() RETURNS TABLE(username TEXT, graph TEXT, permission TEXT)
```

```sql
SELECT * FROM pg_ripple.list_graph_access();
```

---

### `enable_schema_summary`

启用后台 schema 摘要生成（需要 pg_trickle）。

```sql
pg_ripple.enable_schema_summary() RETURNS VOID
```

```sql
SELECT pg_ripple.enable_schema_summary();
```

---

### `schema_summary`

返回所有谓词、类型和计数的单次 schema 摘要。

```sql
pg_ripple.schema_summary() RETURNS JSON
```

```sql
SELECT pg_ripple.schema_summary();
```

---

## CDC

用于变更数据捕获订阅的函数。

---

### `subscribe`

订阅三元组存储上的变更事件。返回订阅 ID。

```sql
pg_ripple.subscribe(
    channel  TEXT DEFAULT 'pg_ripple_changes',
    filter   TEXT DEFAULT NULL
) RETURNS TEXT
```

```sql
SELECT pg_ripple.subscribe('my_changes', 'predicate=<https://example.org/knows>');
```

---

### `unsubscribe`

取消订阅变更事件订阅。

```sql
pg_ripple.unsubscribe(
    subscription_id TEXT
) RETURNS VOID
```

```sql
SELECT pg_ripple.unsubscribe('sub_abc123');
```

---

## 索引

用于查询谓词索引的函数。

---

### `subject_predicates`

返回给定主语使用的所有谓词。

```sql
pg_ripple.subject_predicates(
    subject TEXT
) RETURNS TABLE(predicate TEXT)
```

```sql
SELECT * FROM pg_ripple.subject_predicates('<https://example.org/alice>');
```

---

### `object_predicates`

返回给定资源作为对象出现的所有谓词。

```sql
pg_ripple.object_predicates(
    object TEXT
) RETURNS TABLE(predicate TEXT)
```

```sql
SELECT * FROM pg_ripple.object_predicates('<https://example.org/alice>');
```

---

## 缓存

用于查询计划缓存管理的函数。

---

### `plan_cache_stats`

返回有关 SPARQL 到 SQL 计划缓存的统计信息。

```sql
pg_ripple.plan_cache_stats() RETURNS JSON
```

```sql
SELECT pg_ripple.plan_cache_stats();
```

---

### `plan_cache_reset`

清除 SPARQL 到 SQL 计划缓存。

```sql
pg_ripple.plan_cache_reset() RETURNS VOID
```

```sql
SELECT pg_ripple.plan_cache_reset();
```

---

### `relay_available`

检查中继集成是否已启用以及是否安装了 pg_tide 配套扩展。

```sql
pg_ripple.relay_available() RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.relay_available();
```

---

### `pg_tide_available`

检查是否安装了 pg_tide 配套扩展。当还必须启用旧版 `pg_ripple.trickle_integration` GUC 时，请使用 `relay_available()`。

```sql
pg_ripple.pg_tide_available() RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.pg_tide_available();
```

---

### `pg_trickle_available`

检查是否安装了 pg_trickle 配套扩展以支持 IVM 视图。

```sql
pg_ripple.pg_trickle_available() RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.pg_trickle_available();
```

---

### `trickle_available`

`relay_available()` 的已弃用中继兼容性别名。

```sql
pg_ripple.trickle_available() RETURNS BOOLEAN
```

```sql
SELECT pg_ripple.trickle_available();
```