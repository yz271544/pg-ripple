# 地理空间查询（GeoSPARQL）

**状态**：自 v0.36.0 起可用（GEO-01）  
**要求**：[PostGIS](https://postgis.net) 扩展（`CREATE EXTENSION postgis`）。如果没有 PostGIS，加载成功但 `geof:` 过滤器函数返回 `NULL`。  
**SQL**：`pg_ripple.sparql_select()` 与 `geof:` 和 `geo:` FILTER 函数  
**降级**：GeoSPARQL 过滤器函数在 PostGIS 不存在时静默返回 `NULL` — 在引入 WKT 数据之前启用 PostGIS。  

---

pg_ripple 为地理数据实现了 [GeoSPARQL 1.1](https://docs.ogc.org/is/22-047r1/22-047r1.html) 查询函数词汇表，将几何计算委托给 [PostGIS](https://postgis.net)。您将 WKT 字面量存储为三元组对象，SPARQL 过滤器函数如 `geof:sfWithin` 和 `geof:sfIntersects` 在查询时根据 PostGIS 解析它们 — 而无需任何额外的 schema 工作。

---

## 您获得的内容

| 功能 | 函数族 | 注释 |
|---|---|---|
| 拓扑过滤器 | `geof:sfWithin`、`geof:sfIntersects`、`geof:sfContains`、`geof:sfTouches` 等 | 简单要素 1.x |
| 距离 | `geof:distance` | 为地理 CRS 返回米 |
| 构造操作 | `geof:buffer`、`geof:convexHull`、`geof:envelope`、`geof:union`、`geof:intersection` | 返回几何字面量 |
| 访问器函数 | `geof:asWKT`、`geof:srid` | 检查 |

所有这些都编译为生成的 SQL 中的 PostGIS 函数调用。当您在相关的 VP 表上注册几何索引时，您会自动继承 PostGIS 的空间索引支持。

---

## 存储几何

几何体作为 Well-Known Text（WKT）字面量存在于您选择的几何谓词上。传统的谓词是 `locn:geometry`：

```sql
SELECT pg_ripple.load_turtle($TTL$
@prefix ex:   <https://example.org/> .
@prefix locn: <https://www.w3.org/ns/locn#> .

ex:berlin   locn:geometry "POINT(13.404954 52.520008)" .
ex:munich   locn:geometry "POINT(11.5820  48.1351)" .
ex:bavaria  locn:geometry "POLYGON((9.0 47.0, 13.8 47.0, 13.8 50.6, 9.0 50.6, 9.0 47.0))" .
$TTL$);
```

为了获得更好的结果，请在 `locn:geometry` 的 VP 表上创建 PostGIS 几何索引（一次性创建，每个谓词）：

```sql
SELECT pg_ripple.create_spatial_index('<https://www.w3.org/ns/locn#geometry>');
```

---

## 查询

### 查找多边形内的每个点

```sparql
PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
PREFIX locn: <https://www.w3.org/ns/locn#>

SELECT ?city WHERE {
    ?city locn:geometry ?g .
    FILTER(geof:sfWithin(?g,
        "POLYGON((9.0 47.0, 13.8 47.0, 13.8 50.6, 9.0 50.6, 9.0 47.0))"))
}
```

### 距离有界的最近邻

```sparql
SELECT ?city ?d WHERE {
    ?city locn:geometry ?g .
    BIND(geof:distance(?g, "POINT(11.5820 48.1351)") AS ?d)
    FILTER(?d < 200000)        # 在慕尼黑 200 公里以内
}
ORDER BY ?d
```

### 缓冲区 + 相交（构造）

```sparql
SELECT ?city WHERE {
    ?city locn:geometry ?g .
    FILTER(geof:sfIntersects(?g,
        geof:buffer("POINT(11.5820 48.1351)", 50000)))   # 慕尼黑周围 50 公里
}
```

---

## 坐标参考系统

GeoSPARQL 字面量可以将 CRS 作为 IRI 后缀：

```turtle
ex:berlin locn:geometry
    "<http://www.opengis.net/def/crs/EPSG/0/4326> POINT(13.4049 52.5200)" .
```

当 CRS 被省略时，pg_ripple 使用 WGS84（EPSG:4326）作为默认值。这与 GeoSPARQL 1.1 默认值匹配。

---

## **未**实现的内容

- Egenhofer（`geof:eh*`）和 RCC8（`geof:rcc8*`）拓扑函数尚未连接。
- 未解析 `gml:Geometry` 字面量数据类型（仅解析 `geo:wktLiteral`）。
- DE-9IM 矩阵查询未公开。

如果您需要其中任何一个，请提交问题 — 这意味着相关 PostGIS 函数之上的薄包装。

---

## 另请参阅

- [GeoSPARQL 函数目录](../reference/geosparql.md)
- [PostGIS 文档](https://postgis.net/docs/)

## 进一步阅读

- [博客：GeoSPARQL + PostGIS 空间查询](https://github.com/trickle-labs/pg-ripple/blob/main/blog/geosparql-postgis-spatial.md) — 组合地理和语义查询