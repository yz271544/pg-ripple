# pg_ripple Glossary

## Ubiquitous Language

This is the shared vocabulary for pg_ripple. It is written for the people who design the extension, use it through SQL or HTTP, operate it in PostgreSQL, and explain it to someone seeing knowledge graphs for the first time. The same word should carry the same meaning in source code, tests, documentation, issue reports, and conversations. When a new feature introduces a new concept, add its name here before the name starts acquiring several meanings.

The examples use the RDF and SPARQL notation that appears in the project. An IRI is shown as `<https://example.org/alice>`, a variable as `?person`, and a literal as `"Oslo"`. Names such as `vp_rare`, `statement_id_seq`, and `pg_ripple_http` refer to concrete project objects and components, not general database ideas.

## RDF and graph language

### Knowledge graph

A knowledge graph is a collection of connected facts about things. In pg_ripple, those facts are RDF statements stored inside PostgreSQL, so relationships can be queried, validated, enriched with rules, and combined with ordinary database operations. The useful distinction is that a knowledge graph keeps the meaning of a relationship visible: `<alice> <worksFor> <acme>` tells us what connects Alice and Acme, rather than leaving that relationship implicit in a join table whose interpretation lives somewhere else.

### RDF dataset

An RDF dataset is one default graph together with any number of named graphs. pg_ripple uses graph ID `0` for the default graph and positive IDs for named graphs. A dataset is therefore the complete scope of RDF data being managed, while a graph is one context within that dataset. Use "dataset" when the default graph and named graphs are considered together; use "graph" when the context of a particular set of facts matters.

### Triple and quad

A triple is the basic RDF statement: subject, predicate, and object. `<alice> <knows> <bob>` is a triple that asserts one relationship. When the graph context is included, the statement is a quad: subject, predicate, object, and graph. pg_ripple stores the graph column even though people usually speak of triples, because the graph context controls isolation, provenance, temporal views, and query matching.

### RDF term

An RDF term is one of the three kinds of thing that can occupy a triple position: an IRI, a blank node, or a literal. "Term" is the precise word when the kind of value has not been narrowed down. In implementation discussions, say "RDF term" rather than simply "value" when the distinction affects encoding, SPARQL matching, or serialization.

### IRI

An IRI, or Internationalized Resource Identifier, names a resource in the graph. `<https://example.org/alice>` and `<urn:employee:42>` are IRIs; they are identifiers, not database locations that pg_ripple has to fetch. SPARQL and N-Triples write IRIs inside angle brackets, while prefixes make repeated vocabulary readable. An IRI can identify a person, a class, a predicate, a graph, or any other resource.

### Blank node

A blank node is an RDF node without a globally meaningful IRI. It is useful for describing a thing whose identity matters only inside the surrounding graph, such as an address structure or an anonymous restriction. A blank node label like `_:address1` is a serialization label, not a durable public identifier. pg_ripple encodes blank nodes as RDF terms just like IRIs and literals, but callers should not treat the generated internal ID as a stable business key.

### Literal

A literal is a value such as text, a number, a date, or a boolean. Literals may carry an RDF datatype, as in `"42"^^xsd:integer`, or a language tag, as in `"hello"@en`. The lexical form is part of the RDF term, so a typed number and an untyped string that happens to contain the same characters are not automatically the same thing. pg_ripple dictionary-encodes literals before storing them, then restores their RDF representation when results are exported.

### Predicate

The predicate is the relationship or property in the middle of a triple. In `<alice> <knows> <bob>`, `<knows>` is the predicate. Predicates are important both semantically and physically: the predicate says what a fact means, and pg_ripple uses predicate-oriented storage to make scans and joins efficient. A predicate normally receives a dedicated VP table when it is promoted; low-volume predicates may remain in the shared rare-predicate table.

### Default graph and named graph

The default graph is the unnamed graph in an RDF dataset, represented internally by graph ID `0`. A named graph is a graph identified by an IRI, with its own graph ID, that lets one dataset hold separate contexts such as sources, tenants, versions, or domains. A named graph is RDF context, not a PostgreSQL schema and not automatically a separate database. Use SPARQL `GRAPH` patterns or the graph-aware SQL APIs when the context must be selected explicitly.

### RDF-star

RDF-star allows a triple to appear as the subject or object of another triple. It lets a graph describe a statement, for example `<< <alice> <knows> <bob> >> <confidence> "0.9"`. This is the project language for statement-level metadata such as confidence, provenance, or timestamps. In storage terms, quoted-triple components are represented through the dictionary's `qt_s`, `qt_p`, and `qt_o` fields; callers should still think in terms of quoted RDF statements rather than those implementation columns.

## Storage language

### Dictionary encoding

