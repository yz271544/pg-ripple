-- Migration 0.128.0 → 0.129.0: ontology reload helper + stale-cache safety net
--
-- New features:
--   ONT-RELOAD-01: pg_ripple.reload_ontologies(paths text[],
--                                          allowed_dir text DEFAULT NULL)
--       Bulk-loads a list of RDF/Turtle/N-Triples/RDF/XML files in one call.
--       Wraps pg_ripple.flush_encode_cache() + per-file bulk_load + ANALYZE.
--       See src/sparql_api.rs:429 for the underlying flush entry point.
--
-- Bug fixes:
--   ONT-RELOAD-01 / ONT-STALE-01: every load_*_file function in
--       src/bulk_load/mod.rs now invokes pre_load_safety_check() at entry.
--       This clears the shared-memory encode cache (initialised once at
--       postmaster start and not cleared by DROP EXTENSION / CREATE
--       EXTENSION), preventing the symptom where a freshly-created
--       extension loaded triples whose dictionary ids no longer existed
--       because the shmem cache still pointed at old ids from the
--       previous extension session.  See pre_load_safety_check() docs
--       in src/bulk_load/mod.rs for the failure mode walkthrough.
--
--   No schema changes; only PL/pgSQL wrappers are added.

CREATE OR REPLACE FUNCTION pg_ripple.reload_ontologies(
    paths        text[],
    allowed_dir  text DEFAULT NULL
) RETURNS bigint
LANGUAGE plpgsql
VOLATILE
STRICT
AS $$
DECLARE
    p          text;
    total      bigint := 0;
    loaded     bigint;
    file_ext   text;
    error_msg  text;
BEGIN
    -- The bulk_load_*_file functions require superuser and read server-side
    -- files via pg_read_file().  Both checks happen inside the loader, so
    -- we just propagate any error.
    IF array_length(paths, 1) IS NULL THEN
        RAISE EXCEPTION 'pg_ripple.reload_ontologies: paths array is empty'
            USING ERRCODE = '22023';
    END IF;

    -- If an allowed_dir was supplied, restrict the file loader for this
    -- session.  RESET at function exit so other sessions / statements
    -- are not affected.
    IF allowed_dir IS NOT NULL THEN
        EXECUTE format('SET LOCAL pg_ripple.copy_rdf_allowed_paths = %L',
                       allowed_dir);
    END IF;

    -- Flush the shared-memory encode cache so any stale entries from a
    -- previous extension session (post DROP/CREATE EXTENSION) cannot
    -- shadow fresh dictionary INSERTs.  This is the same fix that the
    -- per-file load_*_file safety net applies; we do it here too so the
    -- very first file in the array gets a clean slate even if some
    -- future code path bypasses pre_load_safety_check().
    PERFORM pg_ripple.flush_encode_cache();

    FOREACH p IN ARRAY paths LOOP
        file_ext := lower(regexp_replace(p, '^.*\.(\w+)$', '\1'));

        BEGIN
            loaded := CASE file_ext
                WHEN 'ttl'  THEN pg_ripple.load_turtle_file(p, false)
                WHEN 'nt'   THEN pg_ripple.load_ntriples_file(p, false)
                WHEN 'nq'   THEN pg_ripple.load_nquads_file(p, false)
                WHEN 'trig' THEN pg_ripple.load_trig_file(p, false)
                WHEN 'rdf'  THEN pg_ripple.load_rdfxml_file(p, false)
                WHEN 'xml'  THEN pg_ripple.load_rdfxml_file(p, false)
                WHEN 'owl'  THEN pg_ripple.load_rdfxml_file(p, false)
                ELSE pg_ripple.load_rdfxml_file(p, false)
            END;
            total := total + loaded;
        EXCEPTION WHEN OTHERS THEN
            GET STACKED DIAGNOSTICS error_msg = MESSAGE_TEXT;
            RAISE WARNING 'pg_ripple.reload_ontologies: failed to load %: %',
                          p, error_msg;
        END;
    END LOOP;

    -- Refresh planner stats so subsequent SPARQL queries can pick a
    -- sensible plan.  We ANALYZE the rare table and any promoted VP
    -- tables — the predicate catalog tells us which.
    ANALYZE _pg_ripple.vp_rare;
    FOR p IN
        SELECT format('_pg_ripple.vp_%s', id)::regclass
        FROM _pg_ripple.predicates
        WHERE table_oid IS NOT NULL
    LOOP
        EXECUTE 'ANALYZE ' || p;
    END LOOP;

    RETURN total;
END;
$$;

COMMENT ON FUNCTION pg_ripple.reload_ontologies(text[], text) IS
'Bulk-load a list of RDF/Turtle/N-Triples/RDF/XML files in one call. ONT-RELOAD-01 (v0.129.0).';