# Adapted BSBM Workload

These scripts use the Berlin SPARQL Benchmark vocabulary and query shapes through pg_ripple's SQL API. The generated fixture and queries are repository adaptations, not the official BSBM driver or a comparable BSBM score.

## Required CI gate

CI runs the loader at scale 59, asserts that it produced between 900,000 and 1,100,000 triples, and executes all 12 queries with `ON_ERROR_STOP=1`. It retains raw load and query logs plus the total query-mix elapsed time.

The gate proves that the adapted million-triple load and query paths execute. It does not enforce a latency threshold.

## Run locally

Use an empty disposable database with pg_ripple installed:

```bash
psql -X -v ON_ERROR_STOP=1 -v scale=1 \
  -f benchmarks/bsbm/bsbm_load.sql
psql -X -v ON_ERROR_STOP=1 \
  -f benchmarks/bsbm/bsbm_queries.sql
```

Scale 1 creates about 1,000 products. Use scale 59 to reproduce the CI fixture size.

The optional `bsbm_htap.sql` and `bsbm_pgbench.sql` files support local concurrency experiments. Current CI does not run them, and the repository publishes no current throughput or latency baseline for them.
