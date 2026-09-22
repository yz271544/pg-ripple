# Conformance Trends

The repository contains historical rows in [`tests/conformance/history.csv`](https://github.com/trickle-labs/pg-ripple/blob/main/tests/conformance/history.csv). Current CI does not append to that file, so it must not be treated as a live release dashboard.

## Current qualification policy

| Suite | Current CI role | Evidence |
|---|---|---|
| W3C SPARQL 1.1 | Required | All 324 locked cases must pass |
| Apache Jena | Informational | Result artifact when the suite runs |
| WatDiv | Execution gate | 32 checked-in templates on an empty database; no scale or latency claim |
| LUBM | Required | All 14 OWL RL queries must pass |
| OWL 2 RL | Required source check only | Pinned corpus download and checksum; no current conformance result |

Use the artifact from the exact commit under evaluation. Do not compare percentages across harness revisions unless the corpus, fixture, runner, and pass policy are identical.

## Historical data

`history.csv` remains useful as provenance for older runs. It is not a current benchmark producer and does not prove the status of v0.136.0.

To record a new row manually, run the relevant suite first and retain its raw report alongside the row.
