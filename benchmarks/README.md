# Benchmarks

This directory contains benchmark harnesses. Only results that identify the exact commit, fixture, software versions, runner, and raw output should be used as current evidence.

## Current automated producer

The manual `Benchmark` GitHub Actions workflow runs `ci_benchmark.sh`. It measures one bounded synthetic workload:

- load 100,000 triples in 5,000-triple batches;
- run 100 warmed point lookups;
- run 50 warmed single-pattern SPARQL queries;
- write raw output, environment details, and `benchmark_results.json`;
- retain the artifact for 90 days.

The script requires PostgreSQL 18 with pg_ripple already installed in an empty database. It refuses a non-empty database and does not drop user data.

```bash
cargo pgrx start pg18
createdb pg_ripple_benchmark
psql -d pg_ripple_benchmark -c 'CREATE EXTENSION pg_ripple CASCADE'
PGDATABASE=pg_ripple_benchmark \
  BENCH_RUNNER=local \
  RESULT_FILE=benchmark_results.json \
  bash benchmarks/ci_benchmark.sh
```

The JSON contains the extension and PostgreSQL versions, commit SHA, timestamp, runner, fixture ID, insert throughput, and average point and SPARQL latency. Set `BASELINE_FILE` only when the baseline used the same fixture and comparable hardware.

## BSBM execution gate

Required CI loads the repository's adapted BSBM fixture at scale 59, verifies a triple count between 900,000 and 1,100,000, and runs the checked-in query mix with `ON_ERROR_STOP=1`. It retains the load log, query log, and elapsed time.

This is an execution and scale gate. It is not an official BSBM score and does not enforce a latency regression threshold.

## Other harnesses

The SQL and shell files in this directory support targeted experiments for merge workers, PageRank, Datalog, vector search, probabilistic reasoning, federation, and other subsystems. They are not all run by current CI.

`merge_throughput_history.csv`, `pagerank_throughput_history.csv`, and `merge_throughput_baselines.json` are historical, unverified records. Their schemas differ and no retained raw artifacts tie them to the current architecture. Do not use them as v0.136.0 capacity or regression baselines.

Record new results under `benchmarks/results/` only with sanitized environment details and raw output.

## Checked-in result

The first result using this evidence format is for pg_ripple 0.136.0 at commit
`993af4fef88e` on an Apple M1 Pro:

| Fixture | Insert throughput | Point query | SPARQL BGP |
|---|---:|---:|---:|
| `synthetic-100k-v1` | 5,666 triples/s | 0.819 ms | 0.277 ms |

See the [JSON result](results/v0.136.0-993af4fe-m1-pro.json),
[raw output](results/v0.136.0-993af4fe-m1-pro.txt), and
[sanitized environment](results/v0.136.0-993af4fe-m1-pro-environment.txt).
This is one bounded run, not a capacity or cross-system comparison.
