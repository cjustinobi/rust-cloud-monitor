
use axum::{
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    // Create the router with /metrics endpoint
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/", get(root_handler));

    // Define the address to bind to
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://localhost:3000");
    println!("📊 Metrics available at http://localhost:3000/metrics");

    // Start the server
    axum::serve(listener, app)
        .await
        .unwrap();
}

// Handler function for /metrics endpoint
async fn metrics_handler() -> & 'static str {
    "Hello Metrics"
}

async fn root_handler() -> &'static str {
    "Hello, World!"
}