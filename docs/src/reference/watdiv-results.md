# WatDiv Query Execution Status

[WatDiv](https://dsg.uwaterloo.ca/watdiv/) defines query templates that stress star, chain, snowflake, and complex graph patterns.

## Current v0.136.0 CI coverage

The current job executes 32 checked-in templates:

| Class | Templates |
|---|---:|
| Star | 7 |
| Chain | 3 |
| Snowflake | 5 |
| Complex | 17 |

CI creates a clean extension in an empty database and fails on query execution errors. It does not load a WatDiv dataset. The checked-in latency baselines are null, so the job does not establish correctness at 10-million-triple scale or enforce a performance threshold.

```sh
cargo test --test watdiv_suite -- --nocapture
```

## What a scale qualification run must retain

A reproducible WatDiv result must identify the generator revision, dataset size and checksum, concrete substitutions, PostgreSQL and pg_ripple versions, hardware, raw result counts, and raw timings. No such current-version result is checked in yet.

## See also

- [Running Conformance Tests](running-conformance-tests.md)
- [Performance Results](../evaluate/performance-results.md)
