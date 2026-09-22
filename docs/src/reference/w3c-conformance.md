# W3C Conformance

This page summarizes pg_ripple's current standards evidence. Suite names do not imply that every upstream test or deployment mode is covered.

As of v0.136.0, the locked 324-case W3C SPARQL suite and the 14-query LUBM suite are required CI gates. Jena and WatDiv are additional signals. OWL 2 RL CI currently verifies only that the pinned upstream corpus can be downloaded and checksum-validated.

## Test suites

pg_ripple runs these complementary qualification jobs:

| Suite | Tests | What it validates |
|---|---|---|
| **W3C SPARQL 1.1** | 324 locked cases | Required query and update conformance gate |
| **Apache Jena** | ~1 000 | Implementation edge cases (type coercion, date-time, blank-node scoping) |
| **WatDiv** | 32 checked-in templates | Query execution against an empty database; no current scale or latency claim |
| **LUBM** | 14 queries | OWL RL inference correctness under ontological reasoning (v0.44.0+) |
| **OWL 2 RL** | 489 source cases | Required corpus acquisition and checksum check; cases are not executed by the current harness |

The execution suites write reports where supported. The OWL source job uploads the validated `all.rdf` corpus instead of a conformance report.

See [Running Conformance Tests](running-conformance-tests.md) for local setup instructions, the [WatDiv Results](watdiv-results.md) page for performance metrics, and the [LUBM Results](lubm-results.md) page for OWL RL conformance details.

---

## W3C SPARQL 1.1 test harness (v0.41.0+)

