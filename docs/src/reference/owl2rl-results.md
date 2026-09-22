# OWL 2 RL Source Status

pg_ripple implements OWL 2 RL rules and has regression coverage for individual inference behaviors. The current CI job does not execute the W3C OWL 2 RL conformance corpus.

## Current v0.136.0 evidence

CI downloads the pinned W3C `all.rdf` source, verifies its SHA-256 checksum from `tests/conformance/sources.lock`, parses the XML, and checks that it contains 489 OWL test identifiers. The corpus contains 91 references to the RL profile.

This is a fail-closed source acquisition check. It proves that the intended corpus is available and intact. It does not produce a pass rate.

```sh
bash scripts/fetch_conformance_tests.sh --owl2rl
```

The historical harness expects per-case fixtures and cannot execute the monolithic `all.rdf` source without an adapter. Until that adapter exists, the project makes no current-version OWL 2 RL conformance percentage claim.

## Historical results

Older results used different fixture sets and are not comparable with the current source check:

| Version | Passing / Total | Status |
|---|---:|---|
| v0.47.0 | 62 / 66 | Historical harness |
| v0.119.0 | 66 / 66 | Historical harness |
| v0.136.0 | Not run | Source acquisition only |

The PostgreSQL regression suite remains the required evidence for implemented OWL rule behavior, including `owl:propertyChainAxiom`.