Dictionary encoding maps each IRI, blank node, and literal to an internal `BIGINT` ID before the term reaches a VP table. The dictionary is the boundary between readable RDF terms and compact integer storage. Query translation encodes bound terms before generating SQL, which means joins compare integers instead of repeatedly comparing long strings. Decoding happens at the result boundary, during export, or when a diagnostic needs to show the original RDF term.

### Dictionary

The dictionary is the shared catalog of RDF terms and their internal IDs. It is not a cache and it is not a second graph: it is the lookup layer that gives the storage engine a compact representation while preserving the original term kind, lexical form, datatype, language tag, and RDF-star components. When debugging a query, a dictionary ID has meaning only through this catalog; never present the integer as if it were the IRI or literal itself.

### VP table

A VP table is a vertical-partitioning table organized around one predicate. Its principal columns are `s` for subject, `o` for object, `g` for graph, `i` for the statement identifier, and `source` for whether the row was asserted or inferred. The physical name is usually `_pg_ripple.vp_{predicate_id}`. The predicate catalog, rather than guessed string concatenation, is the authority for finding the table and its OID.

### Rare predicate

A rare predicate is a predicate whose triple count is below the configured promotion threshold. Instead of creating and maintaining a table for every small predicate, pg_ripple stores these rows in `_pg_ripple.vp_rare` with the predicate ID included as a column. When a predicate becomes common enough, the promotion path moves it into its own VP table. "Rare" describes a storage decision, not a statement about the importance of the predicate or the quality of its data.

### Predicate catalog

The predicate catalog records the known predicates, their IDs, the physical table OID when they have a dedicated table, and their counts. It is the control plane for predicate storage. Code that needs to address a VP table must consult this catalog and use identifier-safe quoting; the catalog is what keeps logical predicate names separate from physical PostgreSQL identifiers and protects dynamic SQL from treating data as SQL syntax.

### Statement identifier (SID)

A statement identifier, or SID, is the globally unique `BIGINT` assigned to a stored statement by the shared `statement_id_seq` sequence. The SID lives in the `i` column of VP storage and gives one row a stable handle within the database. CDC, provenance, temporal bookkeeping, and APIs such as `get_statement()` can use that handle without reconstructing identity from the subject, predicate, object, and graph columns.

### Asserted fact and derived fact

An asserted fact is data supplied by a caller or imported from a source. A derived fact is produced by applying Datalog, RDFS, OWL, or another inference process to facts already known. The distinction matters when data changes: an asserted fact is an input that a user can retract, while a derived fact is an output that must be withdrawn or re-derived when its supporting inputs change. In VP storage, `source = 0` means explicit or asserted and `source = 1` means inferred or derived.

### HTAP storage

HTAP means Hybrid Transactional/Analytical Processing. pg_ripple uses the term for the split storage path that lets fresh writes remain inexpensive while larger reads benefit from a consolidated layout. New rows land in a delta partition, main-resident rows can be hidden by tombstones, and readers combine the visible main rows with the delta rows. The design is expressed as `(main EXCEPT tombstones) UNION ALL delta`, so a query sees one logical predicate table even while physical maintenance is in progress.

### Delta, main, and tombstone

The delta contains recent writes in heap and B-tree storage. The main partition contains the consolidated body of a predicate, usually organized for analytical scans with BRIN indexes. A tombstone records that a main-resident row has been deleted or superseded before the next merge. These names describe roles in the HTAP lifecycle, not three different kinds of RDF fact; the logical result is assembled from all three according to visibility rules.

### Merge worker

The merge worker is a PostgreSQL background worker that folds delta rows and tombstone information into a fresh main representation. It is a maintenance process, not part of the meaning of a query and not a second inference engine. A successful merge should preserve the same logical facts while changing where they live, which is why tests and operational metrics treat merge correctness and merge throughput as separate concerns.

## Query language

### SPARQL

SPARQL is the W3C query and update language for RDF. pg_ripple accepts SPARQL text, parses it into an algebra, translates that algebra into SQL over the encoded store, executes it through PostgreSQL SPI, and decodes the result back into RDF terms or tabular output. SPARQL is the user-facing language; SQL is the execution target. That distinction keeps a query's graph semantics visible even though the engine ultimately runs inside PostgreSQL.

### Basic graph pattern (BGP)

A basic graph pattern is a set of triple patterns joined by shared variables. `?person <worksFor> ?company` and `?company <locatedIn> <Oslo>` form a small BGP whose shared variable connects the two facts. BGPs are the core units that the SPARQL planner turns into scans and joins. A star pattern is a particularly useful BGP shape in which several predicates share a subject, allowing the planner to avoid redundant work.

### SPARQL algebra

