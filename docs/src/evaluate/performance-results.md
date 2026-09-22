# Performance and Conformance Results

This page distinguishes current fail-closed evidence from historical or informational checks.

## Current conformance evidence

| Suite | Current result | CI role |
|---|---|---|
| W3C SPARQL 1.1 | 324 / 324 locked cases pass | Required |
| LUBM | 14 / 14 queries pass | Required |
| Apache Jena | Reported by its suite | Informational |
| WatDiv | 32 templates execute on an empty database | Execution check; no scale or latency claim |
| OWL 2 RL | Pinned 489-case corpus downloads and validates | Source check only; no current conformance result |

Feature matrices describe implemented behavior. They do not expand the scope of these test results.

## Current performance evidence

The manual `Benchmark` workflow runs a bounded `synthetic-100k-v1` workload. It retains:

- exact pg_ripple, PostgreSQL, and Git versions;
- runner and operating-system details;
- raw command output;
- JSON containing insert throughput and average point-query and SPARQL BGP latency.

The required BSBM job separately loads about one million triples from the repository's adapted fixture and fails on load, scale, or query errors. It records elapsed time but has no latency regression threshold.

The checked-in v0.136.0 run at commit `993af4fef88e` measured 5,666 inserted triples/s, 0.819 ms average point-query latency, and 0.277 ms average single-pattern SPARQL latency on an Apple M1 Pro. The [JSON, raw output, and sanitized environment](https://github.com/trickle-labs/pg-ripple/tree/main/benchmarks/results) are retained together. This is one bounded run, not a capacity or cross-system comparison.

Historical merge and PageRank CSV files have incompatible schemas and no retained raw evidence for current releases. They are not current baselines.

## Not yet qualified by current evidence

- production-scale or multi-node Citus performance;
- WatDiv at 10 million or 100 million triples;
- sustained HTTP, queue, or mixed read/write load;
- current merge-worker, PageRank, vector, or hybrid-search comparisons;
- long-duration soak, failover, restore, or PITR performance.

Do not use repository data to make capacity claims for these paths. Measure the pinned build with the target dataset, hardware, and workload.

## Run the bounded benchmark

Use a new empty database. The script refuses to run when triples already exist.

```bash
createdb pg_ripple_benchmark
psql -d pg_ripple_benchmark -c 'CREATE EXTENSION pg_ripple CASCADE'
PGDATABASE=pg_ripple_benchmark \
  BENCH_RUNNER=local \
  RESULT_FILE=benchmark_results.json \
  bash benchmarks/ci_benchmark.sh
```

See [`benchmarks/`](https://github.com/trickle-labs/pg-ripple/tree/main/benchmarks) for the harness and evidence requirements.

## See also

- [SPARQL compliance matrix](../reference/sparql-compliance.md)
- [WatDiv status](../reference/watdiv-results.md)
- [LUBM results](../reference/lubm-results.md)
- [OWL 2 RL source status](../reference/owl2rl-results.md)
