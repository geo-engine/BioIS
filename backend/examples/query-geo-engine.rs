#![allow(clippy::unwrap_used, clippy::print_stderr)] // ok for example

use geoengine_api_client::{
    apis::{
        configuration::Configuration, general_api::server_info_handler, ogcwfs_api::wfs_handler,
        session_api::anonymous_handler, workflows_api::register_workflow_handler,
    },
    models::{
        Coordinate2D, LegacyTypedOperator, LegacyTypedOperatorOperator, SpatialPartition2D,
        WfsRequest, WfsService, Workflow, legacy_typed_operator::Type,
    },
};

#[tokio::main]
async fn main() {
    let mut configuration = Configuration::new();
    configuration.base_path = "http://localhost:3030/api".into();
    let server_info = server_info_handler(&configuration).await.unwrap();
    eprintln!("{server_info:?}");

    let session = anonymous_handler(&configuration).await.unwrap();
    eprintln!("{session:#?}");
    configuration.bearer_access_token = Some(session.id.to_string());

    let workflow = Workflow::LegacyTypedOperator(Box::new(LegacyTypedOperator {
        operator: Box::new(LegacyTypedOperatorOperator {
            params: Some(serde_json::json!({
                "data": "ne_10m_ports"
            })),
            sources: None,
            r#type: "OgrSource".into(),
        }),
        r#type: Type::Vector,
    }));

    let workflow_id = register_workflow_handler(&configuration, workflow)
        .await
        .unwrap();

    eprintln!("{workflow_id:?}");

    let workflow_id = workflow_id.id.to_string();

    let bbox_germany = SpatialPartition2D::new(
        Coordinate2D::new(15.016_995_883_9, 47.302_487_697_9),
        Coordinate2D::new(5.988_658_074_58, 54.983_104_153),
    );

    let feature_collection = wfs_handler(
        &configuration,
        &workflow_id,
        WfsRequest::GetFeature,
        Some(&bbox_germany.to_string()),
        None,
        None,
        None,
        None,
        None,
        Some(WfsService::Wfs),
        None,
        Some("EPSG:4326"),
        None,
        Some(&workflow_id),
        None,
    )
    .await
    .unwrap();

    eprintln!("{feature_collection:#?}");
}
