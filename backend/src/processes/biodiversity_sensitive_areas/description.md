# Sites in or near Biodiversity-Sensitive Areas

[VSME](https://www.efrag.org/sites/default/files/sites/webpublishing/SiteAssets/VSME%20Standard.pdf) B5 Paragraph 33 states:
"The undertaking shall disclose the number and area (in hectares) of sites that it owns, has leased,
or manages in or near a biodiversity sensitive area."

This process identifies sites in or near biodiversity-sensitive areas and calculates the area of these sites that overlaps with the biodiversity-sensitive areas.

## Inputs

### Sites

A collection of sites with geo coordinates, location names, and site types.
The collection should be provided as a [GeoJSON](https://geojson.org/) `FeatureCollection`, where each feature represents a site with the following properties:

- `geometry`: The geographical coordinates of the site, which can be a `Point` or `Polygon`. Should use the `EPSG:4326` (WGS 84, latitude/longitude) coordinate reference system.
- `properties`: A JSON object containing the fields for location name and site type, as specified by the `Location Name Field` and `Site Type Field` parameters.

### Location Name Field

Reference to the property in the input `GeoJSON` features that contains the location information.
This is used to identify the site by a name or ID, which is included in the output for auditing and traceability purposes.

### Site Type Field

Reference to the property in the input `GeoJSON` features that contains the site type information.

This property is used to determine the buffer distance for identifying biodiversity sensitive areas around each site, based on the specified site type.
Possible types are:

- `office`: Office buildings, with a buffer distance of 5 km.
- `agriculture`: Agricultural fields, with a buffer distance of 10 km.
- `marine`: Marine sites, with a buffer distance of 20 km.
- `mining`: Mining sites, with a buffer distance of 50 km.
- `other`: Other types of sites, with a buffer distance of 20 km.

### Unit for Area

Unit for area measurement, with options for hectares (ha) or square meters (m²).

#### Reasoning

##### Offices, Warehouses, Low-input agriculture

Minimum recommended size; sealed surfaces; assumes lower freshwater pollution.

##### High-input agriculture, Onshore wind, Oil & Gas (terrestrial)

Covers most pressures; extended fresh-water pollution; addresses eutrophication & runoff.

##### Offshore wind, Oil & Gas (marine), Hydropower

Higher marine influence (noise) & wide-ranging species.

##### Mining

Observed deforestation effects reach significant distances.

##### References

- UNEP-WCMC (2021/22): The Area of Influence of site-based operations (Direct & Indirect)

- Amec Foster Wheeler (2015): Habitats Regulations Assessment: 14th Onshore Oil and Gas Licensing Round.
- Poore & Nemecek (2018): Reducing food’s environmental impacts. Science, 360\_.

- UNEP-WCMC (2021): The Area of Influence of site-based operations (Direct & Indirect).
- Weaver J (2020): Wales National Development Framework - Habitats Regulations Assessment.

- Sonter et al. (2017): Mining drives extensive deforestation in the Brazilian Amazon. _Nature Communications_.
- Maddox et al. (2019): Forest-Smart Mining (World Bank).

## Outputs

### Biodiversity-sensitive Areas

Table representation of the identified biodiversity-sensitive areas with the following columns:

| Location                                          | Area (ha/m²)                                                       | Site Is In Biodiversity-sensitive Areas                                     | Site Is Near Biodiversity-sensitive Areas                          | Biodiversity-sensitive Area (ha/m²)                                                                                                                                   | Specification                                                                                                                                    |
| ------------------------------------------------- | ------------------------------------------------------------------ | --------------------------------------------------------------------------- | ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Name or ID of the site, as provided in the input. | Total area of the site in hectares or square meters, if available. | True/False: Does the site itself overlap with biodiversity-sensitive areas? | True/False: Is the site located near biodiversity-sensitive areas? | Area of the site that overlaps with biodiversity-sensitive areas, calculated based on the buffering and spatial intersection with known biodiversity-sensitive areas. | Description of the biodiversity-sensitive area, including the type of biodiversity present and any relevant conservation status or designations. |

### Documentation Sources

List of data sources and workflow references used for audits.

### Processing Errors

List of errors encountered during processing, if any.

### Input Parameters

Echo of inputs for auditing.
