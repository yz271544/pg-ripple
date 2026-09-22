# GeoSPARQL 参考

pg_ripple v0.25.0 使用 PostGIS 作为底层几何引擎实现 [GeoSPARQL 1.1](https://www.ogc.org/standards/geosparql) 的一个子集。当未安装 PostGIS 时，所有地理函数都会优雅降级（返回 `false` 或 `NULL`）。

## 要求

| 要求 | 版本 |
|---|---|
| PostGIS | 2.5+（推荐 3.x） |
| PostGIS SQL 扩展 | 必须安装在同一个数据库中 |

检查可用性：

```sql
SELECT EXISTS(SELECT 1 FROM pg_proc WHERE proname = 'st_geomfromtext') AS postgis_available;
```

## 几何表示

几何以 WKT（Well-Known Text）字面值的形式存储在 RDF 三元组存储中。使用 N-Triples 或 Turtle 语法加载它们：

```sql
SELECT pg_ripple.load_ntriples(
    '<https://geo.example/city/berlin> <https://www.w3.org/ns/locn#geometry>
     "POINT(13.404954 52.520008)" .'
);
```

`geo:wktLiteral` 数据类型 IRI 存储为常规字面值；WKT 字符串在查询时直接传递给 `ST_GeomFromText()`。

## 拓扑关系函数（FILTER 上下文）

这些函数用于 `FILTER(...)` 子句中并返回 `true` 或 `false`。

| SPARQL IRI | PostGIS 等效项 | 描述 |
|---|---|---|
| `geo:sfIntersects(a, b)` | `ST_Intersects(…)` | 几何至少共享一个点 |
| `geo:sfContains(a, b)` | `ST_Contains(…)` | A 完全包含 B |
| `geo:sfWithin(a, b)` | `ST_Within(…)` | A 完全在 B 内 |
| `geo:sfOverlaps(a, b)` | `ST_Overlaps(…)` | 几何重叠 |
| `geo:sfTouches(a, b)` | `ST_Touches(…)` | 几何在边界处接触 |
| `geo:sfCrosses(a, b)` | `ST_Crosses(…)` | 几何交叉 |
| `geo:sfDisjoint(a, b)` | `ST_Disjoint(…)` | 几何不共享任何点 |
| `geo:sfEquals(a, b)` | `ST_Equals(…)` | 几何在空间上相等 |
| `geo:ehIntersects(a, b)` | `ST_Intersects(…)` | Egenhofer 相交 |
| `geo:ehContains(a, b)` | `ST_Contains(…)` | Egenhofer 包含 |
| `geo:ehCoveredBy(a, b)` | `ST_CoveredBy(…)` | A 被 B 覆盖 |
| `geo:ehCovers(a, b)` | `ST_Covers(…)` | A 覆盖 B |

命名空间前缀：`http://www.opengis.net/def/function/geosparql/`

### 示例：查找相交的几何

```sparql
PREFIX geo: <http://www.opengis.net/def/function/geosparql/>
SELECT ?city WHERE {
  ?city <https://www.w3.org/ns/locn#geometry> ?geom .
  FILTER(geo:sfIntersects(?geom, "POLYGON((13.0 52.0, 14.0 52.0, 14.0 53.0, 13.0 53.0, 13.0 52.0))"))
}
```

## 测量函数（BIND 上下文）

这些函数用于 `BIND(...)` 子句中并返回数值或 WKT 值。

| SPARQL IRI | 返回 | 描述 |
|---|---|---|
| `geof:distance(a, b, unit)` | `xsd:double`（米） | 两个几何之间的测地距离 |
| `geof:area(a, unit)` | `xsd:double`（平方米） | 几何的表面积 |
| `geof:boundary(a)` | WKT 字面值 | 边界几何作为 WKT 字符串 |

`unit` 参数被接受以实现 API 兼容性，但所有结果都以 SI 基本单位（米/平方米）返回。

### 示例：距离查询

```sparql
PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
SELECT ?city ?dist WHERE {
  ?city <https://www.w3.org/ns/locn#geometry> ?geom .
  BIND(geof:distance(?geom, "POINT(13.404954 52.520008)",
                     <http://www.opengis.net/def/uom/OGC/1.0/metre>) AS ?dist)
  FILTER(?dist < 100000)
}
ORDER BY ?dist
```

## PostGIS 不存在时的行为

当未安装 PostGIS 时：

- 拓扑过滤函数求值为 `false` — 查询返回零行
- 测量函数求值为 `NULL` — `BIND` 变量未绑定
- 不会引发 `ERROR`；查询正常完成

这允许将地理空间查询部署到 PostGIS 是可选组件的环境中。

## 限制

- 3D 几何（`POINT Z`、`POLYGON Z`）存储为字面值，但 PostGIS 2D 函数会将它们投影到 2D。
- 坐标系（CRS）处理：在传递给 `geof:distance` 和 `geof:area` 的 `geography` 转换时，假定所有几何都位于 WGS84（SRID 4326）中。
- RDF-star 引号三元组不支持作为几何参数。