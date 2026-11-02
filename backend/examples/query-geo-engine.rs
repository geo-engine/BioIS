#![allow(clippy::unwrap_used, clippy::print_stderr)] // ok for example

use geoengine_api_client::{
    apis::{
        configuration::Configuration, general_api::server_info_handler, ogcwfs_api::wfs_handler,
        processing_graphs_api::register_processing_graph_handler, session_api::anonymous_handler,
    },
    models::{
        Coordinate2D, OgrSource, OgrSourceParameters, ProcessingGraph, SpatialPartition2D,
        TypedVectorOperator, VectorOperator, WfsRequest, WfsService, typed_vector_operator::Type,
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

    let processing_graph = ProcessingGraph::TypedVectorOperator(Box::new(TypedVectorOperator {
        operator: Box::new(VectorOperator::OgrSource(Box::new(OgrSource::new(
            Default::default(),
            OgrSourceParameters::new("ne_10m_ports".into()),
        )))),
        r#type: Type::Vector,
    }));

    let processing_graph_id = register_processing_graph_handler(&configuration, processing_graph)
        .await
        .unwrap();

    eprintln!("{processing_graph_id:?}");

    let processing_graph_id = processing_graph_id.id.to_string();

    let bbox_germany = SpatialPartition2D::new(
        Coordinate2D::new(15.016_995_883_9, 47.302_487_697_9),
        Coordinate2D::new(5.988_658_074_58, 54.983_104_153),
    );

    let mut computation_id = String::new();

    let feature_collection = wfs_handler(
        &configuration,
        &processing_graph_id,
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
        Some(&processing_graph_id),
        None,
        Some(&mut computation_id),
    )
    .await
    .unwrap();

    eprintln!("Computation ID: {computation_id}");
    eprintln!("{feature_collection:#?}");
}
