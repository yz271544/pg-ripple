#[path = "../src/llm/generated_query.rs"]
mod generated_query;

use generated_query::normalize_generated_query;
use spargebra::SparqlParser;

const KPI_QUERY: &str = r#"SELECT ?kpiName WHERE {
  ?kpi a ?type .
  ?kpi skos:prefLabel ?kpiName .
  FILTER(CONTAINS(LCASE(STR(?type)), "kpi"))
}"#;

#[test]
fn omitted_skos_prefix_in_reported_kpi_query() {
    assert!(SparqlParser::new().parse_query(KPI_QUERY).is_err());
    let normalized = normalize_generated_query(KPI_QUERY).unwrap();
    let parsed = SparqlParser::new().parse_query(&normalized).unwrap();
    let expected = SparqlParser::new()
        .parse_query(&format!(
            "PREFIX skos: <http://www.w3.org/2004/02/skos/core#>\n{KPI_QUERY}"
        ))
        .unwrap();
    assert_eq!(parsed, expected);
}

#[test]
fn explicit_prefix_is_preserved() {
    let query =
        "PREFIX skos: <https://example.test/custom/> SELECT ?s WHERE { ?s skos:prefLabel ?name }";
    assert_eq!(normalize_generated_query(query).unwrap(), query);
}

#[test]
fn explicit_prefix_wins_when_another_prefix_is_missing() {
    let query =
        "PREFIX skos: <https://example.test/custom/> SELECT ?s WHERE { ?s rdf:type skos:Concept }";
    let normalized = normalize_generated_query(query).unwrap();
    assert_eq!(
        SparqlParser::new().parse_query(&normalized).unwrap(),
        SparqlParser::new()
            .parse_query(&format!(
                "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> {query}"
            ))
            .unwrap()
    );
}

#[test]
fn unknown_business_prefix_is_rejected() {
    assert!(normalize_generated_query("SELECT ?s WHERE { ?s business:kpi ?o }").is_err());
}

#[test]
fn malformed_query_is_rejected() {
    assert!(normalize_generated_query("SELECT ?s WHERE { ?s skos:prefLabel }").is_err());
}

#[test]
fn valid_query_text_is_preserved() {
    let query = "SELECT ?s WHERE { ?s <https://example.test/name> \"skos:prefLabel\" }";
    assert_eq!(normalize_generated_query(query).unwrap(), query);
}

#[test]
fn standard_vocabulary_defaults_are_materialized() {
    let query = "SELECT ?s WHERE { ?s rdf:type owl:Class ; rdfs:label ?label ; skos:prefLabel ?name ; dcterms:title ?title ; schema:name ?n ; foaf:name ?f . FILTER(?label = \"name\"^^xsd:string) }";
    let normalized = normalize_generated_query(query).unwrap();
    assert!(SparqlParser::new().parse_query(&normalized).is_ok());
}
