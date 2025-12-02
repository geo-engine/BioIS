use anyhow::{Context, Result};

/// Converts a Geo Engine operator to an Geo Engine OpenAPI workflow.
pub fn to_api_workflow(
    operator: &geoengine_operators::engine::TypedOperator,
) -> Result<geoengine_openapi_client::models::Workflow> {
    use geoengine_openapi_client::models::{
        TypedOperatorOperator, Workflow, workflow::Type as WorkflowType,
    };
    use geoengine_operators::engine::TypedOperator;

    let serde_json::Value::Object(mut json_object) = serde_json::to_value(operator)? else {
        anyhow::bail!("expected operator to serialize `TypedOperator` to a JSON object");
    };
    let serde_json::Value::Object(mut json_object) = json_object
        .remove("operator")
        .context("missing `operator` field")?
    else {
        anyhow::bail!("`operator` field is not a JSON object");
    };
    let serde_json::Value::String(r#type) =
        json_object.remove("type").context("missing `type` field")?
    else {
        anyhow::bail!("`type` field is not a string");
    };

    Ok(Workflow {
        operator: Box::new(TypedOperatorOperator {
            params: json_object.remove("params"),
            sources: json_object.remove("sources"),
            r#type,
        }),
        r#type: match operator {
            TypedOperator::Vector(..) => WorkflowType::Vector,
            TypedOperator::Raster(..) => WorkflowType::Raster,
            TypedOperator::Plot(..) => WorkflowType::Plot,
        },
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_converts_operator_to_api_workflow() {
        use geoengine_datatypes::{dataset::NamedData, raster::RasterDataType};
        use geoengine_operators::engine::{RasterOperator, TypedOperator};
        use geoengine_operators::processing::{RasterTypeConversion, RasterTypeConversionParams};
        use geoengine_operators::source::{GdalSource, GdalSourceParameters};

        let gdal_source = TypedOperator::Raster(
            RasterTypeConversion {
                params: RasterTypeConversionParams {
                    output_data_type: RasterDataType::F32,
                },
                sources: GdalSource {
                    params: GdalSourceParameters {
                        data: NamedData::with_system_name("ndvi"),
                    },
                }
                .boxed()
                .into(),
            }
            .boxed(),
        );

        let api_workflow = to_api_workflow(&gdal_source).unwrap();

        assert_eq!(
            serde_json::to_value(api_workflow).unwrap(),
            serde_json::json!({
                "type": "Raster",
                "operator": {
                    "type": "RasterTypeConversion",
                    "params": {
                        "outputDataType": "F32"
                    },
                    "sources": {
                        "raster": {
                            "type": "GdalSource",
                            "params": {
                                "data": "ndvi"
                            }
                        }
                    }
                }
            })
        );
    }
}
