use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = biois_service::create_app();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Starting BioIS service on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
