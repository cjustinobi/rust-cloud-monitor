mod config;
mod monitor;
mod routes;

use routes::targets::{get_targets, add_target, remove_target};

use crate::config::Config;
use crate::monitor::Monitor;
use tokio::net::TcpListener;
use axum::{routing::{get, delete}, Router};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;

    // Create monitor with targets from config
    let monitor = Arc::new(Monitor::new(config.targets.clone()));

    let m = monitor.clone();
    tokio::spawn(async move {
        m.run().await;
    });

    let app = Router::new()
        .route("/metrics", get({
            let monitor = monitor.clone();
            move || async move { monitor.gather_metrics() }
        }))
        .route("/targets", get(get_targets).post(add_target))
        .route("/targets/{name}", delete(remove_target))
        .with_state(monitor.clone());

    let listener = TcpListener::bind(&config.addr).await?;
    println!("🚀 Metrics server running on http://{}", config.addr);

    axum::serve(listener, app).await?;
    Ok(())
}