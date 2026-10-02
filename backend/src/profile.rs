//! The `schema.biois` display extension of `BioIS` data resources, as a published JSON Schema.
//!
//! Climate-risk result tables carry a `schema.biois` extension that declares how a column is
//! rendered (a label and a color, e.g. a risk probability class or a signed anomaly). The
//! extension is defined here so that external consumers of a data package can validate it; the
//! `BioIS` frontend instead reads the extension inline from each resource's own schema.

use axum::{
    http::{HeaderValue, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};

/// The path of the climate-risk table schema profile, cf. [`CLIMATE_RISK_TABLE_SCHEMA`].
pub const CLIMATE_RISK_TABLE_SCHEMA_PROFILE: &str =
    "/profiles/table-schema/climate-risk/1.0/schema.json";

const CLIMATE_RISK_TABLE_SCHEMA: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BioIS climate-risk Table Schema extension",
  "allOf": [
    { "$ref": "https://datapackage.org/profiles/2.0/tableschema.json" },
    {
      "type": "object",
      "properties": {
        "biois": {
          "type": "object",
          "required": ["display"],
          "properties": {
            "display": {
              "type": "object",
              "additionalProperties": {
                "$ref": "#/$defs/display"
              }
            },
            "hiddenFields": {
              "type": "array",
              "items": { "type": "string" },
              "uniqueItems": true
            }
          },
          "additionalProperties": false
        }
      }
    }
  ],
  "$defs": {
    "display": {
      "type": "object",
      "required": ["kind"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": ["riskProbability", "riskAnomaly"]
        },
        "labelField": {
          "type": "string",
          "description": "Name of a row property carrying the complete display label for this field."
        },
        "colorField": {
          "type": "string",
          "description": "Name of a row property carrying the CSS color for this field."
        }
      },
      "additionalProperties": false
    }
  }
}"##;

/// The profile of the `schema.biois` extension: a Table Schema extension that declares per-column
/// display metadata for `BioIS` result tables.
#[utoipa::path(
    get,
    path = CLIMATE_RISK_TABLE_SCHEMA_PROFILE,
    tag = "Profiles",
    responses(
        (
            status = OK,
            description = "The BioIS table schema profile.",
            content_type = "application/schema+json",
            body = String
        )
    )
)]
pub async fn climate_risk_table_schema_profile() -> Response {
    (
        [(
            CONTENT_TYPE,
            HeaderValue::from_static("application/schema+json"),
        )],
        CLIMATE_RISK_TABLE_SCHEMA,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn it_serves_the_climate_risk_profile() {
        let response = climate_risk_table_schema_profile().await;
        assert_eq!(response.headers()[CONTENT_TYPE], "application/schema+json");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            profile["title"],
            "BioIS climate-risk Table Schema extension"
        );
        let display = &profile["$defs"]["display"];
        assert_eq!(display["properties"]["kind"]["enum"][0], "riskProbability");
        assert_eq!(display["properties"]["kind"]["enum"][1], "riskAnomaly");
        assert_eq!(display["properties"]["labelField"]["type"], "string");
        assert_eq!(display["properties"]["colorField"]["type"], "string");
        assert_eq!(
            profile["allOf"][1]["properties"]["biois"]["properties"]["hiddenFields"]["type"],
            "array"
        );
        assert_eq!(display["additionalProperties"], false);
    }
}
