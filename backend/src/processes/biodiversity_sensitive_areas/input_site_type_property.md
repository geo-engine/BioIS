# Site Type Property

Reference to the property in the input `GeoJSON` features that contains the site type information.

This property is used to determine the buffer distance for identifying biodiversity sensitive areas around each site, based on the specified site type.
Possible types are:

- `office`: Office buildings, with a buffer distance of 5 km.
- `agriculture`: Agricultural fields, with a buffer distance of 10 km.
- `marine`: Marine sites, with a buffer distance of 20 km.
- `mining`: Mining sites, with a buffer distance of 50 km.
- `other`: Other types of sites, with a buffer distance of 20 km.

## Reasoning

### Offices, Warehouses, Low-input agriculture

Minimum recommended size; sealed surfaces; assumes lower freshwater pollution.

#### References

- UNEP-WCMC (2021/22): The Area of Influence of site-based operations (Direct & Indirect)

### High-input agriculture, Onshore wind, Oil & Gas (terrestrial)

Covers most pressures; extended fresh-water pollution; addresses eutrophication & runoff.

#### References

_Amec Foster Wheeler (2015): Habitats Regulations Assessment: 14th Onshore Oil and Gas Licensing Round.
_ Poore & Nemecek (2018): Reducing food’s environmental impacts. _Science, 360_.

### Offshore wind, Oil & Gas (marine), Hydropower

Higher marine influence (noise) & wide-ranging species.

#### References

_UNEP-WCMC (2021): The Area of Influence of site-based operations (Direct & Indirect).
_ Weaver J (2020): Wales National Development Framework - Habitats Regulations Assessment.

### Mining

Observed deforestation effects reach significant distances.

#### References

- Sonter et al. (2017): Mining drives extensive deforestation in the Brazilian Amazon. _Nature Communications_.
- Maddox et al. (2019): Forest-Smart Mining (World Bank).
