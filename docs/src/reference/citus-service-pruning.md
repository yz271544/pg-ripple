# Citus SERVICE Shard Pruning

When pg_ripple is deployed on a [Citus](https://www.citusdata.com/) distributed PostgreSQL
cluster, SPARQL `SERVICE` queries that target Citus worker shards can benefit from shard
annotation pruning. This reduces the number of shards queried when the subject IRI can be
used to identify the target shard.

## What is SERVICE shard annotation?

In a SPARQL SERVICE query such as:

```sparql
SELECT ?name WHERE {
    SERVICE <https://worker-node.example/sparql> {
        <https://example.org/Alice> schema:name ?name
    }
}
```

When the subject IRI (`<https://example.org/Alice>`) is bound, pg_ripple can use the Citus
shard key to compute which shards contain triples for that subject. With pruning enabled,
only the relevant shards are queried, bypassing the rest.

## How shard annotation works

1. At query planning time, pg_ripple calls `citus_service_shard_annotation(endpoint_url)` for
   each `SERVICE` clause.
2. If the endpoint is identified as a Citus worker (`is_citus_worker_endpoint()` returns true),
   and a bound subject IRI is present, the shard routing key is computed.
3. The generated SQL appends `WHERE dist_key = $shard_key` to prune the shard fan-out.

## Configuration

| GUC | Default | Description |
|-----|---------|-------------|
| `pg_ripple.citus_service_pruning` | `off` | Enable SERVICE shard annotation for Citus workers |

```sql
-- Enable for the session
SET pg_ripple.citus_service_pruning = on;

-- Or set globally (recommended for Citus deployments)
ALTER SYSTEM SET pg_ripple.citus_service_pruning = on;
SELECT pg_reload_conf();
```

## Verification

The repository does not retain raw evidence for a current multi-node latency comparison. Use `EXPLAIN` on the target cluster to verify the routing decision, then retain the plan, dataset identity, topology, and timings.

To reproduce with `EXPLAIN`:

```sql
-- Without pruning
SET pg_ripple.citus_service_pruning = off;
SELECT pg_ripple.explain_sparql(
    'SELECT ?name WHERE {
        SERVICE <http://citus-worker-1:5432/sparql> {
            <https://example.org/Alice> <https://schema.org/name> ?name
        }
    }',
    false
);

-- With pruning
SET pg_ripple.citus_service_pruning = on;
SELECT pg_ripple.explain_sparql(
    'SELECT ?name WHERE {
        SERVICE <http://citus-worker-1:5432/sparql> {
            <https://example.org/Alice> <https://schema.org/name> ?name
        }
    }',
    false
);
```

On a correctly configured cluster, compare the shard count in the two plans. Do not assume a specific latency improvement without measuring the target topology.

## Status

`citus_service_pruning` is experimental. Single-node CI tests verify the GUC and explain plumbing without Citus. They do not qualify multi-node routing, correctness, or performance.

See also: [Approximate Aggregates (HLL)](approximate-aggregates.md), [Citus Integration](../operations/citus-integration.md),
[Compatibility Matrix](../operations/compatibility.md).
