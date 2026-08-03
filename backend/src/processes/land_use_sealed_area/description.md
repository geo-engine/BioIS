# Land-Use Calculation for Sealed Areas

[VSME](https://www.efrag.org/sites/default/files/sites/webpublishing/SiteAssets/VSME%20Standard.pdf) B5 Paragraphs 138-141 provide guidance on calculating and reporting land-use according to the VSME standard.

## Definitions

### Sealed Area

A 'sealed area' is an area where the original soil has been covered (e.g. roads, buildings, parking lots), making it impermeable and resulting in an impact on the environment.

### Nature-Oriented Area

A 'nature-oriented area' is an area that primarily preserves or restores nature. Near-natural/green areas may be located on the organization's site and may include roofs, facades, water-drainage systems or other features designed, adapted or managed to promote biodiversity.

Near-natural areas may also be located off the organization's site if they are owned or managed by the organization and primarily serve to promote biodiversity.

## Guidance on Calculation

When disclosing land-use information, undertakings shall not only consider local impacts but also direct and indirect impacts on biodiversity (e.g. through raw material extraction, procurement, supply chain, production and products, transportation and logistics, and marketing and communications).

## Inputs

### Sites Data

A collection of sites with geographical information and land-use classification.
The collection should be provided as a [GeoJSON](https://geojson.org/) `FeatureCollection`, where each feature represents a site or land-use area with the following properties:

- `geometry`: The geographical coordinates of the area, which need to be a `Polygon` or `MultiPolygon`. Should use the `EPSG:4326` (WGS 84, latitude/longitude) coordinate reference system.
- `properties`: A JSON object containing fields for location identification and land-use type classification.

### Land-Use Type Field

Reference to the property in the input `GeoJSON` features that identifies the land-use type:

- `site`: Area is a site of the organization
- `natureOnSite`: Nature-oriented areas located on the organization's site
- `natureOffSite`: Nature-oriented areas owned or managed by the organization but located off-site

### Unit for Area

Unit for area measurement, with options for hectares (ha) or square meters (m²).

### Previous Year's Land-Use Data (Optional)

Optional input containing land-use values from the previous reporting period for comparison purposes. When provided, this data is used to populate the "Previous year" column in the output table and to calculate the percentage change. The data should include values for each land-use category:

- Total sealed area
- Total nature-oriented area on-site
- Total nature-oriented area off-site
- Total use of land

If not provided, the "Previous year" and "% change" columns in the output table will remain empty or indicate that data is unavailable.

## Outputs

### Land-Use Summary Table

Table representation of land-use information with the following structure:

| Land-use type                       | Previous year           | Reporting year                            | % change                     |
| ----------------------------------- | ----------------------- | ----------------------------------------- | ---------------------------- |
| Total sealed area                   | Value from prior period | Sum of all sealed site areas              | Calculated percentage change |
| Total nature-oriented area on-site  | Value from prior period | Sum of all nature-oriented areas on-site  | Calculated percentage change |
| Total nature-oriented area off-site | Value from prior period | Sum of all nature-oriented areas off-site | Calculated percentage change |
| Total use of land                   | Value from prior period | Sum of all land-use areas                 | Calculated percentage change |

### Documentation Sources

List of data sources and workflow references used for audits, including:

- [EMAS Guidance EU Commission Regulation 2018/2026](https://eur-lex.europa.eu/legal-content/EN/TXT/PDF/?uri=CELEX:32018R2026&rid=2)
- [User's guide](https://green-business.ec.europa.eu/document/download/98357f3d-f891-416e-81ea-a26f3ff3c61f_en?filename=PDF%20version%20C_2023_7207EN_annexe_acte_autonome_cp_part1_0.pdf)

### Processing Errors

List of errors encountered during processing, if any.

### Input Parameters

Echo of inputs for auditing.
