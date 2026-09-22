# Running Conformance Tests

pg_ripple has required execution gates, informational execution suites, and source-only checks. Run each suite against PostgreSQL 18 with the current extension installed.

## Fixture acquisition

The acquisition script pins and validates external corpora:

```sh
bash scripts/fetch_conformance_tests.sh --w3c
bash scripts/fetch_conformance_tests.sh --jena
bash scripts/fetch_conformance_tests.sh --watdiv
bash scripts/fetch_conformance_tests.sh --owl2rl
```

The OWL command downloads and checksum-validates the W3C `all.rdf` corpus. The current OWL harness does not execute that monolithic corpus, so this command is a source check, not an OWL conformance run.

## W3C SPARQL 1.1

The required suite contains 324 locked cases and runs serially because its cases share one database.

```sh
cargo pgrx start pg18
cargo test --test w3c_smoke -- --nocapture
cargo test --test w3c_suite -- --nocapture
```

Required CI sets `REQUIRE_CONFORMANCE=1`. A missing fixture, an empty run, a skip, an expected failure, or an unexpected failure then fails the job.

## Apache Jena

The Jena suite is informational. It covers additional SPARQL query, update, syntax, and algebra cases.

```sh
bash scripts/fetch_conformance_tests.sh --jena
cargo test --test jena_suite -- --nocapture
```

## WatDiv query execution

The current CI job executes 32 checked-in WatDiv templates against an empty database. Query errors fail the job. It does not load a 10-million-triple dataset, compare result counts with populated baselines, or enforce latency thresholds.

```sh
cargo test --test watdiv_suite -- --nocapture
```

Use a generated WatDiv dataset only for a separate, explicitly recorded local qualification run.

## LUBM

The required LUBM suite is self-contained. It runs 14 canonical queries against the bundled fixture and checks OWL RL inference behavior.

```sh
cargo test --test lubm_suite -- --nocapture
```

## Known failures and reports

Expected failures live in `tests/conformance/known_failures.txt`. An unexpected pass should remove the corresponding entry.

Execution suites write JSON reports where supported. CI artifacts are the authoritative result for a specific commit; `tests/conformance/history.csv` is historical data and is not appended by current CI.

## See also

- [W3C Conformance](w3c-conformance.md)
- [LUBM Results](lubm-results.md)
- [WatDiv Results](watdiv-results.md)
- [OWL 2 RL Source Status](owl2rl-results.md)
