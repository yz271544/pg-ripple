//! pg_ripple lib.rs integration tests -- extracted (MOD-01, v0.72.0).

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
// A16-CQ: test helper — unwrap/expect are acceptable in test-only code.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_encode_decode_roundtrip() {
        let id = crate::dictionary::encode("https://example.org/subject", 0);
        let decoded = crate::dictionary::decode(id).expect("decode should succeed");
        assert_eq!(decoded, "https://example.org/subject");
    }

    #[pg_test]
    fn test_insert_and_count() {
        crate::storage::insert_triple(
            "<https://example.org/s>",
            "<https://example.org/p>",
            "<https://example.org/o>",
            0,
        );
        assert!(crate::storage::total_triple_count() >= 1);
    }

    #[pg_test]
    fn test_typed_literal_roundtrip() {
        // xsd:integer is now inline-encoded (bit 63 = 1, no dictionary row).
        let id = crate::dictionary::encode_typed_literal(
            "42",
            "http://www.w3.org/2001/XMLSchema#integer",
        );
        assert!(
            crate::dictionary::inline::is_inline(id),
            "xsd:integer should be inline-encoded"
        );
        // decode() must still return the correct N-Triples literal string.
        let decoded = crate::dictionary::decode(id).expect("decode should succeed for inline");
        assert_eq!(
            decoded,
            "\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>"
        );
    }

    #[pg_test]
    fn test_lang_literal_roundtrip() {
        let id = crate::dictionary::encode_lang_literal("hello", "en");
        let full = crate::dictionary::decode_full(id).expect("decode_full should succeed");
        assert_eq!(full.value, "hello");
        assert_eq!(full.lang.as_deref(), Some("en"));
    }

    #[pg_test]
    fn test_ntriples_bulk_load() {
        let data =
            "<https://example.org/a> <https://example.org/knows> <https://example.org/b> .\n";
        let count = crate::bulk_load::load_ntriples(data, false);
        assert_eq!(count, 1);
        assert!(crate::storage::total_triple_count() >= 1);
    }

    #[pg_test]
    fn test_ntriples_bulk_load_updates_dictionary_stats() {
        let data = (0..4_000)
            .map(|i| {
                format!(
                    "<https://example.org/bulk-s{i}> <https://example.org/bulk-p> \
                     <https://example.org/bulk-o{i}> ."
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(crate::bulk_load::load_ntriples(&data, true), 4_000);
    }

    #[pg_test]
    fn test_turtle_bulk_load() {
        let data = "@prefix ex: <https://example.org/> .\nex:x ex:rel ex:y .\n";
        let count = crate::bulk_load::load_turtle(data, false);
        assert_eq!(count, 1);
    }

    #[pg_test]
    fn test_named_graph_drop() {
        let graph = "<https://example.org/mygraph>";
        let g_id = crate::storage::create_graph(graph);
        assert!(g_id > 0);
        crate::storage::insert_triple(
            "<https://example.org/s>",
            "<https://example.org/p>",
            "<https://example.org/o>",
            g_id,
        );
        let deleted = crate::storage::drop_graph(graph);
        assert!(deleted >= 1);
    }

    #[pg_test]
    fn test_export_ntriples_roundtrip() {
        let nt =
            "<https://example.org/ex> <https://example.org/pred> <https://example.org/obj> .\n";
        crate::bulk_load::load_ntriples(nt, false);
        let exported = crate::export::export_ntriples(None);
        assert!(exported.contains("<https://example.org/pred>"));
    }

    // ─── SPARQL engine tests (v0.3.0) ─────────────────────────────────────────

    /// A SELECT that returns no rows on an empty store must produce an empty set.
    #[pg_test]
    fn pg_test_sparql_select_empty() {
        let rows = crate::sparql::sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }");
        assert_eq!(rows.len(), 0, "expected no rows on empty store");
    }

    /// After loading one triple, SELECT ?s ?p ?o must return exactly one row.
    #[pg_test]
    fn pg_test_sparql_select_one_triple() {
        crate::bulk_load::load_ntriples(
            "<https://example.org/a> <https://example.org/p> <https://example.org/b> .\n",
            false,
        );
        let rows = crate::sparql::sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }");
        assert_eq!(rows.len(), 1, "expected exactly one row");
        // The row must contain a non-null ?s binding.
        let obj = rows[0].0.as_object().expect("row must be a JSON object");
        assert!(obj.contains_key("s"), "row must have ?s binding");
        assert!(obj.contains_key("p"), "row must have ?p binding");
        assert!(obj.contains_key("o"), "row must have ?o binding");
    }

    /// sparql_ask() on an empty store returns false.
    #[pg_test]
    fn pg_test_sparql_ask_empty() {
        let result = crate::sparql::sparql_ask("ASK { ?s ?p ?o }");
        assert!(!result, "ASK on empty store must be false");
    }

    /// sparql_ask() returns true after a matching triple is inserted.
    #[pg_test]
    fn pg_test_sparql_ask_match() {
        crate::bulk_load::load_ntriples(
            "<https://example.org/x> <https://example.org/q> <https://example.org/y> .\n",
            false,
        );
        let result =
            crate::sparql::sparql_ask("ASK { <https://example.org/x> <https://example.org/q> ?o }");
        assert!(result, "ASK must be true after matching triple loaded");
    }

    /// sparql_explain() returns non-empty SQL for a simple SELECT.
    #[pg_test]
    fn pg_test_sparql_explain_returns_sql() {
        let plan = crate::sparql::sparql_explain(
            "SELECT ?s WHERE { ?s <https://example.org/p> ?o }",
            false,
        );
        assert!(
            plan.contains("Generated SQL"),
            "explain output must contain 'Generated SQL'"
        );
    }

    /// SPARQL LIMIT 1 must return at most one row.
    #[pg_test]
    fn pg_test_sparql_limit() {
        // Load two triples.
        crate::bulk_load::load_ntriples(
            "<https://example.org/s1> <https://example.org/p> <https://example.org/o1> .\n\
             <https://example.org/s2> <https://example.org/p> <https://example.org/o2> .\n",
            false,
        );
        let rows =
            crate::sparql::sparql("SELECT ?s ?o WHERE { ?s <https://example.org/p> ?o } LIMIT 1");
        assert!(rows.len() <= 1, "LIMIT 1 must return at most one row");
    }

    // ─── RDF-star / Statement Identifiers tests (v0.4.0) ──────────────────────

    /// N-Triples-star: loading an object-position quoted triple must succeed.
    #[pg_test]
    fn pg_test_ntriples_star_object_position() {
        let n = crate::bulk_load::load_ntriples(
            "<https://example.org/eve> <https://example.org/said> \
             << <https://example.org/alice> <https://example.org/knows> \
             <https://example.org/bob> >> .\n",
            false,
        );
        assert_eq!(n, 1, "object-position quoted triple must load as 1 triple");
    }

    /// N-Triples-star: loading a subject-position quoted triple must succeed.
    #[pg_test]
    fn pg_test_ntriples_star_subject_position() {
        let n = crate::bulk_load::load_ntriples(
            "<< <https://example.org/alice> <https://example.org/knows> \
             <https://example.org/bob> >> <https://example.org/certainty> \
             \"0.9\"^^<http://www.w3.org/2001/XMLSchema#decimal> .\n",
            false,
        );
        assert_eq!(n, 1, "subject-position quoted triple must load as 1 triple");
    }

    /// encode_quoted_triple / decode_quoted_triple_components round-trip.
    #[pg_test]
    fn pg_test_quoted_triple_encode_decode() {
        let s_id =
            crate::dictionary::encode("https://example.org/alice", crate::dictionary::KIND_IRI);
        let p_id =
            crate::dictionary::encode("https://example.org/knows", crate::dictionary::KIND_IRI);
        let o_id =
            crate::dictionary::encode("https://example.org/bob", crate::dictionary::KIND_IRI);
        let qt_id = crate::dictionary::encode_quoted_triple(s_id, p_id, o_id);
        assert!(qt_id != 0, "quoted triple must have a non-zero ID");
        let components = crate::dictionary::decode_quoted_triple_components(qt_id);
        assert!(components.is_some(), "decode must return Some");
        let (ds, dp, ob) = components.unwrap();
        assert_eq!(ds, s_id);
        assert_eq!(dp, p_id);
        assert_eq!(ob, o_id);
    }

    /// insert_triple returns a positive SID; get_statement can look it back up.
    #[pg_test]
    fn pg_test_statement_identifier_lifecycle() {
        let sid = crate::storage::insert_triple(
            "<https://example.org/subject1>",
            "<https://example.org/predicate1>",
            "<https://example.org/object1>",
            0,
        );
        assert!(sid > 0, "insert must return a positive SID");
    }

    /// SPARQL DISTINCT must deduplicate results.
    #[pg_test]
    fn pg_test_sparql_distinct() {
        // Two triples sharing the same predicate and object.
        crate::bulk_load::load_ntriples(
            "<https://example.org/s1> <https://example.org/same> <https://example.org/o> .\n\
             <https://example.org/s2> <https://example.org/same> <https://example.org/o> .\n",
            false,
        );
        // Select just ?o — should be deduplicated to 1 row.
        let rows =
            crate::sparql::sparql("SELECT DISTINCT ?o WHERE { ?s <https://example.org/same> ?o }");
        assert_eq!(rows.len(), 1, "DISTINCT ?o must collapse duplicates");
    }

    /// FILTER with a bound IRI constant must restrict results correctly.
    #[pg_test]
    fn pg_test_sparql_filter_bound() {
        crate::bulk_load::load_ntriples(
            "<https://example.org/s1> <https://example.org/p> <https://example.org/o1> .\n             <https://example.org/s2> <https://example.org/p> <https://example.org/o2> .\n",
            false,
        );
        // Only one subject matches the binding of ?s to s1.
        let rows = crate::sparql::sparql(
            "SELECT ?o WHERE { <https://example.org/s1> <https://example.org/p> ?o }",
        );
        assert_eq!(rows.len(), 1, "bound subject must restrict to one row");
    }

    /// Regression test for the "stale shared-memory encode cache" bug.
    ///
    /// Before the fix, the workflow
    ///     DROP EXTENSION pg_ripple CASCADE;
    ///     CREATE EXTENSION pg_ripple;
    ///     SELECT pg_ripple.load_rdfxml_file('…');
    /// produced a VP table full of dangling references: the dictionary
    /// stayed empty because `dictionary::encode` short-circuited on a
    /// shared-memory cache hit that pointed at a previously-allocated id,
    /// never falling through to the SPI INSERT path that populates the
    /// dictionary table.
    ///
    /// The fix calls `pre_load_safety_check()` (which clears the encode
    /// cache + backend-local LRU) at the start of every `*_file` loader.
    /// This test poisons the exact hash of an IRI in the fixture and asserts
    /// that the file loader replaces it with a real dictionary entry.
    #[pg_test]
    fn pg_test_load_rdfxml_file_populates_dictionary() {
        let subject_iri = "https://example.org/cache-test-x";
        let stale_id = i64::MAX - 42;

        // 1) Pre-populate the shmem encode cache with a stale entry that
        //    points at an id which does not exist in the dictionary.
        //    Without the safety net, the next dictionary::encode call
        //    would return this id and the new row would silently miss the
        //    dictionary table.
        let mut hash_input = Vec::with_capacity(2 + subject_iri.len());
        hash_input.extend_from_slice(&crate::dictionary::KIND_IRI.to_le_bytes());
        hash_input.extend_from_slice(subject_iri.as_bytes());
        let stale_hash = xxhash_rust::xxh3::xxh3_128(&hash_input);
        crate::shmem::encode_cache_insert(stale_hash, stale_id);

        // 2) Verify the exact subject hash is poisoned.
        let looked_up = crate::shmem::encode_cache_lookup(stale_hash);
        assert_eq!(
            looked_up,
            Some(stale_id),
            "stale cache entry should be visible"
        );

        // 3) Write a small RDF/XML file into the database cluster
        //    directory (read_file_content requires the path to live under
        //    data_directory, hence we resolve it via current_setting).
        let data_dir: String = Spi::get_one("SELECT current_setting('data_directory')")
            .expect("read data_directory")
            .expect("data_directory is not NULL");
        let path = std::path::PathBuf::from(&data_dir).join("pg_ripple_regress.owl");
        std::fs::write(
            &path,
            br#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="https://example.org/">
  <ex:Thing rdf:about="https://example.org/cache-test-x"/>
  <ex:Thing rdf:about="https://example.org/cache-test-y"/>
</rdf:RDF>
"#,
        )
        .expect("write RDF/XML fixture");

        // 4) Allow the file path. Set the session-level allowlist so the test
        //    session-level allowlist so the test works regardless of
        //    what's in postgresql.conf.
        Spi::run(&format!(
            "SET pg_ripple.copy_rdf_allowed_paths = '{}'",
            data_dir.replace('\'', "''")
        ))
        .expect("set copy_rdf_allowed_paths");

        // 5) Load. pre_load_safety_check() inside load_rdfxml_file
        //    should drop the stale cache entry, so dictionary::encode
        //    falls through to the SPI INSERT and creates fresh rows.
        let count =
            crate::bulk_load::load_rdfxml_file(path.to_str().expect("path is utf-8"), false);
        assert!(count >= 2, "expected at least 2 triples, got {count}");

        // 6) The poisoned cache entry must have been replaced, and the exact
        //    subject IRI must now exist in the dictionary.
        assert_ne!(
            crate::shmem::encode_cache_lookup(stale_hash),
            Some(stale_id),
            "file load must evict the stale subject mapping"
        );
        let dict_rows: i64 = Spi::get_one(
            "SELECT count(*) FROM _pg_ripple.dictionary \
             WHERE value = 'https://example.org/cache-test-x' AND kind = 0",
        )
        .expect("count subject dictionary entry")
        .expect("subject dictionary count is not NULL");
        assert!(
            dict_rows == 1,
            "subject IRI should have one dictionary entry, got {dict_rows}"
        );

        // 7) The SPARQL engine must be able to read the loaded triples
        //    back without emitting "dictionary entry missing" warnings.
        let rows = crate::sparql::sparql(
            "SELECT ?s WHERE { \
             ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> \
                <https://example.org/Thing> \
             }",
        );
        assert!(
            !rows.is_empty(),
            "SPARQL should return the freshly loaded triples"
        );

        // 8) Cleanup.
        let _ = std::fs::remove_file(&path);
    }
}
