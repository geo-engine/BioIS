WITH reference AS (
    SELECT
        'Test' AS location,
        ST_Transform(
            ST_GeomFromText('SRID=4326; POINT(8.770273718309227 50.80746831824467)'),
            3035
        ) AS geom,
        ST_Buffer(
            ST_Transform(
                ST_GeomFromText('SRID=4326; POINT(8.770273718309227 50.80746831824467)'),
                3035
            ),
            20 * 1_000
        ) AS buffered_geom,
        TRUE AS was_point,
        20 AS buffer_size_km,
        'Test specification' AS specification,
        ST_Area(
            ST_Transform(
                ST_GeomFromText('SRID=4326; POINT(8.770273718309227 50.80746831824467)'),
                3035
            )
        ) AS area_m2
)
SELECT
    r.location,
    NULLIF(r.area_m2, 0) AS area_m2,
    COALESCE(ST_AREA(ST_INTERSECTION(s_in.geom, r.geom)), 0) > 0 AS site_in_biodiversity_sensitive_area,
    ST_AREA(ST_INTERSECTION(s_out.geom, r.buffered_geom)) > 0 AS site_near_biodiversity_sensitive_area,
    ST_AREA(ST_INTERSECTION(s_out.geom, r.buffered_geom)) AS biodiversity_sensitive_area_m2,
    'Type: ' || r.specification || E'\n'
    || CASE WHEN r.was_point THEN '(derived from point)' ELSE '' END || E'\n'
    || COALESCE('Intersection with:' || E'\n' || s_in.site_list, '')
    || 'Intersection with ' || r.buffer_size_km || ' km buffer zone to:' || E'\n' 
    || s_out.site_list AS specification
FROM reference r
LEFT JOIN (
    SELECT
        r.location,
        ST_COLLECT(s.geom) as geom,
        STRING_AGG(s.sitename || ' (' || s.sitecode || ')', E'\n') as site_list
    FROM "Natura2000".naturasite_polygon s, reference r
    WHERE ST_Intersects(s.geom, r.geom)
    GROUP BY r.location
) as s_in USING (location)
JOIN (
    SELECT
        r.location,
        ST_COLLECT(s.geom) as geom,
        STRING_AGG(s.sitename || ' (' || s.sitecode || ')', E'\n') as site_list
    FROM "Natura2000".naturasite_polygon s, reference r
    WHERE ST_Intersects(s.geom, r.buffered_geom)
    GROUP BY r.location
) as s_out USING (location)
ORDER BY biodiversity_sensitive_area_m2 DESC;