SPARQL algebra is the structured intermediate representation produced after parsing a query. It makes operations such as joins, filters, optional matches, unions, aggregates, property paths, and graph selection explicit. pg_ripple's planner operates on this representation before emitting SQL. When discussing a planner issue, name the algebra operation that is wrong or missing rather than saying only that "SPARQL is broken"; the algebra is the boundary where graph semantics become an executable plan.

### Property path

A property path is SPARQL syntax for traversing one or more relationships. Sequence, alternative, inverse, zero-or-more, one-or-more, and zero-or-one paths let a query express graph reachability without spelling out a fixed number of joins. pg_ripple compiles recursive paths to PostgreSQL recursive CTEs and uses the PostgreSQL 18 `CYCLE` clause for cycle detection. A path is a traversal expression, not a materialized predicate and not an instruction to permanently add every reachable pair to the graph.

### Federation and SERVICE

Federation means answering one SPARQL request with data from more than one endpoint. A `SERVICE` block identifies the remote SPARQL endpoint and the subquery that should run there; pg_ripple joins the returned bindings with local results. The local engine owns query coordination, timeouts, result decoding, caching, and circuit-breaker behavior, while the remote endpoint owns its own data and execution. "Federated" therefore means distributed query execution, not replicated storage.

### JSON-LD and JSON-LD frame

JSON-LD is a JSON serialization of RDF that uses `@context`, `@id`, and related keywords to make linked data comfortable for application code. A JSON-LD frame is a shape for arranging matching graph data into nested JSON, such as a person with embedded reports. In pg_ripple, framing is a result-shaping operation applied after the graph query has identified the data; it does not change the underlying triples or infer relationships that were not selected.

## Reasoning and data quality

### Datalog rule

A Datalog rule states how one fact can be derived from other facts. Its head is the conclusion and its body contains the conditions, for example `?x ex:indirectManager ?z :- ?x ex:manager ?y, ?y ex:manager ?z`. Rules operate over encoded predicate data, but their meaning is expressed in RDF terms and variables. A ruleset is a named collection of rules loaded for inference; calling `infer()` evaluates that ruleset and may store its conclusions as derived facts.

### Inference and entailment

Inference is the process of deriving facts that follow from asserted facts and rules. Entailment is the semantic relationship behind that process: a graph entails a statement when the rules and vocabulary say the statement must hold. In everyday project language, "inference" describes the work the engine performs and "entailment" describes why the result is justified. An inferred row is still queryable RDF data, but it should remain distinguishable from the source facts that support it.

### Materialization

Materialization computes derived facts and stores them in the VP store so later queries can read them directly. It trades work during inference for faster repeated reads and makes it possible to maintain derivation provenance. Full materialization can be wasteful when only a small part of a ruleset is relevant to a question, so pg_ripple also provides demand-filtered and goal-directed evaluation. Materialization is about persisting conclusions, not about converting RDF into a different data model.

### Semi-naive evaluation

Semi-naive evaluation is the standard way pg_ripple avoids repeating the same Datalog work at every fixpoint iteration. Each round tracks the newly derived delta and uses it in joins with the accumulated relation, rather than evaluating every rule against every known tuple again. The resulting facts are equivalent to full re-evaluation, but the engine spends its effort on consequences that are genuinely new. "Delta" in this reasoning context means newly derived tuples; it is separate from the HTAP delta partition unless the surrounding sentence says otherwise.

### Stratum and stratification

A stratum is one level in the evaluation order of a Datalog program. Stratification arranges rules so that a rule sees the completed results of the strata it depends on, which gives negation and aggregation a well-defined order. A positive recursive rule can stay within a stratum, while a negative dependency generally moves the dependent rule to a later one. When a program contains a cycle through negation that cannot be ordered this way, use well-founded semantics instead of calling the program merely "slow" or "invalid."

### Magic sets and demand filtering

Magic sets are a rewriting technique that turns a query's bindings into additional goals for a Datalog program. Demand filtering applies the same general idea: evaluate only the parts of inference that can contribute to the requested subject, predicate, object, or goal. These modes are goal-directed alternatives to computing every possible conclusion. They change the amount of work, not the intended meaning of a sound ruleset.

### Well-founded semantics (WFS)

Well-founded semantics gives Datalog programs with difficult negation a three-valued interpretation: true, false, or undefined. The undefined state is important because mutually dependent negative claims cannot always be settled by ordinary two-valued reasoning. pg_ripple exposes WFS through `infer_wfs()` for programs that cannot be cleanly stratified. Do not describe an undefined result as either a confirmed fact or a confirmed absence.

### Tabling

Tabling memoizes sub-goals and their results during recursive evaluation. If several rules ask the same sub-question, the engine can reuse the table instead of walking the same graph repeatedly. Tabling is an evaluation strategy, not a new kind of RDF storage and not the same thing as materializing the entire ruleset. Its value is greatest when recursion revisits the same sub-goals or when left-recursive rule structures would otherwise repeat work.

