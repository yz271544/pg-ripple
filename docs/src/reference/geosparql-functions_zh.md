# GeoSPARQL 函数

> **SC13-03 (v0.86.0)**：此页面记录 pg_ripple 中所有 GeoSPARQL 1.1 函数的实现状态。

pg_ripple 通过 PostGIS 集成层实现 GeoSPARQL 1.1 扩展（OGC，2022）。几何字面值存储为具有 `geo:wktLiteral` 数据类型的 WKT 字符串。

---

## 拓扑关系

GeoSPARQL 拓扑关系函数对几何字面值进行操作并返回 `xsd:boolean`。

| 函数 | 前缀 | 状态 | 注释 |
|---|---|---|---|
| `geo:sfEquals` | geo: | ✅ 已实现 | PostGIS `ST_Equals` |
| `geo:sfDisjoint` | geo: | ✅ 已实现 | PostGIS `ST_Disjoint` |
| `geo:sfIntersects` | geo: | ✅ 已实现 | PostGIS `ST_Intersects` |
| `geo:sfTouches` | geo: | ✅ 已实现 | PostGIS `ST_Touches` |
| `geo:sfCrosses` | geo: | ✅ 已实现 | PostGIS `ST_Crosses` |
| `geo:sfWithin` | geo: | ✅ 已实现 | PostGIS `ST_Within` |
| `geo:sfContains` | geo: | ✅ 已实现 | PostGIS `ST_Contains` |
| `geo:sfOverlaps` | geo: | ✅ 已实现 | PostGIS `ST_Overlaps` |
| `geo:ehEquals` | geo: | ✅ 已实现 | Egenhofer equals |
| `geo:ehDisjoint` | geo: | ✅ 已实现 | Egenhofer disjoint |
| `geo:ehMeet` | geo: | ✅ 已实现 | Egenhofer meet |
| `geo:ehOverlap` | geo: | ✅ 已实现 | Egenhofer overlap |
| `geo:ehCovers` | geo: | ✅ 已实现 | Egenhofer covers |
| `geo:ehCoveredBy` | geo: | ✅ 已实现 | Egenhofer covered by |
| `geo:ehInside` | geo: | ✅ 已实现 | Egenhofer inside |
| `geo:ehContains` | geo: | ✅ 已实现 | Egenhofer contains |
| `geo:rcc8dc` | geo: | ✅ 已实现 | RCC8 DC |
| `geo:rcc8ec` | geo: | ✅ 已实现 | RCC8 EC |
| `geo:rcc8po` | geo: | ✅ 已实现 | RCC8 PO |
| `geo:rcc8tppi` | geo: | ✅ 已实现 | RCC8 TPPi |
| `geo:rcc8tpp` | geo: | ✅ 已实现 | RCC8 TPP |
| `geo:rcc8ntppi` | geo: | ✅ 已实现 | RCC8 NTPPi |
| `geo:rcc8ntpp` | geo: | ✅ 已实现 | RCC8 NTPP |
| `geo:rcc8eq` | geo: | ✅ 已实现 | RCC8 EQ |

---

## 非拓扑函数

| 函数 | 状态 | 返回 | 注释 |
|---|---|---|---|
| `geo:distance` | ✅ 已实现 | `xsd:double` | PostGIS `ST_Distance`；单位 = 度（EPSG:4326） |
| `geo:buffer` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_Buffer` |
| `geo:convexHull` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_ConvexHull` |
| `geo:intersection` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_Intersection` |
| `geo:union` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_Union` |
| `geo:difference` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_Difference` |
| `geo:symDifference` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_SymDifference` |
| `geo:envelope` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_Envelope` |
| `geo:boundary` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_Boundary` |
| `geo:area` | ✅ 已实现 | `xsd:double` | PostGIS `ST_Area`；单位 = 平方度 |
| `geo:length` | ✅ 已实现 | `xsd:double` | PostGIS `ST_Length` |
| `geo:asWKT` | ✅ 已实现 | `geo:wktLiteral` | PostGIS `ST_AsText` |
| `geo:asGeoJSON` | ✅ 已实现 | `xsd:string` | PostGIS `ST_AsGeoJSON` |

---

## 已知差距

| 函数 | 状态 | 原因 |
|---|---|---|
| `geo:metricArea` | ⏳ 计划中 | 需要通过 `GEOGRAPHY` 类型进行 CRS 感知单位转换 |
| `geo:metricLength` | ⏳ 计划中 | 需要 CRS 感知单位转换 |
| `geo:metricPerimeter` | ⏳ 计划中 | 需要 CRS 感知单位转换 |
| `geo:metricDistance` | ⏳ 计划中 | 需要 CRS 感知单位转换 |
| DGGS 函数 | 🔜 未来 | 离散全球网格系统（DGGS）尚未实现 |

---

## 启用 GeoSPARQL

GeoSPARQL 需要 PostGIS 扩展：

```sql
CREATE EXTENSION postgis;
CREATE EXTENSION pg_ripple CASCADE;
```

请参阅 [GeoSPARQL + PostGIS 集成](https://github.com/trickle-labs/pg-ripple/blob/main/blog/geosparql-postgis-spatial.md)了解完整教程。