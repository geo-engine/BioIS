# Mock Data for Early Stage Processes

## Natura 2000

Download gpkg from the [European Environment Agency](https://sdi.eea.europa.eu/datashare/s/mwzs9eNsJ9Sn4Q4/download?path=%2F&files=Natura2000_end2024.gpkg).

Import to PostGIS with:

```bash
psql -U geoengine -d biois -h localhost -p 5432 -c "CREATE SCHEMA IF NOT EXISTS \"Natura2000\";"
ogr2ogr \
    -f "PostgreSQL" \
    PG:"dbname=biois user=geoengine password=geoengine host=localhost port=5432" \
    -lco SCHEMA=Natura2000 \
    Natura2000_end2024.gpkg
```
