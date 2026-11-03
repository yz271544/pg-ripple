-- Read-only checks for the ChatBI 20261027-v5 ontology bundle.
-- Run with: usql -X chatbiontology_v1 -f examples/chatbi_ontology_diagnostics.sql
-- Expected counts below were checked against the source files, not a live server.

-- Expected: 30173 when this graph contains only the seven bundle files.
SELECT * FROM pg_ripple.sparql($sparql$
SELECT (COUNT(*) AS ?triples) WHERE {
  GRAPH <https://arcp.sxmcc.com/tenant/chatbi> { ?s ?p ?o }
}
$sparql$);

-- Expected: eight ontology-defined measures and their Chinese names.
-- These are measure definitions, not rows loaded from the KPI code table.
SELECT * FROM pg_ripple.sparql($sparql$
SELECT DISTINCT ?kpi ?kpiName WHERE {
  GRAPH <https://arcp.sxmcc.com/tenant/chatbi> {
    ?kpi a <https://arcp.sxmcc.com/zb_data/chatbi/dw#Measure> .
    ?kpi <http://www.w3.org/2000/01/rdf-schema#label> ?kpiName .
  }
}
ORDER BY ?kpi
$sparql$);

-- Expected: zero. The bundle defines KpiCode and KpiName, but contains
-- neither KPI code table instances nor values of the KpiName property.
SELECT * FROM pg_ripple.sparql($sparql$
SELECT (COUNT(*) AS ?instances) WHERE {
  GRAPH <https://arcp.sxmcc.com/tenant/chatbi> {
    ?kpi a <https://arcp.sxmcc.com/zb_data/chatbi/metric#KpiCode> .
  }
}
$sparql$);

SELECT * FROM pg_ripple.sparql($sparql$
SELECT (COUNT(*) AS ?nameValues) WHERE {
  GRAPH <https://arcp.sxmcc.com/tenant/chatbi> {
    ?kpi <https://arcp.sxmcc.com/zb_data/chatbi/metric#KpiName> ?name .
  }
}
$sparql$);
