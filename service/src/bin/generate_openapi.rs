use biois_service::ApiDoc;
use std::fs;
use utoipa::OpenApi;

fn main() {
    let openapi_json = ApiDoc::openapi().to_pretty_json().unwrap();
    fs::write("openapi.json", openapi_json).expect("Failed to write openapi.json");
    println!("OpenAPI spec written to openapi.json");
}