### Delete-Rederive (DRed)

Delete-Rederive is the maintenance strategy used when a source change may invalidate inferred facts. The engine first removes conclusions that may depend on the deleted input, then re-derives the remaining consequences from the facts still present. This is more precise than dropping every inferred fact and recomputing the whole graph, while remaining conservative about dependencies that need to be checked again. DRed concerns correctness after changes; it is not a PostgreSQL `DELETE` command with a special name.

### SHACL and shape

SHACL, the Shapes Constraint Language, describes what valid RDF data must look like. A shape can require a property, limit its count, constrain a datatype, compare values, or describe a path through related nodes. A shape is a constraint definition, not an inference rule: it reports that data satisfies or violates an expectation rather than asserting new business facts. pg_ripple supports synchronous checks, asynchronous validation, and SHACL-SPARQL features for constraints that need a query.

### Validation report

A validation report records the result of applying shapes to a graph. It identifies the focus node, the constraint that was checked, and the value or path involved in a violation, with decoded RDF terms where that makes the report useful to a human. A report is evidence about data quality at a point in time. It should not be confused with an error log, and a successful report does not mean that the graph is complete or that every possible domain rule has been tested.

## Operations and integrations

### Change Data Capture (CDC)

Change Data Capture is the stream of graph changes that other PostgreSQL sessions or services can consume. pg_ripple can publish matching inserts and deletes through PostgreSQL notifications and expose query subscriptions over Server-Sent Events through `pg_ripple_http`. CDC describes the event flow, not the storage layout that produced it. A consumer should use the statement ID, graph context, and event metadata when it needs to distinguish two similar triples or reconstruct provenance.

### GUC

A GUC is a PostgreSQL Grand Unified Configuration parameter. pg_ripple uses names such as `pg_ripple.dictionary_cache_size`, `pg_ripple.sparql_max_algebra_depth`, and `pg_ripple.vp_promotion_threshold` for settings that control resource limits, caches, storage policy, and optional features. GUCs are configuration inputs, not hidden arguments to a particular SQL function. When a behavior depends on one, document the setting name and the scope at which PostgreSQL applies it.

### RAG and embedding

Retrieval-Augmented Generation, or RAG, retrieves context before a language model generates an answer. An embedding is a numeric vector that represents the semantic content of text or an entity, making approximate similarity search possible. pg_ripple combines vector similarity with graph patterns so retrieval can ask both "what is semantically close?" and "how is it connected?" The vector score is evidence for ranking, while the RDF graph supplies explicit relationships and provenance that a similarity score cannot provide on its own.

### GraphRAG

GraphRAG is the use of a knowledge graph as part of a retrieval and generation workflow. In this project, the term also refers to the Microsoft GraphRAG-compatible export path for entities, relationships, and text units. A GraphRAG export is an interoperability format, not a different graph engine and not a synonym for every application that happens to use vectors. Keep the distinction clear when discussing whether a feature changes pg_ripple's store or only prepares data for an external pipeline.

### Citus and shard pruning

Citus distributes PostgreSQL tables across worker nodes. With pg_ripple sharding enabled, VP data can be distributed and a query with a sufficiently bound subject can often be routed to the shard that owns it. Shard pruning means eliminating workers that cannot contribute to the requested result; it is a planning optimization, not a correctness filter. A query that cannot provide the routing key may still be correct while needing to visit more shards.

### pg_ripple_http

`pg_ripple_http` is the companion service that exposes pg_ripple capabilities over HTTP. It handles protocol concerns such as authentication, content negotiation, streaming responses, metrics, and the REST surface for features that are awkward to call directly from a remote client. It is versioned independently from the PostgreSQL extension and checks compatibility with the installed extension at startup. The service is an access layer over pg_ripple; it does not replace PostgreSQL as the system of record.

## Naming habits

Use "fact" for an RDF statement when its role as knowledge matters, "row" when discussing the physical PostgreSQL representation, and "binding" when discussing a variable assigned during SPARQL evaluation. Use "term" for an IRI, blank node, or literal, and "ID" for its dictionary-encoded integer. Say "graph context" when the named-graph distinction matters, and say "dataset" when the default and named graphs are considered together.

Use "explicit" or "asserted" for input facts and "inferred" or "derived" for conclusions produced by rules. Use "validation" for checking data against SHACL constraints and "inference" for deriving data from rules. Use "merge" for moving HTAP data between physical partitions, and "materialization" for storing logical conclusions from reasoning. Those distinctions keep a storage operation, a quality check, and a semantic computation from collapsing into the same vague word: "processing."
