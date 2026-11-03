//! Validation of generated queries, independent of PostgreSQL and the LLM.

use spargebra::{SparqlParser, SparqlSyntaxError};

pub(super) fn normalize_generated_query(query: &str) -> Result<String, SparqlSyntaxError> {
    if SparqlParser::new().parse_query(query).is_ok() {
        return Ok(query.to_owned());
    }

    // Defaults apply only to generated text. Explicit declarations in the
    // query override parser defaults; unknown domain prefixes still fail.
    let mut parser = SparqlParser::new();
    for (prefix, iri) in [
        ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
        ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
        ("owl", "http://www.w3.org/2002/07/owl#"),
        ("xsd", "http://www.w3.org/2001/XMLSchema#"),
        ("skos", "http://www.w3.org/2004/02/skos/core#"),
        ("dcterms", "http://purl.org/dc/terms/"),
        ("schema", "http://schema.org/"),
        ("foaf", "http://xmlns.com/foaf/0.1/"),
    ] {
        #[allow(clippy::expect_used)] // CLIPPY-OK: static, valid vocabulary IRIs
        let next = parser
            .with_prefix(prefix, iri)
            .expect("valid vocabulary IRI");
        parser = next;
    }

    // Materialize resolved IRIs so the returned text can be parsed and executed
    // by the normal strict parser without these defaults.
    parser.parse_query(query).map(|parsed| parsed.to_string())
}