The test harness (`tests/w3c/`) runs a locked set of 324 cases derived from the official [W3C SPARQL 1.1 test suite](https://www.w3.org/2009/sparql/docs/tests/) against a live pg_ripple installation. It runs serially because the cases share one PostgreSQL database. The checked v0.136.0 report has 324 passes and no failures, skips, or expected failures.

### Per-category coverage

| Sub-suite | Tests | CI status |
|---|---|---|
| aggregates | 47 | Required |
| basic-update | 13 | Required |
| bind | 10 | Required |
| exists | 6 | Required |
| functions | 75 | Required |
| grouping | 6 | Required |
| negation | 12 | Required |
| project-expression | 7 | Required |
| property-path | 33 | Required |
| service | 7 | Required |
| subquery | 14 | Required |
| syntax-query | 94 | Required |

### Running locally

```sh
# Download test data first (one-time setup):
bash scripts/fetch_conformance_tests.sh --w3c

# Run the separate curated smoke subset:
cargo test --test w3c_smoke

# Run the locked 324-case suite serially:
cargo test --test w3c_suite -- --nocapture
```

---

## Apache Jena test suite (v0.43.0+)

The Jena adapter (`tests/jena/`) runs ~1 000 tests from Apache Jena's `sparql-query`, `sparql-update`, `sparql-syntax`, and `algebra` sub-suites.  Jena tests cover implementation edge cases that the W3C suite leaves underspecified.

### Jena-specific coverage areas

| Area | Tests |
|---|---|
| XSD numeric promotions (`xsd:integer` → `xsd:decimal` → `xsd:double`) | sparql-query |
| Mixed-type arithmetic and comparisons | sparql-query |
| Timezone-aware `xsd:dateTime` comparisons | sparql-query |
| Date/time built-ins: `NOW()`, `YEAR()`, `MONTH()`, `DAY()`, `HOURS()`, `MINUTES()`, `SECONDS()`, `TZ()` | sparql-query |
| `xsd:decimal` arithmetic: `ROUND()`, `CEIL()`, `FLOOR()`, `ABS()` | sparql-query |
| Blank nodes in CONSTRUCT templates | sparql-query |
| Blank-node identity across OPTIONAL and GRAPH boundaries | sparql-query |
| String functions: `STRLEN()`, `SUBSTR()`, `UCASE()`, `LCASE()`, `STRSTARTS()`, `STRENDS()`, `CONTAINS()`, `ENCODE_FOR_URI()`, `CONCAT()` | sparql-query |
| SPARQL UPDATE edge cases | sparql-update |
| Syntax acceptance / rejection (positive/negative syntax tests) | sparql-syntax |
| Algebra normalisation equivalences | algebra |

### CI status

The `jena-suite` CI job is **non-blocking** until pass rate ≥ 95%, then promoted to required.  Known failures for type-coercion and date-time edge cases are tracked in `tests/conformance/known_failures.txt` with the `jena:` prefix.

### Running locally

```sh
# Download Jena test data:
bash scripts/fetch_conformance_tests.sh --jena

# Run the full Jena suite:
cargo test --test jena_suite
```

---

## SPARQL 1.1 Query

**Test suite**: [W3C SPARQL 1.1 Query test suite (2013-03-27)](https://www.w3.org/2009/sparql/test-suite-20130327/)

**Gate**: all 324 locked cases must pass.

### Supported features

| Feature | Status |
|---|---|
| Basic Graph Patterns (BGP) | Supported |
| FILTER with all comparison and logical operators | Supported |
| OPTIONAL | Supported |
| UNION | Supported |
| Subqueries (`SELECT … { SELECT … }`) | Supported |
| BIND | Supported |
| VALUES | Supported |
| Property paths (`/`, `|`, `*`, `+`, `?`, `^`) | Supported |
| Negated property sets (`!(p1|p2)`) | Supported |
| Aggregates: COUNT, SUM, AVG, MIN, MAX | Supported |
| GROUP BY, HAVING | Supported |
| ORDER BY, LIMIT, OFFSET | Supported |
| DISTINCT | Supported |
| ASK | Supported |
| CONSTRUCT | Supported |
| DESCRIBE | Supported |
| Named graphs (`GRAPH ?g { … }`) | Supported |
| Federated query (`SERVICE`) | Supported (v0.16.0) |
| All XPath/SPARQL built-in functions (STR, STRLEN, UCASE, LCASE, STRSTARTS, STRENDS, CONTAINS, REGEX, ABS, CEIL, FLOOR, ROUND, IF, COALESCE, isIRI, isLiteral, isBlank, DATATYPE, LANG, BIND) | Supported |
| Language-tagged literals (storage and LANG() function) | Supported |
| Typed literals with xsd:integer, xsd:decimal, xsd:double, xsd:dateTime, xsd:boolean | Supported |
| NOT EXISTS | Supported |
| MINUS | Supported |
| RDF-star (quoted triples, SPARQL-star BGP) | Supported (v0.4.0) |

### Known limitations

| Feature | Status |
|---|---|
| `langMatches()` function | Not supported. Returns 0 rows without error. Full BCP 47 language tag matching is planned for a future release. |
| Custom aggregate extensions (property functions) | Not supported. Standard aggregates (COUNT, SUM, AVG, MIN, MAX) are fully supported. |
| Variable-inside-quoted-triple patterns (`<< ?s ?p ?o >>`) | Returns 0 rows with a WARNING. Ground quoted-triple patterns work. |
| `LOAD <url>` from arbitrary HTTP URIs | Network-access dependent; supported via `pg_ripple_http` companion service. |

---

## SPARQL 1.1 Update

**Test suite**: [W3C SPARQL 1.1 Update test suite (2013)](https://www.w3.org/2013/sparql-update-tests/)

**Target**: ≥ 95% of applicable tests pass.

### Supported features

| Feature | Status |
|---|---|
| INSERT DATA | Supported |
| DELETE DATA | Supported |
| INSERT WHERE | Supported |
| DELETE WHERE | Supported |
| DELETE/INSERT WHERE | Supported |
| CLEAR GRAPH | Supported |
| CREATE GRAPH / DROP GRAPH | Supported |
| Multi-statement updates (`;` separator) | Supported |
| Named graph update operations | Supported |
| Idempotent re-insert (ON CONFLICT DO NOTHING) | Supported |

### Known limitations

| Feature | Status |
|---|---|
| `COPY`, `MOVE`, `ADD` graph operations | Supported. |
| `LOAD <url>` | Same as for queries above. |

---

## SHACL Core

**Test suite**: [W3C SHACL Core test suite](https://w3c.github.io/shacl/tests/)

**Target**: ≥ 95% of SHACL Core tests pass.

### Supported constraints

| Constraint | Status |
|---|---|
| `sh:targetClass` | Supported |
| `sh:targetNode` | Supported |
| `sh:targetSubjectsOf` | Supported |
| `sh:targetObjectsOf` | Supported |
| `sh:property` with `sh:path` | Supported |
| `sh:minCount` / `sh:maxCount` | Supported |
| `sh:datatype` | Supported |
| `sh:pattern` (regex) | Supported |
| `sh:minLength` / `sh:maxLength` | Supported |
| `sh:minInclusive` / `sh:maxInclusive` | Supported |
| `sh:minExclusive` / `sh:maxExclusive` | Supported |
| `sh:in` (enumeration) | Supported |
| `sh:hasValue` | Supported |
| `sh:class` | Supported |
| `sh:nodeKind` (IRI, BlankNode, Literal) | Supported |
| `sh:or` | Supported |
| `sh:and` | Supported |
| `sh:not` | Supported |
| `sh:node` (nested shape reference) | Supported |
| `sh:qualifiedValueShape` + `sh:qualifiedMinCount` / `sh:qualifiedMaxCount` | Supported |
| Async validation pipeline (`process_validation_queue`) | Supported |
| Sync mode (insert rejection) | Supported |

### Known limitations

| Feature | Status |
|---|---|
| SHACL Advanced Features (SPARQL-based constraints, `sh:SPARQLConstraint`) | Deferred to v0.21.0. |
| SHACL-AF (rules, `sh:TripleRule`) | Partial implementation via Datalog; full SHACL-AF integration planned. |

---

## Running the conformance gate

The conformance tests run as part of the standard pg_regress suite:

```bash
cargo pgrx regress pg18 --postgresql-conf "allow_system_table_mods=on"
```

The relevant test files are:

- `tests/pg_regress/sql/w3c_sparql_query_conformance.sql`
- `tests/pg_regress/sql/w3c_sparql_update_conformance.sql`
- `tests/pg_regress/sql/w3c_shacl_conformance.sql`
- `tests/pg_regress/sql/crash_recovery_merge.sql`
