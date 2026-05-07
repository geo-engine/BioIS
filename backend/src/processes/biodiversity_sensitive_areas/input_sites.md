# Sites

A collection of sites with geo coordinates and site types.

Coordinates should be provided in GeoJSON format with EPSG:4326 coordinate reference system.
Features can be of type Point or Polygon.
If a Point geometry is provided, it will be converted to a minimal polygon for buffering.

Sites types can be _office_, _agriculture_, _marine_, _mining_ and _other_.
The process will apply different impact buffer zones based on the site type (e.g. 5 km for office buildings, 10 km for agricultural fields, etc.).
